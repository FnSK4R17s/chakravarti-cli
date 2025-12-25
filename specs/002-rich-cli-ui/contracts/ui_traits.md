# UI Contracts & Interfaces

**Language**: Rust
**Location**: `crates/ckrv-cli/src/ui/mod.rs` (and submodules)

## Core Traits

### `Renderable`

Any struct that can be displayed using the rich UI engine must implement this.

```rust
pub trait Renderable {
    /// Render the component to a string, respecting the provided theme.
    fn render(&self, theme: &Theme) -> String;
}
```

### `UiContext`

The main entry point for commands to interact with the UI.

```rust
pub struct UiContext {
    theme: Theme,
    is_interactive: bool,
}

impl UiContext {
    /// Create a new UI context, auto-detecting terminal capabilities.
    pub fn new() -> Self;

    /// Print a component to stdout.
    pub fn print(&self, component: impl Renderable);

    /// Start a spinner with a message.
    /// Returns a guard that stops the spinner on drop.
    pub fn spinner(&self, msg: impl Into<String>) -> SpinnerGuard;
    
    /// Print a success message in a panel.
    pub fn success(&self, title: &str, msg: &str);
    
    /// Print an error message in a panel.
    pub fn error(&self, title: &str, msg: &str);
    
    /// Render markdown content.
    pub fn markdown(&self, content: &str);
}
```

## External Defines

### `indicatif` Usage

We will wrap `indicatif::ProgressBar` to simplify usage for the rest of the app:

```rust
pub struct SpinnerGuard {
    inner: Option<indicatif::ProgressBar>, // Option serves non-interactive mode
}

impl SpinnerGuard {
    pub fn set_message(&self, msg: &str);
    pub fn finish_with_message(&self, msg: &str);
    pub fn success(&self, msg: &str);
    pub fn error(&self, msg: &str);
}
```

### `tabled` Integration

We will provide a helper to standardise table styles:

```rust
pub fn create_table<T: Tabled>(data: Vec<T>) -> Table {
    let mut table = Table::new(data);
    table.with(Style::modern()); // Apply our standard style
    table
}
```
