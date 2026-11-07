use super::{State,Write,Call,World};
use crate::{Game,original_combat,native_math,original_path_contact::game::Boundary};
struct Env<'a>{game:&'a mut Game,index:usize}
impl Game{
 pub(crate) fn dispatch_native_follow(&mut self,index:usize)->bool{
  let Some(u)=self.units.get(index)else{return false;};
  if !self.native_route_mode||!self.owns_native_route(index)||u.native_phase35!=7{return false;}
  let mode=self.native_callback_mode!=0;
  if let Err(e)=super::tick(&mut Env{game:self,index},mode){self.native_route_boundary=Some(e);}true
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{
  let mut g=Game::new(0,1,457590);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.bridge_base=vec![-1;4096];g.map.native_road_graph=Some(crate::original_route::RoadGraph{regions:vec![crate::original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.sectors.clear();g.walls.clear();g.native_world=None;g.native_group_rosters=Default::default();g.ai_teams.clear();g.native_route_mode=true;g.winner=None;
  g.spawn("grunt",0,640.,640.,1,None);g.spawn("grunt",0,704.,640.,1,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,1);g.native_group_rosters.set_group_angle(0,1,0);g.import_native_world(&crate::Value::Null);
  let slot=g.units[1].native_slot.unwrap();g.native_routes.get_mut().managed[slot]=true;let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[63]=0;g.native_group_rosters.pool[group].roster[9]=1;g.native_group_rosters.pool[group].roster[59]=255;
  g.start_native_walking_direct(1,[350,320]);let u=&mut g.units[1];u.native_phase35=7;u.native_frame32=1;u.native_timer3c=0;u.native_speed8b=0;u.native_field62=1;u.native_flags64&=!16;g.native_callback_mode=0;g
 }
 #[test]fn follower_callback_moves_once_before_rejoining_and_restores_animation(){
  let mut g=field();assert!(g.dispatch_native_follow(1));assert_eq!(g.native_route_boundary,None);assert_eq!((g.units[1].x,g.units[1].y),(702.,640.));assert_eq!((g.units[1].native_frame32,g.units[1].native_timer3c),(2,10));assert_eq!(g.units[1].native_phase35,2);let slot=g.units[1].native_slot.unwrap();assert_ne!(g.native_routes.borrow().pool.links[slot].primary,0);assert!(g.units[1].original_motion.is_some());
 }
 #[test]fn movement_suppression_does_not_suppress_the_phase7_animation_counter(){
  let mut g=field();g.units[1].native_remaining48=0;g.units[1].original_motion=None;let before=(g.units[1].x,g.units[1].y);assert!(g.dispatch_native_follow(1));assert_eq!(g.native_route_boundary,None);assert_eq!((g.units[1].x,g.units[1].y),before);assert_eq!((g.units[1].native_frame32,g.units[1].native_timer3c),(2,10));
 }
}
impl World for Env<'_>{type Error=Boundary;
 fn state(&self)->Result<State,Boundary>{let u=&self.game.units[self.index];let m=u.native_slot.and_then(|s|self.game.native_group_rosters.members[s].as_ref()).ok_or(Boundary::MissingMember)?;let g=self.game.native_group_rosters.current_roster.and_then(|s|self.game.native_group_rosters.pool.get(s)).ok_or(Boundary::MissingMember)?;
  let leader_order=g.member(0).and_then(|s|self.game.units.iter().rfind(|v|v.native_slot==Some(usize::from(s)))).map_or(0,|v|v.native_order5c);
  Ok(State{remaining:u.native_remaining48,frame:u.native_frame32,timer:u.native_timer3c,speed:u.native_speed8b,heading:original_combat::facing(u.angle),flags31:m.flags31,flags64:u.native_flags64,range:u.native_range4.unwrap_or((self.game.native_unit_stats(u).range/8.)as i8),aim:u.original_combat.as_ref().map_or_else(||original_combat::CombatState::new(&u.kind).aim_skill,|s|s.aim_skill)as i8,group_target:g.roster[59],group_timer:g.roster[61],leader_order,group_flags:g.roster[9],group_heading:g.roster[17]})
 }
 fn write(&mut self,w:Write){let u=&mut self.game.units[self.index];match w{Write::Frame(v)=>u.native_frame32=v,Write::Timer(v)=>u.native_timer3c=v,Write::Heading(v)=>u.angle=native_math::angle(v),Write::GroupFlags(v)=>{if let Some(g)=self.game.native_group_rosters.current_roster{self.game.native_group_rosters.pool[g].roster[9]=v;}}}}
 fn call(&mut self,c:Call)->Result<u16,Boundary>{let i=self.index;let result=match c{
  Call::Move=>{self.game.native_preserved_inline_motion(i)?;0},Call::Random=>self.game.native_random()as u16,
  Call::GroupAttack=>{let target=self.state()?.group_target;let context=self.game.native_group_rosters.current_roster;self.game.react_native_idle_group(i,Some(target),context);0},
  Call::WeaponScan=>{let c=self.game.native_walk_weapon_scan(i)?;u16::from(c.slot)*256+u16::from(c.distance)},
  Call::TargetInvalid(slot)=>{let (_,m)=self.game.native_reaction_source(slot).map_err(Boundary::UpstreamReaction)?;u16::from(crate::original_idle::reset::target_invalid(self.state()?.flags31,m.flags31))},
  Call::Acquire(slot)=>{self.game.native_walk_acquire(i,slot);0},
  Call::ObjectScan=>{let flags=self.game.units[i].native_flags64;let(slot,distance)=self.game.native_directed_object_query(i,0,flags).unwrap_or((255,0));u16::from(slot)*256+u16::from(distance)},
  Call::Claim(slot)=>u16::from(!self.game.native_walk_claim(i,slot)),
  Call::Follow=>{self.game.follow_native_idle_group(i);0},Call::Sync=>{self.game.reset_native_idle(i);0}
 };if let Some(e)=self.game.native_route_boundary{return Err(e);}Ok(result)}
}
