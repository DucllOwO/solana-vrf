use anchor_lang::prelude::*;

pub const ROUND_STATE_SEED: &[u8] = b"Round";

#[account]
pub struct RoundState {
    pub bump: u8,
    pub merkle_root: [u8; 32],
    pub nft_count: u32,          // number of nft in the in the merkle tree
    pub prize_count: u32,        // number of prize in this round
    pub prize_remaining: u32,    // number of prize has not draw yet in this round
    pub result_buffer: [u8; 32], //  original randomness that switchboard return
    pub timestamp: i64,
    pub vrf: Pubkey,
    pub winner_indexes: Vec<u32>,
}
