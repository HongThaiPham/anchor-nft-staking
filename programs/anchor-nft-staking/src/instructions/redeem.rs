use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use crate::{StakeError, StakeState, UserStakeInfo};

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: manual check
    #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<Redeem>) -> Result<()> {
    require!(
        ctx.accounts.stake_state.is_initialized,
        StakeError::UninitializedAccount
    );

    require!(
        ctx.accounts.stake_state.stake_state == StakeState::Staked,
        StakeError::InvalidStakeState
    );
    let clock = Clock::get()?;
    msg!(
        "Stake last redeem: {:?}",
        ctx.accounts.stake_state.last_stake_redeem
    );

    msg!("Current time: {:?}", clock.unix_timestamp);
    let unix_time = clock.unix_timestamp - ctx.accounts.stake_state.last_stake_redeem;
    msg!("Seconds since last redeem: {}", unix_time);
    let redeem_amount = (10 * i64::pow(10, 2) * unix_time) / (24 * 60 * 60);
    msg!("Elligible redeem amount: {}", redeem_amount);

    msg!("Minting staking rewards");
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_stake_ata.to_account_info(),
                authority: ctx.accounts.stake_authority.to_account_info(),
            },
            &[&[
                b"mint".as_ref(),
                &[*ctx.bumps.get("stake_authority").unwrap()],
            ]],
        ),
        redeem_amount.try_into().unwrap(),
    )?;

    ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
    msg!(
        "Updated last stake redeem time: {:?}",
        ctx.accounts.stake_state.last_stake_redeem
    );
    Ok(())
}
