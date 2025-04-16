use bolt_lang::*;
use player::{Player, PlayerError};
use room::{Room, RoomError};

// declare_id!("JoinRmSysYourProgramIdHere12345678901234567890");
declare_id!("8f6SZ35jjJqHGpkrwZubfLsRaCnNRd7TKX4qRGSy1KpZ");


#[system]
pub mod system_join_room {
    pub fn execute(ctx: Context<Components>, args: Args) -> Result<Components> {
        // Get component references
        let room = &mut ctx.accounts.room;
        let player = &mut ctx.accounts.player;

        // Check if room exists and has space
        if room.status != 0 || room.player_count >= room.max_players {
            // Error handling would go here in a real implementation
            // return Err(Error::InvalidArgument.into());
            return Err(RoomError::NotInLobbyState.into());

            // To (depending on what's available in the current bolt-lang version)
            // return Err(Error::InvalidInput.into());
            // or
            // return Err(Error::Other("Invalid argument").into());
        }

        // Add player ID to room's player_ids array
        room.player_ids[room.player_count as usize] = args.player_id;
        
        // Increment player count
        room.player_count += 1;

        // Initialize player data
        player.id = args.player_id;
        player.session_id = args.player_id; // Initially the same as player_id
        player.name = args.player_name;
        player.color = args.player_color;
        player.connected = true;
        player.ready = false;
        player.last_activity = args.timestamp;
        
        // Position and rotation
        player.position_x = 0;
        player.position_y = 0;
        player.position_z = 0;
        player.rotation_x = 0;
        player.rotation_y = 0;
        player.rotation_z = 0;
        player.rotation_w = 1; // Default quaternion
        
        // Other attributes
        player.animation = 0; // Idle
        player.model = 0; // Default
        player.is_host = false;
        player.is_ready = false;
        player.status = 0; // Available

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub room: Room,
        pub player: Player,
    }

    #[arguments]
    struct Args {
        player_id: [u8; 32],
        player_name: [u8; 16],
        player_color: u32,
        timestamp: i64,
    }
}