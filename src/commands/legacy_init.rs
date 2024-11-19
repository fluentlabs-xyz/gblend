use anyhow::Result;
use dialoguer::Select;
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

const ASCII_ART: &str = r#"
██████╗ ██████╗ ██╗     ███████╗███╗   ██╗██████╗ 
██╔════╝ ██╔══██╗██║     ██╔════╝████╗  ██║██╔══██╗
██║  ███╗██████╔╝██║     █████╗  ██╔██╗ ██║██║  ██║
██║   ██║██╔══██╗██║     ██╔══╝  ██║╚██╗██║██║  ██║
╚██████╔╝██████╔╝███████╗███████╗██║ ╚████║██████╔╝
 ╚═════╝ ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═══╝╚═════╝ 
"#;

pub async fn legacy_init() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let use_erc20 = args.len() > 1 && args[1] == "--erc20";

    println!("{}", ASCII_ART);
    println!("Welcome to gblend dev tool 🚀\n");

    let selections = [
        "Hardhat JavaScript (Solidity & Vyper)",
        "Hardhat TypeScript (Solidity & Vyper)",
        "Rust",
        "Blendedapp 🔄",
        "Exit",
    ];
    let selection = Select::new()
        .with_prompt("Choose your setup")
        .default(0)
        .items(&selections[..])
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Convert dialoguer error to io::Error

    match selection {
        0 => spin_js(use_erc20)?,
        1 => spin_ts(use_erc20)?,
        2 => spin_rust()?,
        3 => spin_blended_app()?,
        4 => {
            println!("Exiting program.");
            return Ok(()); // Exit the program gracefully
        }
        _ => unreachable!(),
    };
    Ok(())
}

fn spin_js(use_erc20: bool) -> Result<()> {
    if use_erc20 {
        //Contracts
        const ERC20: &str = include_str!("../../templates/contract-templates/erc20per.vy");
        const ERC20SOL: &str = include_str!("../../templates/contract-templates/erc20.sol");
        //Deploy files
        const DEPLOY_ERC20: &str = include_str!("../../templates/js-template/deployerc20.js");
        const DEPLOY_VYPER: &str = include_str!("../../templates/js-template/deployvy20.js");
        create_file_with_content("contracts/erc20.sol", ERC20SOL)?;
        create_file_with_content("contracts/erc20per.vy", ERC20)?;
        create_file_with_content("scripts/deployerc20.js", DEPLOY_ERC20)?;
        create_file_with_content("scripts/deployvy20.js", DEPLOY_VYPER)?;
    } else {
        const VYPER_SC: &str = include_str!("../../templates/contract-templates/hello-v.vy");
        const SOL_SC: &str = include_str!("../../templates/contract-templates/hello.sol");
        const SOL_SCRIPT: &str = include_str!("../../templates/js-template/deploy-solidity.js");
        const VYPER_SCRIPT: &str = include_str!("../../templates/js-template/deploy-vyper.js");
        create_file_with_content("scripts/deploy-solidity.js", SOL_SCRIPT)?;
        create_file_with_content("scripts/deploy-vyper.js", VYPER_SCRIPT)?;
        create_file_with_content("contracts/hello-v.vy", VYPER_SC)?;
        create_file_with_content("contracts/hello.sol", SOL_SC)?;
    }

    const HARDHAT_CONFIG: &str = include_str!("../../templates/js-template/hardhat.config.js");
    const PACKAGE_JSON: &str = include_str!("../../templates/js-template/package.json");
    create_file_with_content("hardhat.config.js", HARDHAT_CONFIG)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;

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

fn spin_rust() -> Result<()> {
    println!("Creating Rust Project ..");
    const LIB: &str = include_str!("../../templates/rust-template/lib.rs");
    const CARGO: &str = include_str!("../../templates/rust-template/Cargo.toml");

    const GIT_IG: &str = include_str!("../../templates/rust-template/gitignore.txt");

    create_file_with_content("lib.rs", LIB)?;
    create_file_with_content("Cargo.toml", CARGO)?;
    create_file_with_content(".gitignore", GIT_IG)?;

    println!("Rust template created sucessfully");

    Ok(())
}

fn spin_ts(use_erc20: bool) -> Result<()> {
    println!("Creating Typescript Project ..");
    if use_erc20 {
        const ERC20: &str = include_str!("../../templates/contract-templates/erc20per.vy");
        const ERC20SOL: &str = include_str!("../../templates/contract-templates/erc20.sol");
        //Deploy files
        const DEPLOY_ERC20: &str = include_str!("../../templates/ts-template/deployerc20.ts");
        const DEPLOY_VYPER: &str = include_str!("../../templates/ts-template/deploy20vyper.ts");
        create_file_with_content("contracts/erc20.sol", ERC20SOL)?;
        create_file_with_content("contracts/erc20per.vy", ERC20)?;
        create_file_with_content("scripts/deployerc20.ts", DEPLOY_ERC20)?;
        create_file_with_content("scripts/deployvy20.ts", DEPLOY_VYPER)?;
    } else {
        const VYPER_SC: &str = include_str!("../../templates/contract-templates/hello-v.vy");
        const SOL_SC: &str = include_str!("../../templates/contract-templates/hello.sol");
        const SOL_SCRIPT: &str = include_str!("../../templates/ts-template/deploy.ts");
        const VYPER_SCRIPT: &str = include_str!("../../templates/ts-template/deployvyper.ts");
        create_file_with_content("contracts/hello.sol", SOL_SC)?;
        create_file_with_content("contracts/hello-v.vy", VYPER_SC)?;
        create_file_with_content("scripts/deploy.ts", SOL_SCRIPT)?;
        create_file_with_content("scripts/deployvyper.ts", VYPER_SCRIPT)?;
    }
    // Base file for typescript project

    const HARDHAT_CONFIG: &str = include_str!("../../templates/ts-template/hardhat.config.ts");
    const PACKAGE_JSON: &str = include_str!("../../templates/ts-template/package.json");
    const TS_CONFIG: &str = include_str!("../../templates/ts-template/tsconfig.json");
    create_file_with_content("hardhat.config.ts", HARDHAT_CONFIG)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;
    create_file_with_content("tsconfig.json", TS_CONFIG)?;
    Ok(())
}

fn spin_blended_app() -> Result<()> {
    println!("Creating blended app ...");

    // Embed the files in the binary using `include_str!`
    const HARDHAT_CONFIG: &str = include_str!("../../templates/blendedapp/hardhat.config.ts");
    const PACKAGE_JSON: &str = include_str!("../../templates/blendedapp/package.json");
    const TS_CONFIG: &str = include_str!("../../templates/blendedapp/tsconfig.json");
    const DEPLOYMENT_SCRIPT: &str =
        include_str!("../../templates/blendedapp/deploy/00_deploy_contracts.ts");
    const GREETING_TASK: &str = include_str!("../../templates/blendedapp/tasks/greeting.ts");
    const LIB: &str = include_str!("../../templates/blendedapp/greeting/src/lib.rs");
    const CARGO_TOML: &str = include_str!("../../templates/blendedapp/greeting/Cargo.toml");
    const GREETING_SC: &str =
        include_str!("../../templates/blendedapp/contracts/GreetingWithWorld.sol");
    const INTERFACE_SC: &str =
        include_str!("../../templates/blendedapp/contracts/IFluentGreeting.sol");
    const README: &str = include_str!("../../templates/blendedapp/README.md");
    const GIT_IGNORE: &str = include_str!("../../templates/blendedapp/.gitignore");

    const ENV: &str = include_str!("../../templates/blendedapp/.env");
    // Create necessary directories and write files
    create_directories("contracts")?;
    create_directories("tasks")?;
    create_directories("deploy")?;
    create_directories("greeting")?;

    create_file_with_content("hardhat.config.ts", HARDHAT_CONFIG)?;
    create_file_with_content("contracts/GreetingWithWorld.sol", GREETING_SC)?;
    create_file_with_content("contracts/IFluentGreeting.sol", INTERFACE_SC)?;
    create_file_with_content("package.json", PACKAGE_JSON)?;
    create_file_with_content("tsconfig.json", TS_CONFIG)?;
    create_file_with_content("tasks/greeting.ts", GREETING_TASK)?;
    create_file_with_content("deploy/00_deploy_contracts.ts", DEPLOYMENT_SCRIPT)?;

    create_file_with_content("greeting/Cargo.toml", CARGO_TOML)?;
    create_file_with_content("greeting/src/lib.rs", LIB)?;
    create_file_with_content("README.md", README)?;
    create_file_with_content(".env", ENV)?;
    create_file_with_content(".gitignore", GIT_IGNORE)?;

    println!("Blended app created successfully!");

    Ok(())
}
