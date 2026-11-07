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
#[cfg(test)]#[path="original_follow_tests.rs"]mod tests;

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
