use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token::{self, Approve, Mint, Token, TokenAccount};

use crate::{
    schema::{Metadata, StakeState, UserStakeInfo},
    StakeError,
};
use mpl_token_metadata::{instruction::freeze_delegated_account, ID as MetadataTokenId};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        associated_token::mint=nft_mint,
        associated_token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        space = std::mem::size_of::<UserStakeInfo>() + 8,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
}

pub fn exec(ctx: Context<Stake>) -> Result<()> {
    let clock = Clock::get().unwrap();
    msg!("Approving delegate");

    let cpi_approve_program = ctx.accounts.token_program.to_account_info();
    let cpi_approve_accounts = Approve {
        to: ctx.accounts.nft_token_account.to_account_info(),
        delegate: ctx.accounts.program_authority.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_approve_ctx = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
    token::approve(cpi_approve_ctx, 1)?;

    msg!("Freezing token account");
    let authority_bump = *ctx.bumps.get("program_authority").unwrap();
    invoke_signed(
        &freeze_delegated_account(
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

    ctx.accounts.stake_state.token_account = ctx.accounts.nft_token_account.key();
    ctx.accounts.stake_state.user_pubkey = ctx.accounts.user.key();
    ctx.accounts.stake_state.stake_state = StakeState::Staked;
    ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
    ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
    ctx.accounts.stake_state.is_initialized = true;

    require!(
        ctx.accounts.stake_state.stake_state == StakeState::Unstaked,
        StakeError::AlreadyStaked
    );
    Ok(())
}
