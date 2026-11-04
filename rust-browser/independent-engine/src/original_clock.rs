//!46c510's UI word is distinct from434a70 and46ade0's portrait calls.
//!UiClock is a bounded browser scheduler for an explicit paused UI callback;
//!it does not reproduce all SDL/menu/connection early-return branches.
use crate::Game;
#[derive(Default)]pub(crate) struct UiClock{remainder:f64}
impl UiClock{
 pub fn reset(&mut self){self.remainder=0.;}
 pub fn consume(&mut self,dt:f64)->usize{if !dt.is_finite()||dt<=0.{return 0;}self.remainder+=dt.min(0.1);let mut n=0;while self.remainder+1e-10>=1./17.{self.remainder-=1./17.;n+=1;}n}
}
impl Game{
 ///Host/offline UI only: no simulation, animation, production, RNG or
 ///portrait advance. Clamp elapsed time instead of accumulating a backlog.
 pub(crate) fn step_native_ui_only(&mut self,dt:f64){if !self.map.original{return;}let callbacks=self.native_ui_clock.consume(dt);for _ in 0..callbacks{self.advance_native_ui_callback();}}
}
#[cfg(test)]mod tests{
 use super::*;use crate::{json,original_audio::Gate,original_combat::Attack};
 fn game()->Game{let mut g=Game::new(0,1,41);g.map.original=true;g.ai_teams.clear();g.native_feedback.ui_clock=500;g.native_feedback.sound_enabled=1;g}
 #[test]fn source_ui_callbacks_advance_while_all_game_counters_and_portrait_stand(){let mut g=game();g.native_feedback.portrait_mode=3;g.native_feedback.phrase_length=100;g.native_feedback.phrase_counter=2;g.native_feedback.phrase_frame=7;let a=Attack{target:0,damage:1,aim:0,facing:0,bullet:false};g.spawn_native_blast(100,100,a,0,None);let old=json!(g);let rng=g.original_rng.words;let crt=g.native_cosmetic_rng;for _ in 0..12{g.step_native_ui_only(1./17.);}assert_eq!(g.native_feedback.ui_clock,512);assert_eq!(json!(g),old,"UI word is not part of the serialized simulation state");assert_eq!((g.native_feedback.phrase_counter,g.native_feedback.phrase_frame),(2,7));assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_cosmetic_rng,crt);assert_eq!((g.native_frame16,g.native_object_frame,g.native_flag_frame,g.native_wait_clock,g.native_callback_mode),(0,0,0,0,0));}
 #[test]fn real_minimum_twelve_sample_descriptor_admits_after_twelve_paused_callbacks_including_wrap(){for start in[500i16,32762,-6]{let mut g=game();g.native_feedback.ui_clock=start;g.native_audio.last[0].fill(start);for _ in 0..11{g.step_native_ui_only(1./17.);assert!(!g.emit_native_sample(125,Gate::Direct));}g.step_native_ui_only(1./17.);assert!(g.emit_native_sample(125,Gate::Direct));assert_eq!(g.events.last().unwrap()["clip"],"RATTLEXX");}}
 #[test]fn ui_accumulation_is_bounded_and_never_becomes_a_resume_catchup(){let mut g=game();g.step_native_ui_only(1e6);assert_eq!(g.native_feedback.ui_clock,501);g.step_native_ui_only(f64::NAN);g.step_native_ui_only(-1.);assert_eq!(g.native_feedback.ui_clock,501);g.step(0.);g.step_native_ui_only(0.01);assert_eq!(g.native_feedback.ui_clock,501,"the old paused scheduling remainder was discarded on normal tick");let mut g=game();g.step(0.059);assert_eq!(g.native_feedback.ui_clock,501,"ordinary direct tick retains its previous callback semantics");}
 #[test]fn actual_protocol_ui_tick_reports_only_the_ui_word(){let g=game();let initial=crate::request(json!({"op":"init","originalMission":{"map":g.map,"sectors":g.sectors,"buildings":g.buildings,"crates":g.crates,"spawns":[]}}));assert!(initial.get("error").is_none(),"{initial}");let before=initial["nativeUiClock"].as_i64().unwrap();let current=crate::request(json!({"op":"uiTick","dt":1./17.}));assert_eq!(current["nativeUiClock"],before+1);for key in["time","units","buildings","nativePortrait","nativeEffects","projectiles","blasts"]{assert_eq!(current[key],initial[key],"{key}");}}
 #[test]fn game_and_ui_split_match_396_original_caller_clock_and_sample_outcomes(){let mut hash=2166136261u32;for scenario in 0..18u32{let mode=[0,1,16][scenario as usize%3];let mut g=game();let start=[500i16,32762,-6][scenario as usize/3%3];g.native_feedback.ui_clock=start;g.native_audio.last[0].fill(start);g.maintain_native_group_rosters();for member in g.native_group_rosters.members.iter_mut().flatten(){member.flags69|=64;}
  for tick in 0..22u32{if mode==0&&(4..17).contains(&tick){g.step_native_ui_only(1./17.);}else{g.step(0.059);}let played=g.emit_native_sample(125,Gate::Direct);for value in[scenario,tick,u32::from(g.native_feedback.ui_clock as u16),g.native_frame16 as u32,g.native_object_frame as u32,g.native_wait_clock as u32,g.native_flag_frame as u32,u32::from(g.native_callback_mode),u32::from(played),u32::from(!played)]{hash=(hash^value).wrapping_mul(16777619);}}
 }assert_eq!(hash,3506412265,"V41PauseClockOracle actual46c510/465050/435a50/437660/434a70/46e660; menu/world/render leaf bodies remain bounded");}
}
