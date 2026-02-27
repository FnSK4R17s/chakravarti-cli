# ==============================================================================
# Chakravarti CLI - Justfile
# ==============================================================================
# Primary task runner for development. Use `just --list` to see all recipes.
# 
# Installation:
#   macOS:   brew install just
#   Linux:   cargo install just
#   Windows: cargo install just
# ==============================================================================

# Project configuration
binary_name := "ckrv"
npm_dir := "npm"
bin_dir := npm_dir / "bin"
rust_bin := "target" / "release" / binary_name

# Skip Docker builds? Set via: CKRV_SKIP_DOCKER=true just install
skip-docker := env_var_or_default("CKRV_SKIP_DOCKER", "false")

# ==============================================================================
# DEFAULT
# ==============================================================================

# List all available recipes
default:
    @just --list

# ==============================================================================
# BUILD & INSTALL
# ==============================================================================

# Build the Rust binary in release mode
build: ui-setup
    @echo "Building Rust binary in release mode..."
    cargo build --release -p ckrv-cli

# Build in development mode (faster, with debug symbols)
build-dev:
    cargo build -p ckrv-cli

# Install: build, Docker images (optional), npm link
# Optional arg: just install skip-docker=true
install *args:
    #!/usr/bin/env bash
    set -euo pipefail

    # Resolve skip-docker from env default, allow CLI override via positional args
    effective_skip="{{ skip-docker }}"
    for arg in {{args}}; do
        case "$arg" in
            skip-docker=true|skip_docker=true)
                effective_skip="true"
                ;;
            skip-docker=false|skip_docker=false)
                effective_skip="false"
                ;;
        esac
    done

    # Stop/cleanup containers
    docker ps -q --filter "name=ckrv-" | xargs -r docker stop 2>/dev/null || true
    docker ps -aq --filter "name=ckrv-" | xargs -r docker rm 2>/dev/null || true
    
    if [ "$effective_skip" = "true" ]; then
        echo "Skipping Docker images (skip-docker=true)"
        echo ""
        echo "Note: Agent sandboxing requires Docker images. Pull them later with:"
        echo "  just docker-pull"
    else
        echo "Pulling Docker agent images from GHCR..."
        docker pull {{ ghcr_prefix }}/ckrv-claude:latest
        docker pull {{ ghcr_prefix }}/ckrv-codex:latest
        docker pull {{ ghcr_prefix }}/ckrv-kilo:latest
    fi
    
    # Install CLI binary to ~/.cargo/bin in a path-agnostic way.
    cargo install --path crates/ckrv-cli --bin {{ binary_name }} --force

    built_bin="$HOME/.cargo/bin/{{ binary_name }}"
    if [ ! -f "$built_bin" ]; then
        echo "Error: installed binary not found at $built_bin" >&2
        exit 1
    fi

    mkdir -p {{ bin_dir }}
    cp "$built_bin" {{ bin_dir }}/{{ binary_name }}
    chmod +x {{ bin_dir }}/{{ binary_name }}
    cd {{ npm_dir }} && npm link
    echo ""
    echo "✓ Chakravarti CLI installed and linked successfully!"
    echo "Run 'ckrv --version' to verify."

# Install without Docker (shorthand)
install-quick:
    CKRV_SKIP_DOCKER=true just install

# Remove build artifacts
clean:
    cargo clean
    rm -rf {{ bin_dir }}/{{ binary_name }}

# ==============================================================================
# UI FRONTEND
# ==============================================================================

# Install UI frontend dependencies
ui-setup:
    @echo "Installing UI frontend dependencies..."
    cd crates/ckrv-ui/frontend && npm install

# Build frontend for production
ui-build: ui-setup
    cd crates/ckrv-ui/frontend && npm run build

# Run frontend dev server
ui-dev: ui-setup
    cd crates/ckrv-ui/frontend && npm run dev

# ==============================================================================
# AI INTERFACE
# ==============================================================================

# Generate SKILL.md for AI agents
skill:
    @echo "Generating SKILL.md..."
    @mkdir -p .agent/skills/chakravarti-cli
    cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
    @echo "Validating SKILL.md..."
    @if command -v uvx >/dev/null 2>&1; then \
        uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli && echo "✓ SKILL.md is valid"; \
    else \
        echo "⚠ uvx not found, skipping validation. Install with: pip install uv"; \
    fi

# Build MCP server binary
mcp:
    @echo "Building MCP server..."
    cargo build --release -p ckrv-mcp
    @echo "✓ MCP server built: target/release/ckrv-mcp"

# Build MCP and show Claude Desktop config
install-mcp: mcp
    @echo ""
    @echo "✓ MCP server installed!"
    @echo ""
    @echo "Add the following to your Claude Desktop config:"
    @echo "(macOS: ~/Library/Application Support/Claude/claude_desktop_config.json)"
    @echo "(Linux: ~/.config/claude/claude_desktop_config.json)"
    @echo ""
    @echo '{'
    @echo '  "mcpServers": {'
    @echo '    "chakravarti": {'
    @echo '      "command": "{{ justfile_directory() }}/target/release/ckrv-mcp"'
    @echo '    }'
    @echo '  }'
    @echo '}'
    @echo ""

# ==============================================================================
# DEVELOPMENT
# ==============================================================================

# Run all linters (Rust + frontend)
lint:
    cargo clippy --workspace -- -D warnings
    cd crates/ckrv-ui/frontend && npm run lint

# Run tests (optional: just test integration, just test unit, just test --test name)
test filter="":
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{ filter }}" in
        "")
            cargo test --workspace
            ;;
        integration)
            cargo test --workspace --test integration
            ;;
        unit)
            cargo test --workspace --lib
            ;;
        *)
            cargo test --workspace "{{ filter }}"
            ;;
    esac

# Format code (Rust + frontend)
fmt:
    cargo fmt --all
    cd crates/ckrv-ui/frontend && npx prettier --write . || true

# Watch mode: rebuild on changes
watch:
    cargo watch -x "build --release -p ckrv-cli"

# ==============================================================================
# DOCKER OPERATIONS
# ==============================================================================

# GHCR registry prefix
ghcr_prefix := "ghcr.io/fnsk4r17s"

# Pull pre-built agent images from GHCR
docker-pull:
    @echo "Pulling Docker agent images from GHCR..."
    docker pull {{ ghcr_prefix }}/ckrv-claude:latest
    docker pull {{ ghcr_prefix }}/ckrv-codex:latest
    docker pull {{ ghcr_prefix }}/ckrv-kilo:latest
    @echo "✓ Docker images pulled"

# Build all Docker agent images locally (for debugging)
docker-build:
    @echo "Building Docker agent images locally..."
    docker build -t {{ ghcr_prefix }}/ckrv-claude:latest -f docker/Dockerfile.claude docker/
    docker build -t {{ ghcr_prefix }}/ckrv-codex:latest -f docker/Dockerfile.codex docker/
    docker build -t {{ ghcr_prefix }}/ckrv-kilo:latest -f docker/Dockerfile.kilo docker/
    @echo "✓ Docker images built"

# Stop all ckrv containers
docker-stop:
    @echo "Stopping ckrv containers..."
    -docker ps -q --filter "name=ckrv-" | xargs -r docker stop 2>/dev/null || true
    -docker ps -aq --filter "name=ckrv-" | xargs -r docker rm 2>/dev/null || true
    @echo "✓ Containers stopped"

# ==============================================================================
# DOCUMENTATION
# ==============================================================================

# Generate Rust documentation
docs:
    cargo doc --no-deps --open

# ==============================================================================
# CI
# ==============================================================================

# Run full CI pipeline locally (build, lint, test)
ci: build lint test
    @echo ""
    @echo "✓ CI pipeline passed"

# ==============================================================================
# RELEASE
# ==============================================================================

# Bump version and prepare release (just release patch|minor|major)
release bump:
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Get current version from workspace Cargo.toml
    current=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    
    # Parse version parts
    major=$(echo "$current" | cut -d. -f1)
    minor=$(echo "$current" | cut -d. -f2)
    patch=$(echo "$current" | cut -d. -f3)
    
    # Calculate new version
    case "{{ bump }}" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            echo "Error: bump must be 'major', 'minor', or 'patch'"
            exit 1
            ;;
    esac
    
    new_version="${major}.${minor}.${patch}"
    
    echo "Bumping version: $current -> $new_version"
    
    # Update workspace Cargo.toml
    sed -i.bak "s/^version = \"\(.*\)\"/version = \"$new_version\"/" Cargo.toml
    rm -f Cargo.toml.bak
    
    # Update all crate Cargo.toml files
    for crate in crates/*/; do
        if [ -f "${crate}Cargo.toml" ]; then
            sed -i.bak "s/^version = \"\(.*\)\"/version = \"$new_version\"/" "${crate}Cargo.toml"
            rm -f "${crate}Cargo.toml.bak"
        fi
    done
    
    echo "✓ Version bumped to $new_version"
    echo ""
    echo "Next steps:"
    echo "  1. Review changes: git diff"
    echo "  2. Commit: git commit -am 'chore: release v$new_version'"
    echo "  3. Tag: git tag -a v$new_version -m 'Release v$new_version'"

