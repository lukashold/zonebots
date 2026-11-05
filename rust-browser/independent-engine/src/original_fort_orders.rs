#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bounds {
    pub(crate) fn fort(base: [i16;2]) -> Self {
        Self { left:i32::from(base[0]), top:i32::from(base[1]),
            right:i32::from(base[0].wrapping_add(160)), bottom:i32::from(base[1].wrapping_add(160)) }
    }
    fn includes(self, point: [i16;2]) -> bool {
        let [x,y] = point.map(i32::from);
        self.left <= x && x <= self.right && self.top <= y && y <= self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Actor {
    pub flags31: u8,
    pub class: i8,
    pub position: [i16;2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Move {
    pub slot: u8,
    pub target: [i16;2],
}

pub(crate) trait World {
    type Boundary;

    fn bounds(&self) -> Bounds;
    fn store_bounds(&mut self, bounds: Bounds);
    fn actor(&self, physical_slot: u8) -> Option<Actor>;
    fn store_order_arguments(&mut self, command: Move);
    fn update_order_symbol(&mut self, slot: u8, symbol: u16) -> Result<(),Self::Boundary>;
    fn set_forced_order(&mut self, enabled: bool);
    fn route_order(&mut self, command: Move) -> Result<(),Self::Boundary>;
}

/// Both the door ejection and the fort rectangle use the same move-command
/// prefix. Routing decides between a local order and multiplayer staging.
pub(crate) fn force_move<W: World>(world: &mut W, command: Move) -> Result<(),W::Boundary> {
    world.store_order_arguments(command);
    world.update_order_symbol(command.slot,0x0abb)?;
    world.set_forced_order(true);
    world.route_order(command)?;
    world.set_forced_order(false);
    Ok(())
}

pub(crate) fn disperse<W: World>(world: &mut W, initial: Bounds, selected_fort: u16, reverse: bool) -> Result<(),W::Boundary> {
    let target = [((initial.right.wrapping_sub(initial.left)>>1).wrapping_add(initial.left)) as i16,
        (initial.top as i16).wrapping_add(if reverse {300}else{-300})];
    let defeated_team = selected_fort as u8 & 3;
    world.store_bounds(initial);
    for slot in 0..240 {
        let Some(actor) = world.actor(slot) else { continue; };
        if actor.flags31&4 == 0 || actor.flags31&3 == defeated_team || actor.class == 2 || !world.bounds().includes(actor.position) {
            continue;
        }
        // The destination is fixed before scanning, but the next rectangle
        // check sees the live bound registers after each call to the router.
        force_move(world,Move {slot,target})?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "original_fort_orders_tests.rs"]
mod tests;

#[path = "original_fort_order_queue.rs"]
pub(crate) mod queue;

#[path = "original_fort_orders_game.rs"]
pub(crate) mod game;
