# Contributing Guide

Thank you for considering contributing to this project! This guide will help you understand our workflow, coding standards, and release process. Whether you're fixing a bug, adding a feature, or improving documentation, your contribution is highly appreciated!

## Ways to contribute

There are fundamentally four ways an individual can contribute:

1. **By opening an issue:** For example, if you believe that you have uncovered a bug,
   creating a new issue in the issue tracker is the way to report it.
2. **By adding context:** Providing additional context to existing issues,
   such as screenshots and code snippets.
3. **By resolving issues:** Either demonstrating that the issue is not a problem,
   or opening a pull request with a fix.
4. **By reviewing pull requests:** Help review and discuss proposed changes.

**Anybody can participate in any stage of contribution**. We urge you to participate in the discussion around bugs and participate in reviewing PRs.

## Development Workflow

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/fluentlabs-xyz/gblend.git
   cd gblend
   ```

2. **Create a new branch**

   ```bash
   git checkout -b feat/your-feature
   ```

3. **Sync with the main repository** (if necessary):

   ```bash
   git remote add upstream https://github.com/fluentlabs-xyz/gblend.git
   git pull upstream main
   ```

4. **Make changes following the commit convention**

5. **Test your changes**

   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

6. **Push changes**

   ```bash
   git push origin feat/your-feature
   ```

7. **Open a Pull Request**
   - Go to your fork on GitHub
   - Click "New Pull Request"
   - Choose the appropriate template:
     - Use the default template for general changes
     - Use the feature template for new features
     - Use the bugfix template for bug fixes
   - Fill in all required sections of the template
   - Link any related issues using the `Closes #issue-number` syntax
   - Request reviews from maintainers
   - Ensure all checks are passing

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format
<type>[optional scope]: <description>

# Types
feat:     New features
fix:      Bug fixes
docs:     Documentation changes
style:    Code style changes
refactor: Code changes without features/fixes
test:     Tests changes
chore:    Build process changes

# Examples
feat(auth): add OAuth support
fix(ui): resolve button alignment issue
docs: update installation guide
test: add unit tests for auth module
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/) (MAJOR.MINOR.PATCH):

- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

### Creating a Release

1. **Install cargo-edit**

   ```bash
   cargo install cargo-edit
   ```

2. **Update version**

   For incremental version updates:

   ```bash
   # For MAJOR version bump (breaking changes)
   cargo set-version --bump major

   # For MINOR version bump (new features)
   cargo set-version --bump minor

   # For PATCH version bump (bug fixes, small changes)
   cargo set-version --bump patch
   ```

   For setting a specific version:

   ```bash
   cargo set-version 1.2.3
   ```

   For pre-release versions:

   ```bash
   # For alpha releases
   cargo set-version 1.2.3-alpha.1

   # For beta releases
   cargo set-version 1.2.3-beta.1

   # For release candidate versions
   cargo set-version 1.2.3-rc.1
   ```

3. **Generate changelog using git-cliff**

   ```bash
   # Update changelog
   git cliff --tag v1.2.3 -o CHANGELOG.md
   
   # Or append unreleased changes
   git cliff --unreleased --tag v1.2.3 --prepend CHANGELOG.md
   ```

4. **Commit version and changelog**

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release version 1.2.3"
   ```

5. **Create and push a release tag**

   ```bash
   git tag -a v1.2.3 -m "Release v1.2.3"
   git push origin v1.2.3
   ```

6. **Automated Publishing**
   - Pushing the tag `v*.*.*` will automatically trigger the GitHub Action workflow
   - The workflow will:
     1. Check code formatting
     2. Run clippy lints
     3. Run tests
     4. Publish to crates.io using the `CARGO_REGISTRY_TOKEN`

7. **Create GitHub Release**
   - Go to Releases page
   - Click "Draft a new release"
   - Select the tag
   - Use git-cliff to generate release notes:

     ```bash
     git cliff --current --strip header
     ```

### Managing Published Versions

#### Yanking a Release

```bash
# Yank a specific version
cargo yank --version 1.2.3

# Undo a yank if needed
cargo yank --version 1.2.3 --undo
```

Important notes about yanking:

- Yanking does **not** delete the version
- Existing projects can still use yanked versions
- New projects cannot add yanked versions as dependencies
- Use yanking when a version:
  - Has critical bugs
  - Contains security vulnerabilities
  - Has backwards compatibility issues
  - Was accidentally published

### Release Checklist

| Checkpoint                       | Command/Action                        | Status |
|---------------------------------|--------------------------------------|--------|
| Install cargo-edit             | `cargo install cargo-edit`           | [ ]    |
| Version bump                   | `cargo set-version 1.2.3`            | [ ]    |
| Changelog generated           | `git cliff`                          | [ ]    |
| Tests passing                   | `cargo test`                         | [ ]    |
| Formatting checked              | `cargo fmt -- --check`               | [ ]    |
| Linting checked                 | `cargo clippy -- -D warnings`        | [ ]    |
| Dependencies reviewed           | Check `Cargo.toml`                   | [ ]    |
| Breaking changes documented     | Update CHANGELOG.md                  | [ ]    |
| `CARGO_REGISTRY_TOKEN` present  | Check GitHub Secrets                 | [ ]    |

### Troubleshooting Releases

If the automated publishing fails:

1. Check the GitHub Actions logs for errors
2. Verify the `CARGO_REGISTRY_TOKEN` is properly set
3. Ensure version hasn't been published before
4. Fix any issues and delete the tag if needed:

   ```bash
   git tag -d v1.2.3
   git push --delete origin v1.2.3
   ```

After yanking a version:

1. Create a new patch version with fixes
2. Update security advisory if needed
3. Notify users through GitHub Issues
4. Update release notes to indicate the version is yanked

## Code of Conduct

We follow the [Contributor Covenant](https://www.contributor-covenant.org/). Please be respectful and inclusive while contributing to the project.

## Getting Help

If you need help with your contribution:

- Open a discussion in the GitHub Discussions tab
- Ask in the project's communication channels
- Check existing documentation and issues

Remember: every contributor was once a beginner. Don't hesitate to ask for help or clarification.
