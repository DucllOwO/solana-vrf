use crate::instructions::RequestRandomnessParams;
use anchor_lang::prelude::*;
use instructions::consume_randomness::*;
use instructions::draw_random_number::*;
use instructions::vrf_request_randomness::*;
pub mod errors;
pub mod instructions;
pub mod states;

declare_id!("8VmbofKHtWostC5AqZ74r1SJdUpASE3wZPaNccXyGP9L");

#[program]
mod random {
    use super::*;

    pub fn request_randomness(
        ctx: Context<RequestRandomness>,
        round_id: String,
        params: RequestRandomnessParams,
    ) -> Result<()> {
        request_randomness_handler(ctx, round_id, params)
    }

    pub fn consume_randomness(ctx: Context<ConsumeRandomness>) -> Result<()> {
        consume_randomness_handler(ctx)
    }

    pub fn draw_random_number(ctx: Context<DrawRandomNumber>, num_prize: u16) -> Result<()> {
        // Call the handler function and return its result
        draw_random_number_handler(ctx, num_prize)
    }
}
