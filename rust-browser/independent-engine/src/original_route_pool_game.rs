//! Shared native path ownership. The Vec route is a display/compatibility
//! projection; managed UNITs retain the original finite linked records.
use super::{Pool,State,WorkWorld,Completion,Function,Actor,Dispatch};
use crate::{Game,Unit,original_route as route,original_route_game::RouteResult,original_terrain};
use std::collections::VecDeque;
use crate::original_path_contact::game::Boundary;
fn raw(p:(f64,f64))->[i16;2]{[((p.0/2.).floor()as i16).wrapping_add(8),((p.1/2.).floor()as i16).wrapping_add(8)]}
pub(crate) fn anchors(pool:&Pool,link:u32)->VecDeque<(f64,f64)>{pool.points(link).into_iter().map(|p|(f64::from(p[0].wrapping_sub(8))*2.,f64::from(p[1].wrapping_sub(8))*2.)).collect()}
impl Game{
 fn native_owned_contact_goal(&mut self,index:usize)->Result<[i16;2],crate::original_path_contact::game::Boundary>{let pool=std::mem::replace(&mut self.native_routes.get_mut().pool,Pool{nodes:vec![],links:[Default::default();240]});let goal=self.native_contact_goal(index,&pool);self.native_routes.get_mut().pool=pool;goal.map(|g|g.position)}
 ///518e0 for a terrain collision. Dynamic UNIT overlays have a separate
 ///wrapper; an unowned compatibility line remains with its existing caller.
 pub(crate) fn native_route_contact(&mut self,index:usize,contact:crate::original_collision::Contact)->Result<bool,crate::original_path_contact::game::Boundary>{
  use crate::original_path_contact::Context;
  if !self.owns_native_route(index){return Ok(false);}let u=self.units[index].clone();let slot=u.native_slot.unwrap();let Some(member)=self.native_group_rosters.members[slot].as_ref().cloned()else{return Ok(false);};let mut proposed=[contact.attempted.0,contact.attempted.1];let old=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let mut context={let paths=self.native_routes.borrow();Context{blocked:[contact.x.unwrap_or(-1),contact.y.unwrap_or(-1)],proposed,force:false,callback_idle:paths.scheduler.function==Function::Idle,current_actor:usize::from(paths.scheduler.current)==slot,dummy_actor:false}};
  self.native_attack_walk_marked=true;self.units[index].native_pose7e|=128;
  if u.native_class7c==1&&u.native_flags64&64!=0{self.finish_native_walk_line(index);return Ok(true);}
  if u.native_class7c!=1{let speed=u.native_speed8b.max(0)as usize;let thresholds=[10i8,8,6,5,5,4,3,2,2,2,1,1,1];let v=&mut self.units[index];v.native_timer3c=v.native_timer3c.wrapping_sub(1);if v.native_timer3c<0{v.native_timer3c=thresholds[speed.min(12)];v.native_frame32=v.native_frame32.wrapping_add(1)&3;}}
  if self.native_autoattack_contact(index,context,|g,i|g.native_owned_contact_goal(i))?{return Ok(true);}
  if u.native_flags64&64!=0{self.finish_native_walk_line(index);return Ok(true);}
  if u.native_class7c==1{if self.native_vehicle_goal(index,&mut context)?{self.enqueue_native_contour(index,context.blocked);if let Some(w)=u.native_hardware_slot.and_then(|s|self.native_hardware_pool.get_mut(s)){let mut o=w.order;o.field17|=2;w.set_order(o);}else{self.native_hardware_dummy.field17|=2;}proposed=old;}else{proposed=context.proposed;}}
  else{
   if member.role>0&&member.flags31&8==0&&member.flags8a&8==0{
    if u.native_counter66!=8{if contact.axes&1!=0{proposed[0]=old[0];}if contact.axes&2!=0{proposed[1]=old[1];}if u.native_remaining48==0{self.clear_native_attack_paths(index);self.finish_native_walk_line(index);return Ok(true);}self.units[index].native_counter66=u.native_counter66.wrapping_add(1);self.apply_native_contact_position(index,proposed);return Ok(true);}
    self.units[index].native_counter66=0;
    if member.flags8a&4==0{let leader=self.native_group_rosters.current_roster.and_then(|g|self.native_group_rosters.pool[g].member(0)).and_then(|slot|self.units.iter().rfind(|u|u.native_slot==Some(usize::from(slot))));if let Some(leader)=leader{let lp=[(leader.x/2.).floor()as i16,(leader.y/2.).floor()as i16];if (i32::from(lp[0])-i32::from(old[0])).abs()<96&&(i32::from(lp[1])-i32::from(old[1])).abs()<96{if self.native_route_segment_clear((old[0],old[1]),(lp[0],lp[1]),&u.kind,u.team){self.clear_native_attack_paths(index);self.finish_native_walk_line(index);return Ok(true);}}else{self.clear_native_attack_paths(index);self.follow_native_idle_group(index);}}}
   }
   if self.native_group_rosters.current_roster.and_then(|g|self.native_group_rosters.pool.get(g)).is_some_and(|g|g.roster[63]==0){self.enqueue_native_contour(index,context.blocked);self.units[index].native_phase35=15;if member.role<0{if let Some(g)=self.native_group_rosters.current_roster.and_then(|g|self.native_group_rosters.pool.get_mut(g)){g.roster[9]&=!1;}}}self.finish_native_walk_line(index);proposed=old;
  }
  self.apply_native_contact_position(index,proposed);Ok(true)
 }
 fn enqueue_native_contour(&mut self,index:usize,blocked:[i16;2]){let slot=self.units[index].native_slot.unwrap();let paths=self.native_routes.get_mut();paths.scheduler.cancel(slot);paths.pool.clear_secondary(slot);paths.contact[slot]=blocked;if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags31=(m.flags31&!32)|64;m.secondary_path=false;}self.units[index].native_secondary_route.set(false);}
 fn apply_native_contact_position(&mut self,index:usize,p:[i16;2]){let u=&mut self.units[index];let next=(f64::from(p[0])*2.,f64::from(p[1])*2.);u.moving=(u.x,u.y)!=next;u.x=next.0;u.y=next.1;if let Some(segment)=u.original_motion.as_mut(){segment.correct_contact_position(p);}}
 pub(crate) fn native_path_active_slots(&self)->[bool;240]{std::array::from_fn(|s|self.native_group_rosters.members[s].as_ref().is_some_and(|m|m.flags31&4!=0))}
 pub(crate) fn owns_native_route(&self,index:usize)->bool{self.units.get(index).and_then(|u|u.native_slot).is_some_and(|s|self.native_routes.borrow().managed[s])}
 pub(crate) fn has_native_route(&self,index:usize)->bool{self.units.get(index).and_then(|u|u.native_slot).is_some_and(|s|{let paths=self.native_routes.borrow();paths.managed[s]&&(paths.pool.links[s].primary!=0||paths.pool.links[s].secondary!=0)})}
 pub(crate) fn clear_native_owned_paths(&mut self,index:usize){let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return;};let Some(m)=self.native_group_rosters.members[slot].as_ref()else{return;};if !self.native_routes.borrow().managed[slot]{return;}let clear_hardware=self.native_routes.get_mut().pool.clear(slot,m.flags31&4!=0,m.role);if clear_hardware{if let Some(w)=self.units[index].native_hardware_slot.and_then(|s|self.native_hardware_pool.get_mut(s)){let mut o=w.order;o.field17&=!1;w.set_order(o);}else{self.native_hardware_dummy.field17&=!1;}}
  if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags31&=!32;m.primary_path=self.native_routes.get_mut().pool.links[slot].primary!=0;m.secondary_path=false;}
 }
 ///53730/490b0 allocate the global road list first. Local obstacle contours
 ///are requested by518e0 when movement actually contacts the changed world.
 pub(crate) fn plan_native_primary_route(&self,u:&Unit,dx:f64,dy:f64)->Option<RouteResult>{
  if !self.map.original||self.native_world.is_none()||self.map.native_road_graph.is_none(){return None;}let slot=u.native_slot?;let m=self.native_group_rosters.members[slot].as_ref()?;
  if ![u.x,u.y,dx,dy].iter().all(|n|n.is_finite()&&*n>=0.&&*n<=4094.)||u.native_class7c==2{return Some(RouteResult::Blocked);}
  let active=self.native_path_active_slots();let goal_link={let mut paths=self.native_routes.borrow_mut();paths.managed[slot]=true;paths.pool.clear(slot,active[slot],m.role);let Some(goal_link)=paths.pool.allocate_goal(raw((dx,dy)))else{return Some(RouteResult::Blocked);};goal_link};match self.plan_native_primary_route_with_goal(u,dx,dy,goal_link){Ok(result)=>result.map(|p|p.0),Err(_)=>Some(RouteResult::Blocked)}
 }
 ///43bbe0 with the goal record already allocated by its actual caller.
 ///This helper performs no52500, allocation of that goal, or538f0 writes.
 pub(crate) fn plan_native_primary_route_with_goal(&self,u:&Unit,dx:f64,dy:f64,goal_link:u32)->Result<Option<(RouteResult,bool)>,super::road::Boundary>{self.build_native_primary_goal(u,dx,dy,goal_link,false)}
 ///4490b0 assigns44d1a0 BEFORE43bbe0; preserve that prefix on errors.
 pub(crate) fn plan_native_primary_assigned_goal(&self,u:&Unit,dx:f64,dy:f64,goal_link:u32)->Result<Option<(RouteResult,bool)>,super::road::Boundary>{self.build_native_primary_goal(u,dx,dy,goal_link,true)}
 fn build_native_primary_goal(&self,u:&Unit,_dx:f64,_dy:f64,goal_link:u32,assign_before:bool)->Result<Option<(RouteResult,bool)>,super::road::Boundary>{
  if !self.map.original||self.native_world.is_none(){return Ok(None);}let Some(graph)=self.map.native_road_graph.as_ref()else{return Ok(None);};let Some(slot)=u.native_slot else{return Ok(None);};let Some(m)=self.native_group_rosters.members[slot].as_ref()else{return Ok(None);};
  let start=((u.x/2.).floor()as i16,(u.y/2.).floor()as i16);let profile=if u.kind=="apc"&&u.crew.as_ref().is_some_and(|c|c.kind=="tough"){original_terrain::Profile::Tracked}else{original_terrain::profile(&u.kind)};
  let allowed=|code|if u.kind=="tough"&&m.role<1{route::tough_leader_passable(code)}else{original_terrain::passable(code,self.map.world,profile)};
  let terrain=|p|self.route_terrain_code(p);let price=|c|if u.native_class7c!=1&&self.map.world==1&&c==9{12}else{original_terrain::route_cost(c,profile)};
  let active=self.native_path_active_slots();let mut paths=self.native_routes.borrow_mut();paths.managed[slot]=true;if assign_before{paths.pool.assign_primary(slot,goal_link,&active);}
  let State{pool,road_scratch,..}=&mut *paths;
  let input=crate::original_road_scratch::Input{actor:start,actor_token:0x5e30c8+slot as u32*192,vehicle:u.native_class7c==1,suppressed:false,dimensions:((self.map.width*16)as i16,(self.map.height*16)as i16),level:self.map.native_level_record};
  let outcome=road_scratch.build_primary(&mut pool.nodes,goal_link,input,graph,terrain,|c|!allowed(c),price);
  let result=match outcome{Ok(result)=>result,Err(boundary)=>return Err(boundary)};
  if !assign_before{paths.pool.assign_primary(slot,goal_link,&active);}let view=anchors(&paths.pool,paths.pool.links[slot].primary);u.native_route_pending.set(None);u.native_secondary_route.set(false);Ok(Some((RouteResult::Path(view),result.set_road_flag)))
 }
 pub(crate) fn native_owned_route_view(&self,index:usize)->Option<VecDeque<(f64,f64)>>{let u=self.units.get(index)?;let slot=u.native_slot?;let paths=self.native_routes.borrow();if !paths.managed[slot]{return None;}let m=self.native_group_rosters.members[slot].as_ref()?;Some(anchors(&paths.pool,if m.flags31&32!=0{paths.pool.links[slot].secondary}else{paths.pool.links[slot].primary}))}
}
//Successful43e190 path commits preserve the actor's inline40..53.
//Its pricing rays may call525a0 on633950; that shared dummy is not this actor.
struct Env<'a>{game:&'a mut Game,paths:&'a mut State,index:usize,grace:u8,phase_override:Option<i16>}
impl Env<'_>{
 fn slot(&self)->usize{self.game.units[self.index].native_slot.unwrap()}
 fn position(&self)->(i16,i16){let u=&self.game.units[self.index];((u.x/2.).floor()as i16,(u.y/2.).floor()as i16)}
 fn goal(&mut self)->Option<(i16,i16)>{match self.game.native_contact_goal(self.index,&self.paths.pool){Ok(g)if !g.failed=>Some((g.position[0],g.position[1])),Ok(_)=>None,Err(e)=>{self.game.native_route_boundary=Some(e);None}}}
 fn open(&self,cell:i32)->bool{if !(0..16384).contains(&cell){return false;}let u=&self.game.units[self.index];let code=self.game.route_terrain_code(((cell&127)as i16,(cell>>7)as i16));let role=self.game.native_group_rosters.members[self.slot()].as_ref().map_or(0,|m|m.role);if u.kind=="tough"&&role<1{route::tough_leader_passable(code)}else{let profile=if u.kind=="apc"&&u.crew.as_ref().is_some_and(|c|c.kind=="tough"){original_terrain::Profile::Tracked}else{original_terrain::profile(&u.kind)};original_terrain::passable(code,self.game.map.world,profile)}}
 fn set_flags(&mut self,clear:u8,set:u8){let slot=self.slot();if let Some(m)=self.game.native_group_rosters.members[slot].as_mut(){m.flags31=(m.flags31&!clear)|set;m.primary_path=self.paths.pool.links[slot].primary!=0;m.secondary_path=self.paths.pool.links[slot].secondary!=0;}}
 fn abort_path(&mut self){let slot=self.slot();let role=self.game.native_group_rosters.members[slot].as_ref().map_or(0,|m|m.role);self.set_flags(64,0);if role>0{if let Some(m)=self.game.native_group_rosters.members[slot].as_mut(){m.flags8a&=!8;}}else{let p=self.position();if role<0{self.game.units[self.index].native_order5c=1;if let Some(m)=self.game.native_group_rosters.members[slot].as_mut(){m.promotion.order=1;m.promotion.target_x=i32::from(p.0.wrapping_add(8));m.promotion.target_y=p.1.wrapping_add(8);}if let Some(g)=self.game.native_group_rosters.current_roster.and_then(|s|self.game.native_group_rosters.pool.get_mut(s)){g.roster[58]=1;g.roster[44..46].copy_from_slice(&p.0.wrapping_add(8).to_le_bytes());g.roster[46..48].copy_from_slice(&p.1.wrapping_add(8).to_le_bytes());}}self.game.units[self.index].native_flags64&=!8;}
  let active=self.game.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.flags31&4!=0);self.paths.pool.clear(slot,active,role);self.game.units[self.index].path.clear();self.game.units[self.index].native_route_pending.set(None);self.game.units[self.index].native_secondary_route.set(false);self.game.reset_native_idle(self.index);if self.game.units[self.index].native_class7c==1{self.game.units[self.index].native_flags64&=!64;if let Some(w)=self.game.units[self.index].native_hardware_slot.and_then(|s|self.game.native_hardware_pool.get_mut(s)){let mut o=w.order;o.field17&=!3;w.set_order(o);}else{self.game.native_hardware_dummy.field17&=!3;}}
 }
 fn finish_without_candidate(&mut self)->Completion{
  let slot=self.slot();let u=self.game.units[self.index].clone();let vehicle=u.native_class7c==1;
  if self.grace==127{if vehicle{return Completion::RetryVehicle;}self.grace=1;self.phase_override=Some(0);self.abort_path();return Completion::Done;}
  self.set_flags(64,0);let hardware17=u.native_hardware_slot.and_then(|s|self.game.native_hardware_pool.get(s)).map_or(self.game.native_hardware_dummy.field17,|w|w.order.field17);if u.native_phase35!=15&&(!vehicle||hardware17&2==0){return Completion::Done;}
  let old=self.position();let context=crate::original_path_contact::Context{blocked:self.paths.contact[slot],proposed:[old.0,old.1],force:true,callback_idle:false,current_actor:true,dummy_actor:false};let pool=&self.paths.pool;
  match self.game.native_autoattack_contact(self.index,context,|g,i|g.native_contact_goal(i,pool).map(|goal|goal.position)){Ok(true)=>return Completion::Done,Err(e)=>{self.game.native_route_boundary=Some(e);return Completion::Done;},Ok(false)=>{}}
  let goal=match self.game.native_contact_goal(self.index,&self.paths.pool){Ok(g)=>g,Err(e)=>{self.game.native_route_boundary=Some(e);return Completion::Done;}};
  let footprint_blocked=if goal.failed{true}else if goal.node!=self.paths.pool.links[slot].primary{false}else{let(x,y)=(goal.position[0],goal.position[1]);let add=if vehicle{23}else{7};let xs=[x.wrapping_sub(8)>>4,x.wrapping_add(add)>>4];let ys=[y.wrapping_sub(8)>>4,y.wrapping_add(add)>>4];xs.into_iter().any(|x|ys.into_iter().any(|y|!self.open(i32::from(x)+i32::from(y)*128)))};
  if footprint_blocked{self.abort_path();return Completion::Done;}let contact=self.paths.contact[slot];let point=if contact[1]== -1{if old.1&15==0{self.abort_path();return Completion::Done;}[old.0.wrapping_add(8),(old.1&!15).wrapping_add(24)]}else{if contact[0]!= -1||old.0&15==0{self.abort_path();return Completion::Done;}[(old.0&!15).wrapping_add(8),old.1.wrapping_add(8)]};
  if let Some(index)=self.paths.pool.first_free(super::PRIMARY_COUNT,super::SECONDARY_COUNT){self.paths.pool.nodes[index]=super::Node{point,next:0};self.paths.pool.links[slot].secondary=Pool::address(index);self.set_flags(0,32);let u=&mut self.game.units[self.index];u.native_secondary_route.set(true);u.path=anchors(&self.paths.pool,self.paths.pool.links[slot].secondary);if self.game.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.role<0){if let Some(g)=self.game.native_group_rosters.current_roster.and_then(|s|self.game.native_group_rosters.pool.get_mut(s)){g.roster[8]=255;}}}Completion::Done
 }
}
impl WorkWorld for Env<'_>{
 fn initialize_origin(&mut self){let u=&self.game.units[self.index];self.paths.steps=0;if u.native_class7c!=1{self.paths.origin=u.native_cell10 as i16;return;}let flip=!u.native_route_flip.get();u.native_route_flip.set(flip);let(x,y)=self.position();self.paths.origin=if flip{((i32::from(y)*8-57)&0xff80).wrapping_add((i32::from(x)-7)>>4)as i16}else{(((i32::from(y)+8)*8)&0xff80).wrapping_add((i32::from(x)+8)>>4)as i16};}
 fn clear_mask(&mut self,start:usize,length:usize){self.paths.mask[start..start+length].fill(65);}
 fn reset_candidates(&mut self){
  self.paths.candidates=std::array::from_fn(|_|None);
  self.paths.headers.fill((255,-1));
  for (i,&header) in self.paths.headers.iter().enumerate(){self.paths.road_scratch.record_candidate(i,header,&[]);}
 }
 fn target_ray(&mut self)->bool{let Some(goal)=self.goal()else{return true;};let u=&self.game.units[self.index];let vehicle=u.native_class7c==1;let marked=route::local::target_mask(self.position(),goal,vehicle,u.native_route_flip.get(),self.grace,|cell|{let code=self.game.route_terrain_code(((cell&127)as i16,(cell>>7)as i16));let admit66=u.native_slot.and_then(|s|self.game.native_group_rosters.members[s].as_ref()).is_some_and(|m|m.flagsb7&1!=0);code==30||(code!=66||admit66)&&self.open(cell)});for(a,b)in self.paths.mask.iter_mut().zip(marked){if b==255{*a=255;}}self.paths.origin<128}
 fn seed(&mut self,d:usize,vehicle:bool)->bool{let slot=self.slot();let contact=self.paths.contact[slot];let origin=i32::from(self.paths.origin);let mut mask=std::mem::take(&mut self.paths.mask);let candidate=route::local::seed(origin,(contact[0]>0).then_some(contact[0]),(contact[1]>0).then_some(contact[1]),d,vehicle,self.paths.steps,&mut mask,|cell|self.open(cell));self.paths.mask=mask;let failed=candidate.is_none();if let Some(c)=&candidate{self.paths.steps=c.steps;self.paths.headers[d]=(c.score,c.count());}else{self.paths.headers[d]=(0,0);}self.paths.road_scratch.record_candidate(d,self.paths.headers[d],candidate.as_ref().map_or(&[],|c|c.points.as_slice()));self.paths.candidates[d]=candidate;failed}
 fn walk_pending(&mut self,d:usize,vehicle:bool)->bool{let Some(mut c)=self.paths.candidates[d].take()else{return false;};let mut mask=std::mem::take(&mut self.paths.mask);let progress=if vehicle{c.vehicle(&mut mask,|cell|self.open(cell))}else{c.foot(&mut mask,|cell|self.open(cell))};self.paths.steps=c.steps;self.paths.headers[d]=(c.score,c.count());self.paths.road_scratch.record_candidate(d,self.paths.headers[d],&c.points);self.paths.mask=mask;self.paths.candidates[d]=Some(c);progress==route::local::Progress::Pending}
 fn finish(&mut self)->Completion{let slot=self.slot();if self.game.native_group_rosters.members[slot].as_ref().is_none_or(|m|m.flags31&4==0){self.set_flags(64,0);return Completion::Done;}let vehicle=self.game.units[self.index].native_class7c==1;let Some(selected)=route::choose_local(self.paths.headers)else{return self.finish_without_candidate();};let mut packed=self.paths.candidates[selected].as_ref().unwrap().points.clone();if let Some(goal)=self.goal(){let cell=i32::from(goal.0>>4)+i32::from(goal.1>>4)*128;if[-129,-128,-127,-1,1,127,128,129].into_iter().all(|d|self.open(cell+d)){let cell=cell as i16;if packed.last()!=Some(&cell){if packed.len()>1&&packed[packed.len()-2]==cell{packed.pop();}else{packed.push(cell);}}}}
  let start=self.position();let unit=self.game.units[self.index].clone();let mut removals=Vec::new();let mut shortcut_error=None;self.paths.road_scratch.record_candidate(selected,(self.paths.headers[selected].0,-(packed.len()as i8)),&packed);route::local::simplify_with_removals(&mut packed,|from,to|{let from=from.map(|p|route::segment_anchor(route::local_waypoint(p as u16,vehicle))).unwrap_or(start);let to=route::segment_anchor(route::local_waypoint(to as u16,vehicle));if shortcut_error.is_some(){return false;}let clear=match self.game.native_road_shortcut(self.index,to,from,&mut self.paths.road_scratch){Ok(clear)=>clear,Err(e)=>{shortcut_error.get_or_insert(e);false}};clear},|index|removals.push(index));for index in &removals{self.paths.road_scratch.remove_candidate_point(selected,*index);}let removed=!removals.is_empty();self.paths.headers[selected].1=if shortcut_error.is_some(){packed.len()as i8}else{-(packed.len()as i8)};if removed&&selected<3{self.paths.headers[selected+1]=(255,-1);}self.paths.road_scratch.synchronize_candidate(selected,self.paths.headers[selected].0,&packed,removed);if let Some(e)=shortcut_error{let offset=369+selected*100;if offset<640{self.paths.road_scratch.frame[offset]=packed.len()as u8;}self.game.native_route_boundary=Some(e);return Completion::Done;}let cursor=0x4d15f0+selected as u32*100+(packed.len()as u32+1)*2;self.paths.road_scratch.synchronize_context(self.paths.origin,cursor,vehicle);
  let profile=if unit.kind=="apc"&&unit.crew.as_ref().is_some_and(|c|c.kind=="tough"){original_terrain::Profile::Tracked}else{original_terrain::profile(&unit.kind)};let role=self.game.native_group_rosters.members[slot].as_ref().map_or(0,|m|m.role);let allowed=|code|if unit.native_class7c==3&&role<1{route::tough_leader_passable(code)}else{original_terrain::passable(code,self.game.map.world,profile)};
  let replacement=if let Some(graph)=self.game.map.native_road_graph.as_ref(){self.paths.road_scratch.consider(crate::original_road_scratch::Input{actor:start,actor_token:0x5e30c8+slot as u32*192,vehicle,suppressed:unit.native_flags9c&2!=0,dimensions:((self.game.map.width*16)as i16,(self.game.map.height*16)as i16),level:self.game.map.native_level_record},&packed,graph,|p|self.game.route_terrain_code(p),|c|self.game.native_collision_masks.value(unit.native_footprint18,c).map_or_else(|_|!allowed(c),|mask|mask!=0),|c|if !vehicle&&self.game.map.world==1&&c==9{12}else{original_terrain::route_cost(c,profile)})}else{self.game.native_route_boundary=Some(Boundary::MissingNode(crate::original_road_scratch::BASE));return Completion::Done;};
  self.paths.origin=self.paths.road_scratch.dword(356) as i16;
  for (i,header) in self.paths.headers.iter_mut().enumerate(){
   if let Some(raw)=self.paths.road_scratch.changed_candidate_header(i){*header=raw;}
  }
  let replacement=match replacement{
   Ok(crate::original_road_scratch::Decision::Contour)=>{
    if let Err(boundary)=self.paths.road_scratch.reread_candidate(selected,&mut packed){
     self.game.native_route_boundary=Some(Boundary::RoadScratch(boundary));
     return Completion::Done;
    }
    None
   },
   Ok(crate::original_road_scratch::Decision::Road(points))=>Some(points),
   Err(boundary)=>{self.game.native_route_boundary=Some(Boundary::RoadScratch(boundary));return Completion::Done;}
  };
  if let Some(points)=replacement{let active=self.game.native_path_active_slots();let clear_hardware=self.paths.pool.clear(slot,active[slot],role);self.set_flags(32,0);if clear_hardware{if let Some(w)=unit.native_hardware_slot.and_then(|s|self.game.native_hardware_pool.get_mut(s)){let mut order=w.order;order.field17&=!1;w.set_order(order);}else{self.game.native_hardware_dummy.field17&=!1;self.game.native_hardware_dummy_attributes.bytes[23]=self.game.native_hardware_dummy.field17;}}if !self.paths.pool.copy_replacement(slot,&points){self.paths.road_scratch.partial_adoption_head=Some(self.paths.pool.links[slot].primary);self.paths.pool.links[slot].primary=0;self.game.native_route_boundary=Some(Boundary::RoadScratch(crate::original_road_scratch::Boundary::FiniteAdoption));return Completion::Done;}let head=self.paths.pool.links[slot].primary;self.paths.pool.assign_primary(slot,head,&active);self.set_flags(96,0);let u=&mut self.game.units[self.index];u.native_flags9c|=130;u.native_counter68=100;u.native_secondary_route.set(false);u.native_route_road_replaced.set(true);u.path=anchors(&self.paths.pool,self.paths.pool.links[slot].primary);if role<0{if let Some(g)=self.game.native_group_rosters.current_roster.and_then(|s|self.game.native_group_rosters.pool.get_mut(s)){g.roster[8]=g.roster[60];}}return Completion::Done;}
  let points=packed.into_iter().map(|c|{let p=route::local_waypoint(c as u16,vehicle);[p.0,p.1]}).collect::<Vec<_>>();let success=self.paths.pool.contour(slot,&points);if success{self.set_flags(64,32);let u=&mut self.game.units[self.index];u.native_flags9c&=125;u.native_counter68=100;u.native_secondary_route.set(true);u.native_route_road_replaced.set(false);u.path=anchors(&self.paths.pool,self.paths.pool.links[slot].secondary);if let Some(m)=self.game.native_group_rosters.members[slot].as_ref(){if m.role<0{if let Some(g)=self.game.native_group_rosters.pool.get_mut(usize::from(m.group)){g.roster[8]=255;}}}}else{self.set_flags(64,0);}Completion::Done
 }
 fn abort(&mut self){self.abort_path();}
 fn clear_unit_counter67(&mut self){self.game.units[self.index].native_counter67=0;}
}
impl Game{
 ///435d74 is after all UNIT callbacks and before the OBJECT pool callbacks.
 pub(crate) fn update_native_route_worker(&mut self){if !self.native_route_mode||!self.map.original||self.native_world.is_none()||self.map.native_road_graph.is_none(){return;}let mut paths=std::mem::replace(self.native_routes.get_mut(),State::vacant());let actors=std::array::from_fn(|slot|{let m=self.native_group_rosters.members[slot].as_ref();let u=self.units.iter().rfind(|u|u.native_slot==Some(slot));Actor{flags31:m.map_or(0,|m|m.flags31),group:m.map_or(255,|m|m.group),family:u.map_or(0,|u|u.native_family()),class:u.map_or(0,|u|u.native_class7c),permissions:u.is_some()}});
  match paths.scheduler.dispatch(&actors){Dispatch::Run{slot,group,..}=>{self.native_group_rosters.current_roster=Some(usize::from(group));if let Some(index)=self.units.iter().rposition(|u|u.native_slot==Some(usize::from(slot))){let mut worker=paths.worker;let mut scheduler=paths.scheduler;let grace=scheduler.side_grace;let vehicle=self.units[index].native_class7c==1;let mut env=Env{game:self,paths:&mut paths,index,grace,phase_override:None};worker.advance(&mut scheduler,vehicle,&mut env);if let Some(p)=env.phase_override{scheduler.phase=p;scheduler.side_grace=env.grace;}paths.worker=worker;paths.scheduler=scheduler;}},Dispatch::Invalid{group,..}=>self.native_group_rosters.current_roster=Some(usize::from(group)),_=>{}}
  *self.native_routes.get_mut()=paths;
 }
}
#[cfg(test)]mod tests{
 use super::*;use crate::{original_group_game,original_path_contact::game::Boundary};
 struct UntilAdoption<'a>{inner:Env<'a>,stopped:bool}
 impl WorkWorld for UntilAdoption<'_>{
  fn initialize_origin(&mut self){self.inner.initialize_origin();}fn clear_mask(&mut self,s:usize,n:usize){self.inner.clear_mask(s,n);}fn reset_candidates(&mut self){self.inner.reset_candidates();}fn target_ray(&mut self)->bool{self.inner.target_ray()}fn seed(&mut self,d:usize,v:bool)->bool{self.inner.seed(d,v)}fn walk_pending(&mut self,d:usize,v:bool)->bool{self.inner.walk_pending(d,v)}fn finish(&mut self)->Completion{self.stopped=true;Completion::Done}fn abort(&mut self){panic!("native fixture must reach the43e190 adoption boundary");}fn clear_unit_counter67(&mut self){}
 }
 fn world(kind:&str,x:i16,y:i16)->Game{let mut g=Game::new(0,1,44439);let(w,h)=(128,128);g.map.original=true;g.map.width=w;g.map.height=h;g.map.world=0;g.map.tiles=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.native_terrain_plane=vec![0;16384];g.map.territory=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_road_graph=Some(route::RoadGraph{regions:vec![route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});g.units.clear();g.buildings.clear();g.objects.clear();g.sectors.clear();g.crates.clear();g.walls.clear();g.native_world=None;g.native_group_rosters=original_group_game::State::default();g.ai_teams.clear();g.winner=None;g.next_id=1;for y in 20..=40{for x in 30..=34{g.map.terrain_codes[y*128+x]=43;g.map.native_terrain_plane[y*128+x]=43;}}g.import_native_world(&crate::json!({}));g.spawn(kind,0,f64::from(x)*2.,f64::from(y)*2.,0,None);g.maintain_native_group_rosters();g}

 fn check_commit_line(branch:u8){let mut g=world(if branch==2{"crane"}else{"heavy"},if branch==2{161}else{160},if branch==2{657}else{656});g.map.native_level_record=5;for y in 0..128{for x in 0..128{let mut code=if(40..=42).contains(&x){40}else{0};if(40..=41).contains(&y)&&(8..=100).contains(&x){code=7;}g.map.terrain_codes[y*128+x]=code;g.map.native_terrain_plane[y*128+x]=code;}}let mut regions=vec![route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(7).enumerate(){r.bounds=[(8+i as i16*12)*16,512,(20+i as i16*12)*16-1,800];r.waypoint=(14+i*12+40*128)as u16;for j in 0..7{r.next[j]=if i==j{j as u8}else if j>i{i as u8+1}else{i as u8-1};}}g.map.native_road_graph=Some(route::RoadGraph{regions});let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.observe_unit_context(slot);let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[8]=1;g.native_group_rosters.pool[group].roster[60]=3;g.native_group_rosters.members[slot].as_mut().unwrap().flags31=68;g.units[0].native_phase35=if branch==2{15}else{55};g.units[0].native_flags64=0;g.units[0].native_flags9c=if branch==0{2}else{0};let h=g.units[0].native_hardware_slot.unwrap();let mut w=g.native_hardware_pool[h].order;w.field17=1;g.native_hardware_pool[h].set_order(w);let mut line=crate::original_motion::Segment::new(g.units[0].x,g.units[0].y,(2200.,1300.));line.set_source_fixture([89,89,44,2]);g.units[0].native_remaining48=89;g.units[0].original_motion=Some(line);let fields=g.units[0].original_motion.as_ref().unwrap().source_line_fields();let mut paths=State::default();paths.managed[slot]=true;paths.pool.links[slot].primary=paths.pool.allocate_goal([1120,656]).unwrap();let primary=paths.pool.links[slot].primary;paths.contact[slot]=[10,-1];paths.origin=5258;paths.headers.fill((255,-1));if branch!=2{let mut c=route::local::Contour::new(0,0,0,0,route::local::Edge::North);c.points=vec![10+20*128,70+20*128,70+41*128];c.complete=true;c.score=0;paths.headers[0]=(0,-3);paths.candidates[0]=Some(c);}let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:0,phase_override:None};assert_eq!(env.finish(),Completion::Done);assert_eq!(g.native_route_boundary,None,"branch{branch}");let flags=g.native_group_rosters.members[slot].as_ref().unwrap().flags31;if branch==1{assert_eq!(paths.pool.links[slot].primary,primary,"real43deb0 may reuse the just-freed first record");assert_eq!(paths.pool.points(paths.pool.links[slot].primary),vec![[431,655],[623,655],[815,655],[1007,655],[1135,664]],"native road commit replaces the old one-node route");assert_eq!(paths.pool.links[slot].secondary,0);assert_eq!(flags&96,0);assert!(g.units[0].native_route_road_replaced.get());assert_eq!(g.units[0].native_flags9c&130,130);assert_eq!(g.native_hardware_pool[h].order.field17&1,0,"actual52500 still owns the hardware clear");}else{assert_eq!(paths.pool.links[slot].primary,primary);assert_ne!(paths.pool.links[slot].secondary,0);assert_eq!(flags&96,32);assert_eq!(g.native_hardware_pool[h].order.field17,1);if branch==2{assert_eq!(paths.pool.points(paths.pool.links[slot].secondary),vec![[169,680]]);}else{assert!(!g.units[0].native_route_road_replaced.get());assert_eq!(g.units[0].native_counter68,100);}}assert!(g.units[0].original_motion.is_some(),"source43e190 branch{branch} preserves the existing line owner");assert_eq!(g.units[0].original_motion.as_ref().unwrap().source_line_fields(),fields);assert_eq!(g.units[0].native_remaining48,89);}
 #[test]fn contour_commit_keeps_existing_inline_fields_and_commits_real_secondary(){check_commit_line(0);}
 #[test]fn road_commit_keeps_existing_inline_fields_and_clears_old_finite_heads(){check_commit_line(1);}
 #[test]fn fallback_commit_keeps_existing_inline_fields_and_allocates_real_single_node(){check_commit_line(2);}
 #[test]fn ordinary_foot_and_vehicle_complete_real_queued_contours_in_the_main_loop(){
  for kind in["grunt","medium"]{let mut g=world(kind,464,512);let id=g.units[0].id;g.spawn("grunt",1,3000.,3000.,20,None);g.maintain_native_group_rosters();g.native_route_mode=true;for u in &g.units{if let Some(s)=u.native_slot{g.native_routes.get_mut().managed[s]=true;}}g.command(&crate::json!({"ids":[id],"kind":"move","x":1280.,"y":1024.}));let slot=g.units[0].native_slot.unwrap();let(mut queued,mut local,mut work)=(false,false,false);
   for tick in 0..3000{g.step(crate::original_motion::TICK_SECONDS);assert_eq!(g.native_route_boundary,None,"{kind},tick{tick}");let flags=g.native_group_rosters.members[slot].as_ref().unwrap().flags31;queued|=flags&64!=0;local|=flags&32!=0;work|=g.native_routes.borrow().scheduler.function!=Function::Idle;if (g.units[0].x-1280.).hypot(g.units[0].y-1024.)<2.{break;}assert!(g.winner.is_none(),"{kind} must retain the pending route while both teams live");}
   assert!((g.units[0].x-1280.).hypot(g.units[0].y-1024.)<2.,"{kind}: position {:?},phase {},path {:?},flags {:02x},job {:?}",(g.units[0].x,g.units[0].y),g.units[0].native_phase35,g.units[0].path,g.native_group_rosters.members[slot].as_ref().unwrap().flags31,g.native_routes.borrow().scheduler);assert!(queued&&local&&work,"{kind} must use real queued/local paths rather than a precomputed compatibility contour");assert_eq!(g.native_routes.borrow().pool.links[slot],super::super::Links::default());
  }
 }
 #[test]fn actual_game_route_world_matches_original_rays_seeds_contours_and_callback_states(){
  let native=std::env::var("NATIVE_ROUTE_WORLD_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());let starts=[(464,512,4126,-1),(560,512,4130,-1),(512,304,-1,2592),(512,656,-1,5152)];let goals=[[648,520],[648,648],[168,520]];let(mut hash,mut rows)=(2166136261u32,0usize);
  for vehicle in 0..2{for previous in 0..2{for(si,&(x,y,cx,cy))in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){let mut g=world(if vehicle==0{"grunt"}else{"medium"},x,y);let slot=g.units[0].native_slot.unwrap();let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.flags31=68;m.role=0;m.flags8a=0;g.units[0].native_flags64=0;g.units[0].native_route_flip.set(previous!=0);g.units[0].native_cell10=i32::from((x+8)>>4)+i32::from((y+8)>>4)*128;let mut paths=State::default();paths.managed[slot]=true;paths.pool.links[slot].primary=paths.pool.allocate_goal(goal).unwrap();paths.mask.fill(0);paths.contact[slot]=[cx,cy];let mut scheduler=super::super::Scheduler{function:Function::Prepare,side_grace:127,..Default::default()};let mut worker=super::super::Worker::default();
   for tick in 0..200{let before=scheduler.function;let clearing=before==Function::Prepare&&scheduler.phase<=3;let mut env=UntilAdoption{inner:Env{game:&mut g,paths:&mut paths,index:0,grace:scheduler.side_grace,phase_override:None},stopped:false};worker.advance(&mut scheduler,vehicle!=0,&mut env);let stopped=env.stopped;let actual_function=match if stopped{before}else{scheduler.function}{Function::Idle=>0x40c810,Function::Prepare=>0x43e4a0,Function::Vehicle=>0x43dad0};let paths=env.inner.paths;let mh=paths.mask.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619));let values=[vehicle,previous,si as u32,gi as u32,tick,actual_function,scheduler.phase as u32,worker.countdown as u32,if clearing{worker.mask_cursor as u32}else{0},paths.origin as u32,paths.steps,u32::from(env.inner.game.units[0].native_route_flip.get()),mh,u32::from(stopped)];let candidates=std::array::from_fn::<_,4,_>(|d|{let h=paths.headers[d];let mut v=vec![i32::from(h.0),i32::from(h.1)];if h.1!= -1{if let Some(c)=&paths.candidates[d]{v.extend(c.points.iter().copied().map(i32::from));}}v});
    if let Some(records)=&native{let r=&records[rows];let fields=["vehicle","previous","start","goal","tick","function","phase","countdown","cursor","origin","steps","b4","maskHash","stopped"];let expected=fields.map(|f|if f=="stopped"{u32::from(r[f].as_bool().unwrap())}else if f=="cursor"&&!clearing{0}else{r[f].as_i64().unwrap()as u32});assert_eq!(values,expected,"v{vehicle}/p{previous}/s{si}/g{gi}/tick{tick}");for d in 0..4{let expected=r["candidates"][d].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()as i32).collect::<Vec<_>>();assert_eq!(candidates[d],expected,"v{vehicle}/p{previous}/s{si}/g{gi}/tick{tick}/candidate{d}");}}
    for v in values{hash=(hash^v).wrapping_mul(16777619);}for candidate in candidates{for v in candidate{hash=(hash^v as u32).wrapping_mul(16777619);}}rows+=1;assert_eq!(env.inner.game.native_route_boundary,None::<Boundary>);if stopped{break;}assert_ne!(tick,199,"all48 complete native searches must reach adoption");
   }
  }}}}if let Some(records)=&native{assert_eq!(rows,records.len());}assert_eq!((rows,hash),(459,3380812522),"actual Game world, complete native ray/seed/walk timing, adoption as explicit boundary");
 }
 fn road_fixture(kind:&str,start:(i16,i16),goal:(i16,i16),mode:usize)->(Game,State){let mut g=world(kind,start.0,start.1);g.map.terrain_codes.fill(0);g.map.native_terrain_plane.fill(0);for y in 0..128{for x in 0..128{let code=if(40..=41).contains(&y)&&(8..=100).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0};g.map.terrain_codes[y*128+x]=code;g.map.native_terrain_plane[y*128+x]=code;}}let mut regions=vec![route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(7).enumerate(){r.bounds=[(8+i as i16*12)*16,512,(20+i as i16*12)*16-1,800];r.waypoint=(14+i*12+40*128)as u16;for j in 0..7{r.next[j]=if i==j{j as u8}else if j>i{i as u8+1}else{i as u8-1};}}g.map.native_road_graph=Some(route::RoadGraph{regions});let slot=g.units[0].native_slot.unwrap();let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.flags31=68;m.role=0;m.flags8a=0;g.units[0].native_flags64=0;g.units[0].native_flags9c=0;g.native_group_rosters.current_roster=Some(usize::from(g.native_group_rosters.members[slot].as_ref().unwrap().group));let mut paths=State::default();paths.managed[slot]=true;paths.headers.fill((255,-1));paths.headers[0]=(0,-3);let mut c=route::local::Contour::new(0,0,0,0,route::local::Edge::North);c.points=vec![(start.0>>4)+20*128,(goal.0>>4)+20*128,(goal.0>>4)+(goal.1>>4)*128];c.complete=true;paths.candidates[0]=Some(c);(g,paths)}
 fn run_road_caller_dataset(controlled:bool,oracle_name:&str){let oracle=std::env::var(oracle_name).ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<crate::Value>(s).unwrap()).collect::<Vec<_>>());let(mut row,mut adopted,mut hash)=(0usize,0,2166136261u32);for warm in 0..2{for mode in 0..2{for start in[(160,656),(160,500)]{for goal in[(1120,656),(1120,500)]{for kind in["grunt","medium"]{for suppressed in[false,true]{let(mut g,mut paths)=road_fixture(kind,start,goal,mode);let slot=g.units[0].native_slot.unwrap();if controlled{let u=&g.units[0];let mut tokens=vec![u.native_footprint18,u.native_permission1c];for heading in[0,4]{let mut token=u.native_permission1c;crate::original_dynamic_collision::select_permission(&mut token,heading);tokens.push(token);}for token in tokens{for code in 0..67{g.native_collision_masks.write(token+code,if code==40{255}else{0}).unwrap();}}}g.units[0].native_flags9c=if suppressed{2}else{0};if warm!=0{for(i,b)in paths.road_scratch.frame.iter_mut().enumerate(){*b=(0x80+i*13)as u8;}}let before_rng=g.original_rng.words;let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};assert_eq!(env.finish(),Completion::Done);assert_eq!(g.native_route_boundary,None,"row{row}");assert_eq!(g.original_rng.words,before_rng);let m=g.native_group_rosters.members[slot].as_ref().unwrap();let road=paths.pool.links[slot].primary!=0;adopted+=usize::from(road);let head=if road{paths.pool.links[slot].primary}else{paths.pool.links[slot].secondary};let points=paths.pool.points(head);for n in [paths.pool.links[slot].primary,paths.pool.links[slot].secondary,u32::from(m.flags31),u32::from(g.units[0].native_flags9c),g.units[0].native_counter68 as u32,paths.road_scratch.old_price.unwrap_or(-1)as u32,paths.road_scratch.new_price.unwrap_or(-1)as u32,points.len()as u32].into_iter().chain(points.iter().flat_map(|p|[p[0]as u32,p[1]as u32])){hash=(hash^n).wrapping_mul(16777619);}if let Some(rs)=&oracle{let r=&rs[row];assert_eq!(serde_json::json!(points),r["points"],"points row{row}");assert_eq!(paths.pool.links[slot].primary as u64,r["primary"].as_u64().unwrap());assert_eq!(paths.pool.links[slot].secondary as u64,r["secondary"].as_u64().unwrap());assert_eq!(u64::from(m.flags31),r["flags31"].as_u64().unwrap());assert_eq!(u64::from(g.units[0].native_flags9c),r["flags9c"].as_u64().unwrap());{assert_eq!(paths.road_scratch.old_price.unwrap_or(-1)as i64,r["oldCost"].as_i64().unwrap());assert_eq!(paths.road_scratch.new_price.unwrap_or(-1)as i64,r["newCost"].as_i64().unwrap());}}assert_eq!(g.units[0].native_counter68,100);assert_eq!(m.flags31,if road{4}else{36});assert_eq!(g.units[0].native_flags9c,if road{130}else{0});if !suppressed{assert_eq!(paths.road_scratch.dword(352),0x5e30c8+slot as u32*192);assert_eq!(paths.road_scratch.dword(360),0x4d15f0+((paths.headers[0].1.wrapping_neg()as u8 as u32)+1)*2);assert_eq!(paths.road_scratch.dword(364),crate::original_road_scratch::BASE);}assert!(paths.headers[0].1<0);assert_eq!(paths.headers[1],(255,-1));row+=1;}}}}}}assert_eq!(row,64);assert_eq!(hash,if controlled{2807892797}else{4035984813},"complete43e190 caller: native heads/points/flags/prices");assert!(adopted>0,"actual profile must exercise both comparison/adoption branches");}
 #[test]fn finite_copy_exhaustion_keeps_known_52500_prefix_and_reports_a_boundary(){for free in 0..2{let(mut g,mut paths)=road_fixture("medium",(160,500),(1120,500),0);let slot=g.units[0].native_slot.unwrap();let w=g.units[0].native_hardware_slot.unwrap();let mut o=g.native_hardware_pool[w].order;o.field17=0xff;g.native_hardware_pool[w].set_order(o);for n in &mut paths.pool.nodes[..9600]{n.next=0;}if free==1{paths.pool.nodes[0].next=super::super::FREE;}let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};env.finish();assert_eq!(g.native_route_boundary,Some(Boundary::RoadScratch(crate::original_road_scratch::Boundary::FiniteAdoption)));assert_eq!(g.native_hardware_pool[w].order.field17,0xfe);assert_eq!(paths.pool.links[slot].primary,0);assert_eq!(g.units[0].native_flags9c,0);assert_eq!(g.units[0].native_counter68,0);assert!(paths.road_scratch.new_price.unwrap()<paths.road_scratch.old_price.unwrap());assert_eq!(paths.road_scratch.partial_adoption_head,Some(if free==0{0}else{super::super::PRIMARY}));if free==1{assert_eq!(paths.pool.nodes[0].next,super::super::FREE);}}}
 #[test]fn bridge_retirement_and_repair_keep_actual_next_tables_and_warm_scratch(){let(mut g,mut paths)=road_fixture("grunt",(160,500),(1120,500),0);g.map.bridge_hp=vec![1.];g.map.bridge_max_hp=vec![200.];g.import_native_world(&crate::json!({"bridges":[{"id":0,"nativeSlot":0,"nativeOrientation":0,"x":1408,"y":1248,"w":128,"h":64,"hp":1,"maxHp":200}]}));let before=g.map.native_road_graph.as_ref().unwrap().clone();paths.road_scratch.frame.fill(0x31);let warm=paths.road_scratch.frame.clone();g.map.native_terrain_plane[40*128+45]=23;g.map.terrain_codes[40*128+45]=23;g.native_blast_environment(708,608,1,0,false);let after=g.map.native_road_graph.as_ref().unwrap();assert_eq!(after.regions[3].bounds,[0;4]);for(i,r)in after.regions.iter().enumerate(){assert_eq!(r.next,before.regions[i].next);}assert_eq!(paths.road_scratch.frame,warm);g.map.bridge_hp[0]=200.;g.sync_native_world();let restored=g.map.native_road_graph.as_ref().unwrap();assert_eq!(restored.regions[3].bounds,before.regions[3].bounds);for(i,r)in restored.regions.iter().enumerate(){assert_eq!(r.next,before.regions[i].next);}let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};env.finish();assert_eq!(g.native_route_boundary,None);assert!(paths.road_scratch.old_price.is_some());}

 #[test]fn candidate_three_simplification_preserves_warm_price_y_when_roads_are_suppressed(){let native=std::env::var("NATIVE_CANDIDATE_THREE_ALIAS_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<crate::Value>(s).unwrap()).collect::<Vec<_>>());let mut row=0;for warm in 0..2{for kind in["grunt","medium"]{let(mut g,mut paths)=road_fixture(kind,(160,656),(1120,656),0);let slot=g.units[0].native_slot.unwrap();let candidate=paths.candidates[0].take();paths.candidates[3]=candidate;paths.headers.fill((255,-1));paths.headers[3]=(0,-3);g.units[0].native_flags9c=2;if warm!=0{for(i,b)in paths.road_scratch.frame.iter_mut().enumerate(){*b=(0x80+i*13)as u8;}}paths.road_scratch.pairs[0]=[1234,-2000];let old_rng=g.original_rng.words;let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};assert_eq!(env.finish(),Completion::Done);assert_eq!(g.native_route_boundary,None);assert_eq!(paths.road_scratch.pairs[0],[-1,-2000]);assert_eq!(g.original_rng.words,old_rng);assert!(paths.road_scratch.old_price.is_none());assert!(paths.headers[3].1<0);assert_eq!(paths.pool.links[slot].primary,0);if let Some(rs)=&native{let r=&rs[row];assert_eq!(i64::from(paths.road_scratch.pairs[0][0]),r["pricePairX"].as_i64().unwrap());assert_eq!(i64::from(paths.road_scratch.pairs[0][1]),r["pricePairY"].as_i64().unwrap());assert_eq!(serde_json::json!(paths.pool.points(paths.pool.links[slot].secondary)),r["points"]);assert_eq!(u64::from(paths.pool.links[slot].secondary),r["secondary"].as_u64().unwrap());}row+=1;}}assert_eq!(row,4);}
 #[test]fn real_profile_caller_matches_live_scratch_binding_and_real_head_adoption(){run_road_caller_dataset(false,"NATIVE_ROAD_SCRATCH_REAL_PROFILE_ORACLE");}
 #[test]
 fn long_road_caller_keeps_the_native_pool_and_rereads_a_rejected_contour(){
  let trace=std::env::var_os("Z_NATIVE_LONG_ROAD_CALLER").map(|p|std::fs::read_to_string(p).unwrap());
  let mut trace=trace.as_deref().map(str::lines);
  let mut digest=2166136261u32;
  let mut rows=0;
  for hops in [38,39,41,47,60,79,80]{for selected in [0,3]{for warm in [false,true]{for vehicle in [false,true]{
   let start=(136,500);
   let goal=((8+hops as i16)*16+8,500);
   let (mut g,mut paths)=road_fixture(if vehicle{"medium"}else{"grunt"},start,goal,0);
   let graph=g.map.native_road_graph.as_mut().unwrap();
   for (i,region) in graph.regions.iter_mut().enumerate(){
    region.next.fill(255);
    if i<=hops{
     region.bounds=[(8+i as i16)*16,512,(9+i as i16)*16-1,800];
     region.waypoint=(8+i+40*128)as u16;
     for (j,next) in region.next.iter_mut().enumerate().take(hops+1){
      *next=if i==j{j}else if j>i{i+1}else{i-1}as u8;
     }
    }
   }
   if selected==3{
    paths.candidates[3]=paths.candidates[0].take();
    paths.headers[0]=(255,-1);
    paths.headers[3]=(0,-3);
   }
   let slot=g.units[0].native_slot.unwrap();
   g.units[0].native_permission1c=g.units[0].native_footprint18;
   let mut masks=vec![g.units[0].native_footprint18];
   for heading in [0,4]{
    let mut token=g.units[0].native_permission1c;
    crate::original_dynamic_collision::select_permission(&mut token,heading);
    masks.push(token);
   }
   for token in masks{for code in 0..67{g.native_collision_masks.write(token+code,u8::from(code==40)*255).unwrap();}}
   paths.road_scratch.frame.fill(if warm{255}else{0});
   paths.origin=if warm{-1}else{0};
   for i in 0..3{paths.road_scratch.record_candidate(i,(255,0),&[]);}
   paths.road_scratch.record_candidate(selected,(0,-3),&paths.candidates[selected].as_ref().unwrap().points);
   let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};
   assert_eq!(env.finish(),Completion::Done);
   assert_eq!(g.native_route_boundary,None,"case {rows}, hops {hops}, candidate {selected}");
   let links=paths.pool.links[slot];
   let points=paths.pool.points(if links.primary!=0{links.primary}else{links.secondary});
   let scratch=&paths.road_scratch;
   let fields=[links.primary,links.secondary,u32::from(g.native_group_rosters.members[slot].as_ref().unwrap().flags31),
    u32::from(g.units[0].native_flags9c),g.units[0].native_counter68 as u32,
    scratch.old_price.unwrap_or(-1)as u32,scratch.new_price.unwrap_or(-1)as u32,
    scratch.trace.recorded as u8 as u32,scratch.record_cursor];
   if let Some(trace)=&mut trace{
    let row:crate::Value=serde_json::from_str(trace.next().unwrap()).unwrap();
    assert_eq!(crate::json!([hops,selected,u8::from(warm),u8::from(vehicle)]),crate::json!([row["hops"],row["selected"],row["warm"],row["vehicle"]]));
    let keys=["primary","secondary","flags31","flags9c","counter68","oldCost","newCost","recorded","recordCursor"];
    assert_eq!(fields,keys.map(|key|row[key].as_i64().unwrap()as u32),"fields in case {rows}");
    assert_eq!(crate::json!(points),row["points"],"adopted path in case {rows}");
    let raw=row["after"].as_str().unwrap();
    for (i,&byte) in scratch.frame.iter().enumerate(){
     assert_eq!(byte,u8::from_str_radix(&raw[i*2..i*2+2],16).unwrap(),"scratch byte {i} in case {rows}");
    }
   }
   for n in fields.into_iter().chain(scratch.frame.iter().copied().map(u32::from)).chain(points.iter().flatten().map(|&p|p as u32)){
    digest=(digest^n).wrapping_mul(16777619);
   }
   rows+=1;
  }}}}
  assert!(trace.is_none_or(|mut t|t.next().is_none()));
  assert_eq!((rows,digest),(56,2062151207),"whole 43e190 original caller");
 }
 #[test]fn primary_and_contour_callers_share_the_game_raster_across_a_near_goal_exit(){
  let(mut g,mut paths)=road_fixture("medium",(160,656),(1120,656),0);
  paths.road_scratch.frame[..320].fill(0x67);
  *g.native_routes.get_mut()=paths;
  let unit=g.units[0].clone();let slot=unit.native_slot.unwrap();
  assert!(matches!(g.plan_native_primary_route(&unit,2224.,1296.),Some(RouteResult::Path(_))));
  let mut paths=std::mem::replace(g.native_routes.get_mut(),State::vacant());
  assert!(paths.road_scratch.frame[..320].iter().all(|&b|b==0x67),"primary leaves the temporary node arena alone");
  assert_eq!(paths.road_scratch.dword(364),paths.pool.links[slot].primary);
  assert_eq!(paths.road_scratch.raster.destination,[70,41,0]);
  let mut env=Env{game:&mut g,paths:&mut paths,index:0,grace:1,phase_override:None};
  assert_eq!(env.finish(),Completion::Done);
  assert_eq!(g.native_route_boundary,None);
  assert_eq!(paths.road_scratch.dword(364),crate::original_road_scratch::BASE);
  assert!(paths.road_scratch.new_price.is_some());
  let after_contour=paths.road_scratch.raster;
  *g.native_routes.get_mut()=paths;
  assert!(matches!(g.plan_native_primary_route(&unit,336.,1312.),Some(RouteResult::Path(_))));
  let paths=g.native_routes.borrow();
  assert_eq!(paths.road_scratch.raster,after_contour,"distance gate leaves the previous contour raster intact");
  assert_eq!(paths.road_scratch.record_cursor,0x4d1340);
 }
 #[test]fn controlled_permission_caller_matches_identical_table_values_in_game(){run_road_caller_dataset(true,"NATIVE_ROAD_SCRATCH_CONTROLLED_ORACLE");}

 #[test]#[ignore="requires private campaign input; sequential warm RNG/pools"]fn first_three_missions_keep_real_cached_removal_readers_in_the_main_loop(){let path=std::env::var("NATIVE_SOAK_CAMPAIGN").unwrap();let c:crate::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();crate::GAME.with(|s|*s.borrow_mut()=None);for mission in 0..3{let init=crate::request(crate::json!({"op":"init","mission":mission,"difficulty":1,"seed":9281+(mission+1)*151,"aiTeams":[0,1],"originalMission":c["missions"][mission]}));assert!(init["error"].is_null());crate::GAME.with(|state|{let mut state=state.borrow_mut();let g=state.as_mut().unwrap();for tick in 0..3000{if g.winner.is_some(){break;}g.step(0.1);assert_eq!(g.native_route_boundary,None,"M{} tick{} time{}",mission+1,tick,g.time);}});}}


}

#[cfg(test)]mod v47_m13_diagnostic{
 #[test]#[ignore="explicit private original M13 both-AI main diagnostic"]fn m13_both_ais_keep_real_main_lines_for3000_updates(){let path=std::env::var("NATIVE_SOAK_CAMPAIGN").unwrap();let c:crate::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();crate::GAME.with(|s|*s.borrow_mut()=None);let init=crate::request(crate::json!({"op":"init","mission":12,"difficulty":1,"seed":9281+13*151,"aiTeams":[0,1],"originalMission":c["missions"][12]}));assert!(init["error"].is_null());crate::GAME.with(|state|{let mut state=state.borrow_mut();let g=state.as_mut().unwrap();for tick in 0..3000{if g.winner.is_some(){eprintln!("M13 winner {:?} time {} tick{tick}",g.winner,g.time);break;}g.step(0.1);if g.native_route_boundary.is_some(){for u in &g.units{if u.native_slot==Some(27){eprintln!("M13 SLOT27 tick{tick} t{} xy{:?} phase{} order{} remaining{} line{} flags64{} kind{}",g.time,(u.x,u.y),u.native_phase35,u.native_order5c,u.native_remaining48,u.original_motion.is_some(),u.native_flags64,u.kind);}}}assert_eq!(g.native_route_boundary,None,"M13 tick{tick} time{}",g.time);}eprintln!("M13 finaltime {} winner {:?}",g.time,g.winner);});}
}
