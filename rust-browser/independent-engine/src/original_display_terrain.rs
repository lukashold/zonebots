//!454d90's family4 prefix runs before55f10 visibility/culling.97 is a
//!signed sinking offset;454bb0 clips the picture height, rather than alpha.
use super::hardware::{Draw,Header,Layer};
use super::PHASES;
#[derive(Clone,Copy,Debug)]pub struct Input{pub position:[i16;2],pub camera:[i16;2],pub size:[i16;2],pub family:u8,pub disabled:bool,pub world:u8,pub heading:i8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct State{pub counter:i8,pub phase:u8,pub frame:i16,pub counter82:i16,pub counter84:i16,pub transition:u16,pub cycles:u16,pub timer:u8,pub flags64:u8}
pub trait World{fn terrain(&mut self,cell:i16)->u8;fn random(&mut self)->u32;fn remove_forced(&mut self);fn ambient(&mut self,position:[i16;2]);}
pub enum Outcome{Regular,Handled(Option<Draw>),Unavailable}
//499be0 is indexed by the masked terrain byte. Keep the entire addressable
//range: negative/signed97 comparisons are not replaced with a float depth.
pub const LEVEL:[u8;128]=[0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,11,0,0,16,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,0,0,0,80,0,0,0,240,0,1,0,80,161,67,0,64,137,67,0,64,137,67,0,0,0,0,0,80,0,199,0,240,0,200,0,128,161,67,0,64,137,67,0,64,137,67,0,0,0,0,0,0,0,50,0,1,0,150,0];
pub fn cell(p:[i16;2])->i16{(p[1].wrapping_add(11).wrapping_shl(3)&!127i16).wrapping_add(p[0].wrapping_add(5)>>4)}
fn water(code:u8)->bool{matches!(code&127,9|38|39|42)}
pub fn advance(i:Input,s:&mut State,w:&mut impl World)->Outcome{
 if i.family!=4||i.disabled{return Outcome::Regular;}
 let at=cell(i.position);let target=LEVEL[usize::from(w.terrain(at)&127)];
 if i16::from(target)<i16::from(s.counter){s.counter=s.counter.wrapping_sub(1);}else if i16::from(target)>i16::from(s.counter){s.counter=s.counter.wrapping_add(1);}
 if s.counter>=16{if w.random()as u8&63==0{w.ambient([i.position[0],i.position[1].wrapping_sub(16)]);}return Outcome::Handled(None);}
 if s.counter==0{return Outcome::Regular;}
 if i.world==3{w.remove_forced();return Outcome::Handled(None);}
 let modulus=if matches!(s.phase,0|1){1023}else{4095};let r=w.random()as u16;
 if u32::from(r)%modulus+1==2&&s.phase!=1&&matches!(i.world,0|1){
  let submerged=[0i16,1,-1].into_iter().all(|dx|[0i16,128,256,384].into_iter().all(|dy|water(w.terrain(at.wrapping_add(dx).wrapping_add(dy)))));
  if submerged{s.frame=0;s.counter82=0;s.transition=0x443;s.phase=42;s.counter84=-1;s.cycles=((w.random()as u16)%3)+5;s.timer=(w.random()as u8&1)+3;}
 }
 let screen=[i.position[0].wrapping_sub(i.camera[0]),i.position[1].wrapping_sub(i.camera[1])];
 if screen[0]<=-32||screen[0]>=i.size[0]||screen[1]<=-48||screen[1]>=i.size[1]{return Outcome::Handled(None);}
 let Some(&[base,stride])=PHASES.get(usize::from(s.phase))else{return Outcome::Unavailable;};
 let y=screen[1].wrapping_add(i16::from(s.counter));let sprite=base.wrapping_add(i16::from(i.heading).wrapping_mul(stride)).wrapping_add(s.frame);let palette=s.flags64&7;s.flags64&=!4;
 Outcome::Handled(Some(Draw{header:Header{x:None,y,sprite:0,palette:255,offset:-1},layers:vec![Layer{position:[screen[0],y],sprite,palette,flags:3,clip:None,parameter8:Some(16u8.wrapping_sub(s.counter as u8))}],flags64:s.flags64,mask98:0}))
}
#[cfg(test)]mod tests{
 use super::*;
 struct Env{s:usize,calls:u32,at:i16,code:u8,mode:usize,removed:u32,ambient:Vec<[i16;2]>}
 impl World for Env{
  fn terrain(&mut self,cell:i16)->u8{if cell==self.at{return self.code;}let j=i32::from(cell)+16384;if !(0..32768).contains(&j){return 0;}match self.mode{0=>9,1=>0,_=>if j%3==0{9}else{38}}}
  fn random(&mut self)->u32{let c=self.calls;self.calls+=1;if c==0{[0,1,1024,4096,63,64,65535,0xffff0001][self.s/11%8]}else{(self.s as u32*7919)^(c*2027)^0x7731a5c9}}
  fn remove_forced(&mut self){self.removed+=1;}
  fn ambient(&mut self,p:[i16;2]){self.ambient.push(p);}
 }
 fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
 fn hash(b:impl IntoIterator<Item=u8>)->u32{let mut h=2166136261;mix(&mut h,b.into_iter().map(u32::from));h}
 #[test]fn terrain_prefix_matches_8192_executed_original_submissions(){
  let csv=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../work/parity-audit/native-infantry-terrain.csv");let csv=std::fs::read_to_string(csv).ok();let rows=csv.as_ref().map(|s|s.lines().map(|l|l.split(',').collect::<Vec<_>>()).collect::<Vec<_>>());let mut all=2166136261;let mut raster=2166136261;let(mut draws,mut removes,mut spawns)=(0,0,0);
  for n in 0..8192usize{
   let position:[i16;2]=[[500,600],[-33,-49],[7,8],[32760,-32768],[8,9]][n/71%5];let camera=[if n/73%2==0{40}else{position[0].wrapping_add(33)},if n/79%2==0{60}else{position[1].wrapping_add(49)}];
   let i=Input{position,camera,size:[1024,1024],family:if n/43%7==0{6}else{4},disabled:n/41%2!=0,world:((n/1024%5)+if n/37%2!=0{2}else{0})as u8%5,heading:(n/47%8)as i8};
   let mut s=State{counter:[-128,-17,-2,-1,0,1,3,6,7,10,11,14,15,16,17,127][n/8%16],phase:[0,1,2,3,19,23,42,65][n/128%8],frame:(n/59%3)as i16,counter82:55,counter84:66,transition:77,cycles:0x3412,timer:88,flags64:[0,1,4,5,16,21][n/53%6]};
   let mut w=Env{s:n,calls:0,at:cell(position),code:[0,9,38,39,42,137,166,167][n%8],mode:n/67%3,removed:0,ambient:vec![]};let result=advance(i,&mut s,&mut w);let mut header=[0u8;8];let old_x=(n as i16).wrapping_mul(997);header[..2].copy_from_slice(&old_x.to_le_bytes());let old=std::array::from_fn(|j|(n*13+j*17)as u8);let mut extra=old;let mut q=0;let mut e=0;let consumed=match result{Outcome::Regular=>0,Outcome::Handled(draw)=>{if let Some(mut d)=draw{let l=d.layers[0];let rect=l.rectangle(0).unwrap();let p=l.palette&15;mix(&mut raster,[n as u32,l.position[0]as u32,l.position[1]as u32,l.sprite as u32,if p<4{u32::from(p)}else{255},rect[0]as u32,rect[1]as u32,rect[2]as u32,rect[3]as u32]);d.header.sprite=(n/61%4)as i16;header=d.header.bytes_with_previous_x(old_x);extra=d.layers[0].bytes_with_previous(old);q=8;e=16;draws+=1;}1},Outcome::Unavailable=>panic!("unexpected phase boundary{n}")};
   let mut values=vec![consumed,s.counter as u32,u32::from(s.phase),s.frame as u32,s.counter82 as u32,s.counter84 as u32,u32::from(s.transition),u32::from(s.cycles),u32::from(s.timer),u32::from(s.flags64),w.calls,w.removed,w.ambient.len()as u32,q,e,hash(header),hash(extra)];
   if let Some(p)=w.ambient.first(){values.extend([p[0]as u32,p[1]as u32,25,0,2,u32::MAX,u32::MAX,u32::MAX,0,0,16]);}
   mix(&mut all,values);removes+=w.removed;spawns+=w.ambient.len();
   if let Some(rows)=&rows{let row=&rows[n];let hex=|b:&[u8]|b.iter().map(|v|format!("{v:02x}")).collect::<String>();assert_eq!(row[19].parse::<i8>().unwrap(),s.counter,"counter{n}");assert_eq!(row[28].parse::<u32>().unwrap(),w.calls,"rng{n}");assert_eq!(row[33],hex(&header),"header{n}");assert_eq!(row[34],hex(&extra),"extra{n}");}
  }
  assert_eq!((draws,removes,spawns),(132,524,238));assert_eq!(raster,2968198360,"132 actual4549f0/454630 native raster submissions, including signed heights");assert_eq!(all,1816197881,"original454d90/454d00/454d40/454bb0/459460/4593f0; explicit RNG, removal and ambient-constructor leaf inputs");
 }
}
