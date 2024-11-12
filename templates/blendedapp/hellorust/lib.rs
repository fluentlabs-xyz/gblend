#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;
extern crate fluentbase_sdk;

use alloy_sol_types::sol_data::Address;
// use alloc::string::String;
use fluentbase_sdk::{
    basic_entrypoint,
    derive::{router, signature, Contract},
    AccountManager, ContextReader, SharedAPI,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::RngCore;
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
        // Either way if we are usinf .block_number()
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

// 
//  pnpm hardhat get-greeting --contract 0xDde063eBe8E85D666AD99f731B4Dbf8C98F29708
// 

// 
// #[cfg(test)]
// mod test {
//     use super::*;
//     use alloy_sol_types::SolCall;
//     use fluentbase_sdk::{codec::Encoder, Address, Bytes, ContractInput, LowLevelSDK};

//     fn with_test_input<T: Into<Bytes>>(input: T, caller: Option<Address>) {
//         let mut contract_input = ContractInput::default();
//         contract_input.contract_caller = caller.unwrap_or_default();
//         LowLevelSDK::with_test_context(contract_input.encode_to_vec(0));
//         let input: Bytes = input.into();
//         LowLevelSDK::with_test_input(input.into());
//     }

//     fn get_output() -> Vec<u8> {
//         LowLevelSDK::get_test_output()
//     }

//     #[test]
//     pub fn test_rand() {
//         let rand = randomCall {}.abi_encode();

//         with_test_input(rand, None);

//         let router = ROUTER::default();
//         router.deploy::<LowLevelSDK>();
//         router.main::<LowLevelSDK>();

//         let output = get_output();

//         let result = randomCall::abi_decode_returns(&output, true).unwrap_or_else(|e| {
//             panic!("Failed to decode output {:?}", e);
//         });


//         assert_eq!(result._0, 123);
//     }
// }