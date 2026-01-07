# Quickstart: Testing Persistent Run History

## Prerequisites

- Chakravarti CLI built (`make install`)
- `ckrv ui --port 3002` running
- A spec with batches (e.g., in `/apps/chakra-test`)

## Manual Testing Scenarios

### Scenario 1: Run History Persists After Refresh

1. Navigate to Runner page (`http://localhost:3002/runner`)
2. Select a specification with a plan
3. Click **Run Execution** (or Dry Run)
4. Wait for at least one batch to complete
5. **Refresh the page** (F5 or Cmd+R)
6. Select the same specification

**Expected**:
- Run appears in the History panel with correct status
- If still running, shows "Running" with elapsed time
- If completed, shows "Completed ✓" with final stats

**Verify**:
- Check `.specs/<spec-name>/runs.yaml` exists with run data
- Run ID matches what's displayed in UI

---

### Scenario 2: Run History Persists After Tab Switch

1. Start an execution on Runner page
2. Navigate to another page (e.g., Planner)
3. Return to Runner page

**Expected**:
- Run history shows the in-progress or completed run
- Status is accurate (not stale)

---

### Scenario 3: Multiple Runs Show in Chronological Order

1. Run an execution (dry run is fine)
2. Wait for completion
3. Run another execution
4. Check history panel

**Expected**:
- Both runs appear in the history
- Newest run is at the top
- Each has correct timestamp and status

---

### Scenario 4: Completion Summary Displays

1. Run a full execution (not dry run)
2. Wait for all batches to complete

**Expected**:
- Completion summary panel appears
- Shows: "X/Y batches completed"
- Shows: Total elapsed time
- Shows: Number of branches merged
- Visual celebration (checkmark, success color)

---

### Scenario 5: Partial Failure Summary

1. Create a spec that will fail (e.g., invalid task)
2. Run execution
3. Wait for failure

**Expected**:
- Summary shows partial results: "3/5 batches completed"
- Failed batches are clearly marked
- Error message is visible
- Status shows "Failed" (not "Completed")

---

### Scenario 6: UI Consistency Check

1. Open Runner page
2. Open Planner page in another tab
3. Compare visually:
   - Header height and styling
   - Panel layout (sidebar + content)
   - Card shadows and borders
   - Button styles
   - Empty state messages

**Expected**:
- Visual consistency across pages
- Same color palette, typography, spacing
- Same component patterns

---

### Scenario 7: Concurrent Run Prevention

1. Open Runner in two browser tabs
2. In Tab 1: Start an execution
3. In Tab 2: Try to start another execution for same spec

**Expected**:
- Tab 2 shows warning: "Another run is in progress"
- Options: Cancel, View Existing, Override
- "View Existing" navigates to the running execution

---

### Scenario 8: Corrupted History Graceful Degradation

1. Find `.specs/<spec-name>/runs.yaml`
2. Corrupt the file (add invalid YAML: `{{{{{`)
3. Refresh Runner page and select that spec

**Expected**:
- Page loads without crashing
- Warning message: "Unable to load run history"
- New runs can still be started
- File is fixed/recreated on next successful run

---

## API Testing (curl)

### List Runs
```bash
curl http://localhost:3002/api/history/001-build-todo-list
```

### Get Single Run
```bash
curl http://localhost:3002/api/history/001-build-todo-list/run-2026-01-07-abc123
```

### Check for Empty History
```bash
curl http://localhost:3002/api/history/new-spec-without-runs
# Should return { "success": true, "runs": [], "total_count": 0 }
```

---

## Verification Checklist

- [ ] runs.yaml is created in spec directory on first run
- [ ] Run data persists across page refresh
- [ ] Run data persists across tab switches
- [ ] Multiple runs appear in correct order (newest first)
- [ ] Completion summary shows accurate stats
- [ ] Failed runs show clear error information
- [ ] UI matches other pages (header, panels, cards)
- [ ] Concurrent run prevention works
- [ ] Corrupted file doesn't crash the app
- [ ] File is atomic-written (no partial corruption)

---

## File Locations Reference

| File | Purpose |
|------|---------|
| `.specs/<spec>/runs.yaml` | Per-spec run history |
| `crates/ckrv-ui/src/services/history.rs` | YAML read/write service |
| `crates/ckrv-ui/src/api/history.rs` | REST API handlers |
| `frontend/src/components/RunHistoryPanel.tsx` | History list UI |
| `frontend/src/components/CompletionSummary.tsx` | Run summary UI |
