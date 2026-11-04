//! Call once in5a080 order, replacing the separate AA/leader calls.
use super::{Pose,PoseEnvironment,collision_tables,clamp_frame,idle_pose,tick_pose};
use crate::{Game,original_combat};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Outcome{Continue,Frozen,Removed,Unavailable,Retired(crate::original_retired_unit_preamble::Completion)}
struct PoseEnv<'a>{game:&'a mut Game,index:usize}
impl PoseEnvironment for PoseEnv<'_>{
 fn random(&mut self)->u32{self.game.native_random()}
 fn prevented(&mut self)->bool{self.game.native_idle_pose_prevented(self.index)}
 fn target_direction(&mut self,relative:u8)->Option<u8>{
  let g=&mut self.game;let old=g.units[self.index].native_initiative_scan.flags71;g.units[self.index].native_initiative_scan.flags71=old|8;let result=g.native_reset_weapon_scan_relative(self.index,relative);g.units[self.index].native_initiative_scan.flags71=old;
  let range=g.units[self.index].native_range4.unwrap_or((g.native_unit_stats(&g.units[self.index]).range/8.)as i8);if result&0xff00==0xff00||result as i8>range{return None;}
  let target=g.units.iter().rfind(|u|u.native_slot==Some(usize::from(result>>8)))?;let actor=&g.units[self.index];Some(crate::native_math::direction(((target.x-actor.x)/2.)as i32,((target.y-actor.y)/2.)as i32))
 }
}

#[cfg(test)]mod tests{
 use super::*;use crate::{json,original_group_game,original_idle};
 fn field(kind:&str)->Game{let mut g=Game::new(0,1,337);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.bridge_base=vec![-1;4096];g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.native_world=None;g.ai_teams.clear();g.native_group_rosters=original_group_game::State::default();g.spawn(kind,0,640.,640.,1,None);g.spawn(kind,0,672.,640.,1,None);g.maintain_native_group_rosters();g.import_native_world(&json!({}));g}
 #[test]fn frozen_actor_still_refreshes_tough_tables_clamps_frame_and_advances_prefix_once(){
  let mut g=field("tough");let slot=g.units[0].native_slot.unwrap();let group=usize::from(g.native_group_rosters.members[slot].as_ref().unwrap().group);g.native_group_rosters.current_roster=Some(100);g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=64;g.native_group_rosters.pool[group].roster[9]=255;g.native_group_rosters.pool[group].roster[63]=2;g.native_group_rosters.pool[group].roster[64]=1;g.native_group_rosters.pool[group].roster[59]=8;g.native_group_rosters.pool[group].roster[16]=0;let u=&mut g.units[0];u.native_phase35=1;u.native_frame32=100;u.native_counter6e=254;u.native_protectionaa=1;u.native_flags9c=255;let rng=g.original_rng.words;
  assert_eq!(g.advance_native_unit_preamble(0),Outcome::Frozen);let u=&g.units[0];assert_eq!((u.native_frame32,u.native_counter6e,u.native_protectionaa,u.native_flags9c),(0,255,0,235));assert_eq!((u.native_footprint18,u.native_permission1c),(0x63a9d0,0x632698));assert_eq!(g.native_group_rosters.current_roster,Some(group));let r=&g.native_group_rosters.pool[group].roster;assert_eq!((r[9],r[63],r[64],r[59]),(223,1,0,255));assert!(g.native_protected_order_seen);assert_eq!(g.original_rng.words,rng);
 }
 #[test]fn role_zero_selects_own_context_and_negative_frames_are_retained(){let mut g=field("grunt");let slot=g.units[1].native_slot.unwrap();let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.role=0;m.flags69|=64;let own=usize::from(m.group);g.native_group_rosters.current_roster=Some(100);g.units[1].native_phase35=1;g.units[1].native_frame32=-1;assert_eq!(g.advance_native_unit_preamble(1),Outcome::Frozen);assert_eq!(g.native_group_rosters.current_roster,Some(own));assert_eq!(g.units[1].native_frame32,-1);}
 #[test]fn deleted_enter_object_cleans_actual_finite_paths_then_returns_actor_to_phase_zero(){let mut g=field("grunt");let first=g.units[0].native_slot.unwrap();let second=g.units[1].native_slot.unwrap();g.units[0].native_phase35=61;g.native_group_rosters.record_contact_target(first,0);g.native_group_rosters.members[first].as_mut().unwrap().flags8a=24;g.native_group_rosters.members[second].as_mut().unwrap().flags8a=24;let node=g.native_routes.get_mut().pool.allocate_goal([408,328]).unwrap();g.native_routes.get_mut().managed[second]=true;g.native_routes.get_mut().pool.links[second].primary=node;
  assert_eq!(g.advance_native_unit_preamble(0),Outcome::Continue);assert_eq!(g.units[0].native_phase35,0);assert_eq!(g.native_group_rosters.members[first].as_ref().unwrap().flags8a&24,0);assert_eq!(g.native_group_rosters.members[second].as_ref().unwrap().flags8a&24,0);assert_eq!(g.native_routes.borrow().pool.links[second].primary,0);assert!(g.units[0].hp>0.);
 }
 #[test]fn deleted_enter_object_with_inside_follower_removes_actor_without_inventing_follower_death(){let mut g=field("grunt");let first=g.units[0].native_slot.unwrap();let second=g.units[1].native_slot.unwrap();g.units[0].native_phase35=61;g.native_group_rosters.record_contact_target(first,0);g.native_group_rosters.members[second].as_mut().unwrap().flags8a=32;g.native_deaths=Default::default();let mut expected=g.original_rng.clone();for _ in 0..3{expected.next_u32();}let Outcome::Retired(c)=g.advance_native_unit_preamble(0)else{panic!("verified5c0 cleanup must continue through44dd00");};assert_eq!((c.unit.phase,c.unit.frame,c.unit.flags31&4),(1,1,0));assert_eq!(g.original_rng.words,expected.words);assert!(g.units[0].native_slot.is_none());assert!(g.native_group_rosters.members[first].is_none());assert!(g.units[0].hp<0.);assert!(g.units[1].hp>0.);assert_eq!(g.native_deaths.records.iter().filter(|r|r.sprite!= -1).count(),1);}
 #[test]fn descriptor_constructor_can_precede_registration_and_is_not_reapplied_by_maintenance(){let mut g=field("grunt");g.spawn("grunt",0,900.,800.,77,None);g.native_group_descriptors48[2]=2;assert!(g.initialize_native_group_descriptor(0,77,2));g.maintain_native_group_rosters();let group=g.native_group_rosters.group_slot(0,77).unwrap();assert_eq!(g.native_group_rosters.pool[group].roster[65],2);g.native_group_rosters.pool[group].roster[65]=111;g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.pool[group].roster[65],111);}
 #[test]fn constructor_uses_zeroed_role_and_actual_saved_context_then_tough_callback_updates_follower(){let mut g=field("tough");let slot=g.units[1].native_slot.unwrap();assert!(g.initialize_native_collision_tables(1));assert_eq!(g.units[1].native_footprint18,0x63a9d0);assert!(g.refresh_native_collision_tables(1,None));assert_eq!(g.units[1].native_footprint18,0x6471a0);assert_eq!(g.units[1].native_permission1c,0x632698);g.units[1].native_class7c=1;g.units[1].kind="apc".into();g.native_selection.contexts[slot].saved_class8e=None;assert!(!g.initialize_native_collision_tables(1));assert_eq!(g.units[1].native_footprint18,0x6471a0);g.native_selection.contexts[slot].saved_class8e=Some(3);assert!(g.initialize_native_collision_tables(1));assert_eq!(g.units[1].native_footprint18,0x6381f0);}
 #[test]fn enabled_pose_consumes_exact_rng_and_its_real_handlers_return_to_idle(){let mut g=field("grunt");let slot=g.units[1].native_slot.unwrap();let group=usize::from(g.native_group_rosters.members[slot].as_ref().unwrap().group);g.units[1].native_phase35=1;g.units[1].native_idle=Some(original_idle::State::enter(0,60,||0));g.native_group_rosters.members[slot].as_mut().unwrap().promotion.delay7d=1;g.native_group_rosters.pool[group].roster[65]=2;g.native_flag_frame=0;let mut rng=g.original_rng.clone();let mut expected=Pose{phase:1,class:0,role:1,group:group as u8,delay:1,heading:original_combat::facing(g.units[1].angle),..Default::default()};idle_pose(&mut expected,false,0,2,0,||rng.next_u32(),||panic!("descriptor2 never queries58590"));assert_eq!(g.advance_native_unit_preamble(1),Outcome::Continue);assert_eq!((g.units[1].native_phase35,g.units[1].native_transition88),(expected.phase,expected.transition as i16));assert_eq!(g.original_rng.words,rng.words);assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.delay7d,0);
  for _ in 0..500{g.native_callback_mode=0;if !g.update_native_feedback_pose(1){g.update_native_idle_pose(1);}if g.units[1].native_phase35==1{break;}}
  assert_eq!(g.units[1].native_phase35,1);assert!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.delay7d>0);
 }
}
impl Game{
 ///1b5d0's descriptor48 write. The caller must be an actual authored or
 ///produced group construction;4b2f0 hardware reform deliberately omits it.
 pub(crate) fn initialize_native_group_descriptor(&mut self,team:i32,squad:usize,gid:usize)->bool{
  let Some(&descriptor)=self.native_group_descriptors48.get(gid)else{return false;};self.native_group_rosters.record_constructed_descriptor(team,squad,descriptor);true
 }
 ///41c090 after the fresh zeroed UNIT/selection context and hardware class
 ///initializer, before1b5d0 registers a leader/follower. The raw role is0.
 pub(crate) fn initialize_native_collision_tables(&mut self,index:usize)->bool{
  let Some(u)=self.units.get(index)else{return false;};let Some(slot)=u.native_slot else{return false;};let saved=self.native_selection.contexts.get(slot).and_then(|c|c.saved_class8e);let Some(tables)=collision_tables(u.native_class7c,original_combat::animation_family(&u.kind),saved,(u.team as u8&3)|4,0)else{return false;};let u=&mut self.units[index];u.native_footprint18=tables.footprint;u.native_permission1c=tables.permission;true
 }
 ///41b950 constructor writer. Saved APC8e is explicit because a missing
 ///transported context must not silently choose an ordinary footprint.
 pub(crate) fn refresh_native_collision_tables(&mut self,index:usize,saved_class:Option<u8>)->bool{
  let Some(u)=self.units.get(index)else{return false;};let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref)else{return false;};let Some(tables)=collision_tables(u.native_class7c,original_combat::animation_family(&u.kind),saved_class,m.flags31,m.role)else{return false;};let u=&mut self.units[index];u.native_footprint18=tables.footprint;u.native_permission1c=tables.permission;true
 }
 pub(crate) fn advance_native_unit_preamble(&mut self,index:usize)->Outcome{
  let Some(u)=self.units.get(index)else{return Outcome::Unavailable;};if !self.map.original||u.team<0||!original_combat::alive(true,u.hp){return Outcome::Unavailable;}
  let Some(slot)=u.native_slot else{return Outcome::Unavailable;};let Some(m)=self.native_group_rosters.members.get(slot).and_then(Option::as_ref)else{return Outcome::Unavailable;};
  //Unconditional632194 selection comes before even the role0 branch.
  self.native_group_rosters.current_roster=(usize::from(m.group)<240).then_some(usize::from(m.group));
  if u.native_phase35==61{let target=self.native_group_rosters.contact_target(slot).unwrap_or(0);let Some(raw)=self.native_object_raw_position(target)else{return Outcome::Unavailable;};if raw[0]== -1{
   self.units[index].native_phase35=0;if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags8a&=!0x18;}
   let retained=self.capture_native_retired_preamble(index);self.cleanup_native_order_transports(index);if self.remove_native_failed_transport_actor(index){return self.finish_native_retired_preamble(index,retained);}
  }}
  if self.units[index].native_class7c==3{self.refresh_native_collision_tables(index,None);}
  let u=&mut self.units[index];if !clamp_frame(u.native_phase35,&mut u.native_frame32){return Outcome::Unavailable;}if let Some(idle)=u.native_idle.as_mut(){idle.frame=u.native_frame32 as u16;}
  self.advance_native_order_preamble(index);self.advance_original_group_leader(index);
  if self.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.flags69&64!=0){return Outcome::Frozen;}
  //586977 is initialized to1 by46227a, with no later writer.
  self.advance_native_idle_pose(index);Outcome::Continue
 }
 fn advance_native_idle_pose(&mut self,index:usize){
  let u=&self.units[index];let Some(slot)=u.native_slot else{return;};let Some(m)=self.native_group_rosters.members[slot].as_ref()else{return;};
  let descriptor=self.native_group_rosters.pool.get(usize::from(m.group)).map_or(0,|g|g.roster[65]);
  let mut pose=Pose{phase:u.native_phase35,class:u.native_class7c,flags8a:m.flags8a,role:m.role,group:m.group,delay:m.promotion.delay7d,cycles:u.native_pose7e as u16,timer:u.native_pose80 as u8,frame:u.native_frame32,counter82:u.native_pose82,heading:original_combat::facing(u.angle),transition:u.native_transition88 as u16};
  let p=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let code=self.native_selection_terrain_code([p[0].wrapping_add(5),p[1].wrapping_add(11)]).unwrap_or(0);
  //Keep the lazy58590 call interleaved with RNG despite both borrowing Game.
  let mut randoms=Vec::new();let mut replay=pose;let mut needed=false;let mut cursor=0;
  idle_pose(&mut replay,self.multiplayer,self.native_flag_frame,descriptor,code,||{let r=self.native_random();randoms.push(r);r},||{needed=true;false});
  let prevented=needed&&self.native_idle_pose_prevented(index);
  idle_pose(&mut pose,self.multiplayer,self.native_flag_frame,descriptor,code,||{let r=randoms[cursor];cursor+=1;r},||prevented);
  let old_phase=self.units[index].native_phase35;let u=&mut self.units[index];u.native_phase35=pose.phase;u.native_frame32=pose.frame;u.native_pose7e=pose.cycles as i16;u.native_pose80=pose.timer as i8;u.native_pose82=pose.counter82;u.native_transition88=pose.transition as i16;u.angle=crate::native_math::angle(pose.heading);
  if let Some(idle)=u.native_idle.as_mut(){idle.search_timer=pose.delay;}
  if pose.phase!=old_phase{u.native_idle=None;u.native_fire=None;u.native_walk=None;u.native_cover=None;}
  if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.promotion.delay7d=pose.delay;m.promotion.phase=pose.phase;}
 }
 ///58590 outside457ae0's tightly scoped4d0e3e override. REF proves that
 ///457ae0 is the only writer and restores0 immediately after its own query.
 fn native_idle_pose_prevented(&mut self,index:usize)->bool{
  let u=&self.units[index];let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members[s].as_ref())else{return true;};let row=self.native_group_rosters.pool.get(usize::from(m.group));let gate=crate::original_reactions::GroupGate{member:m.role,group:m.group,flags09:row.map_or(0,|g|g.roster[9]),timer3d:row.map_or(0,|g|g.roster[61]),command3a:row.map_or(0,|g|g.roster[58]),control10:row.map_or(0,|g|g.roster[16]),count12:row.map_or(0,|g|g.roster[18]),acquisition_override:false,range04:u.native_range4.unwrap_or((self.native_unit_stats(u).range/8.)as i8)};
  if !crate::original_reactions::group_needs_probe(gate){return true;}let probe=self.native_reset_weapon_scan(index);crate::original_reactions::group_prevents_cover(gate,probe)
 }
 ///Actual pose handlers reached by the enabled459eb0 prefix. Place after
 ///the generic42 turn entry, before browser movement/combat. No extra loop.
 pub(crate) fn update_native_idle_pose(&mut self,index:usize)->bool{
  let Some(u)=self.units.get(index)else{return false;};if !self.map.original||u.native_phase35==43&&u.native_cover.is_some(){return false;}let Some(slot)=u.native_slot else{return false;};let Some(m)=self.native_group_rosters.members[slot].as_ref()else{return false;};let mut pose=Pose{phase:u.native_phase35,class:u.native_class7c,flags8a:m.flags8a,role:m.role,group:m.group,delay:m.promotion.delay7d,cycles:u.native_pose7e as u16,timer:u.native_pose80 as u8,frame:u.native_frame32,counter82:u.native_pose82,heading:original_combat::facing(u.angle),transition:u.native_transition88 as u16};
  if !matches!(pose.phase,27|28|29|30|31|34|38|41|43|45){return false;}let p=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let Some(code)=self.native_selection_terrain_code([p[0].wrapping_add(5),p[1].wrapping_add(11)])else{return false;};let mode=self.native_callback_mode;let reset=tick_pose(&mut pose,mode,code,&mut PoseEnv{game:self,index}).unwrap();
  let u=&mut self.units[index];u.native_phase35=pose.phase;u.native_frame32=pose.frame;u.native_pose7e=pose.cycles as i16;u.native_pose80=pose.timer as i8;u.native_pose82=pose.counter82;u.native_transition88=pose.transition as i16;u.angle=crate::native_math::angle(pose.heading);u.moving=false;
  if reset{self.reset_native_idle(index);}true
 }
}
