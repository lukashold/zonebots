//! Ordered fort scene queue. The World owns the live context, effects,
//! selection, feedback and random streams. The queue owns only its 80 bytes.
#[path = "original_fort_scene_game.rs"]
mod game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub counter: i32,
    pub payload: u32,
    pub owner: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    Loss,
    Win,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    HaltMixer,
    Notice { kind: Notice, local_team: u8 },
    SelectFort(u16),
    RemoveTeam(u8),
    PanToFort { base: [i16; 2] },
    Disperse { center: [i16; 2], base_x: u16, reverse: bool },
    FirstBurst { base: [i16; 2] },
    SecondBurst { base: [i16; 2] },
    RetireFortTeam,
    MarkSceneDirty,
}

pub trait World {
    type Error;

    fn context(&self) -> Context;
    fn set_counter(&mut self, counter: i32);
    fn set_owner(&mut self, owner: u8);
    fn set_payload(&mut self, payload: u32);
    fn mixer_loaded(&self) -> bool;
    fn local_team(&self) -> u8;
    fn notice_pending(&self, kind: Notice) -> bool;
    fn fort_base(&mut self, owner: u8) -> Result<[i16; 2], Self::Error>;
    /// The boolean is the original AL continuation flag for pan, bursts and
    /// team retirement. A missing leaf stops at its call with prior writes.
    fn execute(&mut self, action: Action) -> Result<bool, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    Continue,
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefeatedFort { Team0, Team1 }

impl DefeatedFort {
    pub fn team(self) -> u8 { match self { Self::Team0 => 0, Self::Team1 => 1 } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionAction {
    CancelProductionSelection,
    MarkTeamActivity { team: u8, value: u8 },
    WriteTeamMarker { team: u8, value: u8 },
    ClearLocalDefeatFlag,
    ArmLocalDefeat,
    ApplyWorldWrites(DefeatedFort),
}

pub trait AdmissionWorld: World {
    fn team_activity(&self, team: u8) -> u8;
    fn apply_admission(&mut self, action: AdmissionAction) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission { Ineligible, Queued(usize), Full }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queue(pub [[u8; 20]; 4]);

impl Default for Queue {
    fn default() -> Self {
        let mut state = Self([[0; 20]; 4]);
        state.retire_for_mission();
        state
    }
}

impl Queue {
    /// Mission arena reset retires only the first DWORD of each record.
    pub(crate) fn retire_for_mission(&mut self) {
        for slot in 0..4 { self.store_counter(slot, -1); }
    }
    pub fn counter(&self, slot: usize) -> i32 {
        i32::from_le_bytes(self.0[slot][..4].try_into().unwrap())
    }

    pub(crate) fn store_counter(&mut self, slot: usize, counter: i32) {
        self.0[slot][..4].copy_from_slice(&counter.to_le_bytes());
    }

    /// The low-level allocator does not validate mission/owner admission.
    /// Four full entries leave every queue byte and the active gate unchanged.
    pub fn admit(&mut self, world: &mut impl World) -> Option<usize> {
        let slot = (0..4).find(|&slot| self.counter(slot) == -1)?;
        let Context { payload, owner, .. } = world.context();
        self.0[slot][8] = owner;
        self.store_counter(slot, 1);
        self.0[slot][4..8].copy_from_slice(&payload.to_le_bytes());
        world.set_counter(1);
        Some(slot)
    }

    /// Actual fort defeat admission. A full queue does not roll back any
    /// selection, team, terrain or BUILD writes that already took place.
    pub fn admit_defeat<W: AdmissionWorld>(&mut self, world: &mut W, side: DefeatedFort, cause: i8) -> Result<Admission, W::Error> {
        let team = side.team();
        if world.context().counter != 0 || matches!(world.team_activity(team), 0 | 2) {
            return Ok(Admission::Ineligible);
        }
        world.apply_admission(AdmissionAction::CancelProductionSelection)?;
        world.set_payload(i32::from(cause) as u32);
        world.set_owner(team + 1);
        world.set_counter(1);
        world.apply_admission(AdmissionAction::MarkTeamActivity {team,value:2})?;
        world.apply_admission(AdmissionAction::WriteTeamMarker {team,value:255})?;
        if world.local_team() == team {
            world.apply_admission(AdmissionAction::CancelProductionSelection)?;
            world.apply_admission(AdmissionAction::ClearLocalDefeatFlag)?;
            world.apply_admission(AdmissionAction::ArmLocalDefeat)?;
        }
        world.apply_admission(AdmissionAction::ApplyWorldWrites(side))?;
        Ok(self.admit(world).map_or(Admission::Full, Admission::Queued))
    }

    pub fn advance<W: World>(&mut self, world: &mut W) -> Result<Return, W::Error> {
        let saved = world.context();
        for slot in 0..4 {
            let counter = self.counter(slot);
            world.set_counter(counter);
            if counter == -1 {
                continue;
            }
            world.set_owner(self.0[slot][8]);
            world.set_payload(u32::from_le_bytes(self.0[slot][4..8].try_into().unwrap()));
            if counter == 0 {
                continue;
            }
            let counter = counter.wrapping_add(1);
            world.set_counter(counter);

            if counter == 2 {
                if world.mixer_loaded() {
                    world.execute(Action::HaltMixer)?;
                }
                let local_team = world.local_team();
                let kind = if i32::from(local_team as i8) + 1 == i32::from(world.context().owner as i8) {
                    Notice::Loss
                } else {
                    Notice::Win
                };
                if !world.notice_pending(kind) {
                    world.execute(Action::Notice { kind, local_team })?;
                }
            }

            let index = match world.context().owner {
                1 => 0,
                2 => 1,
                3 => 2,
                _ => 3,
            };
            world.execute(Action::SelectFort(index))?;
            if index == 1 {
                world.execute(Action::RemoveTeam(1))?;
            }

            let owner = world.context().owner;
            let base = world.fort_base(if matches!(owner, 2..=4) { owner } else { 1 })?;
            let terminal = match world.context().counter {
                2 => world.execute(Action::PanToFort { base })?,
                3 => {
                    world.execute(Action::Disperse {
                        center: base.map(|coordinate| coordinate.wrapping_add(160)),
                        base_x: base[0] as u16,
                        reverse: owner == 2,
                    })?;
                    false
                }
                15 => world.execute(Action::FirstBurst { base })?,
                16 => world.execute(Action::SecondBurst { base })?,
                17 => world.execute(Action::RetireFortTeam)?,
                170 => {
                    world.execute(Action::MarkSceneDirty)?;
                    true
                }
                _ => false,
            };
            self.store_counter(slot, world.context().counter);
            if terminal {
                self.store_counter(slot, -1);
                world.execute(Action::MarkSceneDirty)?;
                return Ok(Return::Terminal);
            }
        }
        world.set_owner(saved.owner);
        world.set_payload(saved.payload);
        world.set_counter(saved.counter);
        Ok(Return::Continue)
    }
}

#[cfg(test)]
#[path = "original_fort_scene_tests.rs"]
mod tests;
