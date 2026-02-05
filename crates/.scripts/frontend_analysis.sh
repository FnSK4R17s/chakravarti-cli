#!/bin/bash
# Frontend Documentation Analysis Script

FRONTEND_DIR="/apps/chakravarti-cli/crates/ckrv-ui/frontend/src"
COMPONENTS_DIR="$FRONTEND_DIR/components"
HOOKS_DIR="$FRONTEND_DIR/hooks"

echo "# Frontend Documentation Analysis"
echo ""
echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Phase 2.5: Module Header Check
echo "## Module Headers (@module)"
echo ""
echo "| File | Status |"
echo "|------|--------|"

missing_headers=0
total_files=0

for file in "$COMPONENTS_DIR"/*.tsx "$HOOKS_DIR"/*.ts; do
  if [ -f "$file" ]; then
    name=$(basename "$file")
    ((total_files++))
    if head -30 "$file" | grep -q "@module"; then
      echo "| $name | ✅ |"
    else
      echo "| $name | ❌ Missing |"
      ((missing_headers++))
    fi
  fi
done

echo ""
echo "**Module Header Coverage:** $((total_files - missing_headers))/$total_files ($((100 * (total_files - missing_headers) / total_files))%)"
echo ""

# Phase 2.3: State Documentation Check
echo "## useState Documentation"
echo ""
echo "| Component | Documented | Total | Coverage |"
echo "|-----------|------------|-------|----------|"

for file in "$COMPONENTS_DIR"/*.tsx; do
  if [ -f "$file" ]; then
    component=$(basename "$file" .tsx)
    
    # Count documented state (JSDoc or comments before useState)
    documented=$(grep -B 1 "useState" "$file" 2>/dev/null | grep -c "\*\|//" || echo 0)
    
    # Count actual useState calls
    actual=$(grep -c "useState" "$file" 2>/dev/null || echo 0)
    
    if [ "$actual" -gt 0 ]; then
      if [ "$actual" -gt 0 ]; then
        coverage=$((documented * 100 / actual))
      else
        coverage=100
      fi
      
      if [ "$coverage" -lt 50 ]; then
        status="❌"
      elif [ "$coverage" -lt 100 ]; then
        status="⚠️"
      else
        status="✅"
      fi
      
      echo "| $component | $documented | $actual | $status $coverage% |"
    fi
  fi
done

echo ""

# Phase 2.4: Effect Documentation Check
echo "## useEffect Documentation"
echo ""
echo "| Component | Documented | Total | Coverage |"
echo "|-----------|------------|-------|----------|"

for file in "$COMPONENTS_DIR"/*.tsx; do
  if [ -f "$file" ]; then
    component=$(basename "$file" .tsx)
    
    # Count documented effects (comments before useEffect)
    documented=$(grep -B 1 "useEffect" "$file" 2>/dev/null | grep -c "\*\|//" || echo 0)
    
    # Count actual useEffect calls
    actual=$(grep -c "useEffect" "$file" 2>/dev/null || echo 0)
    
    if [ "$actual" -gt 0 ]; then
      if [ "$actual" -gt 0 ]; then
        coverage=$((documented * 100 / actual))
      else
        coverage=100
      fi
      
      if [ "$coverage" -lt 50 ]; then
        status="❌"
      elif [ "$coverage" -lt 100 ]; then
        status="⚠️"
      else
        status="✅"
      fi
      
      echo "| $component | $documented | $actual | $status $coverage% |"
    fi
  fi
done

echo ""

# Phase 3.3: Section Comments Check
echo "## Section Comments (files >200 lines)"
echo ""
echo "| Component | Lines | Sections | Status |"
echo "|-----------|-------|----------|--------|"

for file in "$COMPONENTS_DIR"/*.tsx; do
  if [ -f "$file" ]; then
    component=$(basename "$file" .tsx)
    lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
    
    if [ "$lines" -gt 200 ]; then
      sections=$(grep -c "// ===" "$file" 2>/dev/null || echo 0)
      
      if [ "$sections" -lt 3 ]; then
        status="⚠️ Need 3+"
      else
        status="✅"
      fi
      
      echo "| $component | $lines | $sections | $status |"
    fi
  fi
done

echo ""
echo "## Large Files (>500 lines)"
echo ""

for file in "$COMPONENTS_DIR"/*.tsx; do
  if [ -f "$file" ]; then
    component=$(basename "$file" .tsx)
    lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
    
    if [ "$lines" -gt 500 ]; then
      echo "- 🔴 **$component**: $lines lines"
    fi
  fi
done

echo ""
echo "Analysis complete."
