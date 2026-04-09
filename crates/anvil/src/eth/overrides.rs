//! RPC utilities for working with EVM overrides.

use std::collections::BTreeMap;
use alloy_primitives::{Address, B256, U256, keccak256, map::HashMap};
use alloy_rpc_types_eth::{
    BlockOverrides,
    state::{AccountOverride, StateOverride},
};
use fluentbase_evm::{AnalyzedBytecode, EthereumMetadata};
use fluentbase_types::PRECOMPILE_EVM_RUNTIME;
use revm::{
    Database, DatabaseCommit,
    bytecode::{BytecodeDecodeError, ownable_account::OwnableAccountBytecode},
    context::BlockEnv,
    database::{CacheDB, State},
    state::{Account, AccountStatus, Bytecode, EvmStorageSlot},
};

/// Errors that can occur when applying state overrides.
#[derive(Debug, thiserror::Error)]
pub enum StateOverrideError<E> {
    /// Invalid bytecode provided in override.
    #[error(transparent)]
    InvalidBytecode(#[from] BytecodeDecodeError),
    /// Both state and state_diff were provided for an account.
    #[error("Both 'state' and 'stateDiff' fields are set for account {0}")]
    BothStateAndStateDiff(Address),
    /// Database error occurred.
    #[error(transparent)]
    Database(E),
}

/// Helper trait implemented for databases that support overriding block hashes.
pub trait OverrideBlockHashes {
    /// Overrides the given block hashes.
    fn override_block_hashes(&mut self, block_hashes: BTreeMap<u64, B256>);

    /// Applies the given block overrides to the env and updates overridden block hashes.
    fn apply_block_overrides(&mut self, overrides: BlockOverrides, env: &mut BlockEnv)
    where
        Self: Sized,
    {
        apply_block_overrides(overrides, self, env);
    }
}

impl<DB> OverrideBlockHashes for CacheDB<DB> {
    fn override_block_hashes(&mut self, block_hashes: BTreeMap<u64, B256>) {
        self.cache
            .block_hashes
            .extend(block_hashes.into_iter().map(|(num, hash)| (U256::from(num), hash)))
    }
}

impl<DB> OverrideBlockHashes for State<DB> {
    fn override_block_hashes(&mut self, block_hashes: BTreeMap<u64, B256>) {
        self.block_hashes.extend(block_hashes);
    }
}

/// Applies the given block overrides to the env and updates overridden block hashes in the db.
pub fn apply_block_overrides<DB>(overrides: BlockOverrides, db: &mut DB, env: &mut BlockEnv)
where
    DB: OverrideBlockHashes,
{
    let BlockOverrides {
        number,
        difficulty,
        time,
        gas_limit,
        coinbase,
        random,
        base_fee,
        block_hash,
        ..
    } = overrides;

    if let Some(block_hashes) = block_hash {
        db.override_block_hashes(block_hashes);
    }

    if let Some(number) = number {
        env.number = number.saturating_to();
    }
    if let Some(difficulty) = difficulty {
        env.difficulty = difficulty;
    }
    if let Some(time) = time {
        env.timestamp = U256::from(time);
    }
    if let Some(gas_limit) = gas_limit {
        env.gas_limit = gas_limit;
    }
    if let Some(coinbase) = coinbase {
        env.beneficiary = coinbase;
    }
    if let Some(random) = random {
        env.prevrandao = Some(random);
    }
    if let Some(base_fee) = base_fee {
        env.basefee = base_fee.saturating_to();
    }
}

/// Applies the given state overrides (a set of [`AccountOverride`]) to the database.
pub fn apply_state_overrides<DB>(
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

fn try_override_evm_bytecode(bytecode: Bytecode, code_hash: &mut B256) -> Option<Bytecode> {
    match bytecode {
        Bytecode::LegacyAnalyzed(bytecode) => {
            let evm_bytecode = bytecode.original_byte_slice();
            let analyzed = AnalyzedBytecode::new(evm_bytecode.to_vec().into(), *code_hash);
            let metadata = EthereumMetadata::Analyzed(analyzed).write_to_bytes();
            let ownable_bytecode = OwnableAccountBytecode::new(PRECOMPILE_EVM_RUNTIME, metadata);
            *code_hash = keccak256(ownable_bytecode.raw());
            Some(Bytecode::OwnableAccount(ownable_bytecode))
        }
        // Keep trusted wrappers unchanged and reject unsupported raw rWasm here.
        Bytecode::OwnableAccount(_) | Bytecode::Rwasm(_) => None,
        bytecode => Some(bytecode),
    }
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
        // We need to set both the bytecode and the codehash.
        info.code_hash = keccak256(&code);
        let bytecode = Bytecode::new_raw_checked(code)?;
        info.code = try_override_evm_bytecode(bytecode, &mut info.code_hash);
    }
    if let Some(balance) = account_override.balance {
        info.balance = balance;
    }

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
        // Full state replacement: first clear storage, then mark as created.
        (Some(state), None) => {
            db.commit(HashMap::from_iter([(
                account,
                Account {
                    status: AccountStatus::SelfDestructed | AccountStatus::Touched,
                    ..Default::default()
                },
            )]));
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
                    // Use an inverted original value to force dirty write semantics.
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
