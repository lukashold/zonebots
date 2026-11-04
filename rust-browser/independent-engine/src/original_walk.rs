//!Native452200 ordinary walking and48060 attack-walking callers. Their
//!movement, contact and spatial leaves are explicit ordered callbacks.
#[path="original_walk_game.rs"]mod game;
#[path="original_walk_ground.rs"]pub(crate) mod ground;
#[path="original_walk_wait.rs"]pub(crate) mod wait;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct State{
 pub position:[i16;2],pub role:i8,pub phase:u8,pub frame:i16,pub timer:i8,
 pub heading:u8,pub bucket38:u8,pub axis81:u8,pub speed8b:i8,pub range4:i8,
 pub flags31:u8,pub flags64:u8,pub flags69:u8,pub flags71:u8,pub flags9c:u8,
 pub contact5d:u8,pub contact8c:u8,pub counter9f:u8,pub terrain:u8,
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Group{
 pub flags9:u8,pub layout8:u8,pub saved3c:u8,pub heading11:u8,pub counter3f:u8,pub target3b:u8,
}
#[derive(Clone,Copy,Debug,Default)]pub struct Follower{pub flags8a:u8,pub flags31:u8,pub contact5d:u8}
///521a0 inspects precisely follower roles1..20; it does not inspect theleader.
pub fn waiting(followers:impl IntoIterator<Item=Follower>)->u8{followers.into_iter().take(20).filter(|f|(f.flags8a&4!=0||f.flags8a&8==0&&f.flags31&64!=0)&&f.contact5d==0).count()as u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Choice{pub slot:u8,pub distance:u8}
pub trait World{
 fn near_order_stop(&mut self,s:&mut State,g:&mut Group)->bool;
 fn approach_contact(&mut self,s:&mut State,g:&mut Group);
 fn waiting(&mut self)->u8;
 fn step(&mut self,s:&mut State,g:&mut Group,axis:u8);
 fn arrive_contact(&mut self,s:&mut State,g:&mut Group);
 fn weapon_scan(&mut self,s:&mut State,g:&mut Group)->Choice;
 fn object_scan(&mut self,s:&mut State,g:&mut Group)->Choice;
 fn object_blocked(&mut self,slot:u8)->bool;
 fn claim_object(&mut self,s:&mut State,g:&mut Group,slot:u8);
 fn acquire(&mut self,s:&mut State,g:&mut Group,slot:u8);
 fn release_claim(&mut self,s:&mut State,g:&mut Group);
}
const COUNTER:[i8;13]=[10,8,6,5,5,4,3,2,2,2,1,1,1];
pub fn advance_frame(s:&mut State){s.timer=s.timer.wrapping_sub(1);if s.timer<0{s.timer=COUNTER[if s.speed8b<1{0}else{s.speed8b as usize}];s.frame=s.frame.wrapping_add(1)&3;}}
fn position(s:&mut State,g:&mut Group,w:&mut impl World){
 if s.flags69&8==0{if matches!(s.axis81,0|2|4|6){w.step(s,g,s.axis81);}if s.role<0{g.heading11=s.heading;if g.counter3f==0{g.flags9|=1;}}}
 else if s.role<0{g.heading11=s.heading;}
 s.bucket38=((s.position[1]>>3)as u8&240).wrapping_add((s.position[0]>>7)as u8);
}
///48060 advances its counter even when69:8 suppresses position updates.
///`mode_nonzero` is raw622dc0, toggled by434a70 independently ofMP/team.
pub fn attack(s:&mut State,g:&mut Group,mode_nonzero:bool,w:&mut impl World){
 if s.role== -1&&w.waiting()!=0{g.flags9&=!1;}else{position(s,g,w);advance_frame(s);}
 if s.phase!=2{w.arrive_contact(s,g);return;}
 if !mode_nonzero{let c=w.weapon_scan(s,g);if c.slot!=255&&c.slot!=g.target3b&&i32::from(c.distance)*2<=i32::from(s.range4){w.acquire(s,g,c.slot);}}
 else{let c=w.object_scan(s,g);if c.slot!=255&&i32::from(c.distance)*2<=i32::from(s.range4)&&!w.object_blocked(c.slot){w.claim_object(s,g,c.slot);}}
}
///452200 suppresses its counter as well as movement when69:8 is set. Its
///leader contact/proximity prefix precedes waiting and formation restoration.
pub fn ordinary(s:&mut State,g:&mut Group,mode_nonzero:bool,w:&mut impl World){
 let stalled=if s.role== -1{
  if s.frame<0{s.frame=0;}if s.counter9f!=0{s.counter9f-=1;}
  if w.near_order_stop(s,g){return;}w.approach_contact(s,g);
  if s.flags71&4==0{g.layout8=g.saved3c;}if s.flags64&128!=0||s.terrain&127==23{g.layout8=255;}
  if w.waiting()!=0{g.flags9&=!1;s.frame=i16::from(s.heading as i8).wrapping_mul(-3).wrapping_sub(8);true}else{false}
 }else{false};
 if !stalled{position(s,g,w);if s.flags69&8==0{advance_frame(s);}}
 if s.phase!=2{w.arrive_contact(s,g);return;}
 if s.contact5d==0&&mode_nonzero{let c=w.object_scan(s,g);if c.slot==255{if s.flags31&8!=0&&s.flags9c&64!=0{w.release_claim(s,g);if s.flags9c&16==0{s.contact8c=0;}}}
  else if i32::from(c.distance)*2<=i32::from(s.range4)&&!w.object_blocked(c.slot){w.claim_object(s,g,c.slot);}}
}
#[cfg(test)]mod tests{
 use super::*;
 struct Env{scenario:usize,trace:u32}
 impl Env{fn n(&mut self,n:u32){self.trace=(self.trace^n).wrapping_mul(16777619);}fn choice(&self)->Choice{Choice{slot:[255,5,17][self.scenario%3],distance:(self.scenario*7%128)as u8}}}
 impl World for Env{
  fn near_order_stop(&mut self,_:&mut State,_:&mut Group)->bool{self.n(1);self.scenario%13==0}
  fn approach_contact(&mut self,_:&mut State,_:&mut Group){self.n(2);}
  fn waiting(&mut self)->u8{waiting((1..=20).map(|i|Follower{flags8a:if i==self.scenario%20+1&&self.scenario%4==0{if self.scenario%8==0{4}else{0}}else{8},flags31:if i==self.scenario%20+1&&self.scenario%4==0{64}else{0},contact5d:0}))}
  fn step(&mut self,s:&mut State,_:&mut Group,axis:u8){self.n(10+u32::from(axis));let n=self.scenario;s.position=[500+(n%17)as i16+(n%3)as i16-1,600+(n%13)as i16+(n%5)as i16-2];s.heading=((n+1)%8)as u8;s.phase=if n%11==0{61}else{2};}
  fn arrive_contact(&mut self,_:&mut State,_:&mut Group){self.n(3);}
  fn weapon_scan(&mut self,_:&mut State,_:&mut Group)->Choice{self.n(4);self.choice()}
  fn object_scan(&mut self,_:&mut State,_:&mut Group)->Choice{self.n(5);self.choice()}
  fn object_blocked(&mut self,_:u8)->bool{self.n(6);self.scenario%5==0}
  fn claim_object(&mut self,_:&mut State,_:&mut Group,slot:u8){self.n(u32::from(slot));self.n(7);}
  fn acquire(&mut self,_:&mut State,_:&mut Group,slot:u8){self.n(u32::from(slot));self.n(8);}
  fn release_claim(&mut self,s:&mut State,_:&mut Group){self.n(9);s.flags31&=!8;s.contact5d=0;}
 }
 #[test]fn both_callers_match2048_native_records_including_full_roster_wait(){let mut h=2166136261u32;for mode in 0..2{for n in 0..1024usize{let mut s=State{role:[-1,0,1][n%3],position:[500+(n%17)as i16,600+(n%13)as i16],phase:2,frame:[-4,0,1,3,255,256][n%6],timer:[-128,-1,0,1,2,127][n/6%6],heading:(n%8)as u8,bucket38:(n*31)as u8,axis81:[0,1,2,3,4,5,6,7,255][n%9],speed8b:(n/36%13)as i8,range4:[-128,-1,0,10,60,127][n/9%6],flags69:(n*17)as u8,flags71:(n*19)as u8,flags64:(n*23)as u8,flags31:(n*29)as u8,flags9c:(n*37)as u8,contact5d:(n/6%4)as u8,contact8c:(n*41)as u8,counter9f:(n*43)as u8,terrain:if n%5==0{23}else{10}};let mut g=Group{layout8:(n*3)as u8,flags9:(n*7)as u8,saved3c:(n*11)as u8,heading11:(n*13)as u8,counter3f:(n%5)as u8,target3b:[255,0,1,17][n%4]};let mut e=Env{scenario:n,trace:2166136261};if mode==0{ordinary(&mut s,&mut g,n/3%2!=0,&mut e);}else{attack(&mut s,&mut g,n/3%2!=0,&mut e);}for value in[mode,n as u32,s.position[0]as u32,s.position[1]as u32,u32::from(s.phase),s.frame as u32,s.timer as u32,u32::from(s.heading),u32::from(s.bucket38),u32::from(s.flags31),u32::from(s.contact5d),u32::from(s.contact8c),u32::from(s.counter9f),u32::from(g.layout8),u32::from(g.flags9),u32::from(g.heading11),u32::from(g.saved3c),u32::from(g.counter3f),u32::from(g.target3b),e.trace]{h=(h^value).wrapping_mul(16777619);}}}assert_eq!(h,1564914470,"ZNov04WalkCallerOracle: native452200/48060+521a0, leaf callbacks explicitly recorded");}
}
