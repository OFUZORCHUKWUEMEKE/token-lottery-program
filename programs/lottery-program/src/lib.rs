use anchor_lang::prelude::*;

declare_id!("95wJrxrmf1G73kDy2mpJEWEhPuPnCQQT9RGxyCrnoaur");

#[program]
pub mod lottery_program {
    use super::*;

    pub fn initialize_config(ctx: Context<Initialize>) -> Result<()> {
       ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
       ctx.accounts.token_lottery.start_time= start;
       ctx.accounts.token_lottery.end_time = end;
       ctx.accounts.token_lottery.ticket_price = price;
       ctx.accounts.token_lottery.authority = *ctx.accounts.payer.key;
       ctx.accounts.token_lottery.lottery_pot_amount = 0;
       ctx.accounts.token_lottery.total_tickets =0;
       ctx.accounts.token_lottery.randomness_account=Pubkey::default();
       ctx.accounts.token_lottery.winner_chosen= false

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer:Signer<'info>,

    #[account(
        init,
        payer=payer,
        space=8,
        seeds=[b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery:Account<'info,TokenLottery>,
    pub system_program:Program<'info,System>
}

#[account]
#[derive(InitSpace)]
pub struct TokenLottery{
    pub bump:u8,
    pub winner:u64,
    pub winner_chosen:bool,
    pub start_time:u64,
    pub end_time:u64,
    pub lottery_pot_amount:u64,
    pub total_tickets:u64,
    pub ticket_price:u64,
    pub authority:Pubkey,
    pub randomness_account:Pubkey
}