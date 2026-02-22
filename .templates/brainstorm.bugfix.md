# {{TITLE}} — Bugfix Tasks ({{BUGFIX_NUMBER}})

**Brainstorm**: [notes.md](./notes.md)
**Created**: {{DATE}}
**Source**: Post-implementation review in notes.md

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
<!-- BUGFIX_TABLE -->

---

<!-- BUGFIX_TASKS -->

## Verification

After all bugfixes are applied:

- [ ] `cargo build -p {{CRATE}}` succeeds with no errors
- [ ] `cargo clippy -p {{CRATE}} -- -D warnings` passes (or warnings are reduced)
- [ ] `cargo test -p {{CRATE}}` passes
- [ ] Manual smoke test of affected functionality

## Notes

<!-- Additional notes about the bugfix batch -->
