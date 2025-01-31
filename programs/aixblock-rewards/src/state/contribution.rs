use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
#[repr(u8)] // <--- THIS IS CRITICAL
pub enum ContributionType {
    Code,
    Review,
    Documentation,
    Community,
    Other,
    Testing,
    BugReport,
    PullRequest,
    CodeCommit,
    CodeReview,
}

#[account]
pub struct Contribution {
    // Contributor who made this contribution
    pub contributor: Pubkey,
    
    // Type of contribution
    pub contribution_type: ContributionType,
    
    // Points awarded for this contribution
    pub points: u64,
    
    // Timestamp of contribution
    pub timestamp: i64,
    
    // Optional metadata (e.g., PR number, commit hash)
    pub metadata: [u8; 32],
    
    // Is verified by moderator
    pub is_verified: bool,
    
    // Distribution period (month/year)
    pub period: u16,
    
    // Reserved space for future upgrades
    pub bump: u8,
}

impl Contribution {
    pub const SPACE: usize = 8 + // discriminator
        32 +    // contributor
        1 +     // contribution_type (enum)
        8 +     // points
        8 +     // timestamp
        32 +    // metadata
        1 +     // is_verified
        2 +     // period
        1;      // bump

    pub fn calculate_points(&self) -> Result<u64> {
        let base_points = match self.contribution_type {
            ContributionType::CodeCommit => 10,
            ContributionType::PullRequest => 30,
            ContributionType::CodeReview => 20,
            ContributionType::Documentation => 15,
            ContributionType::BugReport => 10,
            ContributionType::Testing => 15,
            ContributionType::Code => 10,
            ContributionType::Review => 20,
            ContributionType::Community => 5,
            ContributionType::Other => 5,
        };

        Ok(base_points)
    }
}