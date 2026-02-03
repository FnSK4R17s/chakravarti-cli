# ckrv-ui Frontend

React-based dashboard for the Chakravarti CLI orchestration system.

## Overview

This is the frontend application for `ckrv-ui`, providing a visual interface for:

- **Spec Management**: Create, view, and edit feature specifications
- **Task Tracking**: Monitor implementation tasks and progress
- **Plan Visualization**: View execution plans as DAGs with model assignments
- **Execution Monitoring**: Real-time logs and batch progress tracking
- **Agent Configuration**: Manage AI agent configurations
- **Test & QA**: Run tests and quality reviews

## Architecture

```
src/
├── components/       # React components (27 files)
│   ├── AgentManager.tsx      # Agent configuration UI
│   ├── ExecutionRunner.tsx   # Main execution dashboard
│   ├── PlanEditor.tsx        # DAG-based plan visualization
│   ├── SpecEditor.tsx        # Specification viewer
│   ├── TaskEditor.tsx        # Task management interface
│   ├── TestRunner.tsx        # Test execution interface
│   ├── QAReviewer.tsx        # Quality assurance UI
│   └── ...                   # 20 more components
├── hooks/            # Custom React hooks (12 files)
│   ├── useSpec.ts            # Spec CRUD operations
│   ├── useLogStore.ts        # Centralized log management
│   ├── useWebSocketReconnect.ts # Auto-reconnecting WebSocket
│   └── ...                   # 9 more hooks
├── layouts/          # Layout components
│   └── Dashboard.tsx         # Main app layout with navigation
├── lib/              # Utility functions
└── types/            # TypeScript definitions
```

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Type checking
npm run check
```

## Components

All components include `@module` JSDoc headers with:
- Description of purpose
- Context of where component is used
- Dependencies (hooks, parent components)
- Example usage patterns

Large components (>400 lines) include section comments:
- `// === IMPORTS ===`
- `// === TYPES ===`
- `// === MAIN COMPONENT ===`
- `// === STATE ===`
- `// === HANDLERS ===`
- `// === SUB-COMPONENTS ===`

## Hooks

| Hook | Purpose |
|------|---------|
| `useSpec` | Fetch and manage specifications |
| `useLogStore` | Centralized log state management |
| `useWebSocketReconnect` | Auto-reconnecting WebSocket |
| `useAutoSelectedSpec` | Branch-based spec selection |
| `useFocusTrap` | Modal focus management |
| `useRunHistory` | Execution history tracking |
| `useWorkflowProgress` | Pipeline progress tracking |

## API Integration

The frontend communicates with the Rust backend via:
- **REST API**: `/api/*` endpoints for CRUD operations
- **SSE**: `/api/events` for real-time log streaming
- **WebSocket**: `/ws/*` for interactive terminal sessions

## UI Framework

Built with:
- **React 19** with TypeScript
- **Vite** for fast development
- **shadcn/ui** component library
- **TanStack Query** for data fetching
- **Lucide** icons

## AI Context

This documentation is designed for AI agents. Each file includes:
1. `@module` header explaining purpose and context
2. Section comments for navigation in large files
3. Type definitions for all props and state
4. Example usage patterns

See individual component files for detailed documentation.
