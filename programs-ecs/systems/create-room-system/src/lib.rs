use bolt_lang::*;

// declare_id!("CrtRmSysYourProgramIdHere12345678901234567890");
declare_id!("CJ2ddnADWJuBeJ5vr5kCPjMiZfPMT6e83bU4kBAsbCXz");

#[system]
pub mod system_create_room {
    pub fn execute(ctx: Context<Components>, args: Args) -> Result<Components> {
        // Get component references
        let room = &mut ctx.accounts.room;
        let player = &mut ctx.accounts.player;

        // Initialize room data
        room.room_id = args.room_id;
        room.game_id = args.game_id;
        room.created_at = args.timestamp;
        room.status = 0; // Lobby
        room.game_state = 0; // Lobby
        room.max_players = args.max_players;
        room.player_count = 1; // Start with 1 player (the host)
        
        // Store the first player ID in the player_ids array
        room.player_ids[0] = args.player_id;

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
        player.is_host = true;
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
        room_id: [u8; 32],
        game_id: [u8; 32],
        player_id: [u8; 32],
        player_name: [u8; 16],
        player_color: u32,
        timestamp: i64,
        max_players: u8,
    }
}