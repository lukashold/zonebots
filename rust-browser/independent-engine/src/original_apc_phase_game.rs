//! Joint manual/automatic B6 binding. This does not activate the pure APC
//! scanner/emitter kernel. Existing legacy weapon and target adapters remain.
use crate::{Game,original_apc_automatic::Boundary as Missing,
    original_path_contact::game::Boundary};
impl Game {
    pub(crate) fn native_apc_phase_group(&self, index: usize) -> Result<u8,Boundary> {
        let slot=self.units.get(index).and_then(|u|u.native_slot)
            .filter(|&s|s<240).ok_or(Boundary::ApcPhase(Missing::Missing("actor slot")))?;
        let group=self.native_group_rosters.members[slot].as_ref()
            .ok_or(Boundary::ApcPhase(Missing::Missing("actor group")))?.group;
        if group>=240{return Err(Boundary::ApcPhase(Missing::Group(group)));}
        Ok(group)
    }
    pub(crate) fn native_apc_phase_member(&self, group:u8, index:usize) -> Result<u8,Boundary> {
        if group>=240{return Err(Boundary::ApcPhase(Missing::Group(group)));}
        if index>=21{return Err(Boundary::ApcPhase(Missing::Missing("roster index outside 21")));}
        Ok(self.native_group_rosters.pool[usize::from(group)].roster[19+index])
    }
    pub(crate) fn advance_native_apc_phase(&mut self, member:u8) -> Result<u8,Boundary> {
        if member>=240{return Err(Boundary::ApcPhase(Missing::Member(member)));}
        let slot=usize::from(member);
        let generation=self.native_upstream_reactions.generation(slot)
            .ok_or(Boundary::ApcPhase(Missing::Missing("member generation")))?;
        self.native_cranes.advance_phase_b6(slot,generation)
            .ok_or(Boundary::ApcPhase(Missing::Missing("physical B6 receipt")))
    }
}
#[cfg(test)]
#[path="original_apc_phase_tests.rs"]
mod tests;
