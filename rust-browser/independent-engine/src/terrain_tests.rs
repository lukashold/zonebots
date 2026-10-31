//! Small synthetic terrain fixtures; no original maps or placements embedded.
use super::*;

fn field(original: bool) -> Game {
    let mut g = Game::new(0, 1, 8523);
    let (w, h) = (32, 24);
    g.map.width = w;
    g.map.height = h;
    g.map.tiles = vec![0; w * h];
    g.map.terrain_codes = vec![0; w * h];
    g.map.territory = vec![0; w * h];
    g.map.original_tiles = vec![0; w * h];
    g.map.bridge_base = vec![-1; w * h];
    g.map.bridge_hp.clear();
    g.map.original = original;
    g.map.world = 0;
    g.units.clear();
    g.buildings.clear();
    g.sectors.clear();
    g.crates.clear();
    g.objects.clear();
    g.effects.clear();
    g.events.clear();
    g.ai_teams.clear();
    g.ai_clock = 10_000.;
    g.winner = None;
    // Both teams remain present during actual-step checks without nearby combat.
    g.spawn("jeep", 1, 944., 688., 900, None);
    let enemy = g.units.last_mut().unwrap();
    enemy.hp = 1_000_000.;
    enemy.max_hp = 1_000_000.;
    enemy.cooldown = 10_000.;
    enemy.hold_position = true;
    g
}

fn xy(x: usize, y: usize) -> (f64, f64) {
    (x as f64 * TILE + 16., y as f64 * TILE + 16.)
}

fn cell(g: &mut Game, x: usize, y: usize, tile: u8, native: u8) {
    let j = y * g.map.width + x;
    g.map.tiles[j] = tile;
    g.map.terrain_codes[j] = native;
}

fn river(g: &mut Game, x: usize, native: u8) {
    for y in 1..g.map.height - 1 {
        cell(g, x, y, 3, native);
    }
}

fn mover(g: &mut Game, kind: &str, x: usize, y: usize) -> usize {
    let p = xy(x, y);
    g.spawn(kind, 0, p.0, p.1, 1000, None);
    let unit = g.units.last_mut().unwrap();
    unit.hold_position = true;
    unit.cooldown = 10_000.;
    unit.id
}

fn unit(g: &Game, id: usize) -> &Unit {
    g.units.iter().find(|u| u.id == id).unwrap()
}

fn legal_route(g: &Game, start: (f64, f64), path: &VecDeque<(f64, f64)>, kind: &str) {
    let mut previous = start;
    for &point in path {
        assert!(g.passable_for((point.0 / TILE) as i32, (point.1 / TILE) as i32, kind));
        assert!(g.movement_segment_clear_for(previous.0, previous.1, point.0, point.1, kind));
        previous = point;
    }
}

// Check the traveled geometry rather than requiring redundant grid centers.
fn sampled_cells(start: (f64, f64), path: &VecDeque<(f64, f64)>) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut previous = start;
    for &point in path {
        let samples = (point.0 - previous.0).hypot(point.1 - previous.1).ceil().max(1.) as usize;
        for n in 0..=samples {
            let t = n as f64 / samples as f64;
            result.push((((previous.0 + (point.0 - previous.0) * t) / TILE).floor() as usize,
                         ((previous.1 + (point.1 - previous.1) * t) / TILE).floor() as usize));
        }
        previous = point;
    }
    result
}

#[test]
fn vehicle_chooses_short_road_detour_instead_of_fewest_hops() {
    let mut g = field(false);
    for x in 3..=28 {
        cell(&mut g, x, 9, 1, 7);
    }
    let (start, end) = (xy(3, 10), xy(28, 10));
    let path = g.route_for(start.0, start.1, end.0, end.1, "jeep");
    assert_eq!(path.back(), Some(&end));
    assert!(sampled_cells(start, &path).iter().filter(|p| p.1 == 9).count() >= 20 * TILE as usize,
        "A nearby faster road must beat the fewer-hop direct offroad row");
    legal_route(&g, start, &path, "jeep");
    let foot = g.route_for(start.0, start.1, end.0, end.1, "grunt");
    assert!(foot.iter().all(|p| (p.1 / TILE) as usize == 10),
        "The procedural infantry fallback has no vehicle road bonus");
}

#[test]
fn vehicle_does_not_chase_a_road_when_total_detour_is_slower() {
    let mut g = field(false);
    for x in 3..=28 {
        cell(&mut g, x, 2, 1, 7);
    }
    let (start, end) = (xy(3, 12), xy(28, 12));
    let path = g.route_for(start.0, start.1, end.0, end.1, "medium");
    assert_eq!(path.back(), Some(&end));
    assert!(path.iter().all(|p| (p.1 / TILE) as usize == 12),
        "Road preference is a travel-cost choice, not an unconditional detour");
    legal_route(&g, start, &path, "medium");
}

#[test]
fn diagonal_water_corner_and_uncentered_shore_segments_are_blocked() {
    let mut g = field(false);
    g.map.tiles.fill(2);
    cell(&mut g, 3, 3, 0, 0);
    cell(&mut g, 4, 4, 0, 0);
    cell(&mut g, 4, 3, 3, 9);
    cell(&mut g, 3, 4, 3, 9);
    let (a, b) = (xy(3, 3), xy(4, 4));
    assert!(g.route_for(a.0, a.1, b.0, b.1, "jeep").is_empty());
    assert!(!g.movement_segment_clear(a.0, a.1, b.0, b.1, false));
    assert!(!g.movement_segment_clear(127., 127., b.0, b.1, false),
        "Starting near a coastline corner must not skip its blocked side cells");
    assert!(!g.movement_segment_clear(f64::NAN, a.1, b.0, b.1, false));
}

#[test]
fn native_partial_water_uses_each_units_mobility_profile() {
    let mut g = field(true);
    let (start, end) = (xy(5, 12), xy(20, 12));
    for code in [9, 38, 39, 42] {
        river(&mut g, 12, code);
        let path = g.route_for(start.0, start.1, end.0, end.1, "grunt");
        assert!(!path.is_empty(), "Infantry should cross supported wet code {code}");
        legal_route(&g, start, &path, "grunt");
        for kind in ["jeep", "apc", "light", "medium", "heavy", "launcher", "crane", "truck", "firetruck"] {
            assert!(g.route_for(start.0, start.1, end.0, end.1, kind).is_empty(),
                "{kind} must not cross native wet code {code}");
        }
    }
    river(&mut g, 12, 40);
    assert!(g.route_for(start.0, start.1, end.0, end.1, "grunt").is_empty(),
        "Blocked wet code40 is not unrestricted swimming");
    river(&mut g, 12, 9);
    g.map.world = 1;
    assert!(g.route_for(start.0, start.1, end.0, end.1, "grunt").is_empty(),
        "The Volcan wet/lava exception must also apply to infantry commands");
}

#[test]
fn native_routes_preserve_tracked_and_wheeled_mask_difference() {
    let mut g = field(true);
    river(&mut g, 12, 1);
    let (start, end) = (xy(5, 12), xy(20, 12));
    let path = g.route_for(start.0, start.1, end.0, end.1, "medium");
    assert!(!path.is_empty());
    legal_route(&g, start, &path, "medium");
    assert!(g.route_for(start.0, start.1, end.0, end.1, "jeep").is_empty(),
        "Do not collapse every vehicle to the tracked profile");
}

#[test]
fn bridge_is_the_vehicle_crossing_and_destroyed_bridge_cannot_inherit_native_code() {
    let mut g = field(true);
    river(&mut g, 12, 9);
    cell(&mut g, 12, 8, 5, 23);
    let bridge = 8 * g.map.width + 12;
    g.map.bridge_base[bridge] = 0;
    g.map.bridge_hp.push(400.);
    let (start, end) = (xy(5, 12), xy(20, 12));
    let path = g.route_for(start.0, start.1, end.0, end.1, "jeep");
    assert!(sampled_cells(start, &path).contains(&(12, 8)), "Use the intact bridge, not a water shortcut");
    legal_route(&g, start, &path, "jeep");
    g.map.tiles[bridge] = 3;
    g.map.bridge_hp[0] = 0.;
    assert!(!g.passable_for(12, 8, "jeep"));
    assert!(!g.passable_for(12, 8, "grunt"));
    assert!(g.route_for(start.0, start.1, end.0, end.1, "jeep").is_empty(),
        "A destroyed bridge keeps its raw code23 but must be impassable");
}

#[test]
fn group_move_to_water_uses_only_reachable_safe_endpoints() {
    let mut g = field(true);
    river(&mut g, 12, 9);
    river(&mut g, 13, 9);
    let ids: Vec<_> = (0..8).map(|n| mover(&mut g, "jeep", 5 + n % 4, 8 + n / 4)).collect();
    let destination = xy(12, 12);
    g.command(&json!({"ids":ids,"kind":"move","x":destination.0,"y":destination.1}));
    for id in &ids {
        let u = unit(&g, *id);
        assert!(!u.path.is_empty());
        assert!(u.path.back().unwrap().0 < 12. * TILE,
            "Goal fallback must stay on the reachable bank after group offsets");
        legal_route(&g, (u.x, u.y), &u.path, "jeep");
    }
    for _ in 0..90 {
        g.step(0.1);
        for id in &ids {
            let u = unit(&g, *id);
            assert!(u.x < 12. * TILE);
            assert!(g.passable_for((u.x / TILE) as i32, (u.y / TILE) as i32, "jeep"));
        }
    }
}

#[test]
fn actual_vehicle_halts_at_destroyed_planned_bridge_and_can_cross_after_repair() {
    let mut g = field(true);
    river(&mut g, 12, 9);
    cell(&mut g, 12, 12, 5, 23);
    let bridge = 12 * g.map.width + 12;
    g.map.bridge_base[bridge] = 0;
    g.map.bridge_hp.push(400.);
    let id = mover(&mut g, "jeep", 10, 12);
    let end = xy(18, 12);
    g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
    assert!(sampled_cells((unit(&g, id).x, unit(&g, id).y), &unit(&g, id).path).contains(&(12, 12)));
    g.map.tiles[bridge] = 3;
    g.map.bridge_hp[0] = 0.;
    g.map.revision += 1;
    for _ in 0..100 {
        g.step(0.1);
        let u = unit(&g, id);
        assert!(u.x < 12. * TILE, "A saved pre-destruction path cannot carry the Jeep onto water");
    }
    assert!(unit(&g, id).path.is_empty());
    g.map.tiles[bridge] = 5;
    g.map.bridge_hp[0] = 400.;
    g.map.revision += 1;
    g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
    for _ in 0..200 {
        g.step(0.1);
    }
    assert!((unit(&g, id).x - end.0).abs() < 1.);
    assert!((unit(&g, id).y - end.1).abs() < 1.);
}

#[test]
fn stale_ground_paths_do_not_let_infantry_or_vehicles_cross_new_deep_water() {
    for kind in ["grunt", "jeep", "medium", "crane"] {
        let mut g = field(true);
        let id = mover(&mut g, kind, 10, 12);
        let end = xy(18, 12);
        g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
        assert!(sampled_cells((unit(&g, id).x, unit(&g, id).y), &unit(&g, id).path).contains(&(12, 12)));
        river(&mut g, 12, 40);
        g.map.revision += 1;
        for _ in 0..100 {
            g.step(0.1);
            assert!(unit(&g, id).x < 12. * TILE, "{kind} followed a stale path into deep water");
        }
        assert!(unit(&g, id).path.is_empty());
    }
}

#[test]
fn actual_uncentered_units_cannot_cut_a_new_deep_water_corner() {
    for kind in ["grunt", "jeep", "medium"] {
        let mut g = field(true);
        g.map.tiles.fill(2);
        for (x, y) in [(3, 3), (3, 4), (4, 3), (4, 4)] {
            cell(&mut g, x, y, 0, 0);
        }
        let id = mover(&mut g, kind, 3, 3);
        let i = g.units.iter().position(|u| u.id == id).unwrap();
        g.units[i].x = 127.;
        g.units[i].y = 127.;
        let end = xy(4, 4);
        g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
        assert!(!unit(&g, id).path.is_empty());
        cell(&mut g, 3, 4, 3, 40);
        cell(&mut g, 4, 3, 3, 40);
        for _ in 0..20 {
            g.step(0.1);
            let u = unit(&g, id);
            assert_eq!((u.x, u.y), (127., 127.), "{kind} must halt before the corner");
        }
        assert!(unit(&g, id).path.is_empty());
    }
}

fn first_step_distance(kind: &str, tile: u8, native: u8) -> (f64, f64) {
    let mut g = field(true);
    g.map.tiles.fill(tile);
    g.map.terrain_codes.fill(native);
    let id = mover(&mut g, kind, 5, 10);
    let start = xy(5, 10);
    let end = xy(20, 10);
    let factor = g.terrain_factor_for(5, 10, kind);
    g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
    g.step(0.05);
    let u = unit(&g, id);
    ((u.x - start.0).hypot(u.y - start.1), factor)
}

#[test]
fn actual_movement_uses_native_terrain_speed_instead_of_cosmetic_road_bias() {
    for kind in ["jeep", "medium", "grunt"] {
        let (ground, ground_factor) = first_step_distance(kind, 0, 0);
        let (road, road_factor) = first_step_distance(kind, 1, 7);
        assert!(ground > 0. && road > ground, "{kind}: native road must be faster than ground");
        assert!((road / ground - road_factor / ground_factor).abs() < 1e-9,
            "{kind}: actual movement must apply the same native speed used for route costs");
    }
    let (ground, ground_factor) = first_step_distance("grunt", 0, 0);
    let (wet, wet_factor) = first_step_distance("grunt", 3, 39);
    assert!(wet > 0. && wet < ground, "Supported infantry water is slower than dry ground");
    assert!((wet / ground - wet_factor / ground_factor).abs() < 1e-9);
}

#[test]
fn same_cell_move_reaches_the_requested_legal_point() {
    let mut g = field(true);
    let id = mover(&mut g, "grunt", 5, 10);
    let start = xy(5, 10);
    let end = (start.0 + 4., start.1 + 4.);
    g.command(&json!({"ids":[id],"kind":"move","x":end.0,"y":end.1}));
    assert_eq!(unit(&g, id).path.back(), Some(&end),
        "Different positions in one tile still require a short approach segment");
    for _ in 0..4 { g.step(0.1); }
    let u = unit(&g, id);
    assert_eq!((u.x, u.y), end);
    assert!(u.path.is_empty());
}

#[test]
fn released_robot_can_reboard_a_tile_corner_gun_without_changing_its_type() {
    for kind in ["gatling", "cannon", "howitzer", "missile"] {
        let mut g = field(true);
        g.spawn(kind, -1, 320., 320., 71, None);
        let gun = g.units.last().unwrap().id;
        g.spawn("grunt", 0, 320., 330., 72, None);
        let robot = g.units.last().unwrap().id;
        g.command(&json!({"ids":[robot],"kind":"enter","target":gun}));
        g.step(0.05);
        assert_eq!(unit(&g, gun).team, 0);
        g.command(&json!({"ids":[gun],"kind":"unload"}));
        let driver = g.units.iter().find(|u| u.team == 0 && u.kind == "grunt").unwrap();
        assert!((driver.x - 320.).hypot(driver.y - 320.) > 28.);
        assert_eq!(((driver.x / TILE) as usize, (driver.y / TILE) as usize), (10, 10));
        let driver_id = driver.id;
        g.command(&json!({"ids":[driver_id],"kind":"enter","target":gun}));
        for _ in 0..30 {
            g.step(0.05);
            if unit(&g, gun).team == 0 { break; }
        }
        let hardware = unit(&g, gun);
        assert_eq!(hardware.team, 0, "Released robot must approach and reboard {kind}");
        assert_eq!(hardware.kind, kind, "Crew changes must preserve hardware identity");
        assert_eq!(hardware.crew.as_ref().unwrap().kind, "grunt");
        assert_eq!((hardware.x, hardware.y), (320., 320.));
    }
}
