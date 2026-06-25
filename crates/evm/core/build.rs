use std::{env, path::Path, process::Command};

const FLUENTBASE_GENESIS_VERSION: &str = "v1.3.0";
const PERMISSIVE_RUNTIME_ASSET: &str = "evm-runtime-permissive-v1.3.0.rwasm.gz";
const PERMISSIVE_RUNTIME_URL: &str = "https://github.com/fluentlabs-xyz/fluentbase/releases/download/v1.3.0/evm-runtime-permissive-v1.3.0.rwasm.gz";

fn main() {
    println!("cargo:rerun-if-env-changed=GBLEND_PERMISSIVE_EVM_RUNTIME_URL");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by Cargo");
    let output_path = Path::new(&out_dir).join(PERMISSIVE_RUNTIME_ASSET);

    if !output_path.exists() {
        let runtime_url = env::var("GBLEND_PERMISSIVE_EVM_RUNTIME_URL")
            .unwrap_or_else(|_| PERMISSIVE_RUNTIME_URL.to_string());
        download_permissive_runtime(&output_path, &runtime_url);
    }

    println!("cargo:rustc-env=GBLEND_FLUENTBASE_GENESIS_VERSION={FLUENTBASE_GENESIS_VERSION}");
}

fn download_permissive_runtime(output_path: &Path, runtime_url: &str) {
    let status = Command::new("curl")
        .args(["--fail", "--location", "--show-error", "--silent", "--retry", "3"])
        .arg("--output")
        .arg(output_path)
        .arg(runtime_url)
        .status()
        .unwrap_or_else(|err| panic!("failed to run curl for {runtime_url}: {err}"));

    if !status.success() {
        panic!(
            "failed to download permissive EVM runtime from {runtime_url}; \
             set GBLEND_PERMISSIVE_EVM_RUNTIME_URL to an alternate {PERMISSIVE_RUNTIME_ASSET} URL"
        );
    }
}
