//! Sole persistent UNIT9a owner. Physical construction, not world import or
//! renderer identity, establishes a known byte. `Crane` is a transient kernel
//! view; legacy auxiliary values never constitute a complete raw work context.
use super::crane::{Crane, Write96};
use std::ops::{Deref, DerefMut};
#[path = "original_crane_state_game.rs"]
mod game;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub epoch: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knowledge96 {
    Tainted,
    Fresh41b210,
    Mouth44fe18,
    Completion44fe77,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field96 {
    pub value: u8,
    pub knowledge: Knowledge96,
}
impl Field96 {
    fn fresh() -> Self {
        Self {
            value: 0,
            knowledge: Knowledge96::Fresh41b210,
        }
    }
    fn tainted(value: u8) -> Self {
        Self {
            value,
            knowledge: Knowledge96::Tainted,
        }
    }
    pub fn is_known(self) -> bool {
        self.knowledge != Knowledge96::Tainted
    }
}
/// No live legacy caller in this slice admits the complete input readset.
/// Numeric equality, DTO storage and a historical86 receipt cannot grant it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority96 {
    TaintedLegacy,
    ProvenReachedReadset,
}
#[derive(Clone, Debug)]
pub struct Auxiliary {
    pub target: Option<usize>,
    pub frame86: i8,
    pub interval87: i8,
    pub delay82: i16,
    pub saved: [i16; 2],
    pub approach_b1: Option<u8>,
    /// Historical observation only. Legacy frame changes preserve this receipt;
    /// it is NOT proof that the current86 value/readset is source-known.
    pub observed57ff0_86ff: bool,
}
impl Auxiliary {
    fn legacy() -> Self {
        Self {
            target: None,
            frame86: -1,
            interval87: 0,
            delay82: 0,
            saved: [0; 2],
            approach_b1: None,
            observed57ff0_86ff: false,
        }
    }
    pub fn tainted(&self) -> bool {
        true
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub identity: Identity,
    pub entity: usize,
    pub work9a: u8,
    pub work9a_known: bool,
    pub field96: Field96,
    pub auxiliary: Option<Auxiliary>,
}
impl Deref for Record {
    type Target = Auxiliary;
    fn deref(&self) -> &Auxiliary {
        self.auxiliary
            .as_ref()
            .expect("phase-only slot has no work auxiliary")
    }
}
impl DerefMut for Record {
    fn deref_mut(&mut self) -> &mut Auxiliary {
        self.auxiliary
            .as_mut()
            .expect("phase-only slot has no work auxiliary")
    }
}
impl Record {
    fn phase(identity: Identity, entity: usize, work9a: u8, known: bool) -> Self {
        Self {
            identity,
            entity,
            work9a,
            work9a_known: known,
            // A phase-only warm record cannot infer a constructor96 receipt.
            field96: Field96::tainted(0),
            auxiliary: None,
        }
    }
    ///Compatibility values for a typed crane only. These are explicitly tainted
    ///and cannot be read as source-known96/saved/B1/period defaults.
    fn legacy_view(&self) -> Crane {
        let a = self
            .auxiliary
            .as_ref()
            .expect("phase-only slot cannot become a work view");
        Crane {
            entity: self.entity,
            target: a.target,
            work9a: self.work9a,
            work9a_known: self.work9a_known,
            frame86: a.frame86,
            mouth96: self.field96.value,
            interval87: a.interval87,
            delay82: a.delay82,
            saved: a.saved,
            approach_b1: a.approach_b1,
        }
    }
    pub(crate) fn kernel_view(&self) -> Crane {
        self.legacy_view()
    }
    fn store_view(&mut self, c: Crane) {
        let observed57ff0_86ff = self
            .auxiliary
            .as_ref()
            .is_some_and(|a| a.observed57ff0_86ff);
        self.entity = c.entity;
        self.work9a = c.work9a;
        self.work9a_known = c.work9a_known;
        self.auxiliary = Some(Auxiliary {
            target: c.target,
            frame86: c.frame86,
            interval87: c.interval87,
            delay82: c.delay82,
            saved: c.saved,
            approach_b1: c.approach_b1,
            observed57ff0_86ff,
        });
        // field96 is not copied from this bulk DTO. Its reached writer effect
        // already commits at the source stage, even if Common later fails.
    }
    fn initializer57ff0(&mut self) {
        let mut a = self.auxiliary.take().unwrap_or_else(Auxiliary::legacy);
        a.target = None; //Actual28=2084 is not a BUILD/Bridge binding.
        a.frame86 = -1;
        a.observed57ff0_86ff = true;
        self.auxiliary = Some(a);
    }
    #[cfg(test)]
    pub(crate) fn fixture_phase(identity: Identity, entity: usize, value: u8, known: bool) -> Self {
        Self::phase(identity, entity, value, known)
    }
}
pub struct State {
    pub epoch: u64,
    pub slots: Vec<Option<Record>>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            epoch: 0,
            slots: vec![None; 240],
        }
    }
}
impl State {
    pub fn clear_for_mission(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.slots.fill(None);
    }
    pub fn key(&self, generation: u64) -> Identity {
        Identity {
            epoch: self.epoch,
            generation,
        }
    }
    pub fn constructed(&mut self, slot: usize, generation: u64, entity: usize, crane: bool) {
        let key = self.key(generation);
        if self
            .slots
            .get(slot)
            .and_then(Option::as_ref)
            .is_some_and(|r| r.identity == key)
        {
            return;
        }
        let mut r = Record::phase(key, entity, 0, true);
        r.field96 = Field96::fresh();
        if crane {
            r.initializer57ff0();
        }
        if let Some(s) = self.slots.get_mut(slot) {
            *s = Some(r);
        }
    }
    pub fn bound(&self, slot: usize, generation: u64, entity: usize) -> Option<&Record> {
        self.slots
            .get(slot)?
            .as_ref()
            .filter(|r| r.identity == self.key(generation) && r.entity == entity)
    }
    pub fn phase9a(&self, slot: usize, generation: u64, entity: usize) -> Option<u8> {
        self.bound(slot, generation, entity)
            .filter(|r| r.work9a_known)
            .map(|r| r.work9a)
    }
    pub fn write96(
        &mut self,
        slot: usize,
        identity: Identity,
        entity: usize,
        effect: Write96,
        authority: Authority96,
    ) -> bool {
        let epoch = self.epoch;
        let Some(r) = self
            .slots
            .get_mut(slot)
            .and_then(Option::as_mut)
            .filter(|r| identity.epoch == epoch && r.identity == identity && r.entity == entity)
        else {
            return false;
        };
        let knowledge = match effect {
            Write96::Mouth { before, .. }
                if authority == Authority96::ProvenReachedReadset
                    && r.field96.is_known()
                    && r.field96.value == before =>
            {
                Knowledge96::Mouth44fe18
            }
            Write96::CompletionZero if authority == Authority96::ProvenReachedReadset => {
                Knowledge96::Completion44fe77
            }
            _ => Knowledge96::Tainted,
        };
        r.field96 = Field96 {
            value: effect.value(),
            knowledge,
        };
        true
    }
    pub fn rebind(&mut self, slot: usize, generation: u64, old: usize, new: usize) -> bool {
        let key = self.key(generation);
        let Some(r) = self
            .slots
            .get_mut(slot)
            .and_then(Option::as_mut)
            .filter(|r| r.identity == key && r.entity == old)
        else {
            return false;
        };
        r.entity = new;
        true
    }
    pub fn initializer57ff0(&mut self, slot: usize, generation: u64, entity: usize) {
        self.ensure_unknown(slot, generation, entity);
        if let Some(r) = self.slots.get_mut(slot).and_then(Option::as_mut) {
            r.initializer57ff0();
        }
    }
    fn ensure_unknown(&mut self, slot: usize, generation: u64, entity: usize) {
        let key = self.key(generation);
        //A renderer mismatch cannot erase or reassign a physically known byte.
        if self
            .slots
            .get(slot)
            .and_then(Option::as_ref)
            .is_some_and(|r| r.identity == key)
        {
            return;
        }
        if let Some(s) = self.slots.get_mut(slot) {
            *s = Some(Record::phase(key, entity, 0, false));
        }
    }
    ///No constructor is inferred here. The prior fallback's work values remain
    ///tainted, including an unavailable phase; only a real writer can know9a.
    pub fn view_for_legacy_crane(&mut self, slot: usize, generation: u64, entity: usize) -> Crane {
        self.ensure_unknown(slot, generation, entity);
        if self.slots[slot]
            .as_ref()
            .is_some_and(|r| r.entity != entity)
        {
            //Preserve the old physical owner; this is only the previous
            //unbound legacy continuation, never a constructor or a rebind.
            let mut temporary = Record::phase(self.key(generation), entity, 0, false);
            temporary.auxiliary = Some(Auxiliary::legacy());
            return temporary.legacy_view();
        }
        let r = self.slots[slot].as_mut().unwrap();
        if r.auxiliary.is_none() {
            r.auxiliary = Some(Auxiliary::legacy());
        }
        r.legacy_view()
    }
    pub fn store(&mut self, slot: usize, generation: u64, c: Crane) {
        let key = self.key(generation);
        if let Some(r) = self
            .slots
            .get_mut(slot)
            .and_then(Option::as_mut)
            .filter(|r| r.identity == key && r.entity == c.entity)
        {
            r.store_view(c);
        }
    }
}
#[cfg(test)]
#[path = "original_crane_field96_tests.rs"]
mod field96_tests;
#[cfg(test)]
#[path = "original_crane_state_tests.rs"]
mod tests;
