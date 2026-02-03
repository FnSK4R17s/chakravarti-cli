#!/bin/bash
#
# Frontend Documentation Verification Script
# Based on: .agent/workflows/docs.frontend.md
#
# Usage: ./verify-frontend-docs.sh [--verbose] [--component <name>]
#

# Don't use set -e since we want to continue through all checks

# Configuration
FRONTEND_DIR="/apps/chakravarti-cli/crates/ckrv-ui/frontend/src"
COMPONENTS_DIR="$FRONTEND_DIR/components"
HOOKS_DIR="$FRONTEND_DIR/hooks"
LAYOUTS_DIR="$FRONTEND_DIR/layouts"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0
WARN=0

# Parse arguments
VERBOSE=false
COMPONENT=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --component|-c)
            COMPONENT="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

echo "═══════════════════════════════════════════════════════════════"
echo "  Frontend Documentation Verification"
echo "  Generated: $(date +'%Y-%m-%d %H:%M:%S')"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ==============================================================================
# Phase 1: Health Analysis
# ==============================================================================

echo -e "${BLUE}Phase 1: Health Analysis${NC}"
echo "───────────────────────────────────────────────────────────────"

# Step 1.1: Component Size Analysis
echo ""
echo "1.1 Component Size Analysis:"
echo ""
cd "$COMPONENTS_DIR" 2>/dev/null || { echo "Cannot access components directory"; exit 1; }

for file in *.tsx; do
    if [[ -n "$COMPONENT" && "$file" != "$COMPONENT.tsx" ]]; then
        continue
    fi
    lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
    if [[ "$lines" -gt 500 ]]; then
        echo -e "  ${RED}🔴 $file: $lines lines (>500 - needs refactor)${NC}"
        ((WARN++))
    elif [[ "$lines" -gt 200 ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${YELLOW}⚠️  $file: $lines lines (200-500 - consider splitting)${NC}"
        fi
    else
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${GREEN}✅ $file: $lines lines${NC}"
        fi
    fi
done

# ==============================================================================
# Phase 2: Documentation Drift Detection
# ==============================================================================

echo ""
echo -e "${BLUE}Phase 2: Documentation Drift Detection${NC}"
echo "───────────────────────────────────────────────────────────────"

# Step 2.5: Module Header Check
echo ""
echo "2.5 Module Header Check:"
echo ""

module_pass=0
module_fail=0

# Check components
for file in "$COMPONENTS_DIR"/*.tsx; do
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    name=$(basename "$file")
    if head -30 "$file" | grep -q "@module"; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${GREEN}✅ $name${NC}"
        fi
        ((module_pass++))
    else
        echo -e "  ${RED}❌ $name - Missing @module header${NC}"
        ((module_fail++))
    fi
done

# Check hooks
shopt -s nullglob
for file in "$HOOKS_DIR"/*.ts; do
    [[ -f "$file" ]] || continue
    name=$(basename "$file")
    if head -30 "$file" | grep -q "@module"; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${GREEN}✅ $name${NC}"
        fi
        ((module_pass++))
    else
        echo -e "  ${RED}❌ $name - Missing @module header${NC}"
        ((module_fail++))
    fi
done
shopt -u nullglob

# Check layouts
shopt -s nullglob
for file in "$LAYOUTS_DIR"/*.tsx; do
    [[ -f "$file" ]] || continue
    name=$(basename "$file")
    if head -30 "$file" | grep -q "@module"; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${GREEN}✅ $name${NC}"
        fi
        ((module_pass++))
    else
        echo -e "  ${RED}❌ $name - Missing @module header${NC}"
        ((module_fail++))
    fi
    done
shopt -u nullglob

echo ""
echo "  Module Headers: $module_pass pass, $module_fail fail"
if [[ $module_fail -eq 0 ]]; then
    ((PASS++))
else
    ((FAIL++))
fi

# Step 2.3: State Documentation Drift
echo ""
echo "2.3 State Documentation (useState comments):"
echo ""

state_documented=0
state_total=0

for file in "$COMPONENTS_DIR"/*.tsx; do
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    component=$(basename "$file" .tsx)
    
    # Count documented state (comments before useState)
    documented=$(grep -B 1 "useState" "$file" 2>/dev/null | grep -cE "/\*\*|//" 2>/dev/null || true)
    documented=$(echo "$documented" | tr -d '\n\r ' | grep -E '^[0-9]+$' || echo 0)
    [[ -z "$documented" ]] && documented=0
    
    # Count actual useState calls
    actual=$(grep -c "useState" "$file" 2>/dev/null || true)
    actual=$(echo "$actual" | tr -d '\n\r ' | grep -E '^[0-9]+$' || echo 0)
    [[ -z "$actual" ]] && actual=0
    
    if [[ "$actual" -gt 0 ]]; then
        state_total=$((state_total + actual))
        state_documented=$((state_documented + documented))
        coverage=$((documented * 100 / actual))
        if [[ "$coverage" -lt 50 && "$actual" -gt 3 ]]; then
            echo -e "  ${YELLOW}⚠️  $component: $documented/$actual useState documented ($coverage%)${NC}"
        elif [[ "$VERBOSE" == "true" && "$coverage" -ge 80 ]]; then
            echo -e "  ${GREEN}✅ $component: $documented/$actual useState documented${NC}"
        fi
    fi
done

if [[ $state_total -gt 0 ]]; then
    state_pct=$((state_documented * 100 / state_total))
    echo ""
    echo "  State Documentation: $state_documented/$state_total ($state_pct%)"
fi

# Step 2.4: Effect Documentation Drift
echo ""
echo "2.4 Effect Documentation (useEffect comments):"
echo ""

effect_documented=0
effect_total=0

for file in "$COMPONENTS_DIR"/*.tsx; do
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    component=$(basename "$file" .tsx)
    
    # Count actual useEffect calls (with parenthesis, not just mentions)
    total=$(grep -cE "useEffect\s*\(" "$file" 2>/dev/null || true)
    total=$(echo "$total" | tr -d '\n\r ' | grep -E '^[0-9]+$' || echo 0)
    [[ -z "$total" ]] && total=0
    
    if [[ "$total" -gt 0 ]]; then
        # Find useEffect without preceding comment - match actual calls with (
        undocumented=$(awk '
            /useEffect\s*\(/ {
                if (prev !~ /\/\*\*|\/\//) {
                    count++
                }
            }
            { prev = $0 }
            END { print count+0 }
        ' "$file" 2>/dev/null || true)
        undocumented=$(echo "$undocumented" | tr -d '\n\r ' | grep -E '^[0-9]+$' || echo 0)
        [[ -z "$undocumented" ]] && undocumented=0
        
        documented=$((total - undocumented))
        effect_total=$((effect_total + total))
        effect_documented=$((effect_documented + documented))
        
        if [[ "$undocumented" -gt 0 && "$total" -gt 2 ]]; then
            echo -e "  ${YELLOW}⚠️  $component: $undocumented/$total useEffect undocumented${NC}"
        elif [[ "$VERBOSE" == "true" && "$undocumented" -eq 0 ]]; then
            echo -e "  ${GREEN}✅ $component: All $total useEffect documented${NC}"
        fi
    fi
done

if [[ $effect_total -gt 0 ]]; then
    effect_pct=$((effect_documented * 100 / effect_total))
    echo ""
    echo "  Effect Documentation: $effect_documented/$effect_total ($effect_pct%)"
fi

# Step 2.6: Example Code Validation
echo ""
echo "2.6 Example Block Coverage (@example):"
echo ""

example_pass=0
example_fail=0

for file in "$COMPONENTS_DIR"/*.tsx; do
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    component=$(basename "$file" .tsx)
    
    examples=$(grep -c "@example" "$file" 2>/dev/null || echo 0)
    examples=${examples:-0}
    
    if [[ "$examples" -eq 0 ]]; then
        echo -e "  ${RED}❌ $component: No @example blocks${NC}"
        ((example_fail++))
    else
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${GREEN}✅ $component: $examples @example blocks${NC}"
        fi
        ((example_pass++))
    fi
done

echo ""
echo "  Example Blocks: $example_pass pass, $example_fail fail"

# ==============================================================================
# Phase 3: Convention Compliance Check
# ==============================================================================

echo ""
echo -e "${BLUE}Phase 3: Convention Compliance${NC}"
echo "───────────────────────────────────────────────────────────────"

# Step 3.3: Section Comment Check
echo ""
echo "3.3 Section Comments (files >400 lines need 4+ sections):"
echo ""

section_pass=0
section_fail=0

shopt -s nullglob
for file in "$COMPONENTS_DIR"/*.tsx "$LAYOUTS_DIR"/*.tsx; do
    [[ -f "$file" ]] || continue
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    component=$(basename "$file" .tsx)
    lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
    lines=${lines:-0}
    
    # Only check files > 400 lines (MUST have sections per FR-006)
    if [[ "$lines" -gt 400 ]]; then
        sections=$(grep -c "// ===" "$file" 2>/dev/null || echo 0)
        sections=${sections:-0}
        if [[ "$sections" -lt 4 ]]; then
            echo -e "  ${RED}❌ $component ($lines lines): $sections sections (need 4+)${NC}"
            ((section_fail++))
        else
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "  ${GREEN}✅ $component ($lines lines): $sections sections${NC}"
            fi
            ((section_pass++))
        fi
    fi
done
shopt -u nullglob

echo ""
echo "  Section Comments: $section_pass pass, $section_fail fail"
if [[ $section_fail -eq 0 ]]; then
    ((PASS++))
else
    ((FAIL++))
fi

# Step 3.1: Import Order Check (simplified)
echo ""
echo "3.1 Import Order Check:"
echo ""

import_issues=0

for file in "$COMPONENTS_DIR"/*.tsx; do
    if [[ -n "$COMPONENT" && "$(basename "$file" .tsx)" != "$COMPONENT" ]]; then
        continue
    fi
    component=$(basename "$file" .tsx)
    
    # Check if React is in first 5 import lines
    first_imports=$(head -10 "$file" | grep "^import" | head -5)
    if ! echo "$first_imports" | grep -q "from 'react'"; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "  ${YELLOW}⚠️  $component: React import should be near top${NC}"
        fi
        ((import_issues++))
    fi
done

if [[ $import_issues -eq 0 ]]; then
    echo -e "  ${GREEN}✅ Import order looks good${NC}"
fi

# ==============================================================================
# Phase 4: README Check
# ==============================================================================

echo ""
echo -e "${BLUE}Phase 4: README Check${NC}"
echo "───────────────────────────────────────────────────────────────"

README_FILE="/apps/chakravarti-cli/crates/ckrv-ui/frontend/README.md"
echo ""
echo "4.1 Frontend README:"
echo ""

if [[ -f "$README_FILE" ]]; then
    # Check if it's still Vite boilerplate
    if head -5 "$README_FILE" | grep -qi "vite\|template"; then
        echo -e "  ${RED}❌ README contains Vite boilerplate - needs project-specific content${NC}"
        ((FAIL++))
    else
        echo -e "  ${GREEN}✅ README has project-specific content${NC}"
        ((PASS++))
    fi
else
    echo -e "  ${RED}❌ README.md not found${NC}"
    ((FAIL++))
fi

# ==============================================================================
# Summary
# ==============================================================================

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo -e "${BLUE}  SUMMARY${NC}"
echo "═══════════════════════════════════════════════════════════════"
echo ""

total_components=$(ls -1 "$COMPONENTS_DIR"/*.tsx 2>/dev/null | wc -l | tr -d ' ')
total_hooks=$(ls -1 "$HOOKS_DIR"/*.ts 2>/dev/null | wc -l | tr -d ' ')
total_layouts=$(ls -1 "$LAYOUTS_DIR"/*.tsx 2>/dev/null | wc -l | tr -d ' ')

echo "  Files Analyzed:"
echo "    Components: $total_components"
echo "    Hooks: $total_hooks"
echo "    Layouts: $total_layouts"
echo ""
echo "  Documentation Coverage:"
echo "    @module headers: $module_pass/$((module_pass + module_fail)) ($((module_pass * 100 / (module_pass + module_fail + 1)))%)"
echo "    @example blocks: $example_pass/$((example_pass + example_fail)) ($((example_pass * 100 / (example_pass + example_fail + 1)))%)"
if [[ $state_total -gt 0 ]]; then
    echo "    useState docs: $state_documented/$state_total ($state_pct%)"
fi
if [[ $effect_total -gt 0 ]]; then
    echo "    useEffect docs: $effect_documented/$effect_total ($effect_pct%)"
fi
echo "    Section comments: $section_pass/$((section_pass + section_fail)) (files >400 lines)"
echo ""

if [[ $module_fail -eq 0 && $section_fail -eq 0 ]]; then
    echo -e "  ${GREEN}✅ All MUST requirements passed${NC}"
    exit_code=0
else
    echo -e "  ${RED}❌ Some MUST requirements failed${NC}"
    exit_code=1
fi

echo ""
echo "  SHOULD improvements (not blocking):"
echo "    - Add comments to remaining useState hooks"
echo "    - Add comments to remaining useEffect hooks"
echo "    - Add @example blocks to components without them"
echo ""

exit $exit_code
