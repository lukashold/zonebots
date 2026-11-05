//! Source audio opportunities only. No world, RNG, contact, allocator or emitter
//! is accessible to the evaluator. 4d18ee/4a6f4e live in the existing AudioState.
use super::{State,Descriptor,Gate};
use crate::Game;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Boundary{MissingListenerView(u8),MissingEffectRecord(usize),InvalidCycle(u8)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Chance{
 Blast420200{point:[i16;2],bounds:[i16;2],mask4f:u8,quiet:bool},
 Bridge41f870{point:[i16;2]},Owner4Expiry3{point:[i16;2]},FortBurst{point:[i16;2]},Removal6{point:[i16;2]},Crate6{point:[i16;2]},
 FortDebrisWater4{point:[i16;2]},FragmentWater10{point:[i16;2]},FragmentMetal10{point:[i16;2]},FallingWater7{point:[i16;2]},FallingMetal7{point:[i16;2]},
}
#[cfg(test)]#[path="original_spatial_audio_tests.rs"]mod tests;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Policy{MasterDirect,Enabled}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct Request{pub sample:i16,pub policy:Policy}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub(crate) struct Plan{pub request:Option<Request>,pub gate:Option<u8>,pub cycle:Option<u8>}
fn visible(p:[i16;2],camera:[i16;2],size:[i16;2],double:bool)->bool{let camera=if double{camera.map(|v|v.wrapping_mul(2))}else{camera};!crate::original_feedback::outside_view(p,camera,size)}
impl Chance{pub fn point(self)->[i16;2]{match self{Self::Blast420200{point,..}|Self::Bridge41f870{point}|Self::Owner4Expiry3{point}|Self::FortBurst{point}|Self::Removal6{point}|Self::Crate6{point}|Self::FortDebrisWater4{point}|Self::FragmentWater10{point}|Self::FragmentMetal10{point}|Self::FallingWater7{point}|Self::FallingMetal7{point}=>point}}}
impl State{
 ///Candidate request and post-call byte writes; descriptor admission occurs
 ///once in apply_opportunity before these postwrites. Tests can observe the
 ///recorded native wrapper boundary without pretending it executed a mixer.
 pub(crate) fn opportunity_plan(&self,chance:Chance,camera:[i16;2],size:[i16;2])->Result<Plan,Boundary>{
  let p=chance.point();let mut out=Plan::default();let mut request=|sample,policy|out.request=Some(Request{sample,policy});
  match chance{
   Chance::Blast420200{bounds,mask4f,quiet,..}=>{if p[1]< -32||p[0]>bounds[0]||p[1]>bounds[1]||mask4f!=0||quiet||self.opportunity_gate!=0||!visible(p,camera,size,false){return Ok(out);}let sample=*crate::original_fragment_blast::SAMPLES.get(usize::from(self.explosion_cycle)).ok_or(Boundary::InvalidCycle(self.explosion_cycle))?;request(sample,Policy::Enabled);out.cycle=Some(self.explosion_cycle.wrapping_add(1)&7);out.gate=Some(1);}
   Chance::Bridge41f870{..}=>{if !visible(p,camera,size,false){return Ok(out);}let sample=*crate::original_fragment_blast::SAMPLES.get(usize::from(self.explosion_cycle)).ok_or(Boundary::InvalidCycle(self.explosion_cycle))?;request(sample,Policy::MasterDirect);out.cycle=Some(self.explosion_cycle.wrapping_add(1)&7);}
   Chance::Owner4Expiry3{..}|Chance::FortBurst{..}|Chance::Removal6{..}=>{if self.opportunity_gate!=0||!visible(p,camera,size,false){return Ok(out);}request(if matches!(chance,Chance::Removal6{..}){6}else{1},Policy::MasterDirect);out.gate=Some(1);}
   Chance::Crate6{..}=>{if !visible(p,camera,size,true){return Ok(out);}if self.opportunity_gate==0{request(30,Policy::MasterDirect);}out.gate=Some(1);}
   Chance::FortDebrisWater4{..}=>{if visible(p,camera,size,false){request(9,Policy::MasterDirect);}}
   Chance::FragmentWater10{..}=>{if self.opportunity_gate==0&&visible(p,camera,size,false){request(8,Policy::MasterDirect);}}
   Chance::FallingWater7{..}=>{if visible(p,camera,size,false){request(8,Policy::MasterDirect);}}
   Chance::FragmentMetal10{..}|Chance::FallingMetal7{..}=>{if visible(p,camera,size,false){request(11,Policy::Enabled);}}
  }Ok(out)
 }
 pub(crate) fn finish_opportunity(&mut self,plan:Plan){if let Some(g)=plan.gate{self.opportunity_gate=g;}if let Some(c)=plan.cycle{self.explosion_cycle=c;}}
 pub(crate) fn apply_opportunity(&mut self,chance:Chance,camera:[i16;2],size:[i16;2],bank:usize,clock:i16,master:bool)->Option<Descriptor>{
  self.opportunity_boundary=None;let plan=match self.opportunity_plan(chance,camera,size){Ok(p)=>p,Err(e)=>{self.opportunity_boundary=Some(e);return None;}};
  let result=plan.request.and_then(|r|match r.policy{Policy::MasterDirect if !master=>None,Policy::MasterDirect=>{let d=self.admit(bank,r.sample,clock,Gate::Direct,master);if d.is_some(){self.effect_counter=self.effect_counter.wrapping_add(1)&3;}d},Policy::Enabled=>self.admit(bank,r.sample,clock,Gate::Enabled,master)});
  self.finish_opportunity(plan);result
 }
}
impl Game{
 pub(crate) fn begin_native_spatial_audio_main(&mut self){self.native_audio.opportunity_gate=0;if self.multiplayer{self.native_listener_audio.guest_audio.opportunity_gate=0;}}
 pub(crate) fn record_native_audio_boundary(&mut self,error:Boundary){self.native_audio.opportunity_boundary=Some(error);if self.multiplayer{self.native_listener_audio.guest_audio.opportunity_boundary=Some(error);}}
 pub(crate) fn emit_native_spatial_audio(&mut self,chance:Chance){
  self.for_native_listeners(|g|{
   let context=if g.multiplayer{let team=usize::from(g.native_feedback.local_team);let Some(c)=g.native_listener_audio.hardware_views.get(team).and_then(Clone::clone)else{g.native_audio.opportunity_boundary=Some(Boundary::MissingListenerView(g.native_feedback.local_team));return;};Some(c)}else{None};
   let(camera,size)=context.as_ref().map(|c|(c.camera,c.size)).unwrap_or_else(||g.native_view.unwrap_or(([0,0],[(g.map.width*16)as i16,(g.map.height*16)as i16])));let bank=g.native_audio_bank();let clock=g.native_feedback.ui_clock;let master=g.native_feedback.sound_enabled!=0;
   if let Some(sample)=g.native_audio.apply_opportunity(chance,camera,size,bank,clock,master){g.emit_admitted_native_sample(sample,None,context.as_ref().map(|c|c.generation.as_str()));}
  });
 }
}
