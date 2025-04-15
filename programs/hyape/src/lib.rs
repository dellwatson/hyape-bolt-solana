use bolt_lang::prelude::*;

declare_id!("DaPiwFoWxvBcoN94HZjcfz2bgaEQ3hfaDJivMZ7CuPuB");

#[program]
pub mod hyape {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
