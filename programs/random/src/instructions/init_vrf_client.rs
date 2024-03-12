use crate::errors::*;
use crate::states::*;
use anchor_lang::prelude::*;
use switchboard_v2::VrfAccountData;

#[derive(Accounts)]
pub struct InitVrfClient<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == round_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
    pub system_program: Program<'info, System>,
}

pub fn init_vrf_client_handler(ctx: Context<InitVrfClient>) -> Result<()> {
    msg!("init_client validate");

    let mut round_state = ctx.accounts.round_state.load_init()?;
    round_state.bump = ctx.bumps.get("round_state").unwrap().clone();

    round_state.vrf = ctx.accounts.vrf.key();
    round_state.timestamp = 0;

    Ok(())
}
