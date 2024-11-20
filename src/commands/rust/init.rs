use super::{
    constants::{BASIC_TEMPLATE_CARGO_TOML, BASIC_TEMPLATE_LIB_RS},
    template_manager::TemplateManager,
    utils::Tool,
};
use crate::{
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
    for t in Tool::all(false) {
        t.ensure()?;
    }
    let template_manager = TemplateManager::new()?;

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

    init_project(&project_path, args, &template_manager)
}

fn init_project(
    project_path: &PathBuf,
    args: &InitArgs,
    template_manager: &TemplateManager,
) -> Result<(), Error> {
    println!(
        "🦀 Initializing new Rust smart contract project with {} template...",
        args.template
    );

    fs::create_dir_if_not_exists(project_path, true)?;

    if args.template == DEFAULT_TEMPLATE {
        create_default_template(project_path)?;
    } else {
        create_from_template(project_path, args, template_manager)?;
    }

    init_git_repository(project_path);
    print_next_steps(&args.template, project_path);

    Ok(())
}

fn create_default_template(project_path: &PathBuf) -> Result<(), Error> {
    std::fs::write(project_path.join("Cargo.toml"), BASIC_TEMPLATE_CARGO_TOML)
        .map_err(|e| Error::InitializationError(format!("Failed to create Cargo.toml: {}", e)))?;

    std::fs::write(project_path.join("lib.rs"), BASIC_TEMPLATE_LIB_RS)

    std::fs::write(project_path.join("Makefile"), BASIC_TEMPLATE_MAKEFILE)
        .map_err(|e| Error::Initialization(format!("Failed to create Makefile: {}", e)))?;

    std::fs::write(
        project_path.join("rust-toolchain"),
        BASIC_TEMPLATE_RUST_TOOLCHAIN,
    )
    .map_err(|e| Error::Initialization(format!("Failed to create rust-toolchain: {}", e)))?;
    Ok(())
}

fn create_from_template(
    project_path: &PathBuf,
    args: &InitArgs,
    template_manager: &TemplateManager,
) -> Result<(), Error> {
    let template = template_manager.get(&args.template).ok_or_else(|| {
        Error::InitializationError(format!(
            "Template '{}' not found. Use --list to see available templates",
            args.template
        ))
    })?;

    // Initialize project using template manager
    template_manager.init_project(project_path, template)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_create_default_template() -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;
        create_default_template(&temp.path().to_path_buf())?;

        temp.child("Cargo.toml").assert(BASIC_TEMPLATE_CARGO_TOML);
        temp.child("lib.rs").assert(BASIC_TEMPLATE_LIB_RS);

        Ok(())
    }
}
