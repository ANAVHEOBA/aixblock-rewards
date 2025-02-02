use anchor_lang::prelude::*;
use crate::instructions::*;
use crate::state::ContributionType;

pub mod errors;
pub mod instructions;
pub mod state;

declare_id!("BV7MhRzrPUKPjBFYHJkuipQTcKjkSLAFJzsF3zNUYeB6");

#[program]
pub mod aixblock_rewards {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        args: InitializeArgs,
    ) -> Result<()> {
        Initialize::process(ctx, args)
    }

    pub fn create_contributor(
        ctx: Context<CreateContributor>,
    ) -> Result<()> {
        CreateContributor::process(ctx)
    }

    pub fn record_contribution(
        ctx: Context<RecordContribution>,
        contribution_type: ContributionType,
        metadata: [u8; 32],
        impact_score: u8,
        bump: u8,
    ) -> Result<()> {
        validate_impact_score(impact_score)?;
        ctx.accounts.process(
            contribution_type,
            metadata,
            impact_score,
            bump,
        )
    }

    pub fn calculate_monthly_points(
        ctx: Context<CalculateMonthlyPoints>,
    ) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn update_contributor_points(
        ctx: Context<UpdateContributorPoints>,
    ) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn distribute_tokens(
        ctx: Context<DistributeTokens>,
        vault_authority_bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(vault_authority_bump)
    }

    pub fn process_reserve_transfer(
        ctx: Context<ManageReserve>,
        amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        require!(amount > 0, ProgramError::InvalidAmount);
        ctx.accounts.process_reserve_transfer(amount, vault_authority_bump)
    }

    pub fn process_add_to_reserve(
        ctx: Context<ManageReserve>,
        amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        require!(amount > 0, ProgramError::InvalidAmount);
        ctx.accounts.process_add_to_reserve(amount, vault_authority_bump)
    }

    pub fn update_reserve_config(
        ctx: Context<UpdateReserveConfig>,
        new_reserve_ratio: Option<u16>,
        new_monthly_threshold: Option<u64>,
    ) -> Result<()> {
        if let Some(ratio) = new_reserve_ratio {
            validate_reserve_ratio(ratio)?;
        }
        if let Some(threshold) = new_monthly_threshold {
            validate_points(threshold)?;
        }
        ctx.accounts.process(new_reserve_ratio, new_monthly_threshold)
    }
}

// Constants for the program
pub const PROGRAM_SEED: &[u8] = b"aixblock_rewards";
pub const MIN_POINTS: u64 = 1;
pub const MAX_IMPACT_SCORE: u8 = 5;
pub const MONTH_IN_SECONDS: i64 = 30 * 24 * 60 * 60;
pub const MAX_RESERVE_RATIO: u16 = 10_000;

#[error_code]
pub enum ProgramError {
    #[msg("Invalid points calculation")]
    InvalidPointsCalculation,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid contribution type")]
    InvalidContributionType,
    
    #[msg("Distribution period not ended")]
    DistributionPeriodNotEnded,
    
    #[msg("Below minimum threshold")]
    BelowMinimumThreshold,
    
    #[msg("Invalid impact score")]
    InvalidImpactScore,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid reserve ratio")]
    InvalidReserveRatio,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Reserve calculation error")]
    ReserveCalculationError,
}

pub fn validate_impact_score(score: u8) -> Result<()> {
    require!(
        score > 0 && score <= MAX_IMPACT_SCORE,
        ProgramError::InvalidImpactScore
    );
    Ok(())
}

pub fn validate_points(points: u64) -> Result<()> {
    require!(
        points >= MIN_POINTS,
        ProgramError::InvalidPointsCalculation
    );
    Ok(())
}

pub fn validate_reserve_ratio(ratio: u16) -> Result<()> {
    require!(
        ratio <= MAX_RESERVE_RATIO,
        ProgramError::InvalidReserveRatio
    );
    Ok(())
}