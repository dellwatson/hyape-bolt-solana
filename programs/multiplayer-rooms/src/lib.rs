use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
declare_id!("DaPiwFoWxvBcoN94HZjcfz2bgaEQ3hfaDJivMZ7CuPuB");


#[program]
pub mod multiplayer_rooms {
    use super::*;

    // Initialize the program
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    // Create a new room
    pub fn create_room(ctx: Context<RoomOperation>, name: String, max_players: u8) -> Result<()> {
        let room = &mut ctx.accounts.room;
        
        // Initialize room data
        room.name = name;
        room.created_at = Clock::get()?.unix_timestamp;
        room.host = *ctx.accounts.player.key;
        room.status = RoomStatus::Lobby;
        room.max_players = max_players;
        room.player_count = 1;
        
        // Add host as first player
        let player_data = PlayerData {
            pubkey: *ctx.accounts.player.key,
            joined_at: Clock::get()?.unix_timestamp,
            position: Position { x: 0, y: 0, z: 0 },
            rotation: Rotation { x: 0, y: 0, z: 0, w: 1 },
            animation: "idle".to_string(),
            color: "#ffffff".to_string(),
            name: "Player".to_string(),
            ready: false,
            is_host: true,
        };
        
        room.players[0] = Some(player_data);
        
        Ok(())
    }
    
    // Join an existing room
    pub fn join_room(ctx: Context<RoomOperation>, player_name: String, player_color: String) -> Result<()> {
        let room = &mut ctx.accounts.room;
        
        // Check if room is full
        if room.player_count >= room.max_players {
            return err!(ErrorCode::RoomFull);
        }
        
        // Check if room is in lobby state
        if room.status != RoomStatus::Lobby {
            return err!(ErrorCode::GameInProgress);
        }
        
        // Create player data
        let player_data = PlayerData {
            pubkey: *ctx.accounts.player.key,
            joined_at: Clock::get()?.unix_timestamp,
            position: Position { x: 0, y: 0, z: 0 },
            rotation: Rotation { x: 0, y: 0, z: 0, w: 1 },
            animation: "idle".to_string(),
            color: player_color,
            name: player_name,
            ready: false,
            is_host: false,
        };
        
        // Find an empty slot and add player
        for i in 0..room.max_players as usize {
            if room.players[i].is_none() {
                room.players[i] = Some(player_data);
                room.player_count += 1;
                return Ok(());
            }
        }
        
        // This should not happen, but just in case
        err!(ErrorCode::RoomFull)
    }
    
    // Start the game
    pub fn start_game(ctx: Context<RoomOperation>) -> Result<()> {
        let room = &mut ctx.accounts.room;
        
        // Check if player is host
        let player_key = ctx.accounts.player.key();
        if room.host != player_key {
            return err!(ErrorCode::NotHost);
        }
        
        // Check if room is in lobby state
        if room.status != RoomStatus::Lobby {
            return err!(ErrorCode::GameAlreadyStarted);
        }
        
        // Set room status to Playing
        room.status = RoomStatus::Playing;
        room.started_at = Some(Clock::get()?.unix_timestamp);
        
        Ok(())
    }
    
    // Update player state (position, rotation, animation)
    pub fn update_player(
        ctx: Context<RoomOperation>,
        position: Option<Position>,
        rotation: Option<Rotation>,
        animation: Option<String>,
    ) -> Result<()> {
        let room = &mut ctx.accounts.room;
        let player_key = ctx.accounts.player.key();
        
        // Find the player in the room
        for i in 0..room.max_players as usize {
            if let Some(player) = &mut room.players[i] {
                if player.pubkey == player_key {
                    // Update player data if provided
                    if let Some(pos) = position {
                        player.position = pos;
                    }
                    
                    if let Some(rot) = rotation {
                        player.rotation = rot;
                    }
                    
                    if let Some(anim) = animation {
                        player.animation = anim;
                    }
                    
                    return Ok(());
                }
            }
        }
        
        // Player not found in room
        err!(ErrorCode::PlayerNotInRoom)
    }
    
    // Finish the game
    pub fn finish_game(ctx: Context<RoomOperation>) -> Result<()> {
        let room = &mut ctx.accounts.room;
        
        // Check if player is host
        let player_key = ctx.accounts.player.key();
        if room.host != player_key {
            return err!(ErrorCode::NotHost);
        }
        
        // Check if game is in progress
        if room.status != RoomStatus::Playing {
            return err!(ErrorCode::GameNotInProgress);
        }
        
        // Set room status to Completed
        room.status = RoomStatus::Completed;
        room.finished_at = Some(Clock::get()?.unix_timestamp);
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct RoomOperation<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = player,
        space = 8 + Room::MAX_SIZE,
        seeds = [b"room", player.key().as_ref()],
        bump
    )]
    pub room: Account<'info, Room>,
    
    pub system_program: Program<'info, System>,
}

// Room account data
#[account]
pub struct Room {
    pub name: String,           // Room name
    pub created_at: i64,        // Creation timestamp
    pub started_at: Option<i64>, // Start timestamp
    pub finished_at: Option<i64>, // Finish timestamp
    pub host: Pubkey,          // Host public key
    pub status: RoomStatus,    // Room status
    pub max_players: u8,       // Maximum players allowed
    pub player_count: u8,      // Current number of players
    pub players: [Option<PlayerData>; 16], // Array of player data (max 16 players)
}

impl Room {
    pub const MAX_SIZE: usize = 
        32 +    // name (string)
        8 +     // created_at
        9 +     // started_at (Option)
        9 +     // finished_at (Option)
        32 +    // host
        1 +     // status
        1 +     // max_players
        1 +     // player_count
        16 * (
            32 +    // pubkey
            8 +     // joined_at
            3 * 8 + // position
            4 * 8 + // rotation
            36 +    // animation (string)
            36 +    // color (string)
            36 +    // name (string)
            1 +     // ready
            1       // is_host
        );  // players array
}

// Player data within a room
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PlayerData {
    pub pubkey: Pubkey,        // Player public key
    pub joined_at: i64,        // Join timestamp
    pub position: Position,    // 3D position
    pub rotation: Rotation,    // Quaternion rotation
    pub animation: String,     // Current animation state
    pub color: String,         // Player color
    pub name: String,          // Player name
    pub ready: bool,           // Ready status
    pub is_host: bool,         // Whether player is host
}

// Position data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

// Rotation data structure (quaternion)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Rotation {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub w: i64,
}

// Room status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RoomStatus {
    Lobby,
    Playing,
    Completed,
}

impl Default for RoomStatus {
    fn default() -> Self {
        RoomStatus::Lobby
    }
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Room is full")]
    RoomFull,
    #[msg("Game is already in progress")]
    GameInProgress,
    #[msg("Game already started")]
    GameAlreadyStarted,
    #[msg("Game not in progress")]
    GameNotInProgress,
    #[msg("Only the host can perform this action")]
    NotHost,
    #[msg("Player not found in room")]
    PlayerNotInRoom,
}