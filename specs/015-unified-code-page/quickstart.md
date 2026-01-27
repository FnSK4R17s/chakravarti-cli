# Quickstart: Unified Code Page

**Branch**: `015-unified-code-page`  
**Date**: 2026-01-24

## Prerequisites

- Node.js 20+
- pnpm (for frontend)
- Rust toolchain (for backend)
- The `ckrv` CLI installed

## Development Setup

### 1. Checkout the feature branch

```bash
cd /apps/chakravarti-cli
git checkout 015-unified-code-page
```

### 2. Install frontend dependencies

```bash
cd crates/ckrv-ui/frontend
pnpm install
```

### 3. Start the development server

Option A: Use the CLI (starts backend + frontend):
```bash
ckrv ui --port 3002
```

Option B: Start frontend dev server directly (if backend is running separately):
```bash
cd crates/ckrv-ui/frontend
pnpm dev
```

### 4. Access the UI

Open http://localhost:3002 in your browser.

## Key Files to Modify

| File | Change Type | Purpose |
|------|-------------|---------|
| `src/App.tsx` | UPDATE | Add CodePage route, remove old routes |
| `src/layouts/Dashboard.tsx` | UPDATE | Update sidebar navigation |
| `src/components/CodePage.tsx` | NEW | Create unified tabbed component |
| `tests/e2e/code-page.spec.ts` | NEW | E2E tests for tab navigation |

## Implementation Order

1. **Create `CodePage.tsx`** - The new tabbed container component
2. **Update `App.tsx`** - Replace 4 page conditions with single CodePage condition
3. **Update `Dashboard.tsx`** - Reduce nav items from 9 to 5
4. **Add E2E tests** - Verify tab navigation works correctly
5. **Test manually** - Ensure all existing functionality still works

## Testing

### Run E2E tests
```bash
cd crates/ckrv-ui/frontend
pnpm test:e2e
```

### Run specific test file
```bash
pnpm test:e2e tests/e2e/code-page.spec.ts
```

### Run tests in headed mode (see browser)
```bash
pnpm test:e2e:headed
```

## Verification Checklist

- [ ] Code page shows 4 tabs: Spec, Tasks, Plan, Run
- [ ] Clicking each tab shows the correct component
- [ ] Sidebar shows 5 items: Dashboard, Agents, Code, Test, QA
- [ ] Tab switching is instant (no page reload)
- [ ] All existing component functionality works (editing, saving, executing)
- [ ] Keyboard navigation between tabs works (arrow keys)
- [ ] Page header updates based on active tab

## Rollback

If issues arise, revert to the previous branch:

```bash
git checkout main  # or previous working branch
```

The existing components are unchanged, so no data migration is needed.
