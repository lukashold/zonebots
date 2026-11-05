//! Private Source replays use actual leaf results. This tests only the
//! explicitly ported caller, not the replayed scanner/scatter/emitter bodies.
use super::*;
use serde_json::Value;
use std::collections::VecDeque;
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
        .collect()
}
fn word(v: &[u8], p: usize) -> i16 {
    i16::from_le_bytes([v[p], v[p + 1]])
}
fn arr<const N: usize>(s: &str) -> [u8; N] {
    bytes(s).try_into().unwrap()
}
fn side_offset(s: Side) -> usize {
    match s {
        Side::Right => 2,
        Side::Left => 6,
        Side::Front => 7,
        Side::Back => 10,
    }
}
struct Replay {
    raw: [u8; 192],
    w: [u8; 44],
    shared: [u8; 24],
    phases: [u8; 240],
    roster: [u8; 21],
    target: [u8; 192],
    operations: VecDeque<String>,
    mp: bool,
    error: u8,
    probe: [i16; 2],
    scans: usize,
    fires: usize,
    bounds: [[i16; 4]; 64],
    sector_flags: [u8; 64],
    strategy: (u8, u8),
    busy: u8,
    leaf_operations: VecDeque<String>,
    laser: u8,
    bullet_output: Option<u32>,
    bullets: Vec<u8>,
    sound_cycle: u32,
    sound_once: u8,
}
impl Replay {
    fn new() -> Self {
        Self {
            raw: [0; 192],
            w: [0; 44],
            shared: [255; 24],
            phases: [0; 240],
            roster: [255; 21],
            target: [0; 192],
            operations: VecDeque::new(),
            mp: false,
            error: 5,
            probe: [0; 2],
            scans: 0,
            fires: 0,
            bounds: [[-1; 4]; 64],
            sector_flags: [0; 64],
            strategy: (0, 0),
            busy: 0,
            leaf_operations: VecDeque::new(),
            laser: 0,
            bullet_output: None,
            bullets: Vec::new(),
            sound_cycle: 0,
            sound_once: 0,
        }
    }
    fn pop(&mut self, prefix: &str) -> Vec<String> {
        let v = self.operations.pop_front().expect(prefix);
        assert!(v.starts_with(prefix), "expected {prefix}: {v}");
        v.split(':').map(str::to_owned).collect()
    }
    fn leaf_pop(&mut self, prefix: &str) -> Vec<String> {
        let v = self.leaf_operations.pop_front().expect(prefix);
        assert!(v.starts_with(prefix), "expected {prefix}: {v}");
        v.split(':').map(str::to_owned).collect()
    }
    fn apply_rng(rng: &mut OriginalRng, s: &str) {
        let rr = bytes(s);
        for i in 0..5 {
            rng.words[i] = u32::from_le_bytes(rr[i * 4..i * 4 + 4].try_into().unwrap());
        }
    }
}
impl GateWorld for Replay {
    fn multiplayer(&self) -> bool {
        self.mp
    }
    fn strategy(&mut self, g: u8) -> Result<(u8, u8)> {
        if g >= 240 {
            Err(Boundary::Group(g))
        } else {
            Ok(self.strategy)
        }
    }
    fn sector_bounds(&mut self, i: usize) -> Result<[i16; 4]> {
        Ok(self.bounds[i])
    }
    fn sector_flags24(&mut self, i: usize) -> Result<u8> {
        Ok(self.sector_flags[i])
    }
    fn current_group3d(&mut self) -> Result<u8> {
        Ok(self.busy)
    }
    fn trail_error(&self) -> u8 {
        self.error
    }
    fn write_trail_error(&mut self, e: u8) {
        self.error = e;
    }
}
impl ActorTeam for Replay {
    fn actor_team31(&mut self) -> Result<u8> {
        Ok(self.raw[0x31])
    }
}
impl World for Replay {
    fn actor(&self) -> Actor {
        Actor {
            slot: 4,
            position: [word(&self.raw, 0x2c), word(&self.raw, 0x2e)],
            family30: self.raw[0x30],
            class7c: self.raw[0x7c],
            group5f: self.raw[0x5f],
            facing39: self.raw[0x39],
            saved_class8e: self.raw[0x8e],
            rate78: self.raw[0x78],
            reload7b: self.raw[0x7b],
        }
    }
    fn write_reload7b(&mut self, v: u8) {
        self.raw[0x7b] = v;
    }
    fn side_target(&self, s: Side) -> u8 {
        self.shared[side_offset(s)]
    }
    fn write_side_target(&mut self, s: Side, v: u8) {
        self.shared[side_offset(s)] = v;
    }
    fn scan58500(&mut self, _relative: i8) -> Result<u16> {
        let p = self.pop("S:");
        self.scans += 1;
        self.raw[0x71] = p[2].parse().unwrap();
        self.raw[0x77] = p[3].parse().unwrap();
        Ok(p[1].parse().unwrap())
    }
    fn target_position(&mut self, s: u8) -> Result<[i16; 2]> {
        if s != 1 {
            return Err(Boundary::Target(s));
        }
        Ok([word(&self.target, 0x2c), word(&self.target, 0x2e)])
    }
    fn target_flags31(&mut self, s: u8) -> Result<u8> {
        if s != 1 {
            return Err(Boundary::Target(s));
        }
        Ok(self.target[0x31])
    }
    fn weapon_range5(&mut self) -> Result<u8> {
        Ok(self.w[5])
    }
    fn write_range_point(&mut self, p: [i16; 2]) {
        self.probe = p;
    }
    fn roster_member21(&mut self, g: u8, i: usize) -> Result<u8> {
        if g >= 240 {
            Err(Boundary::Group(g))
        } else {
            Ok(self.roster[i])
        }
    }
    fn read_b6(&mut self, s: u8) -> Result<u8> {
        Ok(self.phases[usize::from(s)])
    }
    fn write_b6(&mut self, s: u8, v: u8) {
        self.phases[usize::from(s)] = v;
        if s == 4 {
            self.raw[0xb6] = v;
        }
    }
    fn manual_mode(&mut self) -> Result<u8> {
        Ok(self.shared[3])
    }
    fn target_leaf447790(&mut self, s: Shot, rng: &mut OriginalRng) -> Result<()> {
        let f = self.pop("F:");
        assert_eq!(
            s.origin,
            [
                f[1].parse::<u16>().unwrap() as i16,
                f[2].parse::<u16>().unwrap() as i16
            ]
        );
        assert_eq!(usize::from(s.target_slot), f[3].parse::<usize>().unwrap());
        if !self.leaf_operations.is_empty() {
            target_leaf(self, s, rng)?;
            let a = self.pop("A:");
            assert_eq!(self.raw, arr::<192>(&a[1]));
            assert_eq!(self.w, arr::<44>(&a[2]));
            assert_eq!(self.shared, arr::<24>(&a[3]));
            let rr = bytes(&a[4]);
            for i in 0..5 {
                assert_eq!(
                    rng.words[i],
                    u32::from_le_bytes(rr[i * 4..i * 4 + 4].try_into().unwrap())
                );
            }
        } else {
            let a = self.pop("A:");
            self.raw = arr(&a[1]);
            self.w = arr(&a[2]);
            self.shared = arr(&a[3]);
            Self::apply_rng(rng, &a[4]);
        }
        self.fires += 1;
        Ok(())
    }
    fn direct_manual_leaf(&mut self, _: DirectShot, _: &mut OriginalRng) -> Result<()> {
        Err(Boundary::Missing("explicit direct manual leaf"))
    }
    fn observe(&mut self, e: Event) {
        match e {
            Event::Range { outside, .. } => {
                let p = self.pop("R:");
                assert_eq!(outside, p[1].parse::<u8>().unwrap() != 0);
            }
            Event::Gate(v) => {
                let p = self.pop("G:");
                assert_eq!(v, p[1].parse::<u8>().unwrap() != 0);
                assert_eq!(self.error, p[2].parse::<u8>().unwrap());
            }
            Event::Attempt { .. } => {}
        }
    }
}

impl LeafWorld for Replay {
    fn actor(&self) -> Actor {
        World::actor(self)
    }
    fn view(&mut self) -> Result<View> {
        Ok(View {
            camera: [0, 0],
            width: 1024,
            height: 1024,
        })
    }
    fn master_enabled(&mut self) -> Result<bool> {
        Ok(true)
    }
    fn sound_once(&mut self) -> Result<u8> {
        Ok(self.sound_once)
    }
    fn write_sound_once(&mut self, v: u8) {
        self.sound_once = v;
    }
    fn sound(&mut self, op: Sound) -> Result<()> {
        let q = self.leaf_pop("Q:");
        assert_eq!(
            q[1].parse::<u8>().unwrap(),
            match op {
                Sound::Wrapper27 => 27,
                Sound::Direct43WithCycle => 43,
            }
        );
        let d = self.leaf_pop("D:");
        self.sound_cycle = d[1].parse().unwrap();
        Ok(())
    }
    fn saved_facing(&mut self) -> Result<u8> {
        Ok(self.shared[14])
    }
    fn write_saved_facing(&mut self, v: u8) {
        self.shared[14] = v;
    }
    fn write_facing(&mut self, v: u8) {
        self.raw[0x39] = v;
    }
    fn field46(&mut self) -> Result<u16> {
        Ok(word(&self.raw, 0x46) as u16)
    }
    fn write_field46(&mut self, v: u16) {
        self.raw[0x46..0x48].copy_from_slice(&v.to_le_bytes());
    }
    fn scatter448c20(&mut self, s: u8, rng: &mut OriginalRng) -> Result<()> {
        let c = self.leaf_pop("C:");
        assert_eq!(s, c[1].parse::<u8>().unwrap());
        let t = self.leaf_pop("T:");
        self.raw = arr(&t[1]);
        Self::apply_rng(rng, &t[2]);
        Ok(())
    }
    fn target42(&mut self) -> Result<i16> {
        Ok(word(&self.raw, 0x42))
    }
    fn target44(&mut self) -> Result<i16> {
        Ok(word(&self.raw, 0x44))
    }
    fn speed79(&mut self) -> Result<u8> {
        Ok(self.raw[0x79])
    }
    fn payload65(&mut self) -> Result<u8> {
        Ok(self.raw[0x65])
    }
    fn hardware_token8(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.raw[8..12].try_into().unwrap()))
    }
    fn read_w(&mut self, hw: HardwareRef, p: u8) -> Result<u8> {
        assert_eq!(hw, HardwareRef::Slot(0));
        Ok(self.w[usize::from(p)])
    }
    fn write_w(&mut self, hw: HardwareRef, p: u8, v: u8) {
        assert_eq!(hw, HardwareRef::Slot(0));
        self.w[usize::from(p)] = v;
    }
    fn emit(&mut self, op: Emission, rng: &mut OriginalRng) -> Result<()> {
        let e = self.leaf_pop("E:");
        let (kind, origin, target, speed) = match op {
            Emission::Heavy {
                origin,
                target,
                speed,
                sample,
            } => {
                assert_eq!(sample, 29);
                ("H", origin, target, speed)
            }
            Emission::Flame {
                origin,
                target,
                speed,
            } => ("F", origin, target, speed),
            Emission::Bullet {
                origin,
                target,
                speed,
            } => ("B", origin, target, speed),
        };
        assert_eq!(kind, e[1]);
        assert_eq!(
            origin,
            [
                e[2].parse::<u16>().unwrap() as i16,
                e[3].parse::<u16>().unwrap() as i16
            ]
        );
        assert_eq!(
            target,
            [
                e[4].parse::<u16>().unwrap() as i16,
                e[5].parse::<u16>().unwrap() as i16
            ]
        );
        assert_eq!(speed, e[6].parse::<u8>().unwrap());
        assert_eq!(self.raw, arr::<192>(&e[7]));
        assert_eq!(self.w, arr::<44>(&e[8]));
        assert_eq!(self.shared, arr::<24>(&e[9]));
        let entry_rng = bytes(&e[10]);
        for i in 0..5 {
            assert_eq!(
                rng.words[i],
                u32::from_le_bytes(entry_rng[i * 4..i * 4 + 4].try_into().unwrap())
            );
        }
        let z = self.leaf_pop("Z:");
        assert_eq!(self.w[5], arr::<44>(&z[2])[5]);
        assert_eq!(self.w[6], arr::<44>(&z[2])[6]);
        self.raw = arr(&z[1]);
        self.w = arr(&z[2]);
        self.shared = arr(&z[3]);
        Self::apply_rng(rng, &z[4]);
        self.bullet_output = match z[5].parse::<u32>().unwrap() {
            0 => None,
            v => Some(v),
        };
        self.laser = z[6].parse().unwrap();
        self.bullets = bytes(&z[7]);
        self.sound_cycle = z[8].parse().unwrap();
        self.sound_once = z[9].parse().unwrap();
        Ok(())
    }
    fn write_laser622be6(&mut self, v: u8) {
        self.laser = v;
    }
    fn clear_bullet_output64d7b0(&mut self) {
        self.bullet_output = None;
    }
    fn bullet_output64d7b0(&mut self) -> Result<Option<u32>> {
        Ok(self.bullet_output)
    }
    fn increment_bullet_range11(&mut self, token: u32) -> Result<()> {
        let off = usize::try_from(token - 0x5b02c2).unwrap();
        assert_eq!(off % 28, 0);
        self.bullets[off + 11] = self.bullets[off + 11].wrapping_add(1);
        Ok(())
    }
}

#[test]
fn whole_native_calls_replay_only_the_declared_caller_projection() {
    let Ok(directory) = std::env::var("NATIVE_APC_REALTRIG_DIRECTORY") else {
        return;
    };
    for (dataset, expected_calls, expected_hash) in [
        ("V51-apc-realtrig.jsonl", 384, Some(4082334245u32)),
        (
            "V51-apc-controlled-realtrig.jsonl",
            384,
            Some(1128899081u32),
        ),
        ("V51-apc-leaf-ports-three.jsonl", 96, Some(3616920725u32)),
    ] {
        let path = std::path::Path::new(&directory).join(dataset);
        let text = std::fs::read_to_string(path).unwrap();
        let rows: Vec<Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(rows.len(), expected_calls * 2);
        let mut calls = 0;
        let mut shoots = 0;
        let mut hash = 2166136261u32;
        for pair in rows.chunks_exact(2) {
            let before = &pair[0];
            let after = &pair[1];
            let mut w = Replay::new();
            w.raw = arr(before["unit"].as_str().unwrap());
            w.w = arr(before["hardware"].as_str().unwrap());
            w.shared = arr(before["apcGlobals"].as_str().unwrap());
            w.phases = arr(before["allB6"].as_str().unwrap());
            w.target = arr(before["leader"].as_str().unwrap());
            w.roster = bytes(before["group"].as_str().unwrap())[19..40]
                .try_into()
                .unwrap();
            w.mp = before["mp"].as_bool().unwrap();
            w.error = before["errorE2"].as_u64().unwrap() as u8;
            let pp = bytes(before["probePoint"].as_str().unwrap());
            w.probe = [word(&pp, 0), word(&pp, 2)];
            w.operations = after["ops"]
                .as_str()
                .unwrap()
                .split(';')
                .filter(|x| !x.is_empty())
                .map(str::to_owned)
                .collect();
            if let Some(ops) = after["leafOps"].as_str() {
                w.leaf_operations = ops
                    .split(';')
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned)
                    .collect();
                w.sound_cycle = before["soundCycle"].as_u64().unwrap() as u32;
                w.sound_once = before["soundOnce"].as_u64().unwrap() as u8;
            }
            let words = std::array::from_fn(|i| before[format!("r{i}")].as_u64().unwrap() as u32);
            let mut rng = OriginalRng {
                words,
                multiplayer: w.mp,
            };
            let result = run(&mut w, &mut rng).unwrap();
            assert!(
                w.operations.is_empty(),
                "{} {}",
                before["scenario"],
                before["mp"]
            );
            assert_eq!(w.raw, arr::<192>(after["unit"].as_str().unwrap()));
            assert_eq!(w.w, arr::<44>(after["hardware"].as_str().unwrap()));
            assert_eq!(w.shared, arr::<24>(after["apcGlobals"].as_str().unwrap()));
            assert_eq!(w.phases, arr::<240>(after["allB6"].as_str().unwrap()));
            assert_eq!(w.error, after["errorE2"].as_u64().unwrap() as u8);
            assert_eq!(
                rng.words,
                std::array::from_fn(|i| after[format!("r{i}")].as_u64().unwrap() as u32)
            );
            let pp = bytes(after["probePoint"].as_str().unwrap());
            assert_eq!(w.probe, [word(&pp, 0), word(&pp, 2)]);
            assert_eq!(w.fires, usize::from(result.invoked_targets));
            assert!(w.leaf_operations.is_empty());
            if dataset == "V51-apc-leaf-ports-three.jsonl" {
                assert_eq!(w.sound_cycle, after["soundCycle"].as_u64().unwrap() as u32);
                assert_eq!(w.sound_once, after["soundOnce"].as_u64().unwrap() as u8);
                if !w.bullets.is_empty() {
                    assert_eq!(w.bullets, bytes(after["bullets"].as_str().unwrap()));
                }
            }
            shoots += w.fires;
            calls += 1;
            for v in w
                .raw
                .iter()
                .chain(w.w.iter())
                .chain(w.shared.iter())
                .chain(w.phases.iter())
            {
                hash = (hash ^ u32::from(*v)).wrapping_mul(16777619);
            }
            for v in rng.words {
                hash = (hash ^ v).wrapping_mul(16777619);
            }
        }
        println!(
            "{dataset} caller replay: calls={calls} leaf invocations={shoots} signature={hash}"
        );
        assert_eq!(calls, expected_calls);
        if let Some(h) = expected_hash {
            assert_eq!(hash, h);
        }
    }
}

#[test]
fn acquisition_gate_matches_96_actual_416160_calls() {
    let mut hash = 2166136261u32;
    let mut rows = Vec::new();
    for mp in 0..2 {
        for strategy in 0..4 {
            for sector in 0..3 {
                for busy in 0..2 {
                    for high in 0..2 {
                        let mut w = Replay::new();
                        w.mp = mp != 0;
                        w.strategy = (
                            if strategy & 1 != 0 { 8 } else { 0 },
                            if strategy & 2 != 0 { 1 } else { 0 },
                        );
                        w.busy = busy;
                        if high != 0 {
                            w.bounds[0] = [1001, 1001, 1002, 1002];
                        }
                        if sector != 0 {
                            w.bounds[1] = [0, 0, 64, 64];
                            w.sector_flags[1] = if sector == 2 { 2 } else { 0 };
                        }
                        let a = Actor {
                            position: [320, 320],
                            ..World::actor(&w)
                        };
                        let value = acquisition_blocked(&mut w, a).unwrap();
                        rows.push((u8::from(value), w.error));
                        hash = (hash ^ u32::from(value)).wrapping_mul(16777619);
                        hash = (hash ^ u32::from(w.error)).wrapping_mul(16777619);
                    }
                }
            }
        }
    }
    if let Ok(dir) = std::env::var("NATIVE_APC_REALTRIG_DIRECTORY") {
        let text =
            std::fs::read_to_string(std::path::Path::new(&dir).join("V49-apc-acquire-gate.jsonl"))
                .unwrap();
        let native: Vec<Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(native.len(), 96);
        for ((al, e), v) in rows.iter().zip(native) {
            assert_eq!(u64::from(*al), v["al"].as_u64().unwrap());
            assert_eq!(u64::from(*e), v["globalE2"].as_u64().unwrap());
        }
    }
    println!("416160/17400 96-call projection signature={hash}");
    assert_eq!(hash, 1621652681);
}

#[test]
fn hardware_tokens_select_dummy_only_for_nil_and_minus_one() {
    assert_eq!(hardware_ref(0), Ok(HardwareRef::Dummy));
    assert_eq!(hardware_ref(u32::MAX), Ok(HardwareRef::Dummy));
    for i in 0..240 {
        assert_eq!(
            hardware_ref(0x59e6c2 + i * 44),
            Ok(HardwareRef::Slot(i as u8))
        );
    }
    for token in [1, 0x59e6c1, 0x59e6c3, 0x59e6c2 + 240 * 44] {
        assert_eq!(hardware_ref(token), Err(Boundary::HardwareToken(token)));
    }
}

#[test]
fn raw_view_uses_signed_word_subtraction_and_inclusive_limits() {
    let v = View {
        camera: [32760, -32760],
        width: 320,
        height: 240,
    };
    assert!(v.contains([32760i16.wrapping_add(-48), (-32760i16).wrapping_add(272)]));
    assert!(!v.contains([32760i16.wrapping_add(-49), -32760]));
    assert!(!v.contains([32760i16.wrapping_add(353), -32760]));
}

#[test]
fn multiplayer_gate_never_needs_unavailable_group_or_cp() {
    struct Mp;
    impl GateWorld for Mp {
        fn multiplayer(&self) -> bool {
            true
        }
        fn strategy(&mut self, _: u8) -> Result<(u8, u8)> {
            panic!()
        }
        fn sector_bounds(&mut self, _: usize) -> Result<[i16; 4]> {
            panic!()
        }
        fn sector_flags24(&mut self, _: usize) -> Result<u8> {
            panic!()
        }
        fn current_group3d(&mut self) -> Result<u8> {
            panic!()
        }
        fn trail_error(&self) -> u8 {
            panic!()
        }
        fn write_trail_error(&mut self, _: u8) {
            panic!()
        }
    }
    let a = World::actor(&Replay::new());
    assert!(!acquisition_blocked(&mut Mp, Actor { group5f: 255, ..a }).unwrap());
}
#[test]
fn sector_probe_preserves_one_error_owner_and_second_pass() {
    let mut w = Replay::new();
    w.bounds[0] = [1001, 0, 1002, 1];
    w.bounds[1] = [21, 21, 22, 22];
    w.sector_flags[1] = 2;
    w.strategy = (8, 1);
    w.busy = 1;
    let a = Actor {
        position: [320, 320],
        ..World::actor(&w)
    };
    assert!(acquisition_blocked(&mut w, a).unwrap());
    assert_eq!(w.error, 2);
    w.bounds[1] = [-1; 4];
    assert_eq!(locate_sector(&mut w, a.position).unwrap(), None);
    assert_eq!(w.error, 2);
}
#[test]
fn reload_gate_preserves_all_side_and_member_storage() {
    let mut w = Replay::new();
    w.raw[0x7b] = 1;
    w.shared = [5; 24];
    w.phases = [13; 240];
    let mut rng = OriginalRng::seeded(4);
    let words = rng.words;
    assert_eq!(run(&mut w, &mut rng).unwrap(), Return::default());
    assert_eq!(w.shared, [5; 24]);
    assert_eq!(w.phases, [13; 240]);
    assert_eq!(rng.words, words);
}
#[test]
fn unavailable_cached_raw_target_keeps_the_already_written_slot() {
    let mut w = Replay::new();
    w.operations.push_back("S:61440:0:0".into());
    let mut rng = OriginalRng::seeded(1);
    assert_eq!(run(&mut w, &mut rng), Err(Boundary::Target(240)));
    assert_eq!(w.side_target(Side::Right), 240);
    assert_eq!(w.raw[0x7b], 0);
}

#[test]
fn only_ff_skips_a_scanner_winner_and_no_target_preserves_reload() {
    let mut w = Replay::new();
    for _ in 0..4 {
        w.operations.push_back("S:65287:0:0".into());
    }
    w.shared = [5; 24];
    w.phases = [13; 240];
    let mut rng = OriginalRng::seeded(10);
    let words = rng.words;
    assert_eq!(run(&mut w, &mut rng).unwrap(), Return::default());
    assert!(w.operations.is_empty());
    for s in [Side::Front, Side::Right, Side::Back, Side::Left] {
        assert_eq!(w.side_target(s), 255);
    }
    assert_eq!(w.raw[0x7b], 0);
    assert_eq!(w.phases, [13; 240]);
    assert_eq!(rng.words, words);
}

#[test]
fn unknown_roster_alias_retains_the_native_reload_prefix() {
    let mut w = Replay::new();
    w.raw[0x2c..0x2e].copy_from_slice(&320i16.to_le_bytes());
    w.raw[0x2e..0x30].copy_from_slice(&320i16.to_le_bytes());
    w.raw[0x30] = 32;
    w.raw[0x31] = 4;
    w.raw[0x7c] = 1;
    w.raw[0x78] = 80;
    w.target[0x2c..0x2e].copy_from_slice(&380i16.to_le_bytes());
    w.target[0x2e..0x30].copy_from_slice(&320i16.to_le_bytes());
    w.target[0x31] = 5;
    w.w[5] = 30;
    w.roster[0] = 240;
    for _ in 0..4 {
        for op in ["S:263:0:0", "R:0", "G:0:5"] {
            w.operations.push_back(op.into());
        }
    }
    let mut rng = OriginalRng::seeded(10);
    let words = rng.words;
    assert_eq!(run(&mut w, &mut rng), Err(Boundary::Member(240)));
    assert_eq!(w.raw[0x7b], 30);
    assert_eq!(w.probe, [392, 328]);
    assert_eq!(rng.words, words);
    assert_eq!(w.phases, [0; 240]);
}
