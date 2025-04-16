use bolt_lang::prelude::*;
use anchor_lang::prelude::*;

// // Custom heap implementation for Solana programs
// #[cfg(feature = "custom-heap")]
// #[global_allocator]
// static ALLOC: anchor_lang::solana_program::custom_heap::CustomHeap = anchor_lang::solana_program::custom_heap::CustomHeap::new();

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
        
        // Initialize room data with fixed-size arrays
        // FIXED: Safely copy name to fixed-size array
        let name_bytes = name.as_bytes();
        let mut name_array = [0u8; 32];
        for (i, &byte) in name_bytes.iter().enumerate().take(32) {
            name_array[i] = byte;
        }
        
        room.name = name_array;
        room.created_at = Clock::get()?.unix_timestamp;
        room.host = ctx.accounts.player.key();
        room.status = RoomStatus::Lobby;
        room.max_players = max_players;
        room.player_count = 1;
        
        // Create player data with fixed-size arrays
        let mut player_data = PlayerData::default();
        player_data.pubkey = ctx.accounts.player.key();
        player_data.joined_at = Clock::get()?.unix_timestamp;
        player_data.is_host = true;
        
        // Store the player data
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
        
        // Create player data with fixed-size arrays
        let mut player_data = PlayerData::default();
        player_data.pubkey = ctx.accounts.player.key();
        player_data.joined_at = Clock::get()?.unix_timestamp;
        
        // FIXED: Safely copy player name to fixed-size array
        let name_bytes = player_name.as_bytes();
        for (i, &byte) in name_bytes.iter().enumerate().take(16) {
            player_data.name[i] = byte;
        }
        
        // FIXED: Safely copy player color to fixed-size array
        let color_bytes = player_color.as_bytes();
        for (i, &byte) in color_bytes.iter().enumerate().take(8) {
            player_data.color[i] = byte;
        }
        
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
                        // FIXED: Safely copy animation string to fixed-size array
                        // First reset the animation array
                        player.animation = [0u8; 16];
                        
                        // Then copy new animation
                        let anim_bytes = anim.as_bytes();
                        for (i, &byte) in anim_bytes.iter().enumerate().take(16) {
                            player.animation[i] = byte;
                        }
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
        seeds = [b"room", room_host.key().as_ref()],
        bump
    )]
    pub room: Account<'info, Room>,
    
    /// The host of the room (needed for seed derivation)
    /// CHECK: This account is only used for PDA derivation
    pub room_host: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

// Room account data
#[account]
pub struct Room {
    pub name: [u8; 32],        // Room name as fixed-size array
    pub created_at: i64,        // Creation timestamp
    pub started_at: Option<i64>, // Start timestamp
    pub finished_at: Option<i64>, // Finish timestamp
    pub host: Pubkey,          // Host public key
    pub status: RoomStatus,    // Room status
    pub max_players: u8,       // Maximum players allowed
    pub player_count: u8,      // Current number of players
    pub players: [Option<PlayerData>; 8], // Reduced from 16 to 8 players
}

impl Room {
    pub const MAX_SIZE: usize = 
        32 +    // name (fixed-size array)
        8 +     // created_at
        9 +     // started_at (Option)
        9 +     // finished_at (Option)
        32 +    // host
        1 +     // status
        1 +     // max_players
        1 +     // player_count
        8 * (   // 8 players
            32 +    // pubkey
            8 +     // joined_at
            3 * 8 + // position
            4 * 8 + // rotation
            16 +    // animation (fixed-size array)
            8 +     // color (fixed-size array)
            16 +    // name (fixed-size array)
            1 +     // ready
            1       // is_host
        );  // players array
}

// Player data within a room
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlayerData {
    pub pubkey: Pubkey,        // Player public key
    pub joined_at: i64,        // Join timestamp
    pub position: Position,    // 3D position
    pub rotation: Rotation,    // Quaternion rotation
    pub animation: [u8; 16],   // Current animation state as fixed-size array
    pub color: [u8; 8],        // Player color as fixed-size array
    pub name: [u8; 16],        // Player name as fixed-size array
    pub ready: bool,           // Ready status
    pub is_host: bool,         // Whether player is host
}

// Implement Default manually for fixed-size arrays
impl Default for PlayerData {
    fn default() -> Self {
        // FIXED: Use safer initialization
        let mut animation = [0u8; 16];
        let idle = b"idle";
        for (i, &byte) in idle.iter().enumerate() {
            animation[i] = byte;
        }
        
        let mut color = [0u8; 8];
        let default_color = b"#ffffff";
        for (i, &byte) in default_color.iter().enumerate() {
            color[i] = byte;
        }
        
        let mut name = [0u8; 16];
        let default_name = b"Player";
        for (i, &byte) in default_name.iter().enumerate() {
            name[i] = byte;
        }
        
        Self {
            pubkey: Pubkey::default(),
            joined_at: 0,
            position: Position::default(),
            rotation: Rotation::default(),
            animation,
            color,
            name,
            ready: false,
            is_host: false,
        }
    }
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