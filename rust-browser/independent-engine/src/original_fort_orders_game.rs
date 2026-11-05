use super::{Actor,Bounds,Move,World,disperse,force_move};
use crate::{Game,original_orders_game::Event};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Boundary {
    PhysicalUnit(u8),
    Selection(u8),
    Order(u8),
}

struct Local<'a> {
    game:&'a mut Game,
    bounds:Bounds,
}

fn unit_index(game:&Game, slot:u8) -> Option<usize> {
    game.units.iter().rposition(|u|u.native_slot==Some(usize::from(slot)))
}

impl Game {
    /// The local forced branch of the router. It skips multi-group broadcast
    /// but still passes the classified waypoint through ordinary dispersal.
    pub(crate) fn accept_native_fort_forced_move(&mut self, command:Move) -> Result<(),Boundary> {
        let index=unit_index(self,command.slot).ok_or(Boundary::PhysicalUnit(command.slot))?;
        let event=Event::classified(1,command.target,3);
        let event=self.disperse_native_human_move(index,event).ok_or(Boundary::Selection(command.slot))?;
        if !self.record_native_order_event(index,event) { return Err(Boundary::Order(command.slot)); }
        Ok(())
    }

    pub(crate) fn issue_native_fort_door_local_move(&mut self, command:Move) -> Result<(),Boundary> {
        let mut world=Local {game:self,bounds:Bounds::fort([0,0])};
        force_move(&mut world,command)
    }

    pub(crate) fn issue_native_fort_local_sweep(&mut self, base:[i16;2], selected:u16, reverse:bool) -> Result<(),Boundary> {
        let bounds=Bounds::fort(base);
        let mut world=Local {game:self,bounds};
        disperse(&mut world,bounds,selected,reverse)
    }
}

impl World for Local<'_> {
    type Boundary=Boundary;
    fn bounds(&self)->Bounds { self.bounds }
    fn store_bounds(&mut self,bounds:Bounds) { self.bounds=bounds; }
    fn actor(&self,slot:u8)->Option<Actor> {
        let u=self.game.units.get(unit_index(self.game,slot)?)?;
        let m=self.game.native_group_rosters.members[usize::from(slot)].as_ref()?;
        let x=if u.native_transport.as_ref().is_some_and(|t|t.phase==crate::original_transport::Phase::Inside){10000}else{(u.x/2.).floor() as i16};
        Some(Actor{flags31:m.flags31,class:u.native_class7c as i8,position:[x,(u.y/2.).floor() as i16]})
    }
    fn store_order_arguments(&mut self, command:Move) { self.game.native_order_arguments=[command.target[0],command.target[1],3]; }
    fn update_order_symbol(&mut self,slot:u8,symbol:u16)->Result<(),Boundary> {
        let index=unit_index(self.game,slot).ok_or(Boundary::PhysicalUnit(slot))?;
        self.game.record_native_selection_icon(index,symbol);
        Ok(())
    }
    fn set_forced_order(&mut self,_enabled:bool) {
        // This port binds only the explicitly selected local forced branch.
        // The router's shared bypass/forced bytes are tested by the raw port.
    }
    fn route_order(&mut self,command:Move)->Result<(),Boundary> { self.game.accept_native_fort_forced_move(command) }
}

#[cfg(test)]
#[path = "original_fort_orders_game_tests.rs"]
mod tests;
