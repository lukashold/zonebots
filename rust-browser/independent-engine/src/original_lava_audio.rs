//!406880->46e6d0(122), using the existing listener descriptor/UIword owner.
//!No extra Lava clock/gate/counter. World/CRT/draw/motion/hazard execute once.
use crate::{Game,original_lava::{SoundChance,View,visible}};
use super::Gate;
impl Game{
 pub(crate) fn emit_native_lava_audio(&mut self,chance:SoundChance){
  self.for_native_listeners(|g|{
   let context=if g.multiplayer{let team=usize::from(g.native_feedback.local_team);let Some(c)=g.native_listener_audio.hardware_views.get(team).and_then(Clone::clone)else{g.native_audio.opportunity_boundary=Some(super::spatial::Boundary::MissingListenerView(g.native_feedback.local_team));return;};Some(c)}else{None};
   let(camera,size)=context.as_ref().map(|c|(c.camera,c.size)).unwrap_or_else(||g.native_view.unwrap_or(([0,0],[(g.map.width*16)as i16,(g.map.height*16)as i16])));
   //Browser accepted views describe the full canvas: lower0, upper=size.
   //Nonzero original clip minima/lifecycle are an explicit caller boundary;
   //the pure View helper accepts them, but this transport does not invent them.
   if !visible(chance.point,View{camera,lower:[0,0],upper:size}){return;}
   let bank=g.native_audio_bank();let clock=g.native_feedback.ui_clock;let enabled=g.native_feedback.sound_enabled!=0;
   if let Some(sample)=g.native_audio.admit(bank,122,clock,Gate::Enabled,enabled){g.emit_admitted_native_sample(sample,None,context.as_ref().map(|c|c.generation.as_str()));}
  });
 }
}
#[cfg(test)]#[path="original_lava_audio_tests.rs"]mod tests;
