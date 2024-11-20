# Colors for output
RED    := \033[0;31m
GREEN  := \033[0;32m
YELLOW := \033[1;33m
BLUE   := \033[0;34m
NC     := \033[0m

# Help function
.PHONY: help
help: ## Show available commands with descriptions
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

# Development setup
.PHONY: setup
setup: ## Install required tools (cargo-audit, cargo-edit, git-cliff) and wasm target
	@echo "$(YELLOW)Installing required tools...$(NC)"
	@cargo install cargo-audit || (echo "$(RED)Failed to install cargo-audit$(NC)" && exit 1)
	@cargo install cargo-edit || (echo "$(RED)Failed to install cargo-edit$(NC)" && exit 1)
	@cargo install git-cliff || (echo "$(RED)Failed to install git-cliff$(NC)" && exit 1)
	@echo "$(YELLOW)Installing wasm32-unknown-unknown target...$(NC)"
	@rustup target add wasm32-unknown-unknown || (echo "$(RED)Failed to install wasm32-unknown-unknown target$(NC)" && exit 1)
	@echo "$(GREEN)Development environment setup complete!$(NC)"

# Development checks
.PHONY: check
check: ## Run all checks: format, lint, test, audit, and build
	@echo "$(YELLOW)Checking formatting...$(NC)"
	@cargo fmt -- --check
	@echo "$(YELLOW)Running Clippy...$(NC)"
	@cargo clippy --all-features --all-targets -- -D warnings
	@echo "$(YELLOW)Running tests...$(NC)"
	@cargo test --all-features
	@echo "$(YELLOW)Running security audit...$(NC)"
	-@cargo audit
	@echo "$(YELLOW)Building release...$(NC)"
	@cargo build --release
	@echo "$(GREEN)All development checks passed successfully!$(NC)"

# Version existence check
.PHONY: check-version
check-version: ## Check if current version already exists on crates.io
	@CURRENT_VERSION=$$(cargo pkgid | sed 's/.*#//') && \
	PACKAGE_NAME=$$(cargo pkgid | sed -E 's/.*\/(.*?)#.*/\1/') && \
	if curl -s "https://crates.io/api/v1/crates/$$PACKAGE_NAME/$$CURRENT_VERSION" | grep -q "Version $$CURRENT_VERSION not found"; then \
		echo "$(GREEN)Version $$CURRENT_VERSION is available$(NC)"; \
	else \
		echo "$(RED)Version $$CURRENT_VERSION already exists on crates.io$(NC)" && \
		exit 1; \
	fi

# Release management
.PHONY: release
release: check ## Prepare new release (VERSION=x.y.z|major|minor|patch): run checks, bump version, update changelog, create commit and tag
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
	echo "$(YELLOW)Checking publication readiness...$(NC)" && \
	cargo publish --dry-run --allow-dirty && \
	echo "$(YELLOW)Generating changelog...$(NC)" && \
	git cliff --unreleased --tag v$$NEW_VERSION --prepend CHANGELOG.md && \
	git add Cargo.toml Cargo.lock CHANGELOG.md && \
	git commit -m "chore: release version $$NEW_VERSION" && \
	git tag -a v$$NEW_VERSION -m "Release v$$NEW_VERSION" && \
	echo "$(GREEN)Version $$NEW_VERSION has been prepared!$(NC)" && \
	echo "$(YELLOW)To complete the release:$(NC)" && \
	echo "1. Review the changes: git show" && \
	echo "2. Push commits: git push origin main" && \
	echo "3. After verification, push the tag: git push origin v$$NEW_VERSION"

.PHONY: release-undo
release-undo: ## Revert local release preparation (commit and tag) before pushing changes
	@echo "$(YELLOW)Undoing last release...$(NC)"
	@if [ -n "$$(git tag --points-at HEAD)" ]; then \
		LAST_TAG=$$(git tag --points-at HEAD) && \
		echo "$(YELLOW)Removing tag $$LAST_TAG$(NC)" && \
		git tag -d $$LAST_TAG; \
	fi
	@echo "$(YELLOW)Reverting last commit...$(NC)"
	@if git log -1 --pretty=%B | grep -q "^chore: release version"; then \
		git reset --hard HEAD~1; \
		echo "$(GREEN)Release preparation has been undone!$(NC)"; \
	else \
		echo "$(RED)Last commit is not a release commit$(NC)" && exit 1; \
	fi

# Cleanup
.PHONY: clean
clean: ## Clean all build artifacts and dependencies
	@cargo clean