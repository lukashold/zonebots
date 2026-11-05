use super::*;

fn field(kind: &str) -> Game {
    let mut g = Game::new(0, 1, 0x5431);
    g.map.original = true;
    g.map.width = 128;
    g.map.height = 128;
    g.map.tiles = vec![0; 16384];
    g.map.terrain_codes = vec![0; 16384];
    g.map.original_tiles = vec![0; 16384];
    g.map.native_terrain_plane = vec![0; 16384];
    g.map.native_tile_plane = vec![0; 16384];
    g.map.bridge_base = vec![-1; 16384];
    g.map.native_road_graph = Some(crate::original_route::RoadGraph {
        regions: (0..96).map(|_| crate::original_route::RoadRegion {
            bounds: [0; 4], waypoint: 0, next: vec![0; 96],
        }).collect(),
    });
    g.units.clear();
    g.buildings.clear();
    g.objects.clear();
    g.crates.clear();
    g.walls.clear();
    g.sectors.clear();
    g.native_world = None;
    g.native_group_rosters.clear();
    g.ai_teams.clear();
    g.import_native_world(&json!({"nativeWorldBuildings":[{
        "nativeSlot":0,"kind":4,"x":320,"y":320,"hp":0,
        "maxHp":30000,"buildingId":7
    }]}));
    g.spawn(kind, 0, 640., 640., 1, None);
    g.maintain_native_group_rosters();
    g.native_group_rosters.observe_unit_context(g.units[0].native_slot.unwrap());
    g.prepare_native_ambient_queries();
    g
}

#[test]
fn unused_work_frames_do_not_prevent_the_reached_mouth_and_common_prefix() {
    let mut g = field("crane");
    g.multiplayer = true; // Matches the controlled Source72 no-CRT fixture.
    let slot = g.units[0].native_slot.unwrap();
    assert!(g.native_world.as_ref().unwrap().crane_frames.is_empty());
    let before = g.native_cranes.slots[slot].as_ref().unwrap().field96;
    assert_eq!(before.value, 0);
    assert!(g.update_native_crane_building(0, 0, 1));
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().field96.value, 1);
    assert_eq!(g.native_world.as_ref().unwrap().trail.boundary, None);
    assert_eq!(g.native_route_boundary, None);
}

fn bind_work(g: &mut Game, work: u8) -> usize {
    let slot = g.units[0].native_slot.unwrap();
    let c = g.native_cranes.slots[slot].as_mut().unwrap();
    c.target = Some(0);
    c.work9a = work;
    c.frame86 = -1;
    let s = g.native_world.as_mut().unwrap();
    s.trail.known_builds[0] = true;
    raw_set(&mut s.trail.builds[0], 28, 30000);
    raw_set(&mut s.trail.builds[0], 30, 0);
    raw_set(&mut s.trail.builds[0], 60, 0);
    slot
}

#[test]
fn generic_refresh_and_unbound_main_context_never_create_or_rebind_auxiliaries() {
    let mut foot = field("grunt");
    let slot = foot.units[0].native_slot.unwrap();
    assert!(foot.native_cranes.slots[slot].as_ref().unwrap().auxiliary.is_none());
    foot.map.native_terrain_plane.fill(56);
    foot.map.terrain_codes.fill(56);
    foot.refresh_native_main_pattern_step(0, 0);
    assert!(foot.native_cranes.slots[slot].as_ref().unwrap().auxiliary.is_none());
    let mut g = field("crane");
    let slot = bind_work(&mut g, 255);
    g.map.native_terrain_plane.fill(56);
    g.map.terrain_codes.fill(56);
    g.refresh_native_pattern_step(0, 0);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 255,
        "constructor/generic refresh must not acquire the Main contact callback");
    g.native_cranes.slots[slot] = None;
    g.refresh_native_pattern_step(0, 0);
    g.refresh_native_main_pattern_step(0, 0);
    assert!(g.native_cranes.slots[slot].is_none(), "pre-ctor refresh is not a receipt");
    g.record_native_crane_ctor(0);
    let identity = g.native_cranes.slots[slot].as_ref().unwrap().identity;
    g.record_native_upstream_ctor(0); // Actual next physical generation event.
    g.refresh_native_main_pattern_step(0, 0);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().identity, identity);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 0);
    g.record_native_crane_ctor(0);
    let c = g.native_cranes.slots[slot].as_mut().unwrap();
    c.entity += 100;
    c.work9a = 255;
    let old_entity = c.entity;
    g.refresh_native_main_pattern_step(0, 0);
    let c = g.native_cranes.slots[slot].as_ref().unwrap();
    assert_eq!((c.entity, c.work9a), (old_entity, 255), "no renderer-ID fallback");
}

#[test]
fn main_contact_precedes_pattern_and_only_reads_the_pre_movement_cell() {
    let mut g = field("crane");
    let slot = bind_work(&mut g, 255);
    for y in 18..=22 {
        g.map.native_terrain_plane[y * 128 + 21] = 56;
        g.map.terrain_codes[y * 128 + 21] = 56;
    }
    g.units[0].x = 654.; // Native327, center cell20.
    g.refresh_native_main_pattern_step(0, 0);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 255);
    assert_eq!(g.units[0].native_cell10, 20 * 128 + 20);
    g.units[0].x = 656.; // Controlled next callback point; no movement claim.
    g.refresh_native_pattern_step(0, 0);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 255);
    g.refresh_native_main_pattern_step(0, 0);
    assert_eq!(g.units[0].native_cell10, 20 * 128 + 21);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 1);
    assert_ne!(g.native_group_rosters.members[slot].as_ref().unwrap().flags69 & 4, 0);
    let c = g.native_cranes.slots[slot].as_ref().unwrap();
    assert!(c.tainted() && c.observed57ff0_86ff);
    assert_eq!(c.field96.value, 0, "contact is not a second96-prefix tick");
}

#[test]
fn frozen_and_retired_units_do_not_reach_the_main_contact() {
    let mut frozen = field("crane");
    let slot = bind_work(&mut frozen, 255);
    frozen.map.native_terrain_plane.fill(56);
    frozen.map.terrain_codes.fill(56);
    frozen.native_group_rosters.members[slot].as_mut().unwrap().flags69 |= 64;
    let before = frozen.native_cranes.slots[slot].as_ref().unwrap().kernel_view();
    frozen.step(0.059);
    let after = frozen.native_cranes.slots[slot].as_ref().unwrap().kernel_view();
    assert_eq!((after.work9a, after.frame86, after.mouth96),
        (before.work9a, before.frame86, before.mouth96));
    let mut retired = field("crane");
    let slot = bind_work(&mut retired, 255);
    retired.map.native_terrain_plane.fill(56);
    retired.map.terrain_codes.fill(56);
    assert!(retired.native_remove_actor(0, true, false));
    assert!(retired.units[0].native_slot.is_none());
    let before = retired.native_cranes.slots[slot].as_ref().unwrap().kernel_view();
    retired.refresh_native_main_pattern_step(0, 0);
    retired.step(0.059);
    let after = retired.native_cranes.slots[slot].as_ref().unwrap().kernel_view();
    assert_eq!((after.work9a, after.frame86, after.mouth96),
        (before.work9a, before.frame86, before.mouth96));
}

#[test]
fn source72_contact_and_work_projection_is_executed_without_private_assets() {
    // This is deliberately a projection: callback positions are Source inputs,
    // not a claim that this fixture simulates original Common's entire motion.
    let mut hash = 2166136261u32;
    for scenario in 0..24 {
        let mut g = field("crane");
        g.multiplayer = true;
        let slot = bind_work(&mut g, if scenario / 12 == 0 { 255 } else { 0 });
        for y in 18..=22 {
            g.map.native_terrain_plane[y * 128 + 21] = 56;
            g.map.terrain_codes[y * 128 + 21] = 56;
        }
        let start = if scenario / 6 % 2 == 0 { 327 } else { 326 };
        for callback in 0..3 {
            g.units[0].x = f64::from(start + callback) * 2.;
            g.refresh_native_main_pattern_step(0, 0);
            assert!(g.update_native_crane_building(0, 0, 0));
            assert_eq!(g.native_route_boundary, None);
            let c = g.native_cranes.slots[slot].as_ref().unwrap();
            let work = if scenario / 12 != 0 { 0 }
                else if start + callback < 328 { 255 }
                else { (start + callback - 326) as u8 };
            assert_eq!((c.work9a, c.frame86, c.field96.value),
                (work, -1, (callback + 1) as u8));
            assert!(c.observed57ff0_86ff && c.tainted());
            assert_eq!(c.field96.knowledge, super::super::crane_state::Knowledge96::Tainted);
            let progress = raw_word(&g.native_world.as_ref().unwrap().trail.builds[0], 60);
            for n in [scenario as u32, callback as u32, u32::from(c.work9a),
                u32::from(c.frame86 as u8), u32::from(c.field96.value), progress as u16 as u32] {
                for byte in n.to_le_bytes() { hash = (hash ^ u32::from(byte)).wrapping_mul(16777619); }
            }
        }
    }
    assert_eq!(hash, 2141903685, "frozen Source72 contact/work projection");
}

#[test]
fn accepted_build_setup_defers_missing_frames_until_the_reached_overlay() {
    let mut g = field("crane");
    // Explicit controlled BUILD raw-data availability, independent of frames.
    g.native_world.as_mut().unwrap().trail.known_builds[0] = true;
    assert_eq!(g.begin_native_crane_building(0, 7, Some([336, 400])), Some(true));
    let slot = g.units[0].native_slot.unwrap();
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().work9a, 255);
    assert_eq!(g.native_world.as_ref().unwrap().trail.boundary, None);
    // Reach the existing work-contact transition through the real Main hook;
    // do not invent a completed approach or manually supply an overlay frame.
    g.map.native_terrain_plane.fill(58);
    g.map.terrain_codes.fill(58);
    g.refresh_native_main_pattern_step(0, 0);
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().frame86, 0);
    assert_ne!(g.native_world.as_ref().unwrap().trail.builds[0][6] & 8, 0);
    let before = g.native_cranes.slots[slot].as_ref().unwrap().field96;
    assert!(g.update_native_crane_building(0, 0, 1));
    assert_eq!(g.native_cranes.slots[slot].as_ref().unwrap().field96, before,
        "the reached missing overlay returns before96/Common, not at setup");
    assert_eq!(g.native_world.as_ref().unwrap().trail.boundary,
        Some(trail::Boundary { address: 0x495644, length: 0 }));
    let mut unknown = field("crane");
    unknown.native_world.as_mut().unwrap().trail.known_builds[0] = false;
    assert_eq!(unknown.begin_native_crane_building(0, 7, Some([336, 400])), Some(false));
}

#[test]
fn removing_the_frame_guard_keeps_target_kind_and_geometry_admission() {
    let mut g = field("crane");
    g.native_world.as_mut().unwrap().trail.known_builds[0] = true;
    assert_eq!(g.begin_native_crane_building(0, 999, Some([336, 400])), Some(false));
    assert_eq!(g.begin_native_crane_building(0, 7, Some([-100, -100])), Some(false));
    g.native_world.as_mut().unwrap().buildings[0].record.hp = 1;
    assert_eq!(g.begin_native_crane_building(0, 7, Some([336, 400])), Some(false));
    g.native_world.as_mut().unwrap().buildings[0].record.hp = 0;
    g.native_world.as_mut().unwrap().buildings[0].fort_proxy = true;
    assert_eq!(g.begin_native_crane_building(0, 7, Some([336, 400])), Some(false));
    let mut foot = field("grunt");
    foot.native_world.as_mut().unwrap().trail.known_builds[0] = true;
    assert_eq!(foot.begin_native_crane_building(0, 7, Some([336, 400])), Some(false));
    assert!(foot.native_cranes.slots[foot.units[0].native_slot.unwrap()]
        .as_ref().unwrap().auxiliary.is_none());
}
