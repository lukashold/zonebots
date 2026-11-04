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
    ///Early native dispatch keeps a completed throw's final frames running
    ///after its inventory reaches zero, before browser target-validity checks.
    pub(crate) fn update_native_ground_fire(&mut self,index:usize)->Option<bool>{
        let u=self.units.get(index)?;if !self.map.original||u.native_phase35!=49&&u.native_order5c!= -5{return None;}if u.native_phase35==65{return self.update_native_ground_flame(index);}if !matches!(u.native_phase35,19|49){return None;}let point=self.native_ground_attack_point(index)?;let target=u.target.unwrap_or(usize::MAX);let grenade=u.native_phase35==49;Some(self.fire_native_ground(index,(f64::from(point[0])*2.,f64::from(point[1])*2.),target,grenade))
    }
    ///448e80 when the real native order is a ground attack. The phase selector
    ///has already run; this method never constructs or acquires a target UNIT.
    pub(crate) fn update_native_ground_flame(&mut self,index:usize)->Option<bool>{
        use super::original_ground_fire::{State as Ground,Action as GroundAction};
        let u=self.units.get(index)?;if !self.map.original||u.native_phase35!=65||u.native_order5c!= -5{return None;}let point=self.native_ground_attack_point(index)?;
        let mut state=Ground{frame:u.native_frame32,timer:u.native_timer3c,cooldown:if u.cooldown>0.{1}else{0},rate:u.original_combat.as_ref().map_or(80,|c|c.fire_rate),cycle:u.native_cyclea0,order:u.native_order5c,target:3456};let action=state.advance();
        {let u=&mut self.units[index];u.native_frame32=state.frame;u.native_timer3c=state.timer;u.native_cyclea0=state.cycle;}
        let made=match action{
            GroundAction::Fire=>{self.units[index].native_timer3c=0;let made=self.emit_native_ground_flame(index,point);self.units[index].native_timer3c=1;self.units[index].cooldown=f64::from(state.cooldown)/17.;made},
            GroundAction::GroundProbe=>{let valid=self.native_ground_target_available(index,point);if state.ground_result(valid){self.units[index].native_frame32=state.frame;}else{self.fail_native_group_order(index);self.clear_native_attack_paths(index);self.reset_native_idle(index);}false},
            GroundAction::RefreshOrder=>{self.units[index].native_order5c=state.order;if let Some(m)=self.units[index].native_slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.promotion.order=state.order as u8;}false},
            GroundAction::Wait|GroundAction::LiveContinuation=>false,
        };
        let(frame,timer,cycle)=(self.units[index].native_frame32,self.units[index].native_timer3c,self.units[index].native_cyclea0);if let Some(s)=self.units[index].native_fire.as_mut(){s.frame=frame;s.timer=i16::from(timer);s.cycle=cycle;}
        Some(made)
    }
    ///A ground callback consumes an already selected native phase. It cannot
    ///run44a900's enemy reporting or create a firing phase through a fake UNIT.
    pub(crate) fn advance_native_ground_fire_choice(&mut self,index:usize,target:usize,point:(f64,f64),grenade:bool)->bool{
        let u=&self.units[index];
        if super::stats(&u.kind).cls>0&&!grenade{
            self.ensure_native_hardware(index);let u=&self.units[index];let direction=super::native_math::direction((point.0/2.).floor()as i32-(u.x/2.).floor()as i32,(point.1/2.).floor()as i32-(u.y/2.).floor()as i32);let ready=u.cooldown<=0.;let h=self.units[index].native_hardware.as_mut().unwrap();let release=h.aim(direction,self.motion_tick&1!=0,ready);let turret=h.turret;self.units[index].aim=super::native_math::angle(turret);if super::stats(&self.units[index].kind).cls==2{self.units[index].angle=self.units[index].aim;}return release;
        }
        let phase=u.native_phase35;if !(grenade&&phase==49||!grenade&&phase==19&&u.native_order5c== -5){return false;}
        use super::original_ground_fire::{State as Ground,AuxiliaryAction};let mut state=Ground{frame:u.native_frame32,timer:u.native_timer3c,cooldown:if u.cooldown>0.{1}else{0},rate:u.original_combat.as_ref().map_or(80,|s|s.fire_rate),cycle:u.native_cyclea0,order:u.native_order5c,target:3456};let action=state.advance_auxiliary(phase);let u=&mut self.units[index];u.native_frame32=state.frame;u.native_timer3c=state.timer;u.native_order5c=state.order;if let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.promotion.order=state.order as u8;}u.native_fire=Some(State::acquired(target,phase,state.frame,state.timer));if action==AuxiliaryAction::Fire&&phase==49{u.cooldown=f64::from(state.cooldown)/17.;}if action==AuxiliaryAction::Idle{self.reset_native_idle(index);}action==AuxiliaryAction::Fire
    }
    pub(crate) fn advance_native_fire(&mut self,index:usize,target:&Unit)->bool {
        let u=&self.units[index];
        let grenade=original_grenades::eligible(&u.kind,u.grenades,&target.kind);
        self.advance_native_fire_choice(index,target,grenade)
    }
    pub(crate) fn advance_native_fire_choice(&mut self,index:usize,target:&Unit,grenade:bool)->bool {
        let u=&self.units[index];
        if u.native_fire.as_ref().is_none_or(|s|s.target!=target.id&&!s.reacquire) {
            let state=State::new(target.id,&u.kind,grenade);
            self.units[index].native_phase35=super::original_animation_game::fire_phase(state.mode,self.units[index].native_phase35);
            if state.mode!=Mode::Hardware{let(frame,timer)=state.native_walk_seed();self.units[index].native_frame32=frame;self.units[index].native_timer3c=timer;}self.units[index].native_idle=None;self.units[index].native_walk=None;
            self.units[index].native_fire=Some(state);self.refresh_native_aim(index,target);self.activate_native_combat_groups(index,target.id);self.observe_native_attack(index,target.id);
            if self.units[index].native_fire.as_ref().unwrap().mode!=Mode::Hardware{return false;}
        }
        if self.units[index].native_fire.as_ref().unwrap().reacquire {
            let desired=State::new(target.id,&self.units[index].kind,grenade);
            let state=self.units[index].native_fire.as_mut().unwrap();
            if state.mode!=desired.mode {*state=desired;}else{state.target=target.id;state.reacquire=false;}
            self.units[index].native_phase35=super::original_animation_game::fire_phase(self.units[index].native_fire.as_ref().unwrap().mode,self.units[index].native_phase35);
            self.refresh_native_aim(index,target);
            self.activate_native_combat_groups(index,target.id);
        }
        let u=&mut self.units[index];
        if u.native_fire.as_ref().unwrap().mode==Mode::Hardware{return self.aim_native_hardware(index,target);}
        let rate=u.original_combat.as_ref().map(|s|s.fire_rate).unwrap_or(80);
        if u.native_fire.as_ref().unwrap().mode==Mode::Flame{u.native_fire.as_mut().unwrap().cycle=u.native_cyclea0;}
        let action=u.native_fire.as_mut().unwrap().advance(u.cooldown<=0.,rate);let(frame,timer)=u.native_fire.as_ref().unwrap().native_walk_seed();u.native_frame32=frame;u.native_timer3c=timer;
        if u.native_fire.as_ref().unwrap().mode==Mode::Flame{u.native_cyclea0=u.native_fire.as_ref().unwrap().cycle;}
        if action==Action::Reacquire {
            u.native_fire.as_mut().unwrap().reacquire=true;
        }
        action==Action::Fire
    }
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Action {Wait,Fire,Reacquire}
impl State {
    ///44a900 overwrites phase/frame/timer without advancing its new handler.
    pub(crate) fn acquired(target:usize,phase:u8,frame:i16,timer:i8)->Self{let mode=match phase{57=>Mode::Rapid,65=>Mode::Flame,19=>Mode::Tough,71=>Mode::Laser,49=>Mode::Grenade,_=>Mode::Rifle};Self{target,mode,frame,timer:i16::from(timer),cycle:0,reacquire:false,frozen_aim:None}}
    pub(crate) fn native_walk_seed(&self)->(i16,i8){(self.frame,self.timer as i8)}
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
