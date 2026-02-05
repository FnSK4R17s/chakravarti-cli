#!/bin/bash
# Rust Documentation Analysis Script
# Analyzes all crates for documentation convention compliance

CRATES_DIR="/apps/chakravarti-cli/crates"
OUTPUT_FILE="${CRATES_DIR}/.scripts/analysis_results.md"

echo "# Rust Documentation Analysis" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Get crate list
CRATES=$(ls -d ${CRATES_DIR}/ckrv-* 2>/dev/null | xargs -I{} basename {})

echo "## Summary Statistics" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

total_files=0
total_lines=0
files_with_module_docs=0
files_over_500=0
files_needing_sections=0

# Analyze each crate
for CRATE in $CRATES; do
  CRATE_DIR="${CRATES_DIR}/${CRATE}/src"
  
  if [ ! -d "$CRATE_DIR" ]; then
    continue
  fi
  
  crate_files=0
  crate_lines=0
  crate_module_docs=0
  crate_over_500=0
  crate_needs_sections=0
  
  echo "### ${CRATE}" >> "$OUTPUT_FILE"
  echo "" >> "$OUTPUT_FILE"
  
  # File analysis
  echo "| File | Lines | Module Doc | Sections |" >> "$OUTPUT_FILE"
  echo "|------|-------|------------|----------|" >> "$OUTPUT_FILE"
  
  for f in $(find "$CRATE_DIR" -name "*.rs" 2>/dev/null | sort); do
    fname=$(basename "$f")
    lines=$(wc -l < "$f" 2>/dev/null || echo 0)
    
    # Check module docs
    if head -10 "$f" | grep -q "^//!"; then
      has_mod_doc="✅"
      ((crate_module_docs++))
    else
      has_mod_doc="❌"
    fi
    
    # Check sections (only for files > 100 lines)
    if [ "$lines" -gt 100 ]; then
      sections=$(grep -c "// ===" "$f" 2>/dev/null || echo 0)
      if [ "$sections" -lt 2 ]; then
        section_status="⚠️ $sections"
        ((crate_needs_sections++))
      else
        section_status="✅ $sections"
      fi
    else
      section_status="-"
    fi
    
    # Track large files
    if [ "$lines" -gt 500 ]; then
      ((crate_over_500++))
      lines_display="🔴 $lines"
    elif [ "$lines" -gt 300 ]; then
      lines_display="⚠️ $lines"
    else
      lines_display="$lines"
    fi
    
    echo "| $fname | $lines_display | $has_mod_doc | $section_status |" >> "$OUTPUT_FILE"
    
    ((crate_files++))
    ((crate_lines+=lines))
  done
  
  echo "" >> "$OUTPUT_FILE"
  echo "**Crate Stats:** $crate_files files, $crate_lines lines, $crate_module_docs with module docs, $crate_over_500 over 500 LOC, $crate_needs_sections need sections" >> "$OUTPUT_FILE"
  echo "" >> "$OUTPUT_FILE"
  
  # Update totals
  ((total_files+=crate_files))
  ((total_lines+=crate_lines))
  ((files_with_module_docs+=crate_module_docs))
  ((files_over_500+=crate_over_500))
  ((files_needing_sections+=crate_needs_sections))
done

# Add totals at the top
sed -i "s/## Summary Statistics/## Summary Statistics\n\n| Metric | Value |\n|--------|-------|\n| Total Files | $total_files |\n| Total Lines | $total_lines |\n| Files with Module Docs | $files_with_module_docs |\n| Files > 500 LOC | $files_over_500 |\n| Files Needing Sections | $files_needing_sections |/" "$OUTPUT_FILE"

echo "" >> "$OUTPUT_FILE"
echo "## Files Missing Module Docs" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

for CRATE in $CRATES; do
  CRATE_DIR="${CRATES_DIR}/${CRATE}/src"
  for f in $(find "$CRATE_DIR" -name "*.rs" 2>/dev/null); do
    if ! head -10 "$f" | grep -q "^//!"; then
      echo "- ${f#${CRATES_DIR}/}" >> "$OUTPUT_FILE"
    fi
  done
done

echo "" >> "$OUTPUT_FILE"
echo "## Files > 500 Lines (Need Splitting)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

find "$CRATES_DIR" -name "*.rs" -exec wc -l {} \; 2>/dev/null | awk -v dir="$CRATES_DIR" '$1 > 500 {
  sub(dir "/", "", $2)
  print "- 🔴 **" $1 " lines**: " $2
}' | sort -t: -k1 -rn >> "$OUTPUT_FILE"

echo "" >> "$OUTPUT_FILE"
echo "Analysis complete. Results written to: $OUTPUT_FILE"
cat "$OUTPUT_FILE"
