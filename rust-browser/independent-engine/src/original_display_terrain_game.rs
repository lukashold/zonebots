//!Mutable terrain prefix at the actual55f10 submission, before visibility.
use crate::{Game,json,original_combat,original_ambient};
use super::{terrain::{self,Input,State,Outcome,World},Record};
use serde_json::Value;
struct Environment<'a>{game:&'a mut Game,index:usize}
impl World for Environment<'_>{
 fn terrain(&mut self,cell:i16)->u8{let c=cell as u16 as usize;let(x,y)=(c&127,c>>7);if x<self.game.map.width&&y<self.game.map.height{self.game.map.terrain_codes.get(y*self.game.map.width+x)}else{self.game.map.native_terrain_plane.get(c)}.copied().unwrap_or(0)}
 fn random(&mut self)->u32{self.game.native_random()}
 fn remove_forced(&mut self){self.game.native_remove_actor(self.index,false,true);}
 fn ambient(&mut self,position:[i16;2]){if let Some(a)=self.game.native_ambient.as_mut(){a.pool.spawn(original_ambient::Spawn{position,template:25,arg4:0,arg5:2,arg6:-1,arg7:-1,arg8:-1,heading:0,delay:0,arg11:16});}}
}
impl Game{
 ///None selects the ordinary renderer; Some(None) is a consumed submission
 ///with no picture. In particular an invisible actor still advances97/RNG.
 pub(crate) fn capture_native_terrain_display(&mut self,index:usize,slot:usize)->Option<Option<(Record,Value)>>{
  let u=self.units.get(index)?;if original_combat::animation_family(&u.kind)!=4{return None;}
  let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));let position=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];
  //4102c0's temporary4a653b belongs to menu-object previews; ordinary UNIT
  //callbacks enter with0. The actual map descriptor is independent of UI's
  //world ordering and comes from the extracted native world byte.
  let world=self.map.wall_tile_lookup["nativeWorld"].as_u64().map(|v|v as u8).unwrap_or([0,3,1,4,2][self.map.world.min(4)]);
  let input=Input{position,camera,size,family:4,disabled:false,world,heading:original_combat::facing(u.angle)as i8};
  let mut state=State{counter:u.native_counter97,phase:u.native_phase35,frame:u.native_frame32,counter82:u.native_pose82,counter84:u.native_counter84,transition:u.native_transition88 as u16,cycles:u.native_pose7e as u16,timer:u.native_pose80 as u8,flags64:u.native_flags64};let before=state.phase;
  let result=terrain::advance(input,&mut state,&mut Environment{game:self,index});
  let u=&mut self.units[index];u.native_counter97=state.counter;u.native_phase35=state.phase;u.native_frame32=state.frame;u.native_pose82=state.counter82;u.native_counter84=state.counter84;u.native_transition88=state.transition as i16;u.native_pose7e=state.cycles as i16;u.native_pose80=state.timer as i8;u.native_flags64=state.flags64;
  if before!=state.phase{u.native_idle=None;u.native_walk=None;u.native_fire=None;u.native_cover=None;if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.promotion.phase=state.phase;}}
  let mut draw=match result{Outcome::Regular|Outcome::Unavailable=>return None,Outcome::Handled(None)=>return Some(None),Outcome::Handled(Some(draw))=>draw};
  //GPU errors for corrupt negative97/negative scissor heights are outside
  //the Canvas adapter; all authored0..16 terrain levels stay nonnegative.
  if draw.layers.iter().any(|l|l.rectangle(0).is_some_and(|r|r[3]<0)){return None;}
  draw.header.y=draw.header.y.wrapping_add(camera[1]);for l in &mut draw.layers{l.position=[l.position[0].wrapping_add(camera[0]),l.position[1].wrapping_add(camera[1])];}
  let header=draw.header.record_for_sort();let layers=draw.layers.iter().map(|l|json!({"sprite":l.sprite,"x":i32::from(l.position[0])*2,"y":i32::from(l.position[1])*2,"palette":l.palette,"flags":l.flags,"clip":l.rectangle(0).unwrap().map(|v|v*2)})).collect::<Vec<_>>();
  Some(Some((header,json!({"kind":"unit","id":u.id,"nativeSlot":slot,"queueY":i32::from(header.y)*2,"palette":255,"sprite":null,"time":self.time,"x":i32::from(position[0])*2,"y":i32::from(position[1])*2,"nativeLayers":layers,"art":{"type":u.kind,"x":i32::from(position[0])*2,"y":i32::from(position[1])*2,"team":u.team,"angle":u.angle,"aim":u.aim,"hp":u.hp,"maxHp":u.max_hp,"cooldown":u.cooldown,"moving":u.moving,"originalCombat":{"fireRate":u.original_combat.as_ref().map_or(80,|c|c.fire_rate)},"nativeHardware":null}}))))
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn field(code:u8)->Game{let mut g=Game::new(0,1,36739);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![code;4096];g.map.native_terrain_plane=vec![code;16384];g.map.wall_tile_lookup=json!({"nativeWorld":0});g.units.clear();g.buildings.clear();g.objects.clear();g.sectors.clear();g.crates.clear();g.walls.clear();g.native_world=None;g.native_group_rosters=Default::default();g.native_deaths=Default::default();g.spawn("grunt",0,1000.,1200.,1,None);g.maintain_native_group_rosters();g.native_view=Some(([0,0],[1024,1024]));g}
 fn capture(g:&mut Game){g.begin_native_display_frame();let slot=g.native_unit_display_slot(0);g.capture_native_unit_display(0,slot);}
 #[test]fn shallow_water_prefix_draws_before_visibility_gate_and_keeps_mask98(){let mut g=field(9);let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().flags69=0;g.units[0].native_flags64|=4;g.units[0].native_mask98=1;let mut expected=g.original_rng.clone();expected.next_u32();capture(&mut g);assert_eq!(g.units[0].native_counter97,1);assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.units[0].native_flags64&4,0);assert_eq!(g.units[0].native_mask98,1);let rows=g.native_display_queue();let r=rows.iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(r["palette"],255);assert_eq!(r["queueY"],1202);assert_eq!(r["nativeLayers"][0]["palette"],4);assert_eq!(r["nativeLayers"][0]["clip"],json!([1000,1202,512,30]));assert_eq!(r["art"]["y"],1200);}
 #[test]fn offscreen_shallow_water_still_spends_rng_and_leaves_white_flag_for_later(){let mut g=field(9);g.native_view=Some(([2000,2000],[640,480]));g.units[0].native_flags64|=4;let mut expected=g.original_rng.clone();expected.next_u32();capture(&mut g);assert_eq!(g.units[0].native_counter97,1);assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.units[0].native_flags64&4,4);assert!(!g.native_display_queue().iter().any(|r|r["kind"]=="unit"));}
 #[test]fn frozen_entry_skips_the_entire_prefix_and_dry_land_recovers_one_level_per_callback(){let mut g=field(0);let slot=g.units[0].native_slot.unwrap();g.units[0].native_counter97=2;g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=64;let rng=g.original_rng.words;capture(&mut g);assert_eq!(g.units[0].native_counter97,2);assert_eq!(g.original_rng.words,rng);g.native_group_rosters.members[slot].as_mut().unwrap().flags69=4;capture(&mut g);assert_eq!(g.units[0].native_counter97,1);capture(&mut g);assert_eq!(g.units[0].native_counter97,0);let row=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert!(row.get("nativeLayers").is_none());assert!(row["sprite"].is_number());}
 #[test]fn native_world_three_removes_through_forced_pool_row_without_rng(){let mut g=field(9);g.map.wall_tile_lookup["nativeWorld"]=json!(3);let rng=g.original_rng.words;capture(&mut g);assert_eq!(g.units[0].native_counter97,1);assert!(g.units[0].native_slot.is_none());assert!(g.units[0].hp<0.);assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_deaths.records[0].sprite,167);assert_eq!(g.native_deaths.records[0].position,[500,600]);assert!(!g.native_display_queue().iter().any(|r|r["kind"]=="unit"));}
 #[test]fn submerged_row_uses_the_actual_ambient_pool_and_constructor(){let mut g=field(42);g.units[0].native_counter97=16;g.native_ambient=Some(Default::default());let seed=(1..10000).find(|&seed|{let mut r=original_combat::OriginalRng::seeded(seed);r.next_u32()as u8&63==0}).unwrap();g.original_rng=original_combat::OriginalRng::seeded(seed);let mut expected=g.original_rng.clone();expected.next_u32();capture(&mut g);assert_eq!(g.original_rng.words,expected.words);let r=&g.native_ambient.as_ref().unwrap().pool.records[0];assert_eq!(r.position(),[500,584]);assert_eq!(r.word(20),4212);assert_eq!(r.byte(51),16);assert_eq!(r.byte(46),2);assert!(!g.native_display_queue().iter().any(|r|r["kind"]=="unit"));}
 #[test]fn water_pose_initializer_writes_phase42_before_its_same_callback_picture(){let mut g=field(9);g.units[0].native_counter97=7;g.units[0].native_phase35=2;g.units[0].native_frame32=3;let seed=(1..200000).find(|&seed|{let mut r=original_combat::OriginalRng::seeded(seed);u32::from(r.next_u32()as u16)%4095==1}).unwrap();g.original_rng=original_combat::OriginalRng::seeded(seed);let mut expected=g.original_rng.clone();expected.next_u32();let cycles=(expected.next_u32()as u16)%3+5;let timer=(expected.next_u32()as u8&1)+3;capture(&mut g);let u=&g.units[0];assert_eq!((u.native_phase35,u.native_frame32,u.native_pose82,u.native_transition88,u.native_counter84,u.native_pose7e,u.native_pose80),(42,0,0,0x443,-1,cycles as i16,timer as i8));assert_eq!(g.original_rng.words,expected.words);let r=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(r["nativeLayers"][0]["sprite"],66+original_combat::facing(u.angle));assert_eq!(r["nativeLayers"][0]["clip"][3],18);}
}
