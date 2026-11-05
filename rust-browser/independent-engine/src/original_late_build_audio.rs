//!40e9d0's local584968 DWORD. The caller owns one World/reader execution;
//!this module cannot restore buildings, finish bridge jobs, allocate or use RNG.
use super::{State,Descriptor,Gate};use crate::Game;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Stage{Frame7,BuildComplete,BridgeComplete}
impl State{
 pub(crate) fn reset_late_build_mission(&mut self){self.late_build_clock=0;}
 pub(crate) fn late_build_opportunity(&mut self,stage:Stage,point:[i16;2],camera:[i16;2],size:[i16;2],bank:usize,clock:i16,master:bool)->Option<Descriptor>{
  if crate::original_feedback::outside_view(point,camera,size)||stage==Stage::Frame7&&self.late_build_clock!=0{return None;}
  //Bridge writes100 before its inline master/descriptor sequence. BUILD and
  //frame7 write after it. Neither completion reads the prior timer value.
  if stage==Stage::BridgeComplete{self.late_build_clock=100;}
  let result=if master{self.admit(bank,if stage==Stage::Frame7{44}else{46},clock,Gate::Direct,true)}else{None};
  if result.is_some(){self.effect_counter=self.effect_counter.wrapping_add(1)&3;}
  self.late_build_clock=100;result
 }
 pub(crate) fn end_late_build_reader(&mut self){if self.late_build_clock!=0{self.late_build_clock=self.late_build_clock.wrapping_sub(1);}}
}
impl Game{
 ///33ca0's reset, separate from import helpers, UI/device/session/settings.
 pub(crate) fn reset_native_late_build_audio_for_mission(&mut self){self.native_audio.reset_late_build_mission();self.native_listener_audio.guest_audio.reset_late_build_mission();}
 pub(crate) fn finish_native_late_build_audio_reader(&mut self){self.native_audio.end_late_build_reader();if self.multiplayer{self.native_listener_audio.guest_audio.end_late_build_reader();}}
 pub(crate) fn emit_native_late_build_audio(&mut self,stage:Stage,point:[i16;2]){
  self.for_native_listeners(|g|{
   let context=if g.multiplayer{let team=usize::from(g.native_feedback.local_team);let Some(c)=g.native_listener_audio.hardware_views.get(team).and_then(Clone::clone)else{g.native_audio.opportunity_boundary=Some(super::spatial::Boundary::MissingListenerView(g.native_feedback.local_team));return;};Some(c)}else{None};
   let(camera,size)=context.as_ref().map(|c|(c.camera,c.size)).unwrap_or_else(||g.native_view.unwrap_or(([0,0],[(g.map.width*16)as i16,(g.map.height*16)as i16])));let bank=g.native_audio_bank();let clock=g.native_feedback.ui_clock;let master=g.native_feedback.sound_enabled!=0;
   if let Some(sample)=g.native_audio.late_build_opportunity(stage,point,camera,size,bank,clock,master){g.emit_admitted_native_sample(sample,None,context.as_ref().map(|c|c.generation.as_str()));}
  });
 }
}
#[cfg(test)]#[path="original_late_build_audio_tests.rs"]mod tests;
