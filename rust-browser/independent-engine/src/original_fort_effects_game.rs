//! Adapter for the existing world effect pool. The scene controller has to
//! admit and dispatch the bursts; ordinary building damage does not do so.

use crate::Game;
use super::{bridge_effects::{commit, globals, World}, effects::fort::{self, Burst}};

impl Game {
    pub(super) fn run_native_fort_burst_effects(&mut self, burst: Burst, base: [i16; 2], owner: u8) -> Option<usize> {
        self.native_world.as_ref()?;
        self.emit_native_spatial_audio(crate::original_audio::spatial::Chance::FortBurst {point:base});
        self.spawn_native_fort_burst_particles(burst, base, owner)
    }

    pub(super) fn spawn_native_fort_burst_particles(&mut self, burst: Burst, base: [i16; 2], owner: u8) -> Option<usize> {
        let state = self.native_world.as_mut()?;
        self.original_rng.multiplayer = self.multiplayer;
        let limit = state.effects.len();
        let mut g = globals(state);
        let made = fort::spawn(&mut World {state, rng: &mut self.original_rng, limit}, &mut g, burst, base, owner);
        commit(state, g);
        Some(made)
    }
}

#[cfg(test)]
#[path = "original_fort_effects_game_tests.rs"]
mod tests;
