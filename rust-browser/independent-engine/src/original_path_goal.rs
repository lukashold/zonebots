//!43bdd0's branch order and output-pointer rules. Lookups keep native slot
//!identity and do not substitute a rendered route endpoint.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Input{pub slot:u8,pub position:[i16;2],pub class:u8,pub role:i8,pub group:u8,pub flags31:u8,pub flags8a:u8,pub flags64:u8,pub flags9c:u8,pub object36:u8,pub contact5d:u8,pub target50:i32,pub point54:[i16;2],pub primary:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Goal{pub position:[i16;2],pub node:u32,pub failed:bool}
impl Default for Goal{fn default()->Self{Self{position:[0;2],node:u32::MAX,failed:true}}}
pub trait World{type Error;fn object(&mut self,slot:u8)->Result<[i16;2],Self::Error>;fn leader(&mut self,group:usize)->Result<u8,Self::Error>;fn unit(&mut self,slot:usize)->Result<[i16;2],Self::Error>;fn nearest_group_member(&mut self,group:i16)->Result<Option<[i16;2]>,Self::Error>;fn target(&mut self)->Result<Option<[i16;2]>,Self::Error>;fn node(&mut self,link:u32)->Result<[i16;2],Self::Error>;}
pub fn resolve<W:World>(input:&mut Input,world:&mut W)->Result<Goal,W::Error>{let mut out=Goal::default();
 if input.flags31&8!=0{out.position=world.object(input.object36)?;out.failed=out.position[0]== -1;if out.failed{input.contact5d=0;input.flags31&=!8;}return Ok(out);}
 if input.flags8a&12==0{
  if input.role>0{let leader=world.leader(usize::from(input.group))?;out.position=world.unit(usize::from(leader))?;out.failed=false;return Ok(out);}
  if input.flags31&16!=0{
   if input.target50== -4321{out.position=world.unit(input.point54[0]as u16 as usize)?;out.failed=false;return Ok(out);}
   if input.target50== -1234&&input.primary==0{if let Some(p)=world.nearest_group_member(input.point54[0])?{out.position=p;out.failed=false;}return Ok(out);}
   if matches!(input.primary,0|u32::MAX){return Ok(out);}out.position=world.node(input.primary)?;out.failed=false;return Ok(out);
  }
  if input.flags64&8!=0{
   if input.flags9c&128!=0{out.node=input.primary;if input.primary!=0{if input.primary!=u32::MAX{out.position=world.node(input.primary)?;out.failed=false;}return Ok(out);}}
   if input.target50==3456{out.position=input.point54;out.failed=false;return Ok(out);}
   if input.primary==0{if let Some(p)=world.target()?{out.position=p;out.failed=false;}return Ok(out);}
  }
 }
 out.node=input.primary;if matches!(input.primary,0|u32::MAX){input.flags31&=!64;return Ok(out);}out.position=world.node(input.primary)?;out.failed=false;Ok(out)
}
#[cfg(test)]mod tests{
 use super::*;
 struct Fixture{s:usize,input:Input}
 impl Fixture{fn position(&self,n:usize)->[i16;2]{[(self.s as i32*353+n as i32*47-32000)as i16,(self.s as i32*229-n as i32*61+30000)as i16]}fn flags(&self,n:usize)->u8{if n==0{self.input.flags31}else if(self.s+n)%5==0{0}else{4+((self.s+n)%4)as u8}}fn leader(group:usize)->u8{if group==1{3}else if group==2{4}else{0}}}
 impl World for Fixture{type Error=();fn object(&mut self,n:u8)->Result<[i16;2],()>{Ok([if self.s%7==0{-1}else{100+i16::from(n)*31},-200-i16::from(n)*43])}fn leader(&mut self,g:usize)->Result<u8,()>{Ok(Self::leader(g))}fn unit(&mut self,n:usize)->Result<[i16;2],()>{Ok(self.position(n))}fn nearest_group_member(&mut self,g:i16)->Result<Option<[i16;2]>,()>{let mut best=i32::MAX;let mut point=None;for role in 0..21{let n=if role==0{usize::from(Self::leader(g as usize))}else{role};if role!=0&&(self.s+role)%13==0||n==0||self.flags(n)&4==0{continue;}let p=self.position(n);let d=[i32::from(p[0])-i32::from(self.input.position[0]),i32::from(p[1])-i32::from(self.input.position[1])];let distance=d[0].wrapping_mul(d[0]).wrapping_add(d[1].wrapping_mul(d[1]));if distance<best{best=distance;point=Some(p);}}Ok(point)}fn target(&mut self)->Result<Option<[i16;2]>,()>{let input=self.input;let n=if input.class==1{match input.target50{-1234=>usize::from(Self::leader(input.point54[0]as usize)),-4321=>input.point54[0]as usize,_=>return Ok(None)}}else{if input.target50==3456{return Ok(None);}if input.target50<0{usize::from(Self::leader(usize::from(input.target50.wrapping_neg()as u8)-1))}else{input.target50 as u8 as usize}};let valid=self.flags(n)&4!=0&&(self.flags(n)^input.flags31)&3!=0&&(self.s+n)%7!=0;Ok(valid.then(||self.position(n)))}fn node(&mut self,_:u32)->Result<[i16;2],()>{Ok([(self.s as i32*17-25000)as i16,(self.s as i32*19+22000)as i16])}}
 #[test]fn goal_branches_and_actual_nearest_and_target_selectors_match4096_full_native_calls(){let mut hash=2166136261u32;for s in 0..4096usize{let mut input=Input{slot:0,position:[(s as i32*353-32000)as i16,(s as i32*229+30000)as i16],flags31:[0,8,16,64,24,80,72,88][s%8],flags8a:[0,4,8,12][s/8%4],role:[-1,0,1][s/32%3],group:1,flags64:(s/96%2*8)as u8,flags9c:(s/192%2*128)as u8,object36:(s%5)as u8,contact5d:7,class:[0,1,2,15][s/384%4],target50:[-1234,-4321,3456,1,-2,777][s/7%6],point54:[2,-345],primary:[0,u32::MAX,0x5eee28][s/5%3]};let mut world=Fixture{s,input};let out=resolve(&mut input,&mut world).unwrap();if std::env::var_os("Z_NATIVE_PATH_GOAL_DUMP").is_some(){println!("{} {} {} {} {} {} {}",s,u8::from(out.failed),out.position[0],out.position[1],out.node,input.flags31,input.contact5d);}for n in[s as u32,out.failed as u32,out.position[0]as i32 as u32,out.position[1]as i32 as u32,out.node,u32::from(input.flags31),u32::from(input.contact5d)]{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,3621350079,"full43bdd0,566d0 and448470 without stubs");}
}
