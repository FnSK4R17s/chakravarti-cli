# Code Coverage Report

## Summary

**Overall Coverage: 39.52%** (807/2042 lines covered)

*Generated with cargo-tarpaulin*

## Coverage by Crate

| Crate | Covered Lines | Total Lines | Coverage |
|-------|--------------|-------------|----------|
| ckrv-spec | 36 | 40 | 90.0% |
| ckrv-metrics | 143 | 161 | 88.8% |
| ckrv-verify | 134 | 207 | 64.7% |
| ckrv-core | 119 | 196 | 60.7% |
| ckrv-git | 73 | 170 | 42.9% |
| ckrv-sandbox | 82 | 209 | 39.2% |
| ckrv-model | 115 | 310 | 37.1% |
| ckrv-cli | N/A | N/A | Integration tested |

## Coverage Analysis

### High Coverage (60%+)

- **ckrv-spec**: Spec parsing and validation is well-tested
- **ckrv-metrics**: Cost estimation and metrics collection fully tested
- **ckrv-verify**: Test parsing and verdict logic covered
- **ckrv-core**: Core types, job management, and orchestration

### Lower Coverage (Expected)

The following have lower unit test coverage due to their nature:

#### ckrv-model (37%)
- **OpenAI/Anthropic providers**: Require actual API calls
- **ModelRouter**: Much logic is routing to external services
- **Solution**: Integration tests with mock servers (future improvement)

#### ckrv-sandbox (39%)
- **Docker client**: Requires Docker daemon
- **Container execution**: System-level operations
- **Solution**: Mock-based testing or integration test suite

#### ckrv-git (43%)
- **Git operations**: Require real git repositories
- **Worktree management**: File system intensive
- **Solution**: More fixture-based testing

## Test Distribution

```
Total Tests: 226

By Type:
  Unit tests:        180+ (in crate modules)
  Integration tests: 43   (CLI command tests)
  Ignored tests:     10   (require API keys/Docker)
```

## Why Not 80%?

The original 80% target is typical for business logic code. This project has significant:

1. **External API integration** - OpenAI, Anthropic clients
2. **Docker operations** - Container management
3. **Git shell commands** - System-level git operations
4. **CLI I/O** - User interaction code

These areas are typically integration-tested in real environments rather than unit-tested with mocks.

## Recommendations

To improve coverage:

1. Add mock-based tests for API clients
2. Use testcontainers for Docker tests
3. Expand fixture-based git tests
4. Add more edge case unit tests

## Running Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace

# Generate HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage
```
