use crate::analysis::symbol_identity::SymbolIdentity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolEvent {
    Created {
        commit: String,
        timestamp: i64,
    },
    Renamed {
        from: String,
        to: String,
        commit: String,
    },
    Moved {
        from_file: String,
        to_file: String,
        commit: String,
    },
    SignatureChanged {
        from: String,
        to: String,
        commit: String,
    },
    Removed {
        commit: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolLineage {
    pub canonical_identity: SymbolIdentity,
    pub events: Vec<SymbolEvent>,
    pub current_identity: Option<SymbolIdentity>,
}

impl SymbolLineage {
    pub fn new(identity: SymbolIdentity) -> Self {
        Self {
            canonical_identity: identity,
            events: Vec::new(),
            current_identity: None,
        }
    }

    pub fn is_removed(&self) -> bool {
        self.events
            .iter()
            .any(|e| matches!(e, SymbolEvent::Removed { .. }))
    }

    pub fn last_commit(&self) -> Option<&str> {
        self.events.last().and_then(|e| match e {
            SymbolEvent::Created { commit, .. } => Some(commit.as_str()),
            SymbolEvent::Renamed { commit, .. } => Some(commit.as_str()),
            SymbolEvent::Moved { commit, .. } => Some(commit.as_str()),
            SymbolEvent::SignatureChanged { commit, .. } => Some(commit.as_str()),
            SymbolEvent::Removed { commit, .. } => Some(commit.as_str()),
        })
    }

    pub fn creation_commit(&self) -> Option<&str> {
        self.events.iter().find_map(|e| match e {
            SymbolEvent::Created { commit, .. } => Some(commit.as_str()),
            _ => None,
        })
    }

    pub fn removal_commit(&self) -> Option<&str> {
        self.events.iter().find_map(|e| match e {
            SymbolEvent::Removed { commit, .. } => Some(commit.as_str()),
            _ => None,
        })
    }
}
