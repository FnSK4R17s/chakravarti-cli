//! # Events Handler
//!
//! Handlers for real-time event streaming.
//!
//! Note: This handler has transport-specific implementations.
//! - Axum: Server-Sent Events (SSE)
//! - Tauri: Native event emission
//!
//! Placeholder - full implementation will migrate from ckrv-ui.

use crate::hub::{OrchestrationEvent, SharedHub};

/// Subscribe to events.
///
/// Returns a receiver that will receive all future events.
pub fn subscribe_events(hub: &SharedHub) -> tokio::sync::broadcast::Receiver<OrchestrationEvent> {
    hub.subscribe()
}

/// Emit an event.
pub fn emit_event(hub: &SharedHub, event: OrchestrationEvent) {
    hub.broadcast(event);
}
