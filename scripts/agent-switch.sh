#!/bin/bash

# Agent Switcher Script
# Cycles between .agent/, .claude/, and .opencode/ folder structures
#
# .agent/           -> .claude/   -> .opencode/ -> .agent/
# .agent/workflows/ -> commands/ -> commands/    -> workflows/

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Find the git root directory; fall back to current dir if not in a git repo
GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo ".")

AGENT_DIR="$GIT_ROOT/.agent"
CLAUDE_DIR="$GIT_ROOT/.claude"
OPENCODE_DIR="$GIT_ROOT/.opencode"

print_status() {
    if [[ -d "$OPENCODE_DIR" ]]; then
        echo -e "${BLUE}Current mode: OpenCode (.opencode/)${NC}"
    elif [[ -d "$CLAUDE_DIR" ]]; then
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

    local source_dir=""
    if [[ -d "$AGENT_DIR" ]]; then
        source_dir="$AGENT_DIR"
        echo "Switching from .agent/ to .claude/..."
    elif [[ -d "$OPENCODE_DIR" ]]; then
        source_dir="$OPENCODE_DIR"
        echo "Switching from .opencode/ to .claude/..."
    else
        echo -e "${RED}Error: .agent/ or .opencode/ directory not found${NC}"
        return 1
    fi

    mv "$source_dir" "$CLAUDE_DIR"

    # Only .agent contains workflows/, .opencode already uses commands/
    if [[ -d "$CLAUDE_DIR/workflows" ]]; then
        mv "$CLAUDE_DIR/workflows" "$CLAUDE_DIR/commands"
        echo -e "${GREEN}Renamed workflows/ to commands/${NC}"
    fi

    echo -e "${GREEN}Switched to Claude mode (.claude/)${NC}"
}

to_opencode() {
    if [[ -d "$OPENCODE_DIR" ]]; then
        echo -e "${YELLOW}Already in OpenCode mode${NC}"
        return 0
    fi

    local source_dir=""
    if [[ -d "$CLAUDE_DIR" ]]; then
        source_dir="$CLAUDE_DIR"
        echo "Switching from .claude/ to .opencode/..."
    elif [[ -d "$AGENT_DIR" ]]; then
        source_dir="$AGENT_DIR"
        echo "Switching from .agent/ to .opencode/..."
    else
        echo -e "${RED}Error: .claude/ or .agent/ directory not found${NC}"
        return 1
    fi

    # Only .agent contains workflows/, .claude already uses commands/
    if [[ -d "$source_dir/workflows" ]]; then
        mv "$source_dir/workflows" "$source_dir/commands"
        echo -e "${GREEN}Renamed workflows/ to commands/${NC}"
    fi

    mv "$source_dir" "$OPENCODE_DIR"

    echo -e "${BLUE}Switched to OpenCode mode (.opencode/)${NC}"
}

to_agent() {
    if [[ -d "$AGENT_DIR" ]]; then
        echo -e "${YELLOW}Already in Agent mode${NC}"
        return 0
    fi

    local source_dir=""
    if [[ -d "$CLAUDE_DIR" ]]; then
        source_dir="$CLAUDE_DIR"
        echo "Switching from .claude/ to .agent/..."
    elif [[ -d "$OPENCODE_DIR" ]]; then
        source_dir="$OPENCODE_DIR"
        echo "Switching from .opencode/ to .agent/..."
    else
        echo -e "${RED}Error: .claude/ or .opencode/ directory not found${NC}"
        return 1
    fi

    # Rename commands to workflows if it exists
    if [[ -d "$source_dir/commands" ]]; then
        mv "$source_dir/commands" "$source_dir/workflows"
        echo -e "${GREEN}Renamed commands/ to workflows/${NC}"
    fi

    mv "$source_dir" "$AGENT_DIR"

    echo -e "${GREEN}Switched to Agent mode (.agent/)${NC}"
}

toggle() {
    if [[ -d "$AGENT_DIR" ]]; then
        to_claude
    elif [[ -d "$CLAUDE_DIR" ]]; then
        to_opencode
    elif [[ -d "$OPENCODE_DIR" ]]; then
        to_agent
    else
        echo -e "${RED}No agent configuration found. Create .agent/, .claude/, or .opencode/ first.${NC}"
        return 1
    fi
}

usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status    Show current mode"
    echo "  agent     Switch to Agent mode (.agent/)"
    echo "  claude    Switch to Claude mode (.claude/)"
    echo "  opencode  Switch to OpenCode mode (.opencode/)"
    echo "  toggle    Cycle through modes (default)"
    echo "  help      Show this help message"
    echo ""
    echo "Directory mappings:"
    echo "  .agent/           <-> .claude/   <-> .opencode/"
    echo "  .agent/workflows/ <-> commands/ <-> commands/"
    echo ""
    echo "Cycle order: .agent/ -> .claude/ -> .opencode/ -> .agent/"
}

# Main
case "${1:-toggle}" in
    status)
        print_status
        ;;
    agent)
        to_agent
        ;;
    claude)
        to_claude
        ;;
    opencode)
        to_opencode
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
