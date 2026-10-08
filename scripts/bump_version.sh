#!/usr/bin/env bash
# Monster Control Center Version Bumper
# Synchronizes version across:
#   - tauri.conf.json  ("version")
#   - Cargo.toml       ([package] version)
#   - Cargo.lock       (monster-control-center package)
#
# Usage:
#   ./scripts/bump_version.sh patch          # 0.1.0 -> 0.1.1
#   ./scripts/bump_version.sh minor          # 0.1.0 -> 0.2.0
#   ./scripts/bump_version.sh major          # 0.1.0 -> 1.0.0
#   ./scripts/bump_version.sh 0.2.0          # set explicit version
#   ./scripts/bump_version.sh --set 0.2.0    # set explicit version
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
TAURI_CONF="$PROJECT_ROOT/tauri.conf.json"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
CARGO_LOCK="$PROJECT_ROOT/Cargo.lock"
PACKAGE_JSON="$PROJECT_ROOT/ui/package.json"
PACKAGE_LOCK="$PROJECT_ROOT/ui/package-lock.json"

usage() {
    echo "Usage: $0 {patch|minor|major|X.Y.Z|--set X.Y.Z}" >&2
    echo "Examples: $0 patch | $0 minor | $0 major | $0 0.2.0 | $0 --set 0.2.0" >&2
}

BUMP=""
if [ $# -eq 0 ]; then
    usage; exit 1
elif [ "$1" = "--set" ]; then
    [ $# -eq 2 ] || { usage; exit 1; }
    BUMP="$2"
elif [ $# -eq 1 ]; then
    BUMP="$1"
else
    usage; exit 1
fi

if [ ! -f "$TAURI_CONF" ] || [ ! -f "$CARGO_TOML" ]; then
    echo "[-] Error: Version files not found." >&2
    exit 1
fi

CURRENT="$(python3 -c "import json;print(json.load(open('$TAURI_CONF'))['version'])")"
CARGO_VER="$(python3 -c "import re;print(re.search(r'^version\s*=\s*\"([^\"]+)\"', open('$CARGO_TOML').read(), re.M).group(1))")"

if [ "$CURRENT" != "$CARGO_VER" ]; then
    echo "[!] Warning: versions out of sync: tauri.conf.json=$CURRENT, Cargo.toml=$CARGO_VER" >&2
    echo "[*] Using tauri.conf.json ($CURRENT) as base." >&2
fi

SEMVER_RE='^[0-9]+\.[0-9]+\.[0-9]+$'
NEW=""

case "$BUMP" in
    patch|minor|major)
        IFS='.' read -r MA MI PA <<< "$CURRENT"
        if ! [[ "$CURRENT" =~ $SEMVER_RE ]]; then
            echo "[-] Error: current version is not semver: $CURRENT" >&2
            exit 1
        fi
        case "$BUMP" in
            patch) PA=$((PA + 1)) ;;
            minor) MI=$((MI + 1)); PA=0 ;;
            major) MA=$((MA + 1)); MI=0; PA=0 ;;
        esac
        NEW="$MA.$MI.$PA"
        ;;
    *)
        if [[ "$BUMP" =~ $SEMVER_RE ]]; then
            NEW="$BUMP"
        else
            echo "[-] Error: invalid version format '$BUMP'." >&2
            usage; exit 1
        fi
        ;;
esac

if [ "$NEW" = "$CURRENT" ]; then
    echo "[=] Version is already $NEW, nothing to do."
    exit 0
fi

echo "[+] Bumping version: $CURRENT -> $NEW"

python3 - "$TAURI_CONF" "$CARGO_TOML" "$CARGO_LOCK" "$PACKAGE_JSON" "$PACKAGE_LOCK" "$NEW" <<'EOF'
import json, re, sys, os

tauri_conf, cargo_toml, cargo_lock, package_json, package_lock, new = sys.argv[1:7]

# 1. tauri.conf.json
with open(tauri_conf) as f:
    data = json.load(f)
data["version"] = new
with open(tauri_conf, "w") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
    f.write("\n")

# 2. Cargo.toml -> [package] version only
with open(cargo_toml) as f:
    content = f.read()
in_package, done, out = False, False, []
for line in content.splitlines(keepends=True):
    s = line.strip()
    if s.startswith("["):
        in_package = (s == "[package]")
    if in_package and not done and re.match(r'^version\s*=', s):
        line = re.sub(r'^(\s*version\s*=\s*")[^"]+(".*)$', rf'\g<1>{new}\g<2>', line)
        done = True
    out.append(line)
assert done, "Cargo.toml [package] version not found"
with open(cargo_toml, "w") as f:
    f.writelines(out)

# 3. Cargo.lock -> name = "monster-control-center" package block
if os.path.exists(cargo_lock):
    with open(cargo_lock) as f:
        lines = f.readlines()
    out, i, changed = [], 0, False
    while i < len(lines):
        out.append(lines[i])
        if lines[i].strip() == 'name = "monster-control-center"':
            j = i + 1
            while j < len(lines) and lines[j].strip() != "":
                if re.match(r'^version\s*=', lines[j].strip()):
                    out.append(re.sub(r'"[^"]+"', f'"{new}"', lines[j], count=1))
                    changed = True
                else:
                    out.append(lines[j])
                j += 1
            i = j - 1
        i += 1
    if changed:
        with open(cargo_lock, "w") as f:
            f.writelines(out)
        print("    [~] Cargo.lock synchronized")

# 4. ui/package.json & lock (if existing)
if os.path.exists(package_json):
    with open(package_json) as f:
        pkg = json.load(f)
    pkg["version"] = new
    with open(package_json, "w") as f:
        json.dump(pkg, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print("    [~] ui/package.json synchronized")

if os.path.exists(package_lock):
    with open(package_lock) as f:
        lock = json.load(f)
    lock["version"] = new
    if isinstance(lock.get("packages"), dict) and "" in lock["packages"]:
        lock["packages"][""]["version"] = new
    with open(package_lock, "w") as f:
        json.dump(lock, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print("    [~] ui/package-lock.json synchronized")
EOF

echo "    [~] tauri.conf.json -> $NEW"
echo "    [~] Cargo.toml      -> $NEW"
echo "[✓] All versions synchronized to $NEW."
