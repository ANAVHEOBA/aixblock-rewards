use anchor_lang::prelude::*;
use crate::state::{
    contributor::Contributor,
    points::PointsConfig,
};
use crate::errors::RewardError;

#[derive(Accounts)]
pub struct CalculateMonthlyPoints<'info> {
    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(
        mut,
        constraint = points_config.authority == authority.key() @ RewardError::Unauthorized
    )]
    pub authority: Signer<'info>,

    /// CHECK: Used for PDA derivation
    pub distribution_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateContributorPoints<'info> {
    #[account(mut)]
    pub contributor: Account<'info, Contributor>,

    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(
        constraint = points_config.authority == authority.key() @ RewardError::Unauthorized
    )]
    pub authority: Signer<'info>,
}

impl<'info> CalculateMonthlyPoints<'info> {
    pub fn process(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate distribution period
        self.points_config.validate_monthly_distribution(
            clock.unix_timestamp,
            self.points_config.last_calculation_time,
        )?;

        // Calculate distribution amount based on total points
        let total_points = self.points_config.period_total_points;
        
        if total_points == 0 {
            return Err(RewardError::InvalidPointsCalculation.into());
        }

        // Check if we meet the minimum threshold
        if total_points < self.points_config.monthly_threshold {
            msg!("Below monthly threshold, applying reserve ratio");
            
            // Calculate reserve amount
            let reserve_amount = self.points_config.calculate_reserve_amount(total_points)?;
            
            // Update points config with reserve
            self.points_config.update_reserve(reserve_amount)?;
        }

        // Update period and reset counters
        self.points_config.current_period = self.points_config.current_period
            .checked_add(1)
            .ok_or(RewardError::InvalidPointsCalculation)?;
        
        self.points_config.period_total_points = 0;
        self.points_config.last_calculation_time = clock.unix_timestamp;

        // Emit event
        emit!(MonthlyPointsCalculated {
            period: self.points_config.current_period,
            total_points,
            timestamp: clock.unix_timestamp,
            meets_threshold: total_points >= self.points_config.monthly_threshold,
        });

        Ok(())
    }
}

impl<'info> UpdateContributorPoints<'info> {
    pub fn process(&mut self) -> Result<()> {
        // Reset monthly points for new period
        self.contributor.current_month_points = 0;
        
        // Keep track of total points (historical)
        // No need to reset total_points as it's cumulative

        emit!(ContributorPointsUpdated {
            contributor: self.contributor.key(),
            total_points: self.contributor.total_points,
            period: self.points_config.current_period,
        });

        Ok(())
    }
}

#[event]
pub struct MonthlyPointsCalculated {
    pub period: u16,
    pub total_points: u64,
    pub timestamp: i64,
    pub meets_threshold: bool,
}

#[event]
pub struct ContributorPointsUpdated {
    pub contributor: Pubkey,
    pub total_points: u64,
    pub period: u16,
}