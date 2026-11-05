//!Seeded controller/reader stages, not asserted natural job-birth histories.
use super::*;
use crate::original_audio::main_ambient::{State as Ambient,Late};
fn field(mp:bool)->Game{let mut g=crate::original_audio::main_ambient::tests::field(mp);g.map.world=0;g.import_native_world(&Value::Null);g.maintain_native_group_rosters();for m in g.native_group_rosters.members.iter_mut().flatten(){m.flags69|=64;}g}
fn ambient(g:&Game)->Vec<i16>{g.events.iter().filter_map(|e|e["sampleId"].as_i64().filter(|id|(90..=100).contains(id)).map(|id|id as i16)).collect()}

#[test]fn main_ambient_main_reuses_one_terminal_controller_result_instead_of_consuming_pending(){
 let mut g=field(false);g.native_world.as_mut().unwrap().trail_scenes[0]=169;g.step(0.059);assert_eq!(g.native_world.as_ref().unwrap().trail_scenes[0],-1);assert_eq!(g.native_world.as_ref().unwrap().trail_main_result,Some(1));assert_eq!(g.native_main_ambient,Ambient{reload:39,pending:1});assert_eq!(g.native_cosmetic_rng,2531011,"only the Prefix CRT draw, no late draw after AL1");assert!(ambient(&g).is_empty());
}

#[test]fn main_ambient_main_does_not_turn_unavailable_controller_into_a_zero_result(){
 let mut g=field(false);g.native_display.scene_mode=16;g.step(0.059);assert_eq!(g.native_world.as_ref().unwrap().trail_main_result,None);assert_eq!(g.native_main_ambient.pending,1);assert_eq!(g.native_cosmetic_rng,2531011);assert!(ambient(&g).is_empty());assert_eq!(g.finish_native_main_ambient(None),Late::Unavailable);assert_eq!(g.native_main_ambient.pending,1);
}

#[test]fn main_ambient_real_robot_build_restore_crt_is_between_prefix_and_late(){
 for mp in[false,true]{let mut reference=None;for muted in[false,true]{let mut g=field(mp);g.native_feedback.sound_enabled=u8::from(!muted);g.native_listener_audio.guest_feedback.sound_enabled=u8::from(!muted);let id=g.buildings[0].id;g.buildings[0].kind="robot".into();g.buildings[0].hp=0.;g.buildings[0].max_hp=100.;let s=g.native_world.as_mut().unwrap();s.buildings[0].record.kind=4;s.buildings[0].record.hp=0;s.buildings[0].maximum=100;s.buildings[0].binding=Some(id);s.fragments.bindings[0]=Some(0);s.trail.known_builds[0]=true;let b=&mut s.trail.builds[0];*b=[0;88];b[4]=4;trail::raw_set(b,28,100);trail::raw_set(b,30,0);trail::raw_set(b,80,-1);let h=trail::header(4).unwrap().0;s.trail.put_word(h,1).unwrap();s.trail.write(h+10,&[9,8]).unwrap();let j=&mut s.trail.jobs[0];j.set_ptr(0,h);j.set_ptr(4,BUILD);j.0[12]=2;j.set(20,0);
  g.step(0.059);let s=g.native_world.as_ref().unwrap();assert_eq!(s.buildings[0].record.hp,100);assert_eq!(s.trail.jobs[0].ptr(0),0);assert_eq!(g.native_cosmetic_rng,if mp{0}else{3539360597});assert_eq!(s.trail.builds[0][62],if mp{0}else{7719u16 as u8});assert_eq!(g.native_main_ambient.pending,0);let sample=if mp{91}else{90+21238%11};assert_eq!(ambient(&g),if muted{vec![]}else if mp{vec![sample,sample]}else{vec![sample]});let trace=json!({"rng":g.original_rng.words,"crt":g.native_cosmetic_rng,"jobs":s.trail.jobs.iter().map(|j|j.0.to_vec()).collect::<Vec<_>>(),"build":s.trail.builds[0].to_vec(),"hp":s.buildings[0].record.hp});if let Some(old)=&reference{assert_eq!(&trace,old,"the actual World/restore is not repeated or skipped by master");}else{reference=Some(trace);}assert_eq!(g.finish_native_main_ambient(Some(0)),Late::Empty);
 }}
}
