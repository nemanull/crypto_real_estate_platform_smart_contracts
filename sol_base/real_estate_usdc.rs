use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};

declare_id!("J7kfYRFuCza2jiAbN9M9a6MWh652mNLM4QZL75p3LQYQ");

#[program]
pub mod real_estate_usdc {
    use super::*;

    pub fn create_property(
        ctx: Context<CreateProperty>,
        metadata_hash: [u8; 32],
        metadata_uri: String,
        annual_return_bp: u16,
        total_tokens: u64,
        price_per_token_usdc: u64,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        property.owner = ctx.accounts.owner.key();
        property.metadata_hash = metadata_hash;
        property.metadata_uri = metadata_uri;
        property.annual_return_bp = annual_return_bp;
        property.total_tokens = total_tokens;
        property.tokens_left = total_tokens;
        property.price_per_token_usdc = price_per_token_usdc;
        property.yield_per_token = 0;

        token::mint_to(
            ctx.accounts.into_mint_to_context(),
            total_tokens,
        )?;

        Ok(())
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        let cost = {
            let property = &ctx.accounts.property;
            require!(amount <= property.tokens_left, ErrorCode::InsufficientTokens);
            property.price_per_token_usdc.checked_mul(amount).unwrap()
        };

        token::transfer(
            ctx.accounts.into_transfer_usdc_to_owner(),
            cost,
        )?;
        
        token::transfer(
            ctx.accounts.into_transfer_shares_to_buyer(),
            amount,
        )?;
        
        {
            let property = &mut ctx.accounts.property;
            property.tokens_left -= amount;
        }

        Ok(())
    }

    pub fn deposit_yield(ctx: Context<DepositYield>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);
        let total_tokens = ctx.accounts.property.total_tokens;
        
        token::transfer(
            ctx.accounts.into_transfer_usdc_to_property(),
            amount,
        )?;
        
        let property = &mut ctx.accounts.property;
        property.yield_per_token += (amount as u128 * PRECISION) / total_tokens as u128;

        Ok(())
    }

    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        let holder_key = ctx.accounts.holder.key();
        let holder_balance = ctx.accounts.holder_ata.amount as u128;
        
        let property = &mut ctx.accounts.property;
        let last_yield = property.last_yield.get(&holder_key).copied().unwrap_or(0);
        let current_yield = property.yield_per_token;
        let pending_yield = ((current_yield - last_yield) * holder_balance) / PRECISION;

        require!(pending_yield > 0, ErrorCode::NoYield);

        property.last_yield.insert(holder_key, current_yield);

        token::transfer(
            ctx.accounts.into_transfer_usdc_to_holder(),
            pending_yield as u64,
        )?;

        Ok(())
    }

    pub fn mint_crosschain(ctx: Context<MintCrosschain>, amount: u64) -> Result<()> {
        token::mint_to(
            ctx.accounts.into_mint_mock_context(),
            amount,
        )?;
        Ok(())
    }
}

const PRECISION: u128 = 1_000_000_000;

#[account]
pub struct Property {
    pub owner: Pubkey,
    pub metadata_hash: [u8; 32],
    pub metadata_uri: String,
    pub annual_return_bp: u16,
    pub total_tokens: u64,
    pub tokens_left: u64,
    pub price_per_token_usdc: u64,
    pub yield_per_token: u128,
    pub last_yield: std::collections::BTreeMap<Pubkey, u128>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient tokens available")]
    InsufficientTokens,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("No yield available")]
    NoYield,
}

#[derive(Accounts)]
pub struct CreateProperty<'info> {
    #[account(init, payer = owner, space = 8 + 512)]
    pub property: Account<'info, Property>,
    #[account(init, payer = owner, mint::decimals = 0, mint::authority = owner)]
    pub share_mint: Account<'info, Mint>,
    #[account(init, payer = owner, token::mint = share_mint, token::authority = owner)]
    pub share_vault: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateProperty<'info> {
    fn into_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.share_mint.to_account_info(),
                to: self.share_vault.to_account_info(),
                authority: self.owner.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub buyer_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub share_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_ata: Account<'info, TokenAccount>,
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> BuyTokens<'info> {
    fn into_transfer_usdc_to_owner(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.buyer_usdc.to_account_info(),
                to: self.owner_usdc.to_account_info(),
                authority: self.buyer.to_account_info(),
            },
        )
    }
    fn into_transfer_shares_to_buyer(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.share_vault.to_account_info(),
                to: self.buyer_ata.to_account_info(),
                authority: self.property.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct MintCrosschain<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub recipient_ata: Account<'info, TokenAccount>,
    pub recipient: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> MintCrosschain<'info> {
    fn into_mint_mock_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.recipient_ata.to_account_info(),
                authority: self.property.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct DepositYield<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub depositor_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub property_usdc: Account<'info, TokenAccount>,
    pub depositor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositYield<'info> {
    fn into_transfer_usdc_to_property(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.depositor_usdc.to_account_info(),
                to: self.property_usdc.to_account_info(),
                authority: self.depositor.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(mut)]
    pub property: Account<'info, Property>,
    #[account(mut)]
    pub property_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub holder_usdc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub holder_ata: Account<'info, TokenAccount>,
    pub holder: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimYield<'info> {
    fn into_transfer_usdc_to_holder(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.property_usdc.to_account_info(),
                to: self.holder_usdc.to_account_info(),
                authority: self.property.to_account_info(),
            },
        )
    }
}
