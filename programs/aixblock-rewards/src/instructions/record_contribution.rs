use anchor_lang::prelude::*;
use crate::state::{
    contributor::Contributor,
    contribution::{Contribution, ContributionType},
    points::PointsConfig,
};
use crate::errors::RewardError;

#[derive(Accounts)]
#[instruction(
    contribution_type: ContributionType,
    metadata: [u8; 32],
    impact_score: u8
)]
pub struct RecordContribution<'info> {
    #[account(mut)]
    pub contributor: Account<'info, Contributor>,

    #[account(
        init,
        payer = authority,
        space = Contribution::SPACE,
        seeds = [
            b"contribution",
            contributor.key().as_ref(),
            &contributor.contribution_count.to_le_bytes(),
        ],
        bump
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> RecordContribution<'info> {
    pub fn process(
        &mut self,
        contribution_type: ContributionType,
        metadata: [u8; 32],
        impact_score: u8,
        bump: u8,
    ) -> Result<()> {
        let contribution_type = contribution_type.clone(); // Clone here to avoid move

        // Calculate points for this contribution
        let points = self.points_config.calculate_contribution_points(
            &contribution_type,
            impact_score,
        )?;

        // Initialize the contribution account
        self.contribution.contributor = self.contributor.key();
        self.contribution.contribution_type = contribution_type.clone();
        self.contribution.points = points;
        self.contribution.timestamp = Clock::get()?.unix_timestamp;
        self.contribution.metadata = metadata;
        self.contribution.is_verified = false;
        self.contribution.period = self.points_config.current_period;
        self.contribution.bump = bump;

        // Update contributor's points
        self.contributor.total_points = self.contributor.total_points
            .checked_add(points)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        self.contributor.current_month_points = self.contributor.current_month_points
            .checked_add(points)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        self.contributor.contribution_count = self.contributor.contribution_count
            .checked_add(1)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        // Update total points in the current period
        self.points_config.update_period_points(points)?;

        // Emit an event
        emit!(ContributionRecorded {
            contributor: self.contributor.key(),
            contribution_type,
            points,
            timestamp: self.contribution.timestamp,
            period: self.contribution.period,
        });

        Ok(())
    }
}

#[event]
pub struct ContributionRecorded {
    pub contributor: Pubkey,
    pub contribution_type: ContributionType,
    pub points: u64,
    pub timestamp: i64,
    pub period: u16,
}