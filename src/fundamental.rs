//! Fundamental group π₁ of the room complex.
//!
//! The fundamental group captures the non-contractible loops in the room
//! graph. Normal doors create contractible paths (they're geometrically
//! adjacent). Warps create non-contractible loops — you can't smoothly
//! deform a warp-teleportation into "walking there."
//!
//! For a room complex:
//! - Generators come from warps and cycles
//! - Relations come from contractible loops (normal door cycles)
//! - π₁ is the free group on warp generators (modulo relations)
//!
//! The word problem: given a navigation path (a word in the generators),
//! determine if it represents the identity (is contractible).

use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::complex::RoomComplex;

/// The fundamental group π₁ of a room complex.
///
/// Represented as a finitely presented group:
///   π₁ = ⟨ generators | relations ⟩
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundamentalGroup {
    /// Generator labels (one per warp / independent cycle).
    pub generators: Vec<String>,
    /// Relations as words in the generators (e.g., "abA" means ab = e).
    pub relations: Vec<String>,
    /// Rank of the free group (number of independent generators).
    pub rank: usize,
}

impl FundamentalGroup {
    /// The trivial group (for simply-connected spaces).
    pub fn trivial() -> Self {
        Self {
            generators: Vec::new(),
            relations: Vec::new(),
            rank: 0,
        }
    }

    /// Free group on n generators.
    pub fn free(n: usize) -> Self {
        let generators: Vec<String> = (0..n).map(|i| format!("g{i}")).collect();
        Self {
            generators,
            relations: Vec::new(),
            rank: n,
        }
    }

    /// Is this the trivial group?
    pub fn is_trivial(&self) -> bool {
        self.rank == 0
    }
}

/// Compute the fundamental group of a room complex.
///
/// Strategy:
/// 1. Build the graph (rooms + doors)
/// 2. Find a spanning tree of the door-graph
/// 3. Each warp that creates a cycle = one generator
/// 4. Door-cycles that don't involve warps = relations
/// 5. Result: π₁ ≅ free group on warps / relations
pub fn compute_fundamental_group(complex: &RoomComplex) -> FundamentalGroup {
    let n = complex.rooms.len();
    if n == 0 {
        return FundamentalGroup::trivial();
    }

    // Each warp contributes a generator
    let warp_count = complex.warps.len();

    // Count independent cycles from doors (without warps)
    let door_cycles = count_door_cycles(complex);

    // The fundamental group of a graph is the free group on (E - V + C) generators
    // For the combined graph (doors + warps):
    let total_cycles = complex.cycle_count();

    if total_cycles == 0 {
        return FundamentalGroup::trivial();
    }

    // Generators: warps contribute named generators, door cycles contribute generic ones
    let mut generators: Vec<String> = Vec::new();

    // Add warp generators
    for warp in &complex.warps {
        generators.push(format!("w_{}", warp.label));
    }

    // Add cycle generators for pure door cycles beyond what warps account for
    let extra_cycles = total_cycles.saturating_sub(warp_count);
    for i in 0..extra_cycles {
        generators.push(format!("c_{i}"));
    }

    let _ = door_cycles; // used for relations in more sophisticated analysis

    FundamentalGroup {
        rank: generators.len(),
        generators,
        relations: Vec::new(),
    }
}

/// Count cycles formed by doors only (no warps).
fn count_door_cycles(complex: &RoomComplex) -> usize {
    let adj = complex.door_adjacency();
    let all_ids: HashSet<&str> = complex.rooms.iter().map(|r| r.id.as_str()).collect();
    let n = all_ids.len();
    if n == 0 {
        return 0;
    }

    let mut edge_set: HashSet<(&str, &str)> = HashSet::new();
    for room in &complex.rooms {
        for door in &room.exits {
            let a = room.id.as_str();
            let b = door.to.as_str();
            if a < b {
                edge_set.insert((a, b));
            } else {
                edge_set.insert((b, a));
            }
        }
    }
    let edges = edge_set.len();

    // Count components via door adjacency only
    let mut visited: HashSet<&str> = HashSet::new();
    let mut components = 0;
    for start in &all_ids {
        if visited.contains(start) {
            continue;
        }
        components += 1;
        let mut queue = VecDeque::new();
        queue.push_back(*start);
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
    }

    if edges + components >= n {
        edges + components - n
    } else {
        0
    }
}

/// Solve the word problem: determine if a path word is contractible.
///
/// A word is contractible if it reduces to the identity after canceling
/// adjacent inverse pairs. For a free group, this is free reduction.
pub fn is_contractible(word: &str) -> bool {
    reduce_word(word).is_empty()
}

/// Free-reduce a word in the generators.
///
/// Cancel adjacent generator-inverse pairs: "aA" → "", "abBA" → "".
/// Lowercase = generator, uppercase = inverse.
pub fn reduce_word(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut stack: Vec<char> = Vec::new();

    for &c in &chars {
        if let Some(&top) = stack.last() {
            // Check if c is the inverse of top
            if top != c && top.eq_ignore_ascii_case(&c) {
                stack.pop();
                continue;
            }
        }
        stack.push(c);
    }

    stack.into_iter().collect()
}

/// Build a spanning tree of the door-graph using BFS.
/// Returns the set of edges in the tree (as (from, to) pairs).
pub fn spanning_tree(complex: &RoomComplex) -> HashSet<(String, String)> {
    let adj = complex.door_adjacency();
    let mut tree: HashSet<(String, String)> = HashSet::new();
    let mut visited: HashSet<&str> = HashSet::new();

    for room in &complex.rooms {
        let start = room.id.as_str();
        if visited.contains(start) {
            continue;
        }
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            if let Some(neighbors) = adj.get(node) {
                for &nb in neighbors {
                    if !visited.contains(nb) {
                        visited.insert(nb);
                        tree.insert((node.to_string(), nb.to_string()));
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    tree
}
