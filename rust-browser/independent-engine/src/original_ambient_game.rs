//! Ownership adapter for the separate100x62 ambient-target world pool.
use super::original_ambient as ambient;
use super::{Game,json};
use std::collections::BTreeMap;
#[derive(Clone,Copy)]struct PointOrder{point:[i16;2],pending:bool}
struct GroundQueries{
 objects:Vec<super::original_point_fire::GroundObject>,buildings:Vec<super::original_world_contact::Building>,bridges:Vec<super::original_world_contact::Bridge>,flags69:u8,class:u8,family:u8,grenades:u8,
}
pub struct State{
 pub pool:ambient::State,
 pub blockers:Vec<ambient::Blocker>,
 pub draws:Vec<(usize,ambient::Draw)>,
 pub initial_lava:Vec<ambient::LavaSeed>,
 unit_buckets:[Vec<usize>;256],
 unit_ids:BTreeMap<usize,usize>,
 //Read-only projection of raw UNIT positions/classes. Retired renderer IDs
 //may disappear; the underlying native slots retain these bytes until reuse.
 point_targets:Vec<super::original_point_fire::FlameTarget>,
 target_buckets:[Vec<u8>;256],
 point_orders:BTreeMap<usize,PointOrder>,
}

#[cfg(test)]mod tests{
 use super::*;
 use super::super::{original_group_game,original_route,original_point_fire};
 fn field(kind:&str,members:usize)->Game{
  let mut g=Game::new(0,1,17091);let(w,h)=(64,64);g.map.original=true;g.map.width=w;g.map.height=h;g.map.world=0;g.map.tiles=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.native_terrain_plane=vec![0;16384];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
  g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.events.clear();g.ai_teams.clear();g.native_strategy=None;g.native_world=None;g.native_group_rosters=original_group_game::State::default();g.winner=None;g.next_id=1;
  for i in 0..members{g.spawn(kind,0,1000.+i as f64*4.,1000.,1,None);let u=g.units.last_mut().unwrap();u.native_phase35=1;u.angle=super::super::native_math::angle(0);u.aim=u.angle;}
  g.sync_native_world();g.maintain_native_group_rosters();g.native_group_rosters.observe_unit_context(g.units[0].native_slot.unwrap());g.native_ambient=Some(State::default());g
 }
 fn spawn(state:&mut State,p:[i16;2],template:u8)->usize{usize::from(state.pool.spawn(ambient::Spawn{position:p,template,arg4:255,arg5:2,arg6:-1,arg7:0,arg8:-1,heading:0,delay:0,arg11:0}).unwrap())}
 #[test]fn terrain_autoattack_enters_command12_through_the_actual_dispatcher_and_releases_the_donated_grenade(){
  let mut g=field("grunt",2);g.units[0].grenades=0;g.units[1].grenades=1;g.spawn("grunt",1,1800.,1800.,2,None);g.maintain_native_group_rosters();g.import_native_world(&super::super::Value::Null);let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.observe_unit_context(slot);let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[58]=1;g.native_group_rosters.pool[group].roster[44..46].copy_from_slice(&800i16.to_le_bytes());g.map.terrain_codes[32*64+32]=1;
  let context=super::super::original_path_contact::Context{blocked:[32*128+32,-1],proposed:[501,500],force:false,callback_idle:true,current_actor:false,dummy_actor:false};assert_eq!(g.native_autoattack_contact(0,context,|_,_|panic!()),Ok(true));let m=g.native_group_rosters.members[slot].as_ref().unwrap();let intended=[m.promotion.target_x as i16,m.promotion.target_y];assert_eq!((g.units[0].grenades,g.units[1].grenades,g.units[0].native_order5c),(1,0,12));let rng=g.original_rng.words;
  g.dispatch_recorded_native_order(0);assert_eq!(g.units[0].native_order5c,0);assert_eq!(g.units[0].native_phase35,66);assert_eq!(g.units[1].native_phase35,1);assert_eq!(g.native_ambient.as_ref().unwrap().point_orders[&g.units[0].id].point,intended);assert_eq!(g.native_group_rosters.pool[group].roster[58],1);assert_eq!(g.original_rng.words,rng);
  for _ in 0..16{g.step(0.059);if g.units[0].grenades==0{break;}}assert_eq!(g.units[0].grenades,0,"point phase66 must reach its native grenade release through the real UNIT loop");assert!(g.projectiles.iter().any(|p|p.kind==super::super::original_projectiles::Kind::Grenade)||g.events.iter().any(|e|e["kind"]=="shot"&&e["weapon"]=="grenade"));assert_eq!(g.native_route_boundary,None);
 }
 #[test]fn hardware_point_command_acknowledges_flags_and_target_without_inventing_an_infantry_phase(){let mut g=field("medium",1);g.units[0].native_phase35=9;let slot=g.units[0].native_slot.unwrap();let event=super::super::original_orders_game::Event::classified(12,[510,520],-345);assert!(g.record_native_order_event(0,event));let rng=g.original_rng.words;g.dispatch_recorded_native_order(0);assert_eq!((g.units[0].native_order5c,g.units[0].native_phase35),(0,9));let m=g.native_group_rosters.members[slot].as_ref().unwrap();assert_eq!((m.promotion.target_x,m.promotion.target_y,m.promotion.target_aux),(510,520,-345));assert_ne!(m.flags69&32,0);assert!(g.native_ambient.as_ref().unwrap().point_orders.is_empty());assert_eq!(g.original_rng.words,rng);}
 #[test]fn solitary_role0_point_command_keeps_the_original_command_and_does_not_start_group_fire(){let mut g=field("grunt",1);let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().role=0;assert!(g.record_native_order_event(0,super::super::original_orders_game::Event::classified(12,[510,520],0)));g.dispatch_recorded_native_order(0);assert_eq!((g.units[0].native_order5c,g.units[0].native_phase35),(12,1));assert!(g.native_ambient.as_ref().unwrap().point_orders.is_empty());}
 #[test]fn completed_idle_look_uses_target_buckets_and_records_command12_without_replacing_the_group_order(){
  let mut g=field("grunt",1);let target=spawn(g.native_ambient.as_mut().unwrap(),[500,480],2);g.native_ambient.as_mut().unwrap().pool.records[target].set_word(42,i16::from(super::super::original_initiative::native::coarse_cell([500,480])));
  let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[58]=1;g.native_group_rosters.pool[group].roster[44..46].copy_from_slice(&900i16.to_le_bytes());g.prepare_native_ambient_queries();let rng=g.original_rng.words;
  assert!(g.native_idle_object_scan(0,0,true));assert_eq!(g.units[0].native_order5c,12);assert_eq!(g.units[0].native_flags9c&8,8);assert_eq!(g.native_group_rosters.pool[group].roster[58],1);assert_eq!(g.native_group_rosters.pool[group].word(44),900);assert_eq!(g.original_rng.words,rng);
  assert!(g.update_native_ambient_point_fire(0));assert_eq!((g.units[0].native_phase35,g.units[0].native_frame32,g.units[0].native_timer3c),(20,4,3));assert_eq!(g.units[0].native_flags9c&8,0);assert_eq!(g.units[0].native_order5c,0);assert_eq!(g.bullets.len(),1);assert_eq!((g.bullets[0].x,g.bullets[0].y),(1004.,1004.));assert!(g.units[0].native_fire.is_none());
  for _ in 0..8{g.update_native_bullets();}assert!(g.bullets.is_empty());assert!(!g.native_ambient.as_ref().unwrap().pool.records[target].active());
 }
 #[test]fn actors_use_cached_bucket_membership_but_live_positions_and_draws_precede_crushing(){
  let mut g=field("medium",1);g.units[0].x=960.;g.units[0].y=1000.;let slot=spawn(g.native_ambient.as_mut().unwrap(),[500,500],2);g.prepare_native_ambient_queries();g.units[0].x=1002.;g.units[0].y=1002.;g.update_native_ambient();let state=g.native_ambient.as_ref().unwrap();assert_eq!(state.draws[0].1.flags53&32,0);assert_eq!(state.pool.records[slot].byte(53)&32,32);g.update_native_ambient();let visuals=g.native_ambient_visuals();assert_eq!(visuals[0]["queueY"],1000);assert_eq!(visuals[0]["y"],1032);assert_eq!(visuals[0]["flags53"].as_u64().unwrap()&32,32);
 }
 #[test]fn current_slot_emitter_reuse_does_not_advance_twice_but_later_allocations_do_advance(){
  let mut g=field("grunt",1);g.units[0].x=200.;g.units[0].y=200.;let slot=spawn(g.native_ambient.as_mut().unwrap(),[500,500],0);let r=&mut g.native_ambient.as_mut().unwrap().pool.records[slot];r.set_byte(45,9);r.set_byte(46,1);r.set_byte(47,0);r.set_byte(55,127);r.set_word(18,21);g.prepare_native_ambient_queries();g.update_native_ambient();let s=g.native_ambient.as_ref().unwrap();assert_eq!(s.pool.records[0].word(26),2172);assert_eq!(s.pool.records[0].byte(46),2);assert_eq!(s.pool.records[1].word(26),2162);assert_eq!(s.pool.records[1].byte(46),1);assert_eq!(s.draws.iter().map(|v|v.0).collect::<Vec<_>>(),[0,1]);
 }
 #[test]fn pyro_command12_keeps_the_early_branch_override_and_stops_at_the_first_grenade_member(){
  let mut g=field("pyro",3);g.units[0].grenades=2;g.units[0].native_order5c=12;g.units[0].native_flags9c=8;g.native_ambient.as_mut().unwrap().point_orders.insert(g.units[0].id,PointOrder{point:[500,470],pending:true});assert!(g.update_native_ambient_point_fire(0));assert_eq!((g.units[0].native_phase35,g.units[0].native_frame32,g.units[0].native_timer3c),(66,0,2));assert_eq!(g.units[0].native_flags9c&8,8);assert_eq!((g.units[1].native_phase35,g.units[2].native_phase35),(1,1));
  for _ in 0..8{g.update_native_ambient_point_fire(0);}assert_eq!(g.units[0].grenades,1);assert_eq!(g.projectiles.len(),1);assert_eq!(g.projectiles[0].kind,super::super::original_projectiles::Kind::Grenade);
 }
 #[test]fn only_the_tough_roster_head_receives_the_source_point_rocket_state(){
  let mut g=field("tough",2);g.begin_native_ambient_point_order(0,[500,470]);assert_eq!((g.units[0].native_phase35,g.units[1].native_phase35),(21,1));assert_eq!(g.native_ambient.as_ref().unwrap().point_orders.len(),1);
  for _ in 0..2{g.update_native_ambient_point_fire(0);}assert_eq!(g.projectiles.len(),1);assert_eq!(g.projectiles[0].kind,super::super::original_projectiles::Kind::Rocket);assert!(g.units[0].native_fire.is_none());let _=original_point_fire::Action::Wait;
 }
 #[test]fn mixed_phase20_pyro_uses_the_real_retained_encoded_unit46_and_the_old_muzzle_heading(){
  for mp in[false,true]{let mut g=field("grunt",2);g.multiplayer=mp;g.units[1].kind="pyro".into();g.units[1].native_class7c=15;g.units[1].x=1000.;g.units[1].y=1200.;g.units[1].original_combat=Some(super::super::original_combat::CombatState::new("pyro"));g.units[1].original_combat.as_mut().unwrap().aim_skill=0;g.units[1].native_target46=0x5502;g.spawn("cannon",1,1360.,1020.,2,None);g.maintain_native_group_rosters();g.native_group_rosters.observe_unit_context(g.units[0].native_slot.unwrap());g.original_rng.words=[1,2,3,4,5];g.begin_native_ambient_point_order(0,[500,570]);assert_eq!(g.units[1].native_phase35,20);assert_eq!(g.units[1].native_target46,0x5502);assert!(g.update_native_ambient_point_fire(1));assert_eq!(g.units[1].native_target46,0x5502);assert_eq!(g.units[1].native_frame32,4);assert_eq!(g.units[1].native_timer3c,3);assert_eq!(g.units[1].aim,super::super::native_math::angle(2));assert_eq!(g.projectiles.len(),1);let p=&g.projectiles[0];assert_eq!((p.x,p.y),(996.,1170.));assert_eq!(p.kind,super::super::original_projectiles::Kind::Flame);let target=if mp{(688,522)}else{(682,523)};assert_eq!(p.angle,super::super::native_math::projectile_angle(target.0-498,target.1-585));assert_eq!(g.original_rng.words,if mp{[1,2,3,4,5]}else{[952858006,4149490296,1,2,4943]});}
 }
 #[test]fn explicit_pyro_point_phase_keeps_unit46_until_the_shot_then_clears_it_without_target_scatter(){
  let mut g=field("pyro",1);g.units[0].native_target46=0x5501;g.begin_native_ambient_point_order(0,[500,470]);assert_eq!(g.units[0].native_phase35,73);assert_eq!(g.units[0].native_target46,0x5501);let rng=g.original_rng.words;assert!(g.update_native_ambient_point_fire(0));assert_eq!(g.units[0].native_target46,0x5501);assert!(g.projectiles.is_empty());assert!(g.update_native_ambient_point_fire(0));assert_eq!(g.units[0].native_target46,-1);assert_eq!(g.original_rng.words,rng);assert_eq!(g.projectiles.len(),1);
 }
 #[test]fn raw_point_target_slots_keep_inactive_coordinates_until_reused_and_ignore_team_or_alive_filters(){
  let mut g=field("pyro",1);assert_eq!(g.native_raw_unit_position(239),Some([0,0]));assert_eq!(g.native_raw_unit_position(240),None);g.spawn("medium",1,1360.,1020.,2,None);g.maintain_native_group_rosters();let slot=g.units[1].native_slot.unwrap()as u8;g.prepare_native_ambient_queries();assert!(g.native_direct_removal(1,[680,510],false,false));g.units.retain(|u|super::super::original_combat::alive(true,u.hp));assert_eq!(g.native_point_target(slot),Some(original_point_fire::FlameTarget{position:[680,510],class:1}));assert_eq!(g.native_raw_unit_position(usize::from(slot)),Some([680,510]));g.spawn("cannon",0,800.,900.,3,None);assert_eq!(g.units.last().unwrap().native_slot,Some(usize::from(slot)));assert_eq!(g.native_point_target(slot),Some(original_point_fire::FlameTarget{position:[400,450],class:2}));assert_eq!(g.native_raw_unit_position(usize::from(slot)),Some([400,450]));
 }
 #[test]fn build_flame_uses_raw54_56_as_origin_and_the_original_neighbor_selection_in_both_modes(){
  for mp in[false,true]{let mut g=field("pyro",1);g.multiplayer=mp;g.units[0].x=600.;g.units[0].y=800.;g.units[0].original_combat.as_mut().unwrap().aim_skill=60;g.units[0].native_target46=123;g.units[0].angle=super::super::native_math::angle(6);g.original_rng.words=[7,2,3,4,5];let cell=(601/16)*g.map.width+(502/16)+1;g.map.terrain_codes[cell]=8;assert!(g.emit_native_ground_flame(0,[502,601]));assert_eq!(g.projectiles.len(),1);let p=&g.projectiles[0];assert_eq!((p.x,p.y),(1004.,1202.));assert_eq!(p.kind,super::super::original_projectiles::Kind::Flame);assert_eq!(p.angle,super::super::native_math::projectile_angle(16,-8));assert_eq!(g.units[0].native_target46,123);assert_eq!(g.units[0].angle,super::super::native_math::angle(6));assert_eq!(g.original_rng.words,if mp{[7,2,3,4,5]}else{[4149520986,7,2,3,1]});}
 }
 #[test]fn build_flame_live_build_contact_and_zero_speed_keep_the_native_no_projectile_results(){
  let mut g=field("pyro",1);g.import_native_world(&super::super::json!({"nativeWorldBuildings":[{"nativeSlot":0,"kind":4,"x":494,"y":593,"hp":100,"maxHp":100}]}));let rng=g.original_rng.words;assert!(!g.emit_native_ground_flame(0,[502,601]));assert!(g.projectiles.is_empty());assert_eq!(g.original_rng.words,rng,"live BUILD accepts the original point;464a20 then rejects the identical endpoints");
  let mut g=field("pyro",1);g.units[0].original_combat.as_mut().unwrap().projectile_speed=Some(0);g.original_rng.words=[7,2,3,4,5];let cell=(601/16)*g.map.width+(502/16)+1;g.map.terrain_codes[cell]=8;assert!(!g.emit_native_ground_flame(0,[502,601]));assert!(g.projectiles.is_empty());assert_eq!(g.original_rng.words,[4149520986,7,2,3,1],"selection RNG precedes464a20's speed gate");
 }
 #[test]fn both_point_phases_use_the_build_branch_when_raw50_retains_the_ground_sentinel(){
  for phase in[20,73]{for mp in[false,true]{let mut g=field("pyro",1);g.multiplayer=mp;g.units[0].native_phase35=phase;g.units[0].native_frame32=if phase==20{3}else{0};g.units[0].native_timer3c=if phase==20{1}else{0};g.units[0].native_target46=123;g.units[0].angle=super::super::native_math::angle(6);g.units[0].aim=g.units[0].angle;g.units[0].original_combat.as_mut().unwrap().aim_skill=60;g.original_rng.words=[7,2,3,4,5];let slot=g.units[0].native_slot.unwrap();let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.promotion.target_x=3456;m.promotion.target_y=502;m.promotion.target_aux=601;g.native_ambient.as_mut().unwrap().point_orders.insert(g.units[0].id,PointOrder{point:[900,900],pending:false});let cell=(601/16)*g.map.width+(502/16)+1;g.map.terrain_codes[cell]=8;
   assert!(g.update_native_ambient_point_fire(0));assert_eq!(g.projectiles.len(),1);assert_eq!((g.projectiles[0].x,g.projectiles[0].y),(1004.,1202.));assert_eq!(g.projectiles[0].angle,super::super::native_math::projectile_angle(16,-8));assert_eq!(g.units[0].native_target46,if phase==20{123}else{-1});assert_eq!(g.units[0].angle,super::super::native_math::angle(6));assert_eq!(g.original_rng.words,if mp{[7,2,3,4,5]}else{[4149520986,7,2,3,1]});
  }}
 }
}
impl Default for State{fn default()->Self{Self{pool:Default::default(),blockers:vec![ambient::Blocker::default();50],draws:vec![],initial_lava:vec![],unit_buckets:std::array::from_fn(|_|vec![]),unit_ids:BTreeMap::new(),point_targets:vec![Default::default();240],target_buckets:std::array::from_fn(|_|vec![]),point_orders:BTreeMap::new()}}}

struct Startup<'a>{game:&'a mut Game,lava:&'a mut Vec<ambient::LavaSeed>}
impl ambient::StartupWorld for Startup<'_>{
 fn random(&mut self)->u32{self.game.native_random()}
 fn native_world(&self)->u8{self.game.map.wall_tile_lookup["nativeWorld"].as_u64().map(|n|n as u8).unwrap_or([0,3,1,4,2][self.game.map.world.min(4)])}
 fn terrain_cell(&self,cell:i16)->u8{self.game.ambient_terrain_cell(cell)}
 fn set_terrain_cell(&mut self,cell:i16,code:u8){self.game.set_ambient_terrain_cell(cell,code);}
 fn lava_seed(&mut self,seed:ambient::LavaSeed){self.lava.push(seed);self.game.spawn_native_lava(seed);}
}
struct Emitter<'a>{game:&'a mut Game}
impl super::original_ambient_emitters::World for Emitter<'_>{
 fn random(&mut self)->u32{self.game.native_random()}
 fn chunk(&mut self,kind:u8,point:[i16;2],source35:u8){self.game.spawn_native_ambient_chunk(kind,point,source35);}
}
struct Runtime<'a>{game:&'a mut Game,pool:&'a mut ambient::State,slot:usize,blockers:&'a[ambient::Blocker],unit_buckets:&'a[Vec<usize>;256],unit_ids:&'a BTreeMap<usize,usize>}
impl Runtime<'_>{
 fn with_pool(&mut self,r:&mut ambient::Record,operation:impl FnOnce(&mut ambient::State,&mut Game)){
  self.pool.records[self.slot]=r.clone();operation(self.pool,self.game);*r=std::mem::take(&mut self.pool.records[self.slot]);
 }
}
impl ambient::World for Runtime<'_>{
 fn random(&mut self)->u32{self.game.native_random()}
 fn terrain(&self,p:[i16;2])->u8{self.game.ambient_terrain_cell((p[1]>>4).wrapping_mul(128).wrapping_add(p[0]>>4))}
 fn nearby_actors(&self,p:[i16;2])->Vec<ambient::Actor>{
  let origin=(p[1]>>3&!15).wrapping_add(p[0]>>7);let mut total=0i16;let mut actors=vec![];
  for delta in[-17,-16,-15,-1,0,1,15,16,17]{let bucket=origin.wrapping_add(delta)as u16;if bucket>=256{continue;}let ids=&self.unit_buckets[usize::from(bucket)];total=total.wrapping_add(ids.len()as i16);if total>=288{continue;}
   for &slot in ids{if let Some(u)=self.game.units.iter().find(|u|u.native_slot==Some(slot)&&super::original_combat::alive(true,u.hp)).or_else(||self.game.units.iter().find(|u|u.native_slot==Some(slot))).or_else(||self.unit_ids.get(&slot).and_then(|&id|self.game.units.iter().find(|u|u.id==id))){let inside=u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside);actors.push(ambient::Actor{position:[if inside{10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16],class:u.native_class7c,phase:u.native_phase35});}}
  }actors
 }
 fn blocker(&self,p:[i16;2])->bool{self.blockers.iter().any(|b|b.position()[0]!=-1&&b.position()==p)}
 fn hazard_near(&self,p:[i16;2])->bool{self.game.native_lava_near(p)}
 fn secondary_at(&mut self,r:&mut ambient::Record,p:[i16;2],heading:u8){let slot=self.slot;self.with_pool(r,|pool,game|super::original_ambient_emitters::emit(pool,slot,p,heading,&mut Emitter{game}));}
 fn water_contact(&mut self,r:&mut ambient::Record){self.game.spawn_native_ambient_water(r.position());}
 fn delayed_mark(&mut self,r:&mut ambient::Record,delay:u16){let p=r.position();if self.terrain(p)&127==0{self.with_pool(r,|pool,_|{pool.spawn(ambient::Spawn{position:p.map(|n|n&!15),template:6,arg4:0,arg5:2,arg6:127,arg7:3,arg8:-1,heading:0,delay:delay as i16,arg11:0});});}}
}
impl Game{
 fn ambient_terrain_cell(&self,cell:i16)->u8{
  let i=cell as u16 as usize;let(x,y)=(i&127,i>>7);let raw=self.map.native_terrain_plane.get(i).copied().unwrap_or(127);
  if x<self.map.width&&y<self.map.height{(self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(127)&127)|(raw&128)}else{raw}
 }
 fn set_ambient_terrain_cell(&mut self,cell:i16,code:u8){
  let i=cell as u16 as usize;let(x,y)=(i&127,i>>7);let changed=self.ambient_terrain_cell(cell)!=code;
  if let Some(c)=self.map.native_terrain_plane.get_mut(i){*c=code;}
  if x<self.map.width&&y<self.map.height{if let Some(c)=self.map.terrain_codes.get_mut(y*self.map.width+x){*c=code&127;}}
  if changed{self.map.revision=self.map.revision.wrapping_add(1);}
 }
 pub(crate) fn initialize_native_ambient(&mut self)->Result<(),String>{
  if !self.map.original||self.map.native_road_graph.is_none(){self.native_ambient=None;return Ok(());}
  let mut state=self.native_ambient.take().unwrap_or_default();
  //460430 resets only the live marker. Other bytes survive pool recycling.
  for record in &mut state.pool.records{record.set_word(0,-1);}
  for blocker in &mut state.blockers{blocker.bytes[..2].copy_from_slice(&(-1i16).to_le_bytes());}
  state.draws.clear();state.initial_lava.clear();state.point_orders.clear();
  let mut objects=self.native_world_scan_objects().into_iter().map(|o|ambient::SceneObject{slot:o.slot,position:o.position,sprite:o.sprite,flags13:o.flags13}).collect::<Vec<_>>();
  let mut world=Startup{game:self,lava:&mut state.initial_lava};
  let result=state.pool.initialize_scene(&mut objects,&mut state.blockers,&mut world).map_err(str::to_string);
  for object in objects{world.game.apply_native_ambient_object(object.slot,object.position,object.flags13);}
  self.native_ambient=Some(state);result
 }
 ///456a90/4064f0 build capped native spatial buckets before the UNIT loop.
 ///Ambient records deliberately use their stored42 from the preceding tick.
 pub(crate) fn prepare_native_ambient_queries(&mut self){
  //456a90 rebuilds the global UNIT buckets independently of4060f0's
  //ambient scene setup. Native support worlds can own no ambient records;
  //allocating their cache here performs no scene scan, RNG or UNIT callback.
  if self.native_ambient.is_none()&&(!self.map.original||self.map.native_road_graph.is_none()){return;}
  let mut state=self.native_ambient.take().unwrap_or_default();for bucket in &mut state.unit_buckets{bucket.clear();}for bucket in &mut state.target_buckets{bucket.clear();}state.unit_ids.clear();
  let mut units=self.units.iter().filter_map(|u|u.native_slot.map(|slot|(slot,u))).collect::<Vec<_>>();units.sort_by_key(|&(slot,_)|slot);
  for(slot,u)in units{if let Some(target)=state.point_targets.get_mut(slot){*target=super::original_point_fire::FlameTarget{position:[if u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside){10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16],class:u.native_class7c};}let Some(m)=self.native_group_rosters.members.get(slot).and_then(Option::as_ref)else{continue;};if m.flags31&4==0||m.group==255||self.native_group_rosters.pool[usize::from(m.group)].roster[9]&32!=0{continue;}
   let bucket=u.native_bucket38;let ids=&mut state.unit_buckets[usize::from(bucket)];if ids.len()<32{ids.push(slot);state.unit_ids.insert(slot,u.id);}
  }
  for(slot,r)in state.pool.records.iter().enumerate().filter(|(_,r)|r.active()){if let Ok(bucket)=usize::try_from(r.word(42)){if let Some(ids)=state.target_buckets.get_mut(bucket){if ids.len()<32{ids.push(slot as u8);}}}}
  self.native_ambient=Some(state);
 }
 pub(crate) fn native_cached_unit_buckets(&self)->Option<&[Vec<usize>;256]>{Some(&self.native_ambient.as_ref()?.unit_buckets)}
 pub(crate) fn native_cached_unit_id(&self,slot:usize)->Option<usize>{self.native_ambient.as_ref()?.unit_ids.get(&slot).copied()}
 ///Preserve the raw UNIT origin before a Web death/conversion moves its
 ///renderer to a parked OBJECT origin or discards that renderer altogether.
 pub(crate) fn remember_native_point_target(&mut self,slot:usize,position:[i16;2],class:u8){if let Some(target)=self.native_ambient.as_mut().and_then(|s|s.point_targets.get_mut(slot)){*target=super::original_point_fire::FlameTarget{position,class};}}
 ///Raw UNIT coordinates for native slot queries; retired slots retain their cached bytes.
 pub(crate) fn native_raw_unit_position(&self,slot:usize)->Option<[i16;2]>{if slot>=240{return None;}if let Some(u)=self.units.iter().find(|u|u.native_slot==Some(slot)){return Some([if u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside){10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16]);}self.native_ambient.as_ref()?.point_targets.get(slot).map(|t|t.position)}
 fn native_point_target(&mut self,slot:u8)->Option<super::original_point_fire::FlameTarget>{let slot=usize::from(slot);if slot>=240{return None;}if let Some(u)=self.units.iter().find(|u|u.native_slot==Some(slot)){let point=[if u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside){10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16];let class=u.native_class7c;self.remember_native_point_target(slot,point,class);}self.native_ambient.as_ref()?.point_targets.get(slot).copied()}
 ///435dde visits active100 records in slot order after all UNIT updates.
 ///An emitter can allocate a later slot that runs during this same callback.
 pub(crate) fn update_native_ambient(&mut self){
  let Some(mut state)=self.native_ambient.take()else{return;};state.draws.clear();
  for slot in 0..state.pool.records.len(){if !state.pool.records[slot].active(){continue;}let mut record=std::mem::take(&mut state.pool.records[slot]);let mut world=Runtime{game:self,pool:&mut state.pool,slot,blockers:&state.blockers,unit_buckets:&state.unit_buckets,unit_ids:&state.unit_ids};if let Some(draw)=record.advance(&mut world){state.draws.push((slot,draw));}state.pool.records[slot]=record;}
  self.native_ambient=Some(state);
 }
 ///44d140 is queried only after the completed idle look. It neither cycles
 ///the object scan's A4 nor toggles9c40. The target pool is not OBJECT200.
 pub(crate) fn native_idle_ambient_scan(&mut self,index:usize)->bool{
  use super::original_initiative::native as scan;
  let Some(u)=self.units.get(index)else{return false;};let Some(state)=self.native_ambient.as_ref()else{return false;};let Some(member)=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref)else{return false;};
  if member.flags8a&16!=0||u.native_flags64&32!=0{return false;}
  let position=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let direction=super::original_combat::facing(u.aim);let targets=state.pool.records.iter().enumerate().filter(|(_,r)|r.active()).map(|(slot,r)|scan::Target{slot:slot as u8,position:r.position(),flags34:r.byte(52),flags35:r.byte(53),phase3c:r.byte(60)}).collect::<Vec<_>>();
  let candidates=scan::target_candidates(&state.target_buckets,scan::coarse_cell(position),direction);let cell=(position[1].wrapping_add(8)>>4).wrapping_mul(128).wrapping_add(position[0].wrapping_add(8)>>4);
  let Some(choice)=scan::target_scan(&targets,&candidates,position,direction,|slot|{let target=targets.iter().find(|t|t.slot==slot).unwrap().position;scan::visible(position,u.native_class7c,cell,target,|cell|super::native_weapon_data::TERRAIN_HEIGHT.get(usize::from(self.ambient_terrain_cell(cell)&127)).copied().unwrap_or(0)as i8)})else{return false;};
  let weapon=if u.kind=="apc"{u.crew.as_ref().map_or(u.kind.as_str(),|c|c.kind.as_str())}else{&u.kind};let range=(((super::stats(weapon).range/2.)as i32>>3)*2)as i8;if i32::from(choice.distance)*2>=i32::from(range){return false;}
  let point=state.pool.records[usize::from(choice.slot)].position();let id=u.id;
  //437a60's accepted command12 keeps the roster's primary command intact.
  let event=super::original_orders_game::Event::classified(12,point,i16::from(choice.slot as i8));if !self.record_native_order_event(index,event){return false;}
  self.units[index].native_flags9c|=8;if let Some(slot)=self.units[index].native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags9c=self.units[index].native_flags9c;}}
  self.native_ambient.as_mut().unwrap().point_orders.insert(id,PointOrder{point,pending:true});true
 }
 fn begin_native_ambient_point_order(&mut self,index:usize,point:[i16;2]){
  use super::original_point_fire as fire;
  let u=self.units[index].clone();let role=u.native_slot.and_then(|s|self.native_group_rosters.role(s)).unwrap_or(0);
  if role==0{return;}
  self.units[index].native_order5c=0;if let Some(slot)=u.native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.promotion.order=0;}}
  if u.native_family()!=4{if let Some(slot)=u.native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags69|=32;}}return;}
  let Some(context)=self.native_group_rosters.current_roster else{return;};let ids=(0..21).filter_map(|role|self.native_group_rosters.pool[context].member(role)).collect::<Vec<_>>();
  let mut members=vec![];let mut indices=vec![];let mut headings=BTreeMap::new();
  for slot in ids{let Some((i,v))=self.units.iter().enumerate().find(|(_,v)|v.native_slot==Some(usize::from(slot)))else{continue;};let m=self.native_group_rosters.members[usize::from(slot)].as_ref();
   let inventory=m.map_or(false,|m|{(0..21).filter_map(|role|self.native_group_rosters.pool[usize::from(m.group)].member(role)).filter_map(|s|self.units.iter().find(|p|p.native_slot==Some(usize::from(s)))).fold(0u8,|a,p|a.wrapping_add(p.grenades as u8))!=0});
   let p=[(v.x/2.).floor()as i16,(v.y/2.).floor()as i16];headings.insert(slot,super::native_math::direction(i32::from(point[0])-i32::from(p[0]),i32::from(point[1])-i32::from(p[1])));members.push(fire::Member{slot,live:m.is_some_and(|m|m.flags31&4!=0),phase:v.native_phase35,frame:v.native_frame32,timer:v.native_timer3c,heading:super::original_combat::facing(v.aim),point,grenades:v.grenades as u8,group_grenades:inventory,touched:false});indices.push(i);
  }
  fire::begin(u.native_class7c,role,&mut self.units[index].native_flags9c,point,&mut members,|slot|headings[&slot]);
  for(m,i)in members.into_iter().zip(indices){let v=&mut self.units[i];if !m.touched{continue;}v.native_phase35=m.phase;v.native_frame32=m.frame;v.native_timer3c=m.timer;v.aim=super::native_math::angle(m.heading);v.angle=v.aim;
   if matches!(m.phase,20|21|66|73|74){v.native_idle=None;v.native_walk=None;v.native_fire=None;v.moving=false;self.native_ambient.as_mut().unwrap().point_orders.insert(v.id,PointOrder{point:m.point,pending:false});}
  }
 }
 ///54160's actual opcode12 branch.51330 already stored50/54; only the
 ///infantry family4 invokes53b80. Hardware keeps its current phase, sets
 ///69:20 and clears the personal command through53ff0.
 pub(crate) fn dispatch_native_point_order(&mut self,index:usize)->Result<bool,super::original_path_contact::game::Boundary>{
  use super::original_path_contact::game::Boundary;
  let u=self.units.get(index).ok_or(Boundary::MissingActor)?.clone();let slot=u.native_slot.ok_or(Boundary::MissingActor)?;let m=self.native_group_rosters.members[slot].as_ref().ok_or(Boundary::MissingMember)?;let role=m.role;let point=[m.promotion.target_x as i16,m.promotion.target_y];
  if u.native_family()!=4{let m=self.native_group_rosters.members[slot].as_mut().unwrap();m.flags69|=32;m.promotion.order=0;self.units[index].native_order5c=0;return Ok(false);}
  if role==0{return Ok(false);}if self.native_ambient.is_none(){return Err(Boundary::MissingPointState);}if self.native_group_rosters.current_roster.is_none(){return Err(Boundary::MissingGroup(i16::from(m.group)));}
  self.native_ambient.as_mut().unwrap().point_orders.remove(&u.id);self.begin_native_ambient_point_order(index,point);Ok(true)
 }
 ///54160's command12 initialization is followed by its chosen phase handler
 ///during the same native callback. Later roster slots see that new phase.
 pub(crate) fn update_native_ambient_point_fire(&mut self,index:usize)->bool{
  use super::original_point_fire::{State as Fire,Action};
  let Some(u)=self.units.get(index)else{return false;};let Some(mut order)=self.native_ambient.as_ref().and_then(|s|s.point_orders.get(&u.id)).copied()else{return false;};let id=u.id;
  if order.pending{if u.native_order5c!=12{self.native_ambient.as_mut().unwrap().point_orders.remove(&id);return false;}self.native_ambient.as_mut().unwrap().point_orders.remove(&id);self.begin_native_ambient_point_order(index,order.point);let Some(active)=self.native_ambient.as_ref().and_then(|s|s.point_orders.get(&id)).copied()else{return false;};order=active;}
  let u=self.units[index].clone();if !matches!(u.native_phase35,20|21|66|73|74)||u.team<0||!super::original_combat::alive(true,u.hp){self.native_ambient.as_mut().unwrap().point_orders.remove(&id);return false;}
  let rate=u.original_combat.as_ref().map_or(80,|c|c.fire_rate);let mut state=Fire{phase:u.native_phase35,class:u.native_class7c,frame:u.native_frame32,timer:u.native_timer3c,cooldown:if u.cooldown>0.{1}else{0},rate,target46:u.native_target46};let action=state.advance();let v=&mut self.units[index];v.native_frame32=state.frame;v.native_timer3c=state.timer;
  self.units[index].native_target46=state.target46;
  match action{Action::Wait=>{},Action::Fire(shot)=>{self.units[index].cooldown=f64::from(state.cooldown)/17.;self.emit_native_point_shot(index,order.point,shot);},Action::SyncAndIdle|Action::Idle=>{self.native_ambient.as_mut().unwrap().point_orders.remove(&id);if action==Action::SyncAndIdle{self.sync_native_group_order(index);}self.reset_native_idle(index);}}
  true
 }
 fn emit_native_point_shot(&mut self,index:usize,point:[i16;2],shot:super::original_point_fire::Shot){
  use super::original_point_fire::Shot;use super::original_projectiles::Kind;use super::native_weapon_data as data;
  if shot==Shot::Flame{if let Some(ground)=self.native_ground_attack_point(index){self.emit_native_ground_flame(index,ground);return;}}
  let u=self.units[index].clone();let p=[(u.x/2.).floor()as i32,(u.y/2.).floor()as i32];let heading=usize::from(super::original_combat::facing(u.aim));let range=((super::stats(&u.kind).range/2.)as i32>>3)*8;let mut target=point.map(i32::from);
  let mut attack=super::original_combat::Attack::from_unit(&u,usize::MAX,if shot==Shot::Grenade{50.}else{super::stats(&u.kind).damage},u.aim);
  let launched=match shot{
   Shot::Bullet|Shot::Laser=>{let dx=p[0]-target[0];let dy=p[1]-target[1];if f64::from(((dx*dx+dy*dy)>>6)as f32).sqrt()as f32 as i32>6{let radius=(self.native_random()&65535)/65535;let a=(self.native_random()&511)as i32;target[0]+=(super::native_math::sine(a)*radius as i32)>>10;target[1]+=(super::native_math::sine(a+128)*radius as i32)>>10;}attack.bullet=true;self.emit_native_bullet(&u,(p[0]+2,p[1]+2),(target[0],target[1]),range+8,shot==Shot::Laser,attack)},
   Shot::Rocket=>{attack.bullet=false;let o=data::TOUGH_MUZZLE[heading];self.emit_native_projectile(&u,(p[0]+o[0],p[1]+o[1]),(target[0],target[1]),Kind::Rocket,10,range,attack)},
   Shot::Flame=>{attack.bullet=false;let resolved=if u.native_target46>=0{self.native_point_target(u.native_target46 as u8)}else{None};let flags69=u.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).map_or(0,|m|m.flags69);let plan=super::original_point_fire::point_flame(super::original_point_fire::FlameInput{position:p.map(|v|v as i16),heading:heading as u8,point,target46:u.native_target46,aim:u.original_combat.as_ref().map_or(80,|c|c.aim_skill),flags69,counter97:u.native_counter97 as u8,speed:u.original_combat.as_ref().and_then(|c|c.projectile_speed).unwrap_or(13)as i8},|_|resolved,||self.native_random());self.units[index].angle=super::native_math::angle(plan.heading);self.units[index].aim=self.units[index].angle;attack.facing=plan.heading;self.emit_native_projectile(&u,(plan.origin[0],plan.origin[1]),(plan.target[0],plan.target[1]),Kind::Flame,i32::from(plan.speed),range,attack)},
   Shot::Grenade=>{attack.bullet=false;let o=data::GRENADE_MUZZLE[heading];let speed=u.original_combat.as_ref().map_or(40,|c|c.movement_control)as i8>>3;let made=self.emit_native_projectile(&u,(p[0]+o[0],p[1]+o[1]),(target[0]+8,target[1]+8),Kind::Grenade,if speed==0{1}else{i32::from(speed)},range,attack);if made{self.units[index].grenades=self.units[index].grenades.saturating_sub(1);if let Some(p)=self.projectiles.last_mut(){p.sprite=1336;}}made},
  };
  if launched{self.events.push(json!({"kind":"shot","type":u.kind,"weapon":if shot==Shot::Grenade{"grenade"}else{"primary"},"team":u.team,"x":u.x,"y":u.y}));}
 }
 ///56840 with UNIT50==3456, called at the actual ground-fire release.
 ///The raw54/56 point is both the selection input and the projectile origin;
 ///a rejected target or464a20's zero-distance/speed gate emits no projectile.
 pub(crate) fn emit_native_ground_flame(&mut self,index:usize,point:[i16;2])->bool{
  use super::original_point_fire as fire;
  let Some(u)=self.units.get(index).cloned()else{return false;};if !self.map.original||u.native_class7c!=15{return false;}let Some(query)=self.native_ground_queries(index)else{return false;};let mut rng=self.original_rng.clone();rng.multiplayer=self.multiplayer;
  let plan=fire::build_flame(fire::FlameInput{position:[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],heading:super::original_combat::facing(u.angle),point,target46:u.native_target46,aim:u.original_combat.as_ref().map_or(80,|c|c.aim_skill),flags69:query.flags69,counter97:u.native_counter97 as u8,speed:u.original_combat.as_ref().and_then(|c|c.projectile_speed).unwrap_or(13)as i8},|kind,p|self.native_ground_query(&query,kind,p),||rng.next_u32());self.original_rng=rng;let Some(plan)=plan else{return false;};if plan.shot.speed==0{return false;}let d=self.native_unit_stats(&u);let range=((d.range/2.)as i32>>3)*8;let mut attack=super::original_combat::Attack::from_unit(&u,usize::MAX,d.damage,u.aim);attack.bullet=false;let made=self.emit_native_projectile(&u,(plan.shot.origin[0],plan.shot.origin[1]),(plan.shot.target[0],plan.shot.target[1]),super::original_projectiles::Kind::Flame,i32::from(plan.shot.speed),range,attack);if made{self.events.push(json!({"kind":"shot","type":u.kind,"weapon":"primary","team":u.team,"x":u.x,"y":u.y}));}made
 }
 fn native_ground_queries(&self,index:usize)->Option<GroundQueries>{let u=self.units.get(index)?;Some(GroundQueries{objects:self.native_world_scan_objects().into_iter().map(|o|super::original_point_fire::GroundObject{position:o.position,definition:o.definition,flags18:o.template_flags18}).collect(),buildings:self.native_world_buildings()?,bridges:self.native_world_bridges()?,flags69:u.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).map_or(0,|m|m.flags69),class:u.native_class7c,family:u.native_family(),grenades:self.native_click_grenade_total(index)?})}
 fn native_ground_query(&self,q:&GroundQueries,kind:super::original_point_fire::GroundQuery,p:[i16;2])->bool{
  use super::original_point_fire::{self as fire,GroundQuery};use super::original_world_contact as world;
  match kind{GroundQuery::LiveBridge=>q.bridges.iter().any(|b|world::bridge_contains(*b,p)),GroundQuery::DestroyedBridge=>q.bridges.iter().any(|b|b.position[0]!=-1&&b.flags15&2!=0&&world::bridge_contains(world::Bridge{flags15:b.flags15&!2,..*b},p)),GroundQuery::LiveBuilding=>world::first_building(&q.buildings,p,true,false).is_some(),GroundQuery::Ruin=>world::first_building(&q.buildings,p,true,true).is_some(),
   //448700's temporary74/W5 decrement cancels;44a660 does not read it.
   GroundQuery::Rejected{..}=>fire::fire_at_point(&q.objects,p)||super::original_attack_click::terrain_admission(self.ambient_terrain_cell((p[1]&!15).wrapping_mul(8).wrapping_add(p[0]>>4)),q.class,q.family,0,q.grenades)==0,
   GroundQuery::Destructible=>q.objects.iter().any(|o|o.position[0]>=0&&o.flags18&16!=0&&(0..2).all(|a|{let extent=((i16::from(o.flags18 as i8)+1)*8)&24;o.position[a]<=p[a]&&i32::from(p[a])<i32::from(o.position[a])+i32::from(extent)})),}
 }
 ///The same448750 call used by56840 and448e80's cycle completion. This
 ///may consume original RNG; it deliberately emits no projectile or scatter.
 pub(crate) fn native_ground_target_available(&mut self,index:usize,point:[i16;2])->bool{let Some(query)=self.native_ground_queries(index)else{return false;};let mut rng=self.original_rng.clone();rng.multiplayer=self.multiplayer;let result=super::original_point_fire::resolve_ground_point(point,query.flags69,|kind,p|self.native_ground_query(&query,kind,p),||rng.next_u32()).is_some();self.original_rng=rng;result}
 ///404ec0 is called by41f2d0 and4208f0 before the UNIT rectangle tests.
 pub(crate) fn native_ambient_area_contact(&mut self,point:[i16;2]){
  let Some(mut state)=self.native_ambient.take()else{return;};state.pool.damage(point,false,|fragment|self.spawn_native_ambient_fragment(fragment.position,fragment.sprite));self.native_ambient=Some(state);
 }
 ///404f80's first qualifying100-slot target consumes the travelling bullet.
 pub(crate) fn native_ambient_bullet_contact(&mut self,point:[i16;2])->bool{
  let Some(mut state)=self.native_ambient.take()else{return false;};let hit=state.pool.damage(point,true,|fragment|self.spawn_native_ambient_fragment(fragment.position,fragment.sprite));self.native_ambient=Some(state);hit
 }
 pub(crate) fn native_ambient_terrain_retired(&mut self,point:[i16;2]){if let Some(state)=&mut self.native_ambient{state.pool.clear_cell(point);}}
 pub(crate) fn native_ambient_blocker(&self,point:[i16;2])->Option<(usize,[i16;2],u8)>{let state=self.native_ambient.as_ref()?;let slot=ambient::blocker_at(&state.blockers,point)?;let blocker=&state.blockers[slot];Some((slot,blocker.position(),blocker.bytes[5]))}
 pub(crate) fn retire_native_ambient_blocker(&mut self,slot:usize){if let Some(b)=self.native_ambient.as_mut().and_then(|s|s.blockers.get_mut(slot)){b.bytes[..2].copy_from_slice(&(-1i16).to_le_bytes());}}
 pub(crate) fn native_ambient_visuals(&self)->Vec<serde_json::Value>{
  self.native_ambient.as_ref().map(|s|s.draws.iter().map(|&(slot,d)|{
   let y=d.position[1].wrapping_add(i16::from(d.shade as i8));
   let angle=d.rotation.map_or(0,|r|255-i32::from(r as i8));
   json!({"id":slot,"sprite":d.sprite,"x":i32::from(d.position[0])*2,"y":i32::from(y)*2,"queueY":i32::from(d.position[1])*2,"team":d.palette,"palette":d.palette,"flags52":d.flags52,"flags53":d.flags53,"nativeYOffset":d.shade as i8,"scale":if d.rotation.is_some(){0.5}else{1.},"rotation":f64::from(angle)*std::f64::consts::TAU/256.})
  }).collect()).unwrap_or_default()
 }
}
