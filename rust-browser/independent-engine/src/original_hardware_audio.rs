//! Only4648a0/464e80's audio opportunity, after one646b0 warning and before
//! one464730 allocation. This module never executes world callbacks per view.
use crate::{Game,Value,json,original_audio::Gate,original_feedback::outside_view};
#[derive(Clone,Debug,PartialEq,Eq)]pub(crate) struct Context{pub camera:[i16;2],pub size:[i16;2],pub generation:String}
impl Context{
 pub fn from_settings(v:&Value)->Option<Self>{
  fn pair(v:&Value)->Option<[i16;2]>{let a=v.as_array().filter(|a|a.len()==2)?;Some([i16::try_from(a[0].as_i64()?).ok()?,i16::try_from(a[1].as_i64()?).ok()?])}
  let camera=pair(&v["view"]["camera"])?;let size=pair(&v["view"]["size"])?;if size.iter().any(|&n|n<=0){return None;}
  let generation=v["hardwareContext"].as_str()?;let mut parts=generation.split(':');for _ in 0..2{let s=parts.next()?;let n=s.parse::<u32>().ok()?;if n.to_string()!=s{return None;}}if parts.next().is_some(){return None;}
  Some(Self{camera,size,generation:generation.into()})
 }
}
///Signed16 subtraction and inclusive asymmetric padding are source operations.
pub(crate) fn visible(point:[i16;2],sample:i16,enabled:bool,camera:[i16;2],size:[i16;2])->bool{sample>=0&&enabled&&!outside_view(point,camera,size)}
impl Game{
 pub(crate) fn emit_native_hardware_audio(&mut self,point:[i16;2],sample:i16){
  self.for_native_listeners(|g|{
   let context=if g.multiplayer{let team=usize::from(g.native_feedback.local_team);let Some(c)=g.native_listener_audio.hardware_views.get(team).and_then(Clone::clone)else{return;};Some(c)}else{None};
   let(camera,size)=context.as_ref().map(|c|(c.camera,c.size)).unwrap_or_else(||g.native_view.unwrap_or(([0,0],[(g.map.width*16)as i16,(g.map.height*16)as i16])));
   if !visible(point,sample,g.native_feedback.sound_enabled!=0,camera,size){return;}
   if g.emit_native_sample_for_actor(sample,Gate::Direct,None){
    //46494b/464f25: descriptor admission returns0 even if SDL mixer fails.
    g.native_audio.effect_counter=g.native_audio.effect_counter.wrapping_add(1)&3;
    if let Some(c)=context{if let Some(e)=g.events.last_mut(){e["nativeHardwareAudioContext"]=json!(c.generation);}}
   }
  });
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn hardware_audio_signed16_inclusive_viewport_and_unavailable_input(){
  for x in[-49,-48,352,353]{assert_eq!(visible([x,0],22,true,[0,0],[320,240]),(-48..=352).contains(&x));}
  for y in[-49,-48,272,273]{assert_eq!(visible([0,y],22,true,[0,0],[320,240]),(-48..=272).contains(&y));}
  assert!(visible([-32766,32766],22,true,[32760,-32760],[320,240]));assert!(!visible([0,0],-1,true,[0,0],[320,240]));assert!(!visible([0,0],22,false,[0,0],[320,240]));
  for v in [json!(null),json!({"view":{"camera":[0,0],"size":[0,240]},"hardwareContext":"1:0"}),json!({"view":{"camera":[32768,0],"size":[320,240]},"hardwareContext":"1:0"}),json!({"view":{"camera":[0,0],"size":[320,240]},"hardwareContext":"01:0"})]{assert_eq!(Context::from_settings(&v),None);}
 }
 #[test]fn hardware_audio_actual_native_prefix_fieldwise(){
  let Some(path)=std::env::var_os("NATIVE_HARDWARE_AUDIO_PREFIX")else{return;};let rows=std::fs::read_to_string(path).unwrap();let(mut count,mut signature)=(0usize,2166136261u32);
  for line in rows.lines(){let r:Value=serde_json::from_str(line).unwrap();let pair=|k:&str|[r[k][0].as_i64().unwrap()as i16,r[k][1].as_i64().unwrap()as i16];let p=pair("point");let valid=p!=pair("target")&&r["speed"].as_i64()!=Some(0);let request=valid&&visible(p,r["sample"].as_i64().unwrap()as i16,r["enabled"]==1,pair("camera"),[320,240]);let clock=r["clock"].as_i64().unwrap()as i16;let last=r["last"].as_i64().unwrap()as i16;let mut a=crate::original_audio::State::default();a.last[0].fill(last);a.effect_counter=3;let played=request&&a.admit(0,22,clock,Gate::Direct,true).is_some();if played{a.effect_counter=a.effect_counter.wrapping_add(1)&3;}let stamp=if played{clock}else{last};
   let order=if !valid{""}else if !request{"WA"}else if played{"WSMA"}else{"WSA"};
   assert_eq!((r["warning"].as_u64(),r["allocation"].as_u64(),r["requests"].as_u64(),r["mixes"].as_u64(),r["stamp"].as_i64(),r["counter"].as_u64(),r["order"].as_str()),(Some(u64::from(valid)),Some(u64::from(valid)),Some(u64::from(request)),Some(u64::from(played)),Some(i64::from(stamp)),Some(0x12340000|u64::from(a.effect_counter as u16)),Some(order)),"{}:{}",r["kind"],r["scenario"]);
   for n in [u32::from(valid),u32::from(request),u32::from(played),stamp as u16 as u32,a.effect_counter as u16 as u32]{signature=(signature^n).wrapping_mul(16777619);}count+=1;
  }
  assert_eq!((count,signature),(1024,3479371101));println!("NATIVE_HARDWARE_AUDIO_PREFIX count={count} signature={signature}");
 }
 fn field()->Game{let mut g=Game::new(0,1,470);g.map.original=true;g.multiplayer=true;g.units.clear();g.ai_teams.clear();g.spawn("medium",0,320.,320.,1,None);g.spawn("grunt",1,1800.,1800.,2,None);g.maintain_native_group_rosters();g.initialize_native_listener_mission();g.native_feedback.ui_clock=500;g.native_view=Some(([0,0],[1024,1024]));views(&mut g,true,true,[0,0],[700,700]);g}
 fn views(g:&mut Game,host:bool,guest:bool,h:[i16;2],v:[i16;2]){g.set_native_listener_settings(&json!({"0":{"soundEnabled":host,"view":{"camera":h,"size":[320,240]},"hardwareContext":"1:0"},"1":{"soundEnabled":guest,"view":{"camera":v,"size":[320,240]},"hardwareContext":"1:0"}}));}
 fn shot()->crate::original_hardware_point::Shot{crate::original_hardware_point::Shot{origin:[160,160],target:[240,160],rocket:false,sound:22,speed:10}}
 fn recipients(g:&Game)->Vec<u64>{g.events.iter().filter(|e|e["kind"]=="nativeSample").map(|e|e["recipientTeam"].as_u64().unwrap()).collect()}
 #[test]fn hardware_audio_missing_guest_view_does_not_fall_back_to_shared_world_view(){let mut g=field();g.set_native_listener_settings(&json!({"1":{"soundEnabled":true}}));g.emit_native_hardware_audio([160,160],22);assert_eq!(recipients(&g),[0]);assert_eq!(g.events[0]["nativeHardwareAudioContext"],"1:0");assert_eq!(g.native_view,Some(([0,0],[1024,1024])));assert_eq!(g.native_listener_audio.guest_audio.effect_counter,0);}
 #[test]fn hardware_audio_direct_caller_master_local_view_and_counter_are_independent(){let mut g=field();views(&mut g,false,true,[0,0],[0,0]);let rng=g.original_rng.words;let crt=g.native_cosmetic_rng;g.emit_native_hardware_audio([160,160],22);assert_eq!(recipients(&g),[1]);assert_eq!((g.native_audio.effect_counter,g.native_listener_audio.guest_audio.effect_counter),(0,1));assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_cosmetic_rng,crt);assert!(!g.native_blast_sound_gate);assert_eq!(g.native_blast_sound_cycle,0);}
 #[test]fn hardware_audio_projectile_world_and_warning_are_not_repeated_per_listener(){let(mut a,mut b)=(field(),field());views(&mut b,true,true,[700,700],[0,0]);let ra=a.emit_native_hardware_point_projectile(0,shot());let rb=b.emit_native_hardware_point_projectile(0,shot());assert_eq!(ra,rb);assert_eq!(ra,Ok(true));assert_eq!(a.projectiles.len(),1);assert_eq!(b.projectiles.len(),1);assert_eq!(a.original_rng.words,b.original_rng.words);assert_eq!(a.native_cosmetic_rng,b.native_cosmetic_rng);assert_eq!(a.native_view,b.native_view);assert_eq!(recipients(&a),[0]);assert_eq!(recipients(&b),[1]);a.events.clear();b.events.clear();assert_eq!(json!(a),json!(b),"only listener admissions differ");}
 #[test]fn hardware_audio_full_pool_keeps_audio_chance_and_zero_speed_has_none(){let mut g=field();for _ in 0..50{assert_eq!(g.emit_native_hardware_point_projectile(0,shot()),Ok(true));}assert_eq!(g.projectiles.len(),50);g.events.clear();g.native_feedback.ui_clock=g.native_feedback.ui_clock.wrapping_add(1);let c=g.native_audio.effect_counter;assert_eq!(g.emit_native_hardware_point_projectile(0,shot()),Ok(false));assert_eq!(g.projectiles.len(),50);assert_eq!(recipients(&g),[0]);assert_eq!(g.native_audio.effect_counter,c.wrapping_add(1)&3);
  let mut zero=shot();zero.speed=0;let mut equal=shot();equal.target=equal.origin;for s in[zero,equal]{g.events.clear();let before=g.native_audio.effect_counter;assert_eq!(g.emit_native_hardware_point_projectile(0,s),Ok(false));assert!(g.events.is_empty());assert_eq!(g.native_audio.effect_counter,before);}
 }
 #[test]fn hardware_audio_last_barrel_and_silent_followup_keep_previous_allocated_kind(){let mut g=field();for _ in 0..49{assert_eq!(g.emit_native_hardware_point_projectile(0,shot()),Ok(true));}g.events.clear();g.native_feedback.ui_clock+=1;assert_eq!(g.emit_native_hardware_caller_projectile(0,shot(),true,true),Ok(true));let id=g.projectiles.last().unwrap().id;assert!(matches!(g.projectiles.last().unwrap().kind,crate::original_projectiles::Kind::Howitzer));let mut second=shot();second.sound=-1;assert_eq!(g.emit_native_hardware_caller_projectile(0,second,false,true),Ok(false));assert_eq!(g.projectiles.last().unwrap().id,id);assert!(matches!(g.projectiles.last().unwrap().kind,crate::original_projectiles::Kind::Howitzer));assert_eq!(recipients(&g),[0]);}
 #[test]fn hardware_audio_sp_keeps_native_view_and_has_no_transport_context(){let mut g=field();g.multiplayer=false;g.native_view=Some(([0,0],[320,240]));g.emit_native_hardware_audio([160,160],22);assert_eq!(g.events.len(),1);assert!(g.events[0].get("nativeHardwareAudioContext").is_none());assert_eq!(g.native_audio.effect_counter,1);g.native_view=Some(([700,700],[320,240]));g.events.clear();g.emit_native_hardware_audio([160,160],22);assert!(g.events.is_empty());}
 #[test]fn hardware_audio_listener_stamps_use_one_signed_wrapping_uiword(){for start in[32767i16,-32768,-6]{let mut g=field();views(&mut g,true,true,[0,0],[0,0]);g.native_feedback.ui_clock=start;g.native_audio.last[0].fill(start);g.native_listener_audio.guest_audio.last[0].fill(start);g.emit_native_hardware_audio([160,160],22);assert!(g.events.is_empty());g.advance_native_ui_callback();g.emit_native_hardware_audio([160,160],22);assert_eq!(recipients(&g),[0,1]);assert_eq!((g.native_audio.effect_counter,g.native_listener_audio.guest_audio.effect_counter),(1,1));assert_eq!(g.native_feedback.ui_clock,g.native_listener_audio.guest_feedback.ui_clock);}}
}
