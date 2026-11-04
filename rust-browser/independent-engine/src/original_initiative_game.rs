//! World query bridge for44d3a0. Call sites belong to the native idle/moving
//! states; this query has no independent clock and does not consume RNG.
use super::super::{Game,stats,original_combat,native_weapon_data,workshops};
use super::native;
use super::Initiative;

impl Game{
 fn native_scan_terrain_code(&self,cell:i16)->u8{
  let i=cell as u16 as usize;let(x,y)=(i&127,i>>7);
  if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(127)&127}else{self.map.native_terrain_plane.get(i).copied().unwrap_or(127)&127}
 }
 ///4f1e0 dispatches4ea30 on code35, using the infantry center or either
 ///overlapping2x2 vehicle footprint. Sector lookup uses the actor anchor.
 pub(crate) fn native_flag_contact(&mut self,index:usize)->Option<usize>{
  if !self.map.original||self.map.native_road_graph.is_none()||self.native_world.is_none(){return None;}
  let u=self.units.get(index)?.clone();if u.team<0||!original_combat::alive(true,u.hp)||workshops::hidden(&u){return None;}
  let p=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];
  let center=((p[1].wrapping_add(8)>>4)*128).wrapping_add(p[0].wrapping_add(8)>>4);
  let touched=if u.native_class7c==1{
   let origin=(p[1]>>4).wrapping_mul(128).wrapping_add(p[0]>>4);
   [origin,center].into_iter().any(|base|[0,1,128,129].into_iter().any(|d|self.native_scan_terrain_code(base.wrapping_add(d))==35))
  }else{self.native_scan_terrain_code(center)==35};if !touched{return None;}
  if let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members.get_mut(s)).and_then(Option::as_mut){m.flags69|=4;}
  self.native_group_flag_cleanup(index);
  let sector=self.native_strategy.as_ref()?.sector_at(f64::from(p[0])*2.,f64::from(p[1])*2.)?;
  if !self.native_flag_capture.ready(sector)||self.sectors.iter().find(|s|s.id==sector)?.owner==u.team{return None;}
  self.native_flag_capture.captured(sector);Some(sector)
 }
 pub(crate) fn finish_native_flag_contact(&mut self,index:usize){
  let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return;};
  let object=self.native_group_rosters.members.get(slot).and_then(Option::as_ref).filter(|m|m.contact_order==5).and_then(|m|m.reserved_object).and_then(|o|u8::try_from(o).ok());
  let Some(object)=object else{return;};self.release_original_claim(index);self.set_native_flag_delay(object,0x55);self.native_group_rosters.record_contact_target(slot,0);
  if let Some(m)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut){m.reserved_object=Some(0);}
 }
 pub(crate) fn advance_native_flag_counters(&mut self){if self.map.original{self.native_flag_capture.advance();}}
 ///44d920 calls only on actual animation search actions. `arrived` selects
 ///the strict native range comparison and does not create a second clock.
 pub(crate) fn native_idle_object_scan(&mut self,index:usize,relative:u8,arrived:bool)->bool{
  if self.map.native_road_graph.is_none(){return false;}
  let Some(u)=self.units.get(index)else{return false;};
  if !matches!(u.native_class7c,1|2)&&self.native_object_contact_scan(index,relative,false,true){return true;}
  arrived&&self.native_idle_ambient_scan(index)
 }
 ///52200 scans objects after infantry movement only on622dc0, while5d=0.
 ///The phase gate also admits the callback that consumed its last waypoint.
 pub(crate) fn native_marching_object_scan(&mut self,index:usize)->bool{
  if self.map.native_road_graph.is_none(){return false;}
  if self.motion_tick&1==0{return false;}
  let Some(u)=self.units.get(index)else{return false;};
  if stats(&u.kind).cls!=0||u.native_phase35!=2{return false;}
  let contact=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref).map_or(0,|m|m.contact_order);
  if contact!=0{return false;}
  self.native_object_contact_scan(index,0,true,true)
 }
 fn native_object_contact_scan(&mut self,index:usize,relative:u8,inclusive:bool,guard50a90:bool)->bool{
  if !self.map.original||self.native_world.is_none(){return false;}
  let u=self.units[index].clone();if u.team<0||!original_combat::alive(true,u.hp)||workshops::hidden(&u){return false;}
  let Some(slot)=u.native_slot else{return false;};
  let Some((object,distance))=self.native_directed_object_query(index,relative,u.native_flags64)else{
   if inclusive&&self.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.flags31&8!=0&&m.flags9c&64!=0){self.release_original_claim(index);}
   return false;
  };
  let range=(((self.native_unit_stats(&u).range/2.)as i32>>3)*2)as i8;
  if if inclusive{i32::from(distance)*2>i32::from(range)}else{i32::from(distance)*2>=i32::from(range)}{return false;}
  if guard50a90&&self.native_group_rosters.role(slot).is_some_and(|role|role<0){
   match self.native_group_apc_target(index,object){Some(255)=>return false,Some(found)if found==object=>{},_=>if u.native_initiative_scan.excluded9d==object{return false;}}
  }
  let Some(world)=self.native_world_scan_objects().into_iter().find(|o|o.slot==object)else{return false;};
  let (goal,destination,waypoint)=if let Some(sector)=world.flag_sector{
   let Some(order)=self.claim_native_flag_order(index,object,world.flag_delay1f)else{return false;};
   (Initiative::Flag(sector),order.destination,None)
  }else if let Some(target)=world.unit{
   let Some(order)=self.claim_native_hardware_order(index,target)else{return false;};
   (Initiative::Vehicle(target),order.anchor_destination(),Some(order.destination))
  }else{return false;};
  let point=(f64::from(destination[0])*2.,f64::from(destination[1])*2.);
  if self.owns_native_route(index){
   //The same57860/578b0 route writers apply to idle and walking searches:
   //flags take a direct525a0 detour; hardware prepends59c60 without replacing
   //the retained primary list. Both use the existing source-tested52810 API.
   if let Some(waypoint)=waypoint{if !self.prepend_native_walk_claim(index,waypoint){return false;}}else{self.start_native_walking_direct(index,destination);}
  }else{
   //Graph-less/legacy arenas retain their existing flattened route adapter.
   let route=self.route_unit(&u,point.0,point.1);let p=&mut self.units[index];p.path=route;p.original_motion=None;p.native_walk=None;p.native_phase35=2;
  }
  let p=&mut self.units[index];p.march_resume=if u.order=="move"{u.dest}else{None};p.initiative=Some(goal);p.dest=Some(point);p.native_idle=None;p.hold_position=false;
  match goal{Initiative::Flag(_)=>{p.order="move".into();p.target=None;},Initiative::Vehicle(id)=>{p.order="enter".into();p.target=Some(id);}}
  true
 }
 pub(crate) fn native_directed_object_query(&mut self,index:usize,relative:u8,flags64:u8)->Option<(u8,u8)>{
  if !self.map.original||self.native_world.is_none(){return None;}
  let u=self.units.get(index)?.clone();let slot=u.native_slot?;
  let member=self.native_group_rosters.members.get_mut(slot)?.as_mut()?;
  let mut state=u.native_initiative_scan;
  let mut flags9c=(u.native_flags9c&!64)|(member.flags9c&64);
  let direction=native::direction(original_combat::facing(u.angle),relative,member.flags8a,flags64,state.flags71,&mut flags9c,&mut state.a4);
  member.flags9c=(member.flags9c&!64)|(flags9c&64);self.units[index].native_flags9c=flags9c;
  self.units[index].native_initiative_scan=state;
  let direction=direction?;let objects=self.native_world_scan_objects().into_iter().map(|o|native::Object{slot:o.slot,position:o.position,bucket12:o.bucket12,flags19:o.flags19}).collect::<Vec<_>>();
  let position=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];let candidates=native::object_candidates(&objects,native::coarse_cell(position),direction);
  let actor_cell=((position[1].wrapping_add(8)>>4)*128).wrapping_add(position[0].wrapping_add(8)>>4);
  let selected=native::object_scan(&objects,&candidates,position,u.team as u8,direction,|slot|{
   let target=objects.iter().find(|o|o.slot==slot).unwrap().position;
   native::visible(position,u.native_class7c,actor_cell,target,|cell|{
    let code=self.native_scan_terrain_code(cell);
    native_weapon_data::TERRAIN_HEIGHT.get(usize::from(code)).copied().unwrap_or(0)as i8
   })
  })?;Some((selected.slot,selected.distance))
 }
}

#[cfg(test)]mod tests{
 use super::*;
 use super::super::super::{json,original_group_game,original_route};
 fn field()->Game{
  let mut g=Game::new(0,1,1789);let(w,h)=(64,64);g.map.original=true;g.map.width=w;g.map.height=h;g.map.world=0;
  g.map.tiles=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.terrain_codes=vec![10;w*h];g.map.native_terrain_plane=vec![10;16384];g.map.territory=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();
  g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
  g.units.clear();g.buildings.clear();g.sectors.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.events.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters=original_group_game::State::default();g.next_id=1;g.winner=None;
  g.spawn("grunt",0,320.,320.,1,None);g.units[0].angle=0.;g
 }
 fn car(g:&mut Game,x:i32,y:i32,slot:usize)->usize{
  g.spawn("jeep",-1,f64::from(x),f64::from(y),10+slot,None);let id=g.units.last().unwrap().id;
  g.objects.push(json!({"type":33,"equipmentId":13,"nativeSlot":slot,"x":x,"y":y,"nativeHp":10}));id
 }
 fn imported(g:&mut Game){g.import_native_world(&serde_json::Value::Null);g.maintain_native_group_rosters();}
 #[test]fn object_initiative_follows_the_current_cone_and_uses_the_real_reservation(){
  let mut g=field();let id=car(&mut g,400,320,0);imported(&mut g);g.units[0].angle=std::f64::consts::PI;
  let rng=g.original_rng.words;assert!(!g.native_idle_object_scan(0,0,false));assert_eq!(g.native_world_scan_objects()[0].flags13&1,0);
  g.units[0].angle=0.;assert!(g.native_idle_object_scan(0,0,false));assert_eq!(g.units[0].initiative,Some(Initiative::Vehicle(id)));assert_eq!(g.native_world_scan_objects()[0].flags13&1,1);
  let m=g.native_group_rosters.members[g.units[0].native_slot.unwrap()].as_ref().unwrap();assert_eq!(m.reserved_object,Some(0));assert_eq!(m.contact_order,3);assert_eq!(g.original_rng.words,rng);
 }
 #[test]fn managed_idle_hardware_claim_prepends_the_original_waypoint_without_rebuilding_the_primary_list(){
  let mut g=field();let id=car(&mut g,400,320,0);imported(&mut g);let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.observe_unit_context(slot);let head=g.native_routes.get_mut().pool.allocate_goal([308,208]).unwrap();g.native_routes.get_mut().managed[slot]=true;g.native_routes.get_mut().pool.links[slot].primary=head;g.units[0].path=std::collections::VecDeque::from([(600.,400.)]);g.units[0].dest=Some((600.,400.));g.units[0].order="move".into();assert!(g.native_idle_object_scan(0,0,false));let new=g.native_routes.borrow().pool.links[slot].primary;assert_ne!(new,head);assert_eq!(g.native_routes.borrow().pool.node(new).unwrap().next,head);assert_eq!(g.native_routes.borrow().pool.node(head).unwrap().point,[308,208]);assert_eq!(g.units[0].march_resume,Some((600.,400.)));assert_eq!(g.units[0].initiative,Some(Initiative::Vehicle(id)));assert_eq!(g.units[0].path.back(),Some(&(600.,400.)));assert!(g.units[0].original_motion.is_some());
 }
 #[test]fn a_reserved_nearest_object_is_not_replaced_by_a_farther_candidate(){
  let mut g=field();car(&mut g,384,320,0);car(&mut g,448,320,1);imported(&mut g);assert!(g.reserve_native_object(0));
  assert_eq!(g.native_directed_object_query(0,0,0).map(|c|c.0),Some(0));assert!(!g.native_idle_object_scan(0,0,false));
  assert_eq!(g.native_world_scan_objects().iter().find(|o|o.slot==1).unwrap().flags13&1,0);assert!(g.units[0].initiative.is_none());
 }
 #[test]fn idle_has_strict_range_while_the_march_gate_is_inclusive_and_pulsed(){
  let mut g=field();car(&mut g,560,320,0);imported(&mut g);assert!(!g.native_idle_object_scan(0,0,false));g.units[0].native_phase35=2;
  g.motion_tick=2;assert!(!g.native_marching_object_scan(0));g.motion_tick=3;assert!(g.native_marching_object_scan(0));
 }
 #[test]fn scan_flags_block_without_toggling_or_consuming_a_rotating_direction(){
  let mut g=field();car(&mut g,400,320,0);imported(&mut g);g.units[0].native_flags64=32;g.units[0].native_initiative_scan.flags71=8;g.units[0].native_initiative_scan.a4=4;
  let slot=g.units[0].native_slot.unwrap();let flags=g.native_group_rosters.members[slot].as_ref().unwrap().flags9c;
  assert!(!g.native_idle_object_scan(0,0,false));assert_eq!(g.units[0].native_initiative_scan.a4,4);assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().flags9c,flags);
 }
 #[test]fn original_flag_contact_uses_the_terrain_and_the_43_callback_lockout(){
  let source=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/original/campaign.json");let Ok(source)=std::fs::read_to_string(source)else{return;};
  let campaign:serde_json::Value=serde_json::from_str(&source).unwrap();let mut g=Game::new(0,1,991);g.apply_original(&campaign["missions"][0]).unwrap();g.ai_teams.clear();
  let flag=g.native_world_scan_objects().into_iter().find(|o|o.flag_sector.is_some()).unwrap();let sector=flag.flag_sector.unwrap();g.sectors.iter_mut().find(|s|s.id==sector).unwrap().owner=1;
  g.spawn("grunt",0,f64::from(flag.position[0])*2.,f64::from(flag.position[1])*2.,90,None);let index=g.units.len()-1;
  assert_eq!(g.native_flag_contact(index),Some(sector));assert_eq!(g.native_flag_capture.counter(sector),510);assert_eq!(g.native_flag_contact(index),None);
  for _ in 0..42{g.advance_native_flag_counters();}assert_eq!(g.native_flag_capture.counter(sector),468);assert_eq!(g.native_flag_contact(index),None);
  g.advance_native_flag_counters();assert_eq!(g.native_flag_contact(index),Some(sector));
 }
}
