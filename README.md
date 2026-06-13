# room-topology

A small Rust library that models MUD rooms as a topological space and lets
you compute its invariants: the fundamental group, Betti numbers, the
universal cover.

It's an experiment in taking a piece of game-design intuition — "this
dungeon has *holes* in it" — and rendering it in the same language a
topologist would use for surfaces. The bet is that the language is worth
borrowing even for spaces that live in 64 kB of memory.

## The basic idea

Three nouns, three types:

| Concept | What it is in a MUD | What it is here |
| --- | --- | --- |
| **Room** | A place the player can be | A connected open set, identified by `id` |
| **Door** | A passage between two places | A continuous map `Room → Room` (with a smoothness class) |
| **Warp** | A teleport, a portal, a one-way drop | A non-contractible path that adds a generator to π₁ |

A collection of rooms, doors, and warps is a [`RoomComplex`]. From it you
can read off:

- **H₀** — is the dungeon connected? Are there unreachable regions?
- **H₁** — how many "tunnels" or "loops" does the layout have?
- **π₁** — what's the fundamental group, as a presentation?
- **Euler characteristic χ** — the simplest invariant, V − E + F.

A tree of rooms (no cycles) has χ = 1 and trivial π₁. A ring of five rooms
has χ = 0 and a non-trivial π₁. A ring of five rooms with a warp skipping
one door has χ = −1 and π₁ that is provably *not* the free group on the
ring's natural generator — the warp's "twist" is detectable in the
universal cover.

## A first example

```rust
use room_topology::*;

// A triangle: three rooms, three bidirectional smooth doors.
// Doors are stored on the *target* room via `with_exit`, because a door
// is a continuous map out of the source and into the target.
let a = Room::new("A").with_exit(Door::smooth("A", "B"));
let b = Room::new("B").with_exit(Door::smooth("B", "C"));
let c = Room::new("C").with_exit(Door::smooth("C", "A"));

let complex = RoomComplex::from_parts(vec![a, b, c], vec![]);

let h = homology::compute_homology(&complex);
assert_eq!(h.h0, 1); // path-connected
assert_eq!(h.h1, 1); // one independent loop (the triangle)
assert_eq!(h.h2, 0); // no 2D void in a graph
assert_eq!(h.euler_characteristic(), 0); // χ = H₀ − H₁ + H₂
```

The triangle is the smallest complex with non-trivial H₁. It's also a
nice minimal example to start playing with — add a fourth room with two
doors to the same target and watch the cyclomatic complexity grow.

## Why those invariants?

They answer questions a MUD designer actually asks:

- **H₀ > 1** → "my players keep getting trapped in this wing I forgot to
  wire up." Look at the connected components; you'll see the isolated
  cluster.
- **H₁** → "this dungeon has *how many* loops?" The cyclomatic complexity
  E − V + C of a graph is exactly the first Betti number. It's a
  measurement of how *interesting* the navigation is, in a precise sense:
  how many ways through the space are there that aren't forced by the
  connectivity alone.
- **π₁** → "if I cover this dungeon with copies of itself, what's the
  smallest cover that makes it look like a tree?" That's the universal
  cover [`cover::Cover`], and it answers the question of *how strongly
  the loops constrain the player*.

A MUD is a small enough graph that you can compute these by hand for
sanity checking. The library is most useful when the dungeon gets large
enough that the topology is no longer obvious from a quick look.

## What's in the box

```
src/
  lib.rs           crate-level overview and re-exports
  complex.rs       the RoomComplex type — rooms + doors + warps as one
  room.rs          Room, Door (continuous map with continuity class)
  door.rs          door re-exports + extras
  warp.rs          Warp (non-contractible path)
  continuity.rs    ContinuityClass: C0 (just continuous) and C1 (smooth)
  cover.rs         universal cover construction
  fundamental.rs   π₁ as a group presentation ⟨generators | relators⟩
  homology.rs      H₀, H₁, H₂ as Betti numbers + Euler χ
```

Doors carry a [`ContinuityClass`]. A `C0` door is a passage that's merely
topological — you can go through it, but nothing is implied about the
boundary. A `C1` door is a *smooth* passage: the local charts on either
side match up to first order. For most game design purposes the class is
metadata; the interesting thing is that the type is there, and you can
filter on it (e.g. "give me all C1 doors in this wing").

## Things to try

A few experiments that fall out naturally and might be worth a half hour
of your time:

1. **Build a binary tree dungeon of depth 5.** Compute H₀, H₁, χ. Then
   add one back-edge connecting a leaf back to the root. Watch χ drop
   from 1 to 0 and π₁ acquire its first generator.
2. **Cover a ring of N rooms with the universal cover.** You'll get an
   infinite periodic graph. The period of the cover is a divisor of N
   that you can read off the index — it's the answer to "how many
   copies of the original ring tile the cover without overlap?"
3. **Take a real MUD you know.** A big public MUSH, an MMO starter zone,
   a roguelike you've played. Build a `RoomComplex` for the first 20
   rooms. Compute the invariants. Compare the χ you measure to the χ of
   a tree with the same number of nodes. The gap is the *redundancy* of
   the navigation — and it's a number you probably never had before.

## Where this comes from

The math is the standard algebraic-topology picture for graphs, which is
to say simplicial complexes in dimension 1. The friendliest entry points
are:

- Allen Hatcher's *Algebraic Topology*, Chapter 0 and §1.1. The book is
  free at [hatcher](https://pi.math.cornell.edu/~hatcher/AT/ATpage.html).
  Section 0 deals exactly with the H₀, H₁, Euler characteristic setup
  used here.
- For the MUD side, the lineage of "rooms as nodes, exits as edges" goes
  back to MUD1 (Roy Trubshaw & Richard Bartle, 1978) and is well
  described in the [Evennia docs](https://www.evennia.com/docs/) for
  anyone implementing a modern one.
- The cyclomatic complexity identity `E − V + C = β₁` is in every
  software-engineering textbook; the topological reading of it is less
  common but shows up in network science papers on road networks and
  internet topology.

## What this isn't

- It's not a MUD server. There's no network layer, no scripting, no
  persistent state. It models topology; it doesn't simulate anything
  living in it.
- It doesn't compute H₂ for non-graphs. The complex type here is 1D by
  construction (rooms are 0-cells, doors and warps are 1-cells). If you
  want genuine 2D homology, layer another crate on top that promotes
  regions of rooms to 2-cells.
- The fundamental group is reported as a free group on the cycles it
  detects — it does *not* run Tietze transformations to reduce the
  presentation. A simple 5-cycle will come back as the free group on 5
  generators rather than ℤ, because the 4 relations that would identify
  them aren't being added. The presentation is honest about the cycles
  present; it's just not reduced to the canonical form. If you need the
  reduced form, post-process `pi1.generators` and `pi1.relations`
  yourself.
- The fundamental group is computed as a free group on the cycles. The
  relator-reduction step is exact for small complexes and gets expensive
  fast; for hundreds of cycles you'll want a smarter algorithm.

## License

MIT.
