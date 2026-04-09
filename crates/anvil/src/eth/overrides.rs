pub(crate) use alloy_evm::overrides::{OverrideBlockHashes, StateOverrideError};
use alloy_primitives::{Address, keccak256, map::HashMap};
use alloy_rpc_types_eth::state::{AccountOverride, StateOverride};
use fluentbase_evm::{AnalyzedBytecode, EthereumMetadata};
use fluentbase_types::PRECOMPILE_EVM_RUNTIME;
use revm::{
    Database, DatabaseCommit,
    bytecode::{BytecodeDecodeError, ownable_account::OwnableAccountDecodeError},
    state::{Account, AccountInfo, AccountStatus, Bytecode, EvmStorageSlot},
};

/// Applies the given state overrides (a set of [`AccountOverride`]) to the database.
pub(crate) fn apply_state_overrides<DB>(
    overrides: StateOverride,
    db: &mut DB,
) -> Result<(), StateOverrideError<DB::Error>>
where
    DB: Database + DatabaseCommit,
{
    for (account, account_overrides) in overrides {
        apply_account_override(account, account_overrides, db)?;
    }
    Ok(())
}

/// Applies a single [`AccountOverride`] to the database.
fn apply_account_override<DB>(
    account: Address,
    account_override: AccountOverride,
    db: &mut DB,
) -> Result<(), StateOverrideError<DB::Error>>
where
    DB: Database + DatabaseCommit,
{
    let mut info = db.basic(account).map_err(StateOverrideError::Database)?.unwrap_or_default();

    if let Some(nonce) = account_override.nonce {
        info.nonce = nonce;
    }
    if let Some(code) = account_override.code {
        try_override_evm_bytecode::<DB>(&mut info, Bytecode::new_raw_checked(code)?)?;
    }
    if let Some(balance) = account_override.balance {
        info.balance = balance;
    }

    // Create a new account marked as touched
    let mut acc = revm::state::Account {
        info: info.clone(),
        original_info: Box::new(info),
        status: AccountStatus::Touched,
        storage: Default::default(),
        transaction_id: 0,
    };

    let storage_diff = match (account_override.state, account_override.state_diff) {
        (Some(_), Some(_)) => return Err(StateOverrideError::BothStateAndStateDiff(account)),
        (None, None) => None,
        // If we need to override the entire state, we firstly mark account as destroyed to clear
        // its storage, and then we mark it is "NewlyCreated" to make sure that old storage won't be
        // used.
        (Some(state), None) => {
            // Destroy the account to ensure that its storage is cleared
            db.commit(HashMap::from_iter([(
                account,
                Account {
                    status: AccountStatus::SelfDestructed | AccountStatus::Touched,
                    ..Default::default()
                },
            )]));
            // Mark the account as created to ensure that old storage is not read
            acc.mark_created();
            Some(state)
        }
        (None, Some(state)) => Some(state),
    };

    if let Some(state) = storage_diff {
        for (slot, value) in state {
            acc.storage.insert(
                slot.into(),
                EvmStorageSlot {
                    // we use inverted value here to ensure that storage is treated as changed
                    original_value: (!value).into(),
                    present_value: value.into(),
                    is_cold: false,
                    transaction_id: 0,
                },
            );
        }
    }

    db.commit(HashMap::from_iter([(account, acc)]));

    Ok(())
}

fn try_override_evm_bytecode<DB>(
    account_info: &mut AccountInfo,
    evm_bytecode: Bytecode,
) -> Result<(), StateOverrideError<DB::Error>>
where
    DB: Database + DatabaseCommit,
{
    account_info.code = match evm_bytecode {
        Bytecode::Eip7702(eip7702_bytecode) => Some(Bytecode::Eip7702(eip7702_bytecode)),
        // For EVM bytecode we should wrap it into a bytecode that is owned by EVM
        // runtime
        Bytecode::LegacyAnalyzed(bytecode) => {
            let evm_code_hash = keccak256(bytecode.original_byte_slice());
            let evm_metadata = EthereumMetadata::Analyzed(AnalyzedBytecode::new(
                bytecode.original_bytes(),
                evm_code_hash,
            ));
            let bytecode = revm::bytecode::ownable_account::OwnableAccountBytecode::new(
                PRECOMPILE_EVM_RUNTIME,
                evm_metadata.write_to_bytes(),
            );
            let bytecode = Bytecode::OwnableAccount(bytecode);
            Some(bytecode)
        }
        // rWasm is a trusted code, letting pass invalid bytecode without validation can cause
        // memory out of bounds or UB, ownable accounts can only be controlled by deployer, this can
        // be allowed once fully covered with tests and doesn't cause any side effects
        Bytecode::Rwasm(_) | Bytecode::OwnableAccount(_) => {
            return Err(StateOverrideError::InvalidBytecode(BytecodeDecodeError::OwnableAccount(
                OwnableAccountDecodeError::UnsupportedVersion,
            )));
        }
    };
    account_info.code_hash = account_info.code.as_ref().unwrap().hash_slow();
    Ok(())
}
