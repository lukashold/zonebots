//! Native routing kernels. Road records are authored MAP data, not an inferred
//! road-weight graph. The incremental local-world planner remains separate.
use serde::{Serialize,Deserialize};
#[path="original_route_local.rs"] pub mod local;
pub type Point=(i16,i16);
#[derive(Clone,Serialize,Deserialize,Debug)]
#[serde(rename_all="camelCase")]
pub struct RoadRegion{
 pub bounds:[i16;4],pub waypoint:u16,pub next:Vec<u8>,
}
#[derive(Clone,Serialize,Deserialize,Debug)]
#[serde(rename_all="camelCase")]
pub struct RoadGraph{pub regions:Vec<RoadRegion>}
impl RoadGraph{
 ///00431c10 copies96 records from the version-9 MAP block at0xa971.
 pub fn from_bytes(data:&[u8])->Option<Self>{
  if data.len()!=96*0x88{return None;}
  let regions=data.chunks_exact(0x88).map(|r|RoadRegion{bounds:std::array::from_fn(|i|i16::from_le_bytes([r[i*2],r[i*2+1]])),waypoint:u16::from_le_bytes([r[8],r[9]]),next:r[0x28..0x88].to_vec()}).collect();Some(Self{regions})
 }
 ///0043b5f0: first nonzero-left-bound inclusive rectangle wins.
 pub fn region_at(&self,p:Point)->Option<u8>{self.regions.iter().position(|r|r.bounds[0]!=0&&p.0>=r.bounds[0]&&p.0<=r.bounds[2]&&p.1>=r.bounds[1]&&p.1<=r.bounds[3]).and_then(|n|u8::try_from(n).ok())}
 ///0043b6f0 follows next[current][destination] and excludes the destination
 /// itself. The authored matrix is directional; do not reverse its indices.
 pub fn intermediate_anchors(&self,current:u8,destination:u8)->Result<Vec<Point>,RoadFailure>{
  let walk=self.walk_intermediate_anchors(current,destination);if let Some(failure)=walk.failure{Err(failure)}else{Ok(walk.points)}
 }
 /// Retains partial native output for diagnostics. The caller must discard
 /// partial output on failure, as0043bbe0 does before falling back.
 pub fn walk_intermediate_anchors(&self,mut current:u8,destination:u8)->RoadWalk{
  let mut out=Vec::new();
  while current!=destination{
   let Some(&next)=self.regions.get(usize::from(current)).and_then(|r|r.next.get(usize::from(destination)))else{return RoadWalk{points:out,failure:Some(RoadFailure::InvalidRegion)};};
   if next==255{return RoadWalk{points:out,failure:Some(RoadFailure::Disconnected)};}
   if next==destination{return RoadWalk{points:out,failure:None};}
   let Some(region)=self.regions.get(usize::from(next))else{return RoadWalk{points:out,failure:Some(RoadFailure::InvalidRegion)};};out.push(road_center(region.waypoint));current=next;
   if out.len()>=80{return RoadWalk{points:out,failure:Some(RoadFailure::NodeLimit)};}
  }
  RoadWalk{points:out,failure:None}
 }
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum RoadFailure{Disconnected,InvalidRegion,NodeLimit}
#[derive(Clone,Debug)]
pub struct RoadWalk{pub points:Vec<Point>,pub failure:Option<RoadFailure>}
///0043b6f0's authored intermediate anchor, excluding current/destination.
pub fn road_center(packed:u16)->Point{(((packed&127)*16+15)as i16,((packed>>3)&0x1ff0).wrapping_add(15)as i16)}
///0043e190 decodes local packed cells differently from road-network centers.
pub fn local_waypoint(packed:u16,vehicle:bool)->Point{(((packed&127)*16+if vehicle{15}else{8})as i16,(((packed as i16>>3)&!15)+if vehicle{8}else{6}))}
pub fn segment_anchor(waypoint:Point)->Point{(waypoint.0.wrapping_sub(8),waypoint.1.wrapping_sub(8))}
///0041b950 selects0049a9a0 only for class3 with signed role<1. Its water
///entries are not rewritten by00433b30's ordinary infantry-world overlay.
pub fn tough_leader_passable(code:u8)->bool{
 const MASK:[u8;67]=[0,0,0,0,0,0,0,0,0,0,0,255,255,255,255,255,255,0,0,255,0,0,0,0,255,255,0,0,0,255,0,255,255,255,255,0,0,0,0,0,255,0,0,255,0,0,0,255,255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,0,0,0];
 MASK.get(usize::from(code&127))==Some(&0)
}
fn road(code:u8)->bool{matches!(code&127,7|23|51)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RoadEndpoint{pub point:Point,pub already_on_road:bool}
///0043ae30 aligns only authored two/four-cell-wide roads. Wider or odd-width
/// roads retain the incoming tile before the final+15 anchor adjustment.
pub fn align_road(point:Point,dimensions:Point,mut terrain:impl FnMut(Point)->u8)->Point{
 let tile=(point.0>>4,point.1>>4);if !road(terrain(tile)){return point;}
 let(mut x,mut y)=(point.0&!15,point.1&!15);
 let mut left=tile.0;let mut count=0;while left>=0&&road(terrain((left,tile.1))){count+=1;left-=1;}
 if count<5{let mut right=left+1;count=0;while right*16<dimensions.0&&road(terrain((right,tile.1))){right+=1;count+=1;}if count==4{x=right*16-48;}else if count==2{x=right*16-32;}}
 let mut top=tile.1;count=0;while top>=0&&road(terrain((tile.0,top))){count+=1;top-=1;}
 if count<5{let mut bottom=top+1;count=0;while bottom*16<dimensions.1&&road(terrain((tile.0,bottom))){bottom+=1;count+=1;}if count==4{y=bottom*16-48;}else if count==2{y=bottom*16-32;}}
 (x.wrapping_add(15),y.wrapping_add(15))
}
///0043b090/0043afd0 scan N,S,E,W,NE,SE,SW,NW, preserving ties in that order.
/// A ray crossing a nonzero unit+18 permission byte adds16 once; diagonals
/// multiply the complete integer score by1.5. A failed ray is not a candidate.
#[derive(Clone,Copy,Debug)]pub struct EndpointTrace{pub distances:[i32;8],pub endpoint:Option<RoadEndpoint>}
pub fn road_endpoint(point:Point,dimensions:Point,terrain:impl FnMut(Point)->u8,flagged:impl FnMut(u8)->bool)->Option<RoadEndpoint>{road_endpoint_trace(point,dimensions,terrain,flagged).endpoint}
pub fn road_endpoint_trace(point:Point,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool)->EndpointTrace{
 let mut distances=[i32::MAX;8];let on_road=road(terrain((point.0>>4,point.1>>4)));let mut best=None;let mut best_score=i32::MAX;
 if on_road{best=Some(point);}else{
  for(i,(dx,dy))in [(0,-16),(0,16),(16,0),(-16,0),(16,-16),(16,16),(-16,16),(-16,-16)].into_iter().enumerate(){
   let(mut x,mut y)=point;let mut distance=0;let mut obstructed=false;
   while x>0&&x<2048&&y>0&&y<2048{let code=terrain((x>>4,y>>4))&127;distance+=1;if road(code){let mut score=distance+if obstructed{16}else{0};if i>=4{score+=score>>1;}distances[i]=score;if score<best_score{best_score=score;best=Some((x,y));}break;}obstructed|=flagged(code);x=x.wrapping_add(dx);y=y.wrapping_add(dy);}
  }
 }
 EndpointTrace{distances,endpoint:best.map(|p|RoadEndpoint{point:align_road(p,dimensions,terrain),already_on_road:on_road})}
}
/// Four100-byte candidate headers: price low byte; count high byte, negative
/// only when complete. Native strict comparison keeps the first equal price.
pub fn choose_local(headers:[(u8,i8);4])->Option<usize>{let mut best=255;let mut choice=None;for(i,(price,count))in headers.into_iter().enumerate(){if count<0&&price<best{best=price;choice=Some(i);}}choice}
///0045cdc0/0045cf90's dominant-axis raster, excluding the final cell. Equal
/// x coordinates reverse the endpoints; this matters for sampled terrain.
pub fn line_cells(a:Point,b:Point)->Vec<Point>{
 if a.0<0||a.1<0||b.0<0||b.1<0{return Vec::new();}
 let(mut x,mut y)=(i32::from(a.0>>4),i32::from(a.1>>4));let(mut ex,mut ey)=(i32::from(b.0>>4),i32::from(b.1>>4));
 if x>=ex{std::mem::swap(&mut x,&mut ex);std::mem::swap(&mut y,&mut ey);}
 let ax=ex-x;let ay=(ey-y).abs();let sy=if ey<y{-1}else{1};let length=ax.max(ay);let mut error=length>>1;let mut cells=Vec::with_capacity(length as usize);
 for _ in 0..length{cells.push((x as i16,y as i16));if ay<ax{x+=1;error-=ay;if error<0{error+=ax;y+=sy;}}else{y+=sy;error-=ax;if error<0{error+=ay;x+=1;}}}
 cells
}
#[derive(Clone,Copy,PartialEq,Eq,Debug,Default)]
pub struct Price{pub cost:i32,pub special_tiles:i16}
///0045cf90: the second and subsequent consecutive terrain prices>=10 are
/// halved. This high-price-run state resets for each polyline leg, not tile.
pub fn path_price(points:&[Point],mut terrain:impl FnMut(Point)->u8,mut price:impl FnMut(u8)->u8)->Price{
 let mut result=Price::default();for pair in points.windows(2){let mut high=false;for cell in line_cells(pair[0],pair[1]){let code=terrain(cell)&127;if matches!(code,40|52){result.special_tiles=result.special_tiles.wrapping_add(1);}let raw=price(code);let reduced=if high&&raw>=10{raw>>1}else{raw};result.cost=result.cost.wrapping_add(i32::from(reduced));high=raw>=10;}}
 result
}
///0043ba60: direct wins ties, except level-record5 disallows a direct alternative
/// crossing more than one raw code40/52 sample. This does not assert passability.
pub fn prefer_direct(road:Price,direct:Price,level_record:i16)->bool{road.cost>=if level_record==5&&direct.special_tiles>1{0x0fff_ffff}else{direct.cost}}
#[derive(Clone,Debug)]
pub struct RoadPlan{pub points:Vec<Point>,pub graph_succeeded:bool,pub retained_roads:bool}
///0043bbe0: roads are considered only at squared native distance>=0x3840.
/// Failed endpoint/region/next-hop queries leave the original single goal.
/// Successful candidates are assembled in travel order before0043ba60's
/// price comparison. The raw goal is a queue coordinate, not a segment anchor.
pub fn plan_road(graph:&RoadGraph,start:Point,goal:Point,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8,level_record:i16,compare_direct:bool)->RoadPlan{
 let fallback=||RoadPlan{points:vec![goal],graph_succeeded:false,retained_roads:false};
 let dx=i32::from(goal.0)-i32::from(start.0);let dy=i32::from(goal.1)-i32::from(start.1);if dx*dx+dy*dy<0x3840{return fallback();}
 let Some(end)=road_endpoint(goal,dimensions,&mut terrain,&mut flagged)else{return fallback();};
 let Some(begin)=road_endpoint(start,dimensions,&mut terrain,&mut flagged)else{return fallback();};
 let Some(from)=graph.region_at(begin.point)else{return fallback();};let Some(to)=graph.region_at(end.point)else{return fallback();};
 if from==to||graph.regions.get(usize::from(to)).and_then(|r|r.next.get(usize::from(from)))==Some(&255){return fallback();}
 let Ok(middle)=graph.intermediate_anchors(from,to)else{return fallback();};
 let mut points=Vec::new();if !begin.already_on_road{points.push(begin.point);}points.extend(middle);if !end.already_on_road{points.push(end.point);}points.push(goal);
 if points.len()>80{return fallback();}
 if compare_direct{let mut full=vec![start];full.extend_from_slice(&points);let via=path_price(&full,&mut terrain,&mut price);let direct=path_price(&[start,goal],&mut terrain,&mut price);if prefer_direct(via,direct,level_record){return RoadPlan{points:vec![goal],graph_succeeded:true,retained_roads:false};}}
 RoadPlan{points,graph_succeeded:true,retained_roads:true}
}

///0043df20 considers a new road route only when the completed, simplified
/// local candidate costs>=30 and unit9c bit1 is clear. Ties keep the contour.
pub fn replace_local_with_roads(graph:&RoadGraph,start:Point,local:&[i16],vehicle:bool,suppressed:bool,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8,level_record:i16)->Option<Vec<Point>>{
 if suppressed||local.is_empty(){return None;}
 let mut points=vec![start];points.extend(local.iter().map(|&p|local_waypoint(p as u16,vehicle)));
 let old=path_price(&points,&mut terrain,&mut price).cost;if old<30{return None;}
 let road=plan_road(graph,start,*points.last()?,dimensions,&mut terrain,&mut flagged,&mut price,level_record,false);
 if !road.graph_succeeded{return None;}points.truncate(1);points.extend_from_slice(&road.points);
 (path_price(&points,terrain,price).cost<old).then_some(road.points)
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,ns:&[i32]){for n in ns{*h=(*h^*n as u32).wrapping_mul(16777619);}}
 #[test]fn all_48_native_special_path_masks_match_tough_roles_and_apc_saved_driver_class(){
  use super::super::original_terrain::{self as terrain,Profile};let mut h=2166136261;
  for world in 0..2{for team in 0..3{for kind in 0..2{for role in [-1,0,1,3]{
   hash(&mut h,&[world,team,kind,role]);for code in 0..67{let pass=if kind==0&&role<1{tough_leader_passable(code)}else{terrain::passable(code,world as usize,if kind==0{Profile::Infantry}else if role==3{Profile::Tracked}else{Profile::Wheeled})};hash(&mut h,&[if pass{0}else{-1}]);}
  }}}}
  assert_eq!(h,3487848189);
 }
 #[test]fn all_240_native_local_to_road_replacements_match_threshold_suppression_and_strict_price(){
  let starts=[(160,656),(160,500),(500,160),(1120,656),(960,880)];let goals=[(1120,656),(1120,500),(1000,880),(176,672),(170,656),(1200,1300)];let mut h=2166136261;
  let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(7).enumerate(){r.bounds=[(8+i as i16*12)*16,32*16,(20+i as i16*12)*16-1,50*16];r.waypoint=(14+i*12+40*128)as u16;for j in 0..7{r.next[j]=if i==j{j as u8}else if j>i{i as u8+1}else{i as u8-1};}}let graph=RoadGraph{regions};
  for mode in 0..2{for(si,&start)in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){for vehicle in 0..2{for suppressed in 0..2{
   let sx=start.0>>4;let gx=goal.0>>4;let gy=goal.1>>4;let local=[sx+20*128,gx+20*128,gx+gy*128];let terrain=|(x,y)|if(40..=41).contains(&y)&&(8..=100).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0};let price=|code|match code{7=>2,40=>72,_=>5};
   let replacement=replace_local_with_roads(&graph,start,&local,vehicle==1,suppressed==1,(2048,2048),terrain,|code|code==40,price,if mode==1{5}else{0});let points=replacement.as_deref().unwrap_or(&[]);
   hash(&mut h,&[mode,si as i32,gi as i32,vehicle,suppressed,i32::from(replacement.is_some()),points.len()as i32]);for p in points{hash(&mut h,&[i32::from(p.0),i32::from(p.1)]);}
  }}}}}
  assert_eq!(h,3387675825);
 }
 #[test]fn all_240_native_raster_prices_and_special_tile_counts_match(){
  let starts=[(16,16),(31,47),(127,64),(1023,1023)];let deltas=[(-47,-32),(-32,0),(-17,31),(0,-47),(0,0),(0,63),(16,32),(17,31),(64,-15),(79,48)];let costs=[2,5,10,12,72];let mut h=2166136261;
  for mode in 0..3{for(si,&s)in starts.iter().enumerate(){for(di,&d)in deltas.iter().enumerate(){for legs in 1..3{
   let mut points=vec![s,(s.0+d.0,s.1+d.1)];if legs==2{points.push((s.0+32,s.1+16));}
   let p=path_price(&points,|(x,y)|((i32::from(x)*7+i32::from(y)*11+mode*3)%67)as u8,|code|costs[(i32::from(code)+mode)as usize%5]);hash(&mut h,&[mode,si as i32,di as i32,legs,p.cost,i32::from(p.special_tiles)]);
  }}}}
  assert_eq!(h,2488277043);
 }
 #[test]fn all_49_region_queries_match_the_first_inclusive_authored_rectangle(){
  let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(3).enumerate(){let n=i as i16*100;r.bounds=[n,n,n+200,n+200];}let graph=RoadGraph{regions};let mut h=2166136261;
  for x in [0,99,100,200,300,400,401]{for y in [0,99,100,200,300,400,401]{let r=graph.region_at((x,y));hash(&mut h,&[i32::from(x),i32::from(y),i32::from(r.is_none()),i32::from(r.unwrap_or(96))]);}}
  assert_eq!(h,2347053961);
 }
 #[test]fn all_99_authored_next_hop_chains_match_native_including_failure_and_80_node_limit(){
  let mut h=2166136261;
  for variant in 0..3{for current in 0..7{for destination in 0..7{
   if variant==2&&(current!=1||destination!=6){continue;}
   let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];
   for(i,r)in regions.iter_mut().take(7).enumerate(){r.waypoint=((i+5)+(i+7)*128)as u16;for j in 0..7{r.next[j]=if variant==2{if i==1{2}else{1}}else if variant==1&&i==3{255}else if i==j{j as u8}else if j>i{i as u8+1}else{i as u8-1};}}
   let walk=RoadGraph{regions}.walk_intermediate_anchors(current,destination);hash(&mut h,&[variant,i32::from(current),i32::from(destination),i32::from(walk.failure.is_some()),walk.points.len()as i32]);for p in walk.points{hash(&mut h,&[i32::from(p.0),i32::from(p.1)]);}
  }}}
  assert_eq!(h,2883018638);
 }
 #[test]fn all_4096_local_candidate_header_sets_match_native_selection(){
  let mut h=2166136261;
  for fixture in 0..4096{let mut f=fixture;let headers=std::array::from_fn(|_|{let v=f&7;f>>=3;([0,10,254,255][v&3],if v&4!=0{-2}else{0})});let choice=choose_local(headers).map(|i|i as i32).unwrap_or(-1);hash(&mut h,&[fixture as i32,choice]);}
  assert_eq!(h,2604452508);
 }
 #[test]fn all_20_completed_candidate_waypoints_match_native_body_anchor_offsets(){
  let mut h=2166136261;for vehicle in 0..2{for packed in [0,1,127,128,129,255,256,4095,8191,16383]{let p=local_waypoint(packed,vehicle!=0);hash(&mut h,&[vehicle,i32::from(packed),i32::from(p.0),i32::from(p.1)]);}}
  assert_eq!(h,587214453);assert_eq!(segment_anchor(local_waypoint(129,false)),(16,14));assert_eq!(segment_anchor(local_waypoint(129,true)),(23,16));
 }
 #[test]fn all_216_native_direct_route_comparisons_keep_ties_and_record5_exception(){
  let mut h=2166136261;
  for road in [0,5,10,0x0fffffff]{for direct in [0,5,10]{for special in 0..3{for level in [0,5,6]{for multi in 0..2{
   let choice=prefer_direct(Price{cost:road,special_tiles:0},Price{cost:direct,special_tiles:special},level);hash(&mut h,&[road,direct,i32::from(special),i32::from(level),multi,i32::from(choice)]);
  }}}}}
  assert_eq!(h,2134840169);
 }
 #[test]fn all_64_road_endpoint_scans_match_native_ties_permission_penalty_and_road_width_alignment(){
  let mut h=2166136261;let points=[(512,512),(513,525),(16,16),(1000,900),(0,0),(2047,2047),(960,640),(320,640)];
  let expected=[[(527,655),(527,655),(655,655),(991,911),(0,0),(991,1023),(975,655),(335,655)],[(527,527),(527,527),(31,271),(1039,911),(15,15),(2047,1807),(1039,655),(271,655)],[(0,0);8],[(527,495),(527,495),(31,255),(975,911),(15,15),(2047,1935),(975,655),(335,735)]];
  for mode in 0..4{for(index,&p)in points.iter().enumerate(){for flagged in 0..2{
   let terrain=|(x,y):Point|{let mut code=if (x+y)%9==0{1}else{0};let r=match mode{0=>(40..=41).contains(&y)&&(10..=100).contains(&x)||(60..=63).contains(&x)&&(10..=100).contains(&y),1=>x%16==0||y%16==0,3=>x%15==0||y%15==0,_=>false};if r{code=if mode==3{[7,23,51][((x+y)%3)as usize]}else{7};}code};
   let result=road_endpoint(p,(2048,2048),terrain,|code|flagged!=0&&code==1);let found=result.map(|r|r.point).unwrap_or((0,0));if flagged==0{assert_eq!(found,expected[mode as usize][index],"mode{mode} point{index}");}hash(&mut h,&[mode,index as i32,flagged,i32::from(result.is_none()),i32::from(found.0),i32::from(found.1),i32::from(result.is_some_and(|r|r.already_on_road))]);
  }}}
  assert_eq!(h,2039233695);
 }
 #[test]fn all_240_complete_global_road_plans_match_native_endpoint_graph_and_direct_selection(){
  let starts=[(160,656),(160,500),(500,160),(1120,656),(960,880)];let goals=[(1120,656),(1120,500),(1000,880),(176,672),(170,656),(1200,1300)];let mut h=2166136261;
  let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(7).enumerate(){let n=i as i16;r.bounds=[(8+n*12)*16,32*16,(20+n*12)*16-1,50*16];r.waypoint=(14+i*12+40*128)as u16;for j in 0..7{r.next[j]=if i==j{j as u8}else if j>i{i as u8+1}else{i as u8-1};}}let graph=RoadGraph{regions};
  for mode in 0..2{for(si,&start)in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){for vehicle in 0..2{for compare in 0..2{
   let terrain=|(x,y):Point|if (40..=41).contains(&y)&&(8..=100).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0};
   let p=plan_road(&graph,start,goal,(2048,2048),terrain,|code|code==40,|code|match code{7=>2,40=>72,_=>5},if mode==1{5}else{0},compare!=0);
   hash(&mut h,&[mode,si as i32,gi as i32,vehicle,compare,i32::from(!p.graph_succeeded),p.points.len()as i32]);for point in p.points{hash(&mut h,&[i32::from(point.0),i32::from(point.1)]);}
  }}}}}
  assert_eq!(h,47551869);
 }
}
