use anchor_lang::prelude::*;
use instructions::consume_randomness::*;
use instructions::vrf_request_randomness::*;

pub mod errors;
pub mod instructions;
pub mod states;

declare_id!("CuhmoEskfBtkTivDpadfWqkieMj7jt1z9ciF1292h9eh");

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
}
