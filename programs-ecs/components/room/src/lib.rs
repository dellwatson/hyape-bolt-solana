use bolt_lang::*;

// declare_id!("RoomCmptYourProgramIdHere12345678901234567890");
declare_id!("4E1EMr7Df7mfNkRHnnQQDfdckcMZnq2qDKTrQUKc4wdU");

#[component(delegate)]
#[derive(Copy, Default)]
pub struct Room {
    pub room_id: [u8; 32],    // Room ID as bytes
    pub game_id: [u8; 32],    // Game ID as bytes
    pub created_at: i64,      // Unix timestamp
    pub status: u8,           // 0=lobby, 1=in-progress, 2=completed
    pub game_state: u8,       // 0=lobby, 1=match
    pub max_players: u8,      // Maximum number of players allowed
    pub player_count: u8,     // Current number of players
    pub player_ids: [[u8; 32]; 16], // Array of up to 16 player IDs
}
