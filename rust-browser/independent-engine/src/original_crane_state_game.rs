use crate::Game;
impl Game {
    ///Called immediately after the existing physical41b210 generation event.
    ///This receipt survivesworldNone; neither import nor kind changes call it.
    pub(crate) fn record_native_crane_ctor(&mut self, index: usize) {
        let Some(u) = self.units.get(index) else {
            return;
        };
        let Some(slot) = u.native_slot else {
            return;
        };
        let Some(generation) = self.native_upstream_reactions.generation(slot) else {
            return;
        };
        self.native_cranes
            .constructed(slot, generation, u.id, u.kind == "crane");
    }
    pub(crate) fn note_native_crane_initializer(&mut self, index: usize) {
        let Some(u) = self.units.get(index) else {
            return;
        };
        let Some(slot) = u.native_slot else {
            return;
        };
        if let Some(generation) = self.native_upstream_reactions.generation(slot) {
            self.native_cranes.initializer57ff0(slot, generation, u.id);
        }
    }
    pub(crate) fn rebind_native_crane_identity(&mut self, slot: usize, old: usize, new: usize) {
        if let Some(generation) = self.native_upstream_reactions.generation(slot) {
            if self.native_cranes.rebind(slot, generation, old, new)
                && self
                    .units
                    .iter()
                    .any(|u| u.native_slot == Some(slot) && u.id == new && u.kind != "crane")
            {
                if let Some(a) = self.native_cranes.slots[slot]
                    .as_mut()
                    .and_then(|r| r.auxiliary.as_mut())
                {
                    a.observed57ff0_86ff = false;
                }
            }
        }
    }
    pub(crate) fn native_crane_work9a(
        &self,
        slot: usize,
        entity: usize,
    ) -> Result<i8, crate::original_display::crane::Boundary> {
        if self
            .units
            .iter()
            .any(|u| u.native_slot == Some(slot) && u.id == entity && u.order == "repairBridge")
            && !self
                .units
                .iter()
                .position(|u| u.native_slot == Some(slot) && u.id == entity)
                .is_some_and(|i| self.owns_native_bridge_repair(i))
        {
            return Err(crate::original_display::crane::Boundary::MissingWork9a);
        }
        self.native_upstream_reactions
            .generation(slot)
            .and_then(|g| self.native_cranes.phase9a(slot, g, entity))
            .map(|v| v as i8)
            .ok_or(crate::original_display::crane::Boundary::MissingWork9a)
    }
}
