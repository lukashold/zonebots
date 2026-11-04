//!41d280's64 six-byte records,41d110's animation/cutscene gate and the
//!41cfa0 fort banners. Capture rebuilds clear transition5; no extra lowering
//!or raising writer exists in this executable's flag-record access inventory.
use crate::original_display::Record;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Flag{pub position:[i16;2],pub color:u8,pub transition5:u8}
impl Default for Flag{fn default()->Self{Self{position:[-1,0],color:0,transition5:0}}}
impl Flag{pub fn bytes(self)->[u8;6]{let mut b=[0;6];b[..2].copy_from_slice(&self.position[0].to_le_bytes());b[2..4].copy_from_slice(&self.position[1].to_le_bytes());b[4]=self.color;b[5]=self.transition5;b}}
#[derive(Clone,Debug)]pub struct Pool{pub records:[Flag;64]}
impl Default for Pool{fn default()->Self{Self{records:[Flag::default();64]}}}
pub struct View{pub camera:[i16;2],pub size:[i16;2],pub frame:i16,pub mode:i32,pub forts:[Option<[i16;2]>;2],pub fort_visible:[u8;2]}
fn bounds(p:[i16;2],camera:[i16;2],size:[i16;2],left:i16)->bool{p[0]>=camera[0].wrapping_sub(left)&&p[1]>=camera[1].wrapping_sub(16)&&p[0]<=camera[0].wrapping_add(size[0])&&p[1]<=camera[1].wrapping_add(size[1])}
impl Pool{
 pub fn retire(&mut self){for f in &mut self.records{f.position[0]=-1;}}
 ///41d280 retires X only before copying active CP flags in native slotorder.
 ///Every active record receives current owner color and transition5=0.
 pub fn rebuild(&mut self,seeds:&[([i16;2],u8)]){self.retire();for(f,&(position,color))in self.records.iter_mut().zip(seeds){*f=Flag{position,color,transition5:0};}}
 pub fn advance(&mut self,view:View)->Vec<Record>{let mut rows=Vec::new();self.submit(view,|r,_,_|rows.push(r));rows}
 ///Source0/1 are flag cloth/shadow;2/3 are fort cloth/shadow. Slot stays
 ///the original flag-pool index or fort-owner index, independent of Web IDs.
 pub fn submit(&mut self,view:View,mut emit:impl FnMut(Record,u8,usize)){
  if view.mode==0{for(slot,f)in self.records.iter_mut().enumerate(){if f.position[0]==-1{continue;}if f.transition5!=0&&f.transition5!=255{f.transition5=f.transition5.wrapping_add(1);if matches!(f.transition5,11|18){f.transition5=255;}}
   if !bounds(f.position,view.camera,view.size,16){continue;}let sprite=if matches!(f.transition5,0|255){480i16.wrapping_add(view.frame)}else{3176i16.wrapping_add(i16::from(f.transition5 as i8))};let color=if(f.color as i8)>3{f.color.wrapping_add(4)}else{f.color};emit(Record{x:f.position[0],y:f.position[1].wrapping_add(32),sprite,palette:color,offset:-32},0,slot);emit(Record{x:f.position[0],y:f.position[1].wrapping_sub(16),sprite:sprite.wrapping_add(4),palette:color,offset:16},1,slot);
  }}
  if view.mode<16{for(owner,base,sprite)in[(0,view.forts[1],1234i16),(1,view.forts[0],1238i16)]{let Some(base)=base else{continue;};let p=[base[0].wrapping_add(83),base[1].wrapping_add(27)];if view.fort_visible[owner]==0||!bounds(p,view.camera,view.size,32){continue;}emit(Record{x:p[0],y:p[1],sprite:sprite.wrapping_add(view.frame),palette:0,offset:0},2,owner);emit(Record{x:p[0],y:p[1].wrapping_sub(35),sprite:1242i16.wrapping_add(view.frame),palette:0,offset:48},3,owner);}}
 }
}
#[cfg(test)]mod tests{
 use super::*;fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn all_8192_original_flag_and_fort_submissions_match_modes_signed_colors_transition_bytes_and_bounds(){let mut hash=2166136261;for s in 0..8192usize{let transition=(s&255)as u8;let color=[0,1,3,4,7,127,128,255u8][s/256%8];let frame=(s/2048%4)as i16;let mode=[0,1,15,16,-1][s*7%5];let x=[-17,-16,500,1024,1025][s*3%5];let y=[-17,-16,500,1024,1025][(s*2+1)%5];let a=[0,2,255][s/7%3];let b=[0,2,255][s/11%3];let mut pool=Pool::default();pool.records[0]=Flag{position:[x,y],color,transition5:transition};let rows=pool.advance(View{camera:[0,0],size:[1024,1024],frame,mode,forts:[Some([400,200]),Some([400,700])],fort_visible:[a,b]});let size=rows.len()*8;let mut bytes=2166136261;mix(&mut bytes,rows.into_iter().flat_map(|r|r.bytes()).map(u32::from));mix(&mut hash,[s as u32,u32::from(transition),u32::from(color),frame as u32,mode as u32,x as u32,y as u32,u32::from(a),u32::from(b),u32::from(pool.records[0].transition5),size as u32,bytes]);}assert_eq!(hash,1122415333,"actual41d110→41cfa0, including offscreen transition updates and signed color behavior");}
 #[test]fn all_512_original_capture_rebuild_pool_images_preserve_inactive_bytes_and_reset_active_transitions(){let mut hash=2166136261;for s in 0..512usize{let mut pool=Pool::default();for(i,f)in pool.records.iter_mut().enumerate(){let bytes:[u8;6]=std::array::from_fn(|j|((i*6+j)*17+s)as u8);*f=Flag{position:[i16::from_le_bytes([bytes[0],bytes[1]]),i16::from_le_bytes([bytes[2],bytes[3]])],color:bytes[4],transition5:bytes[5]};}let seeds=(0..64).filter(|&i|i<s%65&&((i+s)%7)&3==0).map(|i|([((i*7+s)%64*16)as i16,((i*13+s)%64*16)as i16],((i+s)%5)as u8)).collect::<Vec<_>>();pool.rebuild(&seeds);let mut bytes=2166136261;mix(&mut bytes,pool.records.into_iter().flat_map(|f|f.bytes()).map(u32::from));mix(&mut hash,[s as u32,bytes]);}assert_eq!(hash,2526159676,"actual41d280 with4a6e6e=1 capture rebuild; initial OBJECT creation remains owned by world import");}
}
