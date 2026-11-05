//!435af1's reload/pending and46e5f0's late choice. There is ONE scheduler
//!beside the existing shared CRT, never a scheduler or random draw per listener.
//!Full33ca0 resource/UI,46c510 seeding and original socket sessions remain
//!external input/lifecycle boundaries; this module neither seeds nor resets.
use crate::{Game,original_audio::Gate};

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct State{pub reload:u32,pub pending:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Chance{pub sample:i16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Late{Unavailable,Skipped,Empty,Chosen(Chance)}

impl State{
 ///435af1..435b45. Master, readiness, camera and listener count are irrelevant.
 pub fn prefix(&mut self,multiplayer:bool,mut random:impl FnMut()->u16){
  if self.reload!=0{self.reload=self.reload.wrapping_sub(1);return;}
  self.reload=if multiplayer{1}else{u32::from(random()&127)+1};self.pending=1;
 }
 ///The caller passes the ONE423140 result after the late BUILD reader. None
 ///is an unavailable source input, not a successful/zero controller result.
 pub fn late(&mut self,result:Option<u8>,multiplayer:bool,mut random:impl FnMut()->u16)->Late{
  let Some(result)=result else{return Late::Unavailable;};
  if result!=0{return Late::Skipped;}if self.pending==0{return Late::Empty;}
  self.pending=0;let value=if multiplayer{0}else{random()&32767};
  let normalized=if value==0{1}else{value};
  Late::Chosen(Chance{sample:90+(normalized%11)as i16})
 }
}
impl Game{
 pub(crate) fn begin_native_main_ambient(&mut self){
  let mut state=self.native_main_ambient;state.prefix(self.multiplayer,||self.native_cosmetic_random());self.native_main_ambient=state;
 }
 pub(crate) fn finish_native_main_ambient(&mut self,result:Option<u8>)->Late{
  let mut state=self.native_main_ambient;let outcome=state.late(result,self.multiplayer,||self.native_cosmetic_random());self.native_main_ambient=state;
  if let Late::Chosen(chance)=outcome{self.emit_native_main_ambient_audio(chance);}outcome
 }
 ///46e5f0's caller-master followed by Direct, with the existing descriptor,
 ///UIword and effect counter. No view/actor/Fort or hardware-generation tag.
 fn emit_native_main_ambient_audio(&mut self,chance:Chance){
  self.for_native_listeners(|g|{
   if g.native_feedback.sound_enabled==0{return;}
   if let Some(sample)=g.admit_native_sample(chance.sample,Gate::Direct){
    g.native_audio.effect_counter=g.native_audio.effect_counter.wrapping_add(1)&3;
    g.emit_admitted_native_sample(sample,None,None);
   }
  });
 }
}
#[cfg(test)]#[path="original_main_ambient_tests.rs"]pub(crate) mod tests;
