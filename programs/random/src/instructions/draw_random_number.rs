use crate::states::*;
use anchor_lang::__private::bytemuck;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DrawRandomNumber<'info> {
    // round state
    #[account(mut)]
    pub round_state: Account<'info, RoundState>,
}
// 754 result
pub fn draw_random_number_handler(ctx: Context<DrawRandomNumber>, num_prize: u16) -> Result<()> {
    let round_state: &mut Account<'_, RoundState> = &mut ctx.accounts.round_state;

    let result_buffer = round_state.result_buffer.clone();

    let value: &[u32] = bytemuck::cast_slice(&result_buffer);

    let mut i = 0;

    let mut nft_count = round_state.nft_count.clone();
    let timestamp = Clock::get().unwrap().unix_timestamp.ilog2();

    loop {
        // loop will be stoped when drawing enough prize
        if i == num_prize || round_state.prize_remaining <= 0 {
            break;
        }

        if nft_count == 0 {
            nft_count = Clock::get().unwrap().epoch.ilog2();
        }

        let result = (value[i as usize % value.len()] + timestamp) % nft_count;

        if round_state.winner_indexes.contains(&result) == false {
            round_state.winner_indexes.push(result);

            round_state.prize_remaining -= 1;
            i += 1;
        } else {
            nft_count -= 1;
        }
    }

    msg!("remaining prize is {:?}", round_state.prize_count);

    Ok(())
}
