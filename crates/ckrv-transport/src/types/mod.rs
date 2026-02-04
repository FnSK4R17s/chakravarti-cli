//! # Transport Types
//!
//! Request and response types for transport handlers.
//!
//! ## Overview
//!
//! This module re-exports all type definitions used by handlers.
//! Types are organized by domain (agents, specs, execution, etc.).
//!
//! ## TypeScript Generation
//!
//! When the `typescript` feature is enabled, all types derive `TS`
//! and can be exported to TypeScript using ts-rs.

pub mod agents;
pub mod common;
pub mod execution;
pub mod history;
pub mod specs;
pub mod test_qa;

// Re-export all types for convenience
pub use agents::*;
pub use common::*;
pub use execution::*;
pub use history::*;
pub use specs::*;
pub use test_qa::*;
