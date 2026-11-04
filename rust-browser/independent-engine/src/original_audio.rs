//!46e660's per-bank descriptor admission and its46e6d0/46f650 wrappers.
//!Playback uses private PCM assets; source filenames are metadata only.
use crate::{Game,json};
#[derive(Clone,Copy,Debug)]pub struct Descriptor{pub id:i16,pub clip:&'static str,pub minimum:i16,pub initial:i16}
include!("native_sound_data.rs");
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Gate{Direct,Enabled,Notice(bool)}
#[derive(Clone,Debug)]pub struct State{pub last:[Vec<i16>;5],pub effect_counter:i16,pub notice_counter:i16}
impl Default for State{fn default()->Self{Self{last:std::array::from_fn(|bank|BANKS[bank].iter().map(|d|d.initial).collect()),effect_counter:0,notice_counter:0}}}
impl State{
 pub fn admit(&mut self,bank:usize,mut id:i16,clock:i16,gate:Gate,enabled:bool)->Option<Descriptor>{
  if matches!(gate,Gate::Enabled|Gate::Notice(_))&&!enabled{return None;}if let Gate::Notice(fort_alive)=gate{if !fort_alive{return None;}id=196;}
  let descriptors=BANKS.get(bank)?;let index=descriptors.iter().position(|d|d.id==id)?;let descriptor=descriptors[index];if clock.wrapping_sub(self.last[bank][index]).wrapping_abs()<descriptor.minimum{return None;}self.last[bank][index]=clock;
  match gate{Gate::Enabled=>self.effect_counter=self.effect_counter.wrapping_add(1)&3,Gate::Notice(_)=>self.notice_counter=self.notice_counter.wrapping_add(1)&3,Gate::Direct=>{}}Some(descriptor)
 }
}
fn is_voice(clip:&str)->bool{clip.starts_with("ROB")||clip.starts_with("COMP")}
impl Game{
 fn native_audio_bank(&self)->usize{self.map.wall_tile_lookup["nativeWorld"].as_u64().filter(|&n|n<5).map_or([0,3,1,4,2][self.map.world.min(4)],|n|n as usize)}
 ///Shared descriptor timestamps also apply when a speech caller emits its
 ///own nativeVoice event and needs admission without a second playback event.
 pub(crate) fn admit_native_sample(&mut self,id:i16,gate:Gate)->Option<Descriptor>{let bank=self.native_audio_bank();let enabled=self.native_feedback.sound_enabled!=0;self.native_audio.admit(bank,id,self.native_feedback.ui_clock,gate,enabled)}
 pub(crate) fn emit_native_sample(&mut self,id:i16,gate:Gate)->bool{self.emit_native_sample_for_actor(id,gate,None)}
 pub(crate) fn emit_native_sample_for_actor(&mut self,id:i16,gate:Gate,actor:Option<i32>)->bool{let mut any=false;self.for_native_listeners(|g|{let Some(sample)=g.admit_native_sample(id,gate)else{return;};let mut event=json!({"kind":"nativeSample","sampleId":sample.id,"clip":sample.clip,"lane":if is_voice(sample.clip){"voice"}else{"effects"},"playAudio":true,"team":g.native_feedback.local_team});g.tag_native_audio_event(&mut event,actor);g.events.push(event);any=true;});any}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_battle_descriptor_wrappers_match_complete_native_calls(){
  let native=std::env::var("NATIVE_SAMPLE_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|s.split(',').map(|v|v.parse::<i64>().unwrap()).collect::<Vec<_>>()).collect::<Vec<_>>());let(mut hash,mut row)=(2166136261u32,0usize);
  let mut check=|bank:usize,index:usize,clock:usize,mode:usize,enabled:bool,fort:bool|{let now=[0,1,12,120,32767,-32768,-32768,0][clock];let last=[0,0,0,0,32766,32767,0,-32768][clock];let mut state=State::default();state.last[bank][index]=last;state.effect_counter=(row*17)as i16;state.notice_counter=(row*29)as i16;let gate=match mode{0=>Gate::Direct,1=>Gate::Enabled,_=>Gate::Notice(fort)};let played=state.admit(bank,BANKS[bank][index].id,now,gate,enabled).is_some();let values=[row as i64,bank as i64,index as i64,clock as i64,mode as i64,i64::from(enabled),i64::from(fort),if mode==0{i64::from(!played)}else{-1},i64::from(played),i64::from(state.last[bank][index]),i64::from(state.effect_counter as u16),i64::from(state.notice_counter as u16),if row%2==0{-1}else{3}];if let Some(records)=&native{assert_eq!(values.as_slice(),records[row].as_slice(),"bank{bank}/record{index}/clock{clock}/gate{mode}");}for v in values{hash=(hash^v as u32).wrapping_mul(16777619);}row+=1;};
  for bank in 0..5{for index in 0..BANKS[bank].len(){for clock in 0..8{for mode in 0..2{check(bank,index,clock,mode,(index+clock)%3!=0,true);}}}let notice=BANKS[bank].iter().position(|d|d.id==196).unwrap();for clock in 0..8{for enabled in[false,true]{for fort in[false,true]{check(bank,notice,clock,2,enabled,fort);}}}}
  if let Some(records)=&native{assert_eq!(row,records.len());}assert_eq!((row,hash),(12560,2239336345),"all real bank descriptors, signed clock wraps, enabled/fort wrappers, SDL success/failure boundary");
 }
 #[test]fn portrait_and_direct_samples_share_the_same_descriptor_but_world_banks_keep_their_own_stamps(){let mut game=Game::new(0,1,170017);game.map.original=true;game.native_feedback.ui_clock=100;game.native_feedback.sound_enabled=1;game.initialize_native_feedback();assert!(game.emit_native_sample(152,Gate::Direct));assert!(game.admit_native_feedback(4,0x419));assert_eq!(game.events[1]["playAudio"],false);assert_eq!(game.events[1]["clip"],"ROB25");game.map.wall_tile_lookup["nativeWorld"]=json!(1);assert!(game.emit_native_sample(152,Gate::Direct));assert_eq!(game.events.last().unwrap()["clip"],"ROB25");}
}
