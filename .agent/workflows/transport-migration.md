---
description: Compare ckrv-ui to ckrv-transport migration completeness
---

# Transport Migration Comparison Workflow

Compare what has been migrated from `ckrv-ui/src/api/` to `ckrv-transport/` and identify gaps.

## 1. Check Old API Snapshot Reference Commit

The old ckrv-ui API code was removed but preserved at commit `c8d96c9`. Use this as reference.

// turbo
```bash
echo "Reference commit: c8d96c9"
git log -1 --oneline c8d96c9 2>/dev/null || echo "Commit not found"
```

## 2. List Old API Modules from Git History

// turbo
```bash
echo "=== Old ckrv-ui API Files (from c8d96c9) ==="
git ls-tree --name-only c8d96c9:crates/ckrv-ui/src/api/ 2>/dev/null || echo "Not found"
```

## 3. List New Transport Handlers and Routes

// turbo
```bash
echo "=== Transport Handlers ==="
ls -1 crates/ckrv-transport/src/handlers/*.rs | xargs -n1 basename | sed 's/.rs$//'

echo ""
echo "=== Transport Axum Routes ==="
ls -1 crates/ckrv-transport/src/axum/*.rs | xargs -n1 basename | sed 's/.rs$//'
```

## 4. Compare Module by Module

For each old API module, show old vs new line counts:

// turbo
```bash
echo "=== Module Comparison (Old vs New) ==="
echo "Module           | Old Lines | New Handler | New Route"
echo "-----------------|-----------+-------------+----------"

for module in agents session terminal execution specs; do
  old_lines=$(git show c8d96c9:crates/ckrv-ui/src/api/${module}.rs 2>/dev/null | wc -l || echo "0")
  handler_lines=$(wc -l < crates/ckrv-transport/src/handlers/${module}.rs 2>/dev/null || echo "0")
  route_lines=$(wc -l < crates/ckrv-transport/src/axum/${module}.rs 2>/dev/null || echo "0")
  printf "%-16s | %9s | %11s | %9s\n" "$module" "$old_lines" "$handler_lines" "$route_lines"
done
```

## 5. Show Old Code Structure for Key Module

Pick a module to deep-dive (e.g., terminal, agents):

// turbo
```bash
echo "=== Old Terminal Handler Structure ==="
git show c8d96c9:crates/ckrv-ui/src/api/terminal.rs 2>/dev/null | grep -E "^(pub |async |fn |struct |impl |// )" | head -40
```

## 6. Show New Code Structure for Same Module

// turbo
```bash
echo "=== New Terminal Handler Structure ==="
grep -E "^(pub |async |fn |struct |impl |// )" crates/ckrv-transport/src/handlers/terminal.rs | head -40
```

## 7. List Frontend API Calls

// turbo
```bash
echo "=== Frontend API Endpoints ==="
grep -roh "/api/[a-zA-Z0-9/_-]*" crates/ckrv-ui/frontend/src/ 2>/dev/null | sort -u
```

## 8. List All Transport Routes

// turbo
```bash
echo "=== Transport Routes ==="
grep -rh '\.route(' crates/ckrv-transport/src/axum/*.rs | sed 's/.*\.route("//' | sed 's/".*//' | sort -u
```

## 9. Identify Missing Features by Diffing Old vs New

Compare specific implementations:

// turbo
```bash
echo "=== Credential Mounting Check ==="
echo "Old code had:"
git show c8d96c9:crates/ckrv-ui/src/api/terminal.rs 2>/dev/null | grep -c "binds.push" || echo "0"
echo "binds.push calls"

echo ""
echo "New code has:"
grep -c "binds.push" crates/ckrv-transport/src/handlers/terminal.rs || echo "0"
echo "binds.push calls"
```

## 10. Check Agent Type Handling

// turbo
```bash
echo "=== Agent Type Handling ==="
echo "Old code patterns:"
git show c8d96c9:crates/ckrv-ui/src/api/terminal.rs 2>/dev/null | grep -E "is_(openrouter|glm|codex)" | wc -l
echo "agent type checks"

echo ""
echo "New code patterns:"
grep -E "is_(openrouter|glm|codex)" crates/ckrv-transport/src/handlers/terminal.rs | wc -l
echo "agent type checks"
```

## 11. Check Environment Variable Setup

// turbo
```bash
echo "=== Environment Variables in Old Code ==="
git show c8d96c9:crates/ckrv-ui/src/api/terminal.rs 2>/dev/null | grep -oE "env_vars\.push\([^)]+\)" | head -20

echo ""
echo "=== Environment Variables in New Code ==="
grep -oE "env_vars\.push\([^)]+\)" crates/ckrv-transport/src/handlers/terminal.rs | head -20
```

## 12. Test Key Endpoints

// turbo
```bash
echo "Testing key endpoints..."
curl -s http://localhost:3000/api/status | head -c 100 && echo ""
curl -s http://localhost:3000/api/agents | head -c 100 && echo ""
curl -s http://localhost:3000/api/docker | head -c 100 && echo ""
```

## 13. Generate Gap Report

After running the above, create a summary noting:

1. **Fully Migrated**: Modules with equivalent functionality
2. **Partially Migrated**: Modules missing specific features
3. **Not Started**: Modules not yet ported
4. **Extra in Transport**: New routes not in old code

Update `specs/019-transport-crate/migration-status.md` with findings.
