use super::*;
use crate::{json,Value,original_audio::{State as Audio,BANKS}};
use std::collections::BTreeMap;
fn number(r:&Value,k:&str)->i64{r[k].as_i64().unwrap()}
fn fnv(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}
fn crt(value:&mut u32)->u16{*value=value.wrapping_mul(214013).wrapping_add(2531011);((*value>>16)&32767)as u16}
fn samples(g:&Game)->Vec<(i16,Option<u64>)>{g.events.iter().filter(|e|e["kind"]=="nativeSample"&&e["sampleId"].as_i64().is_some_and(|id|(90..=100).contains(&id))).map(|e|(e["sampleId"].as_i64().unwrap()as i16,e["recipientTeam"].as_u64())).collect()}
pub(crate) fn field(mp:bool)->Game{let mut g=Game::new(0,1,52);g.map.original=true;g.map.world=1;g.multiplayer=mp;g.ai_teams.clear();g.native_feedback.ui_clock=130;g.initialize_native_feedback();g.native_feedback.sound_enabled=1;g.native_cosmetic_rng=0;g.native_view=Some(([0,0],[320,200]));masters(&mut g,[true,true]);g}
fn masters(g:&mut Game,enabled:[bool;2]){if g.multiplayer{g.set_native_listener_settings(&json!({"0":{"soundEnabled":enabled[0]},"1":{"soundEnabled":enabled[1]}}));}else{g.native_feedback.sound_enabled=u8::from(enabled[0]);}}

#[test]fn main_ambient_source_zero_normalization_reload_wrap_and_unknown_suffix(){
 for mp in[false,true]{for value in[0,1,127,128,32767,65535]{let mut s=State::default();let mut calls=0;s.prefix(mp,||{calls+=1;value});assert_eq!(s.reload,if mp{1}else{u32::from(value&127)+1});assert_eq!(calls,usize::from(!mp));assert_eq!(s.pending,1);let before=s;assert_eq!(s.late(None,mp,||panic!("unavailable")),Late::Unavailable);assert_eq!(s,before);assert_eq!(s.late(Some(255),mp,||panic!("skipped")),Late::Skipped);assert_eq!(s,before);let chosen=s.late(Some(0),mp,||{calls+=1;value});let n=if mp{1}else{let n=value&32767;if n==0{1}else{n}};assert_eq!(chosen,Late::Chosen(Chance{sample:90+(n%11)as i16}));assert_eq!(calls,2*usize::from(!mp));assert_eq!(s.pending,0);assert_eq!(s.late(Some(0),mp,||panic!("empty")),Late::Empty);}}
 let mut s=State{reload:u32::MAX,pending:255};s.prefix(false,||panic!("nonzero timer"));assert_eq!(s,State{reload:u32::MAX-1,pending:255});
}

#[test]fn main_ambient_matches384_actual_prefix_late_descriptor_windows(){
 let Some(path)=std::env::var_os("NATIVE_MAIN_AMBIENT_WINDOWS")else{eprintln!("private NATIVE_MAIN_AMBIENT_WINDOWS not supplied; no original-window claim");return;};let text=std::fs::read_to_string(path).unwrap();let mut audios=BTreeMap::<(i64,i64),Audio>::new();let(mut count,mut signature)=(0,2166136261u32);
 for line in text.lines(){let r:Value=serde_json::from_str(line).unwrap();let(profile,variant,tick,bank)=(number(&r,"profile"),number(&r,"variant"),number(&r,"tick"),number(&r,"bank")as usize);let clock=number(&r,"clock")as i16;let audio=audios.entry((profile,variant)).or_insert_with(||{let mut a=Audio::default();for id in 90..=100{let i=BANKS[bank].iter().position(|d|d.id==id).unwrap();a.last[bank][i]=if variant&2!=0{clock}else{clock.wrapping_sub(BANKS[bank][i].minimum)};}a});audio.effect_counter=number(&r,"counterBefore")as i16;let mut s=State{reload:number(&r,"timerBefore")as u32,pending:number(&r,"flagBefore")as u8};let mut pcalls=0;s.prefix(r["mp"]==1,||{pcalls+=1;number(&r,"rand")as u16});let pf=s.pending;let mut lcalls=0;let choice=s.late(Some(u8::from(r["skipSuffix"]==true)),r["mp"]==1,||{lcalls+=1;number(&r,"rand")as u16});let(mut request,mut mixes,mut sample)=(0,0,-1);if let Late::Chosen(c)=choice{if r["master"]==1{request=1;sample=c.sample;if audio.admit(bank,c.sample,clock,Gate::Direct,true).is_some(){mixes=1;audio.effect_counter=audio.effect_counter.wrapping_add(1)&3;}}}
  let counter=(number(&r,"counterBefore")as u32&0xffff0000)|u32::from(audio.effect_counter as u16);assert_eq!((s.reload,pf,s.pending,pcalls,lcalls,request,mixes,sample,counter),(number(&r,"timer")as u32,number(&r,"flagPrefix")as u8,number(&r,"flag")as u8,number(&r,"prefixCrt"),number(&r,"suffixCrt"),number(&r,"requests"),number(&r,"mixes"),number(&r,"sample")as i16,number(&r,"counter")as u32),"{profile}/{variant}/{tick}");let stamps=(90..=100).map(|id|audio.last[bank][BANKS[bank].iter().position(|d|d.id==id).unwrap()]).collect::<Vec<_>>();assert_eq!(json!(stamps),r["stamps"]);
  for n in[profile as u32,variant as u32,tick as u32,s.reload,u32::from(pf),u32::from(s.pending),pcalls as u32,lcalls as u32,request as u32,mixes as u32,sample as u32,counter]{fnv(&mut signature,n);}count+=1;
 }assert_eq!((count,signature),(384,2392318205));println!("MAIN_AMBIENT_WINDOWS count={count} signature={signature}");
}

#[test]fn main_ambient_matches384_actual_interleaved_main_crt_projections(){
 let Some(path)=std::env::var_os("NATIVE_MAIN_AMBIENT_ORDERED")else{eprintln!("private NATIVE_MAIN_AMBIENT_ORDERED not supplied; no original-interleave claim");return;};let text=std::fs::read_to_string(path).unwrap();let(mut count,mut signature)=(0,2166136261u32);
 for line in text.lines(){let r:Value=serde_json::from_str(line).unwrap();let mut random=number(&r,"crtBefore")as u32;let mut ordinal=number(&r,"ordinalBefore")as u32;let mut draw=||{ordinal+=1;crt(&mut random)};let mut s=State{reload:number(&r,"timerBefore")as u32,pending:number(&r,"pendingBefore")as u8};s.prefix(r["mp"]==1,&mut draw);
  //This fixture supplies the reached real Lava-call ordinal from the native
  //trace. The following Game test executes the actual ported Lava body.
  if r["draws"].as_str().unwrap().contains(":4069cd:"){draw();}s.late(Some(number(&r,"controllerAL")as u8),r["mp"]==1,&mut draw);assert_eq!((s.reload,s.pending,random,ordinal),(number(&r,"timer")as u32,number(&r,"pending")as u8,number(&r,"crt")as u32,number(&r,"ordinal")as u32));
  for n in[number(&r,"scenario")as u32,number(&r,"tick")as u32,number(&r,"mp")as u32,s.reload,u32::from(s.pending),random,ordinal]{fnv(&mut signature,n);}count+=1;
 }assert_eq!((count,signature),(384,4056110389));println!("MAIN_AMBIENT_ORDERED count={count} signature={signature}; seed-guard result is an input, not implemented here");
}

#[test]fn main_ambient_real_lava_remains_between_the_two_shared_crt_draws(){
 let mut g=field(false);g.spawn_native_lava(crate::original_ambient::LavaSeed{position:[150,160],template:0,size:2,interval:0});let world_rng=g.original_rng.words;g.begin_native_main_ambient();assert_eq!(g.native_main_ambient.reload,39);g.update_native_lava();let result=g.finish_native_main_ambient(Some(0));assert_eq!(result,Late::Chosen(Chance{sample:90+21238%11}));assert_eq!(g.native_cosmetic_rng,3539360597);assert_eq!(g.original_rng.words,world_rng);assert_eq!(g.native_lava_visuals().len(),1);assert_eq!(samples(&g),[(90+21238%11,None)]);
}

#[test]fn main_ambient_mp_has_no_crt_or_camera_requirement_and_admits_each_listener_once(){
 let mut g=field(true);g.native_view=Some(([30000,30000],[1,1]));let rng=g.original_rng.words;g.begin_native_main_ambient();assert_eq!(g.native_main_ambient,State{reload:1,pending:1});g.finish_native_main_ambient(Some(0));assert_eq!(samples(&g),[(91,Some(0)),(91,Some(1))]);assert_eq!(g.native_cosmetic_rng,0);assert_eq!(g.original_rng.words,rng);assert_eq!((g.native_audio.effect_counter,g.native_listener_audio.guest_audio.effect_counter),(1,1));assert!(g.events.iter().all(|e|e.get("nativeHardwareAudioContext").is_none()&&e["actorTeam"].is_null()));let events=g.events.clone();g.finish_native_main_ambient(Some(0));assert_eq!(g.events,events);
}

#[test]fn main_ambient_muted_and_rejected_chances_are_consumed_without_replay_or_extra_draws(){
 for mp in[false,true]{let mut g=field(mp);masters(&mut g,[false,false]);g.begin_native_main_ambient();g.finish_native_main_ambient(Some(0));let crt=g.native_cosmetic_rng;assert_eq!(g.native_main_ambient.pending,0);assert!(samples(&g).is_empty());masters(&mut g,[true,true]);g.finish_native_main_ambient(Some(0));assert!(samples(&g).is_empty());assert_eq!(g.native_cosmetic_rng,crt);}
 let mut g=field(true);g.native_audio.last[3].fill(130);g.native_listener_audio.guest_audio.last[3].fill(130);g.begin_native_main_ambient();g.finish_native_main_ambient(Some(0));assert!(samples(&g).is_empty());assert_eq!(g.native_main_ambient.pending,0);assert_eq!((g.native_audio.effect_counter,g.native_listener_audio.guest_audio.effect_counter),(0,0));
}

#[test]fn main_ambient_unknown_and_skipped_late_preserve_pending_and_never_draw(){
 let mut g=field(false);g.native_main_ambient=State{reload:40,pending:255};let before=g.native_cosmetic_rng;for result in[None,Some(1),Some(255)]{let state=g.native_main_ambient;g.finish_native_main_ambient(result);assert_eq!(g.native_main_ambient,state);assert_eq!(g.native_cosmetic_rng,before);assert!(g.events.is_empty());}g.finish_native_main_ambient(Some(0));assert_eq!(g.native_main_ambient.pending,0);assert_ne!(g.native_cosmetic_rng,before);
}

#[test]fn main_ambient_ui_pause_wrap_settings_and_scene_mode_do_not_reset_scheduler(){
 for mp in[false,true]{for start in[32762i16,-6,130]{let mut g=field(mp);g.native_feedback.ui_clock=start;g.native_main_ambient=State{reload:0xffffffff,pending:255};let state=g.native_main_ambient;let crt=g.native_cosmetic_rng;g.native_audio.opportunity_gate=255;g.native_audio.explosion_cycle=7;g.native_audio.late_build_clock=99;for _ in 0..12{g.step_native_ui_only(1./17.);}masters(&mut g,[false,true]);g.native_feedback.scene_gate=16;assert_eq!(g.native_main_ambient,state);assert_eq!(g.native_cosmetic_rng,crt);assert_eq!(g.native_feedback.ui_clock,start.wrapping_add(12));assert_eq!((g.native_audio.opportunity_gate,g.native_audio.explosion_cycle,g.native_audio.late_build_clock),(255,7,99));assert!(samples(&g).is_empty());}}
}

#[test]fn main_ambient_master_variants_keep_one_actual_world_and_shared_crt_trace(){
 for mp in[false,true]{let mut reference=None;for enabled in[[true,true],[false,true],[true,false],[false,false]]{let mut g=field(mp);g.import_native_world(&Value::Null);g.maintain_native_group_rosters();for m in g.native_group_rosters.members.iter_mut().flatten(){m.flags69|=64;}masters(&mut g,enabled);let mut trace=Vec::new();for _ in 0..80{g.step(0.059);g.events.clear();trace.push(json!({"world":g,"crt":g.native_cosmetic_rng,"rng":g.original_rng.words,"reload":g.native_main_ambient.reload,"pending":g.native_main_ambient.pending,"fx":g.native_world_effects(),"queue":g.native_display_queue()}));}if let Some(old)=&reference{assert_eq!(&trace,old);}else{reference=Some(trace);}}}
}

#[test]fn main_ambient_existing_apply_and_request_init_transfer_preserve_warm_state(){
 for mp in[false,true]{let mut g=field(mp);g.native_main_ambient=State{reload:0x87654321,pending:255};let mission=json!({"map":g.map,"sectors":g.sectors,"buildings":g.buildings,"spawns":[],"crates":[]});g.apply_original(&mission).unwrap();assert_eq!(g.native_main_ambient,State{reload:0x87654321,pending:255});let result=crate::request(json!({"op":"init","originalMission":mission,"multiplayer":mp,"nativeCosmeticSeed":991}));assert!(result.get("error").is_none());crate::GAME.with(|cell|{let mut state=cell.borrow_mut();let g=state.as_mut().unwrap();g.native_main_ambient=State{reload:77,pending:17};g.native_cosmetic_rng=0x24681357;});let result=crate::request(json!({"op":"init","originalMission":mission,"multiplayer":!mp,"nativeCosmeticSeed":123}));assert!(result.get("error").is_none());crate::GAME.with(|cell|{let state=cell.borrow();let g=state.as_ref().unwrap();assert_eq!(g.native_main_ambient,State{reload:77,pending:17});assert_eq!(g.native_cosmetic_rng,0x24681357);});}
}
