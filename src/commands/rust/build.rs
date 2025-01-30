use super::utils::Tool;
use crate::error::Error;
use clap::Args;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::from_utf8,
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
    #[arg(short, long, help = "Show build logs", default_value = "false")]
    verbose: bool,

    /// Show build logs
    #[arg(short, long, help = "Target dir")]
    target_dir: Option<String>,
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
    let result = build_project(
        &args.path,
        args.release,
        args.verbose,
        args.target_dir.clone(),
    )?;
    print_build_result(&result);
    Ok(())
}

fn build_project(
    path: &PathBuf,
    release: bool,
    verbose: bool,
    target_dir: Option<String>,
) -> Result<BuildResult, Error> {
    println!("🔨 Building Rust project...");

    // Validate project structure
    validate_project_structure(path)?;

    let start_time = Instant::now();
    ensure_wasm_target()?;

    println!("📦 Running cargo build...");
    run_cargo_build(path, release, verbose, target_dir.clone())?;

    let project_name = {
        let result = Command::new("cargo")
            .arg("read-manifest")
            .output()
            .map_err(|e| {
                if verbose {
                    println!("Failed to read manifest: {:?}", e);
                }
                Error::Build("Failed to read manifest".to_string())
            })?;
        let utf8_string = from_utf8(&result.stdout).map_err(|e| {
            if verbose {
                println!("Failed to decode UTF-8 output: {:?}", e);
            }
            Error::Build("Failed to get target directory".to_string())
        })?;
        let json_value = json::parse(utf8_string).unwrap();
        json_value["name"].to_string()
    };

    if verbose {
        println!(" ~ detected project name: {}", project_name)
    }

    let (target_dir, library_name) = if target_dir.is_none() {
        let result = Command::new("cargo")
            .arg("metadata")
            .arg("--no-deps")
            .output()
            .map_err(|e| {
                if verbose {
                    println!("Failed to get target directory: {:?}", e);
                }
                Error::Build("Failed to get target directory".to_string())
            })?;
        let utf8_string = from_utf8(&result.stdout).map_err(|e| {
            if verbose {
                println!("Failed to decode UTF-8 output: {:?}", e);
            }
            Error::Build("Failed to get target directory".to_string())
        })?;
        let json_value = json::parse(utf8_string).expect("can't parse json with manifest");
        let package = json_value["packages"]
            .members()
            .find(|package| package["name"].to_string() == project_name)
            .expect("can't find package in the manifest");
        let library_name = package["targets"].members().find_map(|target| {
            target["kind"]
                .members()
                .find(|lib| lib.to_string() == "cdylib")?;
            Some(format!("{}.wasm", target["name"].to_string()))
        });
        let target_directory = json_value["target_directory"].to_string();
        (target_directory, library_name)
    } else {
        (target_dir.unwrap(), None)
    };

    let library_name =
        library_name.unwrap_or_else(|| format!("{}.wasm", project_name.replace("-", "_")));

    if verbose {
        println!(" ~ found target dir: {}", target_dir)
    }

    // Define the expected output location
    let build_mode = if release { "release" } else { "debug" };
    let target_dir = path
        .join(target_dir)
        .join("wasm32-unknown-unknown")
        .join(build_mode);

    // Locate the generated .wasm file
    let wasm_file = PathBuf::from(target_dir.to_str().unwrap()).join(library_name);

    if verbose {
        println!(
            " ~ detected wasm binary path: {}",
            wasm_file.to_str().unwrap_or_default()
        )
    }

    // Copy the .wasm file to `lib.wasm`
    let final_wasm_path = path.join("lib.wasm");
    fs::copy(wasm_file, &final_wasm_path)?;

    // Optionally convert to .wat format
    let wasm2wat = Command::new("wasm2wat").arg(&final_wasm_path).output();
    if wasm2wat.is_ok() {
        let wasm2wat = wasm2wat.unwrap();
        let final_wast_path = path.join("lib.wat");
        fs::write(final_wast_path, from_utf8(&wasm2wat.stdout).unwrap())?;
        if verbose {
            println!(
                "💡 Generated .wat file from .wasm at {:?}/lib.wat",
                path.to_str()
            );
        }
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

fn run_cargo_build(
    path: &PathBuf,
    release: bool,
    verbose: bool,
    target_dir: Option<String>,
) -> Result<(), Error> {
    let mut build_args = vec![
        "build".to_string(),
        "--target".to_string(),
        "wasm32-unknown-unknown".to_string(),
        "--no-default-features".to_string(),
    ];
    if let Some(target_dir) = target_dir {
        build_args.push("--target-dir".to_string());
        build_args.push(target_dir);
    }
    if release {
        build_args.push("--release".to_string());
    }

    if verbose {
        println!("~ running command: {}", build_args.join(" "));
    }

    let mut cmd = Command::new("cargo");
    cmd.args(&build_args)
        .env(
            "RUSTFLAGS",
            "-C link-arg=-zstack-size=262144 -C target-feature=+bulk-memory",
        )
        .current_dir(path);
    // if verbose {
    //     cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    // }
    let cmd = cmd
        .spawn()
        .map_err(|e| Error::Build(format!("Failed to start build process: {}", e)))?;

    // Stream output line by line in verbose mode
    // if verbose {
    //     if let Some(stdout) = cmd.stdout.take() {
    //         let stdout_reader = io::BufReader::new(stdout);
    //         for line in stdout_reader.lines().map_while(Result::ok) {
    //             println!("{}", line);
    //         }
    //     }
    //
    //     if let Some(stderr) = cmd.stderr.take() {
    //         let stderr_reader = io::BufReader::new(stderr);
    //         for line in stderr_reader.lines().map_while(Result::ok) {
    //             eprintln!("{}", line);
    //         }
    //     }
    // }

    // Wait for the command to finish and check if it was successful
    let output = cmd
        .wait_with_output()
        .map_err(|e| Error::Build(format!("Build process failed: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Build(error_msg.to_string()));
    }

    Ok(())
}

fn validate_project_structure(path: &Path) -> Result<(), Error> {
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
        .map_err(|e| Error::Build(format!("Failed to check installed targets: {}", e)))?;

    let installed_targets = String::from_utf8_lossy(&output.stdout);

    if !installed_targets.contains("wasm32-unknown-unknown") {
        println!("📦 Adding wasm32-unknown-unknown target...");

        let install_output = Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .output()
            .map_err(|e| Error::Build(format!("Failed to add wasm target: {}", e)))?;

        if !install_output.status.success() {
            return Err(Error::Build(
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
        .map_err(|e| Error::Build(e.to_string()))?;

    Ok(String::from_utf8_lossy(&rustc_version.stdout)
        .trim()
        .to_string())
}

fn print_build_result(result: &BuildResult) {
    println!("\n✅ Build completed successfully!");
    println!("📦 Output: {}", result.wasm_path.display());
    println!("📊 Size: {} bytes", result.size);

    if let Some(metadata) = &result.metadata {
        println!("⚙️ Compiler: {}", metadata.compiler_version);
        println!("🎯 Target: {}", metadata.target);
        println!("⚡ Optimization: {}", metadata.optimization_level);
        println!("⏱️ Build time: {:?}", metadata.build_time);
    }

    if let Some(warnings) = &result.warnings {
        println!("\n⚠️ Warnings:");
        for warning in warnings {
            println!("  {}", warning);
        }
    }
}
