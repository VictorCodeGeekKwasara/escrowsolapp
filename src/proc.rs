use solana_program::{
  account_info::{next_account_info,AccountInfo},
  entrypoint::ProgramResult,
  program_error::ProgramError,
  msg,
  pubkey::Pubkey,
  program_pack::{Pack, IsInitialized},
  sysvar::{rent::Rent, Sysvar},
  program::invoke
};

use crate::{instruction::EscrowInstruction, error::EscroeError, state::Escrow};

pub struct Processor;
impl Processor {
  pub fn process(program_id: &Pubkey, accounts:&[AccountInfo], instruction_data: &[u8])-> ProgramResult{
    let instruction = EscrowIstruction::unpack(instruction_data)?;

    match instruction {
      EscrowInstruction::InitEscrow { amount
      }=> {
        msg!("Instruction: InitEscrow");
        Self::process_init_escrow(accounts,amount,program_id)
      }
    }
  }

  fn process_init_escrow(
    accounts: &[AccountInfo],
    amount:u64,
    program_id: &Pubkey,

  )-> ProgramResult{

    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let temp_token_account = next_account_info(account_info_iter)?;

    let token_to_recieve_account = next_account_info(account_info_iter)?;

    if *token_to_recieve_account.owner != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }

    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()){
      return Err(EscrowError::NotRentExempt.into());
    }

    let mut escroe_info = Escrow::unpacck_unchecked(&escroe_account.try_borrow_data()?)?;

    if escrow_info.is_initialized(){
      return Err(ProgramError::AccountAlreadyInitailezed);
    }

    escroe_info.is_initialized = true ;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    escroe_info.initializer_token_to_recieve_account_pubkey = *token_to_receive_account.key;
    escrow_info.expected_amount = amount ;

    Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;

    let token_program = next_account_info(account_info_iter)?;

    let owner_change_ix = spl_token::instruction::set_authority(
      token_program.key,
      temp_token_account.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initialializer.key],
    );
     msg!("Calling the token program to transfer token account ownership...");

     invoke(
       &owner_change_ix,
       &[temp_token_account.clone(),
       initializer.clone(),
       token_program.clone(),],
     )?;
     Ok(())
  }
}