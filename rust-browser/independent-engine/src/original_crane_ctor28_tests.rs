//! Projection of actual57ff0's28 writer; not a new full constructor model.
//! Work9a remains under the existing raw-slot authority, including unknowns.
use super::*;
use crate::{original_route,original_group_game,stats};

fn field()->Game{
 let mut g=Game::new(0,1,33191);g.map.original=true;g.map.width=64;g.map.height=64;g.map.world=0;g.map.tiles=vec![0;4096];g.map.original_tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.bridge_base=vec![-1;4096];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
 g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters=original_group_game::State::default();g.ai_teams.clear();g.events.clear();g.next_id=1;g.winner=None;g
}
fn admission(kind:&str,definition:u16,equipment:u8)->(Game,usize,usize,usize){
 let mut g=field();g.spawn(kind,-1,640.,640.,0,None);let carrier=g.units[0].id;g.objects.push(json!({"type":definition,"equipmentId":equipment,"x":640,"y":640,"nativeHp":(stats(kind).hp as i16/25),"nativeFlags13":0x70}));g.import_native_world(&Value::Null);
 for n in 0..3{g.spawn("grunt",0,640.+n as f64*4.,660.,7,None);}let actor=g.units[1].id;let slot=g.units[1].native_slot.unwrap();g.units[1].hp=17.;g.maintain_native_group_rosters();g.native_group_rosters.observe_unit_context(slot);g.command(&json!({"kind":"enter","ids":[actor],"target":carrier}));(g,carrier,actor,slot)
}
fn takeover(g:&mut Game,carrier:usize,actor:usize,slot:usize){
 let ai=g.units.iter().position(|u|u.id==actor).unwrap();assert!(g.update_native_hardware_entry(ai));let occupied=g.units.iter().find(|u|u.id==carrier).unwrap();assert_eq!(occupied.team,0);assert_eq!(occupied.native_slot,Some(slot));assert_eq!(occupied.crew.as_ref().unwrap().hp,17.);assert!(g.units[ai].native_slot.is_none());
}
#[test]fn crane_ctor28_fresh_spawn_records_the_real_marker_for_both_human_teams(){
 for team in[0,1]{let mut g=field();g.import_native_world(&Value::Null);g.spawn("crane",team,640.,640.,1,None);let u=g.units.last().unwrap();let slot=u.native_slot.unwrap();assert_eq!(g.native_selection.contexts[slot].link28,Some(0x2084));assert_eq!(g.native_crane_work9a(slot,u.id),Ok(0),"only the separate fresh41b210 allocation establishes work0");}
}
#[test]fn crane_ctor28_actual_takeover_updates_the_retained_native_context_without_creating_work9a(){
 let(mut g,carrier,actor,slot)=admission("crane",35,15);g.native_selection.contexts[slot].link28=Some(0x12345678);assert!(g.native_world.as_ref().unwrap().cranes[slot].is_none());takeover(&mut g,carrier,actor,slot);assert_eq!(g.units.iter().find(|u|u.id==carrier).unwrap().kind,"crane");assert_eq!(g.native_selection.contexts[slot].link28,Some(0x2084));assert!(g.native_world.as_ref().unwrap().cranes[slot].is_none(),"57ff0 does not manufacture a fresh9a authority");assert!(g.native_crane_work9a(slot,carrier).is_err());
}
#[test]fn crane_ctor28_projection_matches_all_64_original_warm_constructor_calls(){
 let trace=std::env::var_os("Z_NATIVE_TRACE_DIR").map(|root|std::fs::read_to_string(std::path::PathBuf::from(root).join("V47-crane-ctor28.jsonl")).expect("explicit private trace path"));let original=trace.as_deref().map(|text|text.lines().map(|line|serde_json::from_str::<Value>(line).unwrap()).collect::<Vec<_>>());if let Some(rows)=&original{assert_eq!(rows.len(),64);}
 let mut hash=2166136261u32;
 for scenario in 0..64u32{
  let raw=|offset:u32|(scenario.wrapping_mul(17).wrapping_add(offset*23))as u8;let before=u32::from_le_bytes(std::array::from_fn(|i|raw(40+i as u32)));let work=raw(154);let(mut g,carrier,actor,slot)=admission("crane",35,15);g.native_selection.contexts[slot].link28=Some(before);
  //A retained slot may have known or unknown old9a. This deliberately does
  //not assert an unobserved whole-game history or rebind its browser owner.
  let known=scenario&1==0;g.native_world.as_mut().unwrap().cranes[slot]=Some(crane::Crane{entity:actor,work9a:work,work9a_known:known,..Default::default()});takeover(&mut g,carrier,actor,slot);
  let after=g.native_selection.contexts[slot].link28.expect("known57ff0 literal writer");assert_eq!(after,0x2084);let retained=g.native_world.as_ref().unwrap().cranes[slot].as_ref().unwrap();assert_eq!((retained.work9a,retained.work9a_known,retained.entity),(work,known,actor));assert!(g.native_crane_work9a(slot,carrier).is_err(),"no new9a owner is introduced during renderer rebind");
  if let Some(rows)=&original{let row=&rows[scenario as usize];let bytes=|key:&str|{let text=row[key].as_str().unwrap();(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect::<Vec<_>>()};let b=bytes("before");let a=bytes("after");assert_eq!(u32::from_le_bytes(b[40..44].try_into().unwrap()),before);assert_eq!(u32::from_le_bytes(a[40..44].try_into().unwrap()),after);assert_eq!((b[154],a[154]),(work,retained.work9a));}
  for byte in scenario.to_le_bytes().into_iter().chain(before.to_le_bytes()).chain(after.to_le_bytes()).chain([work,retained.work9a]){hash=(hash^u32::from(byte)).wrapping_mul(16777619);}
 }
 assert_eq!(hash,2744724549,"28/9a projection of64 actual57ff0->41b950 executions; full native raw output separately verified");
}
#[test]fn crane_ctor28_other_takeover_types_keep_their_existing_link_instead_of_getting_a_crane_reset(){
 for(kind,definition,equipment)in[("medium",8,1),("jeep",33,13),("gatling",22,11),("launcher",24,6)]{let(mut g,carrier,actor,slot)=admission(kind,definition,equipment);g.native_selection.contexts[slot].link28=Some(0x12345678);takeover(&mut g,carrier,actor,slot);assert_eq!(g.native_selection.contexts[slot].link28,Some(0x12345678),"{kind}");}
}
#[test]fn crane_ctor28_other_spawn_types_and_unknown_work_keep_their_previous_meaning(){
 let mut g=field();g.import_native_world(&Value::Null);for(kind,expected)in[("grunt",Some(0)),("jeep",Some(0x2084)),("medium",None),("gatling",None)]{g.spawn(kind,0,640.,640.,1,None);let slot=g.units.last().unwrap().native_slot.unwrap();assert_eq!(g.native_selection.contexts[slot].link28,expected,"{kind}");}
}
