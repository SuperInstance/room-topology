//! # room-topology
//!
//! **Algebraic topology for virtual spaces.**
//!
//! This library models MUD rooms as topological spaces where:
//! - **Rooms** are connected open sets (0-cells)
//! - **Doors** are continuous maps between rooms (1-cells)
//! - **Warps** are non-contractible paths that create holes in the topology
//!
//! The resulting structure is a simplicial complex whose algebraic-topological
//! invariants — fundamental group π₁, homology groups Hₙ, universal cover —
//! characterize the navigation space.
//!
//! ## Quick Start
//!
//! ```
//! use room_topology::*;
//!
//! // Build a simple two-room complex
//! let a = Room::new("A");
//! let b = Room::new("B").with_exit(Door::new("B", "A"));
//! let complex = RoomComplex::from_parts(vec![a, b], vec![]);
//!
//! // Compute invariants
//! let hom = homology::compute_homology(&complex);
//! assert_eq!(hom.h0, 1); // connected
//! assert_eq!(hom.h1, 0); // no cycles
//!
//! let pi1 = fundamental::compute_fundamental_group(&complex);
//! assert!(pi1.is_trivial());
//! ```
//!
//! ## Architecture
//!
//! | Module | Concept |
//! |--------|---------|
//! | [`room`] | Rooms as topological spaces, doors as continuous maps |
//! | [`door`] | Door re-exports |
//! | [`warp`] | Warps as non-contractible paths |
//! | [`fundamental`] | Fundamental group π₁ computation |
//! | [`cover`] | Universal cover construction |
//! | [`homology`] | Simplicial homology H₀, H₁, H₂ |
//! | [`continuity`] | Continuity classes C⁰, C¹ |

pub mod continuity;
pub mod cover;
pub mod door;
pub mod fundamental;
pub mod homology;
pub mod room;
pub mod warp;

mod complex;
pub use complex::RoomComplex;

pub use continuity::ContinuityClass;
pub use room::{Door, Room};
pub use warp::Warp;
pub use fundamental::FundamentalGroup;
pub use cover::Cover;
pub use homology::Homology;
