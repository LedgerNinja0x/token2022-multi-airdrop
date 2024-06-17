use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, TransferChecked};
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

declare_id!("9yk1CYdhn5mLFj6Wzz8rLjJ92HyTowMqELLQmB8YAgSr");

#[program]
pub mod token_airdrop {
    use super::*;

    pub fn send_to_all<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SendTokens<'info>>,
        amounts: Vec<u64>,
    ) -> Result<()> {
        let from_account = ctx.accounts.from.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();
        let authority_info = ctx.accounts.authority.to_account_info();
        let mint = ctx.accounts.mint.to_account_info();

        // Ensure the number of amounts matches the number of recipient accounts.
        require!(
            amounts.len() == ctx.remaining_accounts.len(),
            ErrorCode::MismatchedRecipientAmounts
        );

        for (recipient, &amount) in ctx.remaining_accounts.iter().zip(amounts.iter()) {
            // Attempt to borrow and deserialize the recipient's data to validate initialization.
            let recipient_data = recipient.try_borrow_data()?;
            let mut slice_ref: &[u8] = &recipient_data;
            TokenAccount::try_deserialize(&mut slice_ref)
                .map_err(|_| error!(ErrorCode::InvalidTokenAccount))?;
            drop(recipient_data);

            // Setup the accounts for the transfer checked operation.
            let transfer_cpi_accounts = TransferChecked {
                from: from_account.clone(),
                to: recipient.clone(),
                authority: authority_info.clone(),
                mint: mint.clone(),
            };

            // Create a context for the transfer and execute the transfer_checked instruction.
            let cpi_ctx = CpiContext::new(token_program.clone(), transfer_cpi_accounts);
            token_2022::transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;
        }

        Ok(())
    }
}

// Define the data structure for the accounts involved in the send_to_all function.
#[derive(Accounts)]
pub struct SendTokens<'info> {
    #[account(mut)]
    pub from: Box<InterfaceAccount<'info, TokenAccount>>,
    pub authority: Signer<'info>,
    #[account()]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Program<'info, Token2022>,
}

// Custom errors returned from this program.
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Token Account. Please ensure the account is correctly initialized.")]
    InvalidTokenAccount,
    #[msg("The number of recipient accounts does not match the number of amounts provided.")]
    MismatchedRecipientAmounts,
}
