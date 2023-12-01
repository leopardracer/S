use flat_fee_interface::{
    initialize_verify_account_keys, initialize_verify_account_privileges, InitializeAccounts,
    InitializeKeys,
};
use sanctum_onchain_utils::{
    system_program::{create_pda, CreateAccountAccounts, CreateAccountArgs},
    utils::{load_accounts, log_and_return_acc_privilege_err, log_and_return_wrong_acc_err},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{account_resolvers::InitializeFreeArgs, program, utils::try_program_state_mut};

pub fn process_initialize_unchecked(
    InitializeAccounts {
        payer,
        state,
        system_program: _,
    }: InitializeAccounts,
    initial_manager: Pubkey,
    initial_lp_withdrawal_fee_bps: u16,
) -> ProgramResult {
    create_pda(
        CreateAccountAccounts {
            from: payer,
            to: state,
        },
        CreateAccountArgs {
            space: program::STATE_SIZE,
            owner: program::ID,
        },
        &[&[program::STATE_SEED, &[program::STATE_BUMP]]],
    )?;

    let mut bytes = state.try_borrow_mut_data()?;
    let state = try_program_state_mut(&mut bytes)?;

    state.manager = initial_manager;
    state.lp_withdrawal_fee_bps = initial_lp_withdrawal_fee_bps;

    Ok(())
}

pub fn verify_initialize<'me, 'info>(
    accounts: &'me [AccountInfo<'info>],
) -> Result<InitializeAccounts<'me, 'info>, ProgramError> {
    let actual: InitializeAccounts = load_accounts(accounts)?;

    let free_args = InitializeFreeArgs {
        payer: *actual.payer.key,
    };
    let expected: InitializeKeys = free_args.resolve();

    initialize_verify_account_keys(&actual, &expected).map_err(log_and_return_wrong_acc_err)?;
    initialize_verify_account_privileges(&actual).map_err(log_and_return_acc_privilege_err)?;

    Ok(actual)
}
