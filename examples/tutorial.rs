//! Tutorial: room-topology — MUD rooms as topological spaces
//!
//! Doors are continuous maps, warps create non-trivial π₁.
//! Simplicial homology, universal covers, fundamental groups.

use room_topology::{
    Room, Door, Warp, RoomComplex,
    homology::compute_homology,
    fundamental::{compute_fundamental_group, spanning_tree, reduce_word, is_contractible},
    cover::{universal_cover, is_simply_connected},
};

fn main() {
    println!("=== Room Topology Tutorial ===\n");

    // Part 1: Build rooms with doors
    println!("Part 1: Rooms and doors (continuous maps)");
    let mut entrance = Room::new("entrance");
    let mut hallway = Room::new("hallway");
    let mut cellar = Room::new("cellar");
    let mut tower = Room::new("tower");

    entrance.exits.push(Door::new("entrance", "hallway"));
    entrance.exits.push(Door::new("entrance", "cellar"));
    hallway.exits.push(Door::new("hallway", "entrance"));
    hallway.exits.push(Door::new("hallway", "tower"));
    cellar.exits.push(Door::new("cellar", "entrance"));
    tower.exits.push(Door::new("tower", "hallway"));
    tower.exits.push(Door::new("tower", "entrance"));
    println!("  4 rooms with doors");
    println!();

    // Part 2: Warps — non-trivial topology
    println!("Part 2: Warps (non-trivial 1-cells)");
    let portal = Warp::new("cellar", "tower", "magic_portal");
    println!("  Warp: {} → {} ({})", portal.from, portal.to, portal.label);
    println!();

    // Part 3: Build complex
    println!("Part 3: Room complex");
    let mut complex = RoomComplex::new();
    complex.add_room(entrance);
    complex.add_room(hallway);
    complex.add_room(cellar);
    complex.add_room(tower);
    complex.add_warp(portal);
    println!("  {} rooms, {} warps", complex.room_count(), complex.warps.len());
    println!("  Connected components: {}", complex.connected_components());
    println!("  Is entrance→tower: {}", complex.is_connected("entrance", "tower"));
    println!();

    // Part 4: Homology
    println!("Part 4: Simplicial homology");
    let h = compute_homology(&complex);
    println!("  Euler characteristic: {}", h.euler_characteristic());
    println!();

    // Part 5: Fundamental group
    println!("Part 5: Fundamental group π₁");
    let fg = compute_fundamental_group(&complex);
    println!("  Is trivial: {}", fg.is_trivial());
    let tree = spanning_tree(&complex);
    println!("  Spanning tree edges: {}", tree.len());
    println!();

    // Part 6: Word reduction
    println!("Part 6: Path word reduction");
    let reduced = reduce_word("abcABC");
    println!("  'abcABC' → '{}' (contractible: {})", reduced, is_contractible("abcABC"));
    println!();

    // Part 7: Universal cover
    println!("Part 7: Universal cover");
    let cover = universal_cover(&complex, 3);
    println!("  Lift count: {}", cover.lift_count());
    println!("  Is simply connected: {}", is_simply_connected(&cover));
}
