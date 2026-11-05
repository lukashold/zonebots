//! World writes after fort-scene admission. The caller owns the activity,
//! queue, production selector and mission-result transition.

use crate::{Game, original_dynamic_collision::Error as MaskError, original_world_contact as native};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Boundary {
    Team(u8),
    MissingWorld,
    Terrain(i16),
    Permission(u32),
}

impl Game {
    /// Terrain, permission tables, then the first eligible live BUILD record.
    pub(super) fn apply_native_fort_admission_world(&mut self, defeated_team: u8, base: [i16; 2]) -> Result<Option<usize>, Boundary> {
        if defeated_team > 1 { return Err(Boundary::Team(defeated_team)); }
        if self.native_world.is_none() { return Err(Boundary::MissingWorld); }
        let y = base[1].wrapping_add(i16::from(defeated_team) * 16);
        let origin = (((i32::from(y) * 8 - 249) & 0xff80) as i16).wrapping_add(base[0] >> 4);
        for dy in 0..11 {
            for dx in 0..10 {
                let cell = origin.wrapping_add(dy * 128 + dx);
                if cell < 0 || self.map.native_terrain_plane.get(cell as usize).is_none() {
                    return Err(Boundary::Terrain(cell));
                }
                let code = self.world_code(cell);
                if code & 127 == 16 { self.set_world_code(cell, (code & 128) | 19); }
            }
        }
        const DIRECTIONAL: [[u32; 6]; 2] = [
            [0x632800,0x632578,0x6324e8,0x6326e0,0x632848,0x632410],
            [0x632728,0x632530,0x632608,0x6325c0,0x6324a0,0x6323c8],
        ];
        const COMMON: [u32; 6] = [0x632650,0x6327b8,0x632770,0x632890,0x632698,0x632458];
        let code = 62 + u32::from(defeated_team);
        for address in DIRECTIONAL[usize::from(defeated_team)].into_iter().chain(COMMON) {
            match self.native_collision_masks.write(address + code, 255) {
                Ok(()) => {},
                Err(MaskError::MissingMaskByte(token)) => return Err(Boundary::Permission(token)),
                Err(MaskError::MissingTerrain(cell)) => return Err(Boundary::Terrain(cell)),
            }
        }
        let world = self.native_world.as_mut().ok_or(Boundary::MissingWorld)?;
        let point = base.map(|coordinate| coordinate.wrapping_add(16));
        let records: Vec<_> = world.buildings.iter().map(|b| b.record).collect();
        let slot = native::first_building(&records, point, true, false).map(|(slot, _)| slot);
        if let Some(slot) = slot {
            world.buildings[slot].record.kind = -1;
            if world.trail.known_builds[slot] { world.trail.builds[slot][4] = 255; }
        }
        Ok(slot)
    }
}

#[cfg(test)]
#[path = "original_fort_admission_game_tests.rs"]
mod tests;
