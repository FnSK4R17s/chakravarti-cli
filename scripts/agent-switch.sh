#!/bin/bash

# Agent Switcher Script
# Toggles between .agent/ and .claude/ folder structures
#
# .agent/           <-> .claude/
# .agent/workflows/ <-> .claude/commands/

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find the git root directory
GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")

AGENT_DIR="$GIT_ROOT/.agent"
CLAUDE_DIR="$GIT_ROOT/.claude"

print_status() {
    if [[ -d "$CLAUDE_DIR" ]]; then
        echo -e "${GREEN}Current mode: Claude (.claude/)${NC}"
    elif [[ -d "$AGENT_DIR" ]]; then
        echo -e "${YELLOW}Current mode: Agent (.agent/)${NC}"
    else
        echo -e "${RED}No agent configuration found${NC}"
    fi
}

to_claude() {
    if [[ -d "$CLAUDE_DIR" ]]; then
        echo -e "${YELLOW}Already in Claude mode${NC}"
        return 0
    fi

    if [[ ! -d "$AGENT_DIR" ]]; then
        echo -e "${RED}Error: .agent/ directory not found${NC}"
        return 1
    fi

    echo "Switching from .agent/ to .claude/..."

    # Rename main directory
    mv "$AGENT_DIR" "$CLAUDE_DIR"

    # Rename workflows to commands if it exists
    if [[ -d "$CLAUDE_DIR/workflows" ]]; then
        mv "$CLAUDE_DIR/workflows" "$CLAUDE_DIR/commands"
        echo -e "${GREEN}Renamed workflows/ to commands/${NC}"
    fi

    echo -e "${GREEN}Switched to Claude mode (.claude/)${NC}"
}

to_agent() {
    if [[ -d "$AGENT_DIR" ]]; then
        echo -e "${YELLOW}Already in Agent mode${NC}"
        return 0
    fi

    if [[ ! -d "$CLAUDE_DIR" ]]; then
        echo -e "${RED}Error: .claude/ directory not found${NC}"
        return 1
    fi

    echo "Switching from .claude/ to .agent/..."

    # Rename commands to workflows if it exists
    if [[ -d "$CLAUDE_DIR/commands" ]]; then
        mv "$CLAUDE_DIR/commands" "$CLAUDE_DIR/workflows"
        echo -e "${GREEN}Renamed commands/ to workflows/${NC}"
    fi

    # Rename main directory
    mv "$CLAUDE_DIR" "$AGENT_DIR"

    echo -e "${GREEN}Switched to Agent mode (.agent/)${NC}"
}

toggle() {
    if [[ -d "$CLAUDE_DIR" ]]; then
        to_agent
    elif [[ -d "$AGENT_DIR" ]]; then
        to_claude
    else
        echo -e "${RED}No agent configuration found. Create .agent/ or .claude/ first.${NC}"
        return 1
    fi
}

usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status    Show current mode"
    echo "  claude    Switch to Claude mode (.claude/)"
    echo "  agent     Switch to Agent mode (.agent/)"
    echo "  toggle    Toggle between modes (default)"
    echo "  help      Show this help message"
    echo ""
    echo "Directory mappings:"
    echo "  .agent/           <-> .claude/"
    echo "  .agent/workflows/ <-> .claude/commands/"
}

# Main
case "${1:-toggle}" in
    status)
        print_status
        ;;
    claude)
        to_claude
        ;;
    agent)
        to_agent
        ;;
    toggle)
        toggle
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        usage
        exit 1
        ;;
esac
