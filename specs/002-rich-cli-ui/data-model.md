# Data Model: Rich CLI UI

**Domain**: Presentation Layer & Theme Configuration

## Entities

### `Theme`

Central configuration for all visual elements. This ensures consistency across the CLI.

| Field | Type | Description |
|-------|------|-------------|
| `primary_color` | `Color` | Main brand color (for headers, banners). |
| `secondary_color` | `Color` | Subtitles, less important info. |
| `success_style` | `Style` | Green + Bold (for "Success" messages). |
| `error_style` | `Style` | Red + Bold (for "Error" messages). |
| `warning_style` | `Style` | Yellow (for warnings). |
| `box_chars` | `BoxChars` | Characters used for drawing panels/tables (ASCII vs Unicode). |
| `markdown_skin` | `MadSkin` | `termimad` skin for rendering markdown content. |

### `OutputComponent` (Abstract)

Represents any UI element that can be rendered to the terminal.

| Variant | Properties | Description |
|---------|------------|-------------|
| `Banner` | `title`, `subtitle` | The ASCII art banner shown on startup. |
| `Panel` | `content`, `title`, `level` | A boxed message (Info, Success, Error). |
| `Table` | `headers`, `rows` | Structured data grid. |
| `Markdown` | `raw_text` | Raw markdown string to be rendered. |

### `SpinnerState`

Represents the state of a long-running task.

| State | Description |
|-------|-------------|
| `Active` | Spinner is animating. |
| `Success` | Replaced by a Green Checkmark + Message. |
| `Error` | Replaced by a Red X + Message. |
| `Hidden` | No output (CI/CD mode). |

## Validation & Logic

### TTY Detection Logic

1.  Check `std::io::stdout().is_terminal()`.
2.  Check for `NO_COLOR` environment variable.
3.  Check for `CLICOLOR_FORCE` environment variable.
4.  **Result**: `UiContext.can_render_rich_text` (bool).

### Fallback Logic

*   **If `!can_render_rich_text`**:
    *   Colors are stripped (using `console::strip_ansi_codes`).
    *   Spinners are replaced by a simple log line "Starting..." and "Finished." or disabled.
    *   Tables key-value pairs or simplified ASCII tables.
    *   Box/Panels become plain text headers: `[INFO] Title: Content`.
