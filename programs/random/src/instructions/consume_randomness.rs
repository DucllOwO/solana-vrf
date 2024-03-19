use crate::errors::EscrowErrorCode;
use crate::states::*;
use anchor_lang::prelude::*;
use switchboard_v2::*;

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    // round state
    #[account(mut)]
    pub round_state: Account<'info, RoundState>,
    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == round_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
}
// 796 result
pub fn consume_randomness_handler(ctx: Context<ConsumeRandomness>) -> Result<()> {
    msg!("Successfully consumed randomness.");

    let vrf = ctx.accounts.vrf.load()?;
    let result_buffer = vrf.get_result()?;

    if result_buffer == [0u8; 32] {
        return Ok(());
    }

    let round_state = &mut ctx.accounts.round_state;

    round_state.result_buffer = result_buffer;
    round_state.timestamp = Clock::get().unwrap().unix_timestamp;

    Ok(())
}
