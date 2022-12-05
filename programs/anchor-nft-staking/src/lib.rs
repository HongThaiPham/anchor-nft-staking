use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod schema;
pub use schema::*;

pub mod errors;
pub use errors::*;

declare_id!("6DrV1wLkYvvypRfoHEPxUK4d1rrgPsbQoswRDRAPork2");

#[program]
pub mod anchor_nft_staking {
    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        instructions::stake::exec(ctx)
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        instructions::redeem::exec(ctx)
    }

    // pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
    //     Ok(())
    // }
}
