//! Fluent EVM factory + `NestedEvm` wiring.
//!
//! Mirrors `crates/evm/core/src/evm/tempo.rs` but delegates execution to
//! `fluentbase_revm::RwasmEvm`, which knows how to dispatch
//! `Bytecode::OwnableAccount` (Fluent's deployed-contract wrapper).
//!
//! The `FluentEvmExecutor` wrapper + `EvmFactory` impl below are vendored from
//! `fluentlabs/fluentbase` `crates/node/src/evm.rs` (commit
//! `f7e7e187`) because `RwasmEvm` itself only implements revm's `EvmTr`
//! trait — alloy-evm's `Evm` / `EvmFactory` traits (required by
//! `FoundryEvmFactory`'s supertrait bound) have to be implemented on a wrapper.
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use alloy_evm::{Database, Evm, EvmEnv, EvmFactory, precompiles::PrecompilesMap};
use alloy_primitives::{Address, Bytes};
use fluentbase_revm::{
    DefaultRwasm, RwasmBuilder, RwasmEvm, RwasmFrame, RwasmHaltReason, RwasmHandler,
    RwasmPrecompiles,
    revm::{
        Context, ExecuteEvm, InspectEvm, Inspector, SystemCallEvm,
        context::{
            BlockEnv, CfgEnv, ContextSetters, LocalContextTr, TxEnv,
            result::{EVMError, HaltReason, ResultAndState},
        },
        handler::{
            EthPrecompiles, EvmTr, FrameResult, Handler, PrecompileProvider,
            instructions::EthInstructions,
        },
        inspector::{InspectorHandler, NoOpInspector},
        interpreter::{
            FrameInput, InterpreterResult, SharedMemory, interpreter::EthInterpreter,
            interpreter_action::FrameInit,
        },
        primitives::hardfork::SpecId,
    },
};
use foundry_fork_db::DatabaseError;

use crate::{
    FoundryInspectorExt,
    backend::{DatabaseExt, JournaledState},
    evm::{FoundryEvmFactory, IntoInstructionResult, NestedEvm},
};

/// Type alias for the Fluent EVM context. Structurally identical to
/// `EthEvmContext<DB>` — both are revm `Context` with default journal and `()`
/// chain — so `FoundryContextExt` impls covering the Ethereum context also cover this.
pub type FluentEvmContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB>;

/// `Evm`-trait wrapper around `RwasmEvm`. Vendored from `fluentbase-node`.
#[expect(missing_debug_implementations)]
pub struct FluentEvmExecutor<DB: Database, I, PRECOMPILE = EthPrecompiles> {
    inner: RwasmEvm<
        FluentEvmContext<DB>,
        I,
        EthInstructions<EthInterpreter, FluentEvmContext<DB>>,
        PRECOMPILE,
        RwasmFrame,
    >,
    inspect: bool,
}

impl<DB: Database, I, PRECOMPILE> FluentEvmExecutor<DB, I, PRECOMPILE> {
    pub const fn new(
        evm: RwasmEvm<
            FluentEvmContext<DB>,
            I,
            EthInstructions<EthInterpreter, FluentEvmContext<DB>>,
            PRECOMPILE,
        >,
        inspect: bool,
    ) -> Self {
        Self { inner: evm, inspect }
    }

    pub fn into_inner(
        self,
    ) -> RwasmEvm<
        FluentEvmContext<DB>,
        I,
        EthInstructions<EthInterpreter, FluentEvmContext<DB>>,
        PRECOMPILE,
        RwasmFrame,
    > {
        self.inner
    }

    pub const fn ctx(&self) -> &FluentEvmContext<DB> {
        &self.inner.0.ctx
    }

    pub const fn ctx_mut(&mut self) -> &mut FluentEvmContext<DB> {
        &mut self.inner.0.ctx
    }
}

impl<DB: Database, I, PRECOMPILE> Deref for FluentEvmExecutor<DB, I, PRECOMPILE> {
    type Target = FluentEvmContext<DB>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.ctx()
    }
}

impl<DB: Database, I, PRECOMPILE> DerefMut for FluentEvmExecutor<DB, I, PRECOMPILE> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx_mut()
    }
}

impl<DB, I, PRECOMPILE> Evm for FluentEvmExecutor<DB, I, PRECOMPILE>
where
    DB: Database,
    I: Inspector<FluentEvmContext<DB>>,
    PRECOMPILE: PrecompileProvider<FluentEvmContext<DB>, Output = InterpreterResult>,
{
    type DB = DB;
    type Tx = TxEnv;
    type Error = EVMError<DB::Error>;
    type HaltReason = RwasmHaltReason;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PRECOMPILE;
    type Inspector = I;

    fn block(&self) -> &BlockEnv {
        &self.block
    }

    fn cfg_env(&self) -> &CfgEnv<Self::Spec> {
        &self.cfg
    }

    fn chain_id(&self) -> u64 {
        self.cfg.chain_id
    }

    fn transact_raw(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        if self.inspect { self.inner.inspect_tx(tx) } else { self.inner.transact(tx) }
    }

    fn transact_system_call(
        &mut self,
        caller: Address,
        contract: Address,
        data: Bytes,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        self.inner.system_call_with_caller(caller, contract, data)
    }

    fn db_mut(&mut self) -> &mut Self::DB {
        &mut self.journaled_state.database
    }

    fn finish(self) -> (Self::DB, EvmEnv<Self::Spec>) {
        let Context { block: block_env, cfg: cfg_env, journaled_state, .. } = self.inner.0.ctx;
        (journaled_state.database, EvmEnv { block_env, cfg_env })
    }

    fn set_inspector_enabled(&mut self, enabled: bool) {
        self.inspect = enabled;
    }

    fn enable_inspector(&mut self) {
        self.inspect = true;
    }

    fn disable_inspector(&mut self) {
        self.inspect = false;
    }

    fn precompiles(&self) -> &Self::Precompiles {
        &self.inner.0.precompiles
    }

    fn precompiles_mut(&mut self) -> &mut Self::Precompiles {
        &mut self.inner.0.precompiles
    }

    fn inspector(&self) -> &Self::Inspector {
        &self.inner.0.inspector
    }

    fn inspector_mut(&mut self) -> &mut Self::Inspector {
        &mut self.inner.0.inspector
    }

    fn components(&self) -> (&Self::DB, &Self::Inspector, &Self::Precompiles) {
        (
            &self.inner.0.ctx.journaled_state.database,
            &self.inner.0.inspector,
            &self.inner.0.precompiles,
        )
    }

    fn components_mut(&mut self) -> (&mut Self::DB, &mut Self::Inspector, &mut Self::Precompiles) {
        (
            &mut self.inner.0.ctx.journaled_state.database,
            &mut self.inner.0.inspector,
            &mut self.inner.0.precompiles,
        )
    }
}

/// Foundry-wrapped Fluent EVM type used as `FluentEvmFactory::FoundryEvm`.
pub type FluentRevmEvm<'db, I> =
    FluentEvmExecutor<&'db mut dyn DatabaseExt<FluentEvmFactory>, I, PrecompilesMap>;

/// The underlying revm-level Fluent Evm (what `RwasmHandler` drives directly).
type FluentInnerEvm<'db, I> = RwasmEvm<
    FluentEvmContext<&'db mut dyn DatabaseExt<FluentEvmFactory>>,
    I,
    EthInstructions<EthInterpreter, FluentEvmContext<&'db mut dyn DatabaseExt<FluentEvmFactory>>>,
    PrecompilesMap,
    RwasmFrame,
>;

/// Handler type alias bound to the Fluent inner Evm + Foundry's error shape.
type FluentEvmHandler<'db, I> = RwasmHandler<FluentInnerEvm<'db, I>, EVMError<DatabaseError>>;

/// `EvmFactory` impl — alloy-evm side. Vendored from `fluentbase-node`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FluentEvmFactory;

impl EvmFactory for FluentEvmFactory {
    type Evm<DB: Database, I: Inspector<FluentEvmContext<DB>>> =
        FluentEvmExecutor<DB, I, Self::Precompiles>;
    type Context<DB: Database> = FluentEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = RwasmHaltReason;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        let spec_id = input.cfg_env.spec;
        FluentEvmExecutor {
            inner: Context::rwasm()
                .with_block(input.block_env)
                .with_cfg(input.cfg_env)
                .with_db(db)
                .build_rwasm_with_inspector(NoOpInspector {})
                .with_precompiles(PrecompilesMap::from_static(
                    RwasmPrecompiles::new_with_spec(spec_id).precompiles(),
                )),
            inspect: false,
        }
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        let spec_id = input.cfg_env.spec;
        FluentEvmExecutor {
            inner: Context::rwasm()
                .with_block(input.block_env)
                .with_cfg(input.cfg_env)
                .with_db(db)
                .build_rwasm_with_inspector(inspector)
                .with_precompiles(PrecompilesMap::from_static(
                    RwasmPrecompiles::new_with_spec(spec_id).precompiles(),
                )),
            inspect: true,
        }
    }
}

/// `FoundryEvmFactory` impl — Foundry side.
impl FoundryEvmFactory for FluentEvmFactory {
    type FoundryContext<'db> = FluentEvmContext<&'db mut dyn DatabaseExt<Self>>;
    type FoundryEvm<'db, I: FoundryInspectorExt<Self::FoundryContext<'db>>> = FluentRevmEvm<'db, I>;

    fn create_foundry_evm_with_inspector<'db, I: FoundryInspectorExt<Self::FoundryContext<'db>>>(
        &self,
        db: &'db mut dyn DatabaseExt<Self>,
        evm_env: EvmEnv,
        inspector: I,
    ) -> Self::FoundryEvm<'db, I> {
        let mut executor = Self.create_evm_with_inspector(db, evm_env, inspector);
        executor.cfg.tx_chain_id_check = true;
        executor.inspector().get_networks().inject_precompiles(executor.precompiles_mut());
        executor
    }

    fn create_foundry_nested_evm<'db>(
        &self,
        db: &'db mut dyn DatabaseExt<Self>,
        evm_env: EvmEnv,
        inspector: &'db mut dyn FoundryInspectorExt<Self::FoundryContext<'db>>,
    ) -> Box<dyn NestedEvm<Spec = SpecId, Block = BlockEnv, Tx = TxEnv> + 'db> {
        Box::new(self.create_foundry_evm_with_inspector(db, evm_env, inspector))
    }
}

impl<'db, I> NestedEvm for FluentRevmEvm<'db, I>
where
    I: FoundryInspectorExt<FluentEvmContext<&'db mut dyn DatabaseExt<FluentEvmFactory>>>,
{
    type Spec = SpecId;
    type Block = BlockEnv;
    type Tx = TxEnv;

    fn journal_inner_mut(&mut self) -> &mut JournaledState {
        &mut self.ctx_mut().journaled_state.inner
    }

    fn run_execution(&mut self, frame: FrameInput) -> Result<FrameResult, EVMError<DatabaseError>> {
        let mut handler: FluentEvmHandler<'db, I> = RwasmHandler::default();
        let memory = SharedMemory::new_with_buffer(self.ctx().local.shared_memory_buffer().clone());
        let first_frame_input = FrameInit { depth: 0, memory, frame_input: frame };
        let mut frame_result = handler.inspect_run_exec_loop(&mut self.inner, first_frame_input)?;
        handler.last_frame_result(&mut self.inner, &mut frame_result)?;
        Ok(frame_result)
    }

    fn transact_raw(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ResultAndState<HaltReason>, EVMError<DatabaseError>> {
        self.inner.ctx_mut().set_tx(tx);
        let mut handler: FluentEvmHandler<'db, I> = RwasmHandler::default();
        let result = handler
            .inspect_run(&mut self.inner)?
            .map_haltreason(|h| h.into_instruction_result().into());
        Ok(ResultAndState::new(result, self.ctx().journaled_state.inner.state.clone()))
    }

    fn to_evm_env(&self) -> EvmEnv<Self::Spec, Self::Block> {
        EvmEnv { block_env: self.block.clone(), cfg_env: self.cfg.clone() }
    }
}
