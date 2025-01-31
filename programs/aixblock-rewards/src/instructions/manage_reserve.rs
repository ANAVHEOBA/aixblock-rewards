use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::points::PointsConfig;
use crate::errors::RewardError;

#[derive(Accounts)]
pub struct ManageReserve<'info> {
    #[account(mut)]
    pub points_config: Account<'info, PointsConfig>,

    #[account(
        mut,
        constraint = reserve_vault.owner == reserve_vault_authority.key(),
    )]
    pub reserve_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = distribution_vault.owner == distribution_vault_authority.key(),
    )]
    pub distribution_vault: Account<'info, TokenAccount>,

    /// CHECK: PDA for reserve vault
    pub reserve_vault_authority: AccountInfo<'info>,

    /// CHECK: PDA for distribution vault
    pub distribution_vault_authority: AccountInfo<'info>,

    #[account(
        constraint = points_config.authority == authority.key() @ RewardError::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

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

impl<'info> ManageReserve<'info> {
    pub fn process_reserve_transfer(
        &mut self,
        amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        // Verify reserve has sufficient balance
        if self.reserve_vault.amount < amount {
            return Err(RewardError::InsufficientBalance.into());
        }

        // Store points config pubkey for seeds
        let points_config_pubkey = self.points_config.key();
        let clock = Clock::get()?;

        // Transfer tokens from reserve to distribution vault
        let seeds = &[
            b"reserve_authority".as_ref(),
            points_config_pubkey.as_ref(),
            &[vault_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_ix = Transfer {
            from: self.reserve_vault.to_account_info(),
            to: self.distribution_vault.to_account_info(),
            authority: self.reserve_vault_authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            ),
            amount,
        )?;

        // Update points config state
        self.points_config.update_period_points(amount)?;

        // Emit event
        emit!(ReserveTransfer {
            amount,
            timestamp: clock.unix_timestamp,
            from_reserve: self.reserve_vault.key(),
            to_distribution: self.distribution_vault.key(),
        });

        Ok(())
    }

    pub fn process_add_to_reserve(
        &mut self,
        amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        // Store points config pubkey for seeds
        let points_config_pubkey = self.points_config.key();
        let clock = Clock::get()?;

        let seeds = &[
            b"distribution_authority".as_ref(),
            points_config_pubkey.as_ref(),
            &[vault_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_ix = Transfer {
            from: self.distribution_vault.to_account_info(),
            to: self.reserve_vault.to_account_info(),
            authority: self.distribution_vault_authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            ),
            amount,
        )?;

        // Update reserve stats
        self.points_config.update_reserve(amount)?;

        // Emit event
        emit!(ReserveDeposit {
            amount,
            timestamp: clock.unix_timestamp,
            new_reserve_balance: self.reserve_vault.amount,
        });

        Ok(())
    }
}

impl<'info> UpdateReserveConfig<'info> {
    pub fn process(
        &mut self,
        new_reserve_ratio: Option<u16>,
        new_monthly_threshold: Option<u64>,
    ) -> Result<()> {
        let clock = Clock::get()?;

        if let Some(ratio) = new_reserve_ratio {
            // Validate ratio is between 0-10000 (0-100%)
            require!(ratio <= 10000, RewardError::InvalidPointsCalculation);
            self.points_config.reserve_ratio = ratio;
        }

        if let Some(threshold) = new_monthly_threshold {
            require!(threshold > 0, RewardError::InvalidPointsCalculation);
            self.points_config.monthly_threshold = threshold;
        }

        // Emit event
        emit!(ReserveConfigUpdated {
            reserve_ratio: self.points_config.reserve_ratio,
            monthly_threshold: self.points_config.monthly_threshold,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}

#[event]
pub struct ReserveTransfer {
    pub amount: u64,
    pub timestamp: i64,
    pub from_reserve: Pubkey,
    pub to_distribution: Pubkey,
}

#[event]
pub struct ReserveDeposit {
    pub amount: u64,
    pub timestamp: i64,
    pub new_reserve_balance: u64,
}

#[event]
pub struct ReserveConfigUpdated {
    pub reserve_ratio: u16,
    pub monthly_threshold: u64,
    pub timestamp: i64,
}