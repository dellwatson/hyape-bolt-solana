use bolt_lang::*;
use player::Player;

// declare_id!("UpdPlrSysYourProgramIdHere12345678901234567890");
declare_id!("Gm4mG6xn43RDxsXUjJDiUrq8vE2FWQFWj8Ez3xViPYqk");


#[system]
pub mod system_update_player {
    pub fn execute(ctx: Context<Components>, args: Args) -> Result<Components> {
        // Get player component reference
        let player = &mut ctx.accounts.player;

        // Update player data
        player.last_activity = args.timestamp;
        
        // Update position and rotation if provided
        if args.update_position {
            player.position_x = args.position_x;
            player.position_y = args.position_y;
            player.position_z = args.position_z;
        }
        
        if args.update_rotation {
            player.rotation_x = args.rotation_x;
            player.rotation_y = args.rotation_y;
            player.rotation_z = args.rotation_z;
            player.rotation_w = args.rotation_w;
        }
        
        // Update animation if provided
        if args.update_animation {
            player.animation = args.animation;
        }
        
        // Update status if provided
        if args.update_status {
            player.status = args.status;
        }
        
        // Update ready state if provided
        if args.update_ready {
            player.is_ready = args.is_ready;
        }

        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        pub player: Player,
    }

    #[arguments]
    struct Args {
        timestamp: i64,
        // Flags to indicate which fields to update
        update_position: bool,
        update_rotation: bool,
        update_animation: bool,
        update_status: bool,
        update_ready: bool,
        // Position
        position_x: i64,
        position_y: i64,
        position_z: i64,
        // Rotation
        rotation_x: i64,
        rotation_y: i64,
        rotation_z: i64,
        rotation_w: i64,
        // Animation
        animation: u8,
        // Status
        status: u8,
        // Ready state
        is_ready: bool,
    }
}