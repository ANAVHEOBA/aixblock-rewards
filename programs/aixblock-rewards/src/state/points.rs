use anchor_lang::prelude::*;
use crate::errors::RewardError;
use crate::state::ContributionType;

#[account]
#[derive(Default)]
pub struct PointsConfig {
    // Authority who can update point configurations
    pub authority: Pubkey,
    
    // Minimum points required for monthly distribution
    pub monthly_threshold: u64,
    
    // Maximum points per contribution type per month
    pub max_points_per_type: u64,
    
    // Reserve ratio when below threshold (e.g., 5000 = 50%)
    pub reserve_ratio: u16,
    
    // Current distribution period
    pub current_period: u16,
    
    // Total points in current period
    pub period_total_points: u64,

    // Last calculation timestamp
    pub last_calculation_time: i64,
    
    // Reserved space for future upgrades
    pub bump: u8,
}

impl PointsConfig {
    pub const SPACE: usize = 8 + // discriminator
        32 +    // authority
        8 +     // monthly_threshold
        8 +     // max_points_per_type
        2 +     // reserve_ratio
        2 +     // current_period
        8 +     // period_total_points
        8 +     // last_calculation_time
        1;      // bump

    pub fn calculate_distribution_amount(
        &self,
        total_points: u64,
        monthly_pool: u64,
    ) -> Result<u64> {
        if total_points == 0 {
            return Err(RewardError::InvalidPointsCalculation.into());
        }

        // Check if below monthly threshold
        if total_points < self.monthly_threshold {
            // Apply reserve ratio
            let distribution_amount = (monthly_pool as u128)
                .checked_mul(self.reserve_ratio as u128)
                .ok_or(RewardError::InvalidPointsCalculation)?
                .checked_div(10000) // Convert from basis points
                .ok_or(RewardError::InvalidPointsCalculation)?;
            
            Ok(distribution_amount as u64)
        } else {
            Ok(monthly_pool)
        }
    }

    pub fn calculate_contribution_points(
        &self,
        contribution_type: &ContributionType,
        impact_score: u8,
    ) -> Result<u64> {
        // Base points for each contribution type
        let base_points: u64 = match contribution_type {
            ContributionType::Code => 10,
            ContributionType::Review => 20,
            ContributionType::Documentation => 15,
            ContributionType::Community => 5,
            ContributionType::Other => 5,
            ContributionType::Testing => 15,
            ContributionType::BugReport => 10,
            ContributionType::PullRequest => 30,
            ContributionType::CodeCommit => 10,
            ContributionType::CodeReview => 20,
        };

        // Impact multiplier (1-5 scale)
        let impact_multiplier = impact_score.max(1).min(5) as u64;

        // Calculate total points
        let total_points = base_points
            .checked_mul(impact_multiplier)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        // Ensure not exceeding max points per type
        Ok(total_points.min(self.max_points_per_type))
    }

    pub fn validate_monthly_distribution(
        &self,
        current_timestamp: i64,
        last_distribution: i64,
    ) -> Result<bool> {
        // Check if a month has passed (30 days in seconds)
        const MONTH_IN_SECONDS: i64 = 30 * 24 * 60 * 60;
        
        if current_timestamp - last_distribution < MONTH_IN_SECONDS {
            return Err(RewardError::DistributionPeriodNotEnded.into());
        }

        // Check if total points meet threshold for full distribution
        if self.period_total_points < self.monthly_threshold {
            msg!("Below monthly threshold, applying reserve ratio");
        }

        Ok(true)
    }

    pub fn update_period_points(&mut self, points: u64) -> Result<()> {
        self.period_total_points = self.period_total_points
            .checked_add(points)
            .ok_or(RewardError::InvalidPointsCalculation)?;
        Ok(())
    }

    pub fn calculate_reserve_amount(&self, total_points: u64) -> Result<u64> {
        self.calculate_distribution_amount(total_points, self.period_total_points)
    }

    pub fn update_reserve(&mut self, amount: u64) -> Result<()> {
        self.period_total_points = self.period_total_points
            .checked_add(amount)
            .ok_or(RewardError::InvalidPointsCalculation)?;
        Ok(())
    }
}

// Constants for point calculations
pub const MINIMUM_MONTHLY_THRESHOLD: u64 = 500;  // Minimum points needed for full distribution
pub const DEFAULT_RESERVE_RATIO: u16 = 5000;     // 50% in basis points
pub const MAX_POINTS_PER_TYPE: u64 = 1000;       // Maximum points per contribution type