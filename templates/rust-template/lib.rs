#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;
extern crate fluentbase_sdk;

use fluentbase_sdk::{
    basic_entrypoint,
    derive::{router, signature, Contract},
    AccountManager, ContextReader, SharedAPI,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

#[derive(Contract)]
struct ROUTER<'a, CR: ContextReader, AM: AccountManager> {
    cr: &'a CR,
    am: &'a AM,
}

pub trait RouterAPI {
    fn random<SDK: SharedAPI>(&self) -> u64;
}

#[router(mode = "solidity")]
impl<'a, CR: ContextReader, AM: AccountManager> RouterAPI for ROUTER<'a, CR, AM> {
    #[signature("function random() external view returns (uint256)")]
    fn random<SDK: SharedAPI>(&self) -> u64 {
        // Modify state of a variable & getting state from the VM block_timestamp
        let seed = self.cr.block_timestamp();
        let mut small_rng = SmallRng::seed_from_u64(seed);
        let random_number = small_rng.gen_range(1..15);
        random_number
    }
}

impl<'a, CR: ContextReader, AM: AccountManager> ROUTER<'a, CR, AM> {
    fn deploy<SDK: SharedAPI>(&self) {
        // any custom deployment logic here
    }
}

basic_entrypoint!(ROUTER<'static, fluentbase_sdk::GuestContextReader, fluentbase_sdk::GuestAccountManager>);

