use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface}
};
use anchor_spl::metadata::{
    Metadata,
    MetadataAccount,
    CreateMetadataAccountsV3,
    CreateMasterEditionV3,
    SignMetadata,
    SetAndVerifySizedCollectionItem,
    create_master_edition_v3,
    create_metadata_accounts_v3,
    sign_metadata,
    set_and_verify_sized_collection_item,
    mpl_token_metadata::types::{
            CollectionDetails,
            Creator, 
            DataV2,
        },
};

declare_id!("95wJrxrmf1G73kDy2mpJEWEhPuPnCQQT9RGxyCrnoaur");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";
#[constant]
pub const URI: &str = "Token Lottery";
#[constant]
pub const SYMBOL: &str = "TICKET";

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
        let signer_seeds :&[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint]
        ]];
        msg!("Creating Mint Account");

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo{
                    mint:ctx.accounts.collection_mint.to_account_info(),
                    to:ctx.accounts.collection_token_account.to_account_info(),
                    authority:ctx.accounts.collection_mint.to_account_info()
                },
                signer_seeds,
            ),
        )?;
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: NAME.to_string(),
                symbol: SYMBOL.to_string(),
                uri: URI.to_string(),
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.collection_mint.key(),
                    verified: false,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            true,
            true,
            Some(CollectionDetails::V1 { size: 0 }), // set as collection nft
        )?;

        msg!("Creating Master edition accounts");
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        msg!("verifying collection");
        sign_metadata(CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                creator: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())

    }

}


#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.ticket_num.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program = token_program
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
        ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            ticket_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct CommitWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChooseWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.ticket_num.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program = token_program
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
        ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            ticket_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
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
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: This account will be initialized by the metaplex program
    #[account(
        mut,
        seeds=[
            b"metadata",token_metadata_program.key().as_ref(),collection_mint.key().as_ref()
            ],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This account will be initialized by the metaplex program
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"edition".as_ref()],
        bump,
        token::mint = collection_mint,
        token::authority = collection_token_account
    )]
    pub collection_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    pub bump: u8,
    pub winner: u64,
    pub winner_chosen: bool,
    pub lottery_start: u64,
    pub lottery_end: u64,
    // Is it good practice to store SOL on an account used for something else?
    pub lottery_pot_amount: u64,
    pub ticket_num: u64,
    pub price: u64,
    pub randomness_account: Pubkey,
    pub authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Incorrect randomness account")]
    IncorrectRandomnessAccount,
    #[msg("Lottery not completed")]
    LotteryNotCompleted,
    #[msg("Lottery is not open")]
    LotteryNotOpen,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Randomness already revealed")]
    RandomnessAlreadyRevealed,
    #[msg("Randomness not resolved")]
    RandomnessNotResolved,
    #[msg("Winner not chosen")]
    WinnerNotChosen,
    #[msg("Winner already chosen")]
    WinnerChosen,
    #[msg("Ticket is not verified")]
    NotVerifiedTicket,
    #[msg("Incorrect ticket")]
    IncorrectTicket,
}