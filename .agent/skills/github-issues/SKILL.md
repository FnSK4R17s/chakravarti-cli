---
name: github-issues
description: Fetch GitHub issues for any public repository using curl. No authentication required. Use when you need to view open issues, find tasks, or review bug reports.
license: MIT
compatibility: Requires curl and jq
metadata:
  author: FnSK4R17s
  version: "1.0"
---

# GitHub Issues (curl)

Fetch GitHub issues for public repositories using the GitHub REST API with curl. No authentication required for public repos.

## Prerequisites

- **curl** - Usually pre-installed on Linux/macOS
- **jq** - For JSON parsing (optional but recommended)
  ```bash
  sudo apt install jq  # Ubuntu/Debian
  brew install jq      # macOS
  ```

## Quick Commands

### Get Repo Info from Git Remote

```bash
# Extract owner/repo from current git remote
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
echo $REPO  # e.g., FnSK4R17s/chakravarti-cli
```

### Fetch Open Issues

```bash
# Basic - list open issues (JSON)
curl -s "https://api.github.com/repos/OWNER/REPO/issues?state=open&per_page=15"

# With current repo
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
curl -s "https://api.github.com/repos/$REPO/issues?state=open&per_page=15"
```

### Pretty Print Issue Titles

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
curl -s "https://api.github.com/repos/$REPO/issues?state=open&per_page=15" | jq -r '.[] | "#\(.number) \(.title)"'
```

### Detailed Issue List (Table Format)

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
curl -s "https://api.github.com/repos/$REPO/issues?state=open&per_page=20" | \
  jq -r '["#", "TITLE", "LABELS", "CREATED"], ["---", "---", "---", "---"], (.[] | [.number, .title[:50], (.labels | map(.name) | join(",")), .created_at[:10]]) | @tsv'
```

## API Parameters

| Parameter | Values | Description |
|-----------|--------|-------------|
| `state` | `open`, `closed`, `all` | Filter by issue state |
| `per_page` | 1-100 | Results per page (default: 30) |
| `page` | 1+ | Page number for pagination |
| `sort` | `created`, `updated`, `comments` | Sort field |
| `direction` | `asc`, `desc` | Sort direction |
| `labels` | `bug,help wanted` | Filter by labels (comma-separated) |
| `assignee` | `username` or `none` | Filter by assignee |

## Examples

### Filter by Label

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
curl -s "https://api.github.com/repos/$REPO/issues?labels=bug&state=open" | jq -r '.[] | "#\(.number) \(.title)"'
```

### Get Single Issue Details

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
ISSUE_NUM=42
curl -s "https://api.github.com/repos/$REPO/issues/$ISSUE_NUM" | jq '{title, body, state, labels: [.labels[].name]}'
```

### Get Issue with Comments

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
ISSUE_NUM=42

# Issue details
curl -s "https://api.github.com/repos/$REPO/issues/$ISSUE_NUM"

# Comments
curl -s "https://api.github.com/repos/$REPO/issues/$ISSUE_NUM/comments" | jq '.[] | {user: .user.login, body: .body[:200]}'
```

### Recently Updated Issues

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
curl -s "https://api.github.com/repos/$REPO/issues?state=all&sort=updated&direction=desc&per_page=10" | \
  jq -r '.[] | "\(.state | if . == "open" then "🟢" else "🔴" end) #\(.number) \(.title[:40]) (updated: \(.updated_at[:10]))"'
```

## One-Liner for Current Repo

Copy-paste ready command:

```bash
curl -s "https://api.github.com/repos/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/issues?state=open&per_page=15" | jq -r '.[] | "#\(.number) \(.title)"'
```

## Rate Limiting

- **Unauthenticated**: 60 requests/hour per IP
- **Authenticated**: 5,000 requests/hour

Check your rate limit:
```bash
curl -s "https://api.github.com/rate_limit" | jq '.rate'
```

## Integration with Chakravarti

### Create Spec from Issue

```bash
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
ISSUE_NUM=42

# Fetch issue and create spec
ISSUE=$(curl -s "https://api.github.com/repos/$REPO/issues/$ISSUE_NUM")
TITLE=$(echo "$ISSUE" | jq -r '.title')
BODY=$(echo "$ISSUE" | jq -r '.body')

echo "Creating spec for: $TITLE"
ckrv spec new --name "issue-$ISSUE_NUM" --description "$BODY"
```

## References

- [GitHub REST API - Issues](https://docs.github.com/en/rest/issues/issues)
- [GitHub API Rate Limiting](https://docs.github.com/en/rest/overview/resources-in-the-rest-api#rate-limiting)
