# Contributing Guide

Thank you for considering contributing to this project! This guide will help you understand our workflow, coding standards, and release process.

## Ways to Contribute

1. **Opening Issues**
   - Report bugs with detailed descriptions
   - Suggest enhancements
   - Request features
   - Create detailed issue descriptions using templates
   - Use issue templates for bugs and features

2. **Adding Context**
   - Provide reproduction steps
   - Share code snippets
   - Add screenshots
   - Write use cases
   - Add examples and scenarios
   - Link to related issues or PRs

3. **Resolving Issues**
   - Fix bugs
   - Implement features
   - Improve documentation
   - Enhance performance
   - Demonstrate that an issue is not a problem
   - Open pull requests with fixes

4. **Reviewing Pull Requests**
   - Review code changes
   - Test proposed features
   - Suggest improvements
   - Share domain expertise
   - Help maintain code quality
   - Participate in technical discussions

## Development Setup

We use Make to automate development tasks. Start by setting up your environment:

```bash
make setup
```

This installs required tools:

- `cargo-audit`: Security vulnerability scanning
- `cargo-edit`: Dependency management
- `git-cliff`: Changelog generation

## Development Workflow

1. **Fork and Clone**

   ```bash
   git clone https://github.com/fluentlabs-xyz/gblend.git
   cd gblend
   ```

2. **Setup Environment**

   ```bash
   make setup
   ```

3. **Create Feature Branch**

   ```bash
   git checkout -b feat/your-feature
   # or
   git checkout -b fix/your-bugfix
   ```

4. **Sync with upstream** (if necessary)

   ```bash
   git remote add upstream https://github.com/fluentlabs-xyz/gblend.git
   git pull upstream main
   ```

5. **Make Changes**
   - Write code
   - Add tests
   - Update documentation

6. **Verify Changes**

   ```bash
   # During development - run individual checks
   make fmt           # Check formatting
   make clippy        # Check lints
   make test          # Run tests
   make audit         # Check dependencies

   # Before creating PR - run all checks
   make check         # Full verification
   ```

7. **Create Pull Request**
   - Use conventional commits
   - Link related issues
   - Fill PR template
   - Request reviews

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

Types:
feat:     New features
fix:      Bug fixes
docs:     Documentation only
style:    Style/formatting changes
refactor: Code refactoring
test:     Testing changes
chore:    Maintenance tasks

Examples:
feat(api): add user authentication
fix(ui): correct button alignment
docs: update installation guide
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/) (MAJOR.MINOR.PATCH):

- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

### Creating a Release

1. **Verify Release Readiness**

   ```bash
   make check-release
   ```

2. **Prepare Release**

   ```bash
   # Using semantic versioning
   make release VERSION=major  # Breaking changes
   make release VERSION=minor  # New features
   make release VERSION=patch  # Bug fixes

   # Or specific version
   make release VERSION=1.2.3
   
   # Pre-release versions
   make release VERSION=1.2.3-alpha.1  # Alpha releases
   make release VERSION=1.2.3-beta.1   # Beta releases
   make release VERSION=1.2.3-rc.1     # Release candidates
   ```

3. **Review Changes**
   - Check generated changelog
   - Review version updates
   - Verify all changes are included

4. **Commit Release**

   ```bash
   make commit-version
   ```

5. **Push Release**

   ```bash
   git push origin main v1.2.3
   ```

### Managing the Changelog

```bash
# Preview upcoming changes
make changelog-preview

# Update changelog
make changelog
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

## Development Commands

Run `make help` to see all available commands. Common tasks:

```bash
# Main commands list
make check           # Run all development checks
make check-release   # Full release verification
make clean           # Clean build artifacts

# Individual checks
make fmt            # Check formatting
make clippy         # Run linter
make test          # Run tests
make audit         # Security audit
```

## Best Practices

1. **Code Quality**
   - Write tests for new features
   - Maintain code coverage
   - Follow project style guide
   - Add documentation

2. **Pull Requests**
   - Keep changes focused
   - Update tests
   - Add documentation
   - Respond to reviews

3. **Communication**
   - Be respectful
   - Provide context
   - Ask questions
   - Stay engaged

## Getting Help

- Issues: Report bugs and request features
- Pull Requests: Get feedback on changes

Every contributor was once a beginner - don't hesitate to ask for help!

## Code of Conduct

We follow the [Contributor Covenant](https://www.contributor-covenant.org/). Be respectful and inclusive in all interactions.
