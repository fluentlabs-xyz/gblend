use super::install::DependencyInstallOpts;
use clap::{Parser, ValueHint};
use eyre::Result;
use foundry_cli::utils::Git;
use foundry_common::fs;
use foundry_compilers::artifacts::remappings::Remapping;
use foundry_config::Config;
use std::path::{Path, PathBuf};
use yansi::Paint;

const EXAMPLES_REPO: &str = "https://github.com/fluentlabs-xyz/examples";

/// CLI arguments for `forge init`.
#[derive(Clone, Debug, Default, Parser)]
pub struct InitArgs {
    /// The root directory of the new project.
    #[arg(value_hint = ValueHint::DirPath, default_value = ".", value_name = "PATH")]
    pub root: PathBuf,

    /// The template to start from.
    ///
    /// Can be:
    /// - Example name: "erc20" -> uses fluentlabs-xyz/examples/erc20
    /// - GitHub shorthand: "user/repo" -> uses github.com/user/repo
    /// - Full URL: "https://github.com/user/repo"
    #[arg(long, short)]
    pub template: Option<String>,

    /// List all available examples from fluentlabs-xyz/examples
    #[arg(long, alias = "ls")]
    pub list_examples: bool,

    /// Branch argument that can only be used with template option.
    /// If not specified, the default branch is used.
    #[arg(long, short, requires = "template")]
    pub branch: Option<String>,

    /// Do not install dependencies from the network.
    #[arg(long, conflicts_with = "template", visible_alias = "no-deps")]
    pub offline: bool,

    /// Create the project even if the specified root directory is not empty.
    #[arg(long, conflicts_with = "template")]
    pub force: bool,

    /// Create a .vscode/settings.json file with Solidity settings, and generate a remappings.txt
    /// file.
    #[arg(long, conflicts_with = "template")]
    pub vscode: bool,

    #[command(flatten)]
    pub install: DependencyInstallOpts,
}

impl InitArgs {
    pub fn run(self) -> Result<()> {
        let Self { root, template, branch, install, offline, force, vscode, list_examples } = self;

        if list_examples {
            return print_examples();
        }

        let DependencyInstallOpts { shallow, no_git, commit } = install;

        // create the root dir if it does not exist
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }
        let root = dunce::canonicalize(root)?;
        let git = Git::new(&root).shallow(shallow);

        // if a template is provided, normalize it and fetch
        if let Some(template) = template {
            // Check if this is an example name (no slash) or external repo
            let is_example = !template.contains('/') && !template.contains("://");

            // Handle fluentlabs-xyz/examples templates with sparse checkout
            if is_example {
                sh_println!("Initializing {} from fluentlabs-xyz/examples/{}...", root.display(), template)?;
                clone_example_template(&root, &template, branch.as_deref())?;
                sh_println!("{}", "    Initialized forge project".green())?;
                return Ok(());
            }

            let template = if template.contains("://") {
                template
            } else if template.starts_with("github.com/") {
                "https://".to_string() + &template
            } else {
                "https://github.com/".to_string() + &template
            };
            sh_println!("Initializing {} from {}...", root.display(), template)?;
            // initialize the git repository
            git.init()?;

            // fetch the template - always fetch shallow for templates since git history will be
            // collapsed. gitmodules will be initialized after the template is fetched
            git.fetch(true, &template, branch)?;

            // reset git history to the head of the template
            // first get the commit hash that was fetched
            let commit_hash = git.commit_hash(true, "FETCH_HEAD")?;
            // format a commit message for the new repo
            let commit_msg = format!("chore: init from {template} at {commit_hash}");
            // get the hash of the FETCH_HEAD with the new commit message
            let new_commit_hash = git.commit_tree("FETCH_HEAD^{tree}", Some(commit_msg))?;
            // reset head of this repo to be the head of the template repo
            git.reset(true, new_commit_hash)?;

            // if shallow, just initialize submodules
            if shallow {
                git.submodule_init()?;
            } else {
                // if not shallow, initialize and clone submodules (without fetching latest)
                git.submodule_update(false, false, true, true, std::iter::empty::<PathBuf>())?;
            }
        } else {
            // if target is not empty
            if root.read_dir().is_ok_and(|mut i| i.next().is_some()) {
                if !force {
                    eyre::bail!(
                        "Cannot run `init` on a non-empty directory.\n\
                        Run with the `--force` flag to initialize regardless."
                    );
                }
                sh_warn!("Target directory is not empty, but `--force` was specified")?;
            }

            // ensure git status is clean before generating anything
            if !no_git && commit && !force && git.is_in_repo()? {
                git.ensure_clean()?;
            }

            sh_println!("Initializing {}...", root.display())?;

            // make the dirs
            let src = root.join("src");
            fs::create_dir_all(&src)?;

            let test = root.join("test");
            fs::create_dir_all(&test)?;

            let script = root.join("script");
            fs::create_dir_all(&script)?;

            // Create the power-calculator subdirectory structure
            let power_calc_dir = src.join("power-calculator");
            fs::create_dir_all(&power_calc_dir)?;
            let power_calc_src = power_calc_dir.join("src");
            fs::create_dir_all(&power_calc_src)?;

            // Write the BlendedCounter.sol contract file
            let contract_path = src.join("BlendedCounter.sol");
            fs::write(
                contract_path,
                include_str!("../../examples/blended-counter/src/BlendedCounter.sol"),
            )?;

            // Write the test file
            let test_path = test.join("BlendedCounter.t.sol");
            fs::write(test_path, include_str!("../../examples/blended-counter/test/BlendedCounter.t.sol"))?;

            // Write the deployment script
            let script_path = script.join("BlendedCounter.s.sol");
            fs::write(
                script_path,
                include_str!("../../examples/blended-counter/script/BlendedCounter.s.sol"),
            )?;

            // Write the Rust WASM module files
            let cargo_toml_path = power_calc_dir.join("Cargo.toml");
            fs::write(
                cargo_toml_path,
                include_str!("../../examples/blended-counter/src/power-calculator/Cargo.toml"),
            )?;

            let lib_rs_path = power_calc_src.join("lib.rs");
            fs::write(
                lib_rs_path,
                include_str!("../../examples/blended-counter/src/power-calculator/src/lib.rs"),
            )?;

            // Write the README
            let readme_path = root.join("README.md");
            fs::write(readme_path, include_str!("../../examples/blended-counter/README.md"))?;

            // write foundry.toml, if it doesn't exist already
            let dest = root.join(Config::FILE_NAME);

            // Load config after writing custom foundry.toml
            let mut config = Config::load_with_root(&root)?;
            if !dest.exists() {
                fs::write(dest, config.clone().into_basic().to_string_pretty()?)?;
            }
            let git = self.install.git(&config);

            // set up the repo
            if !no_git {
                init_git_repo(git, commit)?;
            }

            // install forge-std
            if !offline {
                if root.join("lib/forge-std").exists() {
                    sh_warn!("\"lib/forge-std\" already exists, skipping install...")?;
                    self.install.install(&mut config, vec![])?;
                } else {
                    let dep = "https://github.com/foundry-rs/forge-std".parse()?;
                    self.install.install(&mut config, vec![dep])?;
                }
            }

            // init vscode settings
            if vscode {
                init_vscode(&root)?;
            }
        }

        sh_println!("{}", "    Initialized forge project".green())?;
        Ok(())
    }
}
fn print_examples() -> Result<()> {
    sh_println!("Available examples:\n");

    sh_println!("Remote (from fluentlabs-xyz/examples):");

    match fetch_examples_list() {
        Ok(manifest) => {
            for ex in manifest.examples {
                let difficulty_badge = match ex.difficulty.as_str() {
                    "beginner" => "🟢",
                    "intermediate" => "🟡",
                    "advanced" => "🔴",
                    _ => "⚪",
                };
                sh_println!("  {} {:<18} {}", difficulty_badge, ex.name, ex.description);
            }
        }
        Err(e) => {
            sh_warn!("Unable to fetch examples list: {}", e)?;
            sh_println!("  (Check your internet connection)");
        }
    }

    sh_println!("\n Usage:");
    sh_println!("  gblend init ./path                      # default (counter)");
    sh_println!("  gblend init --template erc20-rs ./path  # specific example");
    sh_println!("  gblend init --template user/repo ./path # custom template");

    Ok(())
}

fn clone_example_template(root: &Path, example_name: &str, branch: Option<&str>) -> Result<()> {
    let examples_cache_dir = Config::foundry_cache_dir()
        .ok_or_else(|| eyre::eyre!("Could not find foundry cache directory"))?
        .join("examples");


    if examples_cache_dir.exists() {
        sh_println!("Updating cached examples...")?;
        let git = Git::new(&examples_cache_dir);

        // Fetch latest changes
        git.fetch(true, "origin", branch)?;

        // Reset to latest
        git.reset(true, "FETCH_HEAD")?;
    } else {
        sh_println!("Downloading examples repository...")?;
        fs::create_dir_all(&examples_cache_dir)?;
        Git::clone_with_branch(true, EXAMPLES_REPO, branch.unwrap_or("main"), Some(&examples_cache_dir))?;
    }

    let example_source = examples_cache_dir.join(example_name);
    if !example_source.exists() {
        eyre::bail!(
            "Example '{}' not found in repository.\n\
            Available examples: gblend init --list-examples",
            example_name
        );
    }

    copy_dir_all(&example_source, root)?;

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

const EXAMPLES_JSON_URL: &str =
    "https://raw.githubusercontent.com/fluentlabs-xyz/examples/main/examples.json";

fn fetch_examples_list() -> Result<ExamplesManifest> {
    let content = ureq::get(EXAMPLES_JSON_URL)
        .call()
        .into_string()?;

    let manifest: ExamplesManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}
#[derive(Debug, serde::Deserialize)]
struct ExamplesManifest {
    version: String,
    examples: Vec<Example>,
}

#[derive(Debug, serde::Deserialize)]
struct Example {
    name: String,
    description: String,
    #[serde(default)]
    difficulty: String,
    #[serde(default)]
    tags: Vec<String>,
}

/// Initialises `root` as a git repository, if it isn't one already.
///
/// Creates `.gitignore` and `.github/workflows/test.yml`, if they don't exist already.
///
/// Commits everything in `root` if `commit` is true.
fn init_git_repo(git: Git<'_>, commit: bool) -> Result<()> {
    // git init
    if !git.is_in_repo()? {
        git.init()?;
    }

    // .gitignore
    let gitignore = git.root.join(".gitignore");
    if !gitignore.exists() {
        fs::write(gitignore, include_str!("../../examples/blended-counter/.gitignoreTemplate"))?;
    }

    // commit everything
    if commit {
        git.add(Some("--all"))?;
        git.commit("chore: forge init")?;
    }

    Ok(())
}

/// initializes the `.vscode/settings.json` file
fn init_vscode(root: &Path) -> Result<()> {
    let remappings_file = root.join("remappings.txt");
    if !remappings_file.exists() {
        let mut remappings = Remapping::find_many(&root.join("lib"))
            .into_iter()
            .map(|r| r.into_relative(root).to_relative_remapping().to_string())
            .collect::<Vec<_>>();
        if !remappings.is_empty() {
            remappings.sort();
            let content = remappings.join("\n");
            fs::write(remappings_file, content)?;
        }
    }

    let vscode_dir = root.join(".vscode");
    let settings_file = vscode_dir.join("settings.json");
    let mut settings = if !vscode_dir.is_dir() {
        fs::create_dir_all(&vscode_dir)?;
        serde_json::json!({})
    } else if settings_file.exists() {
        foundry_compilers::utils::read_json_file(&settings_file)?
    } else {
        serde_json::json!({})
    };

    let obj = settings.as_object_mut().expect("Expected settings object");
    // insert [vscode-solidity settings](https://github.com/juanfranblanco/vscode-solidity)
    let src_key = "solidity.packageDefaultDependenciesContractsDirectory";
    if !obj.contains_key(src_key) {
        obj.insert(src_key.to_string(), serde_json::Value::String("src".to_string()));
    }
    let lib_key = "solidity.packageDefaultDependenciesDirectory";
    if !obj.contains_key(lib_key) {
        obj.insert(lib_key.to_string(), serde_json::Value::String("lib".to_string()));
    }

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_file, content)?;

    Ok(())
}