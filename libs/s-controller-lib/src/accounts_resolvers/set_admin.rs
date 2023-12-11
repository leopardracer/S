use s_controller_interface::{SControllerError, SetAdminKeys};
use solana_program::pubkey::Pubkey;
use solana_readonly_account::{ReadonlyAccountData, ReadonlyAccountPubkey};

use crate::{program::POOL_STATE_ID, try_pool_state};

#[derive(Clone, Copy, Debug)]
pub struct SetAdminFreeArgs<S: ReadonlyAccountData + ReadonlyAccountPubkey> {
    pub new_admin: Pubkey,
    pub pool_state: S,
}

impl<S: ReadonlyAccountData + ReadonlyAccountPubkey> SetAdminFreeArgs<S> {
    pub fn resolve(self) -> Result<SetAdminKeys, SControllerError> {
        let SetAdminFreeArgs {
            new_admin,
            pool_state: pool_state_acc,
        } = self;

        if *pool_state_acc.pubkey() != POOL_STATE_ID {
            return Err(SControllerError::IncorrectPoolState);
        }

        let pool_state_data = pool_state_acc.data();
        let pool_state = try_pool_state(&pool_state_data)?;

        Ok(SetAdminKeys {
            current_admin: pool_state.admin,
            new_admin,
            pool_state: POOL_STATE_ID,
        })
    }
}
