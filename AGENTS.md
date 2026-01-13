# AGENTS.md

This file provides guidance to AI agents (Claude Code, Cursor, Gemini, etc.) when working with code in this repository.

## Project Overview

Chakravarti (ckrv) is a Rust-based CLI tool with a React frontend for orchestrating AI-powered code generation tasks. The project structure:

```
chakravarti-cli/
├── crates/
│   ├── ckrv-cli/        # Main Rust CLI binary
│   ├── ckrv-ui/         # UI server crate
│   │   └── frontend/    # React/TypeScript frontend (Vite + Tailwind v4)
│   └── ckrv-cloud/      # Cloud execution crate
├── npm/                 # npm package for distribution
├── specs/               # Feature specifications
└── Makefile            # Build automation
```

## Essential Development Commands

### After Making Changes

**⚠️ CRITICAL**: After making any code changes, you MUST run:

```bash
# From the repository root
make install
```

This command:
1. Builds the Rust CLI in release mode
2. Builds the frontend (npm run build)
3. Copies the binary to npm/bin/
4. Links the npm package globally
5. Installs to ~/.cargo/bin/

### Frontend Development

```bash
# Navigate to frontend directory
cd crates/ckrv-ui/frontend

# Development mode (hot reload)
npm run dev

# Production build
npm run build

# Type checking
npx tsc --noEmit
```

### Rust Development

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Check for errors without building
cargo check

# Run tests
cargo test
```

## Frontend Architecture

### Tech Stack
- **React 18** with TypeScript
- **Tailwind CSS v4** with `@theme inline` for custom utilities
- **Vite** for bundling
- **shadcn/ui** components (Radix-based)
- **TanStack Query** for data fetching

### CSS Theme System

All colors are centralized in `crates/ckrv-ui/frontend/src/index.css` using OKLCH format:

```css
:root {
  /* === THEME COLORS START === */
  --accent-cyan: oklch(0.82 0.19 195);
  --accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  /* ... more colors ... */
  /* === THEME COLORS END === */
}
```

**When styling components:**
- ✅ Use semantic Tailwind utilities: `text-accent-cyan`, `bg-accent-green-dim`
- ✅ Use shadcn semantic colors: `text-foreground`, `bg-muted`, `border-border`
- ❌ Avoid inline `style={}` with hardcoded colors
- ❌ Avoid arbitrary values like `text-[#00ff00]`

### Component Guidelines

1. **Use shadcn/ui components** from `@/components/ui/`
2. **Import paths** use `@/` alias for `src/`
3. **State management** via TanStack Query for server state
4. **Icons** from `lucide-react`

## Testing

### Frontend
```bash
cd crates/ckrv-ui/frontend
npm run build  # Fastest validation - catches type errors
```

### Rust
```bash
cargo test
```

## Common Patterns

### Adding a New shadcn Component
```bash
cd crates/ckrv-ui/frontend
npx shadcn@latest add [component-name]
```

### API Endpoints
All API endpoints are served by the Rust backend through `/api/`. See `crates/ckrv-ui/src/routes/` for available endpoints.

## Important Notes

1. **Tailwind v4**: Uses `@theme inline` for custom utilities, not `tailwind.config.js` extend
2. **OKLCH colors**: All theme colors use OKLCH format for better color manipulation
3. **Dark mode only**: The UI is dark-mode only (no light theme toggle)
4. **Build before testing**: Always run `make install` before testing CLI changes

## Troubleshooting

### "command not found: ckrv"
Run `make install` from the repository root.

### Frontend changes not appearing
1. Run `npm run build` in the frontend directory
2. Run `make install` from root
3. Restart `ckrv ui`

### CSS lint warnings about @plugin, @theme, @apply
These are Tailwind v4 directives - the IDE linter doesn't recognize them but they work correctly at build time.
