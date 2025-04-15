import { PublicKey } from "@solana/web3.js";
import { Room } from "../target/types/room";
import { Player } from "../target/types/player";
import { CreateRoomSystem } from "../target/types/create_room_system";
import { JoinRoomSystem } from "../target/types/join_room_system";
import { UpdatePlayerSystem } from "../target/types/update_player_system";
import { StartGameSystem } from "../target/types/start_game_system";
import { FinalizeGameSystem } from "../target/types/finalize_game_system";
import {
  InitializeNewWorld,
  AddEntity,
  InitializeComponent,
  ApplySystem,
  Program,
} from "@magicblock-labs/bolt-sdk";
import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";

describe("Multiplayer Room Test", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Constants used to test the program
  let worldPda: PublicKey;
  let roomEntityPda: PublicKey;
  let player1EntityPda: PublicKey;
  let player2EntityPda: PublicKey;
  let roomComponentPda: PublicKey;
  let player1ComponentPda: PublicKey;
  let player2ComponentPda: PublicKey;

  // Program references
  const roomComponent = anchor.workspace.Room as Program<Room>;
  const playerComponent = anchor.workspace.Player as Program<Player>;
  const createRoomSystem = anchor.workspace
    .CreateRoomSystem as Program<CreateRoomSystem>;
  const joinRoomSystem = anchor.workspace
    .JoinRoomSystem as Program<JoinRoomSystem>;
  const updatePlayerSystem = anchor.workspace
    .UpdatePlayerSystem as Program<UpdatePlayerSystem>;
  const startGameSystem = anchor.workspace
    .StartGameSystem as Program<StartGameSystem>;
  const finalizeGameSystem = anchor.workspace
    .FinalizeGameSystem as Program<FinalizeGameSystem>;

  // Helper functions to convert between string and buffer
  function stringToBuffer(str: string, length: number): Uint8Array {
    const buffer = Buffer.alloc(length);
    buffer.write(str);
    return buffer;
  }

  function bufferToString(buffer: Uint8Array): string {
    return Buffer.from(buffer).toString().replace(/\0/g, "");
  }

  it("Initialize a new world", async () => {
    const initNewWorld = await InitializeNewWorld({
      payer: provider.wallet.publicKey,
      connection: provider.connection,
    });
    const txSign = await provider.sendAndConfirm(initNewWorld.transaction);
    worldPda = initNewWorld.worldPda;
    console.log(
      `Initialized a new world (ID=${worldPda}). Signature: ${txSign}`
    );
  });

  it("Create room entity", async () => {
    const addEntity = await AddEntity({
      payer: provider.wallet.publicKey,
      world: worldPda,
      connection: provider.connection,
    });
    const txSign = await provider.sendAndConfirm(addEntity.transaction);
    roomEntityPda = addEntity.entityPda;
    console.log(
      `Initialized room entity (ID=${addEntity.entityId}). Signature: ${txSign}`
    );
  });

  it("Create player 1 entity", async () => {
    const addEntity = await AddEntity({
      payer: provider.wallet.publicKey,
      world: worldPda,
      connection: provider.connection,
    });
    const txSign = await provider.sendAndConfirm(addEntity.transaction);
    player1EntityPda = addEntity.entityPda;
    console.log(
      `Initialized player 1 entity (ID=${addEntity.entityId}). Signature: ${txSign}`
    );
  });

  it("Initialize room component", async () => {
    const initializeComponent = await InitializeComponent({
      payer: provider.wallet.publicKey,
      entity: roomEntityPda,
      componentId: roomComponent.programId,
    });
    const txSign = await provider.sendAndConfirm(
      initializeComponent.transaction
    );
    roomComponentPda = initializeComponent.componentPda;
    console.log(`Initialized room component. Signature: ${txSign}`);
  });

  it("Initialize player 1 component", async () => {
    const initializeComponent = await InitializeComponent({
      payer: provider.wallet.publicKey,
      entity: player1EntityPda,
      componentId: playerComponent.programId,
    });
    const txSign = await provider.sendAndConfirm(
      initializeComponent.transaction
    );
    player1ComponentPda = initializeComponent.componentPda;
    console.log(`Initialized player 1 component. Signature: ${txSign}`);
  });

  it("Create a room with player 1 as host", async () => {
    // Generate a unique room ID
    const roomId = "room-" + Date.now().toString(36);
    const gameId = "game-" + Date.now().toString(36);
    const playerName = "Player1";

    // Convert strings to buffer
    const roomIdBuffer = stringToBuffer(roomId, 32);
    const gameIdBuffer = stringToBuffer(gameId, 32);
    const playerIdBuffer = stringToBuffer(
      provider.wallet.publicKey.toString(),
      32
    );
    const playerNameBuffer = stringToBuffer(playerName, 16);

    // Apply create room system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: createRoomSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: roomEntityPda,
          components: [{ componentId: roomComponent.programId }],
        },
        {
          entity: player1EntityPda,
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        room_id: roomIdBuffer,
        game_id: gameIdBuffer,
        player_id: playerIdBuffer,
        player_name: playerNameBuffer,
        player_color: 0xffffff, // White color
        timestamp: Date.now(),
        max_players: 4,
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Created room. Signature: ${txSign}`);

    // Verify room was created
    const roomData = await roomComponent.account.room.fetch(roomComponentPda);
    expect(bufferToString(roomData.roomId)).to.equal(roomId);
    expect(roomData.playerCount).to.equal(1);
    expect(roomData.status).to.equal(0); // Lobby

    // Verify player 1 was initialized
    const playerData = await playerComponent.account.player.fetch(
      player1ComponentPda
    );
    expect(bufferToString(playerData.name)).to.equal(playerName);
    expect(playerData.isHost).to.equal(true);
  });

  it("Create player 2 entity", async () => {
    const addEntity = await AddEntity({
      payer: provider.wallet.publicKey,
      world: worldPda,
      connection: provider.connection,
    });
    const txSign = await provider.sendAndConfirm(addEntity.transaction);
    player2EntityPda = addEntity.entityPda;
    console.log(
      `Initialized player 2 entity (ID=${addEntity.entityId}). Signature: ${txSign}`
    );
  });

  it("Initialize player 2 component", async () => {
    const initializeComponent = await InitializeComponent({
      payer: provider.wallet.publicKey,
      entity: player2EntityPda,
      componentId: playerComponent.programId,
    });
    const txSign = await provider.sendAndConfirm(
      initializeComponent.transaction
    );
    player2ComponentPda = initializeComponent.componentPda;
    console.log(`Initialized player 2 component. Signature: ${txSign}`);
  });

  it("Player 2 joins the room", async () => {
    const playerName = "Player2";

    // Convert strings to buffer
    const playerIdBuffer = stringToBuffer(
      "player2-" + Date.now().toString(36),
      32
    );
    const playerNameBuffer = stringToBuffer(playerName, 16);

    // Apply join room system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: joinRoomSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: roomEntityPda,
          components: [{ componentId: roomComponent.programId }],
        },
        {
          entity: player2EntityPda,
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        player_id: playerIdBuffer,
        player_name: playerNameBuffer,
        player_color: 0xff0000, // Red color
        timestamp: Date.now(),
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Player 2 joined room. Signature: ${txSign}`);

    // Verify player count increased
    const roomData = await roomComponent.account.room.fetch(roomComponentPda);
    expect(roomData.playerCount).to.equal(2);

    // Verify player 2 was initialized
    const playerData = await playerComponent.account.player.fetch(
      player2ComponentPda
    );
    expect(bufferToString(playerData.name)).to.equal(playerName);
    expect(playerData.isHost).to.equal(false);
  });

  it("Update player 1 position", async () => {
    // Apply update player system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: updatePlayerSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: player1EntityPda,
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        timestamp: Date.now(),
        update_position: true,
        update_rotation: true,
        update_animation: true,
        update_status: false,
        update_ready: true,
        position_x: 100, // Some position
        position_y: 200,
        position_z: 300,
        rotation_x: 0,
        rotation_y: 0,
        rotation_z: 0,
        rotation_w: 1,
        animation: 1, // Walking
        status: 0,
        is_ready: true,
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Updated player 1 position. Signature: ${txSign}`);

    // Verify player position was updated
    const playerData = await playerComponent.account.player.fetch(
      player1ComponentPda
    );
    expect(playerData.positionX.toNumber()).to.equal(100);
    expect(playerData.positionY.toNumber()).to.equal(200);
    expect(playerData.positionZ.toNumber()).to.equal(300);
    expect(playerData.animation).to.equal(1); // Walking
    expect(playerData.isReady).to.equal(true);
  });

  it("Update player 2 position and set ready", async () => {
    // Apply update player system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: updatePlayerSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: player2EntityPda,
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        timestamp: Date.now(),
        update_position: true,
        update_rotation: true,
        update_animation: true,
        update_status: false,
        update_ready: true,
        position_x: 400, // Different position
        position_y: 500,
        position_z: 600,
        rotation_x: 0,
        rotation_y: 1,
        rotation_z: 0,
        rotation_w: 0,
        animation: 1, // Walking
        status: 0,
        is_ready: true,
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Updated player 2 position. Signature: ${txSign}`);

    // Verify player position was updated
    const playerData = await playerComponent.account.player.fetch(
      player2ComponentPda
    );
    expect(playerData.positionX.toNumber()).to.equal(400);
    expect(playerData.positionY.toNumber()).to.equal(500);
    expect(playerData.positionZ.toNumber()).to.equal(600);
    expect(playerData.animation).to.equal(1); // Walking
    expect(playerData.isReady).to.equal(true);
  });

  it("Start the game", async () => {
    // Apply start game system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: startGameSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: roomEntityPda,
          components: [{ componentId: roomComponent.programId }],
        },
        {
          entity: player1EntityPda, // Using player 1 as host
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        timestamp: Date.now(),
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Started game. Signature: ${txSign}`);

    // Verify game status changed
    const roomData = await roomComponent.account.room.fetch(roomComponentPda);
    expect(roomData.status).to.equal(1); // In-progress
    expect(roomData.gameState).to.equal(1); // Match
  });

  it("Finalize the game", async () => {
    // Apply finalize game system
    const applySystem = await ApplySystem({
      authority: provider.wallet.publicKey,
      systemId: finalizeGameSystem.programId,
      world: worldPda,
      entities: [
        {
          entity: roomEntityPda,
          components: [{ componentId: roomComponent.programId }],
        },
        {
          entity: player1EntityPda, // Using player 1 as host
          components: [{ componentId: playerComponent.programId }],
        },
      ],
      args: {
        timestamp: Date.now(),
      },
    });

    const txSign = await provider.sendAndConfirm(applySystem.transaction);
    console.log(`Finalized game. Signature: ${txSign}`);

    // Verify game status changed
    const roomData = await roomComponent.account.room.fetch(roomComponentPda);
    expect(roomData.status).to.equal(2); // Completed
  });
});
