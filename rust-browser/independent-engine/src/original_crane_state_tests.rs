use super::*;
use crate::{json, original_route, Game, Value};
fn field() -> Game {
    let mut g = Game::new(0, 1, 50);
    g.map.original = true;
    g.map.width = 64;
    g.map.height = 64;
    g.map.tiles = vec![0; 4096];
    g.map.terrain_codes = vec![0; 4096];
    g.map.native_terrain_plane = vec![0; 16384];
    g.map.original_tiles = vec![0; 4096];
    g.map.bridge_base = vec![-1; 4096];
    g.map.native_road_graph = Some(original_route::RoadGraph { regions: vec![] });
    g.units.clear();
    g.buildings.clear();
    g.objects.clear();
    g.crates.clear();
    g.walls.clear();
    g.sectors.clear();
    g.native_group_rosters.clear();
    g.native_upstream_reactions.clear_for_mission();
    g.native_cranes.clear_for_mission();
    g.native_world = None;
    g
}
#[test]
fn actual_fresh_foot_receipt_survives_world_none_and_reimport_without_creating_work_defaults() {
    let mut g = field();
    g.spawn("grunt", 0, 320., 320., 1, None);
    let u = &g.units[0];
    let (slot, id) = (u.native_slot.unwrap(), u.id);
    assert!(g.native_world.is_none());
    let r = g.native_cranes.slots[slot].as_ref().unwrap();
    assert_eq!((r.work9a, r.work9a_known), (0, true));
    assert!(r.auxiliary.is_none());
    assert!(
        std::panic::catch_unwind(|| r.kernel_view()).is_err(),
        "phase-only is not a work view"
    );
    g.native_cranes.slots[slot].as_mut().unwrap().work9a = 71;
    g.import_native_world(&Value::Null);
    assert_eq!(g.native_crane_work9a(slot, id), Ok(71));
    g.import_native_world(&Value::Null);
    assert_eq!(g.native_crane_work9a(slot, id), Ok(71));
    assert!(g.native_cranes.slots[slot]
        .as_ref()
        .unwrap()
        .auxiliary
        .is_none());
    g.record_native_crane_ctor(0);
    assert_eq!(
        g.native_crane_work9a(slot, id),
        Ok(71),
        "duplicate receipt/import is not another physical ctor"
    );
}
#[test]
fn generation_epoch_and_exact_renderer_rebind_have_one_physical_phase_owner() {
    let mut s = State::default();
    s.constructed(5, 1, 19, false);
    s.slots[5].as_mut().unwrap().work9a = 71;
    assert!(!s.rebind(5, 2, 19, 5));
    assert!(!s.rebind(5, 1, 99, 5));
    assert!(s.rebind(5, 1, 19, 5));
    assert_eq!(s.phase9a(5, 1, 5), Some(71));
    assert_eq!(s.phase9a(5, 1, 19), None);
    s.constructed(5, 2, 5, false);
    assert_eq!(s.phase9a(5, 2, 5), Some(0));
    let old = s.slots[5].clone().unwrap();
    s.clear_for_mission();
    s.slots[5] = Some(old);
    assert_eq!(s.phase9a(5, 2, 5), None);
    assert!(!s.rebind(5, 2, 5, 19));
    s.constructed(5, 2, 5, false);
    assert_eq!(s.phase9a(5, 2, 5), Some(0));
}
#[test]
fn known_or_unknown_warm_phase_survives57ff0_and_only86_is_newly_known() {
    for known in [false, true] {
        for value in [0, 1, 71, 254, 255] {
            let mut s = State::default();
            s.slots[5] = Some(Record::fixture_phase(s.key(1), 19, value, known));
            s.initializer57ff0(5, 1, 19);
            let r = s.slots[5].as_ref().unwrap();
            assert_eq!((r.work9a, r.work9a_known), (value, known));
            let a = r.auxiliary.as_ref().unwrap();
            assert_eq!(a.frame86, -1);
            assert!(a.observed57ff0_86ff && a.tainted());
            assert!(s.rebind(5, 1, 19, 5));
            assert_eq!(s.phase9a(5, 1, 5), known.then_some(value));
        }
    }
}
#[test]
fn legacy_playability_auxiliary_cannot_promote_an_unowned_warm_phase() {
    let mut s = State::default();
    let mut c = s.view_for_legacy_crane(5, 9, 19);
    assert!(!c.work9a_known);
    c.mouth96 = 7;
    assert!(s.write96(
        5,
        s.key(9),
        19,
        Write96::Mouth {
            before: 0,
            after: 7
        },
        Authority96::TaintedLegacy
    ));
    c.saved = [100, 200];
    s.store(5, 9, c);
    assert_eq!(s.phase9a(5, 9, 19), None);
    let r = s.slots[5].as_ref().unwrap();
    assert!(r.auxiliary.as_ref().unwrap().tainted());
    assert!(!r.observed57ff0_86ff);
    assert_eq!(r.field96.value, 7);
    assert_eq!(r.saved, [100, 200]);
}
#[test]
fn constructor57ff0_preserves_existing_tainted_auxiliary_bytes_and_only_replaces_its_actual_fields()
{
    let mut s = State::default();
    let mut c = s.view_for_legacy_crane(5, 1, 19);
    c.work9a = 71;
    c.work9a_known = true;
    c.mouth96 = 37;
    assert!(s.write96(
        5,
        s.key(1),
        19,
        Write96::Mouth {
            before: 0,
            after: 37
        },
        Authority96::TaintedLegacy
    ));
    c.saved = [123, 456];
    c.interval87 = 29;
    c.delay82 = 35;
    c.approach_b1 = Some(3);
    c.target = Some(4);
    s.store(5, 1, c);
    s.initializer57ff0(5, 1, 19);
    let r = s.slots[5].as_ref().unwrap();
    assert_eq!((r.work9a, r.work9a_known), (71, true));
    assert_eq!((r.frame86, r.target), (-1, None));
    assert_eq!(
        (
            r.field96.value,
            r.saved,
            r.interval87,
            r.delay82,
            r.approach_b1
        ),
        (37, [123, 456], 29, 35, Some(3))
    );
    assert!(r.auxiliary.as_ref().unwrap().tainted());
}
#[test]
fn retained_phase_projection_matches_all512_real_constructor_and_takeover_images() {
    let trace = std::env::var_os("Z_NATIVE_TRACE_DIR").map(|p| {
        std::fs::read_to_string(std::path::PathBuf::from(p).join("V47-crane9a-provenance.jsonl"))
            .expect("explicit private trace path")
    });
    let rows = trace.as_deref().map(|t| {
        t.lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    });
    let mut h = 2166136261u32;
    for mode in 0..2u32 {
        for seed in 0..256u32 {
            let mut s = State::default();
            s.constructed(0, 1, 19, false);
            assert!(s.slots[0].as_ref().unwrap().auxiliary.is_none());
            let old = (seed * 17 + 154 * 23) as u8;
            let fresh = s.phase9a(0, 1, 19).unwrap();
            if mode == 1 {
                s.slots[0].as_mut().unwrap().work9a = seed as u8;
            }
            let input = s.phase9a(0, 1, 19).unwrap();
            s.initializer57ff0(0, 1, 19);
            assert!(s.rebind(0, 1, 19, 5));
            let a = s.slots[0].as_ref().unwrap();
            let after = a.work9a;
            let frame = a.frame86 as u8;
            let bytes = mode
                .to_le_bytes()
                .into_iter()
                .chain(seed.to_le_bytes())
                .chain([old, fresh, input, after, frame])
                .chain(0x2084u32.to_le_bytes())
                .collect::<Vec<_>>();
            if let Some(rows) = &rows {
                let r = &rows[(mode * 256 + seed) as usize];
                let raw = |key: &str| {
                    let v = r[key].as_str().unwrap();
                    (0..v.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&v[i..i + 2], 16).unwrap())
                        .collect::<Vec<_>>()
                };
                let o = raw("old");
                let f = raw("fresh");
                let p = raw("input");
                let a = raw("after");
                assert_eq!(
                    [old, fresh, input, after, frame],
                    [o[154], f[154], p[154], a[154], a[134]]
                );
                assert_eq!(&a[40..44], &0x2084u32.to_le_bytes());
            }
            for b in bytes {
                h = (h ^ u32::from(b)).wrapping_mul(16777619);
            }
        }
    }
    assert_eq!(h,1473382597,"512 actual41b210/50be0/57ff0 images, narrow9a/86/28 projection; other raw work bytes remain unclaimed");
}

#[test]
fn missing_renderer_binding_never_erases_or_adopts_a_known_physical_phase() {
    let mut s = State::default();
    s.constructed(5, 1, 19, false);
    s.slots[5].as_mut().unwrap().work9a = 71;
    let c = s.view_for_legacy_crane(5, 1, 5);
    assert!(!c.work9a_known);
    s.store(5, 1, c);
    assert_eq!(s.phase9a(5, 1, 19), Some(71));
    assert_eq!(s.phase9a(5, 1, 5), None);
    assert!(s.slots[5].as_ref().unwrap().auxiliary.is_none());
    s.initializer57ff0(5, 1, 5);
    assert_eq!(s.phase9a(5, 1, 19), Some(71));
    assert_eq!(s.slots[5].as_ref().unwrap().entity, 19);
    assert!(s.rebind(5, 1, 19, 5));
    assert_eq!(s.phase9a(5, 1, 5), Some(71));
}
