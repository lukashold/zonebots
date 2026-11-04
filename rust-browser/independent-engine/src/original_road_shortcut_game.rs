//!43bfb0's physical dummy uses the actor's real18/1c tokens. It does not
//!replace them with a kind/team heuristic. The DDA and table transaction
//!kernels retain their separate native execution goldens.
use crate::{Game,original_collision,original_dynamic_collision as dynamic,original_path_contact::game::Boundary,original_road_scratch::State};
fn boundary(e:dynamic::Error)->Boundary{match e{dynamic::Error::MissingMaskByte(a)=>Boundary::MissingCollisionMask(a),dynamic::Error::MissingTerrain(c)=>Boundary::MissingTerrain(c)}}
impl Game{
 pub(crate) fn native_road_shortcut(&mut self,index:usize,start:(i16,i16),goal:(i16,i16),state:&mut State)->Result<bool,Boundary>{
  let u=self.units.get(index).ok_or(Boundary::MissingActor)?.clone();let vehicle=u.native_class7c==1;let cell=((i32::from(start.1)*8+64)&0x7ff80).wrapping_add((i32::from(start.0)+8)>>4)as i16;let code=self.dynamic_collision_terrain(cell).map_err(boundary)?;let tag=dynamic::initial_terrain_tag(&self.native_collision_masks,u.native_footprint18,code).map_err(boundary)?;
  //525a0's zero-delta early return is AL1 in43bfb0.
  if start==goal{return Ok(false);}
  let group=self.native_group_rosters.current_roster.and_then(|g|self.native_group_rosters.pool.get(g));let command=group.map_or(0,|g|g.roster[58]as i8);let group_goal=group.map(|g|[g.word(44),g.word(46)]);
  let mut actor=dynamic::WrapperActor{position:[start.0,start.1],permission:u.native_permission1c,footprint:u.native_footprint18,terrain_tag:tag,heading:0,pausebc:state.dummy_pause_bc,counter67:state.dummy_counter67};let mut masks=std::mem::take(&mut self.native_collision_masks);
  let result=original_collision::trace_queried(start,goal,vehicle,true,|old,attempted,heading|{if vehicle&&matches!(heading,0|1|7)&&group.is_none(){return Err(Boundary::MissingGroup(-1));}actor.position=[old.0,old.1];actor.heading=heading;dynamic::wrapped(&mut actor,&mut masks,[attempted.0,attempted.1],vehicle,command,||group_goal.is_some_and(|p|self.native_world_buildings().is_some_and(|bs|crate::original_world_contact::first_building(&bs,p,false,false).is_some_and(|(i,_)|bs[i].kind==0))),|cell|self.dynamic_collision_terrain(cell)).map_err(boundary)});
  self.native_collision_masks=masks;state.dummy_pause_bc=actor.pausebc;state.dummy_counter67=actor.counter67;result.map(|c|c.axes==0&&c.position==goal)
 }
}
