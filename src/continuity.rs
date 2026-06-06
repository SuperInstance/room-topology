//! Continuous map classes for door transitions.

use serde::{Deserialize, Serialize};

/// Continuity class of a door's transition map.
///
/// - `C0`: The door provides a continuous (but not necessarily smooth) map
///   between rooms. Think of stepping through a doorway — you arrive, but
///   there may be a jarring change in environment.
/// - `C1`: The door provides a continuously differentiable map. The transition
///   is smooth — gradual lighting changes, consistent temperature, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContinuityClass {
    C0,
    C1,
}

impl Default for ContinuityClass {
    fn default() -> Self {
        Self::C0
    }
}

impl std::fmt::Display for ContinuityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContinuityClass::C0 => write!(f, "C⁰"),
            ContinuityClass::C1 => write!(f, "C¹"),
        }
    }
}
