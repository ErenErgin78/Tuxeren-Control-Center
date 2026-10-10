# ==============================================================================
# Monster Control Center - Linux Makefile
# ==============================================================================

.PHONY: help setup setup-deps dev build build-rpm build-deb build-appimage install install-rpm install-deb install-appimage clean test lint bump bump-patch bump-minor bump-major set-version release release-patch

.DEFAULT_GOAL := help

help: ## Show available make targets
	@echo "Monster Control Center Commands:"
	@echo ""
	@LC_ALL=C grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'
	@echo ""

setup: ## Verify environment and toolchain
	@bash scripts/setup_env.sh

setup-deps: ## Check and prompt to install system-level build dependencies
	@bash scripts/setup_env.sh --install-deps

dev: ## Start Monster Control Center in development mode
	@bash scripts/run.sh

build: ## Build release packages for all configured targets (rpm, deb, appimage)
	@bash scripts/build_packages.sh

install: ## Build release binary and install locally via install.sh
	@cargo build --release
	@bash install.sh

APP_VERSION ?= $(shell grep -m1 '"version"' tauri.conf.json 2>/dev/null | cut -d '"' -f 4)

build-rpm: ## Build RPM package only
	@bash scripts/build_packages.sh rpm

install-rpm: build-rpm ## Build RPM and reinstall on local system (sudo dnf)
	@echo "[+] Installing freshly built Monster Control Center RPM package..."
	@RPM_FILE=$$(ls dist/Monster*Control*Center-$${APP_VERSION:-*}-*.rpm dist/monster*control*center-$${APP_VERSION:-*}-*.rpm dist/*$${APP_VERSION:-*}*.rpm 2>/dev/null | sort -V | tail -n 1); \
	if [ -z "$$RPM_FILE" ]; then \
		RPM_FILE=$$(ls dist/*.rpm 2>/dev/null | sort -V | tail -n 1); \
	fi; \
	if [ -n "$$RPM_FILE" ]; then \
		echo "[*] Installing package: $$RPM_FILE..."; \
		sudo dnf install -y "$$RPM_FILE"; \
	else \
		echo "[-] Error: RPM package not found in dist/!" >&2; exit 1; \
	fi
	@update-desktop-database ~/.local/share/applications 2>/dev/null || true
	@echo "========================================================"
	@echo "  Monster Control Center RPM successfully installed!"
	@echo "========================================================"

build-deb: ## Build DEB package only
	@bash scripts/build_packages.sh deb

install-deb: build-deb ## Build DEB and reinstall on local system (sudo apt)
	@echo "[+] Installing Monster Control Center DEB package..."
	@DEB_FILE=$$(ls dist/Monster*Control*Center*$${APP_VERSION:-*}*.deb dist/monster*control*center*$${APP_VERSION:-*}*.deb dist/*$${APP_VERSION:-*}*.deb 2>/dev/null | sort -V | tail -n 1); \
	if [ -z "$$DEB_FILE" ]; then \
		DEB_FILE=$$(ls dist/*.deb 2>/dev/null | sort -V | tail -n 1); \
	fi; \
	if [ -n "$$DEB_FILE" ]; then \
		echo "[*] Installing package: $$DEB_FILE..."; \
		sudo apt-get update -qq 2>/dev/null || true; \
		sudo apt-get install --reinstall -y "$$DEB_FILE" 2>/dev/null || sudo apt-get install -y "$$DEB_FILE"; \
	else \
		echo "[-] Error: DEB package not found in dist/!" >&2; exit 1; \
	fi
	@update-desktop-database ~/.local/share/applications 2>/dev/null || true
	@echo "========================================================"
	@echo "  Monster Control Center DEB successfully installed!"
	@echo "========================================================"

build-appimage: ## Build AppImage package only
	@bash scripts/build_packages.sh appimage

install-appimage: build-appimage ## Build AppImage and integrate to user system (~/.local/bin)
	@echo "[+] Installing Monster Control Center AppImage to ~/.local/bin..."
	@mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons/hicolor/128x128/apps
	@APPIMAGE_PATH=$$(ls dist/Monster*Control*Center*$${APP_VERSION:-*}*.AppImage dist/monster*control*center*$${APP_VERSION:-*}*.AppImage dist/*$${APP_VERSION:-*}*.AppImage 2>/dev/null | sort -V | tail -n 1); \
	if [ -z "$$APPIMAGE_PATH" ]; then \
		APPIMAGE_PATH=$$(ls dist/*.AppImage 2>/dev/null | sort -V | tail -n 1); \
	fi; \
	if [ -n "$$APPIMAGE_PATH" ]; then \
		cp "$$APPIMAGE_PATH" ~/.local/bin/monster-control-center-gui && chmod +x ~/.local/bin/monster-control-center-gui; \
	fi
	@if [ -f icons/128x128.png ]; then \
		cp icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/monster-control-center.png; \
	fi
	@update-desktop-database ~/.local/share/applications 2>/dev/null || true
	@echo "========================================================"
	@echo "  Monster Control Center AppImage installed to ~/.local/bin/monster-control-center-gui"
	@echo "========================================================"

test: ## Run Rust tests
	@echo "[+] Running Rust tests..."
	@cargo test

bump: ## Bump version (patch|minor|major). Usage: make bump TYPE=minor
	@bash scripts/bump_version.sh $(or $(TYPE),patch)

bump-patch: ## Bump patch version (0.1.0 -> 0.1.1) across tauri.conf.json and Cargo.toml
	@bash scripts/bump_version.sh patch

bump-minor: ## Bump minor version (0.1.0 -> 0.2.0) across tauri.conf.json and Cargo.toml
	@bash scripts/bump_version.sh minor

bump-major: ## Bump major version (0.1.0 -> 1.0.0) across tauri.conf.json and Cargo.toml
	@bash scripts/bump_version.sh major

set-version: ## Set exact version. Usage: make set-version VERSION=0.2.0
	@if [ -z "$(VERSION)" ]; then echo "Usage: make set-version VERSION=X.Y.Z"; exit 1; fi
	@bash scripts/bump_version.sh --set $(VERSION)

release-patch: ## Bump patch version, commit, tag vX.Y.Z and push tag to origin
	@bash scripts/release_patch.sh patch

release: ## Bump version, commit, tag and push to origin. Usage: make release TYPE=patch
	@bash scripts/release_patch.sh $(or $(TYPE),patch)

lint: ## Run formatting and clippy checks
	@echo "[+] Checking Rust formatting and clippy..."
	@cargo fmt --check 2>/dev/null || true
	@cargo clippy -- -D warnings 2>/dev/null || true

clean: ## Clean build and dist output directories
	@echo "[+] Cleaning build artifacts..."
	@rm -rf dist target/release/bundle
	@echo "Clean completed."
