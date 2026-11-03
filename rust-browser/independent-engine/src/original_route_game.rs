//! Adapter for native road/route kernels. All coordinates passed to kernels
//! are native pixels; the public Game route API continues to accept anchors.
use super::{Game,Unit,stats,original_terrain,original_collision as collision,original_route as route};
use std::collections::VecDeque;
#[derive(Debug)]
pub(crate) enum RouteResult{Unavailable,Path(VecDeque<(f64,f64)>),Partial(VecDeque<(f64,f64)>),Blocked}
impl Game{
 fn route_terrain_code(&self,p:route::Point)->u8{
  let(x,y)=(i32::from(p.0),i32::from(p.1));
  if x<0||y<0||x>=128||y>=128{return 127;}
  if x<self.map.width as i32&&y<self.map.height as i32{return self.map.terrain_codes.get(y as usize*self.map.width+x as usize).copied().unwrap_or(127)&127;}
  self.map.native_terrain_plane.get(y as usize*128+x as usize).copied().unwrap_or(127)&127
 }
 fn route_building_open(&self,cell:i32)->bool{
  if self.map.original&&self.map.native_road_graph.is_some(){return true;}
  let(px,py)=((f64::from(cell&127)+0.5)*32.,(f64::from(cell>>7)+0.5)*32.);
  //The current building adapter exposes rectangle/gate geometry. Native
  //OBJECT/bridge/wall overlays already update the raw terrain plane.
  !self.buildings.iter().any(|b|b.alive(self.map.original)&&px>b.x&&px<b.x+b.w&&py>b.y&&py<b.y+b.h&&!b.gate.map(|(gx,gy)|(px-gx).abs()<24.&&(if b.orientation=="north"{py<=gy+24.}else{py>=gy-24.})).unwrap_or(false))
 }
 fn route_collision_kind(kind:&str)->collision::MaskKind{if stats(kind).cls==0{collision::MaskKind::Foot}else if kind=="crane"{collision::MaskKind::Crane}else{collision::MaskKind::Vehicle}}
 pub(crate) fn native_segment_clear(&self,unit:&Unit,start:(f64,f64),end:(f64,f64))->Option<bool>{
  if !self.map.original||self.map.native_road_graph.is_none(){return None;}
  if ![start.0,start.1,end.0,end.1].iter().all(|v|v.is_finite()&&*v>=0.&&*v<=4094.){return Some(false);}
  Some(self.native_route_segment_clear(((start.0/2.).floor()as i16,(start.1/2.).floor()as i16),((end.0/2.).floor()as i16,(end.1/2.).floor()as i16),&unit.kind,unit.team))
 }
 pub(crate) fn native_route_segment_clear(&self,start:route::Point,end:route::Point,kind:&str,team:i32)->bool{
  let mask=Self::route_collision_kind(kind);let vehicle=mask!=collision::MaskKind::Foot;
  let result=collision::trace(start,end,vehicle,|cell,octant|{
   if !(0..16384).contains(&i32::from(cell))||!self.route_building_open(i32::from(cell)){return -1;}
   let code=self.route_terrain_code((cell&127,cell>>7));
   collision::world_permission(code,mask,team.clamp(0,2)as u8,octant,self.map.world==1)
  });result.axes==0&&result.position==end
 }
 pub(crate) fn route_native_unit(&self,unit:&Unit,dx:f64,dy:f64)->RouteResult{
  let result=self.plan_native_unit(unit,dx,dy);
  if let Some(actual)=self.units.iter().find(|u|u.id==unit.id){actual.native_route_pending.set(if matches!(result,RouteResult::Partial(_)){Some((dx,dy))}else{None});actual.native_secondary_route.set(matches!(result,RouteResult::Partial(_)));}
  result
 }
 fn plan_native_unit(&self,unit:&Unit,dx:f64,dy:f64)->RouteResult{
  let Some(graph)=self.map.native_road_graph.as_ref().filter(|_|self.map.original)else{return RouteResult::Unavailable;};
  if ![unit.x,unit.y,dx,dy].iter().all(|n|n.is_finite())||stats(&unit.kind).speed<=0.{return RouteResult::Blocked;}
  let coordinate=|v:f64|->Option<i16>{let n=(v/2.).floor();(n>=0.&&n<=i16::MAX as f64-8.).then_some(n as i16)};
  let Some(start)=coordinate(unit.x).zip(coordinate(unit.y))else{return RouteResult::Blocked;};
  let Some(goal)=coordinate(dx).zip(coordinate(dy)).map(|(x,y)|(x+8,y+8))else{return RouteResult::Blocked;};
  let profile=if unit.kind=="apc"&&unit.crew.as_ref().is_some_and(|c|c.kind=="tough"){original_terrain::Profile::Tracked}else{original_terrain::profile(&unit.kind)};
  let role=unit.native_slot.and_then(|slot|self.native_group_rosters.role(slot)).unwrap_or(0);let special_tough=unit.kind=="tough"&&role<1;
  let path_allowed=|code|if special_tough{route::tough_leader_passable(code)}else{original_terrain::passable(code,self.map.world,profile)};
  let terrain=|p|self.route_terrain_code(p);let mask_kind=Self::route_collision_kind(&unit.kind);let vehicle=mask_kind!=collision::MaskKind::Foot;
  let permission=|cell:i16,octant:u8|{
   if !(0..16384).contains(&i32::from(cell))||!self.route_building_open(i32::from(cell)){return -1;}
   let code=terrain((cell&127,cell>>7));collision::world_permission(code,mask_kind,unit.team.clamp(0,2)as u8,octant,self.map.world==1)
  };
  let open=|cell:i32|{(0..16384).contains(&cell)&&self.route_building_open(cell)&&path_allowed(terrain(((cell&127)as i16,(cell>>7)as i16)))};
  let admit66=unit.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).is_some_and(|m|m.flagsb7&1!=0);
  let ray_open=|cell:i32|{if !(0..16384).contains(&cell){return false;}let code=terrain(((cell&127)as i16,(cell>>7)as i16));code==30||(code!=66||admit66)&&open(cell)};
  let price=|code|if !vehicle&&self.map.world==1&&code==9{12}else{original_terrain::route_cost(code,profile)};
  let plan=route::plan_road(graph,start,goal,((self.map.width*16)as i16,(self.map.height*16)as i16),terrain,|code|!path_allowed(code),price,self.map.native_level_record,true);
  let mut result=VecDeque::new();let mut previous=start;
  let append=|result:&mut VecDeque<(f64,f64)>,point:route::Point|{let p=(f64::from(point.0)*2.,f64::from(point.1)*2.);if result.back()!=Some(&p){result.push_back(p);}};
  for point in plan.points{let anchor=route::segment_anchor(point);let contact=collision::trace_before_response(previous,anchor,vehicle,permission);
   if contact.axes==0{if previous!=anchor{append(&mut result,anchor);}previous=anchor;continue;}
   let local_start=contact.position;
   if previous!=local_start&&self.native_route_segment_clear(previous,local_start,&unit.kind,unit.team){append(&mut result,local_start);}
   let alternate=if vehicle{let current=self.units.iter().find(|u|u.id==unit.id).unwrap_or(unit);let next=!current.native_route_flip.get();current.native_route_flip.set(next);next}else{false};
   let offset=if alternate{(-7,-8)}else{(8,8)};let start_cell=i32::from((local_start.0+offset.0)>>4)+i32::from((local_start.1+offset.1)>>4)*128;
   let mut marked=route::local::target_mask(local_start,point,vehicle,alternate,127,ray_open);
   let(mut candidates,steps)=route::local::search_with_steps(start_cell,contact.x,contact.y,vehicle,0,&mut marked,open);
   let mut selected=route::choose_local(std::array::from_fn(|i|candidates[i].as_ref().map(|c|(c.score,c.count())).unwrap_or((0,0))));
   if selected.is_none()&&vehicle{
    //0043dcf0 does not re-enter43e4a0 or toggleB4. It adds the fallback
    //goal ray to the existing marks before repeating43dad0's four probes.
    let fallback=route::local::target_mask(local_start,point,true,alternate,2,ray_open);
    for(old,new)in marked.iter_mut().zip(fallback){if new==255{*old=255;}}
    candidates=route::local::search_with_steps(start_cell,contact.x,contact.y,true,steps,&mut marked,open).0;
    selected=route::choose_local(std::array::from_fn(|i|candidates[i].as_ref().map(|c|(c.score,c.count())).unwrap_or((0,0))));
   }
   let Some(selected)=selected else{return if result.is_empty(){RouteResult::Blocked}else{RouteResult::Path(result)};};
   let mut packed=candidates[selected].take().unwrap().points;
   let goal_cell=i32::from(point.0>>4)+i32::from(point.1>>4)*128;
   if [-129,-128,-127,-1,1,127,128,129].into_iter().all(|d|open(goal_cell+d)){
    let cell=goal_cell as i16;if packed.last()!=Some(&cell){if packed.len()>1&&packed[packed.len()-2]==cell{packed.pop();}else{packed.push(cell);}}
   }
   route::local::simplify(&mut packed,|from,to|{
    let from=from.map(|p|route::segment_anchor(route::local_waypoint(p as u16,vehicle))).unwrap_or(local_start);let to=route::segment_anchor(route::local_waypoint(to as u16,vehicle));self.native_route_segment_clear(to,from,&unit.kind,unit.team)
   });
   let actual=self.units.iter().find(|u|u.id==unit.id).unwrap_or(unit);
   if let Some(roads)=route::replace_local_with_roads(graph,local_start,&packed,vehicle,actual.native_route_road_replaced.get(),((self.map.width*16)as i16,(self.map.height*16)as i16),terrain,|code|!path_allowed(code),price,self.map.native_level_record){
    actual.native_route_road_replaced.set(true);
    for point in roads{append(&mut result,route::segment_anchor(point));}
    return if result.is_empty(){RouteResult::Blocked}else{RouteResult::Partial(result)};
   }
   actual.native_route_road_replaced.set(false);
   let mut from=local_start;
   for cell in packed{
    let end=route::segment_anchor(route::local_waypoint(cell as u16,vehicle));if end==from{continue;}
    let hit=collision::trace_before_response(from,end,vehicle,permission);
    if hit.axes!=0{
     //4518e0 preserves a real foot actor's corrected position and requests
     //the local planner after contact. Native contour points are not all
     //forward-prevalidated:43c1b0 tests shortcuts in the reverse direction.
     //Keep the traversable prefix under the existing deferred Partial API.
     if hit.position!=from&&self.native_route_segment_clear(from,hit.position,&unit.kind,unit.team){append(&mut result,hit.position);}
     return if result.is_empty(){RouteResult::Blocked}else{RouteResult::Partial(result)};
    }
    append(&mut result,end);from=end;
   }
   return if result.is_empty(){RouteResult::Blocked}else{RouteResult::Partial(result)};
  }
  RouteResult::Path(result)
 }
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn authored_first_mission_foot_route_around_home_fort_does_not_stall(){
  let source=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/original/campaign.json");
  let Ok(source)=std::fs::read_to_string(source)else{return;};let campaign:serde_json::Value=serde_json::from_str(&source).unwrap();
  let mut g=Game::new(0,1,991);g.apply_original(&campaign["missions"][0]).unwrap();g.ai_teams.clear();
  let i=g.units.iter().position(|u|u.id==8).unwrap();let u=g.units[i].clone();assert_eq!((u.x,u.y),(1216.,2240.));
  let route=g.route_native_unit(&u,766.,2137.);assert!(matches!(route,RouteResult::Path(ref p)|RouteResult::Partial(ref p) if !p.is_empty()));
  g.units[i].path=g.route_unit(&u,766.,2137.);g.units[i].order="move".into();g.units[i].dest=Some((766.,2137.));
  for tick in 1..=2400{g.move_original(i,tick,1,true);if (g.units[i].x-766.).hypot(g.units[i].y-2137.)<2.{return;}}
  panic!("native movement did not finish its partial route: {:?}",(g.units[i].x,g.units[i].y));
 }
 fn field(kind:&str)->Game{
  let mut g=Game::new(0,1,5271);let(w,h)=(64,64);g.map.width=w;g.map.height=h;g.map.original=true;g.map.world=0;
  g.map.tiles=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.terrain_codes=vec![10;w*h];g.map.native_terrain_plane=vec![127;16384];g.map.territory=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_level_record=0;
  g.map.native_road_graph=Some(route::RoadGraph{regions:vec![route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
  g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.fires.clear();g.events.clear();g.native_strategy=None;g.native_world=None;g.winner=None;g.next_id=1;g.spawn(kind,0,928.,1024.,0,None);g
 }
 #[test]fn free_native_segments_keep_pixel_targets_and_do_not_start_a_local_search(){
  let g=field("medium");let u=g.units[0].clone();let RouteResult::Path(path)=g.route_native_unit(&u,1282.,1050.)else{panic!("free path");};
  assert_eq!(path,VecDeque::from([(1282.,1050.)]));assert!(!g.units[0].native_route_flip.get());assert_eq!(g.units[0].native_route_pending.get(),None);
 }
 #[test]fn native_foot_contour_uses_original_lower_body_anchor_and_preserves_the_move_goal(){
  let mut g=field("grunt");for y in 20..=40{for x in 30..=34{g.map.terrain_codes[y*64+x]=43;}}
  let u=g.units[0].clone();let RouteResult::Partial(path)=g.route_native_unit(&u,1280.,1024.)else{panic!("wall needs native contour");};assert!(!path.is_empty());assert_eq!(g.units[0].native_route_pending.get(),Some((1280.,1024.)));assert!(!g.units[0].native_route_flip.get());
  let mut from=(u.x,u.y);for &to in &path{assert_eq!(g.native_segment_clear(&u,from,to),Some(true),"{from:?}->{to:?}");from=to;}
  assert!(path.iter().any(|p|p.1==1308.),"Original foot contour anchor is tileY*32-4: {path:?}");
  assert!(path.iter().any(|&to|!g.movement_segment_clear_for(u.x,u.y,to.0,to.1,"grunt")),"The native route must not inherit the old anchor-only predicate");
 }
 #[test]fn local_vehicle_search_toggles_the_actual_actor_when_the_route_argument_is_a_clone(){
  let mut g=field("medium");for y in 20..=40{for x in 30..=34{g.map.terrain_codes[y*64+x]=43;}}
  let u=g.units[0].clone();let first=g.route_native_unit(&u,1280.,1024.);assert!(matches!(first,RouteResult::Partial(_)|RouteResult::Path(_)));assert!(g.units[0].native_route_flip.get());assert!(!u.native_route_flip.get());
  let _=g.route_native_unit(&u,1280.,1024.);assert!(!g.units[0].native_route_flip.get());
 }
 #[test]fn adopting_a_normal_contour_clears_the_original_road_replacement_suppression_bit(){
  let mut g=field("grunt");for y in 20..=40{for x in 30..=34{g.map.terrain_codes[y*64+x]=43;}}g.units[0].native_route_road_replaced.set(true);
  let u=g.units[0].clone();assert!(matches!(g.route_native_unit(&u,1280.,1024.),RouteResult::Partial(_)));assert!(!g.units[0].native_route_road_replaced.get());
 }
 #[test]fn graphless_maps_defer_to_the_compatibility_adapter(){
  let mut g=field("grunt");g.map.native_road_graph=None;let u=g.units[0].clone();assert!(matches!(g.route_native_unit(&u,1280.,1024.),RouteResult::Unavailable));assert_eq!(g.native_segment_clear(&u,(u.x,u.y),(1280.,1024.)),None);
 }
}
