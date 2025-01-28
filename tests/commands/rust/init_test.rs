use crate::common::TestProject;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_init_default_template() -> Result<(), Box<dyn std::error::Error>> {
    let project = TestProject::new()?;

    Command::cargo_bin("gblend")?
        .args(["init", "rust"])
        .current_dir(project.path())
        .assert()
        .success();

    // Check basic file structure
    project.assert_file_exists("Cargo.toml");
    project.assert_file_exists("lib.rs");

    // Verify Cargo.toml contents
    let cargo_toml = project.read_file("Cargo.toml")?;
    assert!(cargo_toml.contains("fluentbase-sdk"));
    assert!(cargo_toml.contains("crate-type = [\"cdylib\", \"staticlib\"]"));

    Ok(())
}

#[test]
fn test_init_with_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let project_dir = temp.child("my-project");

    Command::cargo_bin("gblend")?
        .args([
            "init",
            "rust",
            "--path",
            project_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    project_dir
        .child("Cargo.toml")
        .assert(predicate::path::exists());
    project_dir
        .child("lib.rs")
        .assert(predicate::path::exists());

    // Verify contents
    let cargo_toml = fs::read_to_string(project_dir.child("Cargo.toml").path())?;
    assert!(cargo_toml.contains("fluentbase-sdk"));

    Ok(())
}

#[test]
fn test_init_with_template() -> Result<(), Box<dyn std::error::Error>> {
    let project = TestProject::new()?;

    // Test with greeting template (default)
    Command::cargo_bin("gblend")?
        .args(["init", "rust", "--template", "greeting"])
        .current_dir(project.path())
        .assert()
        .success();

    // Verify file structure and contents
    project.assert_file_exists("Cargo.toml");
    project.assert_file_exists("lib.rs");
    project.assert_file_contains("lib.rs", "struct GREETING")?;

    Ok(())
}

#[test]
fn test_init_fails_with_invalid_template() -> Result<(), Box<dyn std::error::Error>> {
    let project = TestProject::new()?;

    Command::cargo_bin("gblend")?
        .args(["init", "rust", "--template", "non-existent-template"])
        .current_dir(project.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Initializing new Rust smart contract project with non-existent-template template"
        ))
        .stderr(predicate::str::contains(
            "Error: Initialization error: Template 'non-existent-template' not found. Use --list to see available templates"
        ));

    Ok(())
}

#[test]
#[ignore]
fn test_init_creates_valid_project() -> Result<(), Box<dyn std::error::Error>> {
    let project = TestProject::new()?;

    Command::cargo_bin("gblend")?
        .args(["init", "rust"])
        .current_dir(project.path())
        .assert()
        .success();

    // Try to build the project
    Command::new("cargo")
        .arg("build")
        .current_dir(project.path())
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_project_structure_variants() -> Result<(), Box<dyn std::error::Error>> {
    let project = TestProject::new()?;

    // init with default template
    Command::cargo_bin("gblend")?
        .args(["init", "rust"])
        .current_dir(project.path())
        .assert()
        .success();

    // check cargo.toml
    let cargo_toml = project.read_file("Cargo.toml")?;
    assert!(cargo_toml.contains("[package]"));
    assert!(cargo_toml.contains("[dependencies]"));
    assert!(cargo_toml.contains("fluentbase-sdk"));
    assert!(cargo_toml.contains("crate-type = [\"cdylib\", \"staticlib\"]"));

    // check lib.rs
    let lib_rs = project.read_file("lib.rs")?;
    assert!(lib_rs.contains("#![cfg_attr(target_arch = \"wasm32\", no_std)]"));
    assert!(lib_rs.contains("use fluentbase_sdk"));
    assert!(lib_rs.contains("#[derive(Contract)]"));
    assert!(lib_rs.contains("struct GREETING"));

    Ok(())
}
