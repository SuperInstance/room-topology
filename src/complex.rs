//! Room complex: the full topological space of rooms, doors, and warps.
//!
//! A `RoomComplex` is the simplicial complex formed by rooms (0-cells),
//! doors (1-cells), and warps (non-trivial 1-cells). It is the central
//! data structure from which we compute homotopy and homology invariants.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::room::Room;
use crate::warp::Warp;

/// The full topological space: rooms connected by doors and warps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomComplex {
    /// All rooms (0-cells of the complex).
    pub rooms: Vec<Room>,
    /// All warps (non-trivial 1-cells).
    pub warps: Vec<Warp>,
}

impl RoomComplex {
    /// Create an empty complex.
    pub fn new() -> Self {
        Self {
            rooms: Vec::new(),
            warps: Vec::new(),
        }
    }

    /// Create a complex from rooms and warps.
    pub fn from_parts(rooms: Vec<Room>, warps: Vec<Warp>) -> Self {
        Self { rooms, warps }
    }

    /// Add a room.
    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

    /// Add a warp.
    pub fn add_warp(&mut self, warp: Warp) {
        self.warps.push(warp);
    }

    /// Get a room by id.
    pub fn room(&self, id: &str) -> Option<&Room> {
        self.rooms.iter().find(|r| r.id == id)
    }

    /// All room IDs.
    pub fn room_ids(&self) -> Vec<&str> {
        self.rooms.iter().map(|r| r.id.as_str()).collect()
    }

    /// Number of rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Build the adjacency map considering both doors and warps.
    /// Returns map from room_id → set of reachable room_ids.
    pub fn adjacency(&self) -> HashMap<&str, HashSet<&str>> {
        let mut adj: HashMap<&str, HashSet<&str>> = HashMap::new();
        for room in &self.rooms {
            adj.entry(room.id.as_str()).or_default();
        }
        // Doors
        for room in &self.rooms {
            for door in &room.exits {
                adj.entry(room.id.as_str())
                    .or_default()
                    .insert(door.to.as_str());
                if door.bidirectional {
                    adj.entry(door.to.as_str())
                        .or_default()
                        .insert(room.id.as_str());
                }
            }
        }
        // Warps
        for warp in &self.warps {
            adj.entry(warp.from.as_str())
                .or_default()
                .insert(warp.to.as_str());
        }
        adj
    }

    /// Build adjacency from doors only (no warps).
    pub fn door_adjacency(&self) -> HashMap<&str, HashSet<&str>> {
        let mut adj: HashMap<&str, HashSet<&str>> = HashMap::new();
        for room in &self.rooms {
            adj.entry(room.id.as_str()).or_default();
            for door in &room.exits {
                adj.entry(room.id.as_str())
                    .or_default()
                    .insert(door.to.as_str());
                if door.bidirectional {
                    adj.entry(door.to.as_str())
                        .or_default()
                        .insert(room.id.as_str());
                }
            }
        }
        adj
    }

    /// Count connected components (weak connectivity: direction ignored).
    pub fn connected_components(&self) -> usize {
        let adj = self.adjacency();
        let all_ids: HashSet<&str> = self.rooms.iter().map(|r| r.id.as_str()).collect();
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
                    // Follow outgoing edges
                    if let Some(neighbors) = adj.get(node) {
                        for &nb in neighbors {
                            if !visited.contains(nb) {
                                queue.push_back(nb);
                            }
                        }
                    }
                    // Follow incoming edges (reverse direction)
                    for room in &self.rooms {
                        for door in &room.exits {
                            if door.to == node {
                                let src = room.id.as_str();
                                if !visited.contains(src) {
                                    queue.push_back(src);
                                }
                            }
                        }
                    }
                    for warp in &self.warps {
                        if warp.to == node {
                            if !visited.contains(warp.from.as_str()) {
                                queue.push_back(warp.from.as_str());
                            }
                        }
                    }
                }
            }
        }
        components
    }

    /// Check if two rooms are path-connected.
    pub fn is_connected(&self, from: &str, to: &str) -> bool {
        let adj = self.adjacency();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);
        while let Some(node) = queue.pop_front() {
            if node == to {
                return true;
            }
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
        false
    }

    /// Find a path between two rooms (BFS shortest path).
    pub fn find_path<'a>(&'a self, from: &'a str, to: &'a str) -> Option<Vec<&'a str>> {
        let adj = self.adjacency();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut parent: HashMap<&str, &str> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(node) = queue.pop_front() {
            if node == to {
                // Reconstruct path
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

    /// Count cycles. Returns the number of independent cycles.
    /// For a connected graph: cycles = edges - vertices + components (cyclomatic complexity).
    /// Uses multigraph counting: warps are always separate edges.
    pub fn cycle_count(&self) -> usize {
        let all_ids: HashSet<&str> = self.rooms.iter().map(|r| r.id.as_str()).collect();
        let n = all_ids.len();
        if n == 0 {
            return 0;
        }

        // Count door edges (undirected, no double-counting bidirectional)
        let mut door_edges: HashSet<(&str, &str)> = HashSet::new();
        for room in &self.rooms {
            for door in &room.exits {
                let a = room.id.as_str();
                let b = door.to.as_str();
                let key = if a <= b { (a, b) } else { (b, a) };
                door_edges.insert(key);
            }
        }

        // Each warp is a separate edge (multigraph)
        let edges = door_edges.len() + self.warps.len();
        let components = self.connected_components();

        // Cyclomatic complexity: E - V + C
        if edges + components >= n {
            edges + components - n
        } else {
            0
        }
    }
}

impl Default for RoomComplex {
    fn default() -> Self {
        Self::new()
    }
}
