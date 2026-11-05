use crate::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanBoundary {
    Selection(usize),
    Group(u8),
}

impl Game {
    /// The original selection word is physical slot plus one. The active
    /// camera is left alone; the caller receives the two destination words.
    pub(crate) fn prepare_native_fort_pan(&mut self, base: [i16; 2], view_mode: i32) -> Result<[i16; 2], PanBoundary> {
        if let Some(slot) = self.native_selected_slot {
            let member = self.native_group_rosters.members.get(slot).and_then(Option::as_ref).ok_or(PanBoundary::Selection(slot))?;
            if member.role <= 0 {
                let roster = self.native_group_rosters.pool.get_mut(usize::from(member.group)).ok_or(PanBoundary::Group(member.group))?;
                roster.roster[9] &= !2;
            }
            self.native_selected_slot = None;
        }
        self.native_selection.insertion = 255;
        let [dx, dy] = if view_mode == 2 { [128,64] } else { [64,32] };
        Ok([base[0].wrapping_sub(dx), base[1].wrapping_sub(dy)])
    }
}

#[cfg(test)]
#[path = "original_fort_scene_game_tests.rs"]
mod tests;
