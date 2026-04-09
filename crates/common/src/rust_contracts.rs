use eyre::Result;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct RustProjectInfo {
    pub path: PathBuf,
    pub package_name: String,
    pub sdk_version: Option<String>,
}

impl RustProjectInfo {
    /// Get the artifact name for this Rust contract
    /// Format: "{package_name}.wasm"
    /// Examples: "erc20" -> "erc20.wasm", "power-calculator" -> "power-calculator.wasm"
    pub fn artifact_name(&self) -> String {
        format!("{}.wasm", self.package_name)
    }

    /// Get the artifact directory path
    /// Examples: "out/erc20.wasm/", "out/power-calculator.wasm/"
    pub fn artifact_dir(&self, artifacts_root: &Path) -> PathBuf {
        artifacts_root.join(self.artifact_name())
    }

    /// Get the foundry.json artifact path
    /// Examples: "out/erc20.wasm/foundry.json"
    pub fn foundry_artifact_path(&self, artifacts_root: &Path) -> PathBuf {
        self.artifact_dir(artifacts_root).join("foundry.json")
    }
}

#[derive(Debug, Clone)]
pub struct RustContractsRegistry {
    // Key: package_name from Cargo.toml (always kebab-case: "erc20", "power-calculator")
    contracts: BTreeMap<String, RustProjectInfo>,
}

impl RustContractsRegistry {
    /// Create a new registry by scanning a directory for Rust projects
    /// Recursively walks the directory and collects all Cargo.toml files
    ///
    /// Arguments:
    /// - `contract_dir`: Directory to scan (typically `project.paths.sources`)
    /// - `project_root`: Optional project root for gitignore support
    pub fn new(contract_dir: &Path, project_root: Option<&Path>) -> Result<Self> {
        let mut contracts = BTreeMap::new();

        // Return empty registry if source root doesn't exist
        if !contract_dir.exists() || !contract_dir.is_dir() {
            return Ok(Self { contracts });
        }

        // Load gitignore rules if project root is provided
        let gitignore = project_root.and_then(|root| load_gitignore(root).ok()).unwrap_or_default();

        // Walk through directory recursively
        for entry in WalkDir::new(contract_dir)
            .into_iter()
            .filter_entry(|e| {
                // Only filter directories, not files
                if !e.file_type().is_dir() {
                    return true;
                }

                // Check if this directory should be skipped
                !should_skip_directory(e.path(), &gitignore, project_root)
            })
            .filter_map(Result::ok)
        {
            let path = entry.path();

            // Look for Cargo.toml files
            if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
                // Get the directory containing Cargo.toml (project root)
                if let Some(project_dir) = path.parent() {
                    match Self::read_project_info(project_dir) {
                        Ok(info) => {
                            contracts.insert(info.package_name.clone(), info);
                        }
                        Err(e) => {
                            // Log error but continue scanning
                            sh_warn!("Warning: Failed to read Cargo.toml at {path:?}: {e}")?;
                        }
                    }
                }
            }
        }

        Ok(Self { contracts })
    }

    /// Read project info from a directory containing Cargo.toml
    fn read_project_info(project_dir: &Path) -> Result<RustProjectInfo> {
        let cargo_toml_path = project_dir.join("Cargo.toml");
        let cargo_content = fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_content)?;

        // Extract package name (always kebab-case in Cargo.toml)
        let package_name = cargo_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| eyre::eyre!("Package name not found in Cargo.toml"))?
            .to_string();

        // Extract SDK version with multiple strategies
        let sdk_version = Self::extract_sdk_version(&cargo_toml);

        let canonical_path =
            project_dir.canonicalize().unwrap_or_else(|_| project_dir.to_path_buf());

        Ok(RustProjectInfo { path: canonical_path, package_name, sdk_version })
    }

    /// Extract fluentbase-sdk version from dependencies
    /// Supports multiple formats:
    /// - fluentbase-sdk = "0.1.0"
    /// - fluentbase-sdk = { version = "0.1.0" }
    /// - fluentbase-sdk = { tag = "v0.1.0" }
    fn extract_sdk_version(cargo_toml: &toml::Value) -> Option<String> {
        let deps = cargo_toml.get("dependencies")?;
        let sdk = deps.get("fluentbase-sdk")?;

        // Try direct string version first
        if let Some(version_str) = sdk.as_str() {
            return Some(version_str.to_string());
        }

        // Try table format
        if let Some(sdk_table) = sdk.as_table() {
            // Try "version" field
            if let Some(version) = sdk_table.get("version").and_then(|v| v.as_str()) {
                return Some(version.to_string());
            }
            // Try "tag" field
            if let Some(tag) = sdk_table.get("tag").and_then(|t| t.as_str()) {
                return Some(tag.to_string());
            }
        }

        None
    }

    /// Get RustProjectInfo by contract name
    ///
    /// Accepts any of these formats:
    /// - Package name: "erc20", "power-calculator"
    /// - With extension: "erc20.wasm", "power-calculator.wasm"
    /// - Case insensitive: "ERC20", "POWER-CALCULATOR"
    pub fn get(&self, name: &str) -> Option<&RustProjectInfo> {
        let normalized = normalize_contract_name(name);
        self.contracts.get(&normalized)
    }

    /// Check if registry contains a contract with given name
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Get the artifact directory for a contract name
    pub fn artifact_dir(&self, name: &str, artifacts_root: &Path) -> Option<PathBuf> {
        self.get(name).map(|info| info.artifact_dir(artifacts_root))
    }

    /// Get the foundry.json path for a contract name
    pub fn foundry_artifact_path(&self, name: &str, artifacts_root: &Path) -> Option<PathBuf> {
        self.get(name).map(|info| info.foundry_artifact_path(artifacts_root))
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    /// Get number of contracts in registry
    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    /// Get all package names
    pub fn package_names(&self) -> impl Iterator<Item = &String> {
        self.contracts.keys()
    }

    /// Iterate over all contracts
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RustProjectInfo)> {
        self.contracts.iter()
    }
}

/// Normalize contract name to package name format
///
/// Rules:
/// 1. Remove .wasm extension if present
/// 2. Convert to lowercase (for case-insensitive matching)
///
/// Package names in Cargo.toml are always kebab-case or lowercase.
/// We don't convert between naming conventions, just normalize case.
///
/// Examples:
/// - "erc20.wasm" -> "erc20"
/// - "erc20" -> "erc20"
/// - "ERC20" -> "erc20"
/// - "power-calculator.wasm" -> "power-calculator"
/// - "POWER-CALCULATOR" -> "power-calculator"
pub fn normalize_contract_name(name: &str) -> String {
    // Remove .wasm extension if present
    let base_name = name.strip_suffix(".wasm").unwrap_or(name);

    // Convert to lowercase for case-insensitive matching
    base_name.to_lowercase()
}

/// Check if directory should be skipped during scanning
fn should_skip_directory(
    dir: &Path,
    gitignore: &GitignoreRules,
    project_root: Option<&Path>,
) -> bool {
    // Standard directories to skip
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        if matches!(
            name,
            "target" | ".git" | "node_modules" | ".idea" | ".vscode" | "dist" | "build" | "lib"
        ) {
            return true;
        }

        // Skip hidden directories
        if name.starts_with('.') {
            return true;
        }
    }

    // Check gitignore rules
    if let Some(root) = project_root
        && let Ok(relative) = dir.strip_prefix(root)
    {
        return gitignore.should_ignore(relative);
    }

    false
}

#[derive(Default)]
struct GitignoreRules {
    patterns: Vec<String>,
}

impl GitignoreRules {
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        self.patterns.iter().any(|pattern| {
            if let Some(stripped) = pattern.strip_prefix('/') {
                path_str.starts_with(stripped)
            } else if pattern.ends_with('/') {
                path.is_dir() && path_str.contains(&pattern[..pattern.len() - 1])
            } else {
                path.components().any(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .is_some_and(|name| name == pattern || matches_simple_glob(name, pattern))
                })
            }
        })
    }
}

/// Load gitignore rules from file
fn load_gitignore(project_root: &Path) -> eyre::Result<GitignoreRules> {
    let gitignore_path = project_root.join(".gitignore");

    if !gitignore_path.exists() {
        return Ok(GitignoreRules::default());
    }

    let content = std::fs::read_to_string(gitignore_path)?;
    let patterns: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();

    Ok(GitignoreRules { patterns })
}

/// Simple glob pattern matching
fn matches_simple_glob(text: &str, pattern: &str) -> bool {
    match (pattern.starts_with('*'), pattern.ends_with('*')) {
        (true, true) => {
            let middle = &pattern[1..pattern.len() - 1];
            text.contains(middle)
        }
        (true, false) => {
            let suffix = &pattern[1..];
            text.ends_with(suffix)
        }
        (false, true) => {
            let prefix = &pattern[..pattern.len() - 1];
            text.starts_with(prefix)
        }
        (false, false) => text == pattern,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_contract_name() {
        // With .wasm extension
        assert_eq!(normalize_contract_name("erc20.wasm"), "erc20");
        assert_eq!(normalize_contract_name("power-calculator.wasm"), "power-calculator");

        // Without extension
        assert_eq!(normalize_contract_name("erc20"), "erc20");
        assert_eq!(normalize_contract_name("power-calculator"), "power-calculator");

        // Case variations (user might type differently)
        assert_eq!(normalize_contract_name("ERC20"), "erc20");
        assert_eq!(normalize_contract_name("ERC20.wasm"), "erc20");
        assert_eq!(normalize_contract_name("POWER-CALCULATOR"), "power-calculator");
        assert_eq!(normalize_contract_name("Power-Calculator.wasm"), "power-calculator");
    }

    #[test]
    fn test_sdk_version_extraction() {
        // Test string format
        let toml_str = r#"
[dependencies]
fluentbase-sdk = "0.1.0"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        assert_eq!(RustContractsRegistry::extract_sdk_version(&value), Some("0.1.0".to_string()));

        // Test version in table
        let toml_table = r#"
[dependencies]
fluentbase-sdk = { version = "0.2.0" }
"#;
        let value: toml::Value = toml::from_str(toml_table).unwrap();
        assert_eq!(RustContractsRegistry::extract_sdk_version(&value), Some("0.2.0".to_string()));

        // Test tag in table
        let toml_tag = r#"
[dependencies]
fluentbase-sdk = { tag = "v0.3.0" }
"#;
        let value: toml::Value = toml::from_str(toml_tag).unwrap();
        assert_eq!(RustContractsRegistry::extract_sdk_version(&value), Some("v0.3.0".to_string()));
    }

    #[test]
    fn test_empty_registry() {
        let temp_dir = TempDir::new().unwrap();
        let registry = RustContractsRegistry::new(temp_dir.path(), None).unwrap();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_blended_structure() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create power-calculator project
        let power_calc_dir = src_dir.join("power-calculator");
        fs::create_dir_all(power_calc_dir.join("src")).unwrap();
        fs::write(
            power_calc_dir.join("Cargo.toml"),
            r#"[package]
name = "power-calculator"
version = "0.1.0"

[dependencies]
fluentbase-sdk = { version = "0.1.0" }
"#,
        )
        .unwrap();

        // Create erc20 project
        let erc20_dir = src_dir.join("erc20");
        fs::create_dir_all(erc20_dir.join("src")).unwrap();
        fs::write(
            erc20_dir.join("Cargo.toml"),
            r#"[package]
name = "erc20"
version = "0.1.0"
"#,
        )
        .unwrap();

        let registry = RustContractsRegistry::new(&src_dir, Some(temp_dir.path())).unwrap();

        assert_eq!(registry.len(), 2);

        // Test power-calculator
        let power_calc = registry.get("power-calculator").unwrap();
        assert_eq!(power_calc.package_name, "power-calculator");
        assert_eq!(power_calc.sdk_version, Some("0.1.0".to_string()));

        // Test different name formats
        assert!(registry.contains("power-calculator"));
        assert!(registry.contains("power-calculator.wasm"));
        assert!(registry.contains("POWER-CALCULATOR"));

        // Test erc20
        assert!(registry.contains("erc20"));
        assert!(registry.contains("ERC20.wasm"));
    }

    #[test]
    fn test_artifact_paths() {
        let temp_dir = TempDir::new().unwrap();

        let info = RustProjectInfo {
            path: temp_dir.path().join("src/power-calculator"),
            package_name: "power-calculator".to_string(),
            sdk_version: Some("0.1.0".to_string()),
        };

        let artifacts_root = temp_dir.path().join("out");

        assert_eq!(info.artifact_name(), "power-calculator.wasm");
        assert_eq!(
            info.artifact_dir(&artifacts_root),
            artifacts_root.join("power-calculator.wasm")
        );
        assert_eq!(
            info.foundry_artifact_path(&artifacts_root),
            artifacts_root.join("power-calculator.wasm/foundry.json")
        );
    }

    #[test]
    fn test_should_skip_directory() {
        let gitignore =
            GitignoreRules { patterns: vec!["target/".to_string(), "*.log".to_string()] };

        // Standard directories should be skipped
        assert!(should_skip_directory(Path::new("target"), &gitignore, None));
        assert!(should_skip_directory(Path::new(".git"), &gitignore, None));
        assert!(should_skip_directory(Path::new("node_modules"), &gitignore, None));
        assert!(should_skip_directory(Path::new(".hidden"), &gitignore, None));

        // Regular directories should not be skipped
        assert!(!should_skip_directory(Path::new("src"), &gitignore, None));
        assert!(!should_skip_directory(Path::new("tests"), &gitignore, None));
    }

    #[test]
    fn test_simple_glob() {
        assert!(matches_simple_glob("test.log", "*.log"));
        assert!(matches_simple_glob("test", "test*"));
        assert!(matches_simple_glob("mytest", "*test"));
        assert!(matches_simple_glob("mytestfile", "*test*"));
        assert!(!matches_simple_glob("test", "*.log"));
    }
}
