use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const FLUENTBASE_GENESIS_VERSION: &str = "v1.2.0";
const PERMISSIVE_RUNTIME_ASSET: &str = "evm-runtime-permissive-v1.2.0.rwasm.gz";
const PERMISSIVE_RUNTIME_URL: &str = "https://github.com/fluentlabs-xyz/fluentbase/releases/download/v1.2.0/evm-runtime-permissive-v1.2.0.rwasm.gz";

fn main() {
    println!("cargo:rerun-if-env-changed=GBLEND_PERMISSIVE_EVM_RUNTIME_GZ");
    println!("cargo:rerun-if-env-changed=GBLEND_PERMISSIVE_EVM_RUNTIME_URL");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let output_path = out_dir.join(PERMISSIVE_RUNTIME_ASSET);

    if let Some(local_path) = env::var_os("GBLEND_PERMISSIVE_EVM_RUNTIME_GZ") {
        let local_path = PathBuf::from(local_path);
        let source = local_path.canonicalize().unwrap_or_else(|err| {
            panic!("failed to resolve permissive EVM runtime path {}: {err}", local_path.display())
        });
        let output = output_path.canonicalize().unwrap_or_else(|_| output_path.clone());
        if source != output {
            fs::copy(&source, &output_path).unwrap_or_else(|err| {
                panic!("failed to copy permissive EVM runtime from {}: {err}", source.display())
            });
        }
    } else if !output_path.exists() {
        let runtime_url = env::var("GBLEND_PERMISSIVE_EVM_RUNTIME_URL")
            .unwrap_or_else(|_| PERMISSIVE_RUNTIME_URL.to_string());
        download_permissive_runtime(&output_path, &runtime_url);
    }

    println!("cargo:rustc-env=GBLEND_FLUENTBASE_GENESIS_VERSION={FLUENTBASE_GENESIS_VERSION}");
    println!("cargo:rustc-env=GBLEND_PERMISSIVE_EVM_RUNTIME_GZ={}", output_path.display());
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
             set GBLEND_PERMISSIVE_EVM_RUNTIME_GZ to a local {PERMISSIVE_RUNTIME_ASSET} to build without network"
        );
    }
}
