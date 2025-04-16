use bolt_lang::*;

// declare_id!("PlayrCmptYourProgramIdHere12345678901234567890");
declare_id!("AEg8kcweaPo1H8o98zsBuh9VfdJTj9suNdTWDzNJfBu3");


#[component(delegate)]
#[derive(Copy, Default)]
pub struct Player {
    pub id: [u8; 32],         // Player ID as bytes
    pub session_id: [u8; 32], // Session ID as bytes
    pub name: [u8; 16],       // Player name as bytes
    pub color: u32,           // Player color as hex
    pub connected: bool,      // Whether player is connected
    pub ready: bool,          // Whether player is ready
    pub last_activity: i64,   // Last activity timestamp
    
    // Position and rotation
    pub position_x: i64,
    pub position_y: i64,
    pub position_z: i64,
    pub rotation_x: i64,
    pub rotation_y: i64,
    pub rotation_z: i64,
    pub rotation_w: i64,
    
    // Other player attributes
    pub animation: u8,        // 0=idle, 1=walking, etc.
    pub model: u8,            // 0=default, 1=custom, etc.
    pub is_host: bool,        // Whether player is the host
    pub is_ready: bool,       // Whether player is ready
    pub party_id: [u8; 32],   // Party ID as bytes
    pub is_party_leader: bool,// Whether player is party leader
    pub game_mode: [u8; 16],  // Game mode as bytes
    pub status: u8,           // 0=available, 1=busy, etc.
}

// Add this to your existing Player component file
#[error_code]
pub enum PlayerError {
    #[msg("Player is not the host")]
    NotHost,
    
    #[msg("Player is not ready")]
    NotReady,
    
    #[msg("Invalid player state")]
    InvalidState,
    
    #[msg("Generic player error")]
    GenericError,
}