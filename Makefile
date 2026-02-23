# ==============================================================================
# Makefile - Compatibility Shim (DEPRECATED)
# ==============================================================================
# ⚠️  DEPRECATED: This Makefile is deprecated. Use 'just' instead.
#    Install just: https://github.com/casey/just#installation
# ==============================================================================

.PHONY: build install clean skill mcp install-mcp help ui-setup __forward

JUST_CHECK := $(shell command -v just 2>/dev/null)

# Explicit targets so commands like `make install` never become a no-op.
build install clean skill mcp install-mcp ui-setup:
	@$(MAKE) --no-print-directory __forward TARGET=$@

# Fallback for any other command, e.g. `make test` -> `just test`.
%:
	@$(MAKE) --no-print-directory __forward TARGET=$@

__forward:
	@if [ -z "$(JUST_CHECK)" ]; then \
		echo "Error: 'just' is not installed." >&2; \
		echo "" >&2; \
		echo "Install just:" >&2; \
		echo "  macOS:   brew install just" >&2; \
		echo "  Linux:   cargo install just" >&2; \
		echo "  Windows: cargo install just" >&2; \
		echo "" >&2; \
		echo "See: https://github.com/casey/just#installation" >&2; \
		exit 1; \
	fi
	@echo "⚠️  'make $(TARGET)' is deprecated. Use 'just $(TARGET)' instead." >&2
	@just "$(TARGET)"

.DEFAULT_GOAL := help
help:
	@if [ -z "$(JUST_CHECK)" ]; then \
		echo "Error: 'just' is not installed." >&2; \
		echo "" >&2; \
		echo "Install just:" >&2; \
		echo "  macOS:   brew install just" >&2; \
		echo "  Linux:   cargo install just" >&2; \
		echo "  Windows: cargo install just" >&2; \
		echo "" >&2; \
		echo "See: https://github.com/casey/just#installation" >&2; \
		exit 1; \
	fi
	@just --list
