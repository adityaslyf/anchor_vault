use anchor_lang::prelude::*;

declare_id!("HMy8ed8mSHxKmR4h7ArNuV58EL7GfxLZ7YudXTPxU7n5");

#[program]
pub mod anchor_vault { // This say everything inside here are buttons user can press.
    use super::*;


    // This is a button that user can press to initialize the vault.
    // creates account , funds the vault and save the bump for the account.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
       ctx.accounts.initialize(&ctx.bumps);
        Ok(())
    }

    // This is a button that user can press to deposit money into the vault.
    pub fn deposite(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

}

// Things the user needs to provide to the button.
#[derive(Accounts)]
pub struct Initialize<'info> { // to run initalize, i need these things.
    #[account(mut)]
    pub user: Signer<'info>, // user who is initializing the vault, payer of the transaction.


    // account that stores the state of the vault.
    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,

// account that stores the money in the vault.
//This is: a system account holds SOL controlled by the program NOT owned by the user
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System> // this is solana bank that is needed to create accounts and move sol

}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bump: &InitializeBumps) -> Result<()> {

        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len()); // this is the minimum amount of SOL needed to rent the account.

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer<'> = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        } //take sol from user and put it in the vault.

        let cpi_ctx =  CpiContext::new(cpi_program, cpi_accounts); // this is the context of the transaction.

        transfer(cpi_ctx, rent_exempt)?; // this is the transfer of sol from user to vault.


        self.vault_state.state_bump = bump.vault_state; // save the bump for the state account.
        self.vault_state.vault_bump = bump.vault; // save the bump for the vault account.
        Ok(())
    }
}

// this is the state of the vault.
#[derive(InitSpace)]
#[account]

// this stores who owns the vault and how much money is in the vault.
pub struct VaultState {
    pub vault_owner: Pubkey,
    pub vault_amount: u64,
    pub vault_bump: u8,
    pub state_bump: u8,
}
