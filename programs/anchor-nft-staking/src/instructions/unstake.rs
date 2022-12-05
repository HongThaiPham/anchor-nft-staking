use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Revoke, Token, TokenAccount},
};
use mpl_token_metadata::{instruction::thaw_delegated_account, ID as MetadataTokenId};

use crate::{Metadata, StakeState, UserStakeInfo};

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

    ctx.accounts.stake_state.stake_state = StakeState::Unstaked;
    Ok(())
}
