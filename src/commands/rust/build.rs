use crate::{
    commands::common::types::{BuildMetadata, BuildResult},
    error::Error,
};
use clap::Args;
use std::{path::PathBuf, process::Command, time::Instant};

#[derive(Args)]
pub struct BuildArgs {
    /// Use release mode
    #[arg(long, help = "Build with optimizations in release mode")]
    release: bool,

    /// Path to project
    #[arg(short, long, help = "Path to project directory", default_value = ".")]
    path: PathBuf,
}

pub(super) fn execute(args: &BuildArgs) -> Result<(), Error> {
    let build_result = build_project(&args.path, args.release)?;
    print_build_result(&build_result);
    Ok(())
}

fn build_project(path: &PathBuf, release: bool) -> Result<BuildResult, Error> {
    println!("🔨 Building Rust project...");

    // Validate project structure
    validate_project_structure(path)?;

    let start_time = Instant::now();
    ensure_wasm_target()?;

    println!("📦 Running cargo build...");
    let output = run_cargo_build(path, release)?;
    let compiler_version = get_compiler_version()?;
    let warnings = collect_warnings(&output);

    let build_mode = if release { "release" } else { "debug" };
    let wasm_path = path.join(format!(
        "target/wasm32-unknown-unknown/{}/contract.wasm",
        build_mode
    ));
    let size = std::fs::metadata(&wasm_path)?.len();

    Ok(BuildResult {
        wasm_path,
        size,
        warnings: if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        },
        metadata: Some(BuildMetadata {
            build_time: start_time.elapsed(),
            compiler_version,
            target: "wasm32-unknown-unknown".to_string(),
            optimization_level: build_mode.to_string(),
        }),
    })
}

fn validate_project_structure(path: &PathBuf) -> Result<(), Error> {
    // Check if Cargo.toml exists
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(Error::InvalidProject(
            "Cargo.toml not found. Is this a Rust project?".to_string(),
        ));
    }

    // Check if lib.rs or src/lib.rs exists
    let lib_rs = path.join("lib.rs");
    let src_lib_rs = path.join("src/lib.rs");
    if !lib_rs.exists() && !src_lib_rs.exists() {
        return Err(Error::InvalidProject(
            "Neither lib.rs nor src/lib.rs found. This should be a library project.".to_string(),
        ));
    }

    Ok(())
}

fn ensure_wasm_target() -> Result<(), Error> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|e| Error::BuildError(format!("Failed to check installed targets: {}", e)))?;

    let installed_targets = String::from_utf8_lossy(&output.stdout);

    if !installed_targets.contains("wasm32-unknown-unknown") {
        println!("📦 Adding wasm32-unknown-unknown target...");

        let install_output = Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .output()
            .map_err(|e| Error::BuildError(format!("Failed to add wasm target: {}", e)))?;

        if !install_output.status.success() {
            return Err(Error::BuildError(
                "Failed to install wasm32-unknown-unknown target".to_string(),
            ));
        }
    }

    Ok(())
}

fn run_cargo_build(path: &PathBuf, release: bool) -> Result<std::process::Output, Error> {
    let mut build_args = vec!["build", "--target", "wasm32-unknown-unknown"];
    if release {
        build_args.push("--release");
    }

    let output = Command::new("cargo")
        .args(&build_args)
        .current_dir(path)
        .output()
        .map_err(|e| Error::BuildError(format!("Build failed: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(Error::BuildError(error_msg.to_string()));
    }

    Ok(output)
}

fn get_compiler_version() -> Result<String, Error> {
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| Error::BuildError(e.to_string()))?;

    Ok(String::from_utf8_lossy(&rustc_version.stdout)
        .trim()
        .to_string())
}

fn collect_warnings(output: &std::process::Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| line.contains("warning:"))
        .map(String::from)
        .collect()
}

fn print_build_result(result: &BuildResult) {
    println!("\n✅ Build completed successfully!");
    println!("📦 Output: {}", result.wasm_path.display());
    println!("📊 Size: {} bytes", result.size);

    if let Some(metadata) = &result.metadata {
        println!("⚙️  Compiler: {}", metadata.compiler_version);
        println!("🎯 Target: {}", metadata.target);
        println!("⚡ Optimization: {}", metadata.optimization_level);
        println!("⏱️  Build time: {:?}", metadata.build_time);
    }

    if let Some(warnings) = &result.warnings {
        println!("\n⚠️  Warnings:");
        for warning in warnings {
            println!("  {}", warning);
        }
    }
}
