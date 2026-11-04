//!44a040 ground-target entry. Spatial leaves and grenade roster transfer are
//!explicit callbacks; the accepted frame/phase is not advanced in this call.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct State{pub slot:u8,pub position:[i16;2],pub point:[i16;2],pub class:u8,pub flags31:u8,pub role:i8,pub order:i8,pub phase:u8,pub frame:i16,pub timer:i8,pub range4:i8,pub grenades:u8,pub counter68:i8,pub heading:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Effect{Feedback,ClearPaths,FinishOrder,ResetIdle,StopPose,Walk,PointDirection,Broadcast,AssignGrenade(u8)}
pub trait World{fn own_building(&mut self)->bool;fn bridge(&mut self)->bool;fn object(&mut self)->bool;fn terrain(&mut self)->u8;fn ray(&mut self)->bool;fn grenade_total(&mut self)->u8;fn grenadier(&mut self)->Option<u8>;fn effect(&mut self,s:&mut State,flags9:&mut u8,e:Effect);}
pub fn distance(position:[i16;2],target:[i16;2])->i32{let x=(i32::from(position[0])-i32::from(target[0]))as f32;let y=(i32::from(position[1])-i32::from(target[1]))as f32;((((y*y*(1f32/256.)+x*x*(1f32/256.))as f64).sqrt())as f32)as i32}
pub fn enter(s:&mut State,flags9:&mut u8,tough_timer:i8,w:&mut impl World){
 if s.role== -1{*flags9&=!1;}if s.flags31&3==1&&w.own_building(){w.effect(s,flags9,Effect::ClearPaths);w.effect(s,flags9,Effect::ResetIdle);s.order=0;return;}
 let mut admission=1;if !w.bridge()&&!w.object(){admission=w.terrain();if admission==0{for e in[Effect::Feedback,Effect::ClearPaths,Effect::FinishOrder,Effect::ResetIdle,Effect::StopPose]{w.effect(s,flags9,e);}return;}}
 if distance(s.position,s.point).wrapping_mul(4)>=i32::from(s.range4){w.effect(s,flags9,Effect::Walk);return;}
 match s.class{
  2=>{w.effect(s,flags9,Effect::ClearPaths);s.frame=0;s.phase=19;s.timer=tough_timer;common(s,flags9,w);return;},
  15=>{if admission!=2&&!w.ray(){w.effect(s,flags9,Effect::Walk);return;}if w.terrain()!=2{w.effect(s,flags9,Effect::ClearPaths);s.phase=65;s.frame=0;s.timer=1;common(s,flags9,w);return;}},
  0|16=>{},_=>return,
 }
 if s.grenades!=0{s.frame=0;s.phase=49;s.timer=3;w.effect(s,flags9,Effect::PointDirection);return;}
 if w.grenade_total()==0{w.effect(s,flags9,Effect::ClearPaths);w.effect(s,flags9,Effect::ResetIdle);s.order=0;return;}
 if let Some(slot)=w.grenadier(){w.effect(s,flags9,Effect::AssignGrenade(slot));if slot==s.slot{return;}}
 s.counter68=10;s.frame=0;w.effect(s,flags9,Effect::ClearPaths);s.phase=8;
}
fn common(s:&mut State,flags9:&mut u8,w:&mut impl World){s.order=-5;w.effect(s,flags9,Effect::PointDirection);w.effect(s,flags9,Effect::Broadcast);}
#[cfg(test)]mod tests{
 use super::*;
 #[derive(Clone)]struct Actor{order:i8,phase:u8,frame:i16,timer:i8,counter:i8,heading:u8,target:i32,point:[i16;2],saved:[i16;2],flags:u8,grenades:u8}
 struct Env{n:usize,trace:u32,actors:Vec<Actor>,point:[i16;2]}
 impl Env{fn n(&mut self,n:u32){self.trace=(self.trace^n).wrapping_mul(16777619);}fn sync(&mut self,s:&State){let a=&mut self.actors[0];a.order=s.order;a.phase=s.phase;a.frame=s.frame;a.timer=s.timer;a.counter=s.counter68;a.heading=s.heading;a.point=s.point;a.flags=s.flags31;}}
 impl World for Env{
  fn own_building(&mut self)->bool{self.n(1);self.n%7==0&&self.n/19%3==1}
  fn bridge(&mut self)->bool{self.n(2);self.n%5==0}
  fn object(&mut self)->bool{self.n(3);self.n%5==1}
  fn terrain(&mut self)->u8{self.n(4);(self.n%4)as u8}
  fn ray(&mut self)->bool{self.n(7);self.n(12);self.n%3!=0}
  fn grenade_total(&mut self)->u8{self.n(13);(self.n/13%2)as u8}
  fn grenadier(&mut self)->Option<u8>{self.actors.iter().position(|a|a.grenades!=0&&matches!(a.phase,7|1)).map(|n|n as u8)}
  fn effect(&mut self,s:&mut State,_flags9:&mut u8,e:Effect){self.sync(s);match e{
   Effect::Feedback=>self.n(5),Effect::ClearPaths=>{self.n(6);s.flags31&=!32;},Effect::FinishOrder=>self.n(8),Effect::ResetIdle=>{self.n(9);s.phase=1;s.frame=1;s.timer=3;},Effect::StopPose=>{self.n(10);s.phase=42;s.frame=0;},Effect::Walk=>{self.n(11);s.phase=8;s.frame=100;s.timer=-7;s.counter68=31;},
   Effect::PointDirection=>{self.n(14);s.heading=(self.n%8)as u8;},
   Effect::Broadcast=>{if s.role!= -1{return;}for a in &mut self.actors[1..]{if a.flags&0x28==0{a.order=5;a.target=3456;a.point=self.point;}}},
   Effect::AssignGrenade(slot)=>{self.n(14);let a=&mut self.actors[usize::from(slot)];a.target=3456;a.point=s.point;a.saved=[1234+self.n as i16,-2000+self.n as i16*5];a.phase=49;a.frame=0;a.timer=3;a.heading=(self.n%8)as u8;if slot==0{s.phase=a.phase;s.frame=a.frame;s.timer=a.timer;s.heading=a.heading;}},
  }self.sync(s);}
 }
 #[test]fn ground_entry_matches1024_native_calls_and_actual_roster_transfer(){let mut hash=2166136261u32;for n in 0..1024usize{let point=[500+(n/16%9)as i16*40-160,600+(n/144%7)as i16*40-120];let mut s=State{slot:0,position:[500,600],point,class:[0,1,2,3,4,15,16,18][n%8],flags31:4+(n/3%4)as u8,role:[-1,0,1][n/8%3],order:-5,phase:8,frame:(n*11)as i16,timer:(n*13)as i8,range4:[-128,0,30,60,127][n/8%5],grenades:(n/11%2)as u8,counter68:(n*17)as i8,heading:(n%8)as u8};let mut flags=(n*7)as u8;let mut e=Env{n,trace:2166136261,point,actors:(0..21).map(|i|if i==0{Actor{order:s.order,phase:s.phase,frame:s.frame,timer:s.timer,counter:s.counter68,heading:s.heading,target:3456,point,saved:[1234+n as i16,-2000+n as i16*5],flags:s.flags31,grenades:s.grenades}}else{Actor{flags:((n+i*7)&0x38)as u8,phase:[0,1,7,8][((n+i)/7)%4],grenades:u8::from((n+i)%5==0),order:3,target:100+i as i32,point:[222+i as i16,333+i as i16],frame:444+i as i16,timer:55,counter:0,heading:6,saved:[666+i as i16,777+i as i16]}}).collect()};enter(&mut s,&mut flags,20,&mut e);e.sync(&s);let mut roster=2166136261u32;for a in &e.actors{for v in[a.order as u32,u32::from(a.phase),a.frame as u32,a.timer as u32,a.counter as u32,u32::from(a.heading),a.target as u32,a.point[0]as u32,a.point[1]as u32,a.saved[0]as u32,a.saved[1]as u32,u32::from(a.flags)]{roster=(roster^v).wrapping_mul(16777619);}}let values=[n as u32,u32::from(s.phase),s.order as u32,s.frame as u32,s.timer as u32,s.counter68 as u32,u32::from(s.heading),u32::from(flags),e.trace,roster];if let Ok(path)=std::env::var("NATIVE_GROUND_ENTRY_ORACLE"){let data=std::fs::read_to_string(path).unwrap();let r:serde_json::Value=serde_json::from_str(data.lines().nth(n).unwrap()).unwrap();let expected=["scenario","phase","order","frame","timer","counter","heading","g9","trace","roster"].map(|k|r[k].as_i64().unwrap()as u32);assert_eq!(values,expected,"scenario{n}");}for v in values{hash=(hash^v).wrapping_mul(16777619);}}assert_eq!(hash,369284258,"native44a040 with real49e10/49fd0; spatial/movement/cleanup leaves explicit");}
}
