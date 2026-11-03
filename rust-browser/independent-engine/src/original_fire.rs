//! Confirmed infantry attack-state counters. Selection0044a900; handlers
//! 00449e70,00448aa0,00448950,00448e80,00457400,00457270. No inverse-rate
//! shortcut is used for the animation windup. Target acquisition is external.
use serde::{Serialize,Deserialize};
use super::{Game,Unit,original_grenades};

#[derive(Clone,Copy,Serialize,Deserialize,Debug,PartialEq,Eq)]
#[serde(rename_all="camelCase")]
pub enum Mode {Rifle,Rapid,Laser,Flame,Tough,Grenade,Hardware}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct State {
    pub target:usize,pub mode:Mode,pub frame:i16,
    timer:i16,cycle:u8,
    #[serde(skip)] pub(crate) reacquire:bool,
    #[serde(skip)] pub(crate) frozen_aim:Option<(i32,i32)>,
}

impl Game {
    pub(crate) fn advance_native_fire(&mut self,index:usize,target:&Unit)->bool {
        let u=&self.units[index];
        let grenade=original_grenades::eligible(&u.kind,u.grenades,&target.kind);
        if u.native_fire.as_ref().is_none_or(|s|s.target!=target.id&&!s.reacquire) {
            let state=State::new(target.id,&u.kind,grenade);
            self.units[index].native_fire=Some(state);self.refresh_native_aim(index,target);return false;
        }
        if self.units[index].native_fire.as_ref().unwrap().reacquire {
            let desired=State::new(target.id,&self.units[index].kind,grenade);
            let state=self.units[index].native_fire.as_mut().unwrap();
            if state.mode!=desired.mode {*state=desired;}else{state.target=target.id;state.reacquire=false;}
            self.refresh_native_aim(index,target);
        }
        let u=&mut self.units[index];
        let rate=u.original_combat.as_ref().map(|s|s.fire_rate).unwrap_or(80);
        let action=u.native_fire.as_mut().unwrap().advance(u.cooldown<=0.,rate);
        if action==Action::Reacquire {
            u.native_fire.as_mut().unwrap().reacquire=true;
        }
        action==Action::Fire
    }
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Action {Wait,Fire,Reacquire}
impl State {
    pub fn new(target:usize,kind:&str,grenade:bool)->Self {
        let (mode,timer)=if grenade{(Mode::Grenade,5)}else{match kind {
            "grunt"|"sniper"=>(Mode::Rifle,3),"psycho"=>(Mode::Rapid,1),
            "laser"=>(Mode::Laser,1),"pyro"=>(Mode::Flame,1),
            "tough"=>(Mode::Tough,20),_=>(Mode::Hardware,0),
        }};
        Self{target,mode,frame:0,timer,cycle:0,frozen_aim:None,reacquire:false}
    }
    pub fn advance(&mut self,ready:bool,fire_rate:u8)->Action {
        match self.mode {
            Mode::Hardware=>if ready{Action::Fire}else{Action::Wait},
            Mode::Rifle=>{
                self.timer=(self.timer as i8).wrapping_sub(1)as i16;if self.timer!=0{return Action::Wait;}
                self.frame+=1;self.timer=3;
                if self.frame==4 {
                    if !ready{self.frame=3;self.timer=1;Action::Wait}else{Action::Fire}
                }else if self.frame>=5 {
                    // Reacquiring the same live target while already in state5
                    // starts at frame3/countdown1 (0044a93b), not frame0.
                    self.frame=3;self.timer=1;Action::Reacquire
                }else{Action::Wait}
            },
            Mode::Rapid=>{
                self.timer=(self.timer as i8).wrapping_sub(1)as i16;if self.timer!=0{return Action::Wait;}
                self.frame+=1;self.timer=1;
                if self.frame==1 {
                    if ready{Action::Fire}else{self.frame=0;Action::Wait}
                }else{self.frame=0;Action::Reacquire}
            },
            Mode::Laser=>{
                self.timer=(self.timer as i8).wrapping_sub(1)as i16;if self.timer!=0{return Action::Wait;}
                self.timer=3;
                if self.frame==2{self.frame=0;self.timer=1;Action::Reacquire}
                else if !ready{self.timer=1;Action::Wait}
                else{self.frame+=1;if self.frame==2{Action::Fire}else{Action::Wait}}
            },
            Mode::Flame=>{
                self.timer=(self.timer as i8).wrapping_sub(1)as i16;if self.timer>=0{return Action::Wait;}
                self.frame+=1;self.timer=1;
                if self.frame==2 {
                    if ready{Action::Fire}else{self.frame=1;Action::Wait}
                }else if self.frame>2 {
                    self.cycle=(self.cycle+1)&3;self.frame=1;
                    if self.cycle==0{self.timer=(100-i16::from(fire_rate))*8;self.timer=self.timer.min(127);}
                    Action::Reacquire
                }else{Action::Wait}
            },
            Mode::Tough=>{
                let before=self.timer;self.timer=(self.timer as i8).wrapping_sub(1)as i16;if before>=0{return Action::Wait;}
                self.frame+=1;
                if self.frame==2{self.timer=2;Action::Wait}
                else if self.frame>2{self.frame=0;self.timer=20;Action::Reacquire}
                else if ready{self.timer=0;Action::Fire}
                else{self.frame-=1;Action::Wait}
            },
            Mode::Grenade=>{
                self.timer=(self.timer as i8).wrapping_sub(1)as i16;if self.timer!=0{return Action::Wait;}
                self.frame+=1;self.timer=3;
                if self.frame==3{Action::Fire}
                else if self.frame>3{self.frame=0;self.timer=5;Action::Reacquire}
                else{Action::Wait}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_release_matches_each_native_counter_comparison() {
        for (kind,grenade,expected) in [("grunt",false,12),("sniper",false,12),("psycho",false,1),
            ("laser",false,4),("pyro",false,4),("tough",false,22),("grunt",true,11)] {
            let mut s=State::new(1,kind,grenade);let first=(1..100).find(|_|s.advance(true,80)==Action::Fire);
            assert_eq!(first,Some(expected),"{kind}, grenade={grenade}");
        }
    }
    #[test]
    fn rifle_waits_at_frame_three_for_cooldown_without_repeating_its_windup() {
        let mut s=State::new(1,"grunt",false);
        for _ in 0..30{assert_ne!(s.advance(false,80),Action::Fire);}
        assert_eq!(s.frame,3);assert_eq!(s.advance(true,80),Action::Fire);
        for _ in 0..3{assert_ne!(s.advance(false,80),Action::Fire);}
        assert_eq!(s.frame,3);assert_eq!(s.advance(true,80),Action::Fire);
    }
}
