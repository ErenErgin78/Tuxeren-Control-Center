#!/usr/bin/env bash
# Monster Control Center Package Builder (Tauri CLI)
# Builds Linux desktop packages (.rpm, .deb, .AppImage) using Tauri bundler.
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
BUNDLE_DIR="$PROJECT_ROOT/target/release/bundle"

TARGET_BUNDLES="${1:-}"

# Prevent DT_RELR strip issues on modern Linux distros (Fedora 40+)
export NO_STRIP=true

echo "========================================================"
echo "    Monster Control Center Package Builder (Tauri)"
echo "========================================================"

ensure_system_deps() {
    local missing=()
    local is_exe_target=false
    if [[ "$TARGET_BUNDLES" == *"exe"* ]] || [[ "$TARGET_BUNDLES" == *"nsis"* ]]; then
        is_exe_target=true
    fi

    if command -v dnf &>/dev/null; then
        local deps=(webkit2gtk4.1-devel openssl-devel alsa-lib-devel pulseaudio-utils ImageMagick rpm-build)
        if [ "$is_exe_target" = true ]; then
            deps+=(mingw64-gcc mingw64-binutils nsis)
        fi
        for pkg in "${deps[@]}"; do
            if ! rpm -q "$pkg" &>/dev/null; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            echo "[+] Installing missing packages with sudo dnf..."
            sudo dnf install -y "${missing[@]}"
        else
            echo "[✓] System build dependencies are already satisfied (dnf)."
        fi
    elif command -v apt-get &>/dev/null; then
        local deps=(libwebkit2gtk-4.1-dev libssl-dev libasound2-dev pulseaudio-utils imagemagick rpm)
        if [ "$is_exe_target" = true ]; then
            deps+=(gcc-mingw-w64-x86-64 nsis)
        fi
        for pkg in "${deps[@]}"; do
            if ! dpkg -s "$pkg" 2>/dev/null | grep -q "Status: install ok installed"; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            echo "[+] Installing missing packages with sudo apt..."
            sudo apt-get update -qq
            sudo apt-get install -y "${missing[@]}"
        else
            echo "[✓] System build dependencies are already satisfied (apt)."
        fi
    elif command -v pacman &>/dev/null; then
        local deps=(webkit2gtk-4.1 openssl alsa-lib libpulse imagemagick rpm-tools)
        if [ "$is_exe_target" = true ]; then
            deps+=(mingw-w64-gcc nsis)
        fi
        for pkg in "${deps[@]}"; do
            if ! pacman -Q "$pkg" &>/dev/null; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            echo "[+] Installing missing packages with sudo pacman..."
            sudo pacman -S --needed --noconfirm "${missing[@]}"
        else
            echo "[✓] System build dependencies are already satisfied (pacman)."
        fi
    fi
}

ensure_system_deps

if ! command -v cargo &>/dev/null; then
    echo "[-] Error: 'cargo' is not installed." >&2
    exit 1
fi

if [ -f "$PROJECT_ROOT/ui/package.json" ] && [ ! -d "$PROJECT_ROOT/ui/node_modules" ]; then
    echo "[+] Installing frontend dependencies..."
    (cd "$PROJECT_ROOT/ui" && npm install)
fi

mkdir -p "$DIST_DIR"
IS_WINDOWS=false
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) IS_WINDOWS=true ;;
esac

WANTS_EXE=false
if [[ "$TARGET_BUNDLES" == *"exe"* ]] || [[ "$TARGET_BUNDLES" == *"nsis"* ]] || [ "$IS_WINDOWS" = true ]; then
    WANTS_EXE=true
fi

if [ -n "$TARGET_BUNDLES" ]; then
    IFS=',' read -ra BUNDLE_ARR <<< "$TARGET_BUNDLES"
    for b in "${BUNDLE_ARR[@]}"; do
        b_clean="$(echo "$b" | xargs)"
        if [ -n "$b_clean" ]; then
            rm -rf "$BUNDLE_DIR/$b_clean"
            case "$b_clean" in
                rpm) rm -f "$DIST_DIR"/*.rpm ;;
                deb) rm -f "$DIST_DIR"/*.deb ;;
                appimage) rm -f "$DIST_DIR"/*.AppImage ;;
                exe|nsis)
                    rm -f "$DIST_DIR"/*.exe
                    rm -rf "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/bundle/nsis"
                    ;;
            esac
        fi
    done
else
    rm -rf "$BUNDLE_DIR"
    rm -f "$DIST_DIR"/*.rpm "$DIST_DIR"/*.deb "$DIST_DIR"/*.AppImage "$DIST_DIR"/*.exe 2>/dev/null || true
fi

BUILD_CMD() {
    if command -v cargo-tauri &>/dev/null; then
        cargo tauri "$@"
    else
        npx --yes @tauri-apps/cli "$@"
    fi
}

if [ "$WANTS_EXE" = true ] && [ "$IS_WINDOWS" = false ]; then
    echo "[+] Windows (.exe) target requested on Linux. Preparing cross-compilation..."
    WINDOWS_TARGET="x86_64-pc-windows-gnu"

    if ! rustup target list | grep -q "${WINDOWS_TARGET} (installed)"; then
        echo "[+] Adding rust target ${WINDOWS_TARGET}..."
        rustup target add "$WINDOWS_TARGET"
    fi

    echo "[+] Building release binary for $WINDOWS_TARGET..."
    (cd "$PROJECT_ROOT" && cargo build --release --target "$WINDOWS_TARGET" --bin monster-control-center-gui)

    WIN_STANDALONE="$PROJECT_ROOT/target/$WINDOWS_TARGET/release/monster-control-center-gui.exe"
    if [ -f "$WIN_STANDALONE" ]; then
        cp -p "$WIN_STANDALONE" "$DIST_DIR/"
        echo "[✓] Standalone Windows executable copied to $DIST_DIR/monster-control-center-gui.exe"
    fi

    if command -v makensis &>/dev/null && command -v x86_64-w64-mingw32-windres &>/dev/null; then
        echo "[+] Running Tauri NSIS installer build for $WINDOWS_TARGET..."
        (cd "$PROJECT_ROOT" && BUILD_CMD build --target "$WINDOWS_TARGET" --bundles nsis) || true
        WIN_BUNDLE_DIR="$PROJECT_ROOT/target/$WINDOWS_TARGET/release/bundle"
        if [ -d "$WIN_BUNDLE_DIR" ]; then
            while IFS= read -r -d '' file; do
                cp -p "$file" "$DIST_DIR/"
            done < <(find "$WIN_BUNDLE_DIR" -type f -name "*.exe" -print0)
        fi
    else
        echo "[!] Note: 'makensis' or 'x86_64-w64-mingw32-windres' not detected. Standalone .exe was generated; install 'nsis' and 'mingw-w64' for .exe setup bundler."
    fi

    if [ "$TARGET_BUNDLES" = "exe" ] || [ "$TARGET_BUNDLES" = "nsis" ]; then
        echo ""
        echo "========================================================"
        echo "  Windows Package Build Complete! Available in: $DIST_DIR"
        echo "--------------------------------------------------------"
        ls -lh "$DIST_DIR"/*.exe 2>/dev/null || true
        echo "========================================================"
        exit 0
    fi
fi

echo "[+] Building release binaries..."
(cd "$PROJECT_ROOT" && cargo build --release)

echo "[+] Running Tauri build..."
BUILD_ARGS=()
if [ -n "$TARGET_BUNDLES" ]; then
    # Filter out exe from tauri bundles argument on Linux
    BUNDLES_FILTERED=$(echo "$TARGET_BUNDLES" | sed -E 's/(^|,)exe($|,)/\1/g; s/(^|,)nsis($|,)/\1/g; s/^,//; s/,$//')
    if [ -n "$BUNDLES_FILTERED" ]; then
        echo "    Target bundles: $BUNDLES_FILTERED"
        BUILD_ARGS+=(--bundles "$BUNDLES_FILTERED")
    fi
else
    echo "    Target bundles: deb, appimage, rpm"
    BUILD_ARGS+=(--bundles "deb,appimage,rpm")
fi

if [ ${#BUILD_ARGS[@]} -gt 0 ]; then
    (cd "$PROJECT_ROOT" && BUILD_CMD build "${BUILD_ARGS[@]}")
fi

echo ""
echo "[+] Collecting built packages into $DIST_DIR..."
FOUND_PACKAGES=0
if [ -d "$BUNDLE_DIR" ]; then
    while IFS= read -r -d '' file; do
        cp -p "$file" "$DIST_DIR/"
        FOUND_PACKAGES=$((FOUND_PACKAGES + 1))
    done < <(find "$BUNDLE_DIR" -type f \( -name "*.rpm" -o -name "*.deb" -o -name "*.AppImage" -o -name "*.exe" \) -print0)
fi

WIN_BUNDLE_DIR="$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/bundle"
if [ -d "$WIN_BUNDLE_DIR" ]; then
    while IFS= read -r -d '' file; do
        cp -p "$file" "$DIST_DIR/"
        FOUND_PACKAGES=$((FOUND_PACKAGES + 1))
    done < <(find "$WIN_BUNDLE_DIR" -type f -name "*.exe" -print0)
fi

WIN_STANDALONE="$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/monster-control-center-gui.exe"
if [ -f "$WIN_STANDALONE" ] && [ ! -f "$DIST_DIR/monster-control-center-gui.exe" ]; then
    cp -p "$WIN_STANDALONE" "$DIST_DIR/"
    FOUND_PACKAGES=$((FOUND_PACKAGES + 1))
fi

echo "========================================================"
if [ "$FOUND_PACKAGES" -gt 0 ] || ls "$DIST_DIR"/*.{rpm,deb,AppImage,exe} &>/dev/null; then
    echo "  Package Build Complete! Available in: $DIST_DIR"
    echo "--------------------------------------------------------"
    ls -lh "$DIST_DIR"/*.{rpm,deb,AppImage,exe} 2>/dev/null || true
else
    echo "[-] Warning: No package files were found in $BUNDLE_DIR"
fi
echo "========================================================"
