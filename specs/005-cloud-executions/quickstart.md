# Quickstart: Cloud Executions

**Feature**: 005-cloud-executions  
**Audience**: Developers using Chakravarti CLI  
**Time to Complete**: ~10 minutes

---

## Prerequisites

- Chakravarti CLI installed (`ckrv --version` shows v0.2.0+)
- A git repository with a valid spec file
- Internet connection

---

## Step 1: Authenticate with Chakravarti Cloud

```bash
ckrv cloud login
```

This opens your browser to complete authentication. You'll see:

```
▌ ✔ Device Code Generated
▌ Visit: https://chakravarti.dev/activate
▌ Enter code: ABCD-1234
▌ 
▌ Waiting for authorization...
```

After authorizing in your browser:

```
▌ ✔ Authenticated as you@example.com
▌ Subscription: Pro (87 jobs remaining this month)
```

---

## Step 2: Add Git Credentials (for private repos)

If your repository is private, add a Personal Access Token:

```bash
ckrv cloud credentials add --name github-work --provider github
# Prompts for token securely

▌ ✔ Credential 'github-work' saved
```

List your stored credentials:

```bash
ckrv cloud credentials list
```

---

## Step 3: Run a Job in the Cloud

From your project directory with a spec:

```bash
ckrv run --cloud \
  --spec specs/my-feature/spec.md \
  --repo https://github.com/myorg/myrepo.git \
  --base-branch main \
  --credential github-work
```

Output:

```
▌ ✔ Job Dispatched
▌ Job ID: 3f8a9b2c-1234-5678-abcd-ef0123456789
▌ Status: pending
▌ 
▌ Track progress:  ckrv status 3f8a9b2c
▌ Stream logs:     ckrv logs 3f8a9b2c --follow
▌ Pull results:    ckrv pull 3f8a9b2c
```

---

## Step 4: Monitor Your Job

Check status:

```bash
ckrv status 3f8a9b2c

▌ Job: 3f8a9b2c-1234-5678-abcd-ef0123456789
▌ Status: running
▌ Phase: executing (agent: claude-code)
▌ Started: 2 minutes ago
▌ ETA: ~5 minutes
```

Stream logs in real-time:

```bash
ckrv logs 3f8a9b2c --follow

[12:34:56] orchestrator: Cloning repository...
[12:34:59] orchestrator: Starting agent: claude-code
[12:35:02] claude-code: Analyzing spec requirements...
[12:35:15] claude-code: Implementing feature...
...
```

---

## Step 5: Pull Results

When the job succeeds:

```bash
ckrv pull 3f8a9b2c

▌ ✔ Fetching results...
▌ ✔ Applying diff to worktree...
▌ 
▌ Changes applied:
▌   src/auth.rs       | 142 +++++
▌   src/main.rs       |  12 +-
▌   tests/auth_test.rs|  87 +++
▌ 
▌ Review with: git diff HEAD
▌ Commit with: git add . && git commit -m "Feature: cloud auth"
```

---

## Common Commands Reference

| Command | Description |
|---------|-------------|
| `ckrv cloud login` | Authenticate with Chakravarti Cloud |
| `ckrv cloud logout` | Clear stored credentials |
| `ckrv cloud whoami` | Show current user and quota |
| `ckrv cloud credentials add` | Add git credentials |
| `ckrv cloud credentials list` | List stored credentials |
| `ckrv cloud credentials remove` | Delete a credential |
| `ckrv run --cloud` | Dispatch job to cloud |
| `ckrv status <job-id>` | Check job status |
| `ckrv logs <job-id>` | View job logs |
| `ckrv logs <job-id> --follow` | Stream live logs |
| `ckrv pull <job-id>` | Download job results |

---

## Troubleshooting

### "Not authenticated"

Run `ckrv cloud login` to authenticate.

### "Quota exceeded"

Your monthly job quota is exhausted. Options:
- Wait for quota reset (shown in `ckrv cloud whoami`)
- Upgrade your subscription at https://chakravarti.dev/billing

### "Credential not found"

The specified `--credential` name doesn't exist. List available credentials:
```bash
ckrv cloud credentials list
```

### "Repository clone failed"

- Verify the repository URL is correct
- Ensure your credential has read access to the repo
- For GitHub, the PAT needs `repo` scope

---

## Next Steps

- Read the full [Cloud Executions Spec](./spec.md)
- Review [API Contracts](./contracts/cloud-api.yaml)
- Check [Data Model](./data-model.md) for entity details
