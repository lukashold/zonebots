//!457590 phase7 dispatcher. Leaf calls remain ordered and reload live state.
#[path="original_follow_game.rs"]mod game;
#[derive(Clone,Copy,Debug,Default)]
pub struct State {pub remaining:i16,pub frame:i16,pub timer:i8,pub speed:i8,pub heading:u8,pub flags31:u8,pub flags64:u8,pub range:i8,pub aim:i8,pub group_target:u8,pub group_timer:u8,pub leader_order:i8,pub group_flags:u8,pub group_heading:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Call {Move,Random,GroupAttack,WeaponScan,TargetInvalid(u8),Acquire(u8),ObjectScan,Claim(u8),Follow,Sync}
#[derive(Clone,Copy,Debug)]
pub enum Write {Frame(i16),Timer(i8),Heading(u8),GroupFlags(u8)}
pub trait World {type Error;fn state(&self)->Result<State,Self::Error>;fn write(&mut self,w:Write);fn call(&mut self,c:Call)->Result<u16,Self::Error>;}
pub fn tick<W:World>(w:&mut W,mode:bool)->Result<(),W::Error> {
 if w.state()?.remaining!=0{w.call(Call::Move)?;}
 let s=w.state()?;let timer=s.timer.wrapping_sub(1);w.write(Write::Timer(timer));
 if timer<0{let speed=s.speed.max(0)as usize;w.write(Write::Timer([10,8,6,5,5,4,3,2,2,2,1,1,1][speed]));w.write(Write::Frame(s.frame.wrapping_add(1)&3));}
 let s=w.state()?;
 if mode{
  let choice=w.call(Call::ObjectScan)?;let slot=(choice>>8)as u8;
  if slot!=255&&i32::from(choice as u8)*2<=i32::from(w.state()?.range)&&w.call(Call::Claim(slot))?as u8==0{return Ok(());}
 }else if s.flags64&16!=0&&s.group_timer==0{
  if s.group_target!=255&&(matches!(s.leader_order,5|-5)||((w.call(Call::Random)?as u8&127)as i8)<s.aim){w.call(Call::GroupAttack)?;return Ok(());}
  let choice=w.call(Call::WeaponScan)?;let slot=(choice>>8)as u8;
  if slot!=255&&i32::from(choice as u8)*2<=i32::from(w.state()?.range)&&w.call(Call::TargetInvalid(slot))?as u8==0{
   w.call(Call::Acquire(slot))?;if w.state()?.flags31&8!=0{return Ok(());}
  }
 }
 let s=w.state()?;let frame=i16::from(s.frame as u8 as i8);
 w.write(Write::GroupFlags(s.group_flags|1));w.call(Call::Follow)?;
 w.write(Write::GroupFlags(s.group_flags));w.write(Write::Frame(frame));w.write(Write::Timer(s.timer));
 if s.heading.wrapping_sub(w.state()?.heading)&7<2{w.write(Write::Heading(s.heading));}
 let next=w.state()?;
 if next.remaining==0&&next.group_flags&1==0{w.call(Call::Sync)?;return Ok(());}
 if next.flags31&128!=0{w.write(Write::Heading(next.group_heading));}Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 struct Env{n:usize,s:State,hash:u32,switched:bool}
 impl World for Env{type Error=();fn state(&self)->Result<State,()>{Ok(self.s)}fn write(&mut self,w:Write){match w{Write::Frame(v)=>self.s.frame=v,Write::Timer(v)=>self.s.timer=v,Write::Heading(v)=>self.s.heading=v,Write::GroupFlags(v)=>self.s.group_flags=v}}
 fn call(&mut self,c:Call)->Result<u16,()>{let id=match c{Call::Move=>1,Call::Random=>2,Call::GroupAttack=>3,Call::WeaponScan=>4,Call::TargetInvalid(_)=>5,Call::Acquire(_)=>6,Call::ObjectScan=>7,Call::Claim(_)=>8,Call::Follow=>9,Call::Sync=>10};self.hash=(self.hash^id).wrapping_mul(16777619);let n=self.n;let choice=if n%3==0{0xff00}else{0x900+(n%14)as u16};Ok(match c{Call::Move=>{self.s.remaining=if n%7==0{0}else{15};0},Call::Random=>(n*23)as u16,Call::WeaponScan|Call::ObjectScan=>choice,Call::TargetInvalid(_)=>u16::from(n%4==0),Call::Acquire(_)=>{self.s.flags31|=if n&64!=0{8}else{0};0},Call::Claim(_)=>u16::from(n%3!=1),Call::Follow=>{self.switched=n&128!=0;self.s.group_flags=162;self.s.group_heading=if self.switched{3}else{6};self.s.frame=77;self.s.timer=99;self.s.heading=(n*3&7)as u8;self.s.remaining=if n&256!=0{0}else{12};self.s.flags31|=if n&512!=0{128}else{0};0},_=>0})}}
 #[test]fn dispatcher_matches_1024_original_executions(){
  let oracle=std::env::var("Z_NATIVE_FOLLOW_PHASE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|l|serde_json::from_str::<crate::Value>(l).unwrap()).collect::<Vec<_>>());let mut hash=2166136261u32;
  for n in 0..1024{let mut w=Env{n,s:State{remaining:if n%3==0{0}else{24},frame:((n*31)%400)as i16-100,timer:(n%5)as i8-2,speed:(n%13)as i8,heading:(n%8)as u8,flags31:5|if n&2!=0{8}else{0}|if n&4!=0{128}else{0},flags64:if n&8!=0{16}else{0},range:if n%5==0{-1}else{20},aim:(n%128)as i8,group_target:if n&16!=0{9}else{255},group_timer:if n&32!=0{2}else{0},leader_order:[5,-5,0][n%3],group_flags:(n*17)as u8,group_heading:6},hash:2166136261,switched:false};tick(&mut w,n&1!=0).unwrap();let s=w.s;let values=[i64::from(s.remaining),i64::from(s.frame),i64::from(s.timer),i64::from(s.heading),i64::from(s.flags31),i64::from(s.group_flags),i64::from(w.hash),if w.switched{72}else{0}];if let Some(o)=&oracle{assert_eq!(crate::json!(values),o[n]["values"],"scenario{n}");}for value in values{hash=(hash^value as u32).wrapping_mul(16777619);}}
  assert_eq!(hash,3261265149,"ZFollowerPhaseOracle: real457590 with explicit leaf boundaries");
 }
}
