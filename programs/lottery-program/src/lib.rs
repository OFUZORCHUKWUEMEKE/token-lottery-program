use anchor_lang::prelude::*;

declare_id!("95wJrxrmf1G73kDy2mpJEWEhPuPnCQQT9RGxyCrnoaur");

#[program]
pub mod lottery_program {
    use super::*;

    pub fn initialize_config(ctx: Context<Initialize>,start:u64,end:u64,price:u64) -> Result<()> {
       ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
       ctx.accounts.token_lottery.start_time= start;
       ctx.accounts.token_lottery.end_time = end;
       ctx.accounts.token_lottery.ticket_price = price;
       ctx.accounts.token_lottery.authority = *ctx.accounts.payer.key;
       ctx.accounts.token_lottery.lottery_pot_amount = 0;
       ctx.accounts.token_lottery.total_tickets =0;
       ctx.accounts.token_lottery.randomness_account=Pubkey::default();
       ctx.accounts.token_lottery.winner_chosen= false;

       Ok(())
    }

    pub fn initialize_lottery(ctx:Context<InitializeLottery>)->Result<()>{

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

#[derive(Accounts)]
pub struct InitializeLottery<'info>{
    #[account(mut)]
    pub payer:Signer<'info>,
    #[account(
        init,
        payer=payer,
        mint::decimals=0,
        mint::authority=payer,
        mint::freeze_authority=payer,
        seeds=[b"collection_mint".as_ref()]
    )]
    pub collection_mint:InterfaceAccount<'info,Mint>,
    #[account(
        init,
        payer=payer,
        token::mint=collection_mint,
        token::authority=payer,
        seeds=[]
    )]

    pub token_metadata_program:Program<'info,Metadata>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub token_program:Interface<'info,TokenInterface>,
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