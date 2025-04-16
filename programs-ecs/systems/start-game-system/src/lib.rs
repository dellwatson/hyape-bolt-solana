use bolt_lang::*;
use player::{Player, PlayerError};
use room::{Room, RoomError};
// declare_id!("StrtGmSysYourProgramIdHere12345678901234567890");
declare_id!("GPE1cWhMgMDiVsAX8waKaqLoBq4NGD6yxLetoz3KpiHZ");

#[error_code]
pub enum GameError {
    #[msg("Room is full or not in lobby state")]
    RoomFullOrNotInLobby,
}

#[system]
pub mod system_start_game {
    pub fn execute(ctx: Context<Components>, args: Args) -> Result<Components> {
        // Get room component reference
        let room = &mut ctx.accounts.room;
        
        // Only the host can start the game
        if !ctx.accounts.player.is_host {
            // return Err(Error::InvalidArgument.into());
            // If the above doesn't work, try:
            return Err(GameError::RoomFullOrNotInLobby.into());
        }
        
        // Update room status to in-progress
        room.status = 1; // In-progress
        room.game_state = 1; // Match
        
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub room: Room,
        pub player: Player, // The player attempting to start the game (should be host)
    }

    #[arguments]
    struct Args {
        timestamp: i64,
    }
}