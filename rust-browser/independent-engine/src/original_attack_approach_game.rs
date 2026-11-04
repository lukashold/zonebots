//!4490b0 bound to actual finite goal allocation,43bbe0 and52810 owners.
use crate::{Game,original_attack_order::State,original_attack_approach::{self as approach,World,Outcome},original_path_contact::game::Boundary};
pub(crate) struct Env<'a>{pub game:&'a mut Game,pub index:usize}
impl Game{
 pub(crate) fn native_attack_approach(&mut self,index:usize,point:[i16;2],s:&mut State)->Result<Outcome,Boundary>{let mut env=Env{game:self,index};approach::approach(s,point,&mut env)}
}
impl Env<'_>{
 fn commit(&mut self,s:&State)->Result<(),Boundary>{let(_,c)=self.game.attack_order_state(self.index).ok_or(Boundary::MissingMember)?;self.game.store_attack_order_state(self.index,s,&c);Ok(())}
 fn reload(&self,s:&mut State)->Result<(),Boundary>{let family=s.family;*s=self.game.attack_order_state(self.index).ok_or(Boundary::MissingMember)?.0;s.family=family;Ok(())} //4490b0 never writes raw30
}
#[cfg(test)]#[path="original_attack_approach_game_tests.rs"]mod tests;
impl World for Env<'_>{type Error=Boundary;
 fn refresh_counter(&mut self,s:&mut State)->Result<(),Boundary>{let live=self.game.native_resolved_attack_target(self.index).map(|i|{let u=&self.game.units[i];([(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],u.native_class7c)});s.counter68=crate::original_pursuit::refresh_counter(s.target==3456,live.map(|t|t.1),s.flags31&4!=0,s.aim2,s.position,live.map_or([0;2],|t|t.0));self.commit(s)}
 fn clear_paths(&mut self,s:&mut State)->Result<(),Boundary>{self.commit(s)?;self.game.clear_native_attack_paths(self.index);self.reload(s)}
 fn allocate_goal(&mut self,point:[i16;2])->Option<u32>{let slot=self.game.units[self.index].native_slot?;let routes=self.game.native_routes.get_mut();routes.managed[slot]=true;routes.pool.allocate_goal(point)}
 fn build_goal(&mut self,s:&mut State,point:[i16;2],link:u32)->Result<(),Boundary>{let u=self.game.units[self.index].clone();let p=point.map(|v|f64::from(v.wrapping_sub(8))*2.);let outcome=self.game.plan_native_primary_assigned_goal(&u,p[0],p[1],link);if let Some(slot)=u.native_slot{let head=self.game.native_routes.borrow().pool.links[slot].primary;if let Some(m)=self.game.native_group_rosters.members[slot].as_mut(){m.primary_path=head!=0;}}let outcome=outcome.map_err(Boundary::RoadAssembly)?.ok_or(Boundary::MissingNode(link))?;
  if outcome.1{self.game.units[self.index].native_flags9c|=128;if let Some(m)=self.game.units[self.index].native_slot.and_then(|slot|self.game.native_group_rosters.members[slot].as_mut()){m.flags9c|=128;}}
  match outcome.0{crate::original_route_game::RouteResult::Path(path)=>self.game.units[self.index].path=path,_=>return Err(Boundary::MissingNode(link))}
  self.game.units[self.index].dest=Some((p[0],p[1]));self.reload(s)
 }
 fn start_head(&mut self,s:&mut State)->Result<(),Boundary>{self.commit(s)?;let u=&self.game.units[self.index];let before=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let slot=u.native_slot.ok_or(Boundary::MissingActor)?;let head={let routes=self.game.native_routes.borrow();routes.pool.node(routes.pool.links[slot].primary).map(|n|n.point.map(|v|v.wrapping_sub(8)))};
  if head==Some(before){return Err(Boundary::AttackApproachLine(slot));} //unproved queued zero525a0 caller
  self.game.resume_native_reset_path(self.index);if let Some(line)=&self.game.units[self.index].original_motion{self.game.units[self.index].native_axis81=line.axis81();}self.reload(s)
 }
 fn direct_short_line(&mut self,s:&mut State,point:[i16;2])->Result<(),Boundary>{self.commit(s)?;self.game.start_native_walking_direct(self.index,point);let slot=self.game.units[self.index].native_slot.ok_or(Boundary::MissingActor)?;let u=&mut self.game.units[self.index];
  if let Some(line)=&mut u.original_motion{line.shorten_for_attack();u.native_remaining48=line.remaining();}
  else{return Err(Boundary::AttackApproachLine(slot));}
  u.dest=Some((f64::from(point[0])*2.,f64::from(point[1])*2.));self.reload(s)
 }
 fn hardware_start(&mut self,_:&mut State){let slot=self.game.units[self.index].native_hardware_slot;let mut order=slot.map_or(self.game.native_hardware_dummy,|slot|self.game.native_hardware_pool[slot].order);order.field17=(order.field17&!2)|1;if let Some(slot)=slot{self.game.native_hardware_pool[slot].set_order(order);}else{self.game.native_hardware_dummy=order;}}
}
