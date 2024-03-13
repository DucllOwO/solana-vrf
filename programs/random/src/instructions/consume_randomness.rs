use crate::errors::EscrowErrorCode;
use crate::states::*;
use anchor_lang::__private::bytemuck;
use anchor_lang::prelude::*;
use switchboard_v2::*;

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    // round state
    #[account(mut)]
    pub round_state: AccountLoader<'info, RoundState>,
    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == round_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
}

pub fn consume_randomness_handler(ctx: Context<ConsumeRandomness>) -> Result<()> {
    msg!("Successfully consumed randomness.");

    let vrf = ctx.accounts.vrf.load()?;
    let result_buffer = vrf.get_result()?;

    if result_buffer == [0u8; 32] {
        msg!("vrf buffer empty");
        return Ok(());
    }

    let round_state = &mut ctx.accounts.round_state.load_mut()?;

    msg!("Result buffer is {:?}", result_buffer);

    let value: &[u16] = bytemuck::cast_slice(&result_buffer[..]);
    msg!("value cast slice: {:?}", value);

    round_state.result_buffer = result_buffer;
    round_state.timestamp = Clock::get().unwrap().unix_timestamp;

    let _winner_indexes = round_state.winner_indexes;

    for i in 0..value.len() {
        // When will we obtain the complete lottery results
        if i >= _winner_indexes.len() {
            break;
        }

        let result = value[i] % round_state.count;
        msg!("Updating VRF State with random value: {:?}", result);
        let round_winner = round_state.winner_indexes;

        if round_winner.contains(&result) {
            continue;
        }

        round_state.winner_indexes[i] = result;
    }

    Ok(())
}
