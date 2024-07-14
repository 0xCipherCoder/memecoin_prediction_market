use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("AJrErLEJXotf5ECiEHXqr89Qyn6huDzawT9PW6TmpW9e");

#[program]
pub mod memecoin_prediction_market {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_name: String,
        expiry_timestamp: i64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.name = market_name;
        market.creator = ctx.accounts.creator.key();
        market.expiry_timestamp = expiry_timestamp;
        market.outcome = false;
        market.settled = false;
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, prediction: bool) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(!market.settled, ErrorCode::MarketAlreadySettled);
        
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time < market.expiry_timestamp,
            ErrorCode::MarketExpired
        );

        // Transfer tokens from user to market account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.market_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // Update market state
        if prediction {
            market.yes_amount += amount;
        } else {
            market.no_amount += amount;
        }

        // Create bet account
        let bet = &mut ctx.accounts.bet;
        bet.user = ctx.accounts.user.key();
        bet.market = market.key();
        bet.amount = amount;
        bet.prediction = prediction;

        Ok(())
    }

    pub fn settle_market(ctx: Context<SettleMarket>, outcome: bool) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(!market.settled, ErrorCode::MarketAlreadySettled);
        require!(
            Clock::get()?.unix_timestamp >= market.expiry_timestamp,
            ErrorCode::MarketNotExpired
        );

        market.outcome = outcome;
        market.settled = true;

        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let market = &ctx.accounts.market;
        let bet = &mut ctx.accounts.bet;
    
        require!(market.settled, ErrorCode::MarketNotSettled);
        require!(bet.prediction == market.outcome, ErrorCode::NotWinner);
        require!(!bet.winnings_claimed, ErrorCode::WinningsAlreadyClaimed);

        let total_pool = market.yes_amount + market.no_amount;
        let winning_pool = if market.outcome {
            market.yes_amount
        } else {
            market.no_amount
        };

        let winnings = (bet.amount as u128 * total_pool as u128 / winning_pool as u128) as u64;

        // Transfer winnings from market account to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.market_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
                &[&[
                    b"market".as_ref(),
                    market.name.as_bytes(),
                    &[ctx.bumps.market], // Updated this line
                ]],
            ),
            winnings,
        )?;
        bet.winnings_claimed = true;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(market_name: String)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1 + 1 + market_name.len(),
        seeds = [b"market", market_name.as_bytes()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 32 + 8 + 1 + 1,
        seeds = [b"bet", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = market
    )]
    pub market_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut, has_one = creator)]
    pub market: Account<'info, PredictionMarket>,
    pub creator: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut, seeds = [b"market", market.name.as_bytes()], bump)]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        mut,
        seeds = [b"bet", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct PredictionMarket {
    pub name: String,
    pub creator: Pubkey,
    pub expiry_timestamp: i64,
    pub yes_amount: u64,
    pub no_amount: u64,
    pub outcome: bool,
    pub settled: bool,
}

#[account]
pub struct Bet {
    pub user: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub prediction: bool,
    pub winnings_claimed: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Market has already been settled")]
    MarketAlreadySettled,
    #[msg("Market has expired")]
    MarketExpired,
    #[msg("Market has not expired yet")]
    MarketNotExpired,
    #[msg("Market has not been settled")]
    MarketNotSettled,
    #[msg("You are not a winner in this market")]
    NotWinner,
    #[msg("Winnings have already been claimed")]
    WinningsAlreadyClaimed,
}