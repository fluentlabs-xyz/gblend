use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use eyre::{eyre, Result, WrapErr};
use flate2::{write::GzEncoder, Compression};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path, time::Duration};
use tar::Builder;

/// Archive source information
#[derive(Debug, Clone, Serialize)]
pub struct ArchiveSourceInfo {
    pub content: String, // Base64 encoded bytes
    pub project_path: String,
}

/// Compile settings for the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileSettings {
    pub sdk_version: String,
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub rust_flags: Vec<String>,
    pub rust_toolchain: String,
    pub manifest_path: String,
}

/// Everything fluent-verifier needs to reproduce a build, derived from the on-disk
/// artifacts of a prior `forge build` invocation.
///
/// Owns the single bridge between fluentbase-build's `metadata.json` schema and
/// fluent-verifier's `CompileSettings` schema. Designed to be liftable to
/// `fluentbase-build` upstream — no gblend-specific types in the public surface.
#[derive(Debug)]
pub struct VerificationBundle {
    pub compile_settings: CompileSettings,
    pub archive: ArchiveSourceInfo,
    pub abi: serde_json::Value,
}

impl VerificationBundle {
    /// Build the bundle from a Rust contract's pre-built artifacts on disk.
    ///
    /// `artifact_dir` is the per-contract output directory (e.g.
    /// `out/power-calculator.wasm/`) containing `abi.json` and `metadata.json`.
    /// `contract_path` is the Rust crate root that produced those artifacts —
    /// it's what gets packed into the source archive.
    pub async fn from_artifacts(
        contract_path: &Path,
        artifact_dir: &Path,
    ) -> Result<Self> {
        let abi: serde_json::Value =
            foundry_common::fs::read_json_file(&artifact_dir.join("abi.json"))?;
        let metadata: serde_json::Value =
            foundry_common::fs::read_json_file(&artifact_dir.join("metadata.json"))?;
        let archive = ArchiveSourceInfo {
            content: ArchiveSourceBuilder::create_archive_from_path(contract_path).await?,
            project_path: ".".to_string(),
        };
        // Archive layout: pack `contract_path` (the crate root) as the archive root, so
        // `Cargo.toml` lives at the archive root. When workspace packing is added, the
        // manifest path becomes the relative path from the workspace root to the contract's
        // Cargo.toml.
        let manifest_path = "Cargo.toml".to_string();

        let stack_size = metadata["build_config"]["stack_size"].as_u64().unwrap_or(131072);

        // Source the docker tag actually used to build (NOT fluentbase-sdk's crate-internal
        // version — the two can differ; `base_tag` is the ground truth). Already `v`-prefixed.
        let compile_settings = CompileSettings {
            sdk_version: metadata["build_config"]["docker_image"]["base_tag"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            features: metadata["build_config"]["features"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            no_default_features: metadata["build_config"]["no_default_features"]
                .as_bool()
                .unwrap_or(true),
            rust_flags: vec![
                format!("-Clink-arg=-zstack-size={stack_size}"),
                "-Cpanic=abort".to_string(),
                "-Ctarget-feature=+bulk-memory".to_string(),
            ],
            rust_toolchain: metadata["environment"]["rust_toolchain"]
                .as_str()
                .unwrap_or("1.92.0")
                .to_string(),
            manifest_path,
        };

        Ok(Self { compile_settings, archive, abi })
    }
}

/// Verification request structure
#[derive(Debug, Serialize)]
pub struct VerificationRequest {
    pub contract_name: String,
    pub address_hash: String,
    pub archive_source: ArchiveSourceInfo,
    pub compile_settings: CompileSettings,
    pub abi: serde_json::Value,
}

impl VerificationRequest {
    /// Assemble a verification request from a pre-built bundle.
    pub fn from_bundle(
        contract_name: String,
        address_hash: String,
        bundle: VerificationBundle,
    ) -> Self {
        Self {
            contract_name,
            address_hash,
            archive_source: bundle.archive,
            compile_settings: bundle.compile_settings,
            abi: bundle.abi,
        }
    }

}

/// Response wrapper for error cases
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}

/// Main Fluent verification client
pub struct FluentVerificationClient {
    base_url: String,
    http_client: Client,
}

impl FluentVerificationClient {
    /// Create a new verification client
    pub fn new(base_url: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent("fluent-verification-client/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url: base_url.trim_end_matches('/').to_string(), http_client }
    }

    /// Verify contract using prepared request
    pub async fn verify(&self, request: VerificationRequest) -> Result<()> {
        self.send_verification_request(request).await
    }

    /// Send the verification request to Blockscout API
    async fn send_verification_request(&self, request: VerificationRequest) -> Result<()> {
        let url = format!(
            "{}/v2/smart-contracts/{}/verification/via/fluent",
            self.base_url, request.address_hash
        );

        // Add delay to allow contract indexing
        std::thread::sleep(Duration::from_secs(10));

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .wrap_err("Failed to send HTTP request")?;

        let status = response.status();

        if status.is_success() {
            sh_println!(
                "Contract submitted for verification. \
    It will appear at: {} once verified.",
                format!(
                    "{}/address/{}?tab=contract",
                    self.base_url.trim_end_matches("/api"),
                    request.address_hash
                )
            )?;

            Ok(())
        } else {
            let error_text = response.text().await.wrap_err("Failed to read error response")?;

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                Err(eyre!("API error ({}): {}", status.as_u16(), error_response.message))
            } else {
                Err(eyre!("API error ({}): {}", status.as_u16(), error_text))
            }
        }
    }
}

/// Namespace for the private tar.gz packing helpers used by [`VerificationBundle::from_artifacts`].
struct ArchiveSourceBuilder;

impl ArchiveSourceBuilder {
    /// Create a Base64-encoded tar.gz archive from the contract path
    async fn create_archive_from_path(contract_path: &Path) -> Result<String> {
        if contract_path.is_file() {
            // Single file - create archive with the file and infer project structure
            let file_name = contract_path
                .file_name()
                .ok_or_else(|| eyre!("Invalid file name"))?
                .to_string_lossy();

            let content =
                fs::read_to_string(contract_path).wrap_err("Failed to read contract file")?;

            // For single files, check if it's in a project directory structure
            if let Some(parent) = contract_path.parent()
                && parent.join("Cargo.toml").exists()
            {
                // It's part of a Rust project, archive the whole project
                return Self::create_tar_gz_archive(parent).await;
            }

            // Create a minimal project structure for a single file
            let files = vec![(file_name.to_string(), content)];
            Self::create_tar_gz_from_files(&files).await
        } else if contract_path.is_dir() {
            // Directory - create tar.gz archive of the entire directory
            Self::create_tar_gz_archive(contract_path).await
        } else {
            Err(eyre!("Path is neither a file nor a directory"))
        }
    }

    /// Create a tar.gz archive from a directory
    async fn create_tar_gz_archive(dir_path: &Path) -> Result<String> {
        let files = Self::collect_contract_files(dir_path)?;
        Self::create_tar_gz_from_files(&files).await
    }

    /// Create a tar.gz archive from a list of (path, content) tuples
    async fn create_tar_gz_from_files(files: &[(String, String)]) -> Result<String> {
        let mut tar_data = Vec::new();

        // Create tar archive
        {
            let mut tar = Builder::new(&mut tar_data);

            for (path, content) in files {
                let mut header = tar::Header::new_gnu();
                header.set_path(path).wrap_err_with(|| format!("Failed to set path for {path}"))?;
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();

                tar.append(&header, content.as_bytes())
                    .wrap_err_with(|| format!("Failed to append file {path}"))?;
            }

            tar.finish().wrap_err("Failed to finalize tar archive")?;
        }

        // Compress with gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&tar_data).wrap_err("Failed to write tar data to gzip encoder")?;
        let compressed_data = encoder.finish().wrap_err("Failed to finish gzip compression")?;

        // Encode to base64
        Ok(BASE64.encode(&compressed_data))
    }

    /// Collect all contract files from a directory (only .rs, .toml, .lock)
    fn collect_contract_files(dir: &Path) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();

        fn visit_dir(
            dir: &Path,
            base_path: &Path,
            files: &mut Vec<(String, String)>,
        ) -> Result<()> {
            for entry in fs::read_dir(dir)
                .wrap_err_with(|| format!("Failed to read directory: {}", dir.display()))?
            {
                let entry = entry.wrap_err("Failed to read directory entry")?;
                let path = entry.path();

                if path.is_file() {
                    // Include only specific file types for WASM contracts
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if matches!(ext_str.as_ref(), "rs" | "toml" | "lock") {
                            let content = fs::read_to_string(&path).wrap_err_with(|| {
                                format!("Failed to read file: {}", path.display())
                            })?;
                            let relative_path = path
                                .strip_prefix(base_path)
                                .wrap_err("Failed to create relative path")?
                                .to_string_lossy()
                                .to_string();
                            files.push((relative_path, content));
                        }
                    }
                } else if path.is_dir() {
                    // Skip directories that shouldn't be included
                    if let Some(dir_name) = path.file_name() {
                        let dir_str = dir_name.to_string_lossy();
                        if !matches!(dir_str.as_ref(), "target" | ".git" | "node_modules")
                            && !dir_str.starts_with('.')
                        {
                            visit_dir(&path, base_path, files)?;
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(dir, dir, &mut files)?;

        if files.is_empty() {
            return Err(eyre!("No contract files (.rs, .toml, .lock) found in directory"));
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_archive_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lib.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let content = ArchiveSourceBuilder::create_archive_from_path(&file_path).await.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_client_creation() {
        let client = FluentVerificationClient::new("https://example.com/".to_string());
        assert_eq!(client.base_url, "https://example.com");
    }

    #[test]
    fn test_serialization() {
        let bundle = VerificationBundle {
            compile_settings: CompileSettings {
                sdk_version: "v1.2.0".to_string(),
                features: vec![],
                no_default_features: true,
                rust_flags: vec!["-Cpanic=abort".to_string()],
                rust_toolchain: "1.92.0".to_string(),
                manifest_path: "Cargo.toml".to_string(),
            },
            archive: ArchiveSourceInfo {
                content: "dGVzdA==".to_string(),
                project_path: ".".to_string(),
            },
            abi: json!([]),
        };

        let request = VerificationRequest::from_bundle(
            "TestContract".to_string(),
            "0x1234".to_string(),
            bundle,
        );

        let serialized = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert!(parsed.get("archive_source").is_some());
        assert!(parsed.get("contract_name").is_some());
        assert!(parsed.get("address_hash").is_some());
        assert!(parsed.get("compile_settings").is_some());
    }
}
