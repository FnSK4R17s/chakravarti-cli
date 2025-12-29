.PHONY: build install clean

# Binary name
BINARY_NAME = ckrv

# Directories
NPM_DIR = npm
BIN_DIR = $(NPM_DIR)/bin
RUST_BIN = target/release/$(BINARY_NAME)

# Default target: Build the Rust binary
build: ui-setup
	@echo "Building Rust binary in release mode..."
	cargo build --release -p ckrv-cli

# UI Setup: Install frontend dependencies
ui-setup:
	@echo "Installing UI frontend dependencies..."
	cd crates/ckrv-ui/frontend && npm install

# Install target: Build, copy to npm/bin, and link via npm
install: build
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
