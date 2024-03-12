use anchor_lang::prelude::*;

pub const ROUND_STATE_SEED: &[u8] = b"Round";

#[account(zero_copy(unsafe))]
pub struct RoundState {
    pub bump: u8,
    pub merkle_root: [u8; 64],
    pub winner_indexes: [u16; 5], // index of nft in the merkle tree. There are 5 awards
    pub count: u16,               // number of nft in the in the merkle tree
    pub result_buffer: [u8; 32],  //  original randomness that switchboard return
    pub timestamp: i64,
    pub vrf: Pubkey,
}
