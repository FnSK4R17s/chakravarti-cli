use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use std::sync::Arc;

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
