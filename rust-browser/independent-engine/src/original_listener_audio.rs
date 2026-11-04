//! Two host-authoritative listening contexts for the proven local audio callers.
//! The world/BUILD/physics handlers still run once. Only local admission and
//! portrait state are selected here; the UI word remains one shared host clock.
//! Camera-dependent sound/under-attack admission still uses canonical nativeView:
//! this module does not claim a per-client viewport or original MP wire protocol.
use crate::{Game,Value,json,original_audio,original_feedback,original_building_display::Radar};
pub(crate) struct State {
 pub guest_feedback:original_feedback::State,
 pub guest_audio:original_audio::State,
 pub guest_radars:[Option<Radar>;35],
 active:Option<u8>,
 sequence:u64,
}
impl Default for State{fn default()->Self{let mut feedback=original_feedback::State::default();feedback.local_team=1;Self{guest_feedback:feedback,guest_audio:Default::default(),guest_radars:[None;35],active:None,sequence:0}}}
impl Game{
 ///Select audio-only state, never a second world callback or RNG stream.
 pub(crate) fn for_native_listeners(&mut self,mut f:impl FnMut(&mut Self)){
  if !self.multiplayer||self.native_listener_audio.active.is_some(){f(self);return;}
  self.native_listener_audio.active=Some(0);f(self);
  let clock=self.native_feedback.ui_clock;
  std::mem::swap(&mut self.native_feedback,&mut self.native_listener_audio.guest_feedback);
  std::mem::swap(&mut self.native_audio,&mut self.native_listener_audio.guest_audio);
  self.native_listener_audio.active=Some(1);self.native_feedback.ui_clock=clock;f(self);
  std::mem::swap(&mut self.native_feedback,&mut self.native_listener_audio.guest_feedback);
  std::mem::swap(&mut self.native_audio,&mut self.native_listener_audio.guest_audio);
  self.native_listener_audio.active=None;
 }
 pub(crate) fn initialize_native_listener_mission(&mut self){if !self.multiplayer{return;}
  self.native_listener_audio.guest_feedback.local_team=1;
  self.native_listener_audio.guest_feedback.ui_clock=self.native_feedback.ui_clock;
  self.native_listener_audio.guest_feedback.initialize_for_mission();
  self.native_listener_audio.guest_radars=std::array::from_fn(|s|self.native_radar_raw_state(s));
 }
 ///Trusted host ABI only. Network ownership/epoch/sequence checks live at the
 ///controller boundary; a Guest command cannot forward this metadata.
 pub(crate) fn set_native_listener_settings(&mut self,value:&Value){
  if !self.multiplayer{return;}
  for(team,feedback)in [(0,&mut self.native_feedback),(1,&mut self.native_listener_audio.guest_feedback)]{
   if let Some(enabled)=value.get(team.to_string()).and_then(|v|v["soundEnabled"].as_bool()){feedback.sound_enabled=u8::from(enabled);}
  }
 }
 pub(crate) fn tag_native_audio_event(&mut self,event:&mut Value,actor:Option<i32>){
  if !self.multiplayer{return;}
  self.native_listener_audio.sequence=self.native_listener_audio.sequence.wrapping_add(1);
  event["recipientTeam"]=json!(self.native_feedback.local_team);
  event["actorTeam"]=json!(actor);
  event["nativeAudioId"]=json!(self.native_listener_audio.sequence.to_string());
 }
 pub(crate) fn native_portrait(&self,team:u8)->Value{let f=if team==1{&self.native_listener_audio.guest_feedback}else{&self.native_feedback};let speaker=f.display_speaker.and_then(|slot|self.units.iter().rfind(|u|u.native_slot==Some(slot)));json!({"recipientTeam":team,"mode":f.portrait_mode,"message":f.current_message,"phraseIndex":f.phrase_index,"phraseCounter":f.phrase_counter,"phraseFrame":f.phrase_frame,"unitId":speaker.map(|u|u.id),"nativeSlot":f.display_speaker,"rawSpeakerSlot":f.speaker})}
 ///Shared word0A advances once; only the local notice fields are duplicated.
 pub(crate) fn emit_native_radar_audio(&mut self,slot:usize,before:Radar,input:crate::original_building_display::Input,host:crate::original_building_display::Result,after:i16){
  let guest=if self.multiplayer{let r=self.native_listener_audio.guest_radars[slot].get_or_insert(before);r.flags6=(r.flags6&32)|(before.flags6&!32);let secured=r.advance_notice(input.owner,1);r.word0a=after;Some((secured,host.record.is_some()&&after!=before.word0a&&after==0&&input.owner==1))}else{None};
  self.for_native_listeners(|g|{let team=usize::from(g.native_feedback.local_team);let(secured,sweep)=if team==1{guest.unwrap_or((false,false))}else{(host.secured,host.sweep)};if secured{g.emit_native_sample_for_actor(196,original_audio::Gate::Notice(g.native_display.fort_visible.get(team)==Some(&255)),Some(i32::from(input.owner)));}if sweep{g.emit_native_sample_for_actor(16,original_audio::Gate::Enabled,Some(i32::from(input.owner)));}});
 }
 ///Caller-level master+local-fort gate, followed by Direct descriptor admission.
 ///It must not turn into Notice (which substitutes196) or an owner filter.
 pub(crate) fn emit_native_workshop_sample(&mut self,id:i16,actor:Option<i32>){self.for_native_listeners(|g|{let team=usize::from(g.native_feedback.local_team);if g.native_feedback.sound_enabled!=0&&g.native_display.fort_visible.get(team)==Some(&255)&&g.emit_native_sample_for_actor(id,original_audio::Gate::Direct,actor){g.native_audio.notice_counter=g.native_audio.notice_counter.wrapping_add(1)&3;}});}
 pub(crate) fn emit_native_master_sample(&mut self,id:i16,actor:Option<i32>){self.for_native_listeners(|g|{if g.native_feedback.sound_enabled!=0{g.emit_native_sample_for_actor(id,original_audio::Gate::Direct,actor);}});}
}
#[cfg(test)]mod tests{
 use super::*;use crate::original_audio::Gate;
 fn game()->Game{let mut g=Game::new(0,1,43);g.map.original=true;g.multiplayer=true;g.ai_teams.clear();g.native_feedback.ui_clock=500;g.initialize_native_feedback();g.set_native_listener_settings(&json!({"0":{"soundEnabled":true},"1":{"soundEnabled":true}}));g}
 fn recipients(g:&Game)->Vec<u64>{g.events.iter().filter(|e|e["kind"]=="nativeSample").map(|e|e["recipientTeam"].as_u64().unwrap()).collect()}
 #[test]fn listener_master_enabled_direct_notice_and_caller_fort_are_distinct(){let mut g=game();g.set_native_listener_settings(&json!({"0":{"soundEnabled":false}}));assert!(g.emit_native_sample(16,Gate::Enabled));assert_eq!(recipients(&g),[1]);g.events.clear();assert!(g.emit_native_sample(125,Gate::Direct));assert_eq!(recipients(&g),[0,1]);g.events.clear();assert!(!g.emit_native_sample(196,Gate::Notice(false)));assert!(g.events.is_empty());g.native_display.fort_visible=[255,2];g.emit_native_workshop_sample(193,Some(1));assert!(g.events.is_empty());g.set_native_listener_settings(&json!({"0":{"soundEnabled":true},"1":{"soundEnabled":false}}));g.emit_native_workshop_sample(193,Some(1));assert_eq!(recipients(&g),[0]);assert_eq!(g.events[0]["actorTeam"],1);assert_eq!(g.events[0]["clip"],"COMP06");assert_eq!((g.native_audio.notice_counter,g.native_listener_audio.guest_audio.notice_counter),(1,0));}
 #[test]fn simultaneous_listeners_have_independent_stamps_and_unique_once_ids(){let mut g=game();assert!(g.emit_native_sample(16,Gate::Enabled));assert_eq!(recipients(&g),[0,1]);let ids=g.events.iter().map(|e|e["nativeAudioId"].clone()).collect::<Vec<_>>();assert_ne!(ids[0],ids[1]);g.events.clear();assert!(!g.emit_native_sample(16,Gate::Enabled));g.advance_native_ui_callback();assert!(g.emit_native_sample(16,Gate::Enabled));assert_eq!(recipients(&g),[0,1]);assert!(!ids.contains(&g.events[0]["nativeAudioId"]));}
 #[test]fn signed_ui_wrap_minimum_twelve_and_mute_do_not_replay_or_stamp_disabled_calls(){for start in[32762i16,-6,500]{let mut g=game();g.native_feedback.ui_clock=start;g.native_listener_audio.guest_feedback.ui_clock=start;g.native_audio.last[0].fill(start);g.native_listener_audio.guest_audio.last[0].fill(start);g.set_native_listener_settings(&json!({"1":{"soundEnabled":false}}));for _ in 0..11{g.step_native_ui_only(1./17.);assert!(!g.emit_native_sample(125,Gate::Enabled));}assert_eq!(g.native_listener_audio.guest_audio.last[0],vec![start;g.native_audio.last[0].len()]);g.step_native_ui_only(1./17.);assert!(g.emit_native_sample(125,Gate::Enabled));assert_eq!(recipients(&g),[0]);g.events.clear();g.set_native_listener_settings(&json!({"1":{"soundEnabled":true}}));assert!(g.events.is_empty(),"changing master cannot replay an old opportunity");assert!(g.emit_native_sample(125,Gate::Enabled));assert_eq!(recipients(&g),[1],"only a new actual caller opportunity is admitted");assert_eq!(g.native_feedback.ui_clock,g.native_listener_audio.guest_feedback.ui_clock);}}
 #[test]fn settings_and_local_admissions_do_not_advance_world_rng_production_or_fx(){let mut g=game();let old=json!(&g);let rng=g.original_rng.words;let crt=g.native_cosmetic_rng;g.set_native_listener_settings(&json!({"0":{"soundEnabled":false},"1":{"soundEnabled":true}}));g.emit_native_sample(16,Gate::Enabled);g.emit_native_workshop_sample(193,Some(0));g.events.clear();assert_eq!(json!(&g),old);assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_cosmetic_rng,crt);}
 #[test]fn real_world_callbacks_keep_rng_units_production_and_fx_identical_across_listener_settings(){
  fn field()->Game{let mut g=game();g.units.clear();g.map.native_road_graph=None;g.spawn("grunt",0,600.,600.,1,None);g.spawn("gatling",1,700.,600.,2,None);g.spawn("grunt",0,100.,1800.,3,None);g.spawn("grunt",1,1800.,1800.,4,None);g.maintain_native_group_rosters();g.import_native_world(&Value::Null);g}
  let(mut quiet,mut loud)=(field(),field());quiet.set_native_listener_settings(&json!({"0":{"soundEnabled":false},"1":{"soundEnabled":false}}));
  for tick in 0..120{quiet.step(0.059);loud.step(0.059);let(mut a,mut b)=(json!(&quiet),json!(&loud));a["events"]=json!([]);b["events"]=json!([]);assert_eq!(a,b,"world callback{tick}");assert_eq!(quiet.original_rng.words,loud.original_rng.words,"native RNG callback{tick}");assert_eq!(quiet.native_cosmetic_rng,loud.native_cosmetic_rng,"CRT callback{tick}");assert_eq!(quiet.native_world_effects(),loud.native_world_effects());assert_eq!(quiet.native_display_queue(),loud.native_display_queue());}
 }
 #[test]fn portraits_and_one_phrase_clock_belong_to_the_appropriate_listener(){let mut g=game();g.for_native_listeners(|g|{if g.native_feedback.local_team==1{g.admit_native_feedback(4,crate::original_feedback::CANNOT_REACH);}});assert_eq!(g.native_portrait(0)["message"],0);assert_eq!(g.native_portrait(1)["message"],0x725);let before=g.native_portrait(1);for _ in 0..12{g.step_native_ui_only(1./17.);}assert_eq!(g.native_portrait(1),before);let clock=g.native_feedback.ui_clock;g.advance_native_feedback_clock();assert_eq!(g.native_feedback.ui_clock,clock.wrapping_add(1));assert_eq!(g.native_listener_audio.guest_feedback.ui_clock,g.native_feedback.ui_clock);assert_eq!(g.native_portrait(1)["phraseCounter"],1);assert_eq!(g.events[0]["recipientTeam"],1);}
}
