# ==============================================================================
# Makefile - Compatibility Shim (DEPRECATED)
# ==============================================================================
# ⚠️  DEPRECATED: This Makefile is deprecated. Use 'just' instead.
#    Install just: https://github.com/casey/just#installation
# ==============================================================================

.PHONY: build install clean skill mcp install-mcp help ui-setup

JUST_CHECK := $(shell command -v just 2>/dev/null)

%:
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
	@echo "⚠️  'make $@' is deprecated. Use 'just $@' instead." >&2
	@just "$@"

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
