pub mod record_contribution;
pub mod calculate_points;
pub mod distribute_tokens;
pub mod manage_reserve;

pub use record_contribution::*;
pub use calculate_points::*;
pub use distribute_tokens::*;
pub use manage_reserve::*;

use anchor_lang::prelude::*;
use crate::state::{Contributor, PointsConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub monthly_threshold: u64,
    pub reserve_ratio: u16,
    pub max_points_per_type: u64,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = PointsConfig::SPACE,
        seeds = [b"points_config", authority.key().as_ref()],
        bump,
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
        space = Contributor::SPACE,
        seeds = [b"contributor", authority.key().as_ref()],
        bump,
    )]
    pub contributor: Account<'info, Contributor>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn process(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        let points_config = &mut ctx.accounts.points_config;
        let clock = Clock::get()?;
        
        points_config.authority = ctx.accounts.authority.key();
        points_config.monthly_threshold = args.monthly_threshold;
        points_config.reserve_ratio = args.reserve_ratio;
        points_config.max_points_per_type = args.max_points_per_type;
        points_config.current_period = 1;
        points_config.period_total_points = 0;
        points_config.last_calculation_time = clock.unix_timestamp;  // Add this line
        points_config.bump = ctx.bumps.points_config;

        emit!(ProgramInitialized {
            authority: ctx.accounts.authority.key(),
            monthly_threshold: args.monthly_threshold,
            reserve_ratio: args.reserve_ratio,
            max_points_per_type: args.max_points_per_type,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}

impl CreateContributor<'_> {
    pub fn process(ctx: Context<CreateContributor>) -> Result<()> {
        let contributor = &mut ctx.accounts.contributor;
        
        contributor.authority = ctx.accounts.authority.key();
        contributor.total_points = 0;
        contributor.current_month_points = 0;
        contributor.tokens_claimed = 0;
        contributor.last_claim_time = 0;
        contributor.contribution_count = 0;
        contributor.is_verified = false;
        contributor.bump = ctx.bumps.contributor;

        emit!(ContributorCreated {
            authority: ctx.accounts.authority.key(),
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