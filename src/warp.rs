//! Warps as non-contractible paths that create holes in the topology.
//!
//! A warp is a teleportation link between two rooms that does NOT correspond
//! to any geometric adjacency. Warps introduce non-trivial topology:
//!
//! ```text
//!   Room A ──normal door──▶ Room B ──normal door──▶ Room C
//!      └──────────── warp ──────────────────────────┘
//! ```
//!
//! The warp creates a cycle that cannot be contracted to a point,
//! giving rise to non-trivial elements of π₁.

use serde::{Deserialize, Serialize};

/// A warp: a non-contractible path between rooms.
///
/// Unlike doors (which are continuous maps), warps represent discontinuous
/// jumps that create holes in the topology. Each warp contributes a
/// generator to the fundamental group π₁.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Warp {
    /// Source room id.
    pub from: String,
    /// Target room id.
    pub to: String,
    /// Human-readable label for this warp.
    pub label: String,
}

impl Warp {
    /// Create a new warp.
    pub fn new(from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: label.into(),
        }
    }
}
