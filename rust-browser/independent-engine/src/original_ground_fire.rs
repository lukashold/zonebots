//!448e80's Phase65 body. Raw A0 survives acquisitions;50 is the original
//!target discriminator, not a browser entity. World callbacks are explicit.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action{Wait,Fire,GroundProbe,LiveContinuation,RefreshOrder}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum AuxiliaryAction{Wait,Fire,Idle,RefreshOrder,LiveContinuation}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct State{pub frame:i16,pub timer:i8,pub cooldown:u8,pub rate:u8,pub cycle:u8,pub order:i8,pub target:i32}
impl State{
 ///457270(49) and57400(19), after their real ground phase selection.
 pub fn advance_auxiliary(&mut self,phase:u8)->AuxiliaryAction{
  let before=self.timer;self.timer=self.timer.wrapping_sub(1);
  if phase==49{if self.timer!=0{return AuxiliaryAction::Wait;}self.frame=self.frame.wrapping_add(1);self.timer=3;if self.frame==3{self.cooldown=100u8.wrapping_sub(self.rate);return AuxiliaryAction::Fire;}return if self.frame>3{AuxiliaryAction::Idle}else{AuxiliaryAction::Wait};}
  if phase!=19||before>=0{return AuxiliaryAction::Wait;}self.frame=self.frame.wrapping_add(1);if self.frame==2{self.timer=2;return AuxiliaryAction::Wait;}if self.frame>2{self.frame=0;if self.order== -5{self.order=5;return AuxiliaryAction::RefreshOrder;}return AuxiliaryAction::LiveContinuation;}
  if self.cooldown!=0{self.frame=self.frame.wrapping_sub(1);return AuxiliaryAction::Wait;}self.timer=0;self.cooldown=100u8.wrapping_sub(self.rate);AuxiliaryAction::Fire
 }
 pub fn advance(&mut self)->Action{
  self.timer=self.timer.wrapping_sub(1);if self.timer>=0{return Action::Wait;}
  self.frame=self.frame.wrapping_add(1);self.timer=1;if self.frame<2{return Action::Wait;}
  if self.frame==2{if self.cooldown!=0{self.frame=self.frame.wrapping_sub(1);return Action::Wait;}self.cooldown=100u8.wrapping_sub(self.rate);return Action::Fire;}
  self.cycle=self.cycle.wrapping_add(1)&3;if self.cycle==0{self.timer=(100i16.wrapping_sub(i16::from(self.rate)).wrapping_mul(8)).min(127)as i8;}
  self.frame=0;if self.order== -5{if self.target!=3456{self.order=5;return Action::RefreshOrder;}return Action::GroundProbe;}
  self.frame=1;Action::LiveContinuation
 }
 ///Called after448750; false requires45a9b0,52500,44cac0 in that order.
 pub fn ground_result(&mut self,available:bool)->bool{if !available{return false;}if self.cycle!=3{self.frame=1;}true}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_8640_native_flame_callbacks_match_raw_cycle_cooldown_timer_signs_and_ground_revalidation(){
  let(mut count,mut hash)=(0,2166136261u32);
  for frame in[-1,0,1,2,3,32767]{for timer in[-128,-1,0,1,2,127]{for cd in[0,1]{for rate in[0,80,98,101,255]{for cycle in 0..4{for order in[-5,0,5]{for available in[false,true]{
   let mut s=State{frame,timer,cooldown:cd,rate,cycle,order,target:3456};let action=s.advance();let trace=match action{Action::Wait|Action::RefreshOrder=>0,Action::Fire=>1,Action::GroundProbe=>if s.ground_result(available){2}else{2345},Action::LiveContinuation=>5};let shot_timer=if action==Action::Fire{0}else{-999};
   for n in[i32::from(frame),i32::from(timer),i32::from(cd),i32::from(rate),i32::from(cycle),i32::from(order),i32::from(available),i32::from(s.frame),i32::from(s.timer),i32::from(s.cooldown),i32::from(s.cycle),i32::from(s.order),trace,shot_timer]{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}}}
  assert_eq!((count,hash),(8640,1226331653),"real448e80;56840/448750/45a9b0/52500/44cac0 are explicit boundaries; live targetFF selects44cac0");
 }
 #[test]fn live_manual_target_requests_command_refresh_only_after_the_current_burst(){let mut s=State{frame:2,timer:0,cooldown:0,rate:80,cycle:2,order:-5,target:7};assert_eq!(s.advance(),Action::RefreshOrder);assert_eq!((s.frame,s.cycle,s.order),(0,3,5));}
 #[test]fn all_2160_native_auxiliary_ground_callbacks_match_release_idle_and_command_refresh(){
  let(mut count,mut hash)=(0,2166136261u32);
  for phase in[19,49]{for frame in[-1,0,1,2,3,32767]{for timer in[-128,-1,0,1,2,127]{for cd in[0,1]{for rate in[0,80,98,101,255]{for order in[-5,0,5]{
   let mut s=State{frame,timer,cooldown:cd,rate,cycle:0,order,target:3456};let action=s.advance_auxiliary(phase);let trace=match action{AuxiliaryAction::Wait|AuxiliaryAction::RefreshOrder=>0,AuxiliaryAction::Fire=>1,AuxiliaryAction::Idle=>5,AuxiliaryAction::LiveContinuation=>65};let shot_timer=if action==AuxiliaryAction::Fire{if phase==49{3}else{0}}else{-999};let shot_cd=if action==AuxiliaryAction::Fire{if phase==49{i32::from(s.cooldown)}else{i32::from(cd)}}else{-999};
   for n in[i32::from(phase),i32::from(frame),i32::from(timer),i32::from(cd),i32::from(rate),i32::from(order),i32::from(s.frame),i32::from(s.timer),i32::from(s.cooldown),i32::from(s.order),trace,shot_timer,shot_cd]{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}}
  assert_eq!((count,hash),(2160,2778678529),"real457270/457400; projectile/44cac0 explicit,58500 returnsFFFF");
 }
}
