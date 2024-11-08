use super::utils::Tool;
use crate::error::Error;
use clap::Args;
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    process::{Command, Stdio},
    time::Instant,
};

#[derive(Args)]
pub struct BuildArgs {
    /// Use release mode
    #[arg(
        short,
        long,
        help = "Build with optimizations in release mode",
        default_value = "true"
    )]
    release: bool,

    /// Path to project
    #[arg(short, long, help = "Path to project directory", default_value = ".")]
    path: PathBuf,

    /// Create WAT file
    #[arg(long, help = "Create .wat file from .wasm", default_value = "false")]
    wat: bool,

    /// Show build logs
    #[arg(short, long, help = "Show build logs", default_value = "true")]
    verbose: bool,
}

/// Result of the build process
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the generated WASM file
    pub wasm_path: PathBuf,
    /// Size of the generated WASM file in bytes
    pub size: u64,
    /// Optional warnings from the build process
    pub warnings: Option<Vec<String>>,
    /// Optional metadata about the build
    pub metadata: Option<BuildMetadata>,
}

/// Additional metadata about the build
#[derive(Debug)]
pub struct BuildMetadata {
    /// Time taken to build
    pub build_time: std::time::Duration,
    /// Compiler version used
    pub compiler_version: String,
    /// Target architecture
    pub target: String,
    /// Optimization level
    pub optimization_level: String,
}

pub(super) fn execute(args: &BuildArgs) -> Result<(), Error> {
    for t in Tool::all(args.wat) {
        t.ensure()?;
    }
    let result = build_project(&args.path, args.release, args.verbose)?;
    print_build_result(&result);
    Ok(())
}

fn build_project(path: &PathBuf, release: bool, verbose: bool) -> Result<BuildResult, Error> {
    println!("🔨 Building Rust project...");

    // Validate project structure
    validate_project_structure(path)?;

    let start_time = Instant::now();
    ensure_wasm_target()?;

    println!("📦 Running cargo build...");
    run_cargo_build(path, release, verbose)?;

    // Define the expected output location
    let build_mode = if release { "release" } else { "debug" };
    let target_dir = path.join("target/wasm32-unknown-unknown").join(build_mode);

    // Locate the generated .wasm file
    let wasm_file = target_dir
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.path().extension() == Some("wasm".as_ref()))
        .ok_or_else(|| Error::BuildError("No .wasm file found in target directory".to_string()))?;

    // Copy the .wasm file to `lib.wasm`
    let final_wasm_path = path.join("lib.wasm");
    fs::copy(wasm_file.path(), &final_wasm_path)?;

    // Optionally convert to .wat format using wasm2wat
    if Command::new("wasm2wat")
        .arg(&final_wasm_path)
        .output()
        .is_ok()
        && verbose
    {
        println!(
            "💡 Generated .wat file from .wasm at {:?}/lib.wat",
            path.to_str()
        );
    }

    // Gather metadata
    let size = std::fs::metadata(&final_wasm_path)?.len();
    Ok(BuildResult {
        wasm_path: final_wasm_path,
        size,
        warnings: None,
        metadata: Some(BuildMetadata {
            build_time: start_time.elapsed(),
            compiler_version: get_compiler_version()?,
            target: "wasm32-unknown-unknown".to_string(),
            optimization_level: build_mode.to_string(),
        }),
    })
}

fn run_cargo_build(path: &PathBuf, release: bool, verbose: bool) -> Result<(), Error> {
    let mut build_args = vec![
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--no-default-features",
        "--target-dir",
        "./target",
    ];
    if release {
        build_args.push("--release");
    }

    let mut cmd = Command::new("cargo")
        .args(&build_args)
        .env(
            "RUSTFLAGS",
            "-C link-arg=-zstack-size=262144 -C target-feature=+bulk-memory",
        )
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::BuildError(format!("Failed to start build process: {}", e)))?;

    // Stream output line by line in verbose mode
    if verbose {
        if let Some(stdout) = cmd.stdout.take() {
            let stdout_reader = io::BufReader::new(stdout);
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            }
        }

        if let Some(stderr) = cmd.stderr.take() {
            let stderr_reader = io::BufReader::new(stderr);
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    eprintln!("{}", line);
                }
            }
        }
    }

    // Wait for the command to finish and check if it was successful
    let output = cmd
        .wait_with_output()
        .map_err(|e| Error::BuildError(format!("Build process failed: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(Error::BuildError(error_msg.to_string()));
    }

    Ok(())
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

fn check_required_tools(wat: bool) -> Result<(), Error> {
    // Check for cargo
    if Command::new("cargo").arg("--version").output().is_err() {
        return Err(Error::BuildError(
            "Cargo is not installed. Please install Rust and Cargo.".to_string(),
        ));
    }

    // Check for rustup for adding targets if needed
    if Command::new("rustup").arg("--version").output().is_err() {
        return Err(Error::BuildError(
            "Rustup is not installed. Please install Rustup.".to_string(),
        ));
    }

    // Optionally check for wasm2wat if `wat` flag is enabled
    if wat && Command::new("wasm2wat").arg("--version").output().is_err() {
        return Err(Error::BuildError(
            "wasm2wat is not installed. Install it to create .wat files.".to_string(),
        ));
    }

    Ok(())
}
