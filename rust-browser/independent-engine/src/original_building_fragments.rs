//!41f8f0 and41f610: per-building50x18 damage animation records. Drawing,
//!environment contact and sparks are ordered leaves, rather than explosions
//!invented from the building's current HP by the browser renderer.
use crate::original_display::Record as Draw;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Record{pub bytes:[u8;18]}
impl Default for Record{fn default()->Self{let mut r=Self{bytes:[0;18]};r.set(0,-1);r}}
impl Record{pub fn word(&self,n:usize)->i16{i16::from_le_bytes([self.bytes[n],self.bytes[n+1]])}pub fn set(&mut self,n:usize,v:i16){self.bytes[n..n+2].copy_from_slice(&v.to_le_bytes());}}
#[derive(Clone,Debug)]pub struct Pool{pub records:[Record;50]}
impl Default for Pool{fn default()->Self{Self{records:[Record::default();50]}}}
#[derive(Clone,Copy,Debug)]pub struct View{pub camera:[i16;2],pub size:[i16;2],pub multiplayer:bool}
pub trait World{fn contact_mode(&mut self,enabled:bool);fn select_record(&mut self,_slot:usize){}fn submit(&mut self,record:Draw);fn environment(&mut self,point:[i16;2]);fn spark(&mut self,point:[i16;2]);fn crt(&mut self)->u16;}
impl Pool{
 ///40b070 clears only the first signed word of each record, retaining the
 ///warm bytes that41f8f0 does not overwrite when it allocates that record.
 pub fn clear(&mut self){for r in &mut self.records{r.set(0,-1);}}
 ///Templates contain signed x/y, sprite, HP threshold, frame limit and delay.
 ///An x=-1 sentinel terminates the list, including when the pool is full.
 ///The source keeps the FIRST template frame limit/delay for all entries.
 pub fn construct(&mut self,position:[i16;2],templates:&[[i16;6]],factor:i16){
  let Some(header)=templates.first()else{return;};for t in templates{if t[0]==-1{break;}let Some(r)=self.records.iter_mut().find(|r|r.word(0)==-1)else{continue;};r.set(0,position[0].wrapping_add(t[0]));r.set(2,position[1].wrapping_add(t[1]).wrapping_sub(32));r.bytes[14]=u8::from(t[2]<0);r.set(8,t[2].wrapping_abs());r.set(10,if t[3]==-2{-2}else{t[3].wrapping_mul(factor)});r.bytes[13]=header[4]as u8;r.bytes[16]=header[5]as u8;r.bytes[15]=header[5]as u8;r.bytes[12]=0;}
 }
 ///40e790 searches the current pointer only up to the first inactive record.
 ///Its successful transition selects a subrecord; it does not rewrite bytes.
 pub fn destroyed_start(&self,start:usize)->usize{for(i,r)in self.records.iter().enumerate().skip(start).take(50){if r.word(0)==-1{break;}if r.word(10)==-2{return i;}}start}
 pub fn advance(&mut self,damage:i16,view:View,world:&mut impl World){self.advance_from(0,damage,view,world);}
 ///The local record counter starts at zero at the actual incoming pointer.
 pub fn advance_from(&mut self,start:usize,damage:i16,view:View,world:&mut impl World){
  for(i,r)in self.records.iter_mut().skip(start).enumerate(){world.select_record(start+i);let p=[r.word(0),r.word(2)];if p[0]==-1{return;}let flags=r.bytes[14];if flags&4!=0{continue;}
   if flags&2!=0{let screen=[p[0].wrapping_sub(view.camera[0]),p[1].wrapping_sub(view.camera[1])];let sprite=r.word(8).wrapping_add(i16::from(r.bytes[12]as i8));if screen[0]>-32&&screen[0]<=view.size[0]&&screen[1]>-32&&screen[1]<=view.size[1]{world.submit(Draw{x:screen[0],y:screen[1].wrapping_sub(100),sprite,palette:0,offset:100});}
    if sprite==0x6f6&&(view.multiplayer||world.crt()&31==0){world.spark([p[0].wrapping_add(5),p[1]]);}
    r.bytes[15]=r.bytes[15].wrapping_sub(1);if r.bytes[15]==0{r.bytes[12]=r.bytes[12].wrapping_add(1);r.bytes[15]=r.bytes[16];if r.bytes[12]==r.bytes[13]{if flags&1==0{r.bytes[14]|=4;}else{r.bytes[12]=0;}}}
   }else{let original=r.word(10);if original==-2&&damage!=-2{continue;}let(mut threshold,mut trigger)=(original,if damage==-2{0}else{i as i16});if threshold<0{threshold=threshold.wrapping_neg();trigger=1;}r.bytes[14]&=!2;if threshold<=damage{r.bytes[14]|=2;if trigger==1{world.contact_mode(true);if p[0]>=-32{world.environment(p);}world.contact_mode(false);if original!=-2{world.spark(p);}}}}
  }
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}
 fn hash(bytes:impl IntoIterator<Item=u8>)->u32{let mut h=2166136261;for b in bytes{mix(&mut h,u32::from(b));}h}
 struct Env{scenario:usize,crt:u32,calls:u32,event:u32,flag:u8,draws:Vec<Draw>}
 impl World for Env{
  fn contact_mode(&mut self,active:bool){self.flag=u8::from(active);}
  fn submit(&mut self,r:Draw){self.draws.push(r);}
  fn environment(&mut self,p:[i16;2]){self.calls+=1;for n in [0x420200,p[0]as u32,p[1]as u32,u32::from(self.flag),0,0]{mix(&mut self.event,n);}}
  fn spark(&mut self,p:[i16;2]){self.calls+=1;for n in [0x41dbf0,p[0]as u32,p[1]as u32]{mix(&mut self.event,n);}}
  fn crt(&mut self)->u16{let value=(self.scenario as u32*3571+self.crt*131+0x1743)&32767;self.crt+=1;value as u16}
 }
 #[test]fn actual_native_fragment_constructors_and_four_callback_sequences_match(){
  let mut all=2166136261;
  for mode in 0..2{for s in 0..1024usize{let mut pool=Pool::default();for(i,r)in pool.records.iter_mut().enumerate(){for(j,b)in r.bytes.iter_mut().enumerate(){*b=(s*13+i*17+j*29)as u8;}}
   if mode==0{let active=[0,1,17,49,50][s%5];for(i,r)in pool.records.iter_mut().enumerate(){r.set(0,if i<active{100+i as i16}else{-1});}}
   else{let count=[1,2,17,49,50][s%5];for(i,r)in pool.records.iter_mut().enumerate(){r.set(0,if i==count{-1}else{[-33,-32,-31,0,400,800,32767,-32768][(s+i)%8]});r.set(2,[-33,-32,-31,0,300,600,32767,-32768][(s/3+i)%8]);r.set(8,[1782,1778,100,32767,-32768][(s+i)%5]);r.set(10,[-32768,-2,-1,0,1,100,32767][(s/5+i)%7]);r.bytes[12]=[-128i8,-1,0,1,4,127][(s/7+i)%6]as u8;r.bytes[13]=[0,1,3,255][(s/11+i)%4];r.bytes[14]=((s+i*3)&7)as u8;r.bytes[15]=[0,1,2,127,255][(s/13+i)%5];r.bytes[16]=[0,1,3,255][(s/17+i)%4];}}
   let templates=(0..13usize).map(|i|[if i==12{-1}else{i as i16*7-15},i as i16*11-35,if i%2==0{1782}else{-1782},[-32768,-2,-1,0,1,100,32767][(s+i)%7],(i+s)as i16,(i*3+s)as i16]).collect::<Vec<_>>();let mut e=Env{scenario:s,crt:0,calls:0,event:2166136261,flag:13,draws:vec![]};
   for tick in 0..if mode==0{1}else{4}{e.calls=0;e.event=2166136261;e.flag=13;e.draws.clear();if mode==0{pool.construct([(s*71)as i16,(s*137)as i16],&templates,[-32768,-1,0,1,300,32767][s%6]);}else{pool.advance([-32768,-2,-1,0,1,10,100,32767][(s/19+tick)%8],View{camera:if s%3==0{[500,300]}else{[0,0]},size:[800,600],multiplayer:s%2!=0},&mut e);}
    let fields=[mode,s as u32,tick as u32,hash(pool.records.iter().flat_map(|r|r.bytes)),e.draws.len()as u32,hash(e.draws.iter().flat_map(|r|r.bytes())),e.crt,e.calls,e.event,u32::from(e.flag)];if std::env::var_os("Z_FRAGMENT_TRACE").is_some(){println!("FRAGMENT {fields:?}");}for n in fields{mix(&mut all,n);}
   }
  }}
  assert_eq!(all,1413238458,"1024 native constructors plus4096 sequential41f610 callbacks; explicit recorded420200/41dbf0 leaves");
 }
}
