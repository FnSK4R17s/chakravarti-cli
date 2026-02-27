# Chakravarti Documentation System

This document explains how documentation is organized in the Chakravarti CLI project.

## Documentation Hierarchy

```
chakravarti-cli/
├── README.md              # Project overview and quick start
├── CLAUDE.md              # AI agent + development guidelines
├── DESIGN.md              # High-level design decisions
├── CONTRIBUTING.md        # Contributor guidelines
│
├── crates/docs/           # Cross-crate documentation
│   ├── architecture.md    # System architecture and diagrams
│   ├── getting-started.md # New contributor onboarding
│   ├── cli-commands.md    # Complete CLI reference
│   ├── agent-guide.md     # Adding new AI agents
│   └── agent-integration-playbook.md  # Full-stack agent onboarding
│
├── crates/<crate>/docs/   # Per-crate documentation
│   └── README.md          # Crate-specific docs
│
└── docs/                  # Supplementary documentation
    ├── coverage.md        # Test coverage guidelines
    ├── optimization.md    # Performance optimization
    └── decisions/         # Architecture Decision Records
```

## Document Types

### Root-Level Documents

| Document | Audience | Purpose |
|----------|----------|---------|
| `README.md` | All users | Project overview, installation, quick start |
| `CLAUDE.md` | AI agents | Development guidelines + frontend patterns |
| `DESIGN.md` | Contributors | High-level design philosophy |
| `CONTRIBUTING.md` | Contributors | How to contribute code |

> [!IMPORTANT]
> **When updating documentation**, remember to also check:
> - `DESIGN.md` - Update implementation notes if architecture changes
> - `CONTRIBUTING.md` - Update project structure or request flow if crates change
> - `README.md` - Update agents table and crate descriptions
> - `crates/docs/agent-guide.md` - Update if agent architecture or auth methods change

### Cross-Crate Documentation (`crates/docs/`)

These documents span multiple crates and provide holistic guidance:

| Document | Content |
|----------|---------|
| `architecture.md` | Crate dependency graph, execution flow diagrams |
| `getting-started.md` | Environment setup, first contribution |
| `cli-commands.md` | All CLI commands with options and examples |
| `agent-guide.md` | How to add new AI agent integrations |
| `agent-integration-playbook.md` | Full-stack agent onboarding: types, both backends, frontend, Docker, tests |


### Per-Crate Documentation (`crates/<crate>/docs/`)

Each crate has a `docs/` subdirectory with crate-specific documentation:

```
crates/ckrv-cli/docs/README.md          # CLI crate specifics
crates/ckrv-core/docs/README.md         # Core orchestration
crates/ckrv-git/docs/README.md          # Git operations
crates/ckrv-sandbox/docs/README.md      # Docker execution + agent providers
crates/ckrv-spec/docs/README.md         # Spec parsing
crates/ckrv-model/docs/README.md        # LLM abstraction (⚠️ unused)
crates/ckrv-metrics/docs/README.md      # Cost/time tracking
crates/ckrv-verify/docs/README.md       # Test execution (⚠️ unused)
crates/ckrv-integrations/docs/README.md # External services (⚠️ stub)
crates/ckrv-ui/docs/README.md           # Web UI server
crates/ckrv-ui/docs/api-reference.md    # API endpoints
```

**Status Legend:**
- ⚠️ **unused** - Crate exists but is not used in current implementation
- ⚠️ **stub** - Crate has minimal implementation, planned for future

### Supplementary Documentation (`docs/`)

Extended documentation for specific topics:

- `coverage.md` - Test coverage targets and strategies
- `optimization.md` - Performance tuning guidelines
- `decisions/` - Architecture Decision Records (ADRs)

## AI Agent Files

Special files that guide AI coding assistants:

| File | Used By | Purpose |
|------|---------|---------|
| `CLAUDE.md` | All AI agents | Development guidelines, frontend patterns, troubleshooting |
| `.agent/workflows/` | Antigravity | Custom workflow definitions |

## Embedded Prompts

Agent prompts used by Chakravarti itself live in:

```
crates/ckrv-cli/src/prompts/
├── qa_reviewer.md    # QA review agent prompt
└── test_writer.md    # Test writer agent prompt
```

## Templates

Design templates for specification creation:

```
crates/ckrv-cli/src/templates/
└── design-template.md  # Default spec design template
```

## Writing Guidelines

### New Documentation

1. **Crate-specific** → Add to `crates/<crate>/docs/`
2. **Cross-crate** → Add to `crates/docs/`
3. **Operational/process** → Add to `docs/`
4. **Architecture decisions** → Add to `docs/decisions/`

### Format Standards

- Use **Markdown** for all documentation
- Include **Mermaid diagrams** for architecture
- Use **tables** for quick reference
- Keep **examples** in fenced code blocks
- Add **Last updated** timestamps to evolving docs

### Linking

Link to other docs using relative paths:
```markdown
See [Architecture](crates/docs/architecture.md) for details.
```

## Versioning

### Git Commit Tracking

> [!IMPORTANT]
> All generated documentation files MUST include the git commit hash at the time of creation.

**Required header format:**

```markdown
<!-- Generated at commit: abc1234 -->
<!-- Last updated: 2026-01-28 -->
```

Or in YAML frontmatter:

```yaml
---
generated_at_commit: abc1234
last_updated: 2026-01-28
---
```

### Checking for Updates

To see what code changed since a doc was generated:

```bash
# Get the commit from the doc's header
COMMIT=$(grep -oP 'Generated at commit: \K[a-f0-9]+' crates/docs/architecture.md)

# Diff against that commit
git diff $COMMIT -- crates/ckrv-core/

# Or see a summary of changes
git log --oneline $COMMIT..HEAD -- crates/ckrv-core/
```

### When to Update Docs

Update documentation when the diff shows:
- **API changes** - New/modified public functions
- **Architecture changes** - New crates, dependencies, or flows
- **Feature additions** - New commands, options, or behaviors
- **Breaking changes** - Removed or renamed functionality

### Update Workflow

1. **Check diff** against the doc's commit
2. **Identify** what code changed
3. **Update** the relevant sections
4. **Bump** the commit hash and date in the header
5. **Commit** with message: `docs: update <file> to <commit>`

## LLM Documentation Update Workflow

> [!IMPORTANT]
> Use these commands to determine which documentation files need updates.

### Step 1: List All Documented Crates

```bash
# Find all crate documentation files
find crates -path "*/docs/README.md" -type f | grep -v node_modules
```

### Step 2: Extract Commit Hashes from Docs

```bash
# Extract last_commit from YAML frontmatter
grep -oP '^last_commit: \K[a-f0-9]+' crates/ckrv-core/docs/README.md
```

### Step 3: Check for Code Changes Since Doc Commit

```bash
# Get the doc's commit
DOC_COMMIT=$(grep -oP '^last_commit: \K[a-f0-9]+' crates/ckrv-core/docs/README.md)

# Show changed files in that crate since doc was generated
git diff --name-only $DOC_COMMIT HEAD -- crates/ckrv-core/

# Show detailed changes (for semantic analysis)
git diff $DOC_COMMIT HEAD -- crates/ckrv-core/src/

# Count lines changed
git diff --stat $DOC_COMMIT HEAD -- crates/ckrv-core/
```

### Step 4: Identify Public API Changes

```bash
# Check for changes in lib.rs (public exports)
git diff $DOC_COMMIT HEAD -- crates/ckrv-core/src/lib.rs

# Check for signature changes in main modules
git diff $DOC_COMMIT HEAD -- 'crates/ckrv-core/src/*.rs' | grep -E '^[\+\-].*(pub fn|pub struct|pub enum|pub trait)'
```

### Step 5: Check Cross-Crate Dependencies

```bash
# For architecture.md - check ALL crates
DOC_COMMIT=$(grep -oP '^last_commit: \K[a-f0-9]+' crates/docs/architecture.md)

# See which crates have changes
for crate in ckrv-cli ckrv-core ckrv-git ckrv-sandbox ckrv-spec ckrv-model ckrv-metrics ckrv-verify ckrv-integrations ckrv-ui; do
  changes=$(git diff --stat $DOC_COMMIT HEAD -- crates/$crate/src/ | tail -1)
  echo "$crate: $changes"
done
```

### Step 6: Prioritize Updates

| Change Type | Example | Action |
|------------|---------|--------|
| API signature change | `pub fn foo()` → `pub fn foo(bar: bool)` | **Immediate update** |
| New public type | Added `pub struct NewType` | Update within session |
| Internal refactor | Moved private code | Low priority |
| Formatting/comments | Whitespace, doc comments | Skip |

### Step 7: Update Documentation

```bash
# Get current commit hash
NEW_COMMIT=$(git rev-parse --short HEAD)

# Update the frontmatter in the doc
sed -i "s/^last_commit: .*/last_commit: $NEW_COMMIT/" crates/ckrv-core/docs/README.md
sed -i "s/^last_updated: .*/last_updated: $(date +%Y-%m-%d)/" crates/ckrv-core/docs/README.md
```

### Freshness Check Script

```bash
#!/bin/bash
# check-docs-freshness.sh

echo "=== Documentation Freshness Report ==="
echo ""

for doc in $(find crates -path "*/docs/README.md" -type f | grep -v node_modules); do
  crate=$(echo $doc | cut -d'/' -f2)
  commit=$(grep -oP '^last_commit: \K[a-f0-9]+' "$doc" 2>/dev/null || echo "none")
  
  if [ "$commit" = "none" ]; then
    echo "⚠️  $crate: No commit tracking"
    continue
  fi
  
  changes=$(git diff --stat "$commit" HEAD -- "crates/$crate/src/" 2>/dev/null | tail -1)
  
  if [ -z "$changes" ]; then
    echo "✅ $crate: Up to date (at $commit)"
  else
    echo "🔄 $crate: Needs update - $changes"
    echo "   Run: git diff $commit HEAD -- crates/$crate/src/"
  fi
done
```

## Generating Documentation

### Rust API Docs

```bash
cargo doc --open --no-deps
```

### View Docs in Browser

```bash
ckrv ui  # Web UI includes documentation viewer
```
