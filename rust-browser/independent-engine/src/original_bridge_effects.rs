//!09a60/09870 bridge prefixes,402160 common writes,403370 and401210/402770.
//!Raw payloads are caller authorities. No browser address/projection heuristic.
use super::spray::{Context,Pool};
pub const NATIVE_POOL:u32=0x5c9a88;
pub const BRIDGE_CONSTRUCTOR:u32=0x403370;
pub const HIT_CONSTRUCTOR:u32=0x402770;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Raw{pub byte32:u8,pub payload36:u32,pub payload40:u32,pub command:u32,pub last_record:Option<u32>,pub constructor:Option<u32>}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Globals{pub context:Context,pub maximum:u8,pub suppression:u8,pub raw:Raw}
pub trait World:Pool{fn retire(&mut self,slot:usize);fn before_emit(&mut self,_count:u16,_g:&Globals){}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Constructor{Bridge,Hit}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Sprites{pub prefix:i16,pub children:[i16;2]}
pub fn native_sprites(world:usize)->Option<Sprites>{Some(Sprites{prefix:*[1138,1186,1174,1150,1162].get(world)?,children:*[[1578,1586],[3347,3355],[3291,3299],[3235,3243],[3403,3411]].get(world)?})}
pub fn engine_sprites(world:usize)->Option<Sprites>{native_sprites(*[0,3,1,4,2].get(world)?)}
fn word(b:&mut[u8;44],at:usize,value:i16){b[at..at+2].copy_from_slice(&value.to_le_bytes());}
///402160 writes before invoking its selected constructor.34/35 survive.
pub fn common(record:&mut[u8;44],g:&Globals){
 record[..6].fill(0);word(record,6,g.context.origin[0]);word(record,8,g.context.origin[1]);record[22]=g.context.flags;record[25]=g.context.owner;word(record,26,g.context.sprite);record[24]=g.maximum;record[15]=g.context.tint;record[36..40].copy_from_slice(&g.raw.payload36.to_le_bytes());record[40..44].copy_from_slice(&g.raw.payload40.to_le_bytes());record[32]=g.raw.byte32;record[33]=0;record[16..18].fill(g.context.fade);record[28..32].fill(0);
}
pub fn construct(record:&mut[u8;44],kind:Constructor,mut random:impl FnMut()->u32){match kind{
 Constructor::Bridge=>{let seed=crate::original_fragment_blast::water_seed(&mut random);record[23]=seed.phase as u8;for(i,v)in seed.velocity.into_iter().enumerate(){record[10+i]=v as u8;}record[13]=1;record[14]=3;word(record,18,500);word(record,20,11);word(record,30,seed.spin);word(record,28,0);}
 Constructor::Hit=>{record[23]=(random()&3)as u8;let x=(random()&15)as u8;record[10]=x.wrapping_add((random()&7)as u8).wrapping_sub(12);let y=(random()&15)as u8;record[11]=y.wrapping_add((random()&7)as u8).wrapping_sub(12);record[12]=(-4i8).wrapping_sub((random()&7)as i8)as u8;word(record,20,if i16::from_le_bytes(record[26..28].try_into().unwrap())==1903{4}else{0});word(record,18,200);record[13]=2;record[14]=1;}
}}
pub fn emit(w:&mut impl World,g:&mut Globals,count:u16,kind:Constructor)->usize{
 w.before_emit(count,g);if g.suppression!=0{return 0;}let mut made=0;for _ in 0..count{let Some(slot)=w.first_free()else{break;};let mut r=w.load(slot);common(&mut r,g);g.raw.last_record=Some(NATIVE_POOL.wrapping_add((slot as u32).wrapping_mul(44)));construct(&mut r,kind,||w.random());w.store(slot,r);made+=1;}made
}
///All origins and the length subtraction are signed16, including wrapping.
pub fn prefix(w:&mut impl World,g:&mut Globals,position:[i16;2],size:[i16;2],orientation:i16,half:bool,sprite:i16)->usize{
 let disabled=g.suppression;g.raw.command=(g.raw.command&0xffff0000)|1610;g.maximum=12;g.context.owner=0;let horizontal=orientation==0;let n=(size[usize::from(!horizontal)]>>4).wrapping_sub(2);let mut made=0;
 for i in 0..n{for lane in 0..4usize{if half&&(lane==0||lane==3){continue;}g.context.origin=if horizontal{[position[0].wrapping_sub(144).wrapping_add(i.wrapping_mul(16)),position[1].wrapping_sub([108,92,76,60][lane])]}else{[position[0].wrapping_sub([160,144,128,112][lane]),position[1].wrapping_sub(92).wrapping_add(i.wrapping_mul(16))]};g.context.sprite=sprite;g.maximum=11;g.context.fade=0;g.context.flags=12;g.suppression=0;g.raw.constructor=Some(BRIDGE_CONSTRUCTOR);made+=emit(w,g,1,Constructor::Bridge);g.raw.payload36=g.raw.payload36.wrapping_add(10);}}
 g.suppression=disabled;made
}
pub fn expiry(w:&mut impl World,g:&mut Globals,source:usize,projection:[i16;2],sprites:[i16;2])->(u16,usize,usize){
 let r=w.load(source);let origin=[i16::from_le_bytes(r[6..8].try_into().unwrap()),i16::from_le_bytes(r[8..10].try_into().unwrap())];g.context.origin=[origin[0].wrapping_add(projection[0]).wrapping_sub(152),origin[1].wrapping_add(projection[1]).wrapping_sub(84)];g.context.fade=0;g.context.owner=0;g.context.flags=4;g.maximum=5;g.context.sprite=sprites[0];let count=4+(w.random()&3)as u16;g.raw.constructor=Some(HIT_CONSTRUCTOR);let first=emit(w,g,count,Constructor::Hit);g.context.sprite=sprites[1];g.raw.constructor=Some(HIT_CONSTRUCTOR);let second=emit(w,g,4,Constructor::Hit);w.retire(source);(count,first,second)
}
#[cfg(test)]#[path="original_bridge_effects_tests.rs"]mod tests;
