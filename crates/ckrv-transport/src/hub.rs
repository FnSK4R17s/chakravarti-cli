//! # Event Hub
//!
//! Pub-sub infrastructure for broadcasting orchestration events.
//!
//! ## Overview
//!
//! The `Hub` provides a broadcast channel that allows services to publish
//! events (logs, errors, step transitions) that are received by all connected
//! clients in real-time.
//!
//! ## Key Types
//!
//! - [`Hub`] - Broadcast channel wrapper
//! - [`OrchestrationEvent`] - Event payload variants
//! - [`SharedHub`] - Thread-safe shared reference
//!
//! ## Example
//!
//! ```rust,ignore
//! let hub = Hub::new();
//! let rx = hub.subscribe();
//!
//! hub.broadcast(OrchestrationEvent::Log {
//!     message: "Starting...".into(),
//!     timestamp: Utc::now().to_rfc3339(),
//!     metadata: None,
//! });
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Events broadcast during orchestration.
///
/// These events are sent to connected clients (WebSocket for Axum,
/// Tauri events for desktop) to provide real-time updates.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OrchestrationEvent {
    /// General log message
    Log {
        /// Log message content
        message: String,
        /// ISO 8601 timestamp
        timestamp: String,
        /// Optional structured metadata
        metadata: Option<serde_json::Value>,
    },
    /// Step execution started
    StepStart {
        /// Name of the step being started
        step_name: String,
        /// ISO 8601 timestamp
        timestamp: String,
    },
    /// Step execution completed
    StepEnd {
        /// Name of the step that completed
        step_name: String,
        /// ISO 8601 timestamp
        timestamp: String,
        /// Completion status (e.g. "success", "failed")
        status: String,
    },
    /// Error occurred
    Error {
        /// Error message
        message: String,
        /// ISO 8601 timestamp
        timestamp: String,
    },
    /// Operation completed successfully
    Success {
        /// Success message
        message: String,
        /// ISO 8601 timestamp
        timestamp: String,
    },
}

/// Broadcast hub for orchestration events.
///
/// The hub uses a bounded broadcast channel to distribute events
/// to all subscribers. If a subscriber falls behind, oldest events
/// are dropped.
#[derive(Clone)]
pub struct Hub {
    sender: broadcast::Sender<OrchestrationEvent>,
}

impl Hub {
    /// Create a new Hub with default capacity (100 events).
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new Hub with specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to receive broadcast events.
    ///
    /// Returns a receiver that will receive all future events.
    /// If the receiver falls behind, oldest events are dropped.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<OrchestrationEvent> {
        self.sender.subscribe()
    }

    /// Broadcast an event to all subscribers.
    ///
    /// If there are no active subscribers, the event is silently dropped.
    pub fn broadcast(&self, event: OrchestrationEvent) {
        // We ignore the error if there are no receivers
        let _ = self.sender.send(event);
    }

    /// Get the current number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared reference to a Hub.
pub type SharedHub = Arc<Hub>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hub_broadcast() {
        let hub = Hub::new();
        let mut rx = hub.subscribe();

        hub.broadcast(OrchestrationEvent::Log {
            message: "test".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            metadata: None,
        });

        let event = rx.recv().await.expect("should receive event");
        if let OrchestrationEvent::Log { message, .. } = event {
            assert_eq!(message, "test");
        } else {
            panic!("expected Log event");
        }
    }

    #[test]
    fn test_hub_no_receivers() {
        let hub = Hub::new();
        // Should not panic even with no receivers
        hub.broadcast(OrchestrationEvent::Success {
            message: "done".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        });
    }

    #[test]
    fn test_event_serialization() {
        let event = OrchestrationEvent::StepStart {
            step_name: "build".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).expect("serialization failed");
        assert!(json.contains("\"type\":\"stepstart\""));
    }
}
