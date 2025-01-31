use anchor_lang::prelude::*;
use crate::state::points::PointsConfig;
use crate::errors::RewardError;

#[derive(Accounts)]
pub struct UpdateReserveConfig<'info> {
    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(
        constraint = points_config.authority == authority.key() @ RewardError::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdateReserveConfig<'info> {
    pub fn process(
        &mut self,
        new_reserve_ratio: Option<u16>,
        new_monthly_threshold: Option<u64>,
    ) -> Result<()> {
        if let Some(ratio) = new_reserve_ratio {
            require!(ratio <= 10000, RewardError::InvalidPointsCalculation);
            self.points_config.reserve_ratio = ratio;
        }

        if let Some(threshold) = new_monthly_threshold {
            self.points_config.monthly_threshold = threshold;
        }

        emit!(ReserveConfigUpdated {
            reserve_ratio: self.points_config.reserve_ratio,
            monthly_threshold: self.points_config.monthly_threshold,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[event]
pub struct ReserveConfigUpdated {
    pub reserve_ratio: u16,
    pub monthly_threshold: u64,
    pub timestamp: i64,
}