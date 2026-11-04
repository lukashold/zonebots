//!43f260's eight wreck sprites append five ordinary eight-byte records.
//!The Christmas/base5960 branch, shadow raster and expiry damage are separate.
use super::{ObjectInput,Record};
pub const WRECK_OFFSETS:[[i16;6];4]=[[2,8,16,9,4,1],[0,6,2,16,13,16],[2,4,13,13,18,5],[0,4,18,1,20,6]];
#[derive(Clone,Debug)]pub struct Draw{pub records:Vec<Record>,pub spark:Option<[i16;2]>}
///Records remain in world coordinates. The original subtracts the camera
///before queue writes; the compositor applies that same translation later.
pub fn project(input:ObjectInput,variant26:i8,frame4:i16,frame6:i8,camera:[i16;2],size:[i16;2])->Result<Draw,i8>{
 let mut out=Draw{records:vec![],spark:None};let p=input.position;if p[0]<=0{return Ok(out);}let screen=[i32::from(p[0])-i32::from(camera[0]),i32::from(p[1])-i32::from(camera[1])];if screen[0]<=-32||screen[0]>=i32::from(size[0])||screen[1]<=-32||screen[1]>=i32::from(size[1]){return Ok(out);}
 let Some(primary)=super::object(input)else{return Ok(out);};out.records.push(primary);
 if input.flags&32==0||![512,5317,5687,5688,5692,5695,5696,5747].contains(&primary.sprite){return Ok(out);}
 let offsets=WRECK_OFFSETS.get(usize::try_from(variant26).map_err(|_|variant26)?).ok_or(variant26)?;let y=p[1].wrapping_add(if primary.sprite==5687{4}else{0});let record=|x,y,sprite,offset|Record{x,y,sprite,palette:0,offset};
 out.records.push(record(p[0].wrapping_sub(1),y.wrapping_add(1),1782i16.wrapping_add(frame4),-23));out.records.push(record(p[0].wrapping_add(8),y.wrapping_add(2),1800i16.wrapping_add(i16::from(frame6)),-1));
 for i in 0..3{out.records.push(record(p[0].wrapping_add(offsets[i*2]),y.wrapping_add(offsets[i*2+1]),1786i16.wrapping_add(frame4),if i==1{-2}else{0}));}out.spark=Some([p[0].wrapping_add(4),y.wrapping_sub(16)]);Ok(out)
}
///43f605 consumes one CRT draw in SP; MP supplies zero without a draw.
pub fn spark_probe(random:u16)->bool{random&127==0}
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619);}
 fn bytes_hash(bytes:impl IntoIterator<Item=u8>)->u32{let mut h=2166136261;for b in bytes{mix(&mut h,u32::from(b));}h}
 fn case(h:&mut u32,id:u32,kind:usize,animation:i8,offset:i8,flags:u8,variant:i8,frame:i16,six:i8,mp:bool,random:u16,pool_mode:usize,camera:[i16;2]){
  let base=crate::original_world_contact::object_metadata(kind as u16).unwrap().0 as i16;let input=ObjectInput{position:[500,600],base,animation,sprite_offset:offset,flags,depth:super::super::OBJECT_DEPTH[kind]};let draw=project(input,variant,frame,six,camera,[1024,1024]).unwrap();let mut pool=crate::original_ambient_animations::State::default();if pool_mode==1{for(i,r)in pool.records.iter_mut().enumerate(){r.set_word(0,100+i as i16);}}if pool_mode==2{let y=608+if base.wrapping_add(i16::from(animation)).wrapping_add(i16::from(offset))==5687{4}else{0};for o in[0,4,8]{pool.records[1].set_word(o,504);pool.records[1].set_word(o+2,y);}}
  let(mut draws,mut sparks,mut point)=(0u32,0u32,[-999i16,-999]);if let Some(p)=draw.spark{if !mp{draws+=1;}if spark_probe(if mp{0}else{random}){sparks=1;point=p;if !mp{draws+=1;}pool.spawn_spark(p,if mp{0}else{0x1743});}}
  let bytes=draw.records.iter().flat_map(|r|Record{x:r.x.wrapping_sub(camera[0]),y:r.y.wrapping_sub(camera[1]),..*r}.bytes());let qhash=bytes_hash(bytes);let phash=bytes_hash(pool.records.iter().flat_map(|r|r.bytes));for n in[id,(draw.records.len()*8)as u32,qhash,draws,sparks,point[0]as u32,point[1]as u32,phash,pool.records[0].word(0)as u32,pool.records[0].word(2)as u32,pool.records[0].word(20)as u32]{mix(h,n);}
 }
 #[test]fn all_919_native_object_calls_match_full_rows_and_real_spark_pool_side_effects(){
  let mut hash=2166136261;let mut id=0;for kind in 0..78usize{let base=crate::original_world_contact::object_metadata(kind as u16).unwrap().0 as i16;for animation in[-1,0,4,127i8]{for offset in[-128,0,30,127i8]{for flags in[0,32,34,38,40,48,62,112u8]{let sprite=base.wrapping_add(i16::from(animation)).wrapping_add(i16::from(offset));if flags&32!=0&&[512,5317,5687,5688,5692,5695,5696,5747].contains(&sprite){case(&mut hash,id,kind,animation,offset,flags,0,0,0,true,0,0,[0,0]);id+=1;}}}}}assert_eq!(id,119);
  let kinds=[6,25,27,30,32,34,36,38];for s in 0..768usize{case(&mut hash,id,kinds[s%8],0,0,[32,34,38,40,48,62,112][s%7],(s/8%4)as i8,(s/32%4)as i16,(s/128%6)as i8,s/32%2!=0,[0,1,128,255][s/8%4],s/256%3,[400,480]);id+=1;}
  for s in 0..32usize{case(&mut hash,id,kinds[s%8],0,(s/16)as i8,if s/8%2==0{0}else{32},(s%4)as i8,3,5,false,0,0,[[0,0],[531,0],[532,0],[-523,0],[-524,0],[0,631],[0,632],[0,-424]][s%8]);id+=1;}assert_eq!(id,919);assert_eq!(hash,2353676312,"actual43f260→43f200/41dbf0/41d4b0; DLLrand is explicit input, shadow raster unused in these cases");
 }
 #[test]fn an_unwritten_invalid_variant_is_an_explicit_boundary_instead_of_a_masked_guess(){let i=ObjectInput{position:[500,600],base:512,animation:0,sprite_offset:0,flags:32,depth:-25};for variant in[-128,-1,4,127]{assert_eq!(project(i,variant,0,0,[0,0],[1024,1024]).unwrap_err(),variant);}}
}
