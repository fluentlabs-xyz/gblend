use std::env;
use std::fs::{self, File};

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::ParseBoolError;
use dialoguer::Select;
use anyhow::Result;
use anyhow::anyhow;
use ethers::types::Res;

use std::process::Command;

mod deployrs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    //Deploy command 
    

    // Compile command 
    if args.len() == 4 && args[2] == "compile" {
        let file = args[3].trim_end_matches(".rs");
        compile_rust_file(file)?; // Ensure this returns Result
        return Ok(());
    }

    let use_erc20 = args.len() > 1 && args[1] == "--erc20";
    //let blended_app = args.len() > 1  && args[1] == "--blendedapp";

    // if blended_app {
    //     spin_blended_app()?;
    //     return Ok(());
    // }else{
    //     return Err(anyhow!("Not a valid command : try gblend --blendedapp (Rust & Solidity) "));
    // }

    let ascii_art = r#"
    ██████╗ ██████╗ ██╗     ███████╗███╗   ██╗██████╗ 
    ██╔════╝ ██╔══██╗██║     ██╔════╝████╗  ██║██╔══██╗
    ██║  ███╗██████╔╝██║     █████╗  ██╔██╗ ██║██║  ██║
    ██║   ██║██╔══██╗██║     ██╔══╝  ██║╚██╗██║██║  ██║
    ╚██████╔╝██████╔╝███████╗███████╗██║ ╚████║██████╔╝
     ╚═════╝ ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═══╝╚═════╝                                                                                                                                                                           
"#;

    println!("{}", ascii_art);
    println!("Welcome to gblend dev tool 🚀\n");

    let selections = ["Hardhat JavaScript (Solidity & Vyper)", "Hardhat TypeScript (Solidity & Vyper)", "Rust","Blendedapp 🔄" ,"Exit"];
    let selection = Select::new()
        .with_prompt("Choose your setup")
        .default(0)
        .items(&selections[..])
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;  // Convert dialoguer error to io::Error

        match selection {
            0 => spin_js(use_erc20)?,
            1 => spin_ts(use_erc20)?,
            2 => spin_rust()?,
            3 => spin_blended_app()?,
            4 => {
                println!("Exiting program.");
                return Ok(()); // Exit the program gracefully
            },
            _ => unreachable!(),
        };

    Ok(())
}
fn spin_js(use_erc20: bool) -> Result<()>{
    if use_erc20{
        //Contracts
        const ERC20: &str = include_str!("contract-templates/erc20per.vy");
        const ERC20SOL: &str =  include_str!("contract-templates/erc20.sol");
        //Deploy files
        const DEPLOY_ERC20: &str = include_str!("js-template/deployerc20.js");
        const DEPLOY_VYPER: &str = include_str!("js-template/deployvy20.js");
        create_file_with_content("contracts/erc20.sol", ERC20SOL)?;
        create_file_with_content("contracts/erc20per.vy", ERC20)?;
        create_file_with_content("scripts/deployerc20.js", DEPLOY_ERC20)?;
        create_file_with_content("scripts/deployvy20.js", DEPLOY_VYPER)?;
    }else{
        const VYPER_SC: &str = include_str!("contract-templates/hello-v.vy");
        const SOL_SC: &str = include_str!("contract-templates/hello.sol");
        const SOL_SCRIPT: &str  = include_str!("js-template/deploy.js");
        const VYPER_SCRIPT : &str = include_str!("js-template/deployvyper.js");
        create_file_with_content("scripts/deploy.js", SOL_SCRIPT)?;
        create_file_with_content("scripts/deployvyper.js", VYPER_SCRIPT)?;
        create_file_with_content("contracts/hello-v.vy", VYPER_SC)?;
        create_file_with_content("contracts/hello.sol", SOL_SC)?;
    }

    const HARDHAT_CONFIG : &str = include_str!("js-template/hardhat.config.js");
    const PACKAGE_JSON : &str = include_str!("js-template/package.json");
    create_file_with_content("hardhat.config.js", HARDHAT_CONFIG)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;

    Ok(())
}
fn copy_file(source_path: &str, target_path: &str) -> Result<()> {
    // Read the source file
    let mut original_file = File::open(source_path)?;
    let mut contents = String::new();
    original_file.read_to_string(&mut contents)?;

    // Check if target_path has a parent directory and if it needs to be created
    if let Some(target_dir) = Path::new(target_path).parent() {
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }
    }

    // Create and write to the new file at target_path
    let mut new_file = File::create(target_path)?;
    new_file.write_all(contents.as_bytes())?;

    Ok(())
}
fn create_file_with_content(output_path: &str, content: &str) -> Result<()> {
    // Check if the output path has a parent directory and create it if necessary
    if let Some(parent_dir) = Path::new(output_path).parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    // Create and write the content to the new file at the output path
    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

//Create dir 
fn create_directories(output_path: &str) -> Result<()> {
    // Check if the output path has a parent directory and create it if necessary
    if let Some(parent_dir) = Path::new(output_path).parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    // Create the directory at the output path
    fs::create_dir_all(output_path)?;

    Ok(())
}

fn spin_rust() -> Result<()>{

    println!("Creating Rust Project ..");
    const LIB: &str = include_str!("rust-template/lib.rs");
    const CARGO: &str = include_str!("rust-template/Cargo.txt");
    const MAKE_FILE : &str = include_str!("rust-template/Makefile");
    const GIT_IG : &str = include_str!("rust-template/gitignore.txt");
    const DEPLOYOR : &str = include_str!("deployer/deployer.js");
    const PACKAGE_J: &str =  include_str!("deployer/package.json");
    const CARGO_LOCK: &str = include_str!("blendedapp/hellorust/Cargo.lock");

    create_file_with_content("src/lib.rs", LIB)?;
    create_file_with_content("Cargo.toml", CARGO)?;
    create_file_with_content("Makefile", MAKE_FILE)?;
    create_file_with_content(".gitignore", GIT_IG)?;
    create_file_with_content("deployer/deployer.js", DEPLOYOR)?;
    create_file_with_content("deployer/package.json", PACKAGE_J)?;
    create_file_with_content("Cargo.lock", CARGO_LOCK)?;
    println!("Rust template created sucessfully");
    
    Ok(())
}

fn spin_ts(use_erc20: bool) -> Result<()>{
    println!("Creating Typescript Project ..");
    if use_erc20{
        const ERC20: &str = include_str!("contract-templates/erc20per.vy");
        const ERC20SOL: &str =  include_str!("contract-templates/erc20.sol");
        //Deploy files
        const DEPLOY_ERC20: &str = include_str!("ts-template/deployerc20.ts");
        const DEPLOY_VYPER: &str = include_str!("ts-template/deploy20vyper.ts");
        create_file_with_content("contracts/erc20.sol", ERC20SOL)?;
        create_file_with_content("contracts/erc20per.vy", ERC20)?;
        create_file_with_content("scripts/deployerc20.ts", DEPLOY_ERC20)?;
        create_file_with_content("scripts/deployvy20.ts", DEPLOY_VYPER)?;
    }else {
        const VYPER_SC: &str = include_str!("contract-templates/hello-v.vy");
        const SOL_SC: &str = include_str!("contract-templates/hello.sol");
        const SOL_SCRIPT: &str  = include_str!("ts-template/deploy.ts");
        const VYPER_SCRIPT : &str = include_str!("ts-template/deployvyper.ts");
        create_file_with_content("contracts/hello.sol", SOL_SC)?;
        create_file_with_content("contracts/hello-v.vy", VYPER_SC)?;
        create_file_with_content("scripts/deploy.ts", SOL_SCRIPT)?;
        create_file_with_content("scripts/deployvyper.ts", VYPER_SCRIPT)?;
    }
    // Base file for typescript project
    
    const HARDHAT_CONFIG : &str = include_str!("ts-template/hardhat.config.ts");
    const PACKAGE_JSON : &str = include_str!("ts-template/package.json");
    const TS_CONFIG: &str = include_str!("ts-template/tsconfig.json");
    create_file_with_content("hardhat.config.ts", HARDHAT_CONFIG)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;
    create_file_with_content("tsconfig.json", TS_CONFIG)?;
    Ok(())
}

fn spin_blended_app() -> Result<()> {
    println!("Creating blended app ...");

    // Embed the files in the binary using `include_str!`
    const HARDHAT_CONFIG: &str = include_str!("blendedapp/hardhat.config.ts");
    const PACKAGE_JSON: &str = include_str!("blendedapp/package.json");
    const TS_CONFIG: &str = include_str!("blendedapp/tsconfig.json");
    const DEPLOYMENT_SCRIPT: &str = include_str!("blendedapp/deploy/00_deploy_contracts.ts");
    const GREETING_TASK: &str = include_str!("blendedapp/tasks/greeting.ts");
    const MAKE_FILE: &str = include_str!("blendedapp/hellorust/Makefile.txt");
    const LIB: &str = include_str!("blendedapp/hellorust/lib.rs");
    const CARGO_TOML: &str = include_str!("blendedapp/hellorust/cargo.txt");
    const GREETING_SC: &str = include_str!("blendedapp/contracts/GreetingWithWorld.sol");
    const INTERFACE_SC: &str = include_str!("blendedapp/contracts/IFluentGreeting.sol");
    const README: &str = include_str!("blendedapp/README.md");
    const GIT_IGNORE: &str = include_str!("blendedapp/.gitignore");
    const CARGO_LOCK: &str = include_str!("blendedapp/hellorust/Cargo.lock");
    // Create necessary directories and write files
    create_directories("contracts")?;
    create_directories("tasks")?;
    create_directories("deploy")?;
    create_directories("hellorust")?;

    create_file_with_content("hardhat.config.ts", HARDHAT_CONFIG)?;
    create_file_with_content("contracts/GreetingWithWorld.sol", GREETING_SC)?;
    create_file_with_content("contracts/IFluentGreeting.sol", INTERFACE_SC)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;
    create_file_with_content("tsconfig.json", TS_CONFIG)?;
    create_file_with_content("tasks/greeting.ts", GREETING_TASK)?;
    create_file_with_content("deploy/00_deploy_contracts.ts", DEPLOYMENT_SCRIPT)?;
    create_file_with_content("hellorust/Makefile", MAKE_FILE)?;
    create_file_with_content("hellorust/Cargo.toml", CARGO_TOML)?;
    create_file_with_content("hellorust/lib.rs", LIB)?;
    create_file_with_content("README.md", README)?;
    create_file_with_content(".gitignore", GIT_IGNORE)?;
    create_file_with_content("hellorust/Cargo.lock", CARGO_LOCK)?;
    println!("Blended app created successfully!");

    Ok(())

}
fn find_wasm_output(release_dir: &Path) -> Result<PathBuf> {
    // List the files in the release directory and find the .wasm file
    let entries = fs::read_dir(release_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
            return Ok(path);
        }
    }
    Err(anyhow!("No .wasm file found in the release directory"))
}

fn compile_rust_file(file_or_dir: &str) -> Result<()> {
    let path = Path::new(file_or_dir);

    // Debug: Print the provided path
    println!("Checking file or directory: {}", path.display());

    // Check if it's a directory and attempt to find a main.rs or lib.rs, either directly or inside src/
    let rs_file: PathBuf;
    if path.is_dir() {
        // Check in the root of the directory
        let main_file = path.join("main.rs");
        let lib_file = path.join("lib.rs");

        // Check in the src/ subdirectory
        let src_main_file = path.join("src").join("main.rs");
        let src_lib_file = path.join("src").join("lib.rs");

        println!("Looking for main.rs at: {}", main_file.display());
        println!("Looking for lib.rs at: {}", lib_file.display());
        println!("Looking for src/main.rs at: {}", src_main_file.display());
        println!("Looking for src/lib.rs at: {}", src_lib_file.display());

        if main_file.exists() {
            rs_file = main_file;
        } else if lib_file.exists() {
            rs_file = lib_file;
        } else if src_main_file.exists() {
            rs_file = src_main_file;
        } else if src_lib_file.exists() {
            rs_file = src_lib_file;
        } else {
            return Err(anyhow!("No main.rs or lib.rs found in the provided directory or src/ subdirectory: {}", path.display()));
        }
    } else {
        // Treat the path as a specific file
        rs_file = path.to_path_buf();
        println!("Checking if file exists: {}", rs_file.display());
    }

    // Recheck if the file exists, printing the full path for debugging
    if !rs_file.exists() {
        return Err(anyhow!("File does not exist: {}", rs_file.display()));
    }

    println!("File exists: {}", rs_file.display());

    // Find the project directory (go one level up from src/ if the file is in src/)
    let project_dir = if rs_file.starts_with("src") || rs_file.parent().map_or(false, |p| p.ends_with("src")) {
        rs_file.parent().and_then(|p| p.parent()).unwrap_or_else(|| Path::new("."))
    } else {
        rs_file.parent().unwrap_or_else(|| Path::new("."))
    };

    let cargo_file = project_dir.join("Cargo.toml");

    // Check if Cargo.toml exists
    if !cargo_file.exists() {
        return Err(anyhow!("No Cargo.toml found in the project directory: {}", project_dir.display()));
    }

    println!("Compiling Rust file: {} in project {}", rs_file.display(), project_dir.display());

    // Run the cargo build command
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target=wasm32-unknown-unknown")
        .current_dir(project_dir)
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        return Err(anyhow!("Cargo build failed for {}", rs_file.display()));
    }

    // Find the actual .wasm file in the release directory
    let release_dir = project_dir.join("target/wasm32-unknown-unknown/release");
    let wasm_output = find_wasm_output(&release_dir)?;

    println!("WASM binary created at: {}", wasm_output.display());
    Ok(())
}



#[cfg(test)]
pub mod test {
    use std::{fs::{self, File}, io::{self, Read, Write}};
    use anyhow::Result;

    use crate::spin_js;


    #[test]
    fn read_and_write()-> Result<()>{
    let mut original_file = File::open("src/contract-templates/hello.sol")?;

    // Read the contents of the file
    let mut contents = String::new();
    original_file.read_to_string(&mut contents)?;

    // Create the new file and write the contents to it
    fs::create_dir_all("output")?;
    let mut new_file = File::create("output/output-hello.sol")?;
    new_file.write_all(contents.as_bytes())?;
    Ok(())
    }
    #[test]
    fn test_spin_js()-> Result<()>{
        Ok(spin_js(true)?)
    }
}