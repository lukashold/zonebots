//! Finite native route storage and43e780's round-robin dispatcher. Raw links
//! are numeric handles into the two contiguous original arrays, never Rust
//! addresses. The source's different scan/cleanup bounds remain explicit.
pub const PRIMARY:u32=0x005eee28;
pub const SECONDARY:u32=0x005f8428;
pub const PRIMARY_COUNT:usize=4800;
pub const SECONDARY_COUNT:usize=9600;
pub const FREE:u32=u32::MAX;
#[path="original_route_pool_game.rs"]pub mod game;
#[path="original_road_assembly.rs"]pub mod road;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Node{pub point:[i16;2],pub next:u32}
impl Default for Node{fn default()->Self{Self{point:[-1;2],next:FREE}}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Links{pub primary:u32,pub secondary:u32}
#[derive(Clone,Debug)]pub struct Pool{pub nodes:Vec<Node>,pub links:[Links;240]}
impl Default for Pool{fn default()->Self{Self{nodes:vec![Node::default();PRIMARY_COUNT+SECONDARY_COUNT],links:[Links::default();240]}}}
impl Pool{
 pub fn reset_for_mission(&mut self){self.nodes.fill(Node::default());self.links.fill(Links::default());}
 pub fn address(index:usize)->u32{PRIMARY.wrapping_add(index as u32*8)}
 pub fn index(link:u32)->Option<usize>{let delta=link.wrapping_sub(PRIMARY);(delta&7==0&&delta/8<(PRIMARY_COUNT+SECONDARY_COUNT)as u32).then_some((delta/8)as usize)}
 pub fn node(&self,link:u32)->Option<Node>{self.nodes.get(Self::index(link)?).copied()}
 pub fn node_mut(&mut self,link:u32)->Option<&mut Node>{let index=Self::index(link)?;self.nodes.get_mut(index)}
 pub fn first_free(&self,start:usize,count:usize)->Option<usize>{(start..start.saturating_add(count).min(self.nodes.len())).find(|&i|self.nodes[i].next==FREE)}
 ///44d1a0 rejects another active UNIT's head, including the inclusive
 ///source boundary that can name secondary slot0 as primary slot4800.
 pub fn assign_primary(&mut self,slot:usize,link:u32,active:&[bool;240])->u32{
  let rejected=link!=0&&link!=FREE&&(link.wrapping_sub(PRIMARY)>>3>4800||(0..240).any(|s|s!=slot&&active[s]&&self.links[s].primary==link));
  let link=if rejected{0}else{link};self.links[slot].primary=link;link
 }
 fn retire(&mut self,mut link:u32,base:u32,bound:u32){
  while link!=0&&link.wrapping_sub(base)>>3<bound{
   let Some(node)=self.node_mut(link)else{break;};let next=node.next;node.next=FREE;link=next;
  }
 }
 ///52470 and524b0 differ in ownership flags/cancellation; both retire
 ///the same linked list without clearing its stale coordinate words.
 pub fn clear_secondary(&mut self,slot:usize){let link=self.links[slot].secondary;self.retire(link,SECONDARY,0x4b01);self.links[slot].secondary=0;}
 ///52500's inactive leader branch deliberately leaves the primary intact.
 ///The bool reports whether the ordinary branch also clears hardware17:1.
 pub fn clear(&mut self,slot:usize,active:bool,role:i8)->bool{
  if !active&&role== -1{self.clear_secondary(slot);return false;}
  let link=self.links[slot].primary;self.retire(link,PRIMARY,0x2581);self.links[slot].primary=0;self.clear_secondary(slot);true
 }
 ///59c60 searches only the first240 primary records, BEFORE cleanup.
 pub fn direct(&mut self,slot:usize,point:[i16;2],replace:bool,active:&[bool;240],role:i8)->Option<u32>{
  let found=self.first_free(0,240)?;self.nodes[found].point=point;
  if replace{self.clear(slot,active[slot],role);self.nodes[found].point=point;self.nodes[found].next=0;}else{self.clear_secondary(slot);self.nodes[found].next=self.links[slot].primary;}
  Some(self.assign_primary(slot,Self::address(found),active))
 }
 ///53730/490b0's first goal allocation scans the full4800-slot array.
 pub fn allocate_goal(&mut self,point:[i16;2])->Option<u32>{let i=self.first_free(0,PRIMARY_COUNT)?;self.nodes[i]=Node{point,next:0};Some(Self::address(i))}
 ///43deb0 scans9600 contiguous slots, extending into the secondary array.
 ///On exhaustion the last partial record keeps its old FREE link.
 pub fn copy_replacement(&mut self,slot:usize,points:&[[i16;2]])->bool{let mut cursor=0;let mut previous=None;for(i,&point)in points.iter().enumerate(){let Some(index)=self.first_free(cursor,9600-cursor)else{return false;};let link=Self::address(index);if let Some(p)=previous{self.node_mut(p).unwrap().next=link;}else{self.links[slot].primary=link;}self.nodes[index].point=point;if i+1==points.len(){self.nodes[index].next=0;}previous=Some(link);cursor=index+1;}true}
 ///3e190's completed contour list: low-to-high9600 slots; partial
 ///allocation is retained on exhaustion just as in the native loop.
 pub fn contour(&mut self,slot:usize,points:&[[i16;2]])->bool{
  let mut previous:Option<usize>=None;let mut first=0;let mut cursor=PRIMARY_COUNT;
  for &point in points{
   let Some(i)=self.first_free(cursor,PRIMARY_COUNT+SECONDARY_COUNT-cursor)else{return false;};
   let address=Self::address(i);if let Some(p)=previous{self.nodes[p].next=address;}else{first=address;}
   self.nodes[i].point=point;previous=Some(i);cursor=i+1;
  }
  if let Some(p)=previous{self.nodes[p].next=0;self.links[slot].secondary=first;}true
 }
 pub fn points(&self,mut link:u32)->Vec<[i16;2]>{let mut points=vec![];for _ in 0..self.nodes.len(){if link==0||link==FREE{break;}let Some(node)=self.node(link)else{break;};points.push(node.point);link=node.next;}points}
 pub fn consume(&mut self,slot:usize,secondary:bool)->Option<Node>{let link=if secondary{self.links[slot].secondary}else{self.links[slot].primary};let node=self.node(link)?;self.node_mut(link)?.next=FREE;if secondary{self.links[slot].secondary=node.next;}else{self.links[slot].primary=node.next;}Some(node)}
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub enum Function{#[default]Idle,Prepare,Vehicle}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Scheduler{pub current:u8,pub function:Function,pub phase:i16,pub side_grace:u8}
#[derive(Clone,Copy,Debug)]pub struct Actor{pub flags31:u8,pub group:u8,pub family:u8,pub class:u8,pub permissions:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Dispatch{None,Selected(u8),Invalid{slot:u8,group:u8},Run{slot:u8,group:u8,function:Function,permissions:[Option<u8>;5]}}
impl Scheduler{
 pub fn cancel(&mut self,slot:usize){if usize::from(self.current)==slot{self.function=Function::Idle;}}
 ///43e780: idle callbacks select only; computation starts on the next
 ///callback. Selection checks31:40 before the later active/pointer gate.
 pub fn dispatch(&mut self,actors:&[Actor;240])->Dispatch{
  if self.function==Function::Idle{
   for _ in 0..240{self.current=if self.current==239{0}else{self.current+1};if actors[usize::from(self.current)].flags31&64!=0{self.function=Function::Prepare;self.phase=0;self.side_grace=127;return Dispatch::Selected(self.current);}}
   return Dispatch::None;
  }
  let actor=actors[usize::from(self.current)];if !actor.permissions||actor.flags31&4==0{self.function=Function::Idle;return Dispatch::Invalid{slot:self.current,group:actor.group};}
  let mut permissions=[Some(255),Some(255),None,None,Some(255)];permissions[usize::from(actor.flags31&3)]=Some(0);
  //Only teams0/1 overwrite bytes24/25. Team2/3 writes the next raw byte.
  //byte28 is reset toFF after every call by the wrapper.
  permissions[4]=Some(if !matches!(actor.family,34|32|36|38)&&matches!(actor.class,1|3){0}else{255});
  Dispatch::Run{slot:self.current,group:actor.group,function:self.function,permissions}
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Completion{Done,RetryVehicle}
pub trait WorkWorld{
 fn initialize_origin(&mut self);
 fn clear_mask(&mut self,start:usize,length:usize);
 fn reset_candidates(&mut self);
 ///43cb20's return: true means its ray could not be prepared.
 fn target_ray(&mut self)->bool;
 ///Native seed functions return true when there is no contour to run.
 fn seed(&mut self,direction:usize,vehicle:bool)->bool;
 fn walk_pending(&mut self,direction:usize,vehicle:bool)->bool;
 fn finish(&mut self)->Completion;
 fn abort(&mut self);
 fn clear_unit_counter67(&mut self);
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Worker{pub mask_cursor:usize,pub countdown:i8}
#[derive(Clone,Debug)]pub struct State{
 pub pool:Pool,pub managed:[bool;240],pub scheduler:Scheduler,pub worker:Worker,
 pub mask:Vec<u8>,pub candidates:[Option<crate::original_route::local::Contour>;4],
 pub headers:[(u8,i8);4],
 pub origin:i16,pub steps:u32,pub contact:[[i16;2];240],
}
impl Default for State{fn default()->Self{Self{pool:Default::default(),managed:[false;240],scheduler:Default::default(),worker:Default::default(),mask:vec![65;16384],candidates:std::array::from_fn(|_|None),headers:[(0,0);4],origin:0,steps:0,contact:[[-1;2];240]}}}
impl State{
 pub fn vacant()->Self{Self{pool:Pool{nodes:vec![],links:[Links::default();240]},managed:[false;240],scheduler:Default::default(),worker:Default::default(),mask:vec![],candidates:std::array::from_fn(|_|None),headers:[(0,0);4],origin:0,steps:0,contact:[[-1;2];240]}}
 pub fn reset_for_mission(&mut self){self.pool.reset_for_mission();self.managed.fill(false);self.scheduler=Default::default();self.worker=Default::default();self.contact.fill([-1;2]);}
}
impl Worker{
 fn clear(&mut self,world:&mut impl WorkWorld,count:usize){for _ in 0..count{world.clear_mask(self.mask_cursor,2048);self.mask_cursor+=2048;}}
 ///One real43e4a0/43dad0 callback. Failed seeds fall through to the next
 ///direction in the same callback; successful seeds and pending walkers yield.
 pub fn advance(&mut self,scheduler:&mut Scheduler,vehicle:bool,world:&mut impl WorkWorld){
  if scheduler.function==Function::Idle{return;}
  if scheduler.function==Function::Prepare{
   match scheduler.phase{
    0=>{world.initialize_origin();self.mask_cursor=0;self.clear(world,3);scheduler.phase=1;return;},
    1=>{self.clear(world,3);scheduler.phase=2;return;},
    2=>{self.clear(world,2);world.reset_candidates();scheduler.phase=3;return;},
    3=>{if vehicle{scheduler.function=Function::Vehicle;scheduler.phase=0;return;}world.target_ray();scheduler.phase=4;return;},
    4..=12=>{},_=>return,
   }
   loop{
    if scheduler.phase==12{world.finish();world.clear_unit_counter67();scheduler.function=Function::Idle;return;}
    let direction=((scheduler.phase-4)/2)as usize;
    if scheduler.phase&1==0{if !world.seed(direction,false){self.countdown=20;scheduler.phase+=1;return;}scheduler.phase+=2;}
    else{self.countdown=self.countdown.wrapping_sub(1);if self.countdown!=0&&world.walk_pending(direction,false){return;}scheduler.phase+=1;}
   }
  }
  match scheduler.phase{
   0=>{if !world.target_ray(){scheduler.phase=1;return;}world.abort();world.clear_unit_counter67();scheduler.function=Function::Idle;return;},
   1..=9=>{},_=>{world.clear_unit_counter67();scheduler.function=Function::Idle;return;},
  }
  loop{
   if scheduler.phase==9{if world.finish()==Completion::RetryVehicle{scheduler.phase=0;scheduler.side_grace=2;return;}world.clear_unit_counter67();scheduler.function=Function::Idle;return;}
   let direction=((scheduler.phase-1)/2)as usize;
   if scheduler.phase&1!=0{if !world.seed(direction,true){self.countdown=20;scheduler.phase+=1;return;}scheduler.phase+=2;}
   else{self.countdown=self.countdown.wrapping_sub(1);if self.countdown!=0&&world.walk_pending(direction,true){return;}scheduler.phase+=1;}
  }
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn function_address(f:Function)->u32{match f{Function::Idle=>0x40c810,Function::Prepare=>0x43e4a0,Function::Vehicle=>0x43dad0}}
 #[test]fn incremental_worker_timing_and_shared_clears_match_native_callbacks(){
  struct World{s:usize,tick:usize,trace:u32,finishes:usize,mask:Vec<u8>,headers:Vec<u8>,origin:i16,b4:u8,counter67:u8}
  impl World{fn call(&mut self,value:u32){self.trace=(self.trace^value).wrapping_mul(16777619);}fn hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619))}}
  impl WorkWorld for World{
   fn initialize_origin(&mut self){let(x,y)=(500+(self.s%16)as i32,600+(self.s%8)as i32);if self.s&1!=0{self.b4=(!self.b4)&1;self.origin=if self.b4!=0{(((y*8-57)&0xff80)+((x-7)>>4))as i16}else{((((y+8)*8)&0xff80)+((x+8)>>4))as i16};}else{self.origin=(4000+self.s)as i16;}}
   fn clear_mask(&mut self,start:usize,length:usize){self.mask[start..start+length].fill(65);}
   fn reset_candidates(&mut self){self.headers.fill(255);}
   fn target_ray(&mut self)->bool{self.call(2);self.s%17==0}
   fn seed(&mut self,d:usize,vehicle:bool)->bool{self.call((10+d+if vehicle{100}else{0})as u32);(self.s+d*3)%5==0}
   fn walk_pending(&mut self,d:usize,vehicle:bool)->bool{self.call((20+d+if vehicle{100}else{0})as u32);(self.s+self.tick+d)%11!=0}
   fn finish(&mut self)->Completion{self.call(30);let retry=self.s&1!=0&&self.s%7==0&&self.finishes==0;self.finishes+=1;if retry{Completion::RetryVehicle}else{Completion::Done}}
   fn abort(&mut self){self.call(31);}
   fn clear_unit_counter67(&mut self){self.counter67=0;}
  }
  let native=std::env::var("NATIVE_ROUTE_WORKER_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());let(mut hash,mut rows)=(2166136261u32,0);
  for s in 0..64usize{let mut scheduler=Scheduler{function:Function::Prepare,side_grace:127,..Default::default()};let mut worker=Worker::default();let mut world=World{s,tick:0,trace:2166136261,finishes:0,mask:(0..16384).map(|i|(s*13+i*7)as u8).collect(),headers:(0..400).map(|i|(s*19+i*3)as u8).collect(),origin:0,b4:((s>>2)&1)as u8,counter67:(17+s)as u8};
   for tick in 0..160{if scheduler.function==Function::Idle{break;}world.tick=tick;worker.advance(&mut scheduler,s&1!=0,&mut world);let values=[s as u32,tick as u32,function_address(scheduler.function),scheduler.phase as u32,u32::from(scheduler.side_grace),worker.countdown as u32,worker.mask_cursor as u32,world.origin as u32,u32::from(world.b4),u32::from(world.counter67),World::hash(&world.mask),World::hash(&world.headers),world.trace];
    if let Some(records)=&native{let r=&records[rows];let fields=["scenario","tick","function","phase","grace","countdown","cursor","origin","b4","counter67","maskHash","headerHash","trace"];let expected=fields.map(|f|r[f].as_i64().unwrap()as u32);assert_eq!(values,expected,"scenario{s}/tick{tick}");}for v in values{hash=(hash^v).wrapping_mul(16777619);}rows+=1;
   }
  }
  if let Some(records)=&native{assert_eq!(rows,records.len());}assert_eq!((rows,hash),(2207,1767276770),"executed43e4a0/43dad0 callbacks with recorded terrain boundaries");
 }
 #[test]fn dispatcher_and_temporary_permissions_match_512_native_calls(){
  let mut hash=2166136261u32;
  let native=std::env::var("NATIVE_ROUTE_SCHEDULER_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());
  for s in 0..512usize{
   let initial=s%240;let mut scheduler=Scheduler{current:initial as u8,function:[Function::Idle,Function::Prepare,Function::Vehicle][s%3],phase:(s*11)as i16,side_grace:(s*13)as u8};
   let actors=std::array::from_fn(|i|Actor{flags31:((i+s)&3)as u8|if(i+s)%3!=0{4}else{0}|if(i*7+s)%17==0{64}else{0},group:((i*13+s)%240)as u8,family:[4,24,30,32,34,36,38][i%7],class:[0,1,2,3,15][i%5],permissions:(i+s)%5!=0});
   let mut before=[-1i32;7];let mut after=std::array::from_fn::<_,7,_>(|j|{let(byte,extra)=if j<5{(24+j,0)}else{(24+j-5,41)};((s*17+initial*3+byte*13+extra)&255)as i32});
   let(mut group,mut called)=(-1,0);
   match scheduler.dispatch(&actors){
    Dispatch::Run{group:g,permissions,..}=>{group=i32::from(g);called=1;for(i,v)in permissions.into_iter().enumerate(){if let Some(v)=v{before[i]=i32::from(v);}else{before[i]=after[i];}}before[5]=before[0];before[6]=before[1];after[..5].copy_from_slice(&before[..5]);after[4]=255;},
    Dispatch::Invalid{group:g,..}=>group=i32::from(g),_=>{},
   }
   let mut values=vec![s as i32,i32::from(scheduler.current),function_address(scheduler.function)as i32,i32::from(scheduler.phase),i32::from(scheduler.side_grace),group,called];values.extend(before);values.extend(after);
   if let Some(records)=&native{let r=&records[s];let mut expected=["scenario","current","function","phase","grace","group","called"].map(|f|r[f].as_i64().unwrap()as i32).to_vec();for field in["before","after"]{expected.extend(r[field].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()as i32));}assert_eq!(values,expected,"scenario{s}");}
   for v in values{hash=(hash^v as u32).wrapping_mul(16777619);}
  }assert_eq!(hash,641891560,"43e780 with the path worker as explicit recorded call boundary");
 }
 #[test]fn storage_and_ownership_match_1024_complete_native_calls(){
  let mut hash=2166136261u32;
  let native=std::env::var("NATIVE_ROUTE_POOL_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());
  for mode in 0..4{for s in 0..256usize{
   let mut pool=Pool::default();for(i,n)in pool.nodes.iter_mut().enumerate(){*n=Node{point:[(i*7+s)as i16,(i as i32*11-s as i32)as i16],next:0};}
   let free=[0,1,239,240,4799,4800,14399,14400][s%8];if free<14400{pool.nodes[free].next=FREE;}
   pool.nodes[500].next=Pool::address(501);pool.nodes[501].next=0;pool.nodes[5000].next=Pool::address(5001);pool.nodes[5001].next=0;
   pool.links[0]=Links{primary:Pool::address(500),secondary:Pool::address(5000)};pool.links[1].primary=if s&16!=0{Pool::address(free)}else{Pool::address(500)};
   let mut flags=(s*3)as u8&3|32|64|if s&1==0{4}else{0};let role=[-1,0,1][s%3];let mut active=[false;240];active[0]=flags&4!=0;active[1]=s&32!=0;
   let mut scheduler=Scheduler{current:if s&8!=0{0}else{1},function:if s&4!=0{Function::Prepare}else{Function::Idle},..Default::default()};let mut hardware=(s*13)as u8;let mut result=0;
   match mode{
    0=>{flags&=!64;scheduler.cancel(0);let replaces=s&2!=0;let available=pool.first_free(0,240).is_some();result=u32::from(pool.direct(0,[-300+s as i16,800-s as i16],replaces,&active,role).is_none());if available{flags&=!32;if replaces&&(active[0]||role!= -1){hardware&=!1;}}},
    1=>{if pool.clear(0,active[0],role){hardware&=!1;}flags&=!32;},
    2=>{scheduler.cancel(0);pool.clear_secondary(0);flags&=!32;},
    _=>{let candidate=[0,FREE,PRIMARY,SECONDARY,SECONDARY+8,PRIMARY+4799*8,PRIMARY-8][s%7];pool.assign_primary(0,candidate,&active);},
   }
   let mut pool_hash=2166136261u32;for n in &pool.nodes{for v in[n.point[0]as u32,n.point[1]as u32,n.next]{pool_hash=(pool_hash^v).wrapping_mul(16777619);}}
   let function=if scheduler.function==Function::Idle{0x40c810}else{0x43e4a0};let values=[mode,s as u32,pool.links[0].primary,pool.links[0].secondary,u32::from(flags),u32::from(hardware),function,result,pool_hash];
   if let Some(records)=&native{let r=&records[mode as usize*256+s];let fields=["mode","scenario","primary","secondary","flags","hardware17","function","result","hash"];let expected=fields.map(|f|r[f].as_u64().unwrap()as u32);assert_eq!(values,expected,"mode{mode}/scenario{s}");}
   for v in values{hash=(hash^v).wrapping_mul(16777619);}
  }}assert_eq!(hash,104935595,"actual59c60/52500/524b0/44d1a0 with complete shared node arrays");
 }
}
