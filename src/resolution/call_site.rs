// src/resolution/call_site.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallSite {
    pub kind: CallKind,
    pub callee: CalleeExpr,
    pub location: SourceLocation,
    pub receiver_type: Option<String>,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallKind {
    Function,
    Method,
    Constructor,
    Operator,
    Closure,
    Callback,
    Macro,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalleeExpr {
    Name(String),
    Qualified(Vec<String>),
    Member {
        receiver: Box<CalleeExpr>,
        member: String,
    },
    Index {
        object: Box<CalleeExpr>,
    },
    Call {
        function: Box<CalleeExpr>,
    },
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl CalleeExpr {
    pub fn root_name(&self) -> Option<&str> {
        match self {
            CalleeExpr::Name(n) => Some(n),
            CalleeExpr::Qualified(parts) => parts.first().map(|s| s.as_str()),
            CalleeExpr::Member { receiver, .. } => receiver.root_name(),
            CalleeExpr::Call { function } => function.root_name(),
            CalleeExpr::Index { object } => object.root_name(),
            CalleeExpr::Unknown(s) => Some(s),
        }
    }

    pub fn last_name(&self) -> Option<&str> {
        match self {
            CalleeExpr::Name(n) => Some(n),
            CalleeExpr::Qualified(parts) => parts.last().map(|s| s.as_str()),
            CalleeExpr::Member { member, .. } => Some(member),
            CalleeExpr::Call { function } => function.last_name(),
            CalleeExpr::Index { object } => object.last_name(),
            CalleeExpr::Unknown(s) => Some(s),
        }
    }
}
