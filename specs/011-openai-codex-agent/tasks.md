# Implementation Tasks: OpenAI Codex CLI Agent

**Feature**: 011-openai-codex-agent
**Created**: 2026-01-21
**Status**: In Progress

## Phase 1: Setup

- [x] T001: Create agent module structure in ckrv-sandbox
- [x] T002: Define AgentProvider trait and AgentType enum
- [x] T003: Define AgentConfig and AgentOutput structs

## Phase 2: Core Implementation

- [x] T004: Implement ClaudeProvider (extract from existing code)
- [x] T005: Implement CodexProvider
- [x] T006: Create agent factory function
- [x] T007: Update docker.rs to use AgentProvider trait

## Phase 3: Configuration

- [x] T008: Add agent environment variable support in env.rs
- [x] T009: Add --agent CLI flag to run command
- [x] T010: Propagate agent selection through execution engine

## Phase 4: Docker Integration

- [x] T011: Update Dockerfile to install Codex CLI
- [x] T012: Add Codex config mounting in docker.rs

## Phase 5: Testing & Validation

- [x] T013: Add unit tests for agent providers
- [x] T014: Test Claude backward compatibility
- [x] T015: Test Codex execution flow

## Execution Notes

- Phases must be completed in order
- Tasks within each phase can be executed sequentially
- T004 must be completed before T005 (extract pattern first)
- T007 depends on T004, T005, T006

## Completion Summary

**All 15 tasks completed successfully!**

- 14 unit tests passing
- Agent abstraction layer implemented
- CLI support for `--agent` flag added
- Docker Dockerfile updated for both agents
- Engine supports Codex execution path
