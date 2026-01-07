#!/bin/bash
# Script to bump npm package.json versions in sync with Cargo releases
# Called by cargo-release as a pre-release hook
#
# Usage: ./scripts/bump-npm-versions.sh <version>
# Example: ./scripts/bump-npm-versions.sh 0.2.0

set -e

VERSION="${1}"

if [ -z "$VERSION" ]; then
    echo "Error: Version argument required"
    echo "Usage: $0 <version>"
    exit 1
fi

# Get the workspace root (script is in scripts/, so go up one level)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔄 Syncing npm package versions to v${VERSION}..."
echo "   Workspace root: $WORKSPACE_ROOT"

# Root npm package (for npm link)
if [ -f "$WORKSPACE_ROOT/npm/package.json" ]; then
    echo "  → Updating npm/package.json"
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" "$WORKSPACE_ROOT/npm/package.json"
fi

# UI frontend package
if [ -f "$WORKSPACE_ROOT/crates/ckrv-ui/frontend/package.json" ]; then
    echo "  → Updating crates/ckrv-ui/frontend/package.json"
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" "$WORKSPACE_ROOT/crates/ckrv-ui/frontend/package.json"
fi

# Update package-lock.json files (regenerate them)
echo "  → Regenerating package-lock.json files..."

if [ -f "$WORKSPACE_ROOT/crates/ckrv-ui/frontend/package.json" ]; then
    (cd "$WORKSPACE_ROOT/crates/ckrv-ui/frontend" && npm install --package-lock-only --silent 2>/dev/null || true)
fi

echo "✅ npm package versions updated to v${VERSION}"
