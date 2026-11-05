/// The wreck conversion latch is shared by the outer cleanup and any
/// nested UNIT removal. It is not the debris suppression or corpse flag.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct Force(pub u8);

pub(crate) struct Registers {
    pub(crate) count:i16,
    pub(crate) marker:u32,
    pub(crate) release_actor:Option<usize>,
}

impl Default for Registers {
    fn default()->Self { Self{count:2,marker:0,release_actor:None} }
}

impl Force {
    pub(crate) fn begin_conversion(&mut self,multiplayer:bool) -> bool {
        if multiplayer {self.0=1;}
        self.0!=0
    }
    pub(crate) fn finish_conversion(&mut self){self.0=0;}
}

pub(crate) trait World {
    type Boundary;
    fn fort_visibility(&self,team:u8)->u8;
    fn hide_fort(&mut self,team:u8);
    fn team_count(&self)->i16;
    fn set_team_count(&mut self,value:i16);
    fn set_team_marker(&mut self,value:u32);
    fn flags31(&self,slot:u8)->u8;
    fn remove_actor(&mut self,slot:u8,force:&mut Force)->Result<(),Self::Boundary>;
}

pub(crate) fn remove_team<W:World>(world:&mut W,team:u8,force:&mut Force)->Result<(),W::Boundary> {
    if world.fort_visibility(team)!=0 {
        world.hide_fort(team);
        let count=world.team_count().wrapping_sub(1);
        world.set_team_count(count);
        force.0=1;
        if count==1 {world.set_team_marker(u32::MAX);}
        for slot in 0..240 {
            let flags=world.flags31(slot);
            if flags&4!=0&&flags&3==team {world.remove_actor(slot,force)?;}
        }
    }
    force.0=0;
    Ok(())
}

#[cfg(test)]
#[path = "original_fort_team_cleanup_tests.rs"]
mod tests;
