# Quickstart: Using Rich UI

**How to make your command look awesome.**

## 1. Get the Context

The `UiContext` should be initialized at `main.rs` startup and passed to your command (or available via a dependency injection mechanism/global if typical for this codebase).

```rust
let ui = UiContext::new();
```

## 2. Panels and Messages

Don't use `println!` directly for status updates.

```rust
// Good
ui.success("Deployed", "The application is now live at https://...");
ui.error("Failed", "Could not connect to database.");
ui.markdown("# Implementation Plan
Here is the plan...");
```

## 3. Long Running Tasks (Spinners)

Wrap your logic in a spinner block:

```rust
let spinner = ui.spinner("Analyzing codebase...");
// Do heavy work
std::thread::sleep(std::time::Duration::from_secs(2));
spinner.success("Analysis complete!");
```

## 4. Tables

If you have a struct, derive `Tabled`:

```rust
use tabled::Tabled;

#[derive(Tabled)]
struct User {
    name: String,
    role: String,
}

let users = vec![User { name: "Alice".into(), role: "Admin".into() }];
ui.print(create_table(users));
```
