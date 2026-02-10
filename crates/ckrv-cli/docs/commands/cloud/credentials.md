---
command: cloud credentials
generated_from: crates/ckrv-cli/src/commands/cloud/credentials.rs
last_commit: 1b27ca2
---

# ckrv cloud credentials

Manage git credentials for private repositories

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a new git credential |
| `list` | List stored credentials |
| `remove` | Remove a credential |

### add

| Flag | Description |
|------|-------------|
| `--name` | Name for this credential (e.g., "github-work") |
| `--provider` | Git provider: github, gitlab, bitbucket, generic (default: github) |
| `--credential-type` | Credential type: pat, deploy_key (default: pat) |

### list

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

### remove

| Argument | Required | Description |
|----------|----------|-------------|
| `name` | Yes | Name of the credential to remove |
