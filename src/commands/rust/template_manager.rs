use crate::{
    error::Error,
    utils::{fs, repository::Repository},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    name: String,
    description: String,
    path: PathBuf,
}

impl Template {
    /// Create a Template from directory path
    pub(super) fn from_path(path: &Path) -> Result<Option<Self>, Error> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::InitializationError("Invalid template name".to_string()))?
            .to_string();

        if name.starts_with('.') || name.starts_with('_') {
            return Ok(None);
        }

        let description = Self::read_description(path)?;
        Ok(Some(Self {
            name,
            description,
            path: path.to_path_buf(),
        }))
    }

    /// Read template description from README.md
    fn read_description(template_path: &Path) -> Result<String, Error> {
        let readme_path = template_path.join("README.md");
        if !readme_path.exists() {
            return Ok("No description available".to_string());
        }

        let content = std::fs::read_to_string(&readme_path)
            .map_err(|e| Error::InitializationError(format!("Failed to read README: {}", e)))?;

        Ok(content
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("No description available")
            .to_string())
    }

    // Getters
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CargoPackage {
    name: String,
    version: String,
    manifest_path: String,
    features: Option<Vec<String>>,
}

/// Manages project templates and handles workspace dependency resolution
pub struct TemplateManager {
    repository: Repository,
    templates: HashMap<String, Template>,
    root_dependencies: DocumentMut,
}
impl TemplateManager {
    /// Create new instance of TemplateManager and scan available templates
    pub fn new() -> Result<Self, Error> {
        let repository = Repository::clone_fluentbase()?;
        let examples_path = repository.get_examples_path();
        let root_cargo_path = repository.get_root_cargo_path();

        if !examples_path.exists() {
            return Err(Error::InitializationError(format!(
                "Examples directory not found in repository: {}",
                examples_path.display()
            )));
        }

        let root_dependencies = std::fs::read_to_string(&root_cargo_path).map_err(|e| {
            Error::InitializationError(format!("Failed to read root Cargo.toml: {}", e))
        })?;

        let root_doc = root_dependencies.parse::<DocumentMut>().map_err(|e| {
            Error::InitializationError(format!("Failed to parse root Cargo.toml: {}", e))
        })?;

        let templates = Self::scan_templates(&examples_path)?;

        Ok(Self {
            repository,
            templates,
            root_dependencies: root_doc,
        })
    }

    /// Print list of available templates
    pub fn list(&self) {
        println!("\nAvailable templates from Fluentbase:");
        println!("----------------------------------");

        let mut template_names: Vec<_> = self.templates.keys().collect();
        template_names.sort();

        for name in template_names {
            if let Some(template) = self.templates.get(name) {
                println!("\n📦 {}", template.name());
                println!("   {}", template.description());
            }
        }

        println!("\nUse --template <name> to initialize with specific template");
    }

    /// Get template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Initialize project from template
    pub fn init_project(&self, project_path: &Path, template: &Template) -> Result<(), Error> {
        println!("🚀 Initializing project from template: {}", template.name());

        // Convert Path to PathBuf for copy_dir_all
        let src = template.path().to_path_buf();
        let dst = project_path.to_path_buf();
        println!("src: {:?}", src.display());
        println!("dst: {:?}", dst.display());

        // Copy template files
        fs::copy_dir_all(&src, &dst)
            .map_err(|e| Error::InitializationError(format!("Failed to copy template: {}", e)))?;

        // Resolve workspace dependencies if they exist
        self.resolve_dependencies(project_path, template.name())?;

        Ok(())
    }

    /// Scan templates directory and create Template instances
    fn scan_templates(examples_path: &Path) -> Result<HashMap<String, Template>, Error> {
        let mut templates = HashMap::new();

        for entry in std::fs::read_dir(examples_path).map_err(|e| {
            Error::InitializationError(format!("Failed to read examples directory: {}", e))
        })? {
            let entry = entry.map_err(|e| Error::InitializationError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(template) = Template::from_path(&path)? {
                    templates.insert(template.name().to_string(), template);
                }
            }
        }

        Ok(templates)
    }
    /// Resolve workspace dependencies for a project
    fn resolve_dependencies(&self, project_path: &Path, template_name: &str) -> Result<(), Error> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(()); // Exit if Cargo.toml does not exist
        }

        println!("📦 Resolving dependencies...");

        // Fetch dependencies from the workspace section of the root Cargo.toml
        let root_deps = match self.root_dependencies.get("workspace") {
            Some(Item::Table(workspace)) => match workspace.get("dependencies") {
                Some(Item::Table(deps)) => deps,
                _ => {
                    println!("No dependencies found in workspace section.");
                    return Ok(());
                }
            },
            _ => {
                println!("No workspace section found in root Cargo.toml.");
                return Ok(());
            }
        };

        // Parse the project's Cargo.toml file
        let content = std::fs::read_to_string(&cargo_toml_path)
            .map_err(|e| Error::InitializationError(format!("Failed to read Cargo.toml: {}", e)))?;
        let mut doc = content
            .parse::<DocumentMut>()
            .map_err(|e| Error::InitializationError(format!("Failed to parse TOML: {}", e)))?;

        // Locate dependencies section in the template's Cargo.toml
        let template_deps = match doc.get_mut("dependencies") {
            Some(Item::Table(deps)) => deps,
            _ => {
                println!("No dependencies found in template's Cargo.toml.");
                return Ok(());
            }
        };

        for (dep_name, dep_item) in template_deps.iter_mut() {
            // Only process dependencies marked with `workspace = true`
            if dep_item.get("workspace").is_some() {
                let dep_key = dep_name.get();
                let root_dep = root_deps.get(dep_key).unwrap_or_else(|| {
                panic!(
                    "The dependency '{dep_key}', used in the example '{template_name}', is marked with `workspace = true`, \
                    but it is missing from the workspace's Cargo.toml file. Please add '{dep_key}' to the `[dependencies]` \
                    section in the root Cargo.toml to resolve this issue.",
                );
            });

                // Update fluentbase dependencies with specific Git settings, if needed
                if dep_key.starts_with("fluentbase-") {
                    let mut items = toml_edit::InlineTable::new();
                    items.insert(
                        "git",
                        Value::from("https://github.com/fluentlabs-xyz/fluentbase"),
                    );
                    items.insert("branch", Value::from("devel"));

                    // Retain any existing default features
                    if let Some(default_features) = dep_item.get("default-features") {
                        items.insert(
                            "default-features",
                            default_features.clone().as_value().unwrap().clone(),
                        );
                    }

                    *dep_item = Item::Value(Value::InlineTable(items));
                } else {
                    // Otherwise, copy the workspace dependency directly
                    *dep_item = root_dep.clone();
                }
            }
        }

        // Write updated dependencies back to the template's Cargo.toml
        std::fs::write(&cargo_toml_path, doc.to_string()).map_err(|e| {
            Error::InitializationError(format!("Failed to write Cargo.toml: {}", e))
        })?;
        Ok(())
    }
}
