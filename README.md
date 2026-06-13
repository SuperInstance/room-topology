# room-topology

**Algebraic topology for virtual spaces.**

What if every MUD room was a point in a topological space, every door was a continuous map, and every teleporter was a non-contractible path?

This library answers that question by modeling room networks as simplicial complexes and computing their algebraic-topological invariants: the **fundamental group π₁**, **simplicial homology Hₙ**, and the **universal cover**.

```
┌─────────┐    door (C⁰)    ┌─────────┐    door (C¹)    ┌─────────┐
│  Lobby  │ ──────────────▶ │  Hall   │ ──────────────▶ │ Vault   │
│  dim=2  │                 │  dim=2  │                 │  dim=1  │
└─────────┘                 └─────────┘                 └─────────┘
     ▲                                                    │
     │              ╔═══════════════╗                     │
     └──────────────║   W A R P    ║◀────────────────────┘
         teleport   ╚═══════════════╝
                    creates a HOLE in topology!
```

## The Core Idea

In a MUD (Multi-User Dungeon), rooms are connected by doors. Topologically, this is a **graph** — a 1-dimensional simplicial complex. But when you add **warps** (teleporters, portals, one-way shortcuts), something interesting happens: the navigation space develops **non-trivial topology**.

```
Normal MUD:     A ──▶ B ──▶ C       ← contractible (tree)
                                     ← π₁ is trivial
                                     ← every loop can be shrunk to a point

With warps:     A ──▶ B ──▶ C       ← NON-contractible
                 └── WARP ──┘        ← π₁ ≅ ℤ (one generator per warp)
                                     ← the warp creates a "hole"
```

### Why does this matter?

- **Loop detection**: A navigation AI that keeps looping is walking a non-trivial element of π₁
- **Path planning**: The universal cover unfolds all warps into a simply-connected space where pathfinding always works
- **World analysis**: Betti numbers tell you the "shape" of your dungeon — how many connected regions, how many teleporter loops, whether there are enclosed voids

## Topological Invariants

### Homology Hₙ (Betti Numbers)

| Betti Number | Meaning | Example |
|---|---|---|
| **H₀** | Connected components | 3 separate buildings → H₀ = 3 |
| **H₁** | Independent tunnels/loops | 2 teleporter cycles → H₁ = 2 |
| **H₂** | Enclosed voids/cavities | Always 0 for room graphs (1D complex) |

### Fundamental Group π₁

The fundamental group captures all non-contractible navigation loops:

```
One warp:     π₁ ≅ ℤ              (one generator, infinite cycles)
Two warps:    π₁ ≅ ℤ ∗ ℤ          (free group on 2 generators)
N warps:      π₁ ≅ ℤ ∗ ... ∗ ℤ    (free group on N generators)
```

Each warp contributes a generator. The **rank** of π₁ equals the cyclomatic complexity of the room graph.

### Universal Cover

The universal cover "unfolds" all non-trivial topology into a simply-connected (loop-free) space:

```
Original (one warp):        Cover (unfolded):

  ┌───┐  door  ┌───┐         ┌───┐  ┌───┐  ┌───┐  ┌───┐
  │ A │───────▶│ B │         │A₀ │─▶│B₀ │─▶│A₁ │─▶│B₁ │─▶ ...
  └───┘        └───┘         └───┘  └───┘  └───┘  └───┘
     ▲            │           infinite line — no loops!
     └── warp ────┘           navigation is always path-connected
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
room-topology = "0.1"
```

### Build a Room Complex

```rust
use room_topology::*;

// Create rooms
let entrance = Room::new("entrance")
    .with_property("danger", 2.0)
    .with_exit(Door::new("entrance", "hall"));

let hall = Room::new("hall")
    .with_exit(Door::new("hall", "entrance"))
    .with_exit(Door::new("hall", "throne"))
    .with_exit(Door::smooth("hall", "garden"));  // C¹ smooth transition

let throne = Room::new("throne")
    .with_exit(Door::new("throne", "hall"));

// Add a teleporter (warp) — creates non-trivial topology!
let portal = Warp::new("throne", "entrance", "royal_portal");

let complex = RoomComplex::from_parts(
    vec![entrance, hall, throne],
    vec![portal],
);
```

### Compute Homology

```rust
use room_topology::homology;

let hom = homology::compute_homology(&complex);

println!("Connected components: {}", hom.h0);  // 1
println!("Independent loops:    {}", hom.h1);  // 1 (the warp creates a cycle)
println!("Enclosed voids:       {}", hom.h2);  // 0 (always 0 for graphs)
println!("Euler characteristic: {}", hom.euler_characteristic());  // 0
```

### Compute the Fundamental Group

```rust
use room_topology::fundamental;

let pi1 = fundamental::compute_fundamental_group(&complex);

println!("π₁ rank:       {}", pi1.rank);         // 1
println!("Generators:    {:?}", pi1.generators);  // ["w_royal_portal"]
println!("Is trivial:    {}", pi1.is_trivial());  // false
```

### Solve the Word Problem

Is a navigation path contractible (can it be deformed to a point)?

```rust
use room_topology::fundamental;

// "abBA" = go a, go b, go back B, go back A → contractible!
assert!(fundamental::is_contractible("abBA"));

// "ab" = non-trivial element of π₁
assert!(!fundamental::is_contractible("ab"));
```

### Universal Cover

```rust
use room_topology::cover;

// Unfold the topology (depth = how many layers to generate)
let cov = cover::universal_cover(&complex, 10);

println!("Lifts generated: {}", cov.lift_count());

// Every lift projects back to an original room
for lift in &cov.lifts {
    println!("{} → {}", lift, cov.project(lift).unwrap());
}

// The cover is always simply-connected
assert!(cover::is_simply_connected(&cov));

// Navigation on the cover always finds a path
let path = cov.find_path(&cov.lifts[0], &cov.lifts[5]);
```

### Path Finding

```rust
// Find shortest path in the original complex
let path = complex.find_path("entrance", "throne").unwrap();
// → ["entrance", "hall", "throne"]

// Check connectivity
assert!(complex.is_connected("entrance", "throne"));

// Count connected components
assert_eq!(complex.connected_components(), 1);
```

## API Overview

| Type | Description |
|------|-------------|
| `Room` | A topological room: open set with boundary (exits) |
| `Door` | Continuous map between rooms (C⁰ or C¹, one-way or bidirectional) |
| `Warp` | Non-contractible path (teleporter) — creates holes in topology |
| `RoomComplex` | The full simplicial complex of rooms + doors + warps |
| `FundamentalGroup` | π₁: generators, relations, rank |
| `Homology` | H₀, H₁, H₂ Betti numbers + Euler characteristic |
| `Cover` | Universal cover: lifts, projection map, cover edges |

## Serde Support

All public types implement `Serialize` and `Deserialize`:

```rust
let json = serde_json::to_string(&complex).unwrap();
let back: RoomComplex = serde_json::from_str(&json).unwrap();
assert_eq!(complex, back);
```

## Architecture

```
src/
├── lib.rs           ← Public API & re-exports
├── room.rs          ← Room, Door types
├── door.rs          ← Door re-export
├── warp.rs          ← Warp type
├── continuity.rs    ← ContinuityClass (C⁰, C¹)
├── complex.rs       ← RoomComplex: adjacency, connectivity, cycles
├── fundamental.rs   ← π₁ computation, word problem, spanning tree
├── cover.rs         ← Universal cover construction
└── homology.rs      ← H₀, H₁, H₂ computation
```

## Examples

### Two-Building Dungeon

```
Building A:  ┌───┐───┌───┐───┌───┐
             │ A1│──▶│ A2│──▶│ A3│
             └───┘   └───┘   └───┘

Building B:  ┌───┐───┌───┐
             │ B1│──▶│ B2│
             └───┘   └───┘

Warp:        A3 ──portal──▶ B1   (connects the buildings)

H₀ = 1 (connected via warp)
H₁ = 1 (one warp-cycle: A1→A2→A3→B1→...→A1)
π₁ = ℤ (free group on 1 generator)
```

### Mega Dungeon with Multiple Warps

```
        ┌───┐   ┌───┐   ┌───┐
        │ A │──▶│ B │──▶│ C │
        └─┬─┘   └───┘   └─┬─┘
     warp│                 │warp
          │     ┌───┐      │
          └────▶│ D │◀─────┘
                └───┘

H₀ = 1, H₁ = 2 (two independent warp cycles)
π₁ = ℤ ∗ ℤ (free group on 2 generators)
Euler characteristic: χ = 4 - 5 + 1 = 0
```

## Mathematical Background

### Simplicial Complex

A room complex is a 1-dimensional simplicial complex:
- **0-simplices** (vertices): Rooms
- **1-simplices** (edges): Doors and warps

### Cyclomatic Complexity

For a connected graph with V vertices and E edges:

```
μ = E - V + 1 = rank(π₁) = H₁
```

This is the **cyclomatic complexity** — the number of independent cycles. Warps increase E without changing V, so each warp potentially adds a generator.

### Fundamental Group of a Graph

The fundamental group of a connected graph G is a **free group**:

```
π₁(G) ≅ Fᵣ  where r = |E| - |V| + 1
```

A spanning tree T has |V| - 1 edges. Each of the remaining r = |E| - |V| + 1 edges (called *chords*) contributes a generator to π₁.

### Universal Cover

The universal cover X̃ of a graph X with non-trivial π₁ is:
- **Simply connected** (π₁(X̃) = {e})
- An **infinite tree** (no cycles)
- Each vertex of X lifts to infinitely many copies in X̃
- The **projection map** p: X̃ → X is a covering map

For a graph with one cycle, the universal cover is an infinite "unrolled" line.

## License

MIT
