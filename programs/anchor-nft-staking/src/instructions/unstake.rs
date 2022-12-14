use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Revoke, Token, TokenAccount},
};
use mpl_token_metadata::{instruction::thaw_delegated_account, ID as MetadataTokenId};

use crate::{Metadata, StakeError, StakeState, UserStakeInfo};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        constraint = *user.key == stake_state.user_pubkey,
        constraint = nft_token_account.key() == stake_state.token_account
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: manual check
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
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
    pub metadata_program: Program<'info, Metadata>,
}

pub fn exec(ctx: Context<Unstake>) -> Result<()> {
    require!(
        ctx.accounts.stake_state.is_initialized,
        StakeError::UninitializedAccount
    );

    require!(
        ctx.accounts.stake_state.stake_state == StakeState::Staked,
        StakeError::InvalidStakeState
    );

    msg!("Thawing token account");
    let authority_bump = *ctx.bumps.get("program_authority").unwrap();
    invoke_signed(
        &thaw_delegated_account(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.program_authority.key(),
            ctx.accounts.nft_token_account.key(),
            ctx.accounts.nft_edition.key(),
            ctx.accounts.nft_mint.key(),
        ),
        &[
            ctx.accounts.program_authority.to_account_info(),
            ctx.accounts.nft_token_account.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.metadata_program.to_account_info(),
        ],
        &[&[b"authority", &[authority_bump]]],
    )?;

    msg!("Revoking delegate");

    let cpi_revoke_program = ctx.accounts.token_program.to_account_info();
    let cpi_revoke_accounts = Revoke {
        source: ctx.accounts.nft_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_revoke_ctx = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
    token::revoke(cpi_revoke_ctx)?;

    let clock = Clock::get()?;

    msg!(
        "Stake last redeem: {:?}",
        ctx.accounts.stake_state.last_stake_redeem
    );

    msg!("Current time: {:?}", clock.unix_timestamp);
    let unix_time = clock.unix_timestamp - ctx.accounts.stake_state.last_stake_redeem;
    msg!("Seconds since last redeem: {}", unix_time);
    // Swap the next two lines out between prod/testing
    // let redeem_amount = (10000000000 * i64::pow(10, 2) * unix_time) / (24 * 60 * 60);
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
    ctx.accounts.stake_state.total_earned += redeem_amount as u64;
    msg!(
        "Updated last stake redeem time: {:?}",
        ctx.accounts.stake_state.last_stake_redeem
    );

    ctx.accounts.stake_state.stake_state = StakeState::Unstaked;
    Ok(())
}
