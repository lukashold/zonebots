/// Source stage 17 changes both independent team byte arrays. The palette
/// row is a selector for the later rendering pass, not a delay or result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Registers {
    pub scene_dirty: u8,
    pub defeated_owner: u8,
    pub palette_row: i8,
    pub palette_cursor: u8,
}

impl Default for Registers {
    fn default() -> Self { Self{scene_dirty:0,defeated_owner:0,palette_row:-1,palette_cursor:0} }
}

impl Registers {
    pub(crate) fn start_mission(&mut self) {
        self.scene_dirty=0;
        self.palette_row=-1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OwnerBoundary(pub u8);

pub(crate) fn retire(
    owner: u8, activity: &mut [u8;4], visibility: &mut [u8;4], registers: &mut Registers,
) -> Result<bool,OwnerBoundary> {
    let Some(team) = owner.checked_sub(1).filter(|&team| team < 4).map(usize::from) else {
        return Err(OwnerBoundary(owner));
    };
    activity[team] = 0;
    visibility[team] = 0;
    registers.scene_dirty = 1;
    registers.defeated_owner = owner;
    registers.palette_row = 20;
    registers.palette_cursor = 255;
    Ok(false)
}

#[cfg(test)]
#[path = "original_fort_end_tests.rs"]
mod tests;
