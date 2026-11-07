//!43df20's persistent scratch domain, including the road allocator's overlap
//!with the endpoint distances and contour candidate records.
use crate::original_route::{self as road,Point,RoadGraph};
use crate::original_route_pool::{Node,FREE,road as assembly};
#[path="original_road_shortcut_game.rs"]mod shortcut_game;
pub const BASE:u32=0x4d1480;
const CANDIDATE_BASE:usize=368;
const CANDIDATE_BYTES:usize=100;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{ContourBuffer{count:usize,address:u32},ScratchAlias(u32),Region(u8),ScratchCycle(u32),FiniteAdoption}
#[derive(Clone,Copy,Debug)]pub struct Input{pub actor:Point,pub actor_token:u32,pub vehicle:bool,pub suppressed:bool,pub dimensions:Point,pub level:i16}
#[derive(Clone,Debug,PartialEq,Eq)]pub enum Decision{Contour,Road(Vec<[i16;2]>)}
#[derive(Clone,Debug)]pub struct State{
 ///Raw bytes preserve warm coordinates and the road list's shared tail.
 pub frame:Box<[u8;640]>,pub pairs:Box<[[i16;2];41]>,pub trace:assembly::Trace,
 ///The third candidate extends past the 80 road records; the fourth ends
 ///immediately before pairs[0]. Keep their physical suffix between workers.
 pub candidate_tail:Box<[u8;128]>,
 pub scratch_mode:u8,pub compare_mode:u8,pub record_cursor:u32,pub price_table:u32,
 pub(crate) raster:road::raster::Raster,
 pub dummy_pause_bc:u8,pub dummy_counter67:i8,pub old_price:Option<i32>,pub new_price:Option<i32>,pub partial_adoption_head:Option<u32>,
 candidate_writes:[bool;3],
 ///Diagnostic, not a source flag:45cf90/43b950 raster scratch writes are
 ///not yet one shared authority with other pure ray/geometry ports.
 pub unshared_raster_globals:bool,
}
impl Default for State{fn default()->Self{Self{frame:Box::new([0;640]),candidate_tail:Box::new([0;128]),pairs:Box::new([[0;2];41]),trace:Default::default(),scratch_mode:0,compare_mode:0,record_cursor:0,price_table:0,raster:Default::default(),dummy_pause_bc:0,dummy_counter67:0,old_price:None,new_price:None,partial_adoption_head:None,candidate_writes:[false;3],unshared_raster_globals:false}}}
impl State{
 pub fn dword(&self,offset:usize)->u32{u32::from_le_bytes(self.frame[offset..offset+4].try_into().unwrap())}
 fn dword_set(&mut self,offset:usize,n:u32){self.frame[offset..offset+4].copy_from_slice(&n.to_le_bytes());}
 pub fn synchronize_context(&mut self,origin:i16,cursor:u32,vehicle:bool){self.frame[356..358].copy_from_slice(&origin.to_le_bytes());self.dword_set(360,cursor);self.price_table=if vehicle{0x499ec8}else{0x499b98};}
 fn shared_word(&self,offset:usize)->i16{
  if offset<640{i16::from_le_bytes(self.frame[offset..offset+2].try_into().unwrap())}
  else if offset<768{i16::from_le_bytes(self.candidate_tail[offset-640..offset-638].try_into().unwrap())}
  else{let word=(offset-768)/2;self.pairs[word/2][word%2]}
 }
 fn set_shared_word(&mut self,offset:usize,value:i16){
  if offset<640{self.frame[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
  else if offset<768{self.candidate_tail[offset-640..offset-638].copy_from_slice(&value.to_le_bytes());}
  else{let word=(offset-768)/2;self.pairs[word/2][word%2]=value;}
 }
 pub(crate) fn record_candidate(&mut self,index:usize,(score,count):(u8,i8),points:&[i16]){
  if index>4{return;}
  let offset=CANDIDATE_BASE+index*CANDIDATE_BYTES;
  self.set_shared_word(offset,i16::from_le_bytes([score,count as u8]));
  for (i,&cell) in points.iter().take((CANDIDATE_BYTES-2)/2).enumerate(){
   self.set_shared_word(offset+2+i*2,cell);
  }
 }
 pub fn synchronize_candidate(&mut self,selected:usize,price:u8,packed:&[i16],removed:bool){
  self.record_candidate(selected,(price,-(packed.len()as i8)),packed);
  if removed{self.record_candidate(selected+1,(255,-1),&[]);}
 }
 ///43c1b0 shifts the remaining words through the next candidate header,
 ///including old words beyond the active count.
 pub(crate) fn remove_candidate_point(&mut self,candidate:usize,point:usize){
  let base=CANDIDATE_BASE+candidate*CANDIDATE_BYTES;
  let start=base+2+point*2;
  let end=base+CANDIDATE_BYTES;
  for offset in (start..end).step_by(2){self.set_shared_word(offset,self.shared_word(offset+2));}
  self.record_candidate(candidate+1,(255,-1),&[]);
 }
 pub(crate) fn changed_candidate_header(&self,index:usize)->Option<(u8,i8)>{
  if !self.candidate_writes.get(index).copied()?{return None;}
  let offset=CANDIDATE_BASE+index*CANDIDATE_BYTES;
  Some((self.frame[offset],self.frame[offset+1]as i8))
 }
 ///43e190 reloads the selected contour from the shared bytes after a rejected
 ///road. Stop before a corrupt count makes it read beyond the candidate.
 pub(crate) fn reread_candidate(&self,index:usize,packed:&mut Vec<i16>)->Result<(),Boundary>{
  let Some((_,raw_count))=self.changed_candidate_header(index)else{return Ok(());};
  let offset=CANDIDATE_BASE+index*CANDIDATE_BYTES;
  let count=usize::from(raw_count.wrapping_neg()as u8);
  let available=(CANDIDATE_BYTES-2)/2;
  if count==0||count>available{
   let address=BASE+offset as u32+2+(available*2)as u32;
   return Err(Boundary::ContourBuffer{count:if count==0{256}else{count},address});
  }
  packed.resize(count,0);
  for (i,point) in packed.iter_mut().enumerate(){
   *point=self.shared_word(offset+2+i*2);
  }
  Ok(())
 }
 fn nodes(&self)->[Node;assembly::SCRATCH_NODES]{std::array::from_fn(|i|{let at=i*8;Node{point:[i16::from_le_bytes([self.frame[at],self.frame[at+1]]),i16::from_le_bytes([self.frame[at+2],self.frame[at+3]])],next:self.dword(at+4)}})}
 fn write_nodes(&mut self,nodes:&[Node;assembly::SCRATCH_NODES]){for(i,n)in nodes.iter().enumerate(){let at=i*8;self.frame[at..at+2].copy_from_slice(&n.point[0].to_le_bytes());self.frame[at+2..at+4].copy_from_slice(&n.point[1].to_le_bytes());self.dword_set(at+4,n.next);}}
 fn pair(&mut self,index:usize,p:[i16;2],count:usize)->Result<(),Boundary>{let Some(out)=self.pairs.get_mut(index)else{return Err(Boundary::ContourBuffer{count,address:0x4d1780+index as u32*4});};*out=p;Ok(())}
 ///The direct UNIT route and contour replacement share the native scratch
 ///words; the primary call never clears the 40 temporary node records.
 pub(crate) fn build_primary(&mut self,nodes:&mut[Node],head:u32,input:Input,graph:&RoadGraph,terrain:impl FnMut(Point)->u8,flagged:impl FnMut(u8)->bool,price:impl FnMut(u8)->u8)->Result<assembly::ResultState,assembly::Boundary>{
  self.dword_set(352,input.actor_token);self.dword_set(364,head);
  self.price_table=if input.vehicle{0x499ec8}else{0x499b98};
  let result=assembly::primary(nodes,head,input.actor,graph,input.dimensions,terrain,flagged,price,input.level,true,&mut self.trace,&mut self.raster);
  self.record_cursor=0x4d1340+self.trace.record_cursor as u32*4;
  if let Some(distances)=self.trace.endpoint_distances{for(i,n)in distances.into_iter().enumerate(){self.dword_set(320+i*4,n as u32);}}
  result
 }
 ///No mission reset: known60430 does not clear this scratch domain.
 pub fn consider(&mut self,input:Input,packed:&[i16],graph:&RoadGraph,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8)->Result<Decision,Boundary>{
  self.old_price=None;self.new_price=None;self.partial_adoption_head=None;self.candidate_writes.fill(false);if input.suppressed{return Ok(Decision::Contour);}
  self.pair(0,[input.actor.0,input.actor.1],packed.len())?;for(i,&cell)in packed.iter().enumerate(){let p=road::local_waypoint(cell as u16,input.vehicle);self.pair(i+1,[p.0,p.1],packed.len())?;}self.pair(packed.len()+1,[-1,-1],packed.len())?;
  let old_points=self.pairs[..packed.len()+1].iter().map(|p|(p[0],p[1])).collect::<Vec<_>>();let old=self.raster.price(&old_points,&mut terrain,&mut price).cost;self.old_price=Some(old);self.unshared_raster_globals=true;if old<30{return Ok(Decision::Contour);}
  self.scratch_mode=1;for i in 0..assembly::SCRATCH_CLEARED{self.dword_set(i*8+4,FREE);}let goal=self.pairs[packed.len()];self.frame[..2].copy_from_slice(&goal[0].to_le_bytes());self.frame[2..4].copy_from_slice(&goal[1].to_le_bytes());self.dword_set(4,0);self.compare_mode=1;
  //43bbe0's unconditional prefix precedes distance/endpoint failure.
  self.dword_set(352,input.actor_token);self.dword_set(364,BASE);self.price_table=if input.vehicle{0x499ec8}else{0x499b98};self.record_cursor=0x4d1340;
  let mut nodes=self.nodes();
  let before=*self.frame;
  let result=assembly::scratch(&mut nodes,input.actor,graph,input.dimensions,&mut terrain,&mut flagged,&mut price,input.level,&mut self.trace,&mut self.raster);self.write_nodes(&nodes);self.record_cursor=0x4d1340+self.trace.record_cursor as u32*4;
  for (index,written) in self.candidate_writes.iter_mut().enumerate(){
   let start=CANDIDATE_BASE+index*CANDIDATE_BYTES;
   let end=(start+CANDIDATE_BYTES).min(self.frame.len());
   *written=self.frame[start..end]!=before[start..end];
  }
  let result=match result{Ok(r)=>r,Err(assembly::Boundary::Node(p))=>return Err(Boundary::ScratchAlias(p)),Err(assembly::Boundary::Region(r))=>return Err(Boundary::Region(r))};self.compare_mode=0;
  if result.failed{self.scratch_mode=0;return Ok(Decision::Contour);}
  self.pairs[0]=[input.actor.0,input.actor.1];let mut points=Vec::new();let mut link=BASE;let mut seen=[false;assembly::SCRATCH_NODES];
  while link!=0&&points.len()<39{let delta=link.wrapping_sub(BASE);if delta&7!=0||delta/8>=assembly::SCRATCH_NODES as u32{return Err(Boundary::ScratchAlias(link));}let i=(delta/8)as usize;if seen[i]{return Err(Boundary::ScratchCycle(link));}seen[i]=true;let n=nodes[i];points.push(n.point);self.pairs[points.len()]=n.point;self.pairs[points.len()+1]=[-1,-1];link=n.next;}
  let new_points=self.pairs[..points.len()+1].iter().map(|p|(p[0],p[1])).collect::<Vec<_>>();let new=self.raster.price(&new_points,&mut terrain,&mut price).cost;self.new_price=Some(new);self.scratch_mode=0;Ok(if new<old{Decision::Road(points)}else{Decision::Contour})
 }
}
#[cfg(test)]mod tests{
 use super::*;use road::RoadRegion;
 #[test]
 fn candidate_shifts_preserve_physical_suffix_bytes(){
  let trace=std::env::var_os("Z_NATIVE_CANDIDATE_SHIFT").map(|p|std::fs::read_to_string(p).unwrap());
  let mut trace=trace.as_deref().map(str::lines);
  let(mut row,mut digest)=(0,2166136261u32);
  for selected in 0..4{for count in [3,8,37,49]{for pattern in 0..4{
   let before=(0..772).map(|i|(i*23+selected*17+31)as u8).collect::<Vec<_>>();
   let mut s=State::default();s.frame.copy_from_slice(&before[..640]);s.candidate_tail.copy_from_slice(&before[640..768]);
   s.pairs[0]=[i16::from_le_bytes([before[768],before[769]]),i16::from_le_bytes([before[770],before[771]])];
   let mut packed=(0..count).map(|i|(1280+i*17)as i16).collect::<Vec<_>>();
   s.record_candidate(selected,(7,-(count as i8)),&packed);
   let(mut calls,mut removals)=(0,Vec::new());
   road::local::simplify_with_removals(&mut packed,|_,_|{calls+=1;pattern==1||(pattern==2&&calls%2==0)||(pattern==3&&calls==count-1)},|i|removals.push(i));
   for &index in &removals{s.remove_candidate_point(selected,index);}
   s.synchronize_candidate(selected,7,&packed,!removals.is_empty());
   s.dword_set(360,BASE+368+selected as u32*100+(packed.len()as u32+1)*2);
   let actual=s.frame.iter().chain(s.candidate_tail.iter()).copied().chain(s.pairs[0].into_iter().flat_map(i16::to_le_bytes)).collect::<Vec<_>>();
   if let Some(trace)=&mut trace{
    let v:serde_json::Value=serde_json::from_str(trace.next().unwrap()).unwrap();
    assert_eq!(serde_json::json!([selected,count,pattern,calls]),serde_json::json!([v["selected"],v["count"],v["pattern"],v["calls"]]));
    assert_eq!(serde_json::json!(removals),v["removed"],"removal decisions {row}");
    let h=v["after"].as_str().unwrap();
    for i in 0..772{assert_eq!(actual[i],u8::from_str_radix(&h[i*2..i*2+2],16).unwrap(),"row {row}, byte {i}");}
   }
   for n in [selected as u32,count as u32,pattern,calls as u32,s.dword(360)].into_iter().chain(actual.into_iter().map(u32::from)){
    digest=(digest^n).wrapping_mul(16777619);
   }
   row+=1;
  }}}
  assert!(trace.is_none_or(|mut t|t.next().is_none()));
  assert_eq!((row,digest),(64,2477124827),"43c1b0 physical shifts; shortcut outcomes are explicit inputs");
 }
 fn mix(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}fn hash(b:&[u8])->u32{b.iter().fold(2166136261u32,|h,&n|(h^u32::from(n)).wrapping_mul(16777619))}
 fn graph(count:usize,thin:bool)->RoadGraph{let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];for(i,r)in regions.iter_mut().take(count).enumerate(){let left=if thin{8+i}else{8+i*12};let right=if thin{9+i}else{20+i*12};r.bounds=[left as i16*16,512,right as i16*16-1,800];r.waypoint=((if thin{8+i}else{14+i*12})+40*128)as u16;for j in 0..count{r.next[j]=(if i==j{j}else if j>i{i+1}else{i-1})as u8;}}RoadGraph{regions}}
 fn terrain(mode:usize,p:Point)->u8{let(x,y)=p;if(40..=41).contains(&y)&&(8..=100).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0}}
 fn price(code:u8)->u8{match code{7=>2,40=>72,_=>5}}
 #[test]
 #[ignore="requires the private whole contour-to-road raster trace"]
 fn whole_road_replacement_keeps_the_original_raster_after_every_exit(){
  let path=std::env::var("NATIVE_COUPLED_ROAD_RASTER").unwrap();let trace=std::fs::read_to_string(path).unwrap();
  assert_eq!(trace.lines().count(),480);
  let starts=[(160,656),(160,500),(500,160),(1120,656),(960,880)];
  let goals=[(1120,656),(1120,500),(1000,880),(176,672),(170,656),(1200,1300)];
  let graph=graph(7,false);
  for (i,line) in trace.lines().enumerate(){
   let row:serde_json::Value=serde_json::from_str(line).unwrap();let n=|name:&str|row[name].as_i64().unwrap()as usize;
   let actor=starts[n("start")];let goal=goals[n("goal")];let mode=n("mode");let vehicle=n("vehicle")!=0;let suppressed=n("suppressed")!=0;
   let initial=|j:usize|row["initialRay"][j].as_i64().unwrap()as i16;
   let mut s=State::default();s.compare_mode=u8::from(!suppressed);
   for (j,out) in s.frame.iter_mut().enumerate(){let text=row["before"].as_str().unwrap();*out=u8::from_str_radix(&text[j*2..j*2+2],16).unwrap();}
   s.raster=road::raster::Raster{destination:[initial(0),initial(1),initial(2)],position:[initial(3),initial(4),initial(5)],
     delta:[initial(7),initial(8),initial(6)],steps:initial(9),error:[initial(10),initial(11),initial(12)],
     direction:[initial(13),initial(14),initial(15)],special_tiles:initial(16),axis:initial(17)as u8};
   let packed=[(actor.0>>4)+20*128,(goal.0>>4)+20*128,(goal.0>>4)+(goal.1>>4)*128];
   let actual=s.consider(Input{actor,actor_token:0x1000000,vehicle,suppressed,dimensions:(2048,2048),level:if mode==1{5}else{0}},
     &packed,&graph,|p|terrain(mode,p),|c|c==40,price).unwrap();
   assert_eq!(matches!(actual,Decision::Road(_)),n("use")!=0,"choice {i}");
   let r=s.raster;let words=[r.destination[0],r.destination[1],r.destination[2],r.position[0],r.position[1],r.position[2],
     r.delta[2],r.delta[0],r.delta[1],r.steps,r.error[0],r.error[1],r.error[2],r.direction[0],r.direction[1],r.direction[2],r.special_tiles,i16::from(r.axis)];
   assert_eq!(serde_json::json!(words),row["endRay"],"complete route raster {i}");
  }
 }
 #[test]
 #[ignore="requires the original long allocator trace"]
 fn long_road_matches_the_original_allocator_and_shared_tail(){
  let path=std::env::var("NATIVE_LONG_ROAD_ALLOC").unwrap();let trace=std::fs::read_to_string(path).unwrap();
  assert_eq!(trace.lines().count(),160);
  let mut occupied_tail=0;let mut alias_writes=0;
  for (i,line) in trace.lines().enumerate(){
   let row:serde_json::Value=serde_json::from_str(line).unwrap();let n=|name:&str|row[name].as_u64().unwrap();
   let hops=n("hops")as usize;let graph=graph(hops+1,true);let actor=(136,656);let goal=((8+hops as i16)*16+8,656);
   let vehicle=n("vehicle")!=0;let suppressed=n("suppressed")!=0;
   let mut s=State::default();s.compare_mode=u8::from(!suppressed);
   let frame=|name:&str|{let raw=row[name].as_str().unwrap();(0..640).map(|j|u8::from_str_radix(&raw[2*j..2*j+2],16).unwrap()).collect::<Vec<_>>()};
   s.frame.copy_from_slice(&frame("before"));
   let packed=[(actor.0>>4)+20*128,(goal.0>>4)+20*128,(goal.0>>4)+(goal.1>>4)*128];
   let decision=s.consider(Input{actor,actor_token:0x1000000,vehicle,suppressed,dimensions:(2048,2048),level:0},
     &packed,&graph,|p|terrain(0,p),|c|c==40,price);
   let first=n("firstAlias")as u32;
   if first!=0{alias_writes+=1;}
   if !suppressed&&hops>=40{occupied_tail+=1;}
   let decision=decision.unwrap_or_else(|error|panic!("occupied scan {i} stopped: {error:?}"));
   assert_eq!(matches!(decision,Decision::Road(_)),n("use")!=0,"choice {i}");
   if let Decision::Road(points)=decision{
    let native=row["points"].as_array().unwrap().iter().take(39).cloned().collect::<Vec<_>>();
    assert_eq!(serde_json::json!(points),serde_json::json!(native),"adopted route {i}");
   }
   assert_eq!(s.frame.as_slice(),frame("after"),"all physical bytes for case {i}");
   for (actual,key) in [(s.old_price.unwrap_or(-1),"oldCost"),(s.new_price.unwrap_or(-1),"newCost"),
    (i32::from(s.scratch_mode),"scratchMode"),(i32::from(s.compare_mode),"compareMode")]{
    assert_eq!(i64::from(actual),row[key].as_i64().unwrap(),"{key} in case {i}");
   }
  }
  assert!(occupied_tail>0&&alias_writes>0);
 }
 #[test]fn all480_native_warm_scratch_calls_match_full_known_frame_prices_modes_and_record_cursor(){let native=std::env::var("NATIVE_ROAD_SCRATCH_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());let starts=[(160,656),(160,500),(500,160),(1120,656),(960,880)];let goals=[(1120,656),(1120,500),(1000,880),(176,672),(170,656),(1200,1300)];let graph=graph(7,false);let(mut h,mut row)=(2166136261u32,0usize);
 for warm in 0..2{for mode in 0..2{for(si,&actor)in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){for vehicle in[false,true]{for suppressed in[false,true]{let mut s=State::default();if warm!=0{for(i,b)in s.frame.iter_mut().enumerate(){*b=(0x80+i*13)as u8;}}s.frame[..80].fill(0);s.compare_mode=u8::from(!suppressed);let packed=[(actor.0>>4)+20*128,(goal.0>>4)+20*128,(goal.0>>4)+(goal.1>>4)*128];let result=s.consider(Input{actor,actor_token:0x1000000,vehicle,suppressed,dimensions:(2048,2048),level:if mode==1{5}else{0}},&packed,&graph,|p|terrain(mode,p),|c|c==40,price).unwrap();let use_road=matches!(result,Decision::Road(_));if let Some(rs)=&native{let r=&rs[row];let actual=(0..640).map(|i|u8::from_str_radix(&r["after"].as_str().unwrap()[i*2..i*2+2],16).unwrap()).collect::<Vec<_>>();assert_eq!(s.frame.as_slice(),actual,"frame row{row}/w{warm}/m{mode}/s{si}/g{gi}/v{vehicle}/sup{suppressed}");assert_eq!(s.trace.recorded as u8 as u64,r["recorded"].as_u64().unwrap(),"count row{row}");assert_eq!(u64::from(s.record_cursor),r["recordCursor"].as_u64().unwrap());if let Decision::Road(points)=&result{assert_eq!(serde_json::json!(points),r["points"]);}}
  for n in[warm as u32,mode as u32,si as u32,gi as u32,u32::from(vehicle),u32::from(suppressed),u32::from(use_road),s.old_price.unwrap_or(-1)as u32,s.new_price.unwrap_or(-1)as u32,hash(s.frame.as_slice()),s.trace.recorded as u8 as u32,s.record_cursor,u32::from(s.scratch_mode),u32::from(s.compare_mode)]{mix(&mut h,n);}row+=1;
 }}}}}}assert_eq!((row,h),(480,3713235901),"real43df20/43bbe0/43b640/43b6f0/43b950, all640 known bytes, no RNG stubs");}
 #[test]fn suppression_and_below30_preserve_previous_modes_records_and_warm_coordinates(){let g=graph(7,false);for old in[0,29,30,31]{let mut s=State::default();s.frame.fill(0xa5);s.scratch_mode=77;s.compare_mode=88;s.record_cursor=0x12345678;s.trace.recorded=12;let before=s.clone();let input=Input{actor:(160,656),actor_token:0x5e30c8,vehicle:false,suppressed:true,dimensions:(2048,2048),level:0};assert_eq!(s.consider(input,&[11+41*128],&g,|_|7,|_|false,|_|old),Ok(Decision::Contour));assert_eq!(s.frame,before.frame);assert_eq!((s.scratch_mode,s.compare_mode,s.record_cursor),(77,88,0x12345678));assert!(s.old_price.is_none());let mut input=input;input.suppressed=false;assert_eq!(s.consider(input,&[11+41*128],&g,|_|7,|_|false,|_|old),Ok(Decision::Contour));assert_eq!(s.old_price,Some(i32::from(old)));if old<30{assert_eq!(s.frame,before.frame);assert_eq!((s.scratch_mode,s.compare_mode),(77,88));}else{assert_eq!((s.scratch_mode,s.compare_mode),(0,0));}}}
 #[test]fn road_copies_at_most39_points_even_when_the_list_extends_into_the_shared_tail(){
  let start=(136,500);
  for (hops,context_alias) in [(38,false),(39,false),(39,true)]{
   let mut s=State::default();
   s.dword_set(46*8+4,FREE);
   if context_alias{s.dword_set(44*8+4,FREE);}
   let goal_x=(8+hops as i16)*16+8;
   let packed=[8+20*128,(goal_x>>4)+20*128,(goal_x>>4)+31*128];
   let result=s.consider(Input{actor:start,actor_token:0x5e30c8,vehicle:false,suppressed:false,dimensions:(2048,2048),level:0},
    &packed,&graph(hops+1,true),|p|terrain(0,p),|_|false,price);
   let Decision::Road(points)=result.unwrap()else{panic!()};
   assert_eq!(points.len(),39);
   assert_eq!(points.last(),Some(&[751,655]));
   if hops==39{assert_eq!(s.nodes()[39].next,BASE+if context_alias{44}else{46}*8);}
  }
 }
 #[test]fn contour_buffer_boundary_preserves_known_point_writes_without_pricing_or_mode_reset(){let mut s=State::default();s.scratch_mode=9;s.compare_mode=7;let result=s.consider(Input{actor:(160,656),actor_token:0x5e30c8,vehicle:false,suppressed:false,dimensions:(2048,2048),level:0},&[11+41*128;40],&graph(7,false),|_|7,|_|false,price);assert_eq!(result,Err(Boundary::ContourBuffer{count:40,address:0x4d1780+41*4}));assert_eq!(s.pairs[0],[160,656]);assert_eq!(s.pairs[40],[184,662]);assert_eq!((s.scratch_mode,s.compare_mode),(9,7));assert!(s.old_price.is_none());}
 #[test]fn word_origin_updates_preserve_high_word_and_pool_reset_keeps_process_scratch(){let mut paths=crate::original_route_pool::State::default();paths.road_scratch.frame.fill(0xa5);paths.road_scratch.synchronize_context(-1,0x4d15f4,false);assert_eq!(paths.road_scratch.dword(356),0xa5a5ffff);let old=paths.road_scratch.frame.clone();paths.pool.nodes[0]=Node{point:[42,71],next:0};paths.managed[7]=true;paths.reset_for_mission();assert_eq!(paths.road_scratch.frame,old);assert!(!paths.managed[7]);assert_eq!(paths.pool.nodes[0],Node::default());}
 #[test]fn last_candidate_removal_aliases_only_the_warm_price_x_even_when_road_suppressed(){let mut s=State::default();s.pairs[0]=[1234,-2000];s.synchronize_candidate(3,7,&[123],true);assert_eq!(s.pairs[0],[-1,-2000]);let result=s.consider(Input{actor:(160,656),actor_token:0x5e30c8,vehicle:false,suppressed:true,dimensions:(2048,2048),level:0},&[11+41*128],&graph(7,false),|_|7,|_|false,price);assert_eq!(result,Ok(Decision::Contour));assert_eq!(s.pairs[0],[-1,-2000]);}

}
