//! Doors as continuous maps between rooms.
//!
//! A door is a continuous function f: Room_A → Room_B. The orientation
//! determines whether the map has an inverse (two-way) or not (one-way).
//! The continuity class describes how smooth the transition is.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::continuity::ContinuityClass;

/// A door connecting two rooms, modeled as a continuous map.
///
/// ```text
///   Room A ──f()──▶ Room B
///          continuous map
/// ```
///
/// If `bidirectional`, there exists a continuous inverse f⁻¹: Room_B → Room_A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Door {
    /// Source room id.
    pub from: String,
    /// Target room id.
    pub to: String,
    /// Whether the door can be traversed in both directions.
    pub bidirectional: bool,
    /// Smoothness of the transition.
    pub continuity: ContinuityClass,
}

impl Door {
    /// Create a new bidirectional C⁰ door.
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            bidirectional: true,
            continuity: ContinuityClass::C0,
        }
    }

    /// Create a one-way door.
    pub fn one_way(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            bidirectional: false,
            continuity: ContinuityClass::C0,
        }
    }

    /// Create a smooth (C¹) bidirectional door.
    pub fn smooth(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            bidirectional: true,
            continuity: ContinuityClass::C1,
        }
    }

    /// Set the continuity class.
    pub fn with_continuity(mut self, cc: ContinuityClass) -> Self {
        self.continuity = cc;
        self
    }

    /// Returns the reverse door if bidirectional.
    pub fn reverse(&self) -> Option<Door> {
        if self.bidirectional {
            Some(Door {
                from: self.to.clone(),
                to: self.from.clone(),
                bidirectional: true,
                continuity: self.continuity.clone(),
            })
        } else {
            None
        }
    }
}

/// A topological room: a connected open set with boundary (doors).
///
/// The interior of a room is an open set. Doors form the boundary ∂R.
/// The dimension of a room is the number of distinct exits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    /// Unique identifier.
    pub id: String,
    /// Doors forming the boundary of this room.
    pub exits: Vec<Door>,
    /// Arbitrary numeric properties (area, lighting, danger level, etc.).
    pub properties: HashMap<String, f64>,
}

impl Room {
    /// Create a new empty room.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            exits: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a property.
    pub fn with_property(mut self, key: impl Into<String>, value: f64) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Add an exit (door).
    pub fn with_exit(mut self, door: Door) -> Self {
        self.exits.push(door);
        self
    }

    /// The dimension of this room: number of distinct exit targets.
    pub fn dimension(&self) -> usize {
        let targets: std::collections::HashSet<&str> =
            self.exits.iter().map(|d| d.to.as_str()).collect();
        targets.len()
    }

    /// Returns room IDs reachable directly from this room.
    pub fn neighbors(&self) -> Vec<&str> {
        self.exits.iter().map(|d| d.to.as_str()).collect()
    }

    /// Check if this room has a door to the given target.
    pub fn has_exit_to(&self, target: &str) -> bool {
        self.exits.iter().any(|d| d.to == target)
    }
}
