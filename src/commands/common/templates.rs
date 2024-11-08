use crate::{error::Error, utils::repository::Repository};
use std::{collections::HashMap, path::PathBuf};

/// Template information structure
#[derive(Debug)]
pub struct Template {
    /// Template name
    pub name: String,
    /// Template description from README.md
    pub description: String,
    /// Full path to template directory
    pub path: PathBuf,
}

/// Manages available project templates
pub struct TemplateManager {
    _repository: Repository,
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    /// Create new instance of TemplateManager and scan available templates
    pub fn new() -> Result<Self, Error> {
        let repository = Repository::clone_fluentbase()?;
        let examples_path = repository.get_examples_path();

        if !examples_path.exists() {
            return Err(Error::InitializationError(format!(
                "Examples directory not found in repository: {}",
                examples_path.display()
            )));
        }

        let templates = Self::scan_templates(&examples_path)?;

        Ok(Self {
            _repository: repository,
            templates,
        })
    }

    /// Print list of available templates
    pub fn list(&self) {
        println!("\nAvailable templates from Fluentbase:");
        println!("----------------------------------");

        // Get sorted template names for consistent output
        let mut template_names: Vec<_> = self.templates.keys().collect();
        template_names.sort();

        for name in template_names {
            if let Some(template) = self.templates.get(name) {
                println!("\n📦 {}", template.name);
                println!("   {}", template.description);
            }
        }

        println!("\nUse --template <name> to initialize with specific template");
    }

    /// Get template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Get all available templates
    pub fn all(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    // Private helper methods
    fn scan_templates(examples_path: &PathBuf) -> Result<HashMap<String, Template>, Error> {
        let mut templates = HashMap::new();

        for entry in std::fs::read_dir(examples_path).map_err(|e| {
            Error::InitializationError(format!("Failed to read examples directory: {}", e))
        })? {
            let entry = entry.map_err(|e| Error::InitializationError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(template) = Self::create_template_from_path(&path)? {
                    templates.insert(template.name.clone(), template);
                }
            }
        }

        Ok(templates)
    }

    fn create_template_from_path(path: &PathBuf) -> Result<Option<Template>, Error> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::InitializationError("Invalid template name".to_string()))?
            .to_string();

        // Skip special directories
        if name.starts_with('.') || name.starts_with('_') {
            return Ok(None);
        }

        let description = Self::read_template_description(path)
            .unwrap_or_else(|_| "No description available".to_string());

        Ok(Some(Template {
            name,
            description,
            path: path.to_path_buf(),
        }))
    }

    fn read_template_description(template_path: &PathBuf) -> Result<String, Error> {
        let readme_path = template_path.join("README.md");
        if readme_path.exists() {
            let content = std::fs::read_to_string(&readme_path)
                .map_err(|e| Error::InitializationError(format!("Failed to read README: {}", e)))?;

            // Extract first paragraph or first line if no paragraphs
            Ok(content
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("No description available")
                .to_string())
        } else {
            Ok("No description available".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn create_test_template(
        temp: &assert_fs::TempDir,
        name: &str,
        has_readme: bool,
        readme_content: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template_dir = temp.child(name);
        template_dir.create_dir_all()?;

        // Create README.md if needed
        if has_readme {
            let content = readme_content.unwrap_or("Test template description");
            template_dir.child("README.md").write_str(content)?;
        }

        // Create some template files
        template_dir.child("lib.rs").write_str("// Test content")?;
        template_dir.child("Cargo.toml").write_str("[package]")?;

        Ok(())
    }

    #[test]
    fn test_template_scanning() -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;

        // Create test templates
        create_test_template(&temp, "template1", true, Some("Template 1 description"))?;
        create_test_template(&temp, "template2", true, Some("Template 2 description"))?;
        create_test_template(&temp, "template3", false, None)?;

        let templates = TemplateManager::scan_templates(&temp.path().to_path_buf())?;

        assert_eq!(templates.len(), 3);
        assert!(templates.contains_key("template1"));
        assert!(templates.contains_key("template2"));
        assert!(templates.contains_key("template3"));

        assert_eq!(
            templates.get("template1").unwrap().description,
            "Template 1 description"
        );
        assert_eq!(
            templates.get("template3").unwrap().description,
            "No description available"
        );

        Ok(())
    }

    #[test]
    fn test_skip_special_directories() -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;

        create_test_template(&temp, ".hidden", true, None)?;
        create_test_template(&temp, "_internal", true, None)?;
        create_test_template(&temp, "normal", true, None)?;

        let templates = TemplateManager::scan_templates(&temp.path().to_path_buf())?;

        assert_eq!(templates.len(), 1);
        assert!(templates.contains_key("normal"));
        assert!(!templates.contains_key(".hidden"));
        assert!(!templates.contains_key("_internal"));

        Ok(())
    }
}
