//!44cac0 negative-order continuation. World operations retain their source
//!call order; a started52810 line returns AL0 even for a zero-length segment.
#[path="original_reset_game.rs"]mod game;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{None,ClearOrder,ReverseOrder,Acquire(u8)}
pub trait World{
 fn primary_present(&self)->bool;
 fn start_path(&mut self)->bool;
 fn check_path(&mut self)->u8;
 fn group_target(&self)->u8;
 fn scan(&mut self)->u16;
 fn target_invalid(&mut self,target:u8)->bool;
}
pub fn continuation(order:i8,range04:i8,world:&mut impl World)->Action{
 if order>=0{return Action::None;}
 if order!= -5{
  if !world.primary_present()||!world.start_path(){return Action::None;}
  if world.check_path()&1==0{return Action::None;}
  world.start_path();
 }
 if world.group_target()==255{return Action::ClearOrder;}
 let probe=world.scan();let target=(probe>>8)as u8;
 if target==255||i32::from(range04)<i32::from(probe as u8)*2||world.target_invalid(target){Action::ReverseOrder}else{Action::Acquire(target)}
}
///16270. The target byte names a live native slot, not a browser entity ID.
pub fn target_invalid(actor_flags31:u8,target_flags31:u8)->bool{target_flags31&4==0||(actor_flags31^target_flags31)&3==0}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Node{pub point:[i16;2],pub next:u32}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct PathFields{pub position:[i16;2],pub role:i8,pub flags31:u8,pub flags9c:u8,pub primary:u32,pub secondary:u32,pub group_layout8:u8,pub group_saved3c:u8,pub group_flags9:u8}
pub trait Paths{fn node(&self,pointer:u32)->Option<Node>;fn retire_link(&mut self,pointer:u32);fn assign_primary(&mut self,pointer:u32)->u32;fn primary_building_detours(&mut self,fields:&mut PathFields,next:Node);}
///52960 consumes only nodes whose waypoint−8 equals the actor anchor. The
///return bits1/2 distinguish remaining work and a secondary-node advance.
pub fn check_path(fields:&mut PathFields,pool:&mut impl Paths)->u8{
 let restore=|f:&mut PathFields|{f.flags31&=!32;if f.role== -1{f.group_layout8=f.group_saved3c;}};
 let mut pointer=if fields.flags31&32!=0{if fields.secondary==0{restore(fields);fields.primary}else{fields.secondary}}else{fields.primary};
 loop{
  if pointer==0{fields.primary=0;if fields.role== -1{fields.group_flags9&=!8;}return 2;}
  let Some(node)=pool.node(pointer)else{return 0;};
  if node.point.map(|v|v.wrapping_sub(8))!=fields.position{return 1;}
  pool.retire_link(pointer);
  if fields.flags31&32!=0{
   if node.next!=0&&node.next!=u32::MAX{fields.secondary=node.next;return 3;}
   fields.secondary=0;restore(fields);pointer=fields.primary;
  }else{
   fields.flags9c&=!2;
   if node.next!=0&&node.next!=u32::MAX{fields.primary=pool.assign_primary(node.next);pool.primary_building_detours(fields,pool.node(node.next).unwrap_or(node));return 1;}
   fields.primary=0;if fields.role== -1{fields.group_flags9&=!8;}return 2;
  }
 }
}
///528f0/52880 strict signed arithmetic. The ordinary repair-ruin branch
///wraps targetY+64 before widening, whereas the crane branch does not.
pub fn ruin_detour(current_y:i16,target:[i16;2],family:u8,ordinary_kind:Option<i8>,crane_hit:bool)->Option<[i16;2]>{
 if ordinary_kind==Some(0)&&i32::from(target[1].wrapping_add(64))-i32::from(current_y)>64{return Some([target[0],target[1].wrapping_add(64)]);}
 if family==36&&crane_hit&&i32::from(target[1])-i32::from(current_y)>0{return Some([target[0],target[1].wrapping_add(64)]);}None
}
#[derive(Clone,Copy,Debug)]pub struct ScanActor{pub flags31:u8,pub flags69:u8,pub flags8a:u8,pub flags64:u8,pub protection:u8,pub counter97:i8,pub group_flags9:u8,pub position:[i16;2]}
#[derive(Clone,Copy,Debug)]pub struct ScanUnit{pub slot:u8,pub position:[i16;2],pub flags31:u8,pub flags69:u8,pub protection:u8,pub phase35:u8}
pub fn scan_blocked(a:ScanActor)->bool{a.flags69&64!=0||a.protection!=0||a.flags8a&16!=0||a.flags64&32!=0||a.group_flags9&32!=0}
///58500→4cd60→4ce00. Cone return values are deliberately ignored; only
///the callback's signed X rewrite contributes to the quantized distance.
pub fn scan(a:ScanActor,units:&[ScanUnit],candidates:&[u8],direction:u8,mut visible:impl FnMut(u8)->bool)->u16{
 if scan_blocked(a){return 0xff00;}let(mut slot,mut distance)=(255,255);
 for &id in candidates{let Some(u)=units.iter().find(|u|u.slot==id)else{continue;};if(a.flags31^u.flags31)&3==0||u.protection!=0||u.flags69&64!=0||a.counter97>=16||a.flags69&4==0||u.phase35==18{continue;}
  let(_,x)=super::super::original_initiative::native::cone(direction,u.position[0].wrapping_sub(a.position[0]),u.position[1].wrapping_sub(a.position[1]));let x=i32::from(x)>>3;let y=i32::from(u.position[1].wrapping_sub(a.position[1]))>>3;
  let d=(((x*x+y*y)as f32 as f64).sqrt()as f32 as i32)as u8;
  if d<distance&&visible(id){slot=id;distance=d;}
 }u16::from(slot)*256+u16::from(distance)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct FireSelection{pub phase:u8,pub frame:i16,pub timer:i8}
///44a900 selection follows raw class/rate bytes. A Pyro always keeps its
///flame handler; grenades are considered only after all class overrides.
pub fn fire_selection(class:u8,rate:u8,grenades:u8,target_class:u8,old_phase:u8,tough_timer:u8)->FireSelection{
 let mut s=if old_phase==5{FireSelection{phase:5,frame:3,timer:1}}else{FireSelection{phase:5,frame:0,timer:3}};
 if class==15{return FireSelection{phase:65,frame:0,timer:1};}
 if rate>=92{s=FireSelection{phase:57,frame:0,timer:1};}
 if class==3{return FireSelection{phase:19,frame:0,timer:tough_timer as i8};}
 if class==16{return FireSelection{phase:71,frame:0,timer:1};}
 if grenades!=0&&matches!(target_class,1|2){return FireSelection{phase:49,frame:0,timer:5};}s
}
///492f0 uses SSE single-precision squares and an unsigned range byte.
pub fn requires_approach(position:[i16;2],target:[i16;2],range74:u8)->bool{
 let x=(i32::from(target[0])-i32::from(position[0]))as f32;let y=(i32::from(target[1])-i32::from(position[1]))as f32;
 (f32::from(range74)*f32::from(range74))*64.<y*y+x*x
}
#[derive(Clone,Copy,Debug)]pub struct ReactionInput{pub source_flags31:u8,pub flags31:u8,pub flags8a:u8,pub family:u8,pub class:u8,pub group:u8,pub leader_flags31:u8,pub leader_flags69:u8,pub group_timer3d:u8,pub selected_leader:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Reaction{None,Hardware,Leader}
///4a3a0 through its native reaction dispatch. GUI selection is an explicit
///input; it must not be guessed from a unit's browser order string.
pub fn reaction(r:ReactionInput)->Reaction{
 if(r.source_flags31^r.flags31)&3==0||r.flags8a&8!=0||matches!(r.family,36|38){return Reaction::None;}
 if r.class==2{return Reaction::Hardware;}
 if target_invalid(r.flags31,r.source_flags31){return Reaction::None;}
 if r.class==1{return Reaction::Hardware;}
 if r.group==255||r.leader_flags31&3!=1&&r.group_timer3d!=0||r.leader_flags69&128!=0||r.selected_leader{return Reaction::None;}Reaction::Leader
}
///45afb0's (+12,+8) hardware target gate. APC range0 is the special8px case.
pub fn outside_hardware_range(position:[i16;2],target:[i16;2],class:u8,family:u8,hardware_range5:u8)->bool{
 let x=i32::from(position[0])-i32::from(target[0])+12;let y=i32::from(position[1])-i32::from(target[1])+8;let square=x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y));
 let n=if matches!(class,1|2){if family==32{u32::from(hardware_range5.max(1))}else{u32::from(hardware_range5)+1}}else{u32::from(hardware_range5)};let range=(n*8).wrapping_mul(n*8)as i32;square>range
}
#[derive(Clone,Copy,Debug)]pub struct HardwareReactionInput{pub actor_flags31:u8,pub source_flags31:u8,pub family:u8,pub class:u8,pub timer3d:u8,pub command3a:u8,pub outside_range:bool,pub los_blocked:bool,pub source_role:i8,pub source_group:u8,pub source_slot:u8,pub actor_slot:u8,pub flags69:u8,pub flags64:u8,pub target:[i16;2]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct HardwareReaction{pub group_target:Option<u8>,pub target:[i16;2],pub flags69:u8,pub flags64:u8,pub release_claim:bool}
///49bc0. The source group's target is written before the repeated-target
///early return. The command override4d0e37 is cold0 and has no native writer.
pub fn hardware_reaction(r:HardwareReactionInput)->HardwareReaction{
 let mut out=HardwareReaction{group_target:None,target:r.target,flags69:r.flags69,flags64:r.flags64,release_claim:false};
 if(r.actor_flags31^r.source_flags31)&3==0||r.family==36||r.command3a!=0&&r.outside_range||!(r.class==2||r.class==1&&r.family!=38&&(!matches!(r.family,32|34)||!r.los_blocked))||r.timer3d!=0{return out;}
 if r.source_role!=0||r.source_group!=255{out.group_target=Some(r.actor_slot);}
 if r.source_role==0{out.target=[-4321,i16::from(r.source_slot)];}else{if r.flags69&8!=0&&r.target==[-1234,i16::from(r.source_group)]{return out;}out.flags69&=!32;out.target=[-1234,i16::from(r.source_group)];}
 out.flags64|=8;out.flags69|=128;out.release_claim=true;out
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct AttackReport{pub marker:Option<u8>,pub append:bool}
pub fn attack_report(flags71:u8,multiplayer:bool,role:i8,flags31:u8,strategy_flags16:u8,already_present:bool)->AttackReport{
 if flags71&16==0||multiplayer||role==0{return AttackReport{marker:None,append:false};}
 let active=flags31&7==5;AttackReport{marker:Some(u8::from(active)),append:active&&strategy_flags16&16==0&&!already_present}
}
#[derive(Clone,Copy,Debug)]pub struct BroadcastCandidate{pub role:i8,pub flags31:u8,pub class:u8,pub same_actor:bool,pub same_sector:bool,pub order5c:i8,pub flags64:u8,pub flags69:u8,pub flags71:u8,pub phase35:u8,pub family:u8}
pub fn broadcast_candidate(actor_flags71:u8,actor_flags31:u8,c:BroadcastCandidate)->bool{actor_flags71&16!=0&&c.role<0&&c.flags31&4!=0&&(c.flags31^actor_flags31)&3==0&&c.class!=2&&!c.same_actor&&c.same_sector&&c.order5c==0&&c.flags31&0x50==0&&c.flags64&8==0&&c.flags69&64==0&&c.flags71&1==0&&c.phase35!=18&&c.family!=36}
#[cfg(test)]mod tests{
 use super::*;
 struct Env{s:usize,starts:u32,second:bool,first:u8,other:u8,events:u32,hash:u32}
 impl Env{fn event(&mut self,n:u32){self.events+=1;self.hash=(self.hash^n).wrapping_mul(16777619);}}
 impl World for Env{
  fn primary_present(&self)->bool{self.s/60%2!=0}
  fn start_path(&mut self)->bool{self.event(10+self.starts);if self.starts==1&&self.s&1!=0{self.second=true;}self.starts+=1;self.s/480%2==0}
  fn check_path(&mut self)->u8{self.event(20);[1,2,3][self.s/960%3]}
  fn group_target(&self)->u8{if self.second{self.other}else{self.first}}
  fn scan(&mut self)->u16{self.event(30);[0xffff,0,0x010a,0x01ff][self.s/240%4]}
  fn target_invalid(&mut self,_:u8)->bool{self.event(40);self.s/1920%2!=0}
 }
 #[test]fn all_4096_native_reset_continuations_match_call_order_and_command_writes(){let mut all=2166136261u32;
  for s in 0..4096{let class=[0,1,2,3][s%4];let role=[-1,0,1][s/4%3];let mut order=[-128i8,-5,-1,0,5][s/12%5];let mut w=Env{s,starts:0,second:false,first:if role== -1||s%7==0{255}else{2},other:if s%11==0{255}else{3},events:0,hash:2166136261};
   if class!=2{match continuation(order,[-1,0,20,127][s/120%4],&mut w){Action::ClearOrder=>order=0,Action::ReverseOrder=>order=order.wrapping_neg(),Action::Acquire(t)=>w.event(50+u32::from(t)),Action::None=>{}}}
   for n in[s as u32,u32::from(order as u8),u32::from(w.first),u32::from(w.other),if matches!(class,1|2){0}else{2},w.events,w.hash]{all=(all^n).wrapping_mul(16777619);}
  }assert_eq!(all,362787713);
 }
 struct Pool{nodes:[Node;4]}
 impl Pool{fn index(p:u32)->Option<usize>{p.checked_sub(0x5eee28).map(|n|(n/8)as usize).filter(|&i|i<4)}fn bytes(&self)->Vec<u8>{self.nodes.iter().flat_map(|n|n.point[0].to_le_bytes().into_iter().chain(n.point[1].to_le_bytes()).chain(n.next.to_le_bytes())).collect()}}
 impl Paths for Pool{fn node(&self,p:u32)->Option<Node>{Some(self.nodes[Self::index(p)?])}fn retire_link(&mut self,p:u32){if let Some(i)=Self::index(p){self.nodes[i].next=u32::MAX;}}fn assign_primary(&mut self,p:u32)->u32{p}fn primary_building_detours(&mut self,_:&mut PathFields,_:Node){}}
 #[test]fn all_1024_native_path_checks_match_nodes_flags_and_formation_restoration(){let mut all=2166136261u32;
  for s in 0..1024usize{let b=0x5eee28u32;let mut pool=Pool{nodes:std::array::from_fn(|i|Node{point:[328+if s>>(i+2)&1!=0{1}else{0},328],next:if i==0&&s&1!=0{b+8}else if i==2&&s&2!=0{b+24}else{0}})};
   let mut f=PathFields{position:[320,320],role:[-1,0,1][s/32%3],flags31:5|if s&16!=0{32}else{0},flags9c:(s*13)as u8,primary:if s&4!=0{b}else{0},secondary:if s&8!=0{b+16}else{0},group_layout8:9,group_saved3c:4,group_flags9:255};let result=check_path(&mut f,&mut pool);let ph=pool.bytes().iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619));
   for n in[s as u32,u32::from(result),f.primary,f.secondary,u32::from(f.flags31),u32::from(f.flags9c),u32::from(f.group_layout8),u32::from(f.group_flags9),ph]{all=(all^n).wrapping_mul(16777619);}
  }assert_eq!(all,537027165);
 }
 #[test]fn all_1024_native_acquisition_scans_match_raw_gates_rotation_candidates_and_los_order(){let mut all=2166136261u32;for s in 0..1024usize{
  let a=ScanActor{position:[500,600],flags31:4,flags69:[4,0,68,4][s/24%4],flags8a:if s/96%5==2{16}else{0},flags64:if s/96%5==3{32}else{16},protection:if s/96%5==1{1}else{0},counter97:[0,15,16,-1][s/480%4],group_flags9:if s/96%5==4{32}else{0}};
  let mut units=Vec::new();let mut buckets:[Vec<u8>;256]=std::array::from_fn(|_|vec![]);for i in 0..80usize{let p=[(500+(i*37+s*13)%705)as i16-352,(600+(i*71+s*17)%705)as i16-352];units.push(ScanUnit{slot:i as u8,position:p,flags31:4+(i%2)as u8,protection:if i%13==0{1}else{0},flags69:if i%17==0{64}else{4},phase35:if i%19==0{18}else{1}});let b=usize::from(super::super::super::original_initiative::native::coarse_cell(p));if buckets[b].len()<32{buckets[b].push(i as u8);}}
  let mut flags71=(s*13)as u8;let mut h=super::super::super::original_hardware::State::new(0,-1);h.seed_raw_scan((s%8)as u8);let(mut calls,mut lh)=(0u32,2166136261u32);let result=if scan_blocked(a){0xff00}else{let d=h.raw_scan_direction(&mut flags71,[0,1,2][s/8%3],(s%8)as u8,0);let mut candidates=Vec::new();let mut count=0;for delta in super::super::super::original_initiative::native::CELLS[usize::from(d)]{let c=67i16.wrapping_add(delta as i16)as u16;if c<256{let b=&buckets[usize::from(c)];count+=b.len();if count<288{candidates.extend(b);}}}scan(a,&units,&candidates,d,|slot|{calls+=1;lh=(lh^u32::from(slot)).wrapping_mul(16777619);s%2==0||slot%11!=0})};for n in[s as u32,u32::from(result),u32::from(flags71),u32::from(h.raw_scan_cursor()),calls,lh]{all=(all^n).wrapping_mul(16777619);}
 }assert_eq!(all,2199158530);}
 #[test]fn all_480_native_attack_selections_match_raw_class_rate_grenade_priority(){let mut h=2166136261u32;let mut n=0u32;for class in[0,3,15,16,1]{for rate in[0,91,92,255]{for grenades in[0,255]{for target in[0,1,2,3]{for phase in[1,5,57]{let s=fire_selection(class,rate,grenades,target,phase,20);for x in[n,u32::from(s.phase),u32::from(s.frame as u16),u32::from(s.timer as u8)]{h=(h^x).wrapping_mul(16777619);}n+=1;}}}}}assert_eq!(h,2357420069);}
 fn mix(hash:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*hash=(*hash^n).wrapping_mul(16777619);}}
 #[test]fn all_4096_native_reaction_dispatches_match_selected_leader_and_hardware_tail(){let mut h=2166136261;for s in 0..4096u32{let af=[0,4,5,6][s as usize%4];let rf=(4+s/3%4)as u8;let role=if s/11%2==0{-1}else{0};let lh=(s/19%2*128)as u8;let r=reaction(ReactionInput{source_flags31:af,flags31:rf,flags8a:(s/29%2*8)as u8,family:[0,32,36,38][(s/5%4)as usize],class:(s/7%4)as u8,group:if s/13%3==0{255}else{2},leader_flags31:if role==-1{rf}else{(4+s/17%4)as u8},leader_flags69:lh,group_timer3d:(s/23%2)as u8,selected_leader:s/31%2!=0});let leader=r==Reaction::Leader;let hardware=r==Reaction::Hardware;mix(&mut h,[s,if leader{2}else if hardware{1}else{0},u32::from(if leader{lh|128}else{lh}),if leader{251}else{3},if leader{0}else{77},if leader{0}else{55},if hardware&&s/7%4==2{0}else{9},u32::from(leader),u32::from(leader||hardware)]);}assert_eq!(h,4187044972);}
 #[test]fn all_4096_native_hardware_reactions_match_shared_order_and_group_target_writes(){let mut h=2166136261;for s in 0..4096u32{let group=if s/13%3==0{255}else{3};let r=hardware_reaction(HardwareReactionInput{actor_flags31:(4+s%4)as u8,source_flags31:(4+s/3%4)as u8,family:[0,32,34,36,38][(s/5%5)as usize],class:(s/7%3)as u8,timer3d:(s/23%2)as u8,command3a:(s/29%3)as u8,outside_range:s/5%2!=0,los_blocked:s/7%2!=0,source_role:[0,-1,1][(s/11%3)as usize],source_group:group,source_slot:0,actor_slot:1,flags69:(s/17%4*8+s/19%2*128)as u8,flags64:16,target:[if s%2==0{-1234}else{5},i16::from(group)]});mix(&mut h,[s,u32::from(r.group_target.unwrap_or(7)),u32::from(r.target[0]as u16),u32::from(r.target[1]as u16),u32::from(r.flags69),u32::from(r.flags64),u32::from(r.release_claim)]);}assert_eq!(h,3098416148);}
 #[test]fn all_1024_native_attack_reports_match_marker_duplicates_and_ordered_queue(){let mut h=2166136261;for s in 0..1024u32{let mut len=if s/5%3==0{0}else{2};let mut bytes=[0u8;8];bytes[0]=if len==0{255}else{2};bytes[1]=if s/5%3==1{7}else{3};bytes[2]=255;let r=attack_report((s%2*16)as u8,s/2%2!=0,[-1,0,1][(s/7%3)as usize],(s/11%8)as u8,(s/13%2*16)as u8,bytes[..len].contains(&7));if r.append{bytes[len]=7;len+=1;bytes[len]=255;}let mut bh=2166136261;mix(&mut bh,bytes.map(u32::from));mix(&mut h,[s,u32::from(r.marker.unwrap_or(9)),len as u32,bh]);}assert_eq!(h,286759125);}
 #[test]fn all_256_native_sector_broadcasts_match_240_slot_dispatch_order(){let mut h=2166136261;for s in 0..256u32{let(mut n,mut bh)=(0,2166136261);for i in 0..240u32{let candidate=BroadcastCandidate{role:[-1,0,1][i as usize%3],flags31:(4+i/3%4)as u8|if i%17==0{16}else{0},class:if i%13==0{2}else{0},same_actor:i==0,same_sector:i%5!=0,order5c:if i%11==0{1}else{0},flags64:if i%19==0{8}else{16},flags69:if i%23==0{64}else{4},flags71:if i%29==0{1}else{0},phase35:if i%31==0{18}else{1},family:if i%37==0{36}else{0}};if broadcast_candidate(if s%3==0{0}else{16},(s%4)as u8,candidate){n+=1;mix(&mut bh,[i]);}}mix(&mut h,[s,n,bh]);}assert_eq!(h,2331683327);}
}
