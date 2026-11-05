use super::Game;
use crate::original_fort_scene::cleanup::{self,Force,World};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boundary { MissingWorld, Team(u8), UnboundActor(u8) }

struct GameWorld<'a>(&'a mut Game);

impl World for GameWorld<'_> {
    type Boundary=Boundary;
    fn fort_visibility(&self,team:u8)->u8{self.0.native_display.fort_visible[usize::from(team)]}
    fn hide_fort(&mut self,team:u8){self.0.native_display.fort_visible[usize::from(team)]=0;}
    fn team_count(&self)->i16{self.0.native_world.as_ref().unwrap().fort_cleanup.count}
    fn set_team_count(&mut self,value:i16){self.0.native_world.as_mut().unwrap().fort_cleanup.count=value;}
    fn set_team_marker(&mut self,value:u32){self.0.native_world.as_mut().unwrap().fort_cleanup.marker=value;}
    fn flags31(&self,slot:u8)->u8{
        self.0.native_group_rosters.members[usize::from(slot)].as_ref().map_or(0,|u|u.flags31)
    }
    fn remove_actor(&mut self,slot:u8,force:&mut Force)->Result<(),Boundary>{
        let index=self.0.units.iter().rposition(|u|u.native_slot==Some(usize::from(slot))).ok_or(Boundary::UnboundActor(slot))?;
        self.0.native_remove_fort_actor(index,force).then_some(()).ok_or(Boundary::UnboundActor(slot))
    }
}

impl Game {
    pub(crate) fn remove_native_fort_team_actors(&mut self,team:u8)->Result<(),Boundary>{
        self.native_world.as_ref().ok_or(Boundary::MissingWorld)?;
        if team>=4{return Err(Boundary::Team(team));}
        self.original_rng.multiplayer=self.multiplayer;
        cleanup::remove_team(&mut GameWorld(self),team,&mut Force::default())
    }
}

#[cfg(test)]
#[path="original_fort_team_cleanup_game_tests.rs"]
mod tests;
