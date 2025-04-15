use bolt_lang::*;

// declare_id!("FnlGmeSysYourProgramIdHere12345678901234567890");
declare_id!("5j31pAW4KRxYdWM718y2vRiL2963YE9Q3EFLZNdv3DPX");

#[system]
pub mod system_finalize_game {
    pub fn execute(ctx: Context<Components>, args: Args) -> Result<Components> {
        // Get room component reference
        let room = &mut ctx.accounts.room;
        
        // Only the host can finalize the game
        if !ctx.accounts.player.is_host {
            return Err(Error::InvalidArgument.into());
        }
        
        // Set the room status to completed
        room.status = 2; // Completed
        
        // This would be where you trigger any post-game actions
        // like rewards, but that would require interaction with other contracts
        // which would be handled outside of this system
        
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub room: Room,
        pub player: Player, // The player finalizing the game (should be host)
    }

    #[arguments]
    struct Args {
        timestamp: i64,
    }
}