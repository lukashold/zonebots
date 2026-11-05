use super::super::crane::{self, World};
use super::*;
use crate::{original_route, Game, Value};

fn field() -> Game {
    let mut g = Game::new(0, 1, 52);
    g.map.original = true;
    g.map.width = 64;
    g.map.height = 64;
    g.map.tiles = vec![0; 4096];
    g.map.terrain_codes = vec![0; 4096];
    g.map.original_tiles = vec![0; 4096];
    g.map.native_terrain_plane = vec![0; 16384];
    g.map.native_road_graph = Some(original_route::RoadGraph { regions: vec![] });
    g.units.clear();
    g.buildings.clear();
    g.sectors.clear();
    g.objects.clear();
    g.native_group_rosters.clear();
    g.native_upstream_reactions.clear_for_mission();
    g.native_cranes.clear_for_mission();
    g.native_world = None;
    g
}

#[test]
fn real_foot_receipt96_survives_import_without_a_work_auxiliary_or_dto() {
    let mut g = field();
    g.spawn("grunt", 0, 320., 320., 1, None);
    let slot = g.units[0].native_slot.unwrap();
    assert!(g.native_world.is_none());
    let check = |g: &Game| {
        let r = g.native_cranes.slots[slot].as_ref().unwrap();
        assert_eq!(r.field96, Field96::fresh());
        assert!(r.auxiliary.is_none());
        assert!(std::panic::catch_unwind(|| r.kernel_view()).is_err());
    };
    check(&g);
    g.import_native_world(&Value::Null);
    check(&g);
    g.import_native_world(&Value::Null);
    check(&g);
    g.record_native_crane_ctor(0);
    check(&g);
}

#[test]
fn physical_key_rebind_and57ff0_preserve96_but_cannot_adopt_unbound_or_warm_knowledge() {
    let mut s = State::default();
    assert!(!s.write96(
        5,
        s.key(1),
        19,
        Write96::CompletionZero,
        Authority96::ProvenReachedReadset
    ));
    assert!(
        s.slots[5].is_none(),
        "a writer cannot fabricate an unbound record"
    );
    let c = s.view_for_legacy_crane(5, 1, 19);
    assert_eq!(s.slots[5].as_ref().unwrap().field96, Field96::tainted(0));
    s.initializer57ff0(5, 1, 19);
    assert!(!s.slots[5].as_ref().unwrap().field96.is_known());
    s.store(5, 1, c);
    s.constructed(5, 2, 19, false);
    assert_eq!(s.slots[5].as_ref().unwrap().field96, Field96::fresh());
    assert!(s.rebind(5, 2, 19, 5));
    s.initializer57ff0(5, 2, 5);
    assert_eq!(s.slots[5].as_ref().unwrap().field96, Field96::fresh());
    assert!(!s.write96(
        5,
        s.key(1),
        5,
        Write96::CompletionZero,
        Authority96::TaintedLegacy
    ));
    assert!(!s.write96(
        5,
        s.key(2),
        19,
        Write96::CompletionZero,
        Authority96::TaintedLegacy
    ));
    let old_identity = s.key(2);
    s.clear_for_mission();
    s.constructed(5, 2, 5, false);
    assert!(!s.write96(
        5,
        old_identity,
        5,
        Write96::CompletionZero,
        Authority96::ProvenReachedReadset
    ));
    assert_eq!(
        s.slots[5].as_ref().unwrap().field96,
        Field96::fresh(),
        "reused generation/entity in a new epoch cannot admit an old effect"
    );
}

#[test]
fn bulk_dto_store_never_writes96_and_historical86_receipt_is_not_current_proof() {
    let mut s = State::default();
    s.constructed(5, 1, 19, true);
    let mut c = s.view_for_legacy_crane(5, 1, 19);
    c.mouth96 = 99;
    c.frame86 = 6;
    s.store(5, 1, c);
    let r = s.slots[5].as_ref().unwrap();
    assert_eq!(r.field96, Field96::fresh());
    assert_eq!(r.frame86, 6);
    assert!(r.observed57ff0_86ff && r.auxiliary.as_ref().unwrap().tainted());
    assert!(s.write96(
        5,
        s.key(1),
        19,
        Write96::Mouth {
            before: 0,
            after: 0
        },
        Authority96::TaintedLegacy
    ));
    assert_eq!(
        s.slots[5].as_ref().unwrap().field96,
        Field96::tainted(0),
        "equal numbers are not a readset proof"
    );
}

#[test]
fn reached_writer_proof_requires_its_readset_and_completion_constant_is_not_a_ui_event() {
    let mut s = State::default();
    s.constructed(5, 1, 19, true);
    s.write96(
        5,
        s.key(1),
        19,
        Write96::Mouth {
            before: 0,
            after: 1,
        },
        Authority96::ProvenReachedReadset,
    );
    assert_eq!(
        s.slots[5].as_ref().unwrap().field96.knowledge,
        Knowledge96::Mouth44fe18
    );
    s.write96(
        5,
        s.key(1),
        19,
        Write96::Mouth {
            before: 0,
            after: 2,
        },
        Authority96::ProvenReachedReadset,
    );
    assert_eq!(
        s.slots[5].as_ref().unwrap().field96,
        Field96::tainted(2),
        "stale read cannot establish knowledge"
    );
    s.write96(
        5,
        s.key(1),
        19,
        Write96::Mouth {
            before: 2,
            after: 3,
        },
        Authority96::ProvenReachedReadset,
    );
    assert!(
        !s.slots[5].as_ref().unwrap().field96.is_known(),
        "unknown prior byte cannot be repaired by numeric identity"
    );
    s.write96(
        5,
        s.key(1),
        19,
        Write96::CompletionZero,
        Authority96::TaintedLegacy,
    );
    assert_eq!(s.slots[5].as_ref().unwrap().field96, Field96::tainted(0));
    s.write96(
        5,
        s.key(1),
        19,
        Write96::CompletionZero,
        Authority96::ProvenReachedReadset,
    );
    assert_eq!(
        s.slots[5].as_ref().unwrap().field96.knowledge,
        Knowledge96::Completion44fe77
    );
}

struct W {
    state: State,
    calls: Vec<&'static str>,
    effects: Vec<Write96>,
    phase_after_common: Option<u8>,
    common_ok: bool,
    busy: u8,
    overlay: Option<bool>,
    scenario: usize,
    crt: usize,
}
impl W {
    fn new() -> Self {
        let mut state = State::default();
        state.constructed(5, 1, 19, true);
        Self {
            state,
            calls: vec![],
            effects: vec![],
            phase_after_common: None,
            common_ok: true,
            busy: 0,
            overlay: Some(false),
            scenario: 0,
            crt: 0,
        }
    }
}
impl World for W {
    fn overlay(&mut self, _: &Crane) -> Option<bool> {
        self.calls.push("overlay");
        self.overlay
    }
    fn crt(&mut self) -> u16 {
        let n = (self.scenario * 3571 + self.crt * 131 + 0x1743) & 32767;
        self.crt += 1;
        n as u16
    }
    fn write96(&mut self, e: Write96) {
        self.calls.push(if matches!(e, Write96::Mouth { .. }) {
            "mouth"
        } else {
            "zero"
        });
        self.effects.push(e);
        assert!(self
            .state
            .write96(5, self.state.key(1), 19, e, Authority96::TaintedLegacy));
    }
    fn common(&mut self, c: &mut Crane) -> bool {
        self.calls.push("common");
        assert_eq!(
            self.state.slots[5].as_ref().unwrap().field96.value,
            c.mouth96,
            "write must precede Common"
        );
        if let Some(p) = self.phase_after_common {
            c.work9a = p;
        }
        self.common_ok
    }
    fn check_path(&mut self) -> u8 {
        self.calls.push("check");
        self.busy
    }
    fn job(&mut self, _: &Crane) {
        self.calls.push("job");
    }
    fn progress(&mut self, _: &mut Crane) -> bool {
        self.calls.push("progress");
        false
    }
    fn prepend(&mut self, _: &Crane) -> bool {
        panic!("unexpected prepend")
    }
    fn start_path(&mut self) -> bool {
        panic!("unexpected start")
    }
    fn mark_moving(&mut self) {
        panic!("unexpected movement")
    }
    fn cleanup(&mut self) {
        self.calls.push("cleanup");
    }
}

#[test]
fn typed_effects_are_reached_once_at_source_stages_and_read_phase_after_common() {
    let mut w = W::new();
    let mut c = w.state.view_for_legacy_crane(5, 1, 19);
    w.common_ok = false;
    crane::tick(&mut c, &mut w, true, false, false);
    assert_eq!(w.calls, vec!["mouth", "common"]);
    assert_eq!(
        w.state.slots[5].as_ref().unwrap().field96,
        Field96::tainted(1)
    );
    c.mouth96 = 99;
    w.state.store(5, 1, c);
    assert_eq!(w.state.slots[5].as_ref().unwrap().field96.value, 1);
    let mut w = W::new();
    let mut c = w.state.view_for_legacy_crane(5, 1, 19);
    c.work9a = 254;
    w.phase_after_common = Some(0);
    crane::tick(&mut c, &mut w, true, false, false);
    assert_eq!(
        w.calls,
        vec!["mouth", "common"],
        "post-Common0 avoids stale FE completion"
    );
    let mut w = W::new();
    let mut c = w.state.view_for_legacy_crane(5, 1, 19);
    w.phase_after_common = Some(254);
    crane::tick(&mut c, &mut w, true, false, false);
    assert_eq!(
        w.calls,
        vec!["mouth", "common", "check", "zero", "job", "cleanup"]
    );
    assert_eq!(
        w.effects,
        vec![
            Write96::Mouth {
                before: 0,
                after: 1
            },
            Write96::CompletionZero
        ]
    );
    assert_eq!(
        w.state.slots[5].as_ref().unwrap().field96,
        Field96::tainted(0)
    );
    let mut w = W::new();
    let mut c = w.state.view_for_legacy_crane(5, 1, 19);
    c.work9a = 254;
    w.busy = 1;
    crane::tick(&mut c, &mut w, true, false, false);
    assert_eq!(w.calls, vec!["mouth", "common", "check"]);
    let mut w = W::new();
    let mut c = w.state.view_for_legacy_crane(5, 1, 19);
    c.frame86 = 0;
    w.overlay = None;
    crane::tick(&mut c, &mut w, true, false, false);
    assert_eq!(w.calls, vec!["overlay"]);
    assert!(w.effects.is_empty());
    assert_eq!(w.state.slots[5].as_ref().unwrap().field96, Field96::fresh());
}

#[test]
fn live_common_failure_cannot_roll_back_its_already_reached_prefix96_write() {
    let mut g = field();
    g.import_native_world(&Value::Null);
    g.multiplayer = true;
    g.spawn("crane", 0, 640., 640., 1, None);
    g.maintain_native_group_rosters();
    let slot = g.units[0].native_slot.unwrap();
    let id = g.units[0].id;
    let generation = g.native_upstream_reactions.generation(slot).unwrap();
    g.native_world.as_mut().unwrap().crane_frames = vec![vec![]; 12];
    g.native_group_rosters.members[slot] = None;
    assert!(g.update_native_crane_building(0, 0, 1));
    assert!(g.native_route_boundary.is_some());
    assert_eq!(
        g.native_cranes.bound(slot, generation, id).unwrap().field96,
        Field96::tainted(1)
    );
    let mut stale = g.native_cranes.view_for_legacy_crane(slot, generation, id);
    stale.mouth96 = 0;
    g.native_cranes.store(slot, generation, stale);
    assert_eq!(
        g.native_cranes
            .bound(slot, generation, id)
            .unwrap()
            .field96
            .value,
        1
    );
}

#[test]
fn actual_retirement_preserves_committed96_and_reuse_cannot_accept_the_old_effect() {
    let mut g = field();
    g.import_native_world(&Value::Null);
    g.multiplayer = true;
    g.spawn("grunt", 0, 640., 640., 1, None);
    g.maintain_native_group_rosters();
    let slot = g.units[0].native_slot.unwrap();
    let id = g.units[0].id;
    let generation = g.native_upstream_reactions.generation(slot).unwrap();
    g.native_cranes.write96(
        slot,
        g.native_cranes.key(generation),
        id,
        Write96::Mouth {
            before: 0,
            after: 7,
        },
        Authority96::TaintedLegacy,
    );
    assert!(g.native_remove_actor(0, true, false));
    assert_eq!(
        g.native_cranes.bound(slot, generation, id).unwrap().field96,
        Field96::tainted(7)
    );
    g.spawn("grunt", 0, 640., 640., 2, None);
    let fresh = g.units.last().unwrap();
    assert_eq!(fresh.native_slot, Some(slot));
    assert_eq!(
        g.native_cranes.slots[slot].as_ref().unwrap().field96,
        Field96::fresh()
    );
    assert!(!g.native_cranes.write96(
        slot,
        g.native_cranes.key(generation),
        id,
        Write96::CompletionZero,
        Authority96::ProvenReachedReadset
    ));
}

fn add_hash(hash: &mut u32, bytes: impl IntoIterator<Item = u8>) {
    for b in bytes {
        *hash = (*hash ^ u32::from(b)).wrapping_mul(16777619);
    }
}
fn private_rows(name: &str) -> Option<Vec<Value>> {
    std::env::var_os("Z_NATIVE_TRACE_DIR").map(|root| {
        std::fs::read_to_string(std::path::PathBuf::from(root).join(name))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    })
}
fn byte(row: &Value, key: &str, at: usize) -> u8 {
    u8::from_str_radix(&row[key].as_str().unwrap()[at * 2..at * 2 + 2], 16).unwrap()
}

#[test]
fn writer_effect_projection_matches192_real_prefixes_with_fixed_public_golden() {
    let rows = private_rows("V51-crane-idle-prefix.jsonl");
    let mut h = 2166136261;
    for s in 0..192usize {
        let mut w = W::new();
        w.scenario = s;
        let mut c = w.state.view_for_legacy_crane(5, 1, 19);
        c.work9a = if s / 6 % 2 == 0 { 0 } else { 255 };
        c.mouth96 = [0, 2, 3, 255][s / 48];
        w.state.slots[5].as_mut().unwrap().field96 = Field96::tainted(c.mouth96);
        let before = c.mouth96;
        crane::tick(&mut c, &mut w, s / 12 % 2 != 0, false, s / 24 % 2 != 0);
        assert_eq!(w.effects.len(), 1);
        assert_eq!(
            w.effects[0],
            Write96::Mouth {
                before,
                after: c.mouth96
            }
        );
        assert_eq!(w.calls, vec!["mouth", "common"]);
        if let Some(rows) = &rows {
            assert_eq!(byte(&rows[s], "before", 0x96), before);
            assert_eq!(byte(&rows[s], "after", 0x96), c.mouth96);
            assert_eq!(rows[s]["crtCalls"], w.crt);
        }
        add_hash(
            &mut h,
            (s as u32)
                .to_le_bytes()
                .into_iter()
                .chain([before, c.mouth96])
                .chain(w.effects[0].address().to_le_bytes())
                .chain((w.crt as u32).to_le_bytes()),
        );
    }
    assert_eq!(h, 1782045295);
}

#[test]
fn fresh96_receipt_and_retained_takeover_match512_original_images() {
    let rows = private_rows("V47-crane9a-provenance.jsonl");
    let mut h = 2166136261;
    for mode in 0..2u32 {
        for seed in 0..256u32 {
            let mut s = State::default();
            s.constructed(0, 1, 19, false);
            assert!(s.slots[0].as_ref().unwrap().auxiliary.is_none());
            let old = (seed * 17 + 0x96 * 23) as u8;
            let fresh = s.slots[0].as_ref().unwrap().field96.value;
            s.initializer57ff0(0, 1, 19);
            assert!(s.rebind(0, 1, 19, 5));
            let r = s.bound(0, 1, 5).unwrap();
            assert_eq!(r.field96, Field96::fresh());
            if let Some(rows) = &rows {
                let row = &rows[(mode * 256 + seed) as usize];
                assert_eq!(
                    [
                        byte(row, "old", 0x96),
                        byte(row, "fresh", 0x96),
                        byte(row, "input", 0x96),
                        byte(row, "after", 0x96)
                    ],
                    [old, fresh, fresh, r.field96.value]
                );
            }
            add_hash(
                &mut h,
                mode.to_le_bytes()
                    .into_iter()
                    .chain(seed.to_le_bytes())
                    .chain([old, fresh, fresh, r.field96.value]),
            );
        }
    }
    assert_eq!(h, 1875346885);
}
