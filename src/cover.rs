//! Universal cover of the room complex.
//!
//! The universal cover X̃ is the simply-connected space that "unfolds" all
//! non-trivial topology. For a graph with warps, the universal cover is an
//! infinite tree obtained by cutting each warp-cycle and unrolling.
//!
//! ```text
//! Original:     A ──warp──▶ B ──door──▶ A  (creates a loop)
//!
//! Cover:        A₀ ──door──▶ B₀ ──door──▶ A₁ ──door──▶ B₁ ──▶ ...
//! ```
//!
/// Navigation on the cover is always path-connected and loop-free.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::complex::RoomComplex;

/// The universal cover of a room complex.
///
/// The cover is a simply-connected space obtained by unfolding all warps.
/// Each room in the original may have multiple lifts in the cover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cover {
    /// The original complex.
    pub original: RoomComplex,
    /// Lift room IDs (in the cover space).
    pub lifts: Vec<String>,
    /// Projection map: lift_id → original_id.
    pub projection: HashMap<String, String>,
    /// Edges in the cover space: (from_lift, to_lift).
    pub cover_edges: Vec<(String, String)>,
}

impl Cover {
    /// Project a lift back to the original room.
    pub fn project(&self, lift_id: &str) -> Option<&str> {
        self.projection.get(lift_id).map(|s| s.as_str())
    }

    /// Number of lifts generated.
    pub fn lift_count(&self) -> usize {
        self.lifts.len()
    }

    /// Find a path in the cover between two lifts.
    pub fn find_path<'a>(&'a self, from: &'a str, to: &'a str) -> Option<Vec<&'a str>> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for (a, b) in &self.cover_edges {
            adj.entry(a.as_str()).or_default().push(b.as_str());
            adj.entry(b.as_str()).or_default().push(a.as_str());
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut parent: HashMap<&str, &str> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(node) = queue.pop_front() {
            if node == to {
                let mut path = vec![to];
                let mut current = to;
                while let Some(&p) = parent.get(current) {
                    path.push(p);
                    current = p;
                }
                path.reverse();
                return Some(path);
            }
            if let Some(neighbors) = adj.get(node) {
                for &nb in neighbors {
                    if !visited.contains(nb) {
                        visited.insert(nb);
                        parent.insert(nb, node);
                        queue.push_back(nb);
                    }
                }
            }
        }
        None
    }
}

/// Build a finite approximation of the universal cover.
///
/// For a simply-connected complex (no warps, no door cycles), the cover
/// is the complex itself.
///
/// For a complex with one warp cycle, the cover unfolds into an infinite
/// line. We generate `depth` layers of the unfolding.
///
/// For multiple independent cycles, we'd need a tree of covers — here
/// we handle the common cases (0, 1, or multiple warp generators) by
/// generating a bounded-depth approximation.
pub fn universal_cover(complex: &RoomComplex, depth: usize) -> Cover {
    let fg = crate::fundamental::compute_fundamental_group(complex);

    // Simply connected → cover is the space itself
    if fg.is_trivial() {
        let mut lifts = Vec::new();
        let mut projection = HashMap::new();
        let mut cover_edges = Vec::new();

        for room in &complex.rooms {
            lifts.push(room.id.clone());
            projection.insert(room.id.clone(), room.id.clone());
        }

        // Add door edges
        for room in &complex.rooms {
            for door in &room.exits {
                cover_edges.push((room.id.clone(), door.to.clone()));
            }
        }

        return Cover {
            original: complex.clone(),
            lifts,
            projection,
            cover_edges,
        };
    }

    // Non-trivial: unfold by depth
    unfold_cover(complex, depth)
}

/// Unfold the complex into a cover by BFS through warp cycles.
fn unfold_cover(complex: &RoomComplex, depth: usize) -> Cover {
    let room_ids: Vec<&str> = complex.rooms.iter().map(|r| r.id.as_str()).collect();
    if room_ids.is_empty() {
        return Cover {
            original: complex.clone(),
            lifts: Vec::new(),
            projection: HashMap::new(),
            cover_edges: Vec::new(),
        };
    }

    let start = room_ids[0];
    let mut lifts: Vec<String> = Vec::new();
    let mut projection: HashMap<String, String> = HashMap::new();
    let mut cover_edges: Vec<(String, String)> = Vec::new();

    // State: (lift_id, original_id, layer)
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, String, usize)> = VecDeque::new();

    let start_lift = format!("{start}_0");
    queue.push_back((start_lift.clone(), start.to_string(), 0));
    visited.insert(start_lift.clone());

    // Track how many lifts per original room to assign indices
    let mut lift_counter: HashMap<&str, usize> = HashMap::new();
    for id in &room_ids {
        lift_counter.insert(id, 0);
    }

    while let Some((lift_id, orig_id, layer)) = queue.pop_front() {
        lifts.push(lift_id.clone());
        projection.insert(lift_id.clone(), orig_id.clone());

        if layer >= depth {
            continue;
        }

        // Find neighbors via doors
        if let Some(room) = complex.room(&orig_id) {
            for door in &room.exits {
                let target_orig = door.to.as_str();
                let counter = lift_counter.entry(target_orig).or_insert(0);
                let target_lift = format!("{target_orig}_{counter}");
                *counter += 1;

                cover_edges.push((lift_id.clone(), target_lift.clone()));

                if !visited.contains(&target_lift) {
                    visited.insert(target_lift.clone());
                    queue.push_back((target_lift, target_orig.to_string(), layer + 1));
                }
            }
        }

        // Find neighbors via warps
        for warp in &complex.warps {
            if warp.from == orig_id {
                let target_orig = warp.to.as_str();
                let counter = lift_counter.entry(target_orig).or_insert(0);
                let target_lift = format!("{target_orig}_{counter}");
                *counter += 1;

                cover_edges.push((lift_id.clone(), target_lift.clone()));

                if !visited.contains(&target_lift) {
                    visited.insert(target_lift.clone());
                    queue.push_back((target_lift, target_orig.to_string(), layer + 1));
                }
            }
        }
    }

    Cover {
        original: complex.clone(),
        lifts,
        projection,
        cover_edges,
    }
}

/// Check if navigation on the cover always finds a path.
/// In a simply-connected cover, every pair of lifts is path-connected.
pub fn is_simply_connected(cover: &Cover) -> bool {
    if cover.lifts.is_empty() {
        return true;
    }

    let mut adj: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (a, b) in &cover.cover_edges {
        adj.entry(a.as_str()).or_default().insert(b.as_str());
        adj.entry(b.as_str()).or_default().insert(a.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue = VecDeque::new();
    let start = cover.lifts[0].as_str();
    queue.push_back(start);

    while let Some(node) = queue.pop_front() {
        if visited.insert(node) {
            if let Some(neighbors) = adj.get(node) {
                for &nb in neighbors {
                    if !visited.contains(nb) {
                        queue.push_back(nb);
                    }
                }
            }
        }
    }

    visited.len() == cover.lifts.len()
}
