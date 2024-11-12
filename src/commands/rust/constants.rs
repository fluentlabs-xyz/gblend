pub const BASIC_TEMPLATE_CARGO_TOML: &str = r#"[package]
name = "fluentbase-example-greeting"
version = "0.1.0"
edition = "2021"

[dependencies]
fluentbase-sdk = { git = "https://github.com/fluentlabs-xyz/fluentbase", branch = "devel", default-features = false }

[dev-dependencies]
hex-literal = "0.4.1"
hex = "0.4.3"

[lib]
crate-type = ["cdylib", "staticlib"]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "fluentbase-sdk/std"
]
"#;

pub const BASIC_TEMPLATE_LIB_RS: &str = r#"#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate fluentbase_sdk;
use fluentbase_sdk::{basic_entrypoint, derive::Contract, SharedAPI};

#[derive(Contract)]
struct GREETING<SDK> {
    sdk: SDK,
}

impl<SDK: SharedAPI> GREETING<SDK> {
    fn deploy(&mut self) {
        // any custom deployment logic here
    }

    fn main(&mut self) {
        // write "Hello, World" message into output
        self.sdk.write("Hello, World".as_bytes());
    }
}

basic_entrypoint!(GREETING);

#[cfg(test)]
mod tests {
    use super::*;
    use fluentbase_sdk::{journal::JournalState, runtime::TestingContext};

    #[test]
    fn test_contract_works() {
        let native_sdk = TestingContext::empty().with_input("Hello, World");
        let sdk = JournalState::empty(native_sdk.clone());
        let mut greeting = GREETING::new(sdk);
        greeting.deploy();
        greeting.main();
        let output = native_sdk.take_output();
        assert_eq!(&output, "Hello, World".as_bytes());
    }
}
"#;
