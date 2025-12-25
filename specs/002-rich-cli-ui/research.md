# Research: Rich CLI UI

**Context**: Implementing a rich, premium terminal UI for Chakravarti CLI, similar to Gemini/Claude CLIs.
**Goals**: Branding (Banner), Rich Data (Tables), Markdown rendering, Spinners, robust TTY detection.

## Decision 1: Progress Indicators & Spinners

*   **Choice**: `indicatif`
*   **Rationale**: The de-facto standard in the Rust ecosystem. It supports:
    *   Auto-detection of TTY (falls back to logging or silence).
    *   Steady ticks for indeterminate states (spinners).
    *   ProgressBar for determinate states.
    *   Integration with `tracing` (via `tracing-indicatif` if needed later) to prevent log interference.
*   **Alternatives Considered**:
    *   `spinners`: Simpler, but less feature-rich and less maintained.
    *   Manual implementation: Waste of effort for complex TTY handling.

## Decision 2: Markdown & Rich Text Rendering

*   **Choice**: `termimad`
*   **Rationale**: specifically designed to render Markdown snippets in a terminal.
    *   Supports custom "Skins" to define colors for headers, bold, code blocks, etc. (Crucial for the "Premium" look).
    *   Handles wrapping and alignment.
    *   Can render tables from Markdown as well, though a dedicated table crate is often better for data.
*   **Alternatives Considered**:
    *   `bat`: Excellent syntax highlighting, but heavy and focused on files, not snippets.
    *   `skin`: (Actually part of termimad).

## Decision 3: Tables & structured Data

*   **Choice**: `tabled`
*   **Rationale**:
    *   Modern, actively maintained.
    *   Supports "Grid" layouts and heavily customizable borders/padding (essential for the "Box" UI requirement).
    *   Macro support `#[derive(Tabled)]` makes it easy to turn structs into tables.
*   **Alternatives Considered**:
    *   `comfy-table`: Good, but `tabled` has better derive ergonomics and flexible layouting.
    *   `prettytable-rs`: Older, less active.

## Decision 4: Styling & Terminal Manipulation

*   **Choice**: `console` + `dialoguer`
*   **Rationale**:
    *   `console`: Provides `Style`, `Emoji`, and robust terminal abstraction (colors, cursor movement). It works well with `indicatif`.
    *   `dialoguer`: Useful for any future interactive prompts (confirmations, selects), maintaining a consistent look.
*   **Alternatives Considered**:
    *   `crossterm`: Lower level. Good provided `ratatui` isn't used (which is TUI, not CLI).
    *   `anstyle`: Good for simple colors, but `console` offers more "UI" features.

## Decision 5: Theme Management

*   **Strategy**: Centralized `Theme` struct in `ckrv-cli` or `ckrv-core`.
    *   The `Theme` will hold `termimad::MadSkin` for markdown and `console::Style` for raw text.
    *   This ensures consistency across all commands.

## Implementation Details

*   **Docker/CI**: All selected libraries respect `NO_COLOR` and `CLICOLOR` standards, and `is_terminal` checks.
*   **Dependencies**: Add to `crates/ckrv-cli/Cargo.toml`.
