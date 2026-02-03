//! # Event Hub
//!
//! Pub-sub infrastructure for broadcasting orchestration events.
//!
//! ## Overview
//!
//! The `Hub` provides a broadcast channel that allows services to publish
//! events (logs, errors, step transitions) that are received by all connected
//! WebSocket clients in real-time.
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OrchestrationEvent {
    Log {
        message: String,
        timestamp: String,
        metadata: Option<serde_json::Value>,
    },
    StepStart {
        step_name: String,
        timestamp: String,
    },
    StepEnd {
        step_name: String,
        timestamp: String,
        status: String,
    },
    Error {
        message: String,
        timestamp: String,
    },
    Success {
        message: String,
        timestamp: String,
    },
}

#[derive(Clone)]
pub struct Hub {
    sender: broadcast::Sender<OrchestrationEvent>,
}

impl Hub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OrchestrationEvent> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, event: OrchestrationEvent) {
        // We ignore the error if there are no receivers
        let _ = self.sender.send(event);
    }
}

// Ensure Hub is thread-safe and shareable
pub type SharedHub = Arc<Hub>;
