# ckrv-ui Frontend

React-based web dashboard for the Chakravarti CLI orchestration system.

## Overview

This frontend provides a visual interface for the Chakravarti autonomous agent orchestration engine:

- **Spec Management**: Create, view, and edit feature specifications
- **Task Tracking**: Monitor implementation tasks and progress
- **Plan Visualization**: View execution plans with model assignments
- **Execution Monitoring**: Real-time logs and batch progress tracking
- **Agent Configuration**: Manage AI agent configurations
- **Test & QA**: Run tests and quality reviews
- **Terminal Integration**: PTY terminal via Tauri for interactive sessions

## Tech Stack

| Technology | Purpose |
|------------|---------|
| **React 19** | UI framework with TypeScript |
| **Vite 7** | Fast build tool and dev server |
| **shadcn/ui** | Component library (Radix-based) |
| **TanStack Query** | Server state management |
| **Tailwind CSS v4** | Utility-first styling |
| **xterm.js** | Terminal emulator |
| **Tauri 2** | Desktop app framework with PTY support |
| **Lucide** | Icon library |

## Development Commands

```bash
# Install dependencies
npm install

# Start development server (hot reload)
npm run dev

# Build for production
npm run build

# Type checking
npx tsc --noEmit

# Lint
npm run lint

# Generate TypeScript types from Rust
npm run generate:types

# E2E tests
npm run test:e2e
npm run test:e2e:ui      # With UI
npm run test:e2e:headed  # Headed mode
```

## Project Structure

```
src/
├── components/           # React components
│   ├── ui/              # shadcn/ui components (button, card, dialog, etc.)
│   ├── AgentManager.tsx # Agent configuration UI
│   ├── SpecEditor.tsx   # Specification viewer/editor
│   ├── TaskEditor.tsx   # Task management interface
│   ├── PlanEditor.tsx   # Plan visualization
│   ├── TestRunner.tsx   # Test execution interface
│   ├── QAReviewer.tsx   # Quality assurance UI
│   ├── LogViewer.tsx    # Log display component
│   ├── LogTerminal.tsx  # Terminal-style log viewer
│   ├── DiffViewer.tsx   # Git diff visualization
│   ├── WorkflowPanel.tsx # Workflow progress UI
│   └── ...              # More components
├── hooks/               # Custom React hooks
│   ├── useSpec.ts       # Spec CRUD operations
│   ├── useLogStore.ts   # Centralized log management
│   ├── useWebSocketReconnect.ts # Auto-reconnecting WebSocket
│   ├── useWorkflowProgress.ts  # Pipeline progress tracking
│   ├── useRunHistory.ts # Execution history
│   ├── useAutoSelectedSpec.ts  # Branch-based spec selection
│   ├── useTauriPty.ts   # Tauri PTY terminal hook
│   └── ...              # More hooks
├── layouts/             # Layout components
│   └── Dashboard.tsx    # Main app layout with navigation
├── lib/                 # Utility functions
│   ├── api.ts           # API client
│   ├── utils.ts         # General utilities
│   └── theme.ts         # Theme utilities
├── types/               # TypeScript definitions
│   ├── api.generated.ts # Auto-generated from Rust
│   ├── websocket.ts     # WebSocket types
│   └── log.ts           # Log types
├── services/            # Service layer
│   └── logService.ts    # Log handling service
└── App.tsx              # Root component
```

## Key Components

### Application Components

| Component | Description |
|-----------|-------------|
| `AgentManager` | Manage AI agent configurations |
| `SpecEditor` | View and edit specifications |
| `SpecWorkflow` | Specification workflow panel |
| `TaskEditor` | Task management interface |
| `TaskDetailModal` | Detailed task view modal |
| `PlanEditor` | Execution plan visualization |
| `TestRunner` | Test execution interface |
| `TestFixModal` | Test fix suggestions modal |
| `QAReviewer` | Quality assurance review UI |
| `WorkflowPanel` | Workflow progress display |
| `RunHistoryPanel` | Execution history viewer |
| `LogViewer` | Log display component |
| `LogTerminal` | Terminal-style log viewer |
| `BatchLogTerminal` | Batch log terminal |
| `BatchLogCarousel` | Carousel for batch logs |
| `DiffViewer` | Git diff visualization |
| `CommandPalette` | Keyboard command palette |
| `ChatDashboard` | Chat interface dashboard |
| `StatusWidget` | Status indicator widget |
| `ProjectSelector` | Project selection dropdown |
| `AgentCliModal` | CLI modal for agent interaction |
| `CodePage` | Code viewing/editing page |
| `CompletionSummary` | Execution completion summary |
| `ClarifyModal` | Clarification request modal |

### UI Components (shadcn/ui)

Standard shadcn/ui components located in `components/ui/`:
`alert`, `badge`, `button`, `card`, `carousel`, `checkbox`, `collapsible`, `dialog`, `dropdown-menu`, `input`, `label`, `progress`, `radio-group`, `scroll-area`, `select`, `separator`, `skeleton`, `sonner`, `switch`, `tabs`, `textarea`, `toast`, `toaster`, `tooltip`

## API Integration

The frontend communicates with the Rust backend via:

| Endpoint | Protocol | Purpose |
|----------|----------|---------|
| `/api/*` | REST | CRUD operations |
| `/api/events` | SSE | Real-time log streaming |
| `/ws/*` | WebSocket | Interactive terminal sessions |

For detailed API documentation, see [../docs/api-reference.md](../docs/api-reference.md).

## Custom Hooks

| Hook | Purpose |
|------|---------|
| `useSpec` | Fetch and manage specifications |
| `useLogStore` | Centralized log state management |
| `useWebSocketReconnect` | Auto-reconnecting WebSocket |
| `useAutoSelectedSpec` | Branch-based spec selection |
| `useFocusTrap` | Modal focus management |
| `useRunHistory` | Execution history tracking |
| `useWorkflowProgress` | Pipeline progress tracking |
| `useTauriPty` | Tauri PTY terminal integration |
| `useExecutionStream` | Real-time execution streaming |
| `useConnection` | Connection status tracking |
| `useCommand` | Command execution handling |

## Styling

All colors are centralized in `src/index.css` using OKLCH format with Tailwind CSS v4's `@theme inline` directive:

```css
:root {
  --accent-cyan: oklch(0.82 0.19 195);
  --accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  /* ... more theme colors ... */
}
```

Use semantic Tailwind utilities like `text-accent-cyan`, `bg-accent-green-dim` instead of hardcoded colors.

## Code Conventions

All components include `@module` JSDoc headers with:
- Description of purpose
- Context of where component is used
- Dependencies (hooks, parent components)

Large components (>400 lines) use section comments:
- `// === IMPORTS ===`
- `// === TYPES ===`
- `// === MAIN COMPONENT ===`
- `// === STATE ===`
- `// === HANDLERS ===`
- `// === SUB-COMPONENTS ===`

See [FRONTEND_CONVENTIONS.md](../FRONTEND_CONVENTIONS.md) for detailed guidelines.

## Related Documentation

- [API Reference](../docs/api-reference.md) - Backend API documentation
- [Frontend Conventions](../FRONTEND_CONVENTIONS.md) - Coding standards
- [Architecture](../../docs/architecture.md) - System architecture overview
