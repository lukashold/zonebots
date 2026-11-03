//! Directed native object/100-record target scans0044d200/0044d290/0044cf80. These
//! query kernels retain pool order, signed quantization and separate ties.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Object{pub slot:u8,pub position:[i16;2],pub bucket12:u8,pub flags19:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Target{pub slot:u8,pub position:[i16;2],pub flags34:u8,pub flags35:u8,pub phase3c:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Choice{pub slot:u8,pub distance:u8}
pub const CELLS:[[i32;9];8]=[[0,-1,1,-17,-16,-15,-32,-33,-31],[0,1,2,-16,-15,-14,-32,-31,-30],[0,-16,16,1,-15,17,2,-14,18],[0,1,2,16,17,18,32,33,34],[0,-1,1,17,16,15,32,33,31],[0,-1,-2,16,15,14,32,31,30],[0,-16,16,-1,-17,15,-2,-18,14],[0,-1,-2,-16,-17,-18,-32,-33,-34]];
pub fn coarse_cell(point:[i16;2])->u8{(((point[1]>>3)as u8&240).wrapping_add((point[0]>>7)as u8))as u8}
///0044cbe0/cca0/cc40/ccd0/cc10/cd30/cc70/cd00 return a true wedge
///predicate for object/100-record target scans. They also rewrite the signed X delta.
pub fn cone(direction:u8,dx:i16,dy:i16)->(bool,i16){
 match direction&7{
  0=>{let x=dx.wrapping_abs();(x<=dy.wrapping_neg(),x)},
  1=>(dx>=0&&dy<0,dx),2=>(dx>=dy.wrapping_abs(),dx),3=>(dx>=0&&dy>=0,dx),
  4=>{let x=dx.wrapping_abs();(x<=dy,x)},5=>(dx<0&&dy>=0,dx),
  6=>{let x=dx.wrapping_neg();(x>=dy.wrapping_abs(),x)},_ =>(dx<0&&dy<0,dx),
 }
}
///0044d3a0's scan phase. Blocked scans leave both persistent bytes intact.
pub fn direction(heading:u8,relative:u8,flags8a:u8,flags64:u8,flags71:u8,flags9c:&mut u8,a4:&mut u8)->Option<u8>{
 if flags8a&16!=0||flags64&32!=0{return None;}*flags9c^=64;
 Some(if flags71&8!=0{*a4=a4.wrapping_add(1)&7;*a4}else{heading.wrapping_add(relative)&7})
}
pub fn object_candidates(objects:&[Object],origin:u8,direction:u8)->Vec<u8>{
 let mut buckets:[Vec<u8>;256]=std::array::from_fn(|_|Vec::new());let mut sorted=objects.iter().filter(|o|o.position[0]>=0&&o.slot<200).collect::<Vec<_>>();sorted.sort_by_key(|o|o.slot);
 for object in sorted{let b=&mut buckets[usize::from(object.bucket12)];if b.len()!=32{b.push(object.slot);}}
 let mut result=Vec::new();let mut total=0i16;
 for delta in CELLS[usize::from(direction&7)]{let index=(i16::from(origin).wrapping_add(delta as i16))as u16;if index<256{let bucket=&buckets[usize::from(index)];if !bucket.is_empty(){total=total.wrapping_add(bucket.len()as i16);if total<288{result.extend_from_slice(bucket);}}}}
 result
}
pub fn object_scan(objects:&[Object],candidates:&[u8],position:[i16;2],team:u8,direction:u8,mut visible:impl FnMut(u8)->bool)->Option<Choice>{
 let mut distance=255i32;let mut selected=None;
 for &slot in candidates{let Some(o)=objects.iter().find(|o|o.slot==slot)else{continue;};if o.flags19&(1u8.wrapping_shl(u32::from(team&3)))==0{continue;}
  let raw_x=o.position[0].wrapping_sub(position[0]);let(ok,_)=cone(direction,raw_x,o.position[1].wrapping_sub(position[1]));if !ok{continue;}
  //Unlike the 100-record target/weapon scans,44d290 ignores the predicate's rewritten
  //X output. A negative original delta therefore rounds down before squaring.
  let dx=i32::from(raw_x)>>3;let dy=(i32::from(o.position[1])-i32::from(position[1]))>>3;let square=dx*dx+dy*dy;
  //The object loop compares against the square of the *truncated previous
  //distance*. It deliberately does not break equal-bin ties by real range.
  if square<distance*distance&&visible(slot){distance=(square as f64).sqrt()as i32;selected=Some(Choice{slot,distance:distance as u8});}
 }
 selected
}
///0044d0c0 accumulates each offset into the preceding bucket index. This
///differs from the object list's fixed-origin addressing and may repeat IDs.
pub fn target_candidates(buckets:&[Vec<u8>;256],origin:u8,direction:u8)->Vec<u8>{
 let mut cell=u32::from(origin);let mut result=Vec::new();for delta in CELLS[usize::from(direction&7)]{cell=cell.wrapping_add(delta as u32);if cell<256{result.extend(buckets[cell as usize].iter().take(32).copied());}}result
}
pub fn target_scan(flags:&[Target],candidates:&[u8],position:[i16;2],direction:u8,mut visible:impl FnMut(u8)->bool)->Option<Choice>{
 let mut minimum=65025i32;let mut selected=None;
 for &slot in candidates{let Some(f)=flags.iter().find(|f|f.slot==slot)else{continue;};if f.flags34&10!=0||f.flags35&176!=0||matches!(f.phase3c,4|5|6){continue;}
  let(ok,x)=cone(direction,f.position[0].wrapping_sub(position[0]),f.position[1].wrapping_sub(position[1]));if !ok{continue;}let x=i32::from(x)>>3;let y=(i32::from(f.position[1])-i32::from(position[1]))>>3;let square=x*x+y*y;
  if square<minimum&&visible(slot){minimum=square;selected=Some(Choice{slot,distance:(square as f64).sqrt()as u8});}
 }
 selected
}

///00431ee0/00432250 sample after every 3D dominant-axis step, including
///the final cell and excluding the starting cell. Terrain bytes are signed.
pub fn clear_ray(mut start:[i16;3],mut end:[i16;3],mut height:impl FnMut(i16)->i8)->bool{
 if start[2]!=0{start[2]=start[2].wrapping_add(1);}if end[2]!=0{end[2]=end[2].wrapping_add(1);}
 start[0]>>=4;start[1]>>=4;end[0]>>=4;end[1]>>=4;if start[0]>=end[0]{std::mem::swap(&mut start,&mut end);}
 let delta=std::array::from_fn::<_,3,_>(|i|i32::from(end[i])-i32::from(start[i]));let abs=delta.map(i32::abs);let signs=delta.map(|n|if n<0{-1}else{1});
 let major=if abs[1]<abs[0]&&abs[2]<=abs[0]{0}else if abs[0]<=abs[1]&&abs[2]<=abs[1]{1}else{2};let length=abs[major];let mut error=[length>>1;3];let mut at=start.map(i32::from);
 for _ in 0..length{at[major]+=signs[major];for axis in 0..3{if axis!=major{error[axis]-=abs[axis];if error[axis]<0{error[axis]+=length;at[axis]+=signs[axis];}}}let cell=(at[0]+at[1]*128)as i16;if i16::from(height(cell))>at[2]as i16{return false;}}
 true
}
///00432510 object sightline and00432430 100-record target sightline share this setup.
///The target must have zero terrain height before the 3D traversal begins.
pub fn visible(position:[i16;2],class:u8,actor_cell:i16,target:[i16;2],mut height:impl FnMut(i16)->i8)->bool{
 let offset=if class==1{8}else{16};let start=[position[0].wrapping_add(offset)&!15,position[1].wrapping_add(offset)&!15,i16::from(height(actor_cell))];let end=[target[0].wrapping_add(8)&!15,target[1].wrapping_add(8)&!15,0];let cell=(end[0]>>4).wrapping_add((end[1]&!15).wrapping_mul(8));if height(cell)!=0{return false;}clear_ray(start,end,height)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_288_native_target_scans_match_cumulative_buckets_filters_and_terrain_los(){
  let mut hash=2166136261u32;let mut rows=0;
  for heading in 0..8{for fixture in 0..6{for blocked in 0..2{for gate in 0..3{
   let mut targets=vec![];let mut buckets:[Vec<u8>;256]=std::array::from_fn(|_|Vec::new());
   for i in 0..64{if i%11==fixture{continue;}let p=[(500+(i*37+fixture*13)%705-352)as i16,(600+(i*71+fixture*17)%705-352)as i16];targets.push(Target{slot:i as u8,position:p,flags34:if i%17==0{2}else{0},flags35:if i%13==0{16}else{0},phase3c:if i%19==0{4}else{0}});let cell=usize::from(coarse_cell(p));if buckets[cell].len()<32{buckets[cell].push(i as u8);}}
   let chosen=if gate==0{let candidates=target_candidates(&buckets,coarse_cell([500,600]),heading);target_scan(&targets,&candidates,[500,600],heading,|slot|{let target=targets.iter().find(|t|t.slot==slot).unwrap().position;visible([500,600],0,4895,target,|cell|if blocked==1&&cell&127==34{1}else{0})})}else{None};
   let(slot,distance)=if gate!=0{(255,77)}else{chosen.map(|c|(i32::from(c.slot),i32::from(c.distance))).unwrap_or((255,255))};
   for n in[i32::from(heading),fixture,blocked,gate,slot,distance]{hash=(hash^n as u32).wrapping_mul(16777619);}rows+=1;
  }}}}
  assert_eq!(rows,288);assert_eq!(hash,1903420535);
 }
 #[test]fn all_576_native_object_scans_match_direction_phase_pool_order_terrain_los_and_gates(){
  let mut hash=2166136261u32;let mut rows=0;
  for heading in 0..8{for relative in 0..3{for cycle in 0..2{for blocked in 0..2{for team in 0..2{for gate in 0..3{
   let fixture=i32::from(heading)+relative*8;let mut objects=vec![];for i in 0..24{if i%7==fixture%7{continue;}let p=[(500+(i*47+fixture*13)%321-160)as i16,(600+(i*71+fixture*17)%321-160)as i16];objects.push(Object{slot:i as u8,position:p,bucket12:coarse_cell(p),flags19:if i%3==0{3}else{1<<(i&1)}});}
   let mut flags=0x82;let mut a4=(heading+3)&7;
   let dir=direction(heading,relative as u8,if gate==1{16}else{0},if gate==2{32}else{0},if cycle==1{8}else{0},&mut flags,&mut a4);
   let selected=dir.and_then(|direction|{let candidates=object_candidates(&objects,coarse_cell([500,600]),direction);object_scan(&objects,&candidates,[500,600],team as u8,direction,|slot|{let target=objects.iter().find(|o|o.slot==slot).unwrap().position;visible([500,600],0,4895,target,|cell|if blocked==1&&cell&127==34{1}else{0})})});
   let(slot,distance)=if dir.is_none(){(255,77)}else{selected.map(|c|(i32::from(c.slot),i32::from(c.distance))).unwrap_or((255,255))};
   for n in[i32::from(heading),relative,cycle,blocked,team,gate,slot,distance,i32::from(flags),i32::from(a4)]{hash=(hash^n as u32).wrapping_mul(16777619);}rows+=1;
  }}}}}}
  assert_eq!(rows,576);assert_eq!(hash,4197813124);
 }
}
