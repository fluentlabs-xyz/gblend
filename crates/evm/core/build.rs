//! Fetches the Fluent release artifacts that `backend::load_genesis_with_permissive_evm_runtime`
//! embeds with `include_bytes!`: the mainnet genesis and the permissive EVM runtime.
// Build scripts talk to Cargo through stdout, so the workspace-wide `println!` ban does not apply.
#![expect(clippy::disallowed_macros)]
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// The fluentbase release whose genesis and permissive EVM runtime are embedded.
/// Keep in sync with the `fluentbase-*` git dependencies in the workspace `Cargo.toml`.
const FLUENTBASE_RELEASE: &str = "v1.5.1";
const FLUENTBASE_RELEASES_URL: &str =
    "https://github.com/fluentlabs-xyz/fluentbase/releases/download";

/// Overrides the genesis download URL.
const GENESIS_URL_ENV: &str = "GBLEND_GENESIS_URL";
/// Overrides the permissive EVM runtime download URL.
const PERMISSIVE_RUNTIME_URL_ENV: &str = "GBLEND_PERMISSIVE_EVM_RUNTIME_URL";

fn main() {
    println!("cargo:rerun-if-env-changed={GENESIS_URL_ENV}");
    println!("cargo:rerun-if-env-changed={PERMISSIVE_RUNTIME_URL_ENV}");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));

    let genesis = fetch_release_asset(
        &out_dir,
        &format!("genesis-mainnet-{FLUENTBASE_RELEASE}.json.gz"),
        GENESIS_URL_ENV,
    );
    println!("cargo:rustc-env=GBLEND_GENESIS_PATH={}", genesis.display());

    let runtime = fetch_release_asset(
        &out_dir,
        &format!("evm-runtime-permissive-{FLUENTBASE_RELEASE}.rwasm.gz"),
        PERMISSIVE_RUNTIME_URL_ENV,
    );
    println!("cargo:rustc-env=GBLEND_PERMISSIVE_EVM_RUNTIME_PATH={}", runtime.display());
}

/// Downloads `asset` of the pinned fluentbase release into `out_dir` unless it is already
/// there, and returns its path. The asset name carries the release version, so bumping
/// `FLUENTBASE_RELEASE` never reuses a stale file. `url_env` names the environment variable
/// that overrides the download URL.
fn fetch_release_asset(out_dir: &Path, asset: &str, url_env: &str) -> PathBuf {
    let output_path = out_dir.join(asset);
    if output_path.exists() {
        return output_path;
    }

    let url = env::var(url_env)
        .unwrap_or_else(|_| format!("{FLUENTBASE_RELEASES_URL}/{FLUENTBASE_RELEASE}/{asset}"));

    // Download to a temporary path and rename on success, so an interrupted download is
    // never mistaken for a complete asset on the next build.
    let partial_path = output_path.with_extension("part");
    let status = Command::new("curl")
        .args(["--fail", "--location", "--show-error", "--silent", "--retry", "3"])
        .arg("--output")
        .arg(&partial_path)
        .arg(&url)
        .status()
        .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));

    if !status.success() {
        let _ = fs::remove_file(&partial_path);
        panic!("failed to download {asset} from {url}; set {url_env} to an alternate URL");
    }

    fs::rename(&partial_path, &output_path).unwrap_or_else(|err| {
        panic!("failed to move {} to {}: {err}", partial_path.display(), output_path.display())
    });
    output_path
}
