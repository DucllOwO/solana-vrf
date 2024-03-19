use crate::errors::*;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::*;
use anchor_spl::token::{Token, TokenAccount};
use switchboard_v2::{
    OracleQueueAccountData, PermissionAccountData, SbState, VrfAccountData, VrfRequestRandomness,
};

#[derive(Accounts)]
#[instruction(round_id: String, params: RequestRandomnessParams)]
pub struct RequestRandomness<'info> {
    // PAYER ACCOUNTS
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut,
        constraint =
            payer_wallet.owner == user.key()
            && switchboard_escrow.mint == program_state.load()?.token_mint
    )]
    pub payer_wallet: Account<'info, TokenAccount>,
    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == round_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
    // switchboard accounts
    #[account(mut,
        has_one = data_buffer
    )]
    pub oracle_queue: AccountLoader<'info, OracleQueueAccountData>,
    #[account(
        mut,
        constraint = oracle_queue.load()?.authority == queue_authority.key()
    )]
    /// CHECK:
    pub queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK:
    pub data_buffer: AccountInfo<'info>,
    #[account(mut)]
    pub permission: AccountLoader<'info, PermissionAccountData>,
    #[account(mut,
        constraint = switchboard_escrow.owner == program_state.key() && switchboard_escrow.mint == program_state.load()?.token_mint
    )]
    pub switchboard_escrow: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_state: AccountLoader<'info, SbState>,
    /// CHECK:
    #[account(
        address = *vrf.to_account_info().owner,
        constraint = switchboard_program.executable == true
    )]
    pub switchboard_program: AccountInfo<'info>,
    // SYSTEM ACCOUNTS
    /// CHECK:
    #[account(address = recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(
        init,
        seeds = [
            ROUND_STATE_SEED,
            user.key.as_ref(),
            vrf.key().as_ref(),
            round_id.as_bytes()
        ],
        payer = user,
        space = 8 + 1 + 32 + 4 + 4 + 4 + 32 + 8 + 32 + 4 + 4*params.prize_count as usize + 8,
        bump
    )]
    pub round_state: Account<'info, RoundState>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestRandomnessParams {
    pub permission_bump: u8,
    pub switchboard_state_bump: u8,
    pub round_bump: u8,
    pub merkle_root: [u8; 32],
    pub nft_count: u32,
    pub prize_count: u32,
}

pub fn request_randomness_handler(
    ctx: Context<RequestRandomness>,
    round_id: String,
    params: RequestRandomnessParams,
) -> Result<()> {
    let switchboard_program = ctx.accounts.switchboard_program.to_account_info();
    let round_state = &mut ctx.accounts.round_state;

    round_state.vrf = ctx.accounts.vrf.key();
    round_state.bump = ctx.bumps.get("round_state").unwrap().clone();
    round_state.merkle_root = params.merkle_root;
    round_state.nft_count = params.nft_count;
    round_state.prize_count = params.prize_count;
    round_state.prize_remaining = params.prize_count;

    let bump = round_state.bump.clone();

    // build vrf request struct from the Switchboard Rust crate
    let vrf_request_randomness = VrfRequestRandomness {
        authority: ctx.accounts.round_state.to_account_info(),
        vrf: ctx.accounts.vrf.to_account_info(),
        oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
        queue_authority: ctx.accounts.queue_authority.to_account_info(),
        data_buffer: ctx.accounts.data_buffer.to_account_info(),
        permission: ctx.accounts.permission.to_account_info(),
        escrow: ctx.accounts.switchboard_escrow.clone(),
        payer_wallet: ctx.accounts.payer_wallet.clone(),
        payer_authority: ctx.accounts.user.to_account_info(),
        recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
        program_state: ctx.accounts.program_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let vrf_key = ctx.accounts.vrf.key();
    let user_key = ctx.accounts.user.key();
    let state_seeds: &[&[&[u8]]] = &[&[
        &ROUND_STATE_SEED,
        user_key.as_ref(),
        vrf_key.as_ref(),
        round_id.as_bytes(),
        &[bump],
    ]];

    // submit vrf request with PDA signature
    msg!("requesting randomness");
    vrf_request_randomness.invoke_signed(
        switchboard_program,
        params.switchboard_state_bump,
        params.permission_bump,
        state_seeds,
    )?;

    msg!("randomness requested successfully");

    Ok(())
}
