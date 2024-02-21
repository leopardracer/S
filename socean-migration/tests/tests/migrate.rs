use std::io::Write;

use s_controller_lib::{
    sync_sol_value_ix_by_mint_full, try_lst_state_list, try_pool_state, SyncSolValueByMintFreeArgs,
};
use s_controller_test_utils::{LstStateListBanksClient, PoolStateBanksClient};
use sanctum_solana_test_utils::ExtendedBanksClient;
use sanctum_token_lib::{token_account_balance, MintWithTokenProgram};
use socean_migration::{
    ata_program::LAINESOL_RESERVES_ID, lainesol_mint, lainesol_stake_pool, migrate_ix,
};
use solana_program_test::{find_file, read_file, ProgramTestContext};
use solana_readonly_account::sdk::KeyedAccount;
use solana_sdk::{
    account::ReadableAccount,
    bpf_loader_upgradeable::{self, UpgradeableLoaderState},
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};
use spl_calculator_lib::SplLstSolCommonFreeArgsConst;

use crate::common::base_program_test;

#[tokio::test]
async fn migrate_success() {
    // TODO: replace with actual values
    const EXPECTED_LAINESOL_RESERVES_AMT: u64 = 1000;
    const EXPECTED_LAINESOL_RESERVES_SOL_VALUE: u64 = 1001;

    let (pt, migrate_auth) = base_program_test();

    let mut ctx = pt.start_with_context().await;

    let ix = migrate_ix();
    let mut tx = Transaction::new_with_payer(&[ix], Some(&migrate_auth.pubkey()));
    tx.sign(&[&migrate_auth], ctx.last_blockhash);

    ctx.banks_client.process_transaction(tx).await.unwrap();

    upgrade_s_program(&mut ctx).await;

    // SyncSolValue
    let lst_state_list_acc = ctx.banks_client.get_lst_state_list_acc().await;
    let lainesol_stake_pool_acc = ctx
        .banks_client
        .get_account_unwrapped(lainesol_stake_pool::ID)
        .await;
    let ix = sync_sol_value_ix_by_mint_full(
        SyncSolValueByMintFreeArgs {
            lst_state_list: lst_state_list_acc,
            lst_mint: MintWithTokenProgram {
                pubkey: lainesol_mint::ID,
                token_program: spl_token::ID,
            },
        },
        &SplLstSolCommonFreeArgsConst {
            spl_stake_pool: KeyedAccount {
                pubkey: lainesol_stake_pool::ID,
                account: lainesol_stake_pool_acc,
            },
        }
        .resolve_spl_to_account_metas()
        .unwrap(),
        spl_calculator_lib::program::ID,
    )
    .unwrap();

    let mut tx = Transaction::new_with_payer(&[ix], Some(&migrate_auth.pubkey()));
    tx.sign(&[&migrate_auth], ctx.last_blockhash);

    // TODO: this is currently failing with insufficient account keys for instruction
    ctx.banks_client.process_transaction(tx).await.unwrap();

    // Check SOL values
    let lst_state_list_acc = ctx.banks_client.get_lst_state_list_acc().await;
    let lst_state_list = try_lst_state_list(&lst_state_list_acc.data).unwrap();
    assert_eq!(lst_state_list.len(), 1);
    assert_eq!(
        lst_state_list[0].sol_value,
        EXPECTED_LAINESOL_RESERVES_SOL_VALUE
    );
    let lainesol_reserves = ctx
        .banks_client
        .get_account_unwrapped(LAINESOL_RESERVES_ID)
        .await;
    assert_eq!(
        token_account_balance(lainesol_reserves).unwrap(),
        EXPECTED_LAINESOL_RESERVES_AMT
    );
    let pool_state_acc = ctx.banks_client.get_pool_state_acc().await;
    let pool_state = try_pool_state(&pool_state_acc.data).unwrap();
    assert_eq!(
        pool_state.total_sol_value,
        EXPECTED_LAINESOL_RESERVES_SOL_VALUE
    );
}

async fn upgrade_s_program(ctx: &mut ProgramTestContext) {
    let (s_prog_data_addr, _bump) = Pubkey::find_program_address(
        &[s_controller_lib::program::ID.as_ref()],
        &bpf_loader_upgradeable::ID,
    );
    let mut s_prog_data_acc = ctx
        .banks_client
        .get_account_unwrapped(s_prog_data_addr)
        .await;
    let pb = find_file("s_controller.so").expect("s_controller.so not found");
    let so_prog_data = read_file(pb);
    s_prog_data_acc
        .data
        .truncate(UpgradeableLoaderState::size_of_programdata_metadata());
    s_prog_data_acc.data.write_all(&so_prog_data).unwrap();
    ctx.set_account(&s_prog_data_addr, &s_prog_data_acc.to_account_shared_data());
}
