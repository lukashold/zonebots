//! Hardware firing callers. Query/allocator calls are supplied in source order;
//! the byte counters and the fixed-gun branch are independent of tank geometry.
use crate::{native_math,native_weapon_data as data};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Frame{
 pub family:u8,pub class:u8,pub flags31:u8,pub flags64:u8,pub flags69:u8,
 pub position:[i16;2],pub target:[i16;2],pub direction:u8,pub turret:u8,pub direction34:u8,
 pub timer3c:i8,pub timer3d:u8,pub reload7b:i8,pub counter84:i16,
 pub counter14:i8,pub mask12:u8,pub scan77:u8,pub aim2:u8,pub rate:u8,pub unit_rate:u8,pub range4:i8,pub speed:i8,
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Resolved{pub point:[i16;2],pub unit:Option<u8>,pub failed:bool,pub blocked:Option<[i16;2]>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Effect{FinishOrder,Engage(u8),Notice(Option<u8>),CancelRange,ClearGroupTarget(i16)}
pub trait World{
 type Error;
 fn resolve(&mut self,s:&mut Frame,p:[i16;2])->Result<Resolved,Self::Error>;
 fn outside(&mut self,s:&Frame,p:[i16;2])->bool;
 fn effect(&mut self,s:&mut Frame,e:Effect)->Result<(),Self::Error>;
 fn vehicle_fire(&mut self,s:&mut Frame)->Result<bool,Self::Error>;
 fn gun_fire(&mut self,s:&mut Frame,origin:[i16;2],target:[i16;2],unsigned_speed:bool)->Result<(),Self::Error>;
 fn scatter(&mut self,origin:[i16;2],target:[i16;2],amplitude:i16)->[i16;2];
 ///58500 returns packed distance/slot, and may advance the real UNIT77.
 fn scan(&mut self,s:&mut Frame)->Result<Option<(u8,u8)>,Self::Error>;
 fn scan_rejected(&mut self,s:&Frame,slot:u8)->bool;
 fn unit_position(&mut self,slot:u8)->Result<[i16;2],Self::Error>;
}
fn clamp_dec(n:i8)->i8{let v=n.wrapping_sub(1);if v<0{0}else{v}}
fn direction(a:[i16;2],b:[i16;2])->u8{native_math::direction(i32::from(b[0])-i32::from(a[0]),i32::from(b[1])-i32::from(a[1]))}
fn center(p:[i16;2])->[i16;2]{[p[0].wrapping_add(14),p[1].wrapping_add(12)]}
pub fn scatter(origin:[i16;2],mut point:[i16;2],amplitude:i16,mut random:impl FnMut()->u32)->[i16;2]{let dx=i32::from(origin[0])-i32::from(point[0]);let dy=i32::from(origin[1])-i32::from(point[1]);let d=dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));if (((d/64)as f32).sqrt()as i32)>6{let amplitude=if amplitude==0{1}else{amplitude};let divisor=65535/i32::from(amplitude);let divisor=if divisor==0{1}else{divisor};let radius=(random()&65535)as i32/divisor;let angle=(random()&511)as i32;point[0]=point[0].wrapping_add(((native_math::sine(angle)*radius)>>10)as i16);point[1]=point[1].wrapping_add(((native_math::sine(angle+128)*radius)>>10)as i16);}point}
fn turn(s:&mut Frame,wanted:u8,turret:bool,odd:bool){if !odd{return;}let value=if turret{&mut s.turret}else{&mut s.direction};*value=if wanted.wrapping_sub(*value)&4==0{value.wrapping_add(1)&7}else{value.wrapping_sub(1)&7};s.timer3c=2;}
pub fn muzzle(s:&Frame)->[i16;2]{let d=data::GUN_MUZZLE[usize::from(s.direction&7)];[s.position[0].wrapping_add(d[0]as i16),s.position[1].wrapping_add(d[1]as i16)]}
pub fn effective_fixed_aim(aim:u8,manual:bool)->u8{if manual{aim}else{let n=aim.wrapping_add(((aim as i8)>>2)as u8);n.min(127)}}
///45b050. A successful45ae80 return is zero even when the shared shot pool
///is full. Reload is written by this caller, after the attempted release.
pub fn vehicle<W:World>(s:&mut Frame,group_timer:u8,odd:bool,w:&mut W)->Result<(),W::Error>{
 if group_timer!=0{s.flags31&=!16;s.flags64&=!8;s.flags69&=!128;w.effect(s,Effect::FinishOrder)?;return Ok(());}
 s.counter14=clamp_dec(s.counter14);s.timer3c=s.timer3c.wrapping_sub(1);if s.timer3c>=0{return Ok(());}s.timer3c=0;
 let target=w.resolve(s,s.target)?;let wanted=if !target.failed{
  if let Some(slot)=target.unit{w.effect(s,Effect::Engage(slot))?;}let d=direction(center(s.position),target.point);
  if d==s.turret{if s.reload7b!=0{return Ok(());}if !w.vehicle_fire(s)?{s.reload7b=100u8.wrapping_sub(if s.family==32{s.unit_rate}else{s.rate})as i8;}s.timer3c=0;return Ok(());}d
 }else{if s.flags64&8==0{return Ok(());}let Some(p)=target.blocked else{return Ok(());};let d=direction(s.position,p);if d==s.turret{return Ok(());}d};turn(s,wanted,true,odd);Ok(())
}
///44c3d0: two separate449470 resolutions surround44c1c0. A last-shot
///failure keeps the source timer writes and skips both scatter and allocation.
pub fn gun_target<W:World>(s:&mut Frame,odd:bool,w:&mut W)->Result<(),W::Error>{
 s.counter14=clamp_dec(s.counter14);s.timer3c=s.timer3c.wrapping_sub(1);if s.timer3c>=0{return Ok(());}s.timer3c=0;
 let r=w.resolve(s,s.target)?;if r.failed{return Ok(());}if w.outside(s,r.point)&&s.flags69&128!=0{s.flags64&=!8;s.flags31&=!16;s.flags69&=!128;w.effect(s,Effect::CancelRange)?;if s.target[0]== -1234{w.effect(s,Effect::ClearGroupTarget(s.target[1]))?;}return Ok(());}
 let wanted=direction(center(s.position),r.point);if wanted!=s.direction{turn(s,wanted,false,odd);return Ok(());}if s.reload7b!=0{return Ok(());}
 let origin=muzzle(s);let r=w.resolve(s,s.target)?;if !r.failed&&!w.outside(s,r.point){let target=[r.point[0],r.point[1].wrapping_add(8)];let aim=effective_fixed_aim(s.aim2,s.flags69&128!=0);w.effect(s,Effect::Notice(r.unit))?;let target=w.scatter(origin,target,(128-i16::from(aim))>>2);w.gun_fire(s,origin,target,false)?;s.reload7b=100u8.wrapping_sub(s.rate)as i8;}s.timer3c=0;Ok(())
}
///44c610 plus44c830. Manual69:20 consumes the raw W14/W16 point and
///44c220 dispatches each gun family separately; it never uses tank offsets.
pub fn gun<W:World>(s:&mut Frame,odd:bool,w:&mut W)->Result<(),W::Error>{
 if s.counter84!= -1{s.counter84=s.counter84.wrapping_add(1);if s.counter84!=7{return Ok(());}s.counter84= -1;}
 if s.flags69&32!=0{s.timer3d=4;s.flags69&=!32;if s.reload7b==0{s.direction=direction(center(s.position),s.target);w.gun_fire(s,muzzle(s),s.target,true)?;s.reload7b=100u8.wrapping_sub(s.rate)as i8;}return Ok(());}
 if s.flags31&16!=0||s.flags64&8!=0||s.flags69&128!=0{return gun_target(s,odd,w);}
 let mut automatic_even=false;
 if !odd{s.scan77=s.scan77.wrapping_add(1)&7;automatic_even=true;}else{
  s.timer3d=clamp_dec(s.timer3d as i8)as u8;s.timer3c=s.timer3c.wrapping_sub(1);if s.timer3c<0{if s.mask12&!(1u8.wrapping_shl(u32::from(s.direction)&31))!=0{s.direction=s.direction.wrapping_add(1)&7;if s.mask12&(1<<s.direction)==0{return Ok(());}}s.timer3c=15;}if s.timer3d!=0{return Ok(());}
 }
 let Some((distance,slot))=w.scan(s)?else{return Ok(());};if i32::from(s.range4)<i32::from(distance)*2||w.scan_rejected(s,slot){return Ok(());}if automatic_even{s.direction=s.scan77;}w.effect(s,Effect::Engage(slot))?;
 let origin=muzzle(s);let p=w.unit_position(slot)?;let target=[p[0],p[1].wrapping_add(12)];let aim=effective_fixed_aim(s.aim2,s.flags69&128!=0);let target=w.scatter(origin,target,(128-i16::from(aim))>>2);s.direction34=direction(origin,target);if s.reload7b==0{w.effect(s,Effect::Notice(Some(slot)))?;w.gun_fire(s,origin,target,false)?;s.reload7b=100u8.wrapping_sub(s.rate)as i8;}Ok(())
}
#[cfg(test)]mod tests{
 use super::*;use crate::original_hardware_point::{self as point,Shot};
 struct Fixture{scenario:usize,mode:usize,resolves:u32,rng:u32,engage:u32,notice:u32,finish:u32,apc:u32,trace:u32,tails:Vec<(u8,u8)>}
 impl Fixture{
  fn new(s:usize,mode:usize)->Self{Self{scenario:s,mode,resolves:0,rng:0,engage:0,notice:0,finish:0,apc:0,trace:2166136261,tails:Vec::new()}}
  fn random(&mut self)->u32{let r=(self.scenario as u32*7919)^(self.rng*2027)^0x7731a5c9;self.rng+=1;r}
  fn shot(&mut self,shot:Shot,kind:u8,unsigned:bool,howitzer:bool){let bad=self.scenario%5==0||self.scenario%5==1&&self.tails.len()==1;for n in[i64::from(kind),i64::from(shot.origin[0]),i64::from(shot.origin[1]),i64::from(shot.target[0]),i64::from(shot.target[1]),if unsigned{i64::from(shot.speed as u8)}else{i64::from(shot.speed)},i64::from(shot.sound),i64::from(bad)]{mix(&mut self.trace,n);}self.tails.push(if bad||kind==2{(0,0)}else{(6,if howitzer{128}else{0})});}
 }
 impl World for Fixture{type Error=();
  fn resolve(&mut self,_:&mut Frame,_:[i16;2])->Result<Resolved,()>{self.resolves+=1;let s=self.scenario;let p=[560+(s/13%3)as i16*5,612+(s/17%3)as i16*7];Ok(Resolved{point:p,unit:(s%3!=0).then_some(1),failed:s%13==0||s%13==1&&self.resolves==2||s%13==2&&self.resolves==3||s%13==3&&self.resolves==4,blocked:(s%5!=0).then_some([p[0]+40,p[1]-40])})}
  fn outside(&mut self,s:&Frame,p:[i16;2])->bool{crate::original_attack_click::out_of_range(s.position,p,s.class,s.family,[0,1,15,30,127,255][self.scenario/41%6])}
  fn effect(&mut self,_:&mut Frame,e:Effect)->Result<(),()>{match e{Effect::FinishOrder|Effect::CancelRange=>self.finish+=1,Effect::Engage(_)=>self.engage+=if self.mode==0{1}else{2},Effect::Notice(_)=>self.notice+=1,Effect::ClearGroupTarget(_)=>{}}Ok(())}
  fn scatter(&mut self,o:[i16;2],p:[i16;2],a:i16)->[i16;2]{scatter(o,p,a,||self.random())}
  fn scan(&mut self,s:&mut Frame)->Result<Option<(u8,u8)>,()>{s.scan77=s.scan77.wrapping_add(1)&7;Ok((self.scenario%11!=0).then_some(((self.scenario%31)as u8,1)))}
  fn scan_rejected(&mut self,_:&Frame,_:u8)->bool{self.scenario%19==0}
  fn unit_position(&mut self,_:u8)->Result<[i16;2],()>{Ok([560,612])}
  fn vehicle_fire(&mut self,s:&mut Frame)->Result<bool,()>{if s.family==36{return Ok(true);}if s.family==32&&self.scenario%3!=0&&matches!(s.target[0],-1234|-4321){return Ok(true);}let geometry=point::shots(point::State{family:s.family,position:s.position,point:s.target,body:s.direction,turret:s.turret,speed:s.speed,..Default::default()});let r=self.resolve(s,s.target)?;if r.failed{return Ok(true);}if s.family==32{self.apc+=1;return Ok(false);}if s.family==24{let first=r.point;for(i,mut shot)in geometry.into_iter().enumerate(){if i>0&&self.resolve(s,s.target)?.failed{return Ok(true);}let dx=shot.target[0].wrapping_sub(s.target[0]);shot.target=[first[0].wrapping_add(dx),first[1].wrapping_add(dx)];self.shot(shot,1,false,false);}return Ok(false);}if s.family==38{return Ok(false);}if s.family==34&&self.scenario%23==0{return Ok(true);}let mut shot=geometry[0];if s.family==34{self.notice+=1;}let aim=if s.flags69&128!=0{s.aim2}else{let a=s.aim2.wrapping_add(s.aim2>>2);if a>if s.family==34{127}else{128}{127}else{a}};shot.target=self.scatter(shot.origin,r.point,(128-i16::from(aim))>>2);if s.family==34{shot.sound= -1;self.shot(shot,2,false,false);}else{self.shot(shot,0,false,false);}Ok(false)}
  fn gun_fire(&mut self,s:&mut Frame,o:[i16;2],p:[i16;2],unsigned:bool)->Result<(),()>{if s.family==22{self.shot(Shot{origin:o,target:p,rocket:false,sound:-1,speed:s.speed},2,unsigned,false);}else if matches!(s.family,5|23){self.shot(Shot{origin:o,target:p,rocket:false,sound:25,speed:s.speed},0,unsigned,s.family==23);}else{let d=usize::from(s.direction&7);for(o,t)in[(data::MISSILE_A[d],data::MISSILE_TARGET_A[d]),(data::MISSILE_B[d],data::MISSILE_TARGET_B[d])]{self.shot(Shot{origin:[s.position[0]+o[0]as i16,s.position[1]+o[1]as i16],target:[p[0]+t[0]as i16,p[1]+t[1]as i16],rocket:true,sound:32,speed:s.speed},1,unsigned,false);}}Ok(())}
 }
 fn mix(h:&mut u32,n:i64){*h=(*h^n as u32).wrapping_mul(16777619);}
 #[test]fn hardware_target_and_fixed_callers_match16384_native_callbacks(){let oracle=std::env::var("NATIVE_HARDWARE_TARGET_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());let mut hash=2166136261;for mode in 0..2{for s in 0..8192usize{let mut f=Frame{family:if mode==0{[6,25,30,24,32,34,36,38][s%8]}else{[5,21,22,23][s%4]},class:if mode==0{1}else{2},flags31:4+(s/4%2*16)as u8,flags64:(s/8%2*8)as u8,flags69:[0,32,128,160][s/16%4],position:[500,600],target:[560,612],direction:(s/64%8)as u8,turret:(s/64%8)as u8,direction34:0,timer3c:[0,1,2,15,127,-128,-1][s/512%7],timer3d:(s/43%3)as u8,reload7b:[0,1,127,-128,-1][s/7%5],counter84:[-1,0,5,6,7,32767][s/29%6],counter14:[0,1,127,-128,-1][s/37%5],mask12:[255,0,1,128,85][s/31%5],scan77:(s/11%8)as u8,aim2:(s*17)as u8,range4:(s*19)as i8,unit_rate:(s*23)as u8,speed:(s*13)as i8,rate:(s*31)as u8};let mut w=Fixture::new(s,mode);if mode==0{vehicle(&mut f,if s%17==0{4}else{0},s%2!=0,&mut w).unwrap();}else{gun(&mut f,s%2!=0,&mut w).unwrap();}let mut tails=2166136261;for &(a,b)in &w.tails{mix(&mut tails,i64::from(a));mix(&mut tails,i64::from(b));}let values=[mode as i64,s as i64,i64::from(f.timer3c),i64::from(f.timer3d),i64::from(f.reload7b),i64::from(f.counter84),i64::from(f.counter14),i64::from(f.direction),i64::from(f.turret),i64::from(f.direction34),i64::from(f.flags31),i64::from(f.flags64),i64::from(f.flags69),i64::from(f.scan77),i64::from(w.resolves),i64::from(w.rng),i64::from(w.engage),i64::from(w.notice),i64::from(w.finish),i64::from(w.apc),w.tails.len()as i64,i64::from(w.trace),i64::from(tails)];if let Some(oracle)=&oracle{let v=&oracle[mode*8192+s];let names=["mode","scenario","timer","timer3d","reload","counter84","counter14","direction","turret","frame34","flags31","flags64","flags69","scan","resolves","rng","engage","notice","finish","apc","shots","trace","tails"];for(i,k)in names.into_iter().enumerate(){assert_eq!(values[i],v[k].as_i64().unwrap(),"mode{mode} scenario{s} field{k}");}}for n in values{mix(&mut hash,n);}}}if oracle.is_some(){eprintln!("hardware callers native hash={hash}");}else{assert_eq!(hash,644034610,"actual45b050/44c830/44c610/44c3d0/44c220 and44ab00; target/spatial/allocator leaves recorded explicitly");}}
}
