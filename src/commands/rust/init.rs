use super::constants::{BASIC_TEMPLATE_CARGO_TOML, BASIC_TEMPLATE_LIB_RS};
use crate::{
    commands::common::templates::TemplateManager,
    error::Error,
    utils::fs::{self, create_dir_if_not_exists},
};
use clap::Args;
use std::{path::PathBuf, process::Command};

const DEFAULT_TEMPLATE: &str = "greeting";

#[derive(Args)]
pub struct InitArgs {
    /// Project directory
    #[arg(
        short,
        long,
        help = "Project directory (defaults to current directory)"
    )]
    path: Option<String>,

    /// Template to use
    #[arg(
        short,
        long,
        help = "Template to use from Fluentbase repository",
        default_value = DEFAULT_TEMPLATE
    )]
    template: String,

    /// List available templates
    #[arg(short, long, help = "List all available templates")]
    list: bool,

    /// Force directory creation if it already exists
    #[arg(short, long, help = "Force overwriting existing directory")]
    force: bool,
}

pub(super) fn execute(args: &InitArgs) -> Result<(), Error> {
    let template_manager = TemplateManager::new()?;

    // If --list flag is provided, show templates and exit
    if args.list {
        template_manager.list();
        return Ok(());
    }

    let project_path = if let Some(path) = &args.path {
        let path_buf = PathBuf::from(path);
        create_dir_if_not_exists(&path_buf, args.force)?;
        path_buf
    } else {
        std::env::current_dir()?
    };

    init_project(&project_path, &args.template, &template_manager)
}

fn init_project(
    project_path: &PathBuf,
    template_name: &str,
    template_manager: &TemplateManager,
) -> Result<(), Error> {
    println!(
        "🦀 Initializing new Rust smart contract project with {} template...",
        template_name
    );

    fs::create_dir_if_not_exists(project_path, true)?;

    if template_name == DEFAULT_TEMPLATE {
        create_default_template(project_path)?;
    } else {
        create_from_template(project_path, template_name, template_manager)?;
    }

    init_git_repository(project_path);
    print_next_steps(template_name, project_path);

    Ok(())
}

fn create_default_template(project_path: &PathBuf) -> Result<(), Error> {
    std::fs::write(project_path.join("Cargo.toml"), BASIC_TEMPLATE_CARGO_TOML)
        .map_err(|e| Error::InitializationError(format!("Failed to create Cargo.toml: {}", e)))?;

    std::fs::write(project_path.join("lib.rs"), BASIC_TEMPLATE_LIB_RS)
        .map_err(|e| Error::InitializationError(format!("Failed to create lib.rs: {}", e)))?;

    Ok(())
}

fn create_from_template(
    project_path: &PathBuf,
    template_name: &str,
    template_manager: &TemplateManager,
) -> Result<(), Error> {
    let template = template_manager.get(template_name).ok_or_else(|| {
        Error::InitializationError(format!(
            "Template '{}' not found. Use --list to see available templates",
            template_name
        ))
    })?;

    fs::copy_dir_all(&template.path, project_path)
        .map_err(|e| Error::InitializationError(format!("Failed to copy template: {}", e)))?;

    Ok(())
}

fn init_git_repository(project_path: &PathBuf) {
    if !project_path.join(".git").exists() {
        println!("🔧 Initializing git repository...");
        let _ = Command::new("git")
            .arg("init")
            .current_dir(project_path)
            .output();
    }
}

fn print_next_steps(template_name: &str, project_path: &PathBuf) {
    println!("✅ Project initialized successfully!");
    println!("📂 Project directory: {}", project_path.display());
    println!("📝 Next steps:");
    if template_name == DEFAULT_TEMPLATE {
        println!("  1. Review the generated code in lib.rs");
    } else {
        println!("  1. Review the template files");
    }
    println!("  2. Run 'cargo build' to test the setup");
    println!("  3. Try running the tests with 'cargo test'");
}
