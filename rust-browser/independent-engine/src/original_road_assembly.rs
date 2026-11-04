//! 43bbe0's finite road-list construction. 43b640 starts at the current
//! record, can use primary slot4800, and treats the80-record scratch pool
//! differently. A failed final append is deliberately not the same branch
//! as a failed intermediate append in the original caller.
use super::{Node,FREE,PRIMARY};
use crate::original_route::{self as road,Point,RoadGraph};
// Scratch records overlap native global storage from slot44 onward. They are
// not a runtime input here; full scratch/global aliasing remains separate.
const SCRATCH:u32=0x004d1480;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{Node(u32),Region(u8)}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct ResultState{pub failed:bool,pub set_road_flag:bool,pub recorded:i8}
struct Builder<'a>{nodes:&'a mut[Node],base:u32,temporary:bool,head:u32,cursor:u32,records:[u32;80],count:i8}
impl Builder<'_>{
 fn index(&self,link:u32)->Result<usize,Boundary>{let n=link.wrapping_sub(self.base);if n&7!=0||n/8>=self.nodes.len()as u32{return Err(Boundary::Node(link));}Ok((n/8)as usize)}
 fn node(&self,link:u32)->Result<Node,Boundary>{Ok(self.nodes[self.index(link)?])}
 fn node_mut(&mut self,link:u32)->Result<&mut Node,Boundary>{let n=self.index(link)?;Ok(&mut self.nodes[n])}
 ///True is native AL=1. The new record and array pointer writes precede
 ///the80-entry overflow return, so failure is not an atomic operation.
 fn append(&mut self,p:Point)->Result<bool,Boundary>{
  let mut i=self.index(self.cursor)?;
  while self.nodes[i].next!=FREE{i+=1;if (i as i16)>4800||self.temporary&&i>=80{return Ok(true);}if i>=self.nodes.len(){return Err(Boundary::Node(self.base.wrapping_add(i as u32*8)));}}
  let address=self.base.wrapping_add(i as u32*8);self.nodes[i]=Node{point:[p.0,p.1],next:0};
  if address!=self.cursor{self.node_mut(self.cursor)?.next=address;let count=self.count as u8 as usize;if count>=80{return Err(Boundary::Node(address));}self.records[count]=self.cursor;self.count=self.count.wrapping_add(1);self.cursor=address;if self.count as u8>=80{return Ok(true);}}
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
pub fn primary(nodes:&mut[Node],head:u32,start:Point,graph:&RoadGraph,dimensions:Point,terrain:impl FnMut(Point)->u8,flagged:impl FnMut(u8)->bool,price:impl FnMut(u8)->u8,level:i16,compare:bool)->Result<ResultState,Boundary>{assemble(nodes,PRIMARY,false,head,start,graph,dimensions,terrain,flagged,price,level,compare)}
fn assemble(nodes:&mut[Node],base:u32,temporary:bool,head:u32,start:Point,graph:&RoadGraph,dimensions:Point,mut terrain:impl FnMut(Point)->u8,mut flagged:impl FnMut(u8)->bool,mut price:impl FnMut(u8)->u8,level:i16,compare:bool)->Result<ResultState,Boundary>{
 let mut b=Builder{nodes,base,temporary,head,cursor:head,records:[0;80],count:0};let p=b.node(head)?.point;let goal=(p[0],p[1]);
 let dx=i32::from(goal.0)-i32::from(start.0);let dy=i32::from(goal.1)-i32::from(start.1);
 if dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))<0x3840{return b.fallback(goal);}
 let Some(end)=road::road_endpoint(goal,dimensions,&mut terrain,&mut flagged)else{return b.fallback(goal);};
 let Some(begin)=road::road_endpoint(start,dimensions,&mut terrain,&mut flagged)else{return b.fallback(goal);};
 let Some(to)=graph.region_at(end.point)else{return b.fallback(goal);};let Some(from)=graph.region_at(begin.point)else{return b.fallback(goal);};
 if from==to||graph.regions.get(usize::from(to)).and_then(|r|r.next.get(usize::from(from)))==Some(&255){return b.fallback(goal);}
 b.node_mut(head)?.next=FREE;
 if !begin.already_on_road{let _=b.append(begin.point)?;}
 let mut current=from;
 while current!=to{let region=graph.regions.get(usize::from(current)).ok_or(Boundary::Region(current))?;let &next=region.next.get(usize::from(to)).ok_or(Boundary::Region(to))?;if next==255{return b.fallback(goal);}if next==to{break;}let region=graph.regions.get(usize::from(next)).ok_or(Boundary::Region(next))?;if b.append(road::road_center(region.waypoint))?{return b.fallback(goal);}current=next;}
 if !end.already_on_road&&b.append(end.point)?{return b.fallback(goal);}
 //43bd7b ignores AL from the final goal append, including a full pool.
 let _=b.append(goal)?;b.compact_records()?;
 let mut keep=false;if compare{let points=b.points()?;let Some(&last)=points.last()else{return Err(Boundary::Node(head));};let mut full=vec![start];full.extend_from_slice(&points);let via=road::path_price(&full,&mut terrain,&mut price);let direct=road::path_price(&[start,last],&mut terrain,&mut price);keep=!road::prefer_direct(via,direct,level);if !keep{b.choose_direct(last)?;}}
 Ok(ResultState{failed:false,set_road_flag:keep,recorded:b.count})
}

#[cfg(test)]mod tests{
 use super::*;use crate::original_route::RoadRegion;
 fn mix(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}
 #[test]fn complete_native_road_assembly_matches_fragmented_primary_pools(){
  let starts=[(160,656),(160,500),(1120,656),(500,160)];let goals=[(1120,656),(1120,500),(176,672),(1200,1300)];let mut all=2166136261;
  for c in 0..288{let mode=c%3;let layout=c/3%12;let compare=c/36%2!=0;let pair=c/72;let temporary=layout>=8;if temporary{continue;}let count=if temporary{80}else{14400};let head_index=[0,0,0,4796,4798,4799,4800,0,0,76,78,79][layout];let base=if temporary{SCRATCH}else{PRIMARY};let head=base+head_index as u32*8;let long_road=mode==2&&!temporary;let start=if long_road{(160,656)}else{starts[pair]};let goal=if long_road{(1650,656)}else{goals[pair]};
   let mut nodes=(0..count).map(|i|Node{point:[100+(i%200)as i16,200+(i%300)as i16],next:if layout==1&&i%3!=0||layout==2&&i>2||layout==7&&i!=0&&i!=4800{0}else{FREE}}).collect::<Vec<_>>();nodes[head_index]=Node{point:[goal.0,goal.1],next:0};
   let mut regions=vec![RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96];let n=if long_road{96}else{7};for(i,r)in regions.iter_mut().take(n).enumerate(){let left=if long_road{8+i}else{8+i*12};let right=if long_road{9+i}else{20+i*12};r.bounds=[left as i16*16,512,right as i16*16-1,800];r.waypoint=((if long_road{8+i}else{14+i*12})+40*128)as u16;for j in 0..n{r.next[j]=if i==j{j}else if j>i{i+1}else{i-1}as u8;}}
   let graph=RoadGraph{regions};let terrain=|(x,y)|if(40..=41).contains(&y)&&(8..=110).contains(&x){7}else if mode==1&&(40..=42).contains(&x){40}else{0};let result=assemble(&mut nodes,base,temporary,head,start,&graph,(2048,2048),terrain,|code|code==40,|code|match code{7=>2,40=>72,_=>5},if mode==1{5}else{0},compare).unwrap();
   let mut pool=2166136261;for node in &nodes{for byte in node.point[0].to_le_bytes().into_iter().chain(node.point[1].to_le_bytes()).chain(node.next.to_le_bytes()){mix(&mut pool,u32::from(byte));}}
   let mut links=2166136261;let mut link=head;let mut length=0;while link!=0&&link!=FREE&&length<200{mix(&mut links,link);length+=1;link=nodes[((link-base)/8)as usize].next;}
   let fields=[c as u32,u32::from(result.failed),result.recorded as u8 as u32,3|if result.set_road_flag{128}else{0},pool,links,length];if std::env::var_os("Z_ROAD_TRACE").is_some(){println!("ROAD {fields:?}");}for n in fields{mix(&mut all,n);}
  }
  assert_eq!(all,1813887958,"192 complete43bbe0 native finite-primary-pool traces");
 }
}
