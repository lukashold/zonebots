//! Bridge32/44fcc0/40e9d0 kernels after an admitted browser repair order.
//! Original438cf0 stack-local and45ef00 native network routing are separate
//! unported callers. This module does not replace them with zero defaults.
use super::trail::{State,Boundary};
pub const BASE:u32=0x6151e0;
pub const HEADERS:[u32;8]=[0x647210,0x63d1f8,0x6421f8,0x63aa18,0x649a48,0x63f9e0,0x638238,0x6449f0];
pub const FRAMES:[u8;8]=[27,23,23,21,25,21,21,15];
pub fn word(r:&[u8;32],at:usize)->i16{i16::from_le_bytes(r[at..at+2].try_into().unwrap())}
pub fn set(r:&mut[u8;32],at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
///45fef0 writes only these bytes; all unowned runtime bytes retain their
///previous value, including unused records and later mission reuse.
pub fn load(r:&mut[u8;32],d:&[u8;26]){r[..19].copy_from_slice(&d[..19]);set(r,20,i16::from(d[16]as i8).wrapping_mul(256).wrapping_add(i16::from(d[15]as i8)));r[28]=d[21];}
pub fn contains_destroyed(r:&[u8;32],p:[i16;2])->bool{word(r,0)!=-1&&r[15]&2!=0&&(0..2).all(|a|i32::from(word(r,a*2))<=i32::from(p[a])&&i32::from(p[a])<=i32::from(word(r,a*2))+i32::from(word(r,4+a*2)))}
pub fn admitted_waypoint(r:&[u8;32])->[i16;2]{let mut p=[word(r,0).wrapping_add(word(r,4)>>1),word(r,2).wrapping_add(word(r,6)>>1).wrapping_sub(8)];if word(r,12)==0{p[0]=p[0].wrapping_add(8);p[1]=p[1].wrapping_sub(8);}p}
pub fn progress(r:&mut[u8;32],rate:u8)->bool{let n=word(r,8).wrapping_add(i16::from(rate>>4));set(r,8,n);n>=word(r,10)}
pub fn retreat(r:&[u8;32],actor:[i16;2],direction:u8)->[i16;2]{let(x,y)=(word(r,0),word(r,2));match direction as i8{1=>[x.wrapping_sub(48),y.wrapping_add(8)],2=>[x.wrapping_add(8),y.wrapping_sub(48)],3=>[x.wrapping_add(word(r,4)).wrapping_add(48),actor[1].wrapping_add(8)],_=>[actor[0].wrapping_add(8),y.wrapping_add(word(r,6)).wrapping_add(16)]}}
///409e70 receives horizontalX+16 from44fcc0, verticalX unchanged. Each
///returned cell is evaluated/written in source order, preserving overlap.
pub fn restore_terrain(r:&[u8;32],mut read:impl FnMut(i16)->u8)->Vec<(i16,u8)>{let horizontal=word(r,12)==0;let x=word(r,0).wrapping_add(if horizontal{16}else{0});let y=word(r,2);let base=(y as u16&0xfff0).wrapping_mul(8).wrapping_add((x>>4)as u16)as i16;let n=word(r,if horizontal{4}else{6})>>4;let mut out=Vec::new();for i in 0..n{let edge=i==0||i==n-1;let p=base.wrapping_add(if horizontal{i}else{i.wrapping_mul(128)});let offsets=if horizontal{[-1,127,255,383]}else{[0,1,2,3]};for(j,off)in offsets.into_iter().enumerate(){let cell=p.wrapping_add(off);let old=out.iter().rev().find(|&&(at,_)|at==cell).map_or_else(||read(cell),|&(_,n)|n);let v=if j==0||j==3{(old&0xab)|43}else if edge{(old&0x87)|7}else{(old&0x97)|23};out.push((cell,v));}}out}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Birth{pub allocated:usize,pub last_choice:u16}
///Caller executes409e70 BEFORE this function. One random call is required
///even if all64 slots are occupied. No FX allocator or second job pool.
pub fn jobs(s:&mut State,r:&mut[u8;32],bridge:u32,actor:u32,mut random:impl FnMut()->u32)->Result<Birth,Boundary>{r[16]=0;let mut choice=random()as u16;let horizontal=word(r,12)==0;let n=(word(r,if horizontal{4}else{6})>>4).wrapping_sub(2);let mut p=[word(r,0).wrapping_add(if horizontal{16}else{0}),word(r,2)];let mut allocated=0;for _ in 0..n{let Some(slot)=s.free_job()else{break;};choice=choice.wrapping_add(1)&3;let i=usize::from(choice)+if horizontal{4}else{0};let head=HEADERS[i];let flags=s.word(head)?as u16|128;s.put_word(head,flags as i16)?;let frame=s.read(head+11,1)?[0];let j=&mut s.jobs[slot];j.set_ptr(0,head);j.set_ptr(4,bridge);j.set(8,p[0]);j.set(10,p[1]);j.0[12]=frame;j.set_ptr(16,actor);j.set(20,if horizontal{[0,4,4,10][usize::from(choice)]}else{[0,4,4,6][usize::from(choice)]});r[16]=r[16].wrapping_add(1);p[usize::from(!horizontal)]=p[usize::from(!horizontal)].wrapping_add(16);allocated+=1;}Ok(Birth{allocated,last_choice:choice})}
///Frame1 decrements the byte, not a word. Only its zero transition restores
///the bridge and permits the source completion audio/UI branch.
pub fn finish_job(r:&mut[u8;32])->bool{r[16]=r[16].wrapping_sub(1);if r[16]!=0{return false;}r[15]&=0xd1;r[28]=0;let n=word(r,10);set(r,8,if n==0{300}else{n});true}

#[cfg(test)]#[path="original_crane_bridge_tests.rs"]mod tests;
