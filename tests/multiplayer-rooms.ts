import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { MultiplayerRooms } from "../target/types/multiplayer_rooms";
import {
  InitializeNewWorld,
  AddEntity,
  InitializeComponent,
  ApplySystem,
} from "@magicblock-labs/bolt-sdk";
import { expect } from "chai";
import * as bs58 from "bs58";

describe("Multiplayer Rooms", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Load the program
  const program = anchor.workspace
    .MultiplayerRooms as Program<MultiplayerRooms>;

  // Create test wallets
  const hostWallet = provider.wallet;
  const player2Wallet = Keypair.generate();

  // PDAs
  let worldPda: PublicKey;
  let roomPda: PublicKey;
  let hostEntityPda: PublicKey;
  let player2EntityPda: PublicKey;

  it("Airdrops SOL to player 2", async () => {
    // Airdrop 2 SOL to player 2
    const signature = await provider.connection.requestAirdrop(
      player2Wallet.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);
    console.log(
      `Airdropped 2 SOL to player 2: ${player2Wallet.publicKey.toString()}`
    );
  });

  it("Initialize the program", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Initialized program with transaction signature:", tx);
  });

  it("Initialize a new World", async () => {
    const initializeNewWorld = await InitializeNewWorld({
      payer: provider.wallet.publicKey,
      connection: provider.connection,
    });
    const signature = await provider.sendAndConfirm(
      initializeNewWorld.transaction
    );
    worldPda = initializeNewWorld.worldPda;
    console.log(
      `Initialized a new world (ID=${worldPda}). Signature: ${signature}`
    );
  });

  it("Create a room as host", async () => {
    // Calculate the room PDA
    [roomPda] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("room"), hostWallet.publicKey.toBuffer()],
      program.programId
    );

    // Create the room
    const tx = await program.methods
      .createRoom("Test Room", 4)
      .accounts({
        player: hostWallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Created room with transaction signature:", tx);

    // Verify room data
    const roomAccount = await program.account.room.fetch(roomPda);
    expect(roomAccount.name).to.equal("Test Room");
    expect(roomAccount.host.toString()).to.equal(
      hostWallet.publicKey.toString()
    );
    expect(roomAccount.status.lobby).to.not.be.undefined;
    expect(roomAccount.maxPlayers).to.equal(4);
    expect(roomAccount.playerCount).to.equal(1);

    // Check the host player data
    const hostPlayer = roomAccount.players[0];
    expect(hostPlayer.value.pubkey.toString()).to.equal(
      hostWallet.publicKey.toString()
    );
    expect(hostPlayer.value.name).to.equal("Player");
    expect(hostPlayer.value.isHost).to.be.true;
  });

  it("Add host to World as entity with component", async () => {
    // Create an entity for the host
    const addEntity = await AddEntity({
      payer: provider.wallet.publicKey,
      world: worldPda,
      connection: provider.connection,
    });
    const signature = await provider.sendAndConfirm(addEntity.transaction);
    hostEntityPda = addEntity.entityPda;
    console.log(
      `Added host entity (ID=${addEntity.entityId}). Signature: ${signature}`
    );

    // Initialize room component for the entity
    const initializeComponent = await InitializeComponent({
      payer: provider.wallet.publicKey,
      entity: hostEntityPda,
      componentId: program.programId,
    });
    const componentSignature = await provider.sendAndConfirm(
      initializeComponent.transaction
    );
    console.log(`Initialized room component. Signature: ${componentSignature}`);
  });

  it("Player 2 joins the room", async () => {
    // Create transaction to join room
    const tx = await program.methods
      .joinRoom("Player 2", "#ff0000")
      .accounts({
        player: player2Wallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player2Wallet])
      .rpc();

    console.log("Player 2 joined room with transaction signature:", tx);

    // Verify room data
    const roomAccount = await program.account.room.fetch(roomPda);
    expect(roomAccount.playerCount).to.equal(2);

    // Find player 2 in the array
    let foundPlayer2 = false;
    for (let i = 0; i < roomAccount.players.length; i++) {
      const player = roomAccount.players[i];
      if (
        player &&
        player.value.pubkey.toString() === player2Wallet.publicKey.toString()
      ) {
        foundPlayer2 = true;
        expect(player.value.name).to.equal("Player 2");
        expect(player.value.color).to.equal("#ff0000");
        expect(player.value.isHost).to.be.false;
      }
    }
    expect(foundPlayer2).to.be.true;
  });

  it("Add player 2 to World as entity with component", async () => {
    // Create an entity for player 2
    const addEntity = await AddEntity({
      payer: player2Wallet.publicKey,
      world: worldPda,
      connection: provider.connection,
    });
    const signature = await provider.connection.sendTransaction(
      addEntity.transaction,
      [player2Wallet]
    );
    await provider.connection.confirmTransaction(signature);
    player2EntityPda = addEntity.entityPda;
    console.log(
      `Added player 2 entity (ID=${addEntity.entityId}). Signature: ${signature}`
    );

    // Initialize room component for the entity
    const initializeComponent = await InitializeComponent({
      payer: player2Wallet.publicKey,
      entity: player2EntityPda,
      componentId: program.programId,
    });
    const componentSignature = await provider.connection.sendTransaction(
      initializeComponent.transaction,
      [player2Wallet]
    );
    await provider.connection.confirmTransaction(componentSignature);
    console.log(
      `Initialized room component for player 2. Signature: ${componentSignature}`
    );
  });

  it("Host updates their position", async () => {
    const newPosition = {
      x: new anchor.BN(100),
      y: new anchor.BN(50),
      z: new anchor.BN(200),
    };

    const newRotation = {
      x: new anchor.BN(0),
      y: new anchor.BN(1000), // Represent 1.0 as 1000 for fixed-point
      z: new anchor.BN(0),
      w: new anchor.BN(0),
    };

    // Update host position and animation
    const tx = await program.methods
      .updatePlayer(newPosition, newRotation, "running")
      .accounts({
        player: hostWallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Host updated position with transaction signature:", tx);

    // Verify host position was updated
    const roomAccount = await program.account.room.fetch(roomPda);
    const hostPlayer = roomAccount.players[0];
    expect(hostPlayer.value.position.x.toNumber()).to.equal(100);
    expect(hostPlayer.value.position.y.toNumber()).to.equal(50);
    expect(hostPlayer.value.position.z.toNumber()).to.equal(200);
    expect(hostPlayer.value.animation).to.equal("running");
  });

  it("Player 2 updates their position", async () => {
    const newPosition = {
      x: new anchor.BN(300),
      y: new anchor.BN(0),
      z: new anchor.BN(400),
    };

    const newRotation = {
      x: new anchor.BN(0),
      y: new anchor.BN(0),
      z: new anchor.BN(0),
      w: new anchor.BN(1000), // Represent 1.0 as 1000 for fixed-point
    };

    // Update player 2 position and animation
    const tx = await program.methods
      .updatePlayer(newPosition, newRotation, "walking")
      .accounts({
        player: player2Wallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player2Wallet])
      .rpc();

    console.log("Player 2 updated position with transaction signature:", tx);

    // Find player 2 in the array and verify position was updated
    const roomAccount = await program.account.room.fetch(roomPda);
    let foundPlayer2 = false;
    for (let i = 0; i < roomAccount.players.length; i++) {
      const player = roomAccount.players[i];
      if (
        player &&
        player.value.pubkey.toString() === player2Wallet.publicKey.toString()
      ) {
        foundPlayer2 = true;
        expect(player.value.position.x.toNumber()).to.equal(300);
        expect(player.value.position.y.toNumber()).to.equal(0);
        expect(player.value.position.z.toNumber()).to.equal(400);
        expect(player.value.animation).to.equal("walking");
      }
    }
    expect(foundPlayer2).to.be.true;
  });

  it("Host starts the game", async () => {
    // Start the game
    const tx = await program.methods
      .startGame()
      .accounts({
        player: hostWallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Host started game with transaction signature:", tx);

    // Verify game status changed to Playing
    const roomAccount = await program.account.room.fetch(roomPda);
    expect(roomAccount.status.playing).to.not.be.undefined;
    expect(roomAccount.startedAt).to.not.be.null;
  });

  it("Player 2 cannot start the game (not host)", async () => {
    try {
      // Attempt to start the game as player 2
      await program.methods
        .startGame()
        .accounts({
          player: player2Wallet.publicKey,
          room: roomPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player2Wallet])
        .rpc();

      // Should not reach here
      expect.fail("Expected error when non-host tries to start game");
    } catch (error) {
      expect(error.toString()).to.include("NotHost");
    }
  });

  it("Host finishes the game", async () => {
    // Finish the game
    const tx = await program.methods
      .finishGame()
      .accounts({
        player: hostWallet.publicKey,
        room: roomPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Host finished game with transaction signature:", tx);

    // Verify game status changed to Completed
    const roomAccount = await program.account.room.fetch(roomPda);
    expect(roomAccount.status.completed).to.not.be.undefined;
    expect(roomAccount.finishedAt).to.not.be.null;
  });

  // Here you would typically add tests for using the Ephemeral Rollup
  // These would involve delegating the room to an ER, making updates there,
  // and then committing back to the base layer
  it("Delegation to Ephemeral Rollup would go here", async () => {
    console.log("This test would delegate the room to an Ephemeral Rollup");
  });

  it("Updates in Ephemeral Rollup would go here", async () => {
    console.log(
      "This test would update player positions in the Ephemeral Rollup"
    );
  });

  it("Committing back to base layer would go here", async () => {
    console.log(
      "This test would commit the final state back to the base layer"
    );
  });
});
