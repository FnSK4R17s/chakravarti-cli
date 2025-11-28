.PHONY: install build link dev clean unlink help

# Default target
help:
	@echo "Chakravarti CLI - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make install    - Install dependencies"
	@echo "  make build      - Build the TypeScript project"
	@echo "  make link       - Link the CLI globally (requires build)"
	@echo "  make dev        - Install deps, build, and link (full setup)"
	@echo "  make unlink     - Unlink the CLI from global"
	@echo "  make clean      - Remove build artifacts and node_modules"
	@echo "  make help       - Show this help message"

# Install dependencies
install:
	npm install

# Build the project
build:
	npm run build

# Link the CLI globally
link: build
	npm link

# Full development setup
dev: install build link
	@echo ""
	@echo "✓ Development setup complete!"
	@echo "You can now use 'ckrv' or 'chakravarti' from anywhere."

# Unlink the CLI
unlink:
	npm unlink -g chakravarti-cli

# Clean build artifacts
clean:
	rm -rf dist node_modules
	@echo "✓ Cleaned build artifacts and dependencies"
