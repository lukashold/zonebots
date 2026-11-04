//!Radar40cb20/40c9b0: retained BUILDword0A, flags6 and byte15. The dish
//!record is submitted before its health/pulse clock advances the next frame.
use crate::original_display::Record;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Radar{pub word0a:i16,pub flags6:u8,pub notice15:u8}
#[derive(Clone,Copy,Debug)]pub struct Input{pub position:[i16;2],pub maximum:i16,pub hp:i16,pub owner:i8,pub local_team:i8,pub pulse:bool,pub camera:[i16;2],pub size:[i16;2]}
#[derive(Clone,Copy,Debug,Default)]pub struct Result{pub record:Option<Record>,pub sweep:bool,pub secured:bool}
pub const DISH_Y:[i8;8]=[15,14,6,2,1,2,6,13];
///40adfa decrements the signed word36 before calling the BUILD renderer.
///DEC wraps first; JNS then clamps a negative result to zero.
pub fn completion_at_draw(value:i16)->i16{value.wrapping_sub(1).max(0)}
impl Radar{
 ///40c820 resets word0A, clears6:20 and preserves notice15. Disk loading
 ///copied word0A directly; ordinary world sync is not a constructor call.
 pub fn construct(&mut self){self.word0a=0;self.flags6&=!32;}
 ///Only the listener-owned acquisition bit/countdown, without a dish/BUILD tick.
 pub fn advance_notice(&mut self,owner:i8,local_team:i8)->bool{let mut secured=false;if owner!=4{if owner==local_team{if self.flags6&32==0{self.flags6|=32;self.notice15=50;}else if self.notice15!=0{self.notice15=self.notice15.wrapping_sub(1);secured=self.notice15==0;}}else{self.flags6&=!32;}}secured}
 pub fn advance(&mut self,input:Input)->Result{
  let mut out=Result::default();out.secured=self.advance_notice(input.owner,input.local_team);
  let screen=[input.position[0].wrapping_sub(input.camera[0]),input.position[1].wrapping_sub(input.camera[1])];if screen[0]<=-64||screen[0]>input.size[0]||screen[1]<=-48||screen[1]>input.size[1]||input.hp==0{return out;}
  //Fresh40c820 sets0 and40c9b0 masks every subsequent increment to0..7.
  //Do not invent a frame for malformed/stale records outside that domain.
  let Ok(frame)=usize::try_from(self.word0a)else{return out;};let Some(&dy)=DISH_Y.get(frame)else{return out;};out.record=Some(Record{x:input.position[0].wrapping_add(16),y:input.position[1].wrapping_add(i16::from(dy)).wrapping_add(32),sprite:self.word0a.wrapping_add(1488),palette:0,offset:-50});
  let threshold=i32::from(input.maximum)-(i32::from(input.maximum)/5)*4;if input.owner<4&&input.hp>threshold as i16&&input.pulse{self.word0a=self.word0a.wrapping_add(1)&7;out.sweep=self.word0a==0&&input.owner==input.local_team;}out
 }
}
#[cfg(test)]mod tests{
 use super::*;fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn all_1024_executed_main_build_prefixes_decrement_hold_before_the_callback(){let mut hash=2166136261u32;for s in 0..1024u32{let value=if s<10{[0,1,2,68,32767,-32768,-1,-2,-32767,65535][s as usize]as i16}else{s.wrapping_mul(997)as i16};let after=completion_at_draw(value);mix(&mut hash,[s,s%35,value as u32,1,after as u32,after as u32]);}assert_eq!(hash,3044181778,"actual40ad60 35-record loop; work/rate/bulk helpers and per-BUILD callback are explicit boundaries");}
 #[test]fn all_4096_original_radar_caller_records_counters_health_gates_and_audio_actions_match(){let mut hash=2166136261u32;for s in 0..4096usize{let frame=(s%8)as i16;let owner=[0,1,3,4,-1,127][s/8%6];let maximum=[-32768,-1,0,1,9,100,32767i16][s/48%7];let threshold=i32::from(maximum)-(i32::from(maximum)/5)*4;let hp=[0,threshold-1,threshold,threshold+1,32767][s/336%5]as i16;let pulse=s/1680%2!=0;let x=[-64,-63,500,1024,1025][s*7%5];let y=[-48,-47,500,1024,1025][(s*3+1)%5];let flags=(s*13)as u8;let notice=(s*7)as u8;let mut state=Radar{word0a:frame,flags6:flags,notice15:notice};let out=state.advance(Input{position:[x,y],maximum,hp,owner,local_team:0,pulse,camera:[0,0],size:[1024,1024]});let mut bytes=2166136261;if let Some(r)=out.record{mix(&mut bytes,r.bytes().map(u32::from));}mix(&mut hash,[s as u32,frame as u32,owner as u32,maximum as u32,hp as u32,u32::from(pulse),x as u32,y as u32,u32::from(flags),u32::from(notice),state.word0a as u32,u32::from(state.flags6),u32::from(state.notice15),if out.record.is_some(){8}else{0},bytes,u32::from(out.sweep),u32::from(out.secured)]);}assert_eq!(hash,262484407,"actual40cb20+40c9b0; other machinery, fragment/raster/audio callee bodies are explicit boundaries");}
 #[test]fn all_256_original_constructor_reuse_cases_reset_word0a_but_preserve_notice15(){let mut hash=2166136261;for s in 0..256u32{let mut r=Radar{word0a:s.wrapping_mul(997)as i16,flags6:s as u8,notice15:s.wrapping_mul(7)as u8};r.construct();mix(&mut hash,[s,r.word0a as u32,u32::from(r.flags6),u32::from(r.notice15)]);}assert_eq!(hash,3159254469,"actual40c820, terrain stamping/fragment constructors are separate source-tested modules");}
}
