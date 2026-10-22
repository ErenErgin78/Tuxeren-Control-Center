# ==============================================================================
# Tuxeren Control Center (TxeCtrl) - Linux Makefile
# ==============================================================================

.PHONY: help setup setup-deps dev build build-rpm build-deb build-appimage install install-rpm install-deb install-appimage uninstall clean test lint bump bump-patch bump-minor bump-major set-version release release-patch

.DEFAULT_GOAL := help

help: ## Show available make targets
	@echo "Tuxeren Control Center (TxeCtrl) Commands:"
	@echo ""
	@LC_ALL=C grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'
	@echo ""

setup: ## Verify environment and toolchain
	@bash scripts/setup_env.sh

setup-deps: ## Check and prompt to install system-level build dependencies
	@bash scripts/setup_env.sh --install-deps

dev: ## Start Tuxeren Control Center in development mode
	@bash scripts/run.sh

build: ## Build release packages for all configured targets (rpm, deb, appimage)
	@bash scripts/build_packages.sh

install: ## Build release binary and install locally via install.sh
	@cargo build --release
	@bash install.sh

uninstall: ## Remove Tuxeren Control Center and desktop integration
	@bash scripts/uninstall.sh

APP_VERSION ?= $(shell grep -m1 '"version"' tauri.conf.json 2>/dev/null | cut -d '"' -f 4)

build-rpm: ## Build RPM package only
	@bash scripts/build_packages.sh rpm

install-rpm: build-rpm ## Build RPM and reinstall on local system (sudo dnf)
	@sudo -v
	@if (command -v rpm >/dev/null 2>&1 && (rpm -q tuxeren-control-center >/dev/null 2>&1 || rpm -q txectrl >/dev/null 2>&1 || rpm -q monster-control-center >/dev/null 2>&1)) || command -v txectrl >/dev/null 2>&1 || [ -f "$$HOME/.local/bin/txectrl" ]; then \
		echo "[*] Existing installation detected. Running uninstall script..."; \
		bash scripts/uninstall.sh; \
	fi
	@echo "[+] Installing freshly built Tuxeren Control Center RPM package..."
	@RPM_FILE=$$(ls dist/tuxeren-control-center-$${APP_VERSION:-*}-*.rpm dist/Tuxeren*Control*Center-$${APP_VERSION:-*}-*.rpm dist/txectrl-$${APP_VERSION:-*}-*.rpm dist/*$${APP_VERSION:-*}*.rpm 2>/dev/null | sort -V | tail -n 1); \
	if [ -z "$$RPM_FILE" ]; then \
		RPM_FILE=$$(ls dist/*.rpm 2>/dev/null | sort -V | tail -n 1); \
	fi; \
	if [ -z "$$RPM_FILE" ]; then \
		echo "[-] Error: RPM package not found in dist/!" >&2; exit 1; \
	fi; \
	echo "[*] Installing package: $$RPM_FILE..."; \
	sudo dnf install -y "$$RPM_FILE"
	@update-desktop-database ~/.local/share/applications 2>/dev/null || true
	@echo "========================================================"
	@echo "  Tuxeren Control Center RPM successfully installed!"
	@echo "========================================================"

build-deb: ## Build DEB package only
	@bash scripts/build_packages.sh deb

install-deb: build-deb ## Build DEB and reinstall on local system (sudo apt)
	@echo "[+] Installing Tuxeren Control Center DEB package..."
	@DEB_FILE=$$(ls dist/tuxeren-control-center*$${APP_VERSION:-*}*.deb dist/Tuxeren*Control*Center*$${APP_VERSION:-*}*.deb dist/txectrl*$${APP_VERSION:-*}*.deb dist/*$${APP_VERSION:-*}*.deb 2>/dev/null | sort -V | tail -n 1); \
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
	@echo "  Tuxeren Control Center DEB successfully installed!"
	@echo "========================================================"

build-appimage: ## Build AppImage package only
	@bash scripts/build_packages.sh appimage

install-appimage: build-appimage ## Build AppImage and integrate to user system (~/.local/bin)
	@echo "[+] Installing Tuxeren Control Center AppImage to ~/.local/bin..."
	@mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/metainfo \
		~/.local/share/icons/hicolor/256x256/apps \
		~/.local/share/icons/hicolor/128x128/apps \
		~/.local/share/icons/hicolor/scalable/apps
	@APPIMAGE_PATH=$$(ls dist/Tuxeren*Control*Center*$${APP_VERSION:-*}*.AppImage dist/txectrl*$${APP_VERSION:-*}*.AppImage dist/*$${APP_VERSION:-*}*.AppImage 2>/dev/null | sort -V | tail -n 1); \
	if [ -z "$$APPIMAGE_PATH" ]; then \
		APPIMAGE_PATH=$$(ls dist/*.AppImage 2>/dev/null | sort -V | tail -n 1); \
	fi; \
	if [ -n "$$APPIMAGE_PATH" ]; then \
		cp "$$APPIMAGE_PATH" ~/.local/bin/txectrl-gui && chmod +x ~/.local/bin/txectrl-gui; \
	fi
	@cp -f packaging/tuxeren-control-center.desktop ~/.local/share/applications/tuxeren-control-center.desktop 2>/dev/null || true
	@cp -f packaging/tuxeren-control-center.metainfo.xml ~/.local/share/metainfo/tuxeren-control-center.metainfo.xml 2>/dev/null || true
	@if [ -f icons/256x256.png ]; then \
		cp -f icons/256x256.png ~/.local/share/icons/hicolor/256x256/apps/tuxeren-control-center.png; \
	fi
	@if [ -f icons/128x128.png ]; then \
		cp -f icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/tuxeren-control-center.png; \
		cp -f icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/txectrl.png; \
	fi
	@if [ -f icons/tuxeren-control-center.svg ]; then \
		cp -f icons/tuxeren-control-center.svg ~/.local/share/icons/hicolor/scalable/apps/tuxeren-control-center.svg; \
	fi
	@update-desktop-database ~/.local/share/applications 2>/dev/null || true
	@gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
	@echo "========================================================"
	@echo "  Tuxeren Control Center AppImage installed to ~/.local/bin/txectrl-gui"
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
