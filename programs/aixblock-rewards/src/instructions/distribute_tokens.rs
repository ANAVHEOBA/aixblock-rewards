use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{
    contributor::Contributor,
    points::PointsConfig,
};
use crate::errors::RewardError;

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct DistributeTokens<'info> {
    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(mut)]
    pub contributor: Account<'info, Contributor>,

    #[account(
        mut,
        constraint = reward_vault.owner == reward_vault_authority.key(),
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub contributor_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA for reward vault authority
    pub reward_vault_authority: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    #[account(
        init,
        payer = authority,
        space = DistributionPeriod::SPACE,
        seeds = [
            b"distribution".as_ref(),
            points_config.key().as_ref(),
            &points_config.current_period.to_le_bytes(),
        ],
        bump,
    )]
    pub distribution_period: Account<'info, DistributionPeriod>,
}

#[account]
pub struct DistributionPeriod {
    pub period: u16,
    pub total_tokens: u64,
    pub tokens_distributed: u64,
    pub total_points: u64,
    pub is_completed: bool,
    pub start_time: i64,
    pub end_time: i64,
    pub bump: u8,
}

impl DistributionPeriod {
    pub const SPACE: usize = 8 + // discriminator
        2 +     // period
        8 +     // total_tokens
        8 +     // tokens_distributed
        8 +     // total_points
        1 +     // is_completed
        8 +     // start_time
        8 +     // end_time
        1;      // bump
}

impl<'info> DistributeTokens<'info> {
    pub fn process(
        &mut self,
        vault_authority_bump: u8,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // Initialize distribution period
        self.distribution_period.period = self.points_config.current_period;
        self.distribution_period.start_time = clock.unix_timestamp;
        
        // Store points config pubkey for seeds
        let points_config_pubkey = self.points_config.key();
        
        // Get bump from seeds
        let (_, bump) = Pubkey::find_program_address(
            &[
                b"distribution".as_ref(),
                points_config_pubkey.as_ref(),
                &self.points_config.current_period.to_le_bytes(),
            ],
            &crate::ID,
        );
        self.distribution_period.bump = bump;

        // Verify distribution period is active
        if self.contributor.last_claim_time >= clock.unix_timestamp {
            return Err(RewardError::DistributionAlreadyProcessed.into());
        }

        // Calculate tokens to distribute
        let tokens_to_distribute = self.calculate_tokens_for_contributor()?;

        if tokens_to_distribute == 0 {
            return Err(RewardError::InsufficientBalance.into());
        }

        // Transfer tokens using stored pubkey
        let seeds = &[
            b"vault_authority".as_ref(),
            points_config_pubkey.as_ref(),
            &[vault_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_ix = Transfer {
            from: self.reward_vault.to_account_info(),
            to: self.contributor_token_account.to_account_info(),
            authority: self.reward_vault_authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            ),
            tokens_to_distribute,
        )?;

        // Update contributor state
        self.contributor.tokens_claimed = self.contributor.tokens_claimed
            .checked_add(tokens_to_distribute)
            .ok_or(RewardError::InvalidPointsCalculation)?;
        
        self.contributor.last_claim_time = clock.unix_timestamp;
        self.contributor.current_month_points = 0; // Reset monthly points

        // Update distribution period state
        self.distribution_period.tokens_distributed = self.distribution_period.tokens_distributed
            .checked_add(tokens_to_distribute)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        // Set total tokens for the period if not set
        if self.distribution_period.total_tokens == 0 {
            self.distribution_period.total_tokens = self.reward_vault.amount;
        }

        // Update total points for the period
        self.distribution_period.total_points = self.points_config.period_total_points;

        // Emit event
        emit!(TokensDistributed {
            contributor: self.contributor.key(),
            amount: tokens_to_distribute,
            period: self.points_config.current_period,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    fn calculate_tokens_for_contributor(&self) -> Result<u64> {
        let contributor_points = self.contributor.current_month_points;
        let total_period_points = self.points_config.period_total_points;
        
        if total_period_points == 0 || contributor_points == 0 {
            return Ok(0);
        }

        let monthly_pool = self.points_config.calculate_distribution_amount(
            total_period_points,
            self.reward_vault.amount,
        )?;

        // Calculate proportional share
        let tokens_to_distribute = (monthly_pool as u128)
            .checked_mul(contributor_points as u128)
            .ok_or(RewardError::InvalidPointsCalculation)?
            .checked_div(total_period_points as u128)
            .ok_or(RewardError::InvalidPointsCalculation)?;

        Ok(tokens_to_distribute as u64)
    }
}

#[event]
pub struct TokensDistributed {
    pub contributor: Pubkey,
    pub amount: u64,
    pub period: u16,
    pub timestamp: i64,
}