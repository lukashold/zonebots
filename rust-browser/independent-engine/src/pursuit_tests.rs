//! Command/step integration fixtures for live target pursuit. All terrain and
//! placements are synthetic; no campaign geometry or original data is embedded.
use super::*;

fn arena(original: bool) -> (Game, usize, usize) {
    let mut g = Game::new(0, 1, 9127);
    let (w, h) = (64, 48);
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
    g.objects.clear();
    g.crates.clear();
    g.effects.clear();
    g.events.clear();
    g.sectors = vec![Sector {
        id: 0, x: 64., y: 64., owner: 0, home: true,
        counts_for_production: true,
    }];
    g.ai_teams.clear();
    g.ai_clock = 10_000.;
    g.winner = None;
    g.spawn("grunt", 0, 320., 320., 1, None);
    let attacker = g.units.last().unwrap().id;
    g.spawn("apc", 1, 880., 320., 2, None);
    let target = g.units.last().unwrap().id;
    // Keep a hostile team present when target validity is changed in isolation.
    g.spawn("apc", 1, 1900., 1400., 3, None);
    for u in &mut g.units {
        u.hold_position = true;
        u.cooldown = 10_000.;
    }
    (g, attacker, target)
}

fn get(g: &Game, id: usize) -> &Unit {
    g.units.iter().find(|u| u.id == id).unwrap()
}

fn get_mut(g: &mut Game, id: usize) -> &mut Unit {
    g.units.iter_mut().find(|u| u.id == id).unwrap()
}

fn vehicle_arena(original: bool) -> (Game, usize, usize) {
    let (mut g, old, target) = arena(original);
    g.units.retain(|u| u.id != old);
    g.spawn("jeep", 0, 320., 320., 11, None);
    let u = g.units.last_mut().unwrap();
    u.cooldown = 10_000.; u.hold_position = true;
    let attacker = u.id;
    (g, attacker, target)
}

fn deep_barrier(g: &mut Game) {
    for y in 1..g.map.height - 1 {
        let j = y * g.map.width + 20;
        g.map.tiles[j] = 3; g.map.terrain_codes[j] = 40;
    }
}

fn run(g: &mut Game, steps: usize) {
    for _ in 0..steps { g.step(0.1); }
}

fn attack(g: &mut Game, attacker: usize, target: usize) {
    g.command(&json!({"ids":[attacker],"kind":"attack","target":target}));
}

fn move_target(g: &mut Game, target: usize, x: f64, y: f64) {
    g.command(&json!({"team":1,"ids":[target],"kind":"move","x":x,"y":y}));
}

fn last_point(g: &Game, id: usize) -> (f64, f64) {
    *get(g, id).path.back().expect("An out-of-range target needs an active route")
}

fn current_destination(g: &Game, attacker: usize, target: usize) {
    let a = get(g, attacker);
    let t = get(g, target);
    let dest = a.dest.expect("Live unit attacks expose the current target position");
    assert!((dest.0 - t.x).hypot(dest.1 - t.y) < TILE,
        "The destination may lag one simulation snapshot, not an old target position");
}

#[test]
fn live_target_updates_an_attack_route_before_the_old_path_finishes() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        attack(&mut g, attacker, target);
        g.step(0.05);
        let initial = last_point(&g, attacker);
        assert!(get(&g, attacker).path.len() > 3);
        move_target(&mut g, target, 880., 1000.);
        run(&mut g, 40);
        let a = get(&g, attacker);
        let t = get(&g, target);
        let updated = last_point(&g, attacker);
        assert!(updated.1 > initial.1 + TILE,
            "Attack must replan while the previous route still has nodes");
        assert!((updated.0 - t.x).hypot(updated.1 - t.y) <= stats(&a.kind).range + TILE,
            "The refreshed route ends at a firing position for the current target");
        current_destination(&g, attacker, target);
        assert_eq!(a.order, "attack");
        assert_eq!(a.target, Some(target));
        assert!(a.moving);
    }
}

#[test]
fn chase_follows_another_change_of_target_direction() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        attack(&mut g, attacker, target);
        move_target(&mut g, target, 880., 1000.);
        run(&mut g, 20);
        let north = last_point(&g, attacker);
        let y = get(&g, target).y;
        move_target(&mut g, target, 1450., y);
        run(&mut g, 30);
        let east = last_point(&g, attacker);
        assert!(east.0 > north.0 + TILE, "A second turn must replace the stale northward goal");
        let t = get(&g, target);
        assert!((east.0 - t.x).hypot(east.1 - t.y) <= stats("grunt").range + TILE);
        current_destination(&g, attacker, target);
        assert_eq!(get(&g, attacker).target, Some(target));
    }
}

#[test]
fn each_member_of_a_selected_group_keeps_tracking_the_same_live_target() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        let mut ids = vec![attacker];
        for (x, y) in [(320., 352.), (352., 320.), (352., 352.)] {
            g.spawn("grunt", 0, x, y, 1, None);
            let u = g.units.last_mut().unwrap();
            u.cooldown = 10_000.;
            ids.push(u.id);
        }
        g.command(&json!({"ids":ids,"kind":"attack","target":target}));
        g.step(0.05);
        let initial: Vec<_> = ids.iter().map(|id| last_point(&g, *id)).collect();
        move_target(&mut g, target, 880., 1000.);
        run(&mut g, 40);
        for (id, before) in ids.iter().zip(initial) {
            let u = get(&g, *id);
            assert_eq!(u.order, "attack");
            assert_eq!(u.target, Some(target));
            assert!(last_point(&g, *id).1 > before.1 + TILE,
                "Every selected attacker needs its own refreshed path");
            current_destination(&g, *id, target);
        }
    }
}

#[test]
fn reachable_target_in_weapon_range_stops_the_chase_and_receives_fire() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        get_mut(&mut g, target).x = 540.; //220 inside the Grunt's240 range.
        get_mut(&mut g, attacker).cooldown = 0.;
        attack(&mut g, attacker, target);
        g.step(0.05);
        let a = get(&g, attacker);
        assert_eq!((a.x, a.y), (320., 320.), "A valid firing position must not keep walking");
        assert!(a.path.is_empty());
        assert!(!a.moving);
        assert_eq!(a.order, "attack");
        assert_eq!(a.target, Some(target));
        assert!(g.events.iter().any(|e| e["kind"] == "shot" && e["team"] == 0));
        assert!(g.effects.iter().any(|e| e.kind == "shot" && e.x2 == 540. && e.y2 == 320.));
    }
}

#[test]
fn chase_resumes_when_a_previously_reachable_target_leaves_weapon_range() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        get_mut(&mut g, target).x = 500.;
        attack(&mut g, attacker, target);
        g.step(0.05);
        assert!(get(&g, attacker).path.is_empty());
        move_target(&mut g, target, 1100., 320.);
        run(&mut g, 25);
        let a = get(&g, attacker);
        assert!(a.x > 320., "Holding to fire must not lose the explicit attack order");
        assert!(a.moving);
        assert!(!a.path.is_empty());
        assert_eq!(a.target, Some(target));
    }
}

#[test]
fn attack_in_range_with_blocked_sight_keeps_seeking_a_valid_firing_position() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        get_mut(&mut g, target).x = 500.;
        for y in 8..=12 {
            let j = y * g.map.width + 13;
            g.map.tiles[j] = 2;
            g.map.terrain_codes[j] = 19;
        }
        get_mut(&mut g, attacker).cooldown = 0.;
        attack(&mut g, attacker, target);
        g.step(0.05);
        let a = get(&g, attacker);
        assert!(!a.path.is_empty(), "Distance alone must not stop a chase behind an obstacle");
        assert!(a.moving);
        assert!(!g.events.iter().any(|e| e["kind"] == "shot" && e["team"] == 0));
    }
}

#[test]
fn explicit_stop_and_move_override_pursuit_without_later_reactivation() {
    for original in [false, true] {
        for stop in [false, true] {
            let (mut g, attacker, target) = arena(original);
            attack(&mut g, attacker, target);
            g.step(0.05);
            let before = (get(&g, attacker).x, get(&g, attacker).y);
            if stop {
                g.command(&json!({"ids":[attacker],"kind":"stop"}));
            } else {
                g.command(&json!({"ids":[attacker],"kind":"move","x":before.0,"y":1000}));
            }
            move_target(&mut g, target, 880., 1000.);
            run(&mut g, 15);
            let a = get(&g, attacker);
            assert_eq!(a.target, None);
            if stop {
                assert_eq!(a.order, "guard");
                assert_eq!((a.x, a.y), before);
                assert!(a.path.is_empty());
            } else {
                assert_eq!(a.order, "move");
                assert!(a.y > before.1);
                assert!(last_point(&g, attacker).0 < 400., "A new move goal must not snap back to the enemy");
            }
        }
    }
}

#[test]
fn invalid_explicit_targets_cancel_the_old_path_and_target_reference() {
    for original in [false, true] {
        for change in ["dead", "missing", "friendly", "neutral"] {
            let (mut g, attacker, target) = arena(original);
            attack(&mut g, attacker, target);
            g.step(0.05);
            assert!(!get(&g, attacker).path.is_empty());
            let before = (get(&g, attacker).x, get(&g, attacker).y);
            match change {
                "dead" => get_mut(&mut g, target).hp = -1.,
                "missing" => g.units.retain(|u| u.id != target),
                "friendly" => get_mut(&mut g, target).team = 0,
                "neutral" => { let t = get_mut(&mut g, target); t.team = -1; t.crew = None; },
                _ => unreachable!(),
            }
            run(&mut g, 5);
            let a = get(&g, attacker);
            assert_eq!(a.order, "guard", "{change} target must release its attack order");
            assert_eq!(a.target, None, "{change}");
            assert!(a.path.is_empty(), "{change}");
            assert_eq!((a.x, a.y), before, "{change}: do not walk to a dead target's old position");
        }
    }
}

#[test]
fn ai_does_not_replace_an_explicit_attack_while_its_path_is_empty_to_fire() {
    for original in [false, true] {
        let (mut g, attacker, target) = arena(original);
        get_mut(&mut g, target).x = 500.;
        g.sectors.push(Sector {
            id: 1, x: 320., y: 1000., owner: -1, home: false,
            counts_for_production: true,
        });
        g.ai_teams = vec![0];
        g.ai_clock = 0.;
        attack(&mut g, attacker, target);
        g.step(0.05);
        let a = get(&g, attacker);
        assert_eq!(a.order, "attack", "An empty firing path is not an idle unit");
        assert_eq!(a.target, Some(target));
        assert_eq!((a.x, a.y), (320., 320.));
        assert!(a.path.is_empty());
    }
}

#[test]
fn vehicle_pursuit_uses_a_bridge_and_never_follows_a_stale_path_into_water() {
    for original in [false, true] {
        let (mut g, attacker, target) = vehicle_arena(original);
        deep_barrier(&mut g);
        let bridge = 15 * g.map.width + 20;
        g.map.tiles[bridge] = 5; g.map.terrain_codes[bridge] = 23;
        g.map.bridge_base[bridge] = 0; g.map.bridge_hp.push(400.);
        attack(&mut g, attacker, target);
        g.step(0.05);
        assert!(get(&g, attacker).path.contains(&(656., 496.)));
        move_target(&mut g, target, 880., 1000.);
        run(&mut g, 10);
        assert!(get(&g, attacker).path.contains(&(656., 496.)),
            "Updated pursuit must keep using the only legal crossing");
        g.map.tiles[bridge] = 3; g.map.bridge_hp[0] = 0.; g.map.revision += 1;
        for _ in 0..40 {
            g.step(0.1);
            let a = get(&g, attacker);
            assert!(a.x < 640., "Tracking must not bypass vehicle water restrictions");
            assert_eq!(a.target, Some(target));
        }
        assert!(get(&g, attacker).path.is_empty());
        g.map.tiles[bridge] = 5; g.map.bridge_hp[0] = 400.; g.map.revision += 1;
        run(&mut g, 10);
        assert!(!get(&g, attacker).path.is_empty(), "Repair must restore pursuit without another attack click");
        assert!(get(&g, attacker).path.contains(&(656., 496.)));
    }
}

#[test]
fn target_across_water_can_be_attacked_from_a_reachable_same_bank_firing_position() {
    for original in [false, true] {
        let (mut g, attacker, target) = vehicle_arena(original);
        deep_barrier(&mut g);
        get_mut(&mut g, target).x = 720.;
        get_mut(&mut g, attacker).cooldown = 0.;
        attack(&mut g, attacker, target);
        for _ in 0..100 {
            g.step(0.1);
            assert!(get(&g, attacker).x < 640., "A firing route cannot send the Jeep into water");
        }
        let a = get(&g, attacker);
        let t = get(&g, target);
        assert!(a.x > 320., "The bank has a reachable firing position, though the enemy tile is unreachable");
        assert!((a.x - t.x).hypot(a.y - t.y) <= stats("jeep").range);
        assert!(a.path.is_empty());
        assert!(!a.moving);
        assert_eq!(a.order, "attack");
        assert_eq!(a.target, Some(target));
        assert!(g.events.iter().any(|e| e["kind"] == "shot" && e["team"] == 0));
    }
}

#[test]
fn fully_unreachable_live_target_stays_selected_and_retries_after_a_map_change() {
    for original in [false, true] {
        let (mut g, attacker, target) = vehicle_arena(original);
        deep_barrier(&mut g);
        attack(&mut g, attacker, target);
        move_target(&mut g, target, 880., 900.);
        for _ in 0..20 {
            g.step(0.1);
            let a = get(&g, attacker);
            assert_eq!(a.order, "attack", "No route is different from an invalid enemy target");
            assert_eq!(a.target, Some(target));
            assert!(a.path.is_empty());
            assert_eq!((a.x, a.y), (320., 320.));
            current_destination(&g, attacker, target);
        }
        let bridge = 10 * g.map.width + 20;
        g.map.tiles[bridge] = 5; g.map.terrain_codes[bridge] = 23;
        g.map.bridge_base[bridge] = 0; g.map.bridge_hp.push(400.);
        g.map.revision += 1;
        g.step(0.05);
        let a = get(&g, attacker);
        assert_eq!(a.target, Some(target));
        assert!(!a.path.is_empty(), "A new crossing should refresh an unreachable attack without another click");
        assert!(a.moving);
    }
}
