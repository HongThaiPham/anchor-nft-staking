use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod schema;
pub use schema::*;

declare_id!("B8d7ps7aCkQVnPzTn32NtxXYWMW6H1BWfuXk7QTWT39U");

#[program]
pub mod anchor_nft_staking {
    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        Ok(())
    }

    // pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
    //     Ok(())
    // }

    // pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
    //     Ok(())
    // }
}
