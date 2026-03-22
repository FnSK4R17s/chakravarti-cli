#!/usr/bin/env bash
set -euo pipefail

# Usage: is-beary-summon.sh "<raw user message>"
# Exit 0 => message is exactly "beary" (case-insensitive, surrounding whitespace ignored)
# Exit 1 => not a pure BEARY summon

msg="${1:-}"
normalized="$(printf '%s' "$msg" | tr '[:upper:]' '[:lower:]' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"

if [ "$normalized" = "beary" ]; then
  exit 0
fi

exit 1
