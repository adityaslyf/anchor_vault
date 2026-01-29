use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
declare_id!("HMy8ed8mSHxKmR4h7ArNuV58EL7GfxLZ7YudXTPxU7n5");

#[program]
pub mod anchor_vault { // This say everything inside here are buttons user can press.
    use super::*;


    // This is a button that user can press to initialize the vault.
    // creates account , funds the vault and save the bump for the account.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
       ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    // This is a button that user can press to deposit money into the vault.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?; // amount = how much sol to deposit in lamports
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>{
        ctx.accounts.withdraw(amount)?;
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

#[derive(Accounts)]
pub struct Deposit<'info>{
 #[account(mut)]
 pub user: Signer<'info>,

 #[account(
    mut,
    seeds = [b"state", user.key().as_ref()],
    bump = vault_state.state_bump
 )]
pub vault_state: Account<'info, VaultState>,

#[account(
    mut,
    seeds = [b"vault", vault_state.key().as_ref()],
    bump = vault_state.vault_bump
)]
pub vault: SystemAccount<'info>,

pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}


impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {

        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len()); // this is the minimum amount of SOL needed to rent the account.

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        }; //take sol from user and put it in the vault.

        let cpi_ctx =  CpiContext::new(cpi_program, cpi_accounts); // this is the context of the transaction.

        transfer(cpi_ctx, rent_exempt)?; // this is the transfer of sol from user to vault.
        self.vault_state.vault_owner = self.user.key();
        self.vault_state.vault_amount = 0;

        self.vault_state.state_bump = bumps.vault_state; // save the bump for the state account.
        self.vault_state.vault_bump = bumps.vault; // save the bump for the vault account.
        Ok(())
    }
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount:u64) -> Result<()> {
        
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);

        transfer(cpi_ctx, amount)?;

        self.vault_state.vault_amount += amount;
        Ok(())
        }
}


impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        //only owner can withdraw
        require_keys_eq!(
            self.vault_state.vault_owner,
            self.user.key(),
            VaultError::Unauthorized
        );
        
        //can not withdraw more than is in the vault
        require!(
            self.vault_state.vault_amount >= amount,
            VaultError::InsufficientFunds
        );


        let vault_state_key = self.vault_state.key();
        //prepare signer seeds(this is why bumps exist)
        let seeds = &[
            b"vault",
            vault_state_key.as_ref(),
            &[self.vault_state.vault_bump]

        ];

        let signer = &[&seeds[..]];

        //Transfer sol from vault to user

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.user.to_account_info(),
            },
            signer
        );

        transfer(cpi_ctx, amount)?;

        //update vault state
        self.vault_state.vault_amount -= amount;
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


#[error_code]
pub enum VaultError {
    #[msg("You are not the owner of this vault")]
    Unauthorized,
    #[msg("You cannot withdraw more than is in the vault")]
    InsufficientFunds,
}