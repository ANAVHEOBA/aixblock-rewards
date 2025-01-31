use anchor_lang::prelude::*;

#[error_code]
pub enum RewardError {
    #[msg("Contribution amount must be greater than zero")]
    InvalidContributionAmount,

    #[msg("Invalid points calculation")]
    InvalidPointsCalculation,

    #[msg("Insufficient token balance for distribution")]
    InsufficientBalance,

    #[msg("Distribution period not ended")]
    DistributionPeriodNotEnded,

    #[msg("Contributor not found")]
    ContributorNotFound,

    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid contribution type")]
    InvalidContributionType,

    #[msg("Monthly distribution already processed")]
    DistributionAlreadyProcessed,

    #[msg("Below minimum threshold for distribution")]
    BelowDistributionThreshold,

    #[msg("Reserve calculation error")]
    ReserveCalculationError,
}