#!/usr/bin/env bash
# Tuxeren Control Center (TxeCtrl) Release & Tag Script
#
# 1. Bumps version via scripts/bump_version.sh (default: patch)
# 2. Commits updated version files
# 3. Creates 'v[new_version]' git tag
# 4. Pushes branch and tag to origin
#
# Usage:
#   ./scripts/release_patch.sh              # patch bump, commit, tag and push
#   ./scripts/release_patch.sh patch        # same as above
#   ./scripts/release_patch.sh minor        # minor bump
#   ./scripts/release_patch.sh major        # major bump
#   ./scripts/release_patch.sh --dry-run    # simulate without modifying git
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
TAURI_CONF="$PROJECT_ROOT/tauri.conf.json"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
CARGO_LOCK="$PROJECT_ROOT/Cargo.lock"
PACKAGE_JSON="$PROJECT_ROOT/ui/package.json"
PACKAGE_LOCK="$PROJECT_ROOT/ui/package-lock.json"
BUMP_SCRIPT="$PROJECT_ROOT/scripts/bump_version.sh"

usage() {
    echo "Usage: $0 [patch|minor|major] [--dry-run]" >&2
    echo "Examples:" >&2
    echo "  $0                  # Default: patch bump, tag and push" >&2
    echo "  $0 patch            # Patch bump (0.1.0 -> 0.1.1)" >&2
    echo "  $0 minor            # Minor bump (0.1.0 -> 0.2.0)" >&2
    echo "  $0 --dry-run        # Dry run simulation" >&2
}

BUMP_TYPE="patch"
DRY_RUN=0

for arg in "$@"; do
    case "$arg" in
        patch|minor|major)
            BUMP_TYPE="$arg"
            ;;
        --dry-run|-n)
            DRY_RUN=1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "[-] Error: Unknown argument '$arg'" >&2
            usage
            exit 1
            ;;
    esac
done

if [ ! -f "$TAURI_CONF" ]; then
    echo "[-] Error: $TAURI_CONF not found." >&2
    exit 1
fi

HEAD_VERSION="$(git -C "$PROJECT_ROOT" show HEAD:tauri.conf.json 2>/dev/null | python3 -c "import json, sys; print(json.load(sys.stdin)['version'])" 2>/dev/null || echo "")"
CURRENT_VERSION="$(python3 -c "import json; print(json.load(open('$TAURI_CONF'))['version'])")"

if [ "$DRY_RUN" -eq 1 ]; then
    echo "[*] DRY-RUN MODE ACTIVE - No files will be changed and no git operations pushed."
    echo "    Current version (disk): $CURRENT_VERSION"
    if [ -n "$HEAD_VERSION" ] && [ "$HEAD_VERSION" != "$CURRENT_VERSION" ]; then
        echo "    HEAD version          : $HEAD_VERSION"
        echo "    [*] Version files already updated on disk ($HEAD_VERSION -> $CURRENT_VERSION)."
        SIMULATED_VERSION="$CURRENT_VERSION"
    else
        echo "    Bump type             : $BUMP_TYPE"
        IFS='.' read -r MA MI PA <<< "$CURRENT_VERSION"
        case "$BUMP_TYPE" in
            patch) PA=$((PA + 1)) ;;
            minor) MI=$((MI + 1)); PA=0 ;;
            major) MA=$((MA + 1)); MI=0; PA=0 ;;
        esac
        SIMULATED_VERSION="$MA.$MI.$PA"
    fi
    SIMULATED_TAG="v$SIMULATED_VERSION"
    echo "    Target version        : $SIMULATED_VERSION"
    echo "    Target tag            : $SIMULATED_TAG"
    echo ""
    echo "[*] Steps to execute:"
    echo "    1. Synchronize version ($SIMULATED_VERSION)"
    echo "    2. git add version files"
    echo "    3. git commit -m 'chore(release): $SIMULATED_TAG'"
    echo "    4. git push origin \$(current_branch)"
    echo "    5. git tag $SIMULATED_TAG"
    echo "    6. git push origin $SIMULATED_TAG"
    exit 0
fi

# Git checks
if ! git -C "$PROJECT_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[-] Error: $PROJECT_ROOT is not a Git repository." >&2
    exit 1
fi

if ! git -C "$PROJECT_ROOT" remote get-url origin >/dev/null 2>&1; then
    echo "[-] Error: Remote 'origin' not found." >&2
    exit 1
fi

MODIFIED_NON_VERSION="$(git -C "$PROJECT_ROOT" diff --name-only HEAD 2>/dev/null | grep -vE 'tauri\.conf\.json|Cargo\.toml|Cargo\.lock|ui/package.*\.json' || true)"
if [ -n "$MODIFIED_NON_VERSION" ] && [ "${ALLOW_DIRTY:-0}" != "1" ]; then
    echo "[-] Error: Uncommitted changes detected in working tree:" >&2
    echo "$MODIFIED_NON_VERSION" >&2
    echo "    Commit or stash these changes first (override: ALLOW_DIRTY=1 $0)." >&2
    exit 1
fi

# 1. Version Bump
if [ -n "$HEAD_VERSION" ] && [ "$HEAD_VERSION" != "$CURRENT_VERSION" ]; then
    echo "[*] Version already updated on disk ($HEAD_VERSION -> $CURRENT_VERSION), using current version."
else
    echo "[+] 1/4: Bumping version ($BUMP_TYPE)..."
    bash "$BUMP_SCRIPT" "$BUMP_TYPE"
fi

NEW_VERSION="$(python3 -c "import json; print(json.load(open('$TAURI_CONF'))['version'])")"
TAG="v$NEW_VERSION"

if git -C "$PROJECT_ROOT" rev-parse "$TAG" >/dev/null 2>&1; then
    echo "[-] Error: Tag '$TAG' already exists in local repo." >&2
    exit 1
fi

# 2. Stage & Commit
stage_file() {
    local f="$1"
    if [ -f "$f" ]; then
        if ! git -C "$PROJECT_ROOT" check-ignore -q "$f" 2>/dev/null; then
            git -C "$PROJECT_ROOT" add "$f"
        fi
    fi
}

echo "[+] 2/4: Committing release files..."
stage_file "$TAURI_CONF"
stage_file "$CARGO_TOML"
stage_file "$CARGO_LOCK"
stage_file "$PACKAGE_JSON"
stage_file "$PACKAGE_LOCK"

if ! git -C "$PROJECT_ROOT" diff --cached --quiet; then
    git -C "$PROJECT_ROOT" commit -m "chore(release): $TAG"
    echo "    [✓] Created commit: chore(release): $TAG"
else
    echo "    [*] No changes staged or already committed."
fi

CURRENT_BRANCH="$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")"
if [ -n "$CURRENT_BRANCH" ] && [ "$CURRENT_BRANCH" != "HEAD" ]; then
    echo "    [+] Pushing branch: origin $CURRENT_BRANCH..."
    git -C "$PROJECT_ROOT" push origin "$CURRENT_BRANCH"
fi

# 3. Create Tag
echo "[+] 3/4: Creating git tag: $TAG..."
git -C "$PROJECT_ROOT" tag "$TAG"

# 4. Push Tag
echo "[+] 4/4: Pushing git tag: origin $TAG..."
git -C "$PROJECT_ROOT" push origin "$TAG"

echo ""
echo "========================================================"
echo "  [✓] Success! $TAG released and pushed to origin."
echo "========================================================"
