use super::Notice;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct Lane {
    pub flag:i32,
    pub counter:u8,
    pub cursor:usize,
    pub owner:u8,
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct Radio { pub win:Lane, pub loss:Lane }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Speaker { pub flags31:u8, pub role:i8 }

pub(crate) trait World {
    type Boundary;
    fn scene_gate(&self)->i32;
    fn voice_hold(&self)->i32;
    fn set_voice_hold(&mut self,value:i32);
    fn physical_speaker(&mut self,slot:usize)->Result<Option<Speaker>,Self::Boundary>;
    fn announce(&mut self,kind:Notice,slot:usize)->Result<(),Self::Boundary>;
}

pub(crate) fn message(kind:Notice, random:u16)->u16 {
    match kind { Notice::Win=>0x0a3d+random%6, Notice::Loss=>0x0b43+random%3 }
}

impl Radio {
    fn lane_mut(&mut self,kind:Notice)->&mut Lane {
        match kind {Notice::Win=>&mut self.win,Notice::Loss=>&mut self.loss}
    }

    pub(crate) fn arm(&mut self,kind:Notice,owner:u8) {
        let lane=self.lane_mut(kind);
        if lane.flag==0 {
            *lane=Lane{flag:1,counter:if kind==Notice::Win {10}else{1},cursor:0,owner};
        }
    }

    pub(crate) fn advance<W:World>(&mut self,world:&mut W)->Result<(),W::Boundary> {
        self.advance_lane(world,Notice::Win)?;
        self.advance_lane(world,Notice::Loss)
    }

    fn advance_lane<W:World>(&mut self,world:&mut W,kind:Notice)->Result<(),W::Boundary> {
        let hold=world.voice_hold();
        if hold>0 { world.set_voice_hold((hold-2).max(0)); }
        let lane=self.lane_mut(kind);
        if world.scene_gate()>=169||lane.flag<=0 { return Ok(()); }
        match kind {
            Notice::Win if lane.counter as i8<=0 => return Ok(()),
            Notice::Loss => {
                lane.counter=lane.counter.wrapping_sub(1);
                if lane.counter as i8>=0 { return Ok(()); }
                lane.counter=17;
            }
            _ => ()
        }
        if world.voice_hold()!=0 { return Ok(()); }
        if kind==Notice::Loss&&lane.flag==2 {lane.flag=1;}
        loop {
            let slot=lane.cursor;
            let eligible=world.physical_speaker(slot)?.is_some_and(|actor|
                actor.role== -1&&actor.flags31&4!=0&&actor.flags31&3==lane.owner);
            if eligible {
                world.announce(kind,slot)?;
                lane.cursor+=1;
                if kind==Notice::Win {lane.counter=lane.counter.wrapping_sub(1);}
                return Ok(());
            }
            lane.cursor+=1;
            if lane.cursor>=240 {lane.flag=-1;return Ok(());}
        }
    }
}

#[cfg(test)]
#[path = "original_fort_radio_tests.rs"]
mod tests;

#[path = "original_fort_radio_game.rs"]
mod game;
