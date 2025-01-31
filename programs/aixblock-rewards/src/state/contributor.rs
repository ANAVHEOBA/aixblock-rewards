use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Contributor {
    // Unique identifier (usually wallet address)
    pub authority: Pubkey,
    
    // Total points earned
    pub total_points: u64,
    
    // Points earned this month
    pub current_month_points: u64,
    
    // Total tokens claimed
    pub tokens_claimed: u64,
    
    // Last claim timestamp
    pub last_claim_time: i64,
    
    // Contribution count
    pub contribution_count: u32,
    
    // Is verified contributor
    pub is_verified: bool,
    
    // Reserved space for future upgrades
    pub bump: u8,
}

impl Contributor {
    pub const SPACE: usize = 8 + // discriminator
        32 +    // authority
        8 +     // total_points
        8 +     // current_month_points
        8 +     // tokens_claimed
        8 +     // last_claim_time
        4 +     // contribution_count
        1 +     // is_verified
        1;      // bump
}