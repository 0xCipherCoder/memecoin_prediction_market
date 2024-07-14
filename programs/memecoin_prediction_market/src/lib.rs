use anchor_lang::prelude::*;

declare_id!("AJrErLEJXotf5ECiEHXqr89Qyn6huDzawT9PW6TmpW9e");

#[program]
pub mod memecoin_prediction_market {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
