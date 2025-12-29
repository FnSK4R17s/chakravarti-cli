# Quickstart: Contributing to web-ui-extension

## Prerequisites
- Rust 1.75+
- Node.js 18+
- pnpm (recommended) or npm

## Setup Workspace
This feature introduces a hybrid workspace (Rust + Node).

1. **Install Frontend Dependencies**:
   ```bash
   cd crates/ckrv-ui/frontend
   npm install
   ```

2. **Frontend Development (Standalone)**:
   Use Vite dev server for hot-reloading UI work.
   ```bash
   # In crates/ckrv-ui/frontend
   npm run dev
   # Runs on http://localhost:5173
   ```
   *Note*: requires a running backend or mocked API.

3. **Backend Development**:
   Run the CLI with the `ui` command.
   ```bash
   cargo run -p ckrv-cli -- ui
   # Runs on http://localhost:3000
   ```

## Integration Build
To test the full embedded binary flow:

1. Build Frontend:
   ```bash
   cd crates/ckrv-ui/frontend && npm run build
   ```
2. Run Cargo:
   ```bash
   cargo run -p ckrv-cli -- ui
   ```
