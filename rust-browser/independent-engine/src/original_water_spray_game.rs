//!4011a0/401560/4015d0/402ce0 ->402230. Allocations share the late-FX pool.
use crate::{Game,original_combat::OriginalRng};
use super::{State,effects::{Particle,spray::{self,Context,Pool}}};
struct World<'a>{state:&'a mut State,rng:&'a mut OriginalRng,cp_owner:Option<u8>}
impl Pool for World<'_>{
 fn first_free(&self)->Option<usize>{self.state.effects.iter().position(|p|p.as_ref().is_none_or(|p|!p.active||p.position[0]==-1000))}
 fn load(&mut self,slot:usize)->[u8;44]{self.state.effects[slot].as_ref().map_or([0;44],Particle::bytes)}
 fn store(&mut self,slot:usize,record:[u8;44]){self.state.put_effect(slot,Particle::from_bytes(record));self.state.effect_cp_tints[slot]=self.cp_owner;}
 fn random(&mut self)->u32{self.rng.next_u32()}
}
impl Game{
 ///The unported writers of4d18fa can supply this exact raw guard without
 ///equating it with sound suppression or an unrelated graphics preference.
 pub(crate) fn set_native_particle_suppression(&mut self,value:u8){if let Some(s)=&mut self.native_world{s.effect_suppression=value;}}
 pub(super) fn emit_native_water_spray(&mut self,count:i16)->usize{
  self.emit_native_spray_with_cp(count,None)
 }
 pub(super) fn emit_native_spray_with_cp(&mut self,count:i16,cp_owner:Option<u8>)->usize{
  let Some(state)=&mut self.native_world else{return 0;};let g=Context{origin:state.effect_origin,tint:state.effect_tint,flags:state.effect_flags,fade:state.effect_fade,owner:state.effect_owner,sprite:state.effect_sprite};let disabled=state.effect_suppression;self.original_rng.multiplayer=self.multiplayer;
  spray::emit(&mut World{state,rng:&mut self.original_rng,cp_owner},count,disabled,g)
 }
 pub(super) fn finish_native_water_expiry(&mut self,slot:usize){
  let Some(state)=&mut self.native_world else{return;};let Some(source)=state.effects.get(slot).and_then(Option::as_ref)else{return;};
  state.effect_origin=[source.origin[0].wrapping_add(state.last_projection[0]).wrapping_sub(160),source.origin[1].wrapping_add(state.last_projection[1]).wrapping_sub(92)];state.effect_tint=match source.expiry{2=>22,9=>11,_=>9};state.effect_flags=128;state.effect_fade=128;state.effect_owner=0;state.effect_sprite=0;
  //The source remains allocated during all twelve attempts, even on failure.
  self.emit_native_water_spray(12);
  if let Some(p)=self.native_world.as_mut().and_then(|s|s.effects.get_mut(slot)).and_then(Option::as_mut){p.active=false;p.position[0]=-1000;}
 }
}
#[cfg(test)]mod tests{
 use super::*;use crate::{json,original_group_game};use super::super::effects::{self,Kind};
 fn field()->Game{let mut g=Game::new(0,1,597);g.map.original=true;g.map.width=64;g.map.height=64;g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.ai_teams.clear();g.native_world=None;g.native_group_rosters=original_group_game::State::default();g.import_native_world(&json!(null));g}
 fn source(expiry:i16)->Particle{let mut p=effects::initialize(Kind::Destroyed,[200,200],0,||0);p.expiry=expiry;p.lifetime=0;p.maximum_phase=7;p}
 #[test]fn expiry_keeps_the_source_occupied_preserves_recycled_bytes_and_only_then_retires(){
  let mut g=field();let mut old=std::array::from_fn(|i|(i*17+31)as u8);old[..2].copy_from_slice(&(-1000i16).to_le_bytes());let s=g.native_world.as_mut().unwrap();s.effects=vec![Some(Particle::from_bytes(old)),Some(source(8)),None];s.last_projection=[160,100];s.effect_maximum=11;let mut rng=g.original_rng.clone();for _ in 0..10{rng.next_u32();}g.retire_native_fragment_water_particle(1);assert_eq!(g.original_rng.words,rng.words);let s=g.native_world.as_ref().unwrap();assert!(!s.effects[1].as_ref().unwrap().active);assert_eq!(s.effects.iter().flatten().filter(|p|p.active).count(),2);let current=s.effects[0].as_ref().unwrap().bytes();for i in[24,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43]{assert_eq!(current[i],old[i],"reused byte{i}");}assert_eq!(s.effects[0].as_ref().unwrap().origin,[200,208]);assert_eq!((s.effect_maximum,s.effect_tint,s.effect_flags,s.effect_fade,s.effect_owner,s.effect_sprite),(11,9,128,128,0,0));assert!(s.effects[2].as_ref().unwrap().active);
 }
 #[test]fn full_and_disabled_pools_do_not_spend_rng_or_reuse_the_expiring_slot(){
  for disabled in[0,1,255]{let mut g=field();g.native_world.as_mut().unwrap().effects=if disabled==0{vec![Some(source(5))]}else{vec![Some(source(5)),None,None]};g.set_native_particle_suppression(disabled);let before=g.original_rng.words;g.retire_native_fragment_water_particle(0);assert_eq!(g.original_rng.words,before);let s=g.native_world.as_ref().unwrap();assert!(!s.effects[0].as_ref().unwrap().active);assert!(s.effects.iter().flatten().all(|p|!p.active));assert_eq!((s.effect_tint,s.effect_owner,s.effect_flags,s.effect_fade,s.effect_sprite),(9,0,128,128,0));if disabled==0{assert_eq!(g.emit_native_water_spray(1),1,"only the subsequent emitter may reuse the now retired source");assert_ne!(g.original_rng.words,before);}}
 }
 #[test]fn source_slot_order_controls_same_callback_point_draws_and_retains_them_after_ground_retirement(){
  for slot in[0,2]{let mut g=field();let s=g.native_world.as_mut().unwrap();s.effects=vec![None;3];s.effects[slot]=Some(source(8));s.last_projection=[160,100];let mut rng=g.original_rng.clone();for _ in 0..10{rng.next_u32();}g.update_native_world_effects();assert_eq!(g.original_rng.words,rng.words);assert!(!g.native_world.as_ref().unwrap().effects[slot].as_ref().unwrap().active);let rows=g.native_world_effects();assert_eq!(rows.len(),if slot==0{2}else{0});for row in rows{assert_eq!(row["primitive"],"colorPoint");assert_eq!(row["colorIndex"],9);assert_eq!((row["x"].as_i64(),row["y"].as_i64()),(Some(720),Some(616)));for absent in["sprite","team","scale","rotation"]{assert!(row.get(absent).is_none());}}if slot==2{assert!(g.native_world.as_ref().unwrap().effects[..2].iter().flatten().all(|p|p.lifetime>=47));g.update_native_world_effects();assert_eq!(g.native_world_effects().len(),2);}}
 }
 #[test]fn native_multiplayer_zero_rng_results_still_create_the_exact_water_seed(){let mut g=field();g.multiplayer=true;g.native_world.as_mut().unwrap().effects=vec![None,Some(source(5))];let before=g.original_rng.words;g.retire_native_fragment_water_particle(1);assert_eq!(g.original_rng.words,before);let p=g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap();assert_eq!((p.phase,p.velocity,p.lifetime,p.fade_left,p.fade_interval,p.flags,p.sprite),(0,[-8,-8,4],47,-128,-128,128,0));}
 #[test]fn actual_game_callback_exports_a_typed_late_color_point_without_a_sprite(){let mut g=field();g.spawn("grunt",0,100.,100.,1,None);g.spawn("grunt",1,1600.,1600.,2,None);let s=g.native_world.as_mut().unwrap();s.effects=vec![Some(source(5)),None,None];s.last_projection=[160,100];g.step(0.059);let rows=g.native_world_effects();assert_eq!(rows.len(),2);assert!(rows.iter().all(|r|r["primitive"]=="colorPoint"&&r["colorIndex"]==9&&r.get("sprite").is_none()));assert!(g.native_display_queue().iter().all(|r|r["draw"]["primitive"]!="colorPoint"),"the500-pool keeps its original late-FX pass");}
 #[test]fn optional_all_1280_native_vectors_also_match_the_actual_typed_game_pool(){
  let Ok(csv)=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../work/parity-audit/native-water-spray.jsonl"))else{return;};let mut g=field();let hash=|bytes:Vec<[u8;44]>|{let mut h=2166136261u32;for b in bytes.into_iter().flatten(){h=(h^u32::from(b)).wrapping_mul(16777619);}h};let mut count=0;
  for line in csv.lines(){let r:serde_json::Value=serde_json::from_str(line).unwrap();let n=|k:&str|r[k].as_i64().unwrap();let scenario=n("scenario")as usize;let capacity=n("capacity")as usize;let pattern=n("pattern");let mode=n("mode");let src=n("source");let mut records=(0..500usize).map(|slot|{let mut b=std::array::from_fn(|j|(scenario*17+slot*13+j*19)as u8);let free=match pattern{0=>false,1=>true,2=>slot==0,3=>slot==capacity.wrapping_sub(1),4=>slot%2==0,5=>[1,7,31,99,499].contains(&slot),6=>slot<11,_=>slot<13};b[..2].copy_from_slice(&(if free{-1000i16}else{[0,-1,999][slot%3]}).to_le_bytes());b}).collect::<Vec<_>>();if src>=0{records[src as usize][..2].fill(0);records[src as usize][6..8].copy_from_slice(&(n("ox")as i16).to_le_bytes());records[src as usize][8..10].copy_from_slice(&(n("oy")as i16).to_le_bytes());}assert_eq!(hash(records.clone()),n("before")as u32);
   let s=g.native_world.as_mut().unwrap();s.effects=records[..capacity].iter().map(|&b|Some(Particle::from_bytes(b))).collect();s.effect_origin=[n("ox")as i16,n("oy")as i16];s.last_projection=[n("px")as i16,n("py")as i16];s.effect_tint=if mode==0{9}else{99};s.effect_flags=if mode==0{128}else{12};s.effect_fade=if mode==0{128}else{3};s.effect_owner=if mode==0{0}else{3};s.effect_sprite=if mode==0{0}else{557};s.effect_suppression=n("gate")as u8;s.effect_maximum=69;g.multiplayer=n("mp")!=0;g.original_rng=OriginalRng{words:std::array::from_fn(|i|0x173510a7u32.wrapping_add(scenario as u32*7919).wrapping_add((i as u32).wrapping_mul(0x9e3779b9))),multiplayer:g.multiplayer};
   if mode==0{assert_eq!(g.emit_native_water_spray(n("count")as i16),n("allocations")as usize);}else{g.retire_native_fragment_water_particle(src as usize);}let s=g.native_world.as_ref().unwrap();for(slot,p)in s.effects.iter().enumerate(){records[slot]=p.as_ref().unwrap().bytes();}assert_eq!(hash(records),n("hash")as u32,"typed Game pool scenario{scenario}");assert_eq!(g.original_rng.words,std::array::from_fn(|i|n(&format!("r{i}"))as u32));assert_eq!(s.effect_origin,[n("originX")as i16,n("originY")as i16]);assert_eq!((s.effect_tint,s.effect_flags,s.effect_fade,s.effect_owner,s.effect_sprite),(9,128,128,0,0));assert_eq!(s.effect_maximum,69);count+=1;
  }assert_eq!(count,1280);
 }
}
