//!433ca0/43f260's December25 base5960 path and402c80's CP tint.
//!Geometry is the same467c70/467d60 color point as the water branch.
use super::{ObjectInput,Record};
#[derive(Clone,Copy,Debug,Default)]pub struct Cp{pub rect:[i16;4],pub owner:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Hit{pub slot:usize,pub outside_mode:bool}
///417400 checks all64 raw slots, then repeats with tile+(1,1).
pub fn cp_at(records:&[Cp;64],point:[i16;2])->Option<Hit>{let p=[point[0]>>4,point[1]>>4];let mut outside=false;for delta in[0i16,1]{let x=p[0].wrapping_add(delta);let y=p[1].wrapping_add(delta);for(slot,r)in records.iter().enumerate(){if r.rect[0]==-1{continue;}outside|=r.rect[0]>1000;if r.rect[0]<=x&&x<r.rect[2]&&r.rect[1]<=y&&y<r.rect[3]{return Some(Hit{slot,outside_mode:outside});}}}None}
pub fn tint(owner:u8)->u8{owner.wrapping_mul(16).wrapping_add(24)}
pub fn tree(input:ObjectInput,mode:u8)->bool{mode==1&&input.base==5960}
pub fn cp_point(input:ObjectInput)->[i16;2]{[input.position[0],input.position[1].wrapping_sub(i16::from(input.depth))]}
pub fn rows(input:ObjectInput,frame16:i16,owner:u8)->Result<[Record;3],i16>{let frame=usize::try_from(frame16).ok().filter(|&n|n<16).ok_or(frame16)?;let p=cp_point(input);let offset=input.depth.wrapping_add(2);let y=p[1].wrapping_add(8);let c=owner&15;Ok([
 Record{x:p[0],y:p[1],sprite:input.base.wrapping_add(i16::from(input.animation)).wrapping_add(i16::from(input.sprite_offset)),palette:255,offset:input.depth},
 Record{x:p[0].wrapping_add(1),y,sprite:292,palette:if c==4{8}else{c},offset},
 Record{x:p[0].wrapping_add(5),y,sprite:293+[0i16,1,2,0,1,2,0,1][frame>>1],palette:0,offset},
 ])}
///41ed57/41eda6/41ee23 adjusts the argument after the FallingObject emitter.
pub fn destruction_point(position:[i16;2],template_flags:u8)->[i16;2]{[position[0],position[1].wrapping_add(if template_flags&2!=0{16}else{0}).wrapping_sub(8)]}
///403090 follows402160's common writes. Its four draws do not replace the
///unwritten34/35 bytes, the current tint, or the shared raw36/40 payloads.
pub fn falling(record:&mut[u8;44],mut random:impl FnMut()->u32){record[23]=0;record[10]=((random()&31)as u8).wrapping_sub(16);record[11]=((random()&31)as u8).wrapping_sub(16);record[12]=(-4i8).wrapping_sub((random()&3)as i8)as u8;record[18..22].copy_from_slice(&0x00070190u32.to_le_bytes());record[30..32].copy_from_slice(&((random()&63)as i16-32).to_le_bytes());record[28..30].fill(0);record[13..15].fill(4);}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn actual_2048_object_submissions_keep_the_three_records_and_signed_queue_fields(){
  let private=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../work/parity-audit/native-christmas.csv")).ok().map(|s|s.lines().filter(|s|s.starts_with("O,")).map(str::to_owned).collect::<Vec<_>>());let mut hash=2166136261u32;let mut triples=0;
  for s in 0..2048usize{let mode=(s%3)as u8;let frame=(s/3%16)as i16;let owner=[0,1,2,3,4,20][s/48%6];let depth=[-128,-1,0,14,23,127][s/288%6];let input=ObjectInput{position:[500,600],base:if s/19%7==0{5959}else{5960},depth,flags:[0,16,32,112][s/7%4],animation:(s/11%4)as i8,sprite_offset:[-1,0,1][s/13%3]};let camera=[[0,40,531,532,-523,-524][s/23%6],[0,60,631,632,-423,-424][s/29%6]];
   let records=if !super::super::objects::visible(input.position,camera,[1024,1024]){Vec::new()}else if tree(input,mode){triples+=1;rows(input,frame,owner).unwrap().to_vec()}else{super::super::object(input).into_iter().collect()};let mut bytes=Vec::new();for r in records{bytes.extend_from_slice(&r.x.wrapping_sub(camera[0]).to_le_bytes());bytes.extend_from_slice(&r.y.wrapping_sub(camera[1]).to_le_bytes());bytes.extend_from_slice(&r.sprite.to_le_bytes());bytes.push(r.palette);bytes.push(r.offset as u8);}for v in [s as u32,bytes.len()as u32].into_iter().chain(bytes.iter().copied().map(u32::from)){hash=(hash^v).wrapping_mul(16777619);}if let Some(lines)=&private{let values=lines[s].split(',').collect::<Vec<_>>();assert_eq!(bytes.len(),values[13].parse::<usize>().unwrap());assert_eq!(bytes.iter().map(|n|format!("{n:02x}")).collect::<String>(),values[14],"43f260 scenario{s}");}
  }assert_eq!(triples,268);assert_eq!(hash,1105404087,"actual43f260/43f200/417400 queue bytes");
 }
 #[test]fn cp_lookup_uses_all_raw_slots_first_hit_then_plus_one_without_a_ui_neutral_guess(){let mut cp=[Cp{rect:[-1,0,0,0],owner:0};64];cp[63]=Cp{rect:[2,3,4,5],owner:255};assert_eq!(cp_at(&cp,[16,32]),Some(Hit{slot:63,outside_mode:false}));cp[4]=Cp{rect:[2,3,4,5],owner:4};assert_eq!(cp_at(&cp,[32,48]).unwrap().slot,4);cp[1].rect=[1001,0,1002,1];assert!(cp_at(&cp,[32,48]).unwrap().outside_mode);assert_eq!(cp_at(&cp,[-32768,-32768]),None);assert_eq!([0,1,2,3,4,15,20,255].map(tint),[24,40,56,72,88,8,88,8]);}
 #[test]fn destruction_and_draw_points_are_distinct_source_arguments(){let input=ObjectInput{position:[500,600],base:5960,depth:23,flags:112,animation:2,sprite_offset:-1};assert_eq!(cp_point(input),[500,577]);assert_eq!(destruction_point(input.position,14),[500,608]);assert_eq!(destruction_point(input.position,12),[500,592]);assert_eq!(rows(input,-1,0),Err(-1));assert_eq!(rows(input,16,0),Err(16));}
}
