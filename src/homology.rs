//! Simplicial homology of the room complex.
//!
//! We compute H₀, H₁, H₂ (Betti numbers) for the room complex viewed
//! as a simplicial complex:
//!
//! - **H₀**: Number of connected components. H₀ = 1 means the space is
//!   path-connected.
//! - **H₁**: Number of independent tunnels/loops. Each warp or door-cycle
//!   that isn't filled contributes to H₁. For a graph, H₁ = E - V + C
//!   (cyclomatic complexity).
//! - **H₂**: Number of voids/cavities. For a 1-dimensional complex (graph),
//!   H₂ = 0 always — there are no 2-dimensional holes.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::complex::RoomComplex;

/// Homology groups H₀, H₁, H₂ as Betti numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Homology {
    /// H₀: connected components.
    pub h0: usize,
    /// H₁: independent tunnels/warps.
    pub h1: usize,
    /// H₂: voids (always 0 for graphs).
    pub h2: usize,
}

impl Homology {
    /// Zero homology (empty space).
    pub fn zero() -> Self {
        Self { h0: 0, h1: 0, h2: 0 }
    }

    /// The Euler characteristic: χ = H₀ - H₁ + H₂ = V - E + F.
    pub fn euler_characteristic(&self) -> isize {
        self.h0 as isize - self.h1 as isize + self.h2 as isize
    }
}

impl std::fmt::Display for Homology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "H₀={}, H₁={}, H₂={}", self.h0, self.h1, self.h2)
    }
}

/// Compute simplicial homology of the room complex.
///
/// For a 1-dimensional simplicial complex (a graph):
/// - H₀ = number of connected components
/// - H₁ = cyclomatic complexity = E - V + C
/// - H₂ = 0 (no 2-simplices)
pub fn compute_homology(complex: &RoomComplex) -> Homology {
    let v = complex.rooms.len();
    if v == 0 {
        return Homology::zero();
    }

    let components = complex.connected_components();
    let h0 = components;

    // Count edges (doors + warps), multigraph
    let e = count_edges(complex);
    let h1 = if e + components >= v {
        e + components - v
    } else {
        0
    };

    Homology {
        h0,
        h1,
        h2: 0, // Graphs have no 2-dimensional holes
    }
}

/// Count edges as a multigraph: each door and each warp is a separate edge,
/// even if they connect the same pair of rooms.
fn count_edges(complex: &RoomComplex) -> usize {
    let mut count = 0usize;

    // Count door edges (avoid double-counting bidirectional)
    let mut door_edges: HashSet<(&str, &str)> = HashSet::new();
    for room in &complex.rooms {
        for door in &room.exits {
            let a = room.id.as_str();
            let b = door.to.as_str();
            let key = if a <= b { (a, b) } else { (b, a) };
            if !door_edges.contains(&key) {
                door_edges.insert(key);
                count += 1;
            }
        }
    }

    // Each warp is a separate edge (multigraph)
    count += complex.warps.len();

    count
}
