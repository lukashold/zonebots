use super::{Action, Admission, AdmissionAction, AdmissionWorld, Context, DefeatedFort, Notice, Queue, Return, World};

struct Probe {
    context: Context,
    bases: [[i16; 2]; 4],
    local: u8,
    mixer: bool,
    notices: [u8; 2],
    note_values: [u8; 2],
    dirty: bool,
    selected: u16,
    halt_count: u8,
    calls: Vec<[i32; 3]>,
    activities: [u8; 2],
    team_markers: [u8; 4],
    local_defeat: u8,
    admission_actions: Vec<AdmissionAction>,
}

impl Probe {
    fn new(counter: i32, local: u8, mixer: bool) -> Self {
        Self {
            context: Context { counter, payload: 0x12345678, owner: 91 },
            bases: [[400, 500], [100, 200], [700, 800], [1000, 1100]],
            local, mixer, notices: [0; 2], note_values: [0; 2], dirty: false,
            selected: 0x3344, halt_count: 0, calls: Vec::new(),
            activities: [17,19], team_markers: 0x11223344u32.to_le_bytes(), local_defeat:77,
            admission_actions: Vec::new(),
        }
    }
}

impl AdmissionWorld for Probe {
    fn team_activity(&self, team: u8) -> u8 { self.activities[usize::from(team)] }
    fn apply_admission(&mut self, action: AdmissionAction) -> Result<(), Self::Error> {
        self.admission_actions.push(action);
        match action {
            AdmissionAction::MarkTeamActivity {team,value} => self.activities[usize::from(team)] = value,
            AdmissionAction::WriteTeamMarker {team,value} => self.team_markers[usize::from(team)] = value,
            AdmissionAction::ArmLocalDefeat => self.local_defeat = 1,
            _ => {},
        }
        Ok(())
    }
}

impl World for Probe {
    type Error = &'static str;

    fn context(&self) -> Context { self.context }
    fn set_counter(&mut self, counter: i32) { self.context.counter = counter; }
    fn set_owner(&mut self, owner: u8) { self.context.owner = owner; }
    fn set_payload(&mut self, payload: u32) { self.context.payload = payload; }
    fn mixer_loaded(&self) -> bool { self.mixer }
    fn local_team(&self) -> u8 { self.local }
    fn notice_pending(&self, kind: Notice) -> bool { self.notices[usize::from(kind == Notice::Win)] != 0 }
    fn fort_base(&mut self, owner: u8) -> Result<[i16; 2], Self::Error> {
        self.bases.get(usize::from(owner).wrapping_sub(1)).copied().ok_or("base")
    }
    fn execute(&mut self, action: Action) -> Result<bool, Self::Error> {
        let counter = self.context.counter;
        let call = match action {
            Action::HaltMixer => { self.halt_count += 1; Some([0x12346000, -1, 0]) },
            Action::Notice { kind, local_team } => {
                assert_eq!(local_team, self.local);
                let index = usize::from(kind == Notice::Win);
                self.notices[index] = 1;
                self.note_values[index] = [1, 10][index];
                None
            }
            Action::SelectFort(index) => { self.selected = index; None }
            Action::RemoveTeam(team) => Some([0x4230c0, i32::from(team), counter]),
            Action::PanToFort { base } => Some([0x4229e0, i32::from(base[0]), counter]),
            // The intercepted original recorded the base in caller registers;
            // it did not record the rectangle's four stack inputs.
            Action::Disperse { base, reverse } => {
                assert_eq!(reverse, self.context.owner == 2);
                Some([0x456480, i32::from(base[0] as u16), i32::from(base[1])])
            }
            Action::FirstBurst { base } => Some([0x422e00, i32::from(base[0]), i32::from(base[1])]),
            Action::SecondBurst { base } => Some([0x422a60, i32::from(base[0]), i32::from(base[1])]),
            Action::RetireFortTeam => Some([0x4229a0, i32::from(self.fort_base(self.context.owner)?[0]), counter]),
            Action::MarkSceneDirty => { self.dirty = true; None }
        };
        if let Some(call) = call { self.calls.push(call); }
        Ok(false)
    }
}

fn parse(raw: &str) -> Queue {
    assert_eq!(raw.len(), 160);
    Queue(std::array::from_fn(|slot| std::array::from_fn(|i| {
        let n = 2 * (20 * slot + i);
        u8::from_str_radix(&raw[n..n + 2], 16).unwrap()
    })))
}
fn fill(q: &mut Queue, slot: usize, counter: i32, payload: u32, owner: u8) {
    q.0[slot][..4].copy_from_slice(&counter.to_le_bytes());
    q.0[slot][4..8].copy_from_slice(&payload.to_le_bytes());
    q.0[slot][8] = owner;
}

#[test]
#[ignore = "private executed GOG fort queue projections; supply NATIVE_FORT_QUEUE_REPLAY_DIR"]
fn original_single_slot_controller_contexts_and_leaf_order() {
    let mut count = 0;
    for row in trace("single.txt").lines() {
        let v: Vec<i32> = row.split_whitespace().map(|n| n.parse().unwrap()).collect();
        let slot = v[1] as usize;
        let mut world = Probe::new(0x55667788, v[3] as u8, v[5] != 0);
        let mut q = Queue::default();
        fill(&mut q, slot, v[4], 0x11223344 + slot as u32, v[2] as u8);
        let ret = q.advance(&mut world).unwrap();
        assert_eq!((0..4).map(|i| q.counter(i)).collect::<Vec<_>>(), &v[6..10], "row {}", v[0]);
        assert_eq!(i32::from(ret == Return::Terminal), v[10]);
        assert_eq!(world.context, Context {counter: v[11], payload: v[12] as u32, owner: v[13] as u8});
        let actual = [world.notices[0], world.notices[1], world.note_values[0], world.note_values[1], u8::from(world.dirty)];
        assert_eq!(actual.map(i32::from), &v[14..19]);
        assert_eq!(i32::from(world.selected), v[19]);
        assert_eq!(world.calls.into_iter().flatten().collect::<Vec<_>>(), &v[20..], "row {}", v[0]);
        count += 1;
    }
    assert_eq!(count, 512);
}

#[test]
#[ignore = "private executed GOG fort queue projections; supply NATIVE_FORT_QUEUE_REPLAY_DIR"]
fn original_first_free_writes_keep_all_other_queue_bytes() {
    let mut count = 0;
    for row in trace("admission.txt").lines() {
        let v: Vec<_> = row.split_whitespace().collect();
        let scenario: u32 = v[0].parse().unwrap();
        let mut q = parse(v[2]);
        let mut world = Probe::new(0x55667788, 0, false);
        world.context.payload = 0x12340000 + scenario;
        world.context.owner = v[1].parse().unwrap();
        assert_eq!(q.admit(&mut world).is_none(), v[4] == "1");
        assert_eq!(q, parse(v[3]));
        assert_eq!(world.context.counter, v[5].parse::<i32>().unwrap());
        count += 1;
    }
    assert_eq!(count, 64);
}

#[test]
#[ignore = "private executed GOG fort queue projections; supply NATIVE_FORT_QUEUE_REPLAY_DIR"]
fn original_coupled_callbacks_match_all_queue_bytes_and_controller_context() {
    let mut world = Probe::new(0, 0, true);
    let mut q = Queue::default();
    let mut count = 0;
    for row in trace("whole.txt").lines() {
        let v: Vec<_> = row.split_whitespace().collect();
        let scene: usize = v[0].parse().unwrap();
        let tick: usize = v[1].parse().unwrap();
        if tick == 0 {
            world = Probe::new(0, v[2].parse().unwrap(), true);
            world.bases = [[320, 370], [100, 200], [540, 540], [760, 710]];
            for slot in 0..4 {
                for b in 0..20 { q.0[slot][b] = (scene * 23 + (slot * 20 + b) * 17) as u8; }
                let counter = if scene < 32 { [1, 14, 15, 16][slot] } else if slot == scene % 4 { 169 } else { 15 + slot as i32 };
                fill(&mut q, slot, counter, 0x11000000 + slot as u32, 1 + ((slot + scene) % 4) as u8);
            }
        }
        let previous_halts = world.halt_count;
        let terminal = q.advance(&mut world).unwrap() == Return::Terminal;
        assert_eq!(terminal, v[3] == "1");
        assert_eq!(world.halt_count - previous_halts, v[4].parse::<u8>().unwrap());
        assert_eq!(q, parse(v[5]), "scene {scene}, tick {tick}");
        let f: Vec<u32> = v[6..].iter().map(|x| x.parse().unwrap()).collect();
        assert_eq!(world.context, Context {counter:f[0] as i32,payload:f[1],owner:f[2] as u8});
        assert_eq!([world.notices[0],world.notices[1],world.note_values[0],world.note_values[1]].map(u32::from), &f[3..7]);
        assert_eq!(u32::from(world.selected), f[8]);
        if terminal { assert!(world.dirty); }
        count += 1;
    }
    assert_eq!(count, 136);
}

#[test]
#[ignore = "private original fort admission controls; supply NATIVE_FORT_ADMISSION_TRACE"]
fn original_defeat_admission_gates_and_full_queue_keep_the_exact_prefix() {
    use std::io::{BufRead, BufReader};
    let path = std::env::var_os("NATIVE_FORT_ADMISSION_TRACE").expect("NATIVE_FORT_ADMISSION_TRACE");
    let rows = BufReader::new(std::fs::File::open(path).unwrap());
    let mut totals = [0;3];
    for row in rows.lines() {
        let v: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        let n = |field: &str| v[field].as_i64().unwrap();
        let scenario = n("scenario") as usize;
        let mut q = Queue(std::array::from_fn(|slot| std::array::from_fn(|i| (scenario*23+(slot*20+i)*17) as u8)));
        for slot in 0..4 { q.store_counter(slot, if n("occupiedMask") & (1 << slot) == 0 {-1} else {7+slot as i32}); }
        let side = if n("side")==0 { DefeatedFort::Team0 } else { DefeatedFort::Team1 };
        let mut w = Probe::new(n("gateBefore") as i32,n("local") as u8,false);
        w.activities[usize::from(side.team())] = n("visibleBefore") as u8;
        let result = q.admit_defeat(&mut w, side, n("context") as i8).unwrap();
        let result_index = match result {
            Admission::Ineligible => { assert!(w.admission_actions.is_empty()); 0 }
            Admission::Queued(slot) => {
                assert_eq!(slot,(0..4).find(|i|n("occupiedMask")&(1<<i)==0).unwrap());
                1
            }
            Admission::Full => { assert_eq!(n("occupiedMask"),15); 2 }
        };
        if result_index!=0 {
            assert_eq!(w.admission_actions.first(),Some(&AdmissionAction::CancelProductionSelection));
            assert_eq!(w.admission_actions.last(),Some(&AdmissionAction::ApplyWorldWrites(side)));
        }
        totals[result_index] += 1;
        assert_eq!(q,parse(v["queue"].as_str().unwrap()),"scenario {scenario}");
        assert_eq!(w.context,Context {counter:n("gate") as i32,payload:n("activeContext") as u32,owner:n("activeOwner") as u8});
        assert_eq!(w.activities,std::array::from_fn(|i|v["visible"][i].as_u64().unwrap() as u8));
        assert_eq!(u32::from_le_bytes(w.team_markers),n("teamMarker") as u32);
        assert_eq!(w.local_defeat,n("flag4d02ef") as u8);
    }
    assert_eq!(totals,[320,56,8]);
}

fn trace(name: &str) -> String {
    let dir = std::path::PathBuf::from(std::env::var_os("NATIVE_FORT_QUEUE_REPLAY_DIR").expect("NATIVE_FORT_QUEUE_REPLAY_DIR"));
    std::fs::read_to_string(dir.join(name)).unwrap()
}
