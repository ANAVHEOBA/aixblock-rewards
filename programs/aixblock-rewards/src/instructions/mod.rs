pub mod record_contribution;
pub mod calculate_points;
pub mod distribute_tokens;
pub mod manage_reserve;
pub mod update_reserve;

pub use record_contribution::*;
pub use calculate_points::*;
pub use distribute_tokens::*;
pub use manage_reserve::*;
pub use update_reserve::*;

use anchor_lang::prelude::*;
use crate::state::{Contributor, PointsConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub monthly_threshold: u64,
    pub reserve_ratio: u16,
    pub max_points_per_type: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = PointsConfig::SPACE
    )]
    pub points_config: Account<'info, PointsConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateContributor<'info> {
    #[account(
        init,
        payer = authority,
        space = Contributor::SPACE
    )]
    pub contributor: Account<'info, Contributor>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn process(&mut self, args: InitializeArgs) -> Result<()> {
        let points_config = &mut self.points_config;
        
        points_config.authority = self.authority.key();
        points_config.monthly_threshold = args.monthly_threshold;
        points_config.reserve_ratio = args.reserve_ratio;
        points_config.max_points_per_type = args.max_points_per_type;
        points_config.current_period = 1;
        points_config.period_total_points = 0;

        emit!(ProgramInitialized {
            authority: self.authority.key(),
            monthly_threshold: args.monthly_threshold,
            reserve_ratio: args.reserve_ratio,
            max_points_per_type: args.max_points_per_type,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> CreateContributor<'info> {
    pub fn process(&mut self) -> Result<()> {
        let contributor = &mut self.contributor;
        
        contributor.authority = self.authority.key();
        contributor.total_points = 0;
        contributor.current_month_points = 0;
        contributor.tokens_claimed = 0;
        contributor.last_claim_time = 0;
        contributor.contribution_count = 0;
        contributor.is_verified = false;

        emit!(ContributorCreated {
            authority: self.authority.key(),
            contributor: contributor.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[event]
pub struct ProgramInitialized {
    pub authority: Pubkey,
    pub monthly_threshold: u64,
    pub reserve_ratio: u16,
    pub max_points_per_type: u64,
    pub timestamp: i64,
}

#[event]
pub struct ContributorCreated {
    pub authority: Pubkey,
    pub contributor: Pubkey,
    pub timestamp: i64,
}