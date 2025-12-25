# Chakravarti CLI

AI-powered code generation CLI with spec-driven development.

## Installation

```bash
npm install -g @chakravarti/cli
```

Or with npx (no install):

```bash
npx @chakravarti/cli --help
```

## Quick Start

```bash
# Initialize a repository
ckrv init

# Create a spec file
cat > .specs/my-feature.yaml << 'EOF'
id: my_feature
goal: Add a hello world function

constraints:
  - Use idiomatic code
  - Include tests

acceptance:
  - Function exists and works
EOF

# Run the spec
ckrv run .specs/my-feature.yaml

# View changes
ckrv diff <job-id>

# Promote to branch
ckrv promote <job-id> --branch feature/my-feature
```

## Environment Variables

```bash
# Required: At least one API key
export OPENAI_API_KEY="sk-..."
# or
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Commands

| Command | Description |
|---------|-------------|
| `ckrv init` | Initialize repository |
| `ckrv spec validate <file>` | Validate spec file |
| `ckrv run <spec>` | Execute spec with AI |
| `ckrv status <job>` | Check job status |
| `ckrv diff <job>` | View changes |
| `ckrv promote <job> --branch <name>` | Promote to branch |

## Features

- 🤖 **AI Code Generation** - OpenAI & Anthropic support
- 🔒 **Isolated Execution** - Git worktree isolation
- 🐳 **Docker Verification** - Run tests in containers
- 📊 **Metrics Tracking** - Token usage & cost
- ⚡ **Optimization Modes** - Cost, time, or balanced

## Documentation

- [Full Documentation](https://github.com/chakravarti/cli#readme)
- [Contributing Guide](https://github.com/chakravarti/cli/blob/main/CONTRIBUTING.md)

## License

MIT
