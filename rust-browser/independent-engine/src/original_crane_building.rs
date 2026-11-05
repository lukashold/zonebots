//!44fcc0 ordinary BUILD branch. Movement/contact/artwork have ordered inputs;
//! the work/arrival state is never inferred from a Web distance or HP timer.
#[derive(Clone,Debug)]pub struct Crane{pub entity:usize,pub target:Option<usize>,pub work9a:u8,pub work9a_known:bool,pub frame86:i8,pub mouth96:u8,pub interval87:i8,pub delay82:i16,pub saved:[i16;2],pub approach_b1:Option<u8>}
impl Default for Crane{fn default()->Self{Self{entity:0,target:None,work9a:0,work9a_known:false,frame86:-1,mouth96:0,interval87:0,delay82:0,saved:[0;2],approach_b1:None}}}
/// The two actual UNIT96 stores in44fcc0, ordered independently of DTO storage.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Write96{Mouth{before:u8,after:u8},CompletionZero}
impl Write96{
 pub fn value(self)->u8{match self{Self::Mouth{after,..}=>after,Self::CompletionZero=>0}}
 pub fn address(self)->u32{match self{Self::Mouth{..}=>0x44fe18,Self::CompletionZero=>0x44fe77}}
}
pub trait World{
 fn overlay(&mut self,c:&Crane)->Option<bool>;
 fn crt(&mut self)->u16;
 fn common(&mut self,c:&mut Crane)->bool;
 fn check_path(&mut self)->u8;
 fn job(&mut self,c:&Crane);
 fn progress(&mut self,c:&mut Crane)->bool;
 fn prepend(&mut self,c:&Crane)->bool;
 fn start_path(&mut self)->bool;
 fn mark_moving(&mut self);
 fn cleanup(&mut self);
 fn write96(&mut self,_effect:Write96){}
}
pub fn tick(c:&mut Crane,w:&mut impl World,multiplayer:bool,reverse_wait:bool,mouth_suppressed:bool){
 if c.frame86!=-1&&c.frame86!=-2{let Some(terminal)=w.overlay(c)else{return;};if c.frame86 as u8&64==0{if !terminal{c.frame86=c.frame86.wrapping_add(1);}}else if !reverse_wait{let n=(c.frame86 as u8&63).wrapping_sub(1);c.frame86=(if n==255{254}else{n}|64)as i8;}}
 let before96=c.mouth96;let mut low=before96&15;let mut high=(before96 as i8>>4)as u8&15;let r=if multiplayer{0}else{w.crt()};if r&63>3{high=high.wrapping_sub(1)&1;}if !mouth_suppressed{if low<4{low=low.wrapping_add(1)&7;if low>=3{let r=if multiplayer{0}else{w.crt()};if low==3{if r&7!=0{low=1;}}else if r&1!=0{low=5;}}}else{low=4;}}c.mouth96=(high<<4)|low;w.write96(Write96::Mouth{before:before96,after:c.mouth96});
 //Common can change9a through actual contact. Dispatch from that returned
 //current value; the prefix's initial phase is not a completion permission.
 if !w.common(c){return;}
 if c.work9a==254{if w.check_path()&1!=0{return;}c.frame86=-1;c.mouth96=0;w.write96(Write96::CompletionZero);w.job(c);c.work9a=0;w.cleanup();return;}
 if c.work9a!=255&&c.work9a!=0{if c.work9a&64==0{c.work9a=c.work9a.wrapping_add(1);if c.work9a==7{c.work9a=71;}}else{let n=(c.work9a&63).wrapping_sub(1);c.work9a=if n==0{1}else{n|64};}}
 if c.work9a==255||c.work9a==0||c.target.is_none(){return;}
 if !w.progress(c){return;}
 if w.prepend(c)&&!w.start_path(){w.mark_moving();c.work9a=254;return;}
 c.work9a=0;w.cleanup();
}

#[cfg(test)]#[path="original_crane_building_tests.rs"]mod tests;

///5c5c0's body fields; its four ordered callee bodies remain separately owned.
pub fn body_w4(byte:u8)->u8{(byte&240)|(((byte as i8>>4)as i16+i16::from(byte&15)).clamp(0,2)as u8)}
pub fn counter68(byte:i8)->i8{byte.wrapping_sub(1).max(0)}
