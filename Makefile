# Colors for output
RED    := \033[0;31m
GREEN  := \033[0;32m
YELLOW := \033[1;33m
BLUE   := \033[0;34m
NC     := \033[0m

# Help function
.PHONY: help
help: ## Display this help message
	@echo "$(BLUE)Usage:$(NC)"
	@echo "  make $(GREEN)<target>$(NC) $(YELLOW)[OPTION=value]$(NC)"
	@echo ""
	@echo "$(BLUE)Available targets:$(NC)"
	@awk '/^[a-zA-Z0-9_-]+:.*?## .*$$/ { \
		printf "  $(GREEN)%-20s$(NC) %s\n", substr($$1, 1, length($$1)-1), \
		substr($$0, index($$0, "##") + 3) \
	}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(BLUE)Options:$(NC)"
	@echo "  $(YELLOW)VERSION$(NC)    Version number (e.g., 1.2.3) or increment type (major|minor|patch)"
	@echo ""
	@echo "$(BLUE)Examples:$(NC)"
	@echo "  make release VERSION=1.2.3"
	@echo "  make release VERSION=minor"
	@echo "  make release VERSION=major"

# Default target
.PHONY: all
all: check ## Run all development checks (default)

# Development setup
.PHONY: setup
setup: install-tools install-wasm-target ## Set up development environment
	@echo "$(GREEN)Development environment setup complete!$(NC)"

.PHONY: install-tools 
install-tools: ## Install required development tools
	@echo "$(YELLOW)Installing required tools...$(NC)"
	@cargo install cargo-audit || (echo "$(RED)Failed to install cargo-audit$(NC)" && exit 1)
	@cargo install cargo-edit || (echo "$(RED)Failed to install cargo-edit$(NC)" && exit 1)
	@cargo install git-cliff || (echo "$(RED)Failed to install git-cliff$(NC)" && exit 1)

.PHONY: install-wasm-target
install-wasm-target: ## Install wasm32-unknown-unknown target
	@echo "$(YELLOW)Installing wasm32-unknown-unknown target...$(NC)"
	@rustup target add wasm32-unknown-unknown || (echo "$(RED)Failed to install wasm32-unknown-unknown target$(NC)" && exit 1)


# Development checks
.PHONY: fmt
fmt: ## Check code formatting
	@echo "$(YELLOW)Checking formatting...$(NC)"
	@cargo fmt -- --check

.PHONY: clippy
clippy: ## Run clippy lints
	@echo "$(YELLOW)Running Clippy...$(NC)"
	@cargo clippy --all-features --all-targets -- -D warnings

.PHONY: test
test: ## Run tests
	@echo "$(YELLOW)Running tests...$(NC)"
	@cargo test --all-features

.PHONY: audit
audit: ## Run security audit
	@echo "$(YELLOW)Running security audit...$(NC)"
	-@cargo audit

.PHONY: build-release
build-release: ## Build release version
	@echo "$(YELLOW)Building release...$(NC)"
	@cargo build --release

.PHONY: verify-package
verify-package: ## Verify package structure without version check
	@echo "$(YELLOW)Verifying package structure...$(NC)"
	@cargo package --allow-dirty --no-verify

.PHONY: check-publish
check-publish: ## Check full publication readiness
	@echo "$(YELLOW)Checking publication readiness...$(NC)"
	@cargo publish --dry-run

# Combined checks
.PHONY: check
check: fmt clippy test audit build-release verify-package ## Run all development checks
	@echo "$(GREEN)All development checks passed successfully!$(NC)"

.PHONY: check-release
check-release: fmt clippy test audit build-release check-publish ## Run all release checks
	@echo "$(GREEN)All release checks passed successfully!$(NC)"

# Changelog management
.PHONY: changelog-preview
changelog-preview: ## Preview unreleased changes
	@echo "$(YELLOW)Preview of unreleased changes:$(NC)"
	@git cliff --unreleased --tag HEAD

.PHONY: changelog
changelog: ## Update changelog with unreleased changes
	@echo "$(YELLOW)Updating changelog...$(NC)"
	@git cliff --unreleased --tag HEAD --prepend CHANGELOG.md

# Release management
.PHONY: prepare-version
prepare-version: ## Prepare new version: bump version and update changelog
	@if [ "$(VERSION)" = "" ]; then \
		echo "$(RED)Please specify VERSION=x.y.z or VERSION=(major|minor|patch)$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Bumping version...$(NC)"
	@if [ "$(VERSION)" = "major" ] || [ "$(VERSION)" = "minor" ] || [ "$(VERSION)" = "patch" ]; then \
		cargo set-version --bump $(VERSION); \
	else \
		cargo set-version $(VERSION); \
	fi
	@NEW_VERSION=$$(cargo pkgid | sed 's/.*#//') && \
	echo "$(GREEN)Version bumped to $$NEW_VERSION$(NC)" && \
	echo "$(YELLOW)Generating changelog for new version...$(NC)" && \
	git cliff --unreleased --tag v$$NEW_VERSION --prepend CHANGELOG.md
	@echo "$(GREEN)Changelog draft has been prepared!$(NC)"
	@echo "$(YELLOW)Please review CHANGELOG.md and make any necessary adjustments$(NC)"
	@echo "$(YELLOW)Then run: make commit-version$(NC)"

.PHONY: commit-version
commit-version: ## Commit version bump and changelog changes
	@NEW_VERSION=$$(cargo pkgid | sed 's/.*#//') && \
	echo "$(YELLOW)Committing version $$NEW_VERSION...$(NC)" && \
	git add Cargo.toml Cargo.lock CHANGELOG.md && \
	git commit -m "chore: release version $$NEW_VERSION" && \
	git tag -a v$$NEW_VERSION -m "Release v$$NEW_VERSION" && \
	echo "$(GREEN)Version $$NEW_VERSION has been committed and tagged!$(NC)" && \
	echo "$(YELLOW)To finish the release:$(NC)" && \
	echo "1. Review the changes: git show" && \
	echo "2. Push the release: git push origin main v$$NEW_VERSION"

.PHONY: release
release: check-release prepare-version ## Create a new release (VERSION=x.y.z|major|minor|patch)
	@echo "$(YELLOW)Release preparation complete!$(NC)"
	@echo "$(YELLOW)Please review CHANGELOG.md and run: make commit-version$(NC)"

# Cleanup
.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@cargo clean