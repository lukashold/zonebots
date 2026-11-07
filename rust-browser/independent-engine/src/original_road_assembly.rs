//! 43bbe0's finite road-list construction. 43b640 starts at the current
//! record, can use primary slot4800, and treats the80-record scratch pool
//! differently. A failed final append is deliberately not the same branch
//! as a failed intermediate append in the original caller.
use super::{Node,FREE,PRIMARY};
use crate::original_route::{self as road,Point,RoadGraph};
// Scratch records 40..43 alias the endpoint distances; records 44 and 45
// alias the caller context. These all use the same physical scratch bytes.
const SCRATCH:u32=0x004d1480;
pub(crate) const SCRATCH_NODES:usize=80;
pub(crate) const SCRATCH_CLEARED:usize=40;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{Node(u32),Region(u8)}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct ResultState{pub failed:bool,pub set_road_flag:bool,pub recorded:i8}
#[derive(Clone,Debug)]pub struct Trace{pub records:[u32;80],pub recorded:i8,pub record_cursor:usize,pub endpoint_distances:Option<[i32;8]>}
impl Default for Trace{fn default()->Self{Self{records:[0;80],recorded:0,record_cursor:0,endpoint_distances:None}}}
struct Builder<'a>{nodes:&'a mut[Node],base:u32,temporary:bool,head:u32,cursor:u32,records:[u32;80],count:i8,record_cursor:usize,endpoint_distances:Option<[i32;8]>}
impl Builder<'_>{
 fn index(&self,link:u32)->Result<usize,Boundary>{let n=link.wrapping_sub(self.base);if n&7!=0||n/8>=self.nodes.len()as u32{return Err(Boundary::Node(link));}Ok((n/8)as usize)}
 fn node(&self,link:u32)->Result<Node,Boundary>{Ok(self.nodes[self.index(link)?])}
 fn node_mut(&mut self,link:u32)->Result<&mut Node,Boundary>{let n=self.index(link)?;Ok(&mut self.nodes[n])}
 fn record_endpoint_distances(&mut self,distances:[i32;8]){
  self.endpoint_distances=Some(distances);
  if self.temporary{
   for (i,node) in self.nodes[SCRATCH_CLEARED..SCRATCH_CLEARED+4].iter_mut().enumerate(){
    let point=distances[2*i].to_le_bytes();
    node.point=[i16::from_le_bytes([point[0],point[1]]),i16::from_le_bytes([point[2],point[3]])];
    node.next=distances[2*i+1]as u32;
   }
  }
 }
 ///True is native AL=1. The new record and array pointer writes precede
 ///the80-entry overflow return, so failure is not an atomic operation.
 fn append(&mut self,p:Point)->Result<bool,Boundary>{
  let mut i=self.index(self.cursor)?;
  while self.nodes.get(i).ok_or(Boundary::Node(self.base.wrapping_add(i as u32*8)))?.next!=FREE{
   i+=1;
   if (i as i16)>4800||self.temporary&&i>=SCRATCH_NODES{return Ok(true);}
  }
  let address=self.base.wrapping_add(i as u32*8);
  let Some(node)=self.nodes.get_mut(i)else{return Err(Boundary::Node(address));};*node=Node{point:[p.0,p.1],next:0};
  if address!=self.cursor{self.node_mut(self.cursor)?.next=address;let count=self.count as u8 as usize;if count>=80{return Err(Boundary::Node(address));}self.records[count]=self.cursor;self.record_cursor+=1;self.count=self.count.wrapping_add(1);self.cursor=address;if self.count as u8>=80{return Ok(true);}}
  Ok(false)
 }
 fn points(&self)->Result<Vec<Point>,Boundary>{let mut out=Vec::new();let mut link=self.head;for _ in 0..self.nodes.len(){if link==0{return Ok(out);}let n=self.node(link)?;out.push((n.point[0],n.point[1]));link=n.next;}Err(Boundary::Node(link))}
 ///43bba0 frees only the recorded prefix. The last current record is
 ///not in that array, and can remain an allocated orphan on failure.
 fn fallback(&mut self,goal:Point)->Result<ResultState,Boundary>{
  for i in 0..self.count.max(0)as usize{let link=self.records[i];*self.node_mut(link)?=Node::default();self.records[i]=0;}
  *self.node_mut(self.head)?=Node{point:[goal.0,goal.1],next:0};Ok(ResultState{failed:true,recorded:self.count,..Default::default()})
 }
 ///43b840 only compacts its pointer array. It does not unlink the
 ///duplicate nodes from the actual route traversed by43ba60.
 fn compact_records(&mut self)->Result<(),Boundary>{
  if self.count<=1{return Ok(());}for p in &mut self.records[self.count as usize..]{*p=0;}
  let mut at=0usize;
  loop{let anchor=self.node(self.records[at])?.point;let mut removed=0i8;
   for j in at+1..80{let link=self.records[j];if link==0{break;}let n=self.node(link)?;if n.next!=FREE&&n.point==anchor{self.records[j]=0;removed=removed.wrapping_add(1);}}
   if removed!=0{let tail=self.records[at+1..].iter().copied().filter(|&n|n!=0).collect::<Vec<_>>();self.records[at+1..].fill(0);self.records[at+1..at+1+tail.len()].copy_from_slice(&tail);self.count=self.count.wrapping_sub(removed);}
   at+=1;if removed==0{break;}let next=*self.records.get(at).ok_or(Boundary::Node(0))?;if self.node(next)?.next==FREE{break;}
  }Ok(())
 }
 fn choose_direct(&mut self,goal:Point)->Result<(),Boundary>{
  let mut p=self.head;for _ in 0..self.nodes.len(){if p==0{self.cursor=self.head;let _=self.append(goal)?;return Ok(());}let next=self.node(p)?.next;self.node_mut(p)?.next=FREE;p=next;}Err(Boundary::Node(p))
 }
}
pub(crate) fn primary(nodes:&mut[Node],head:u32,start:Point,graph:&RoadGraph,dimensions:Point,terrain:impl FnMut(Point)->u8,flagged:impl FnMut(u8)->bool,price:impl FnMut(u8)->u8,level:i16,compare:bool,trace:&mut Trace,raster:&mut road::raster::Raster)->Result<ResultState,Boundary>{
 assemble_traced(nodes,PRIMARY,false,head,start,graph,dimensions,terrain,flagged,price,level,compare,trace,raster)
}
///43df20 clears forty slots and lets the allocator reuse occupied global
///storage after that prefix when the next word reads as FREE.
pub(crate) fn scratch(nodes:&mut[Node;SCRATCH_NODES],start:Point,graph:&RoadGraph,dimensions:Point,terrain:impl FnMut(Point)->u8,flagged:impl FnMut(u8)->bool,price:impl FnMut(u8)->u8,level:i16,trace:&mut Trace,raster:&mut road::raster::Raster)->Result<ResultState,Boundary>{assemble_traced(nodes,SCRATCH,true,SCRATCH,start,graph,dimensions,terrain,flagged,price,level,false,trace,raster)}
fn assemble_traced(nodes:&mut[Node],base:u32,temporary:bool,head:u32,start:Point,graph:&RoadGraph,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8,level:i16,compare:bool,trace:&mut Trace,raster:&mut road::raster::Raster)->Result<ResultState,Boundary>{
 let mut b=Builder{nodes,base,temporary,head,cursor:head,records:trace.records,count:0,record_cursor:0,endpoint_distances:None};
 let result=assemble_inner(&mut b,start,graph,dimensions,&mut terrain,&mut flagged,&mut price,level,compare,raster);
 trace.records=b.records;trace.recorded=b.count;trace.record_cursor=b.record_cursor;trace.endpoint_distances=b.endpoint_distances;result
}
fn assemble_inner(b:&mut Builder<'_>,start:Point,graph:&RoadGraph,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8,level:i16,compare:bool,raster:&mut road::raster::Raster)->Result<ResultState,Boundary>{
 let head=b.head;let p=b.node(head)?.point;let goal=(p[0],p[1]);
 let dx=i32::from(goal.0)-i32::from(start.0);let dy=i32::from(goal.1)-i32::from(start.1);
 if dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))<0x3840{return b.fallback(goal);}
 let searched=road::road_endpoint_trace(goal,dimensions,&mut terrain,&mut flagged);b.record_endpoint_distances(searched.distances);
 raster.road_endpoint(goal,searched.endpoint.is_some_and(|e|e.already_on_road));
 let Some(end)=searched.endpoint else{return b.fallback(goal);};
 let searched=road::road_endpoint_trace(start,dimensions,&mut terrain,&mut flagged);b.record_endpoint_distances(searched.distances);
 raster.road_endpoint(start,searched.endpoint.is_some_and(|e|e.already_on_road));
 let Some(begin)=searched.endpoint else{return b.fallback(goal);};
 let Some(to)=graph.region_at(end.point)else{return b.fallback(goal);};let Some(from)=graph.region_at(begin.point)else{return b.fallback(goal);};
 if from==to||graph.regions.get(usize::from(to)).and_then(|r|r.next.get(usize::from(from)))==Some(&255){return b.fallback(goal);}
 b.node_mut(head)?.next=FREE;
 if !begin.already_on_road{let _=b.append(begin.point)?;}
 let mut current=from;
 while current!=to{let region=graph.regions.get(usize::from(current)).ok_or(Boundary::Region(current))?;let &next=region.next.get(usize::from(to)).ok_or(Boundary::Region(to))?;if next==255{return b.fallback(goal);}if next==to{break;}let region=graph.regions.get(usize::from(next)).ok_or(Boundary::Region(next))?;if b.append(road::road_center(region.waypoint))?{return b.fallback(goal);}current=next;}
 if !end.already_on_road&&b.append(end.point)?{return b.fallback(goal);}
 //43bd7b ignores AL from the final goal append, including a full pool.
 let _=b.append(goal)?;b.compact_records()?;
 if b.count>5{
   let point=|back|b.record_cursor.checked_sub(back).and_then(|i|b.records.get(i)).copied().ok_or(Boundary::Node(0)).and_then(|p|b.node(p)).map(|n|(n.point[0],n.point[1]));
   raster.road_compaction(point(4)?,point(2)?);
 }
 let mut keep=false;if compare{
  let points=b.points()?;let Some(&last)=points.last()else{return Err(Boundary::Node(head));};
  let mut full=vec![start];full.extend_from_slice(&points);
  let via=raster.price(&full,&mut terrain,&mut price);
  raster.special_tiles=0; // 43ba60 resets the global only before the direct line.
  let direct=raster.price(&[start,last],&mut terrain,&mut price);
  keep=!road::prefer_direct(via,direct,level);if !keep{b.choose_direct(last)?;}
 }
 Ok(ResultState{failed:false,set_road_flag:keep,recorded:b.count})
}

#[cfg(test)]mod tests{
 use super::*;use crate::original_route::RoadRegion;
 fn mix(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}
 #[test]fn complete_native_road_assembly_matches_fragmented_primary_pools(){
  let starts=[(160,656),(160,500),(1120,656),(500,160)];let goals=[(1120,656),(1120,500),(176,672),(1200,1300)];let mut all=2166136261;let mut raster_hash=2166136261;
  let evidence=std::env::var_os("NATIVE_PRIMARY_ROAD_RASTER").map(|p|std::fs::read_to_string(p).unwrap());
  let mut rows=evidence.as_deref().map(str::lines);
  for c in 0..288{let mode=c%3;let layout=c/3%12;let compare=c/36%2!=0;let pair=c/72;let temporary=layout>=8;if temporary{continue;}let count=if temporary{80}else{14400};let head_index=[0,0,0,4796,4798,4799,4800,0,0,76,78,79][layout];let base=if temporary{SCRATCH}else{PRIMARY};let head=base+head_index as u32*8;let long_road=mode==2&&!temporary;let start=if long_road{(160,656)}else{starts[pair]};let goal=if long_road{(1650,656)}else{goals[pair]};
   let mut nodes=(0..count).map(|i|Node{point:[100+(i%200)as i16,200+(i%300)as i16],next:if layout==1&&i%3!=0||layout==2&&i>2||layout==7&&i!=0&&i!=4800{0}else{FREE}}).collect::<Vec<_>>();nodes[head_index]=Node{point:[goal.0,goal.1],next:0};
   let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];let n=if long_road{96}else{7};for(i,r)in regions.iter_mut().take(n).enumerate(){let left=if long_road{8+i}else{8+i*12};let right=if long_road{9+i}else{20+i*12};r.bounds=[left as i16*16,512,right as i16*16-1,800];r.waypoint=((if long_road{8+i}else{14+i*12})+40*128)as u16;for j in 0..n{r.next[j]=if i==j{j}else if j>i{i+1}else{i-1}as u8;}}
   let graph=RoadGraph{regions};let terrain=|(x,y)|if(40..=41).contains(&y)&&(8..=110).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0};
   let mut trace=Trace::default();let b=0x3b00+(c%13)as i16*64;
   let mut r=road::raster::Raster{destination:[b,b+1,b+2],position:[b+3,b+4,b+5],delta:[b+7,b+8,b+6],steps:b+9,error:[b+10,b+11,b+12],direction:[b+13,b+14,b+15],special_tiles:b+16,axis:9};
   let result=primary(&mut nodes,head,start,&graph,(2048,2048),terrain,|code|code==40,|code|match code{7=>2,40=>72,_=>5},if mode==1{5}else{0},compare,&mut trace,&mut r).unwrap();
   let raster=[r.destination[0],r.destination[1],r.destination[2],r.position[0],r.position[1],r.position[2],r.delta[2],r.delta[0],r.delta[1],r.steps,r.error[0],r.error[1],r.error[2],r.direction[0],r.direction[1],r.direction[2],r.special_tiles,i16::from(r.axis)];
   mix(&mut raster_hash,c as u32);for n in raster{mix(&mut raster_hash,n as u32);}
   let mut pool=2166136261;for node in &nodes{for byte in node.point[0].to_le_bytes().into_iter().chain(node.point[1].to_le_bytes()).chain(node.next.to_le_bytes()){mix(&mut pool,u32::from(byte));}}
   let mut links=2166136261;let mut link=head;let mut length=0;while link!=0&&link!=FREE&&length<200{mix(&mut links,link);length+=1;link=nodes[((link-base)/8)as usize].next;}
   let fields=[c as u32,u32::from(result.failed),result.recorded as u8 as u32,3|if result.set_road_flag{128}else{0},pool,links,length];
   if let Some(rows)=&mut rows{
    let native:serde_json::Value=serde_json::from_str(rows.next().unwrap()).unwrap();
    assert_eq!(native["raster"],serde_json::json!(raster),"raster case {c}");
    assert_eq!(fields,["case","failed","recorded","flags","pool","links","length"].map(|key|native[key].as_u64().unwrap() as u32),"route/pool case {c}");
   }
   if std::env::var_os("Z_ROAD_TRACE").is_some(){println!("ROAD {fields:?}");}for n in fields{mix(&mut all,n);}
  }
  assert_eq!(all,1813887958,"192 complete43bbe0 native finite-primary-pool traces");
  assert_eq!(raster_hash,1699589134,"the same 192 original whole-call warm raster states");
  assert!(rows.is_none_or(|mut r|r.next().is_none()));
 }
}
