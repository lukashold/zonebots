//! Group aim reward. The member bytes are visited in their original order,
//! including duplicates; the low cooldown byte is written after that walk.

pub(crate) trait World {
    type Boundary;

    fn cooldown(&self) -> (u8, u8);
    fn set_count(&mut self, value: u8);
    fn member(&self, position: usize) -> Result<u8, Self::Boundary>;
    fn boost_aim(&mut self, slot: u8) -> Result<(), Self::Boundary>;
    fn set_cooldown(&mut self, value: u8);
}

pub(crate) fn apply<W: World>(world: &mut W) -> Result<(), W::Boundary> {
    let (cooldown, count) = world.cooldown();
    if cooldown != 0 || count as i8 >= 2 {
        return Ok(());
    }
    world.set_count(count.wrapping_add(1));
    for position in 0..21 {
        let slot = world.member(position)?;
        if slot != 255 {
            world.boost_aim(slot)?;
        }
    }
    world.set_cooldown(170);
    Ok(())
}

pub(crate) fn boost(value: u8) -> u8 {
    let aim = value as i8;
    let next = aim.wrapping_add(aim >> 2);
    if next < 0 { 127 } else { next as u8 }
}

/// Target reports are one byte per physical roster, including inactive rows.
/// Every match is cleared. An empty matching row cannot erase a prior credit.
pub(crate) fn clear_target_reports<'a>(
    groups: impl IntoIterator<Item = &'a mut [u8; 72]>,
    target: u8,
) -> Option<u8> {
    let mut credited = None;
    for row in groups {
        if row[59] == target {
            row[59] = 255;
            if row[19] != 255 {
                credited = Some(row[19]);
            }
        }
    }
    credited
}

struct Roster<'a> {
    bytes: &'a mut [u8; 72],
    units: &'a mut [crate::Unit],
}

impl World for Roster<'_> {
    type Boundary = std::convert::Infallible;

    fn cooldown(&self) -> (u8, u8) { (self.bytes[66], self.bytes[67]) }
    fn set_count(&mut self, value: u8) { self.bytes[67] = value; }
    fn member(&self, position: usize) -> Result<u8, Self::Boundary> { Ok(self.bytes[19 + position]) }
    fn boost_aim(&mut self, slot: u8) -> Result<(), Self::Boundary> {
        // The ordinary browser roster keeps only existing UNIT adapters.
        // This projection is separate from the retired raw/aliased slots.
        if let Some(combat) = self.units.iter_mut().rfind(|unit| unit.native_slot == Some(usize::from(slot)))
            .and_then(|unit| unit.original_combat.as_mut()) {
            combat.aim_skill = boost(combat.aim_skill);
        }
        Ok(())
    }
    fn set_cooldown(&mut self, value: u8) { self.bytes[66] = value; }
}

impl crate::Game {
    pub(crate) fn reward_native_group_aim(&mut self, index: usize) {
        let Some(group) = self.units[index].native_slot.and_then(|slot| self.native_group_rosters.members.get(slot))
            .and_then(Option::as_ref).map(|member| usize::from(member.group)) else { return; };
        let Some(row) = self.native_group_rosters.pool.get_mut(group) else { return; };
        let mut world = Roster { bytes: &mut row.roster, units: &mut self.units };
        match apply(&mut world) { Ok(()) => (), Err(impossible) => match impossible {} }
    }
}

#[cfg(test)]
#[path = "original_group_reward_tests.rs"]
mod tests;
