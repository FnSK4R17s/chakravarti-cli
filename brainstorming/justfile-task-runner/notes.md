# Justfile as Primary Developer Task Runner

**Issue**: (New - not yet created)
**Created**: 2026-02-22
**Status**: Draft
**Bugfixes**: [bugfix01.md](./bugfix01.md), [bugfix02.md](./bugfix02.md)

## Problem Statement

The Makefile, while functional, has several limitations for developer ergonomics:
- Make syntax is archaic (tabs required, confusing escaping rules)
- No built-in parameter handling (must use `$(VAR)` with make's weird semantics)
- Conditional logic requires shell escape gymnastics
- No native support for listing recipes with descriptions in a structured way
- Error messages are cryptic (`missing separator` instead of clear errors)

Just offers modern ergonomics: built-in help, proper parameter handling, cross-platform path handling, and clearer syntax while maintaining Makefile compatibility.

## Current State

**Makefile targets** (10 recipes):
| Target | Description | Dependencies |
|--------|-------------|--------------|
| `build` | Build Rust release binary | `ui-setup` |
| `install` | Full install (build + Docker + npm link) | `build` |
| `clean` | Remove build artifacts | - |
| `skill` | Generate SKILL.md for AI agents | - |
| `mcp` | Build MCP server binary | - |
| `install-mcp` | Build MCP + print Claude config | `mcp` |
| `ui-setup` | Install frontend deps | - |
| `help` | Show help (default) | - |

**Pain Points**:
1. Docker builds in `install` are slow and unnecessary in containerized dev environments
2. No way to skip Docker for local-only development
3. `ui-setup` runs on every `build` even if deps haven't changed
4. No parallel execution option
5. Make's help text is manually maintained (could drift from actual targets)

## Proposed Solution

Adopt Justfile as primary runner with Makefile as thin compatibility shim.

### Recipe Design

```just
# justfile

# Project configuration
binary_name := "ckrv"
npm_dir := "npm"
bin_dir := npm_dir / "bin"
rust_bin := "target" / "release" / binary_name

# Skip Docker builds? Set via: just install skip-docker=true
skip-docker := env_var_or_default("CKRV_SKIP_DOCKER", "false")

# =============================================================================
# BUILD & INSTALL
# =============================================================================

# Build the Rust binary in release mode (default)
build: ui-setup
    cargo build --release -p ckrv-cli

# Build in development mode (faster, with debug symbols)
build-dev:
    cargo build -p ckrv-cli

# Install: build, Docker images (optional), npm link
install: build
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Stop/cleanup containers
    docker ps -q --filter "name=ckrv-" | xargs -r docker stop 2>/dev/null || true
    docker ps -aq --filter "name=ckrv-" | xargs -r docker rm 2>/dev/null || true
    
    if {{ skip-docker == "true" }}; then
        echo "Skipping Docker builds (skip-docker=true)"
    else
        echo "Building Docker agent images..."
        docker build -t ckrv-claude:latest -f docker/Dockerfile.claude docker/
        docker build -t ckrv-codex:latest -f docker/Dockerfile.codex docker/
        docker build -t ckrv-kilo:latest -f docker/Dockerfile.kilo docker/
    fi
    
    mkdir -p {{ bin_dir }}
    cp {{ rust_bin }} {{ bin_dir }}/{{ binary_name }}
    chmod +x {{ bin_dir }}/{{ binary_name }}
    cd {{ npm_dir }} && npm link
    cp {{ rust_bin }} ~/.cargo/bin/{{ binary_name }}
    echo "✓ Chakravarti CLI installed!"

# Install without Docker (shorthand)
install-quick: (install skip-docker="true")

# Remove build artifacts
clean:
    cargo clean
    rm -rf {{ bin_dir }}/{{ binary_name }}

# =============================================================================
# UI FRONTEND
# =============================================================================

# Install UI frontend dependencies
ui-setup:
    cd crates/ckrv-ui/frontend && npm install

# Build frontend for production
ui-build: ui-setup
    cd crates/ckrv-ui/frontend && npm run build

# Run frontend dev server
ui-dev: ui-setup
    cd crates/ckrv-ui/frontend && npm run dev

# =============================================================================
# AI INTERFACE
# =============================================================================

# Generate SKILL.md for AI agents
skill:
    mkdir -p .agent/skills/chakravarti-cli
    cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
    if command -v uvx >/dev/null 2>&1; then \
        uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli && echo "✓ SKILL.md valid"; \
    else \
        echo "⚠ uvx not found, skipping validation"; \
    fi

# Build MCP server binary
mcp:
    cargo build --release -p ckrv-mcp
    echo "✓ MCP server built: target/release/ckrv-mcp"

# Build MCP and show Claude Desktop config
install-mcp: mcp
    echo "Add to Claude Desktop config:"
    echo '{ "mcpServers": { "chakravarti": { "command": "{{ just_cwd() }}/target/release/ckrv-mcp" } } }'

# =============================================================================
# DEVELOPMENT
# =============================================================================

# Run all linters
lint:
    cargo clippy --workspace -- -D warnings
    cd crates/ckrv-ui/frontend && npm run lint

# Run all tests
test:
    cargo test --workspace

# Format code
fmt:
    cargo fmt --all
    cd crates/ckrv-ui/frontend && npx prettier --write .

# Watch mode: rebuild on changes
watch:
    cargo watch -x "build --release -p ckrv-cli"

# =============================================================================
# DOCKER OPERATIONS
# =============================================================================

# Build all Docker images
docker-build:
    docker build -t ckrv-claude:latest -f docker/Dockerfile.claude docker/
    docker build -t ckrv-codex:latest -f docker/Dockerfile.codex docker/
    docker build -t ckrv-kilo:latest -f docker/Dockerfile.kilo docker/

# Stop all ckrv containers
docker-stop:
    -docker ps -q --filter "name=ckrv-" | xargs -r docker stop
    -docker ps -aq --filter "name=ckrv-" | xargs -r docker rm

# =============================================================================
# DOCS
# =============================================================================

# Generate Rust documentation
docs:
    cargo doc --no-deps --open

# List all recipes (built-in)
default:
    @just --list
```

### Makefile Compatibility Layer

```makefile
# Makefile (compatibility shim - kept for CI and users without just)
.PHONY: build install clean skill mcp install-mcp help ui-setup

%:
	@just "$@"
```

This single Makefile target forwards any unknown recipe to just, so `make build` → `just build`.

**Alternative**: Keep full Makefile with deprecation warnings:

```makefile
# Makefile (deprecated - use just instead)
.PHONY: build install clean skill mcp install-mcp help ui-setup

build:
	@echo "⚠ 'make build' is deprecated. Use 'just build' instead."
	@just build
```

## User Stories

### US1: Skip Docker in Containerized Dev
**As a** developer working in a Docker-based dev environment,
**I want** to skip Docker builds during install,
**So that** I can iterate faster without nested Docker.

### US2: Clear Recipe Discovery
**As a** new contributor,
**I want** to see all available commands with descriptions,
**So that** I can quickly understand the project workflow.

### US3: Parametric Recipes
**As a** developer,
**I want** to pass parameters to recipes (e.g., `just test filter=integration`),
**So that** I can run targeted operations without editing files.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Justfile + thin Makefile shim** | Full Just features, backwards compatible, simple transition | Two files to maintain (shim is trivial) |
| **B: Justfile replaces Makefile entirely** | Single source of truth | Breaks users without just installed |
| **C: Keep Makefile, add npm scripts** | No new dependency | npm scripts awkward for Rust project |
| **D: Keep Makefile as-is** | No migration effort | Retains all current pain points |

### Decision

**Option A**: Justfile as primary with Makefile shim. Rationale:
- CI systems already have just (or easy to install)
- Just installation is single binary, trivial on all platforms
- Shim provides 100% backwards compatibility during transition
- Can remove Makefile entirely after migration period

### Docker-Skip Implementation

Three mechanisms for skipping Docker:

1. **Environment variable**: `CKRV_SKIP_DOCKER=true just install`
2. **Recipe parameter**: `just install skip-docker=true`
3. **Dedicated recipe**: `just install-quick`

All three call the same logic via Just's variable system.

### Container/Dev Environment Detection

Could auto-detect Docker availability:

```just
has-docker := if shell("command -v docker", "") != "" { "true" } else { "false" }

install: build
    if {{ has-docker == "true" }} && {{ skip-docker != "true" }}; then
        # Docker builds
    fi
```

But explicit control is preferred over magic detection.

## Implementation Notes

### Just Features Used

| Feature | Example | Benefit |
|---------|---------|---------|
| Variables | `binary_name := "ckrv"` | DRY, easy config |
| Path joining | `npm_dir / "bin"` | Cross-platform paths |
| Shebang recipes | `#!/usr/bin/env bash` | Complex logic without escaping |
| Conditionals | `if {{ var == "x" }}` | Clear control flow |
| `just --list` | Built-in help | No manual maintenance |
| Recipe parameters | `test filter="":` | Flexible invocation |
| Dependencies | `build: ui-setup` | Same as Make |

### Installation Requirements

```bash
# macOS
brew install just

# Linux
cargo install just
# or
apt install just  # Ubuntu 24.04+

# Windows
cargo install just
# or
scoop install just
```

### CI Updates Required

Update `.github/workflows/*.yml`:

```yaml
- name: Install just
  uses: taiki-e/install-action@v2
  with:
    tool: just

- run: just build
- run: just test
- run: just lint
```

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| CI breaks during migration | Medium | High | Keep Makefile shim, update CI in separate PR |
| Contributors don't have just | Medium | Low | Shim forwards to just, add to README |
| Just version incompatibility | Low | Low | Pin version in CI, document minimum version |
| Recipe duplication during transition | Medium | Low | Makefile only forwards, no duplicate logic |
| IDE/editor integration | Low | Low | Most editors support just via plugin |

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Recipe count | 10 | 15+ (more granular) |
| Docker-skip option | No | Yes (3 methods) |
| Parallel execution | No | Yes (`just -j 4`) |
| Parameterized recipes | 0 | 3+ |
| Help text maintenance | Manual | Automatic |

## Rollout Plan

### Phase 1: Foundation (PR 1)
- [ ] Create `justfile` with all existing recipes
- [ ] Replace Makefile with thin shim
- [ ] Update `AGENTS.md` with just commands
- [ ] Add just to CI setup
- [ ] Document in README

### Phase 2: Enhancements (PR 2)
- [ ] Add `skip-docker` parameter to `install`
- [ ] Add `install-quick` recipe
- [ ] Add `docker-build`, `docker-stop` recipes
- [ ] Add `build-dev`, `watch` recipes
- [ ] Add `lint` recipe combining clippy + frontend lint

### Phase 3: Cleanup (PR 3, after 1-2 weeks)
- [ ] Remove Makefile shim (if no issues)
- [ ] Add `just` to `make install` output warning
- [ ] Update all documentation

### Phase 4: Advanced (Future)
- [ ] Add parameterized recipes for test filtering
- [ ] Add `just ci` recipe that mirrors CI pipeline
- [ ] Add `just release` recipe for version bumps

## Post-Implementation Review

Initial implementation completed quickly, but follow-up review found verification and rollout-safety gaps.

> ℹ️ Bugfix tasks generated: [bugfix01.md](./bugfix01.md)

Primary findings:
- Some acceptance checkboxes were marked complete before full command verification.
- Makefile shim should provide better guidance when `just` is missing.
- Docker-skip behavior and docs need explicit validation for container-first workflows.
- Documentation consistency needs a final grep-based pass.

## References

- [Just documentation](https://github.com/casey/just)
- [Just vs Make comparison](https://github.com/casey/just#why-not-make)
- [Just in CI best practices](https://just.systems/man/en/chapter_5.html)
- Example Justfiles: [ripgrep](https://github.com/BurntSushi/ripgrep/blob/master/justfile), [bat](https://github.com/sharkdp/bat/blob/master/justfile)
