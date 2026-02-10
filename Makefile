.PHONY: build install clean skill mcp install-mcp help

# ============================================================================
# IMPORTANT: When adding new targets, update the help command below!
# ============================================================================

# Binary name
BINARY_NAME = ckrv

# Directories
NPM_DIR = npm
BIN_DIR = $(NPM_DIR)/bin
RUST_BIN = target/release/$(BINARY_NAME)

# Default target: Show help
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo ""
	@echo "Chakravarti CLI - Makefile Commands"
	@echo "===================================="
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build & Install:"
	@echo "  build        Build the Rust binary in release mode"
	@echo "  install      Full install: build, Docker images, npm link"
	@echo "  clean        Remove build artifacts"
	@echo ""
	@echo "AI Interface:"
	@echo "  skill        Generate SKILL.md for AI agents"
	@echo "  mcp          Build the MCP server binary"
	@echo "  install-mcp  Build MCP server and show Claude Desktop config"
	@echo ""
	@echo "Development:"
	@echo "  ui-setup     Install UI frontend dependencies"
	@echo "  help         Show this help message"
	@echo ""

# Build the Rust binary
build: ui-setup
	@echo "Building Rust binary in release mode..."
	cargo build --release -p ckrv-cli

# UI Setup: Install frontend dependencies
ui-setup:
	@echo "Installing UI frontend dependencies..."
	cd crates/ckrv-ui/frontend && npm install

# Install target: Build, cleanup old containers, build Docker images, copy to npm/bin, and link via npm
install: build
	@echo "Stopping any running ckrv containers..."
	-docker ps -q --filter "name=ckrv-" | xargs -r docker stop 2>/dev/null || true
	-docker ps -aq --filter "name=ckrv-" | xargs -r docker rm 2>/dev/null || true
	@echo "Building Docker agent images..."
	docker build -t ckrv-claude:latest -f docker/Dockerfile.claude docker/
	docker build -t ckrv-codex:latest -f docker/Dockerfile.codex docker/
	docker build -t ckrv-kilo:latest -f docker/Dockerfile.kilo docker/
	@echo "Copying binary to npm/bin..."
	mkdir -p $(BIN_DIR)
	cp $(RUST_BIN) $(BIN_DIR)/$(BINARY_NAME)
	chmod +x $(BIN_DIR)/$(BINARY_NAME)
	@echo "Linking via npm..."
	cd $(NPM_DIR) && npm link
	@echo "Installing to Cargo bin..."
	cp $(RUST_BIN) $(HOME)/.cargo/bin/$(BINARY_NAME)
	@echo "\n✓ Chakravarti CLI installed and linked successfully!"
	@echo "Run 'ckrv --version' to verify."

# Clean target: Remove build artifacts
clean:
	cargo clean
	rm -rf $(BIN_DIR)/$(BINARY_NAME)

# SKILL.md generation: Generate and validate SKILL.md for AI agents
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

# MCP Server: Build the MCP server binary
mcp:
	@echo "Building MCP server..."
	cargo build --release -p ckrv-mcp
	@echo "✓ MCP server built: target/release/ckrv-mcp"

# Install MCP: Build MCP server and print Claude Desktop config
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
	@echo '      "command": "$(CURDIR)/target/release/ckrv-mcp"'
	@echo '    }'
	@echo '  }'
	@echo '}'
	@echo ""
