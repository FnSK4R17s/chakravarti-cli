# CLI Contract: `ckrv task`

## Usage

```bash
ckrv task [OPTIONS] <DESCRIPTION>
```

## Arguments

- `<DESCRIPTION>`: (Required) Natural language description of the task.

## Options

- `-w, --workflow <NAME>`: Name of the workflow to use (default: `default` or `swe`).
- `--dry-run`: Don't execute, just show the plan/steps.
- `-c, --continue <TASK_ID>`: Resume a previous task.
- `--json`: Output structured JSON to stdout (for CI/CD and scripting).
- `--no-sandbox`: (Dangerous/Dev-only) Run on host. *Maybe exclude for MVP*.

## Exit Codes

- `0`: Task completed successfully.
- `1`: Task failed (agent error, docker error).
- `2`: Configuration error (missing credentials, docker not running).

## Output (stdout)

```text
[   OK   ] Sandbox created (id: a1b2c3d4)
[   OK   ] Workspace prepared (.ckrv/tasks/1/workspace)
[ RUNNING] Step 1: Planning...
[   OK   ] Step 1 completed. Output: plan.md
[ RUNNING] Step 2: Implementation...
[   OK   ] Step 2 completed.
[SUCCESS ] Task 1 completed.
           View results at: .ckrv/tasks/1/workspace
```
