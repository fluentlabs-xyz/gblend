use assert_fs::prelude::*;
use predicates::prelude::*;
use std::{fs, path::PathBuf};

pub struct TestProject {
    pub temp_dir: assert_fs::TempDir,
}

impl TestProject {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = assert_fs::TempDir::new()?;
        Ok(Self { temp_dir })
    }

    pub fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn create_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.temp_dir.child(path).write_str(content)?;
        Ok(())
    }

    pub fn assert_file_exists(&self, path: &str) {
        self.temp_dir.child(path).assert(predicate::path::exists());
    }

    pub fn read_file(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(fs::read_to_string(self.temp_dir.child(path).path())?)
    }

    pub fn assert_file_contains(
        &self,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_content = self.read_file(path)?;
        assert!(
            file_content.contains(content),
            "File {} does not contain expected content",
            path
        );
        Ok(())
    }
}
