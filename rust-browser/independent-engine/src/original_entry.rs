//! Neutral OBJECT hardware attributes43eee0 and takeover581f0. The shared
//!240-slot allocator remains owned by the native order pool, not this module.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct HardwareAttributes{pub bytes:[u8;44]}
impl Default for HardwareAttributes{fn default()->Self{Self{bytes:[0;44]}}}
impl HardwareAttributes{
 pub fn byte(&self,o:usize)->u8{self.bytes[o]}
 pub fn word(&self,o:usize)->i16{i16::from_le_bytes([self.bytes[o],self.bytes[o+1]])}
 pub fn set_word(&mut self,o:usize,n:i16){self.bytes[o..o+2].copy_from_slice(&n.to_le_bytes());}
 ///581f0 does not reset the order target, selected slot, animation counters,
 ///or any other byte not written by this exact merge.
 pub fn take_over(&mut self,object:ObjectHardware,crew_rate:u8,crew_projectile_speed:u8)->Controls{
  self.set_word(0,0);self.bytes[32]=object.byte(24);self.bytes[41]=crew_rate;self.bytes[42]=crew_projectile_speed;self.bytes[33]=object.byte(25);
  self.bytes[5]=object.byte(14);self.bytes[13]=object.byte(14);self.bytes[6]=object.byte(17);self.bytes[7]=object.byte(16);self.bytes[12]=object.byte(16);
  self.bytes[36]=object.byte(28);self.bytes[35]=object.byte(27);self.bytes[37]=object.byte(29);self.bytes[38]=object.byte(30);
  self.set_word(8,i16::from(object.byte(15))*25);self.set_word(10,i16::from(object.byte(40)as i8)*25);
  Controls{fire_rate:object.byte(24),cooldown:0,projectile_speed:object.byte(25),double_range:object.byte(14).wrapping_mul(2)}
 }
 ///50be0's selected hardware initializer and50cd0's final west-facing write.
 ///Fixed guns use582c0 instead; they retain the vehicle-only W fields.
 pub fn initialize_occupied(&mut self,definition:u16,vehicle:bool)->EntryProfile{
  self.bytes[18]=255;
  if !vehicle{return EntryProfile{kind:match definition{5=>"cannon",22=>"gatling",23=>"howitzer",_=>"missile"},class:2,phase:match definition{5=>4,22=>51,23=>52,_=>50},family:definition as u8,heading:6,timer:15};}
  self.bytes[21]=6;self.bytes[23]=0;
  if definition!=24{self.bytes[4]=0;self.set_word(25,255);}
  let(kind,phase,family)=match definition{24=>("launcher",53,24),26=>("heavy",54,25),29=>("light",55,30),33=>("jeep",58,34),35=>("crane",59,36),_=>("medium",9,6)};
  EntryProfile{kind,class:1,phase,family,heading:6,timer:15}
 }
 ///58090 swaps the retained robot range into W5 while UNIT4 takes the
 ///vehicle's precedingW5. The caller keeps that distinct body-range byte.
 pub fn initialize_apc(&mut self,driver_range:u8)->u8{let rate=if self.bytes[32]==0{80}else{self.bytes[32]};self.bytes[32]=rate;self.bytes[18]=255;self.bytes[21]=0;self.bytes[23]=0;self.bytes[4]=0;self.set_word(25,255);self.bytes[39]=driver_range;self.bytes[5]=driver_range;rate}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct EntryProfile{pub kind:&'static str,pub class:u8,pub phase:u8,pub family:u8,pub heading:u8,pub timer:i8}
///450db0's admission table499738, separate from the scripted equipment ID.
pub fn available(definition:u16)->bool{matches!(definition,5|7|8|9|10|21|22|23|24|26|29|31|33|35|37|58)}
///4967e0 object-to-unit origins. Heavy family25 then adds496934=(6,2).
pub fn occupied_origin(definition:u16,point:[i16;2],vehicle:bool)->[i16;2]{
 let offset=match definition{5=>[4,0],8=>[8,6],22=>[4,1],23=>[1,6],24=>[6,3],26=>[0,2],29=>[11,7],31=>[6,6],33=>[4,7],35=>[6,6],37=>[3,6],58=>[0,4],_=>[0,0]};
 let extra=if vehicle&&definition==26{[6,2]}else{[0,0]};std::array::from_fn(|i|point[i].wrapping_add(offset[i]).wrapping_add(extra[i]))
}
///The narrow hardware path of450db0. Other contact codes are separate states.
pub fn within_entry_radius(actor:[i16;2],object:[i16;2])->bool{let d=std::array::from_fn::<_,2,_>(|i|i32::from(object[i].wrapping_sub(actor[i])));d[0]*d[0]+d[1]*d[1]<401}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Controls{pub fire_rate:u8,pub cooldown:u8,pub projectile_speed:u8,pub double_range:u8}
///Only the source OBJECT fields consumed by581f0 are interpreted. Position,
///flags, animation, pool identity and lifetime belong to the world pool.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct ObjectHardware{pub bytes:[u8;44]}
impl Default for ObjectHardware{fn default()->Self{Self{bytes:[0;44]}}}
impl ObjectHardware{
 pub fn byte(self,o:usize)->u8{self.bytes[o]}pub fn set_byte(&mut self,o:usize,n:u8){self.bytes[o]=n;}
 ///43eee0 overwrites these11 projected bytes and leaves all others intact.
 pub fn set_equipment(&mut self,equipment:u8)->bool{let source=if equipment==255{Self::default()}else{let Some(a)=equipment_attributes(equipment)else{return false;};a};for o in[14,15,16,17,24,25,27,28,29,30,40]{self.bytes[o]=source.bytes[o];}true}
}
///43eee0 field projection of the equipment table in local robots.dat. These
///are typed numeric gameplay values in original equipment-index order.
pub fn equipment_attributes(equipment:u8)->Option<ObjectHardware>{
 //range8,hull25,movement,damage,rate,projectileSpeed,secondaryDamage,
 //secondaryRate,secondarySpeed,secondaryRange8.
 let a=*[
  [15,4,0,75,60,10,0,0,0,0], [15,8,40,80,70,10,4,80,15,14],
  [15,0,0,5,80,15,0,0,0,0], [15,0,0,40,80,10,0,0,0,0],
  [18,0,3,10,80,15,0,0,0,0], [18,12,30,120,60,8,4,80,15,45],
  [20,12,20,120,20,4,0,0,0,96], [15,0,0,2,98,15,0,0,0,0],
  [15,4,50,50,80,12,4,80,15,0], [20,4,0,100,30,5,0,0,0,74],
  [15,8,50,5,80,15,0,0,0,0], [15,2,0,6,97,15,0,0,0,0],
  [25,3,0,100,10,5,0,0,0,0], [15,2,60,4,98,15,0,0,0,0],
  [15,8,50,20,98,8,0,0,0,0], [15,12,50,120,50,0,0,0,0,0],
  [0,3,60,0,0,0,0,0,0,0], [15,0,0,20,98,13,0,0,0,0],
  [17,0,0,15,80,15,0,0,0,0],
 ].get(usize::from(equipment))?;
 let mut out=ObjectHardware::default();for(o,n)in[14,15,16,17,24,25,27,28,29,30].into_iter().zip(a){out.bytes[o]=n;}out.bytes[40]=a[1];Some(out)
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,n|(h^u32::from(*n)).wrapping_mul(16777619))}
 #[test]fn all_320_native_equipment_and_takeover_merges_preserve_recycled_bytes_and_copy_current_hull(){
  let mut total=2166136261u32;let mut count=0;
  for equipment in 0..20{for scenario in 0..16{let eq=if equipment==19{255}else{equipment};let free=scenario%5;let mut object=ObjectHardware::default();for(i,b)in object.bytes.iter_mut().enumerate(){*b=(scenario*7+i*13)as u8;}assert!(object.set_equipment(eq));let mut hardware=HardwareAttributes::default();for(i,b)in hardware.bytes.iter_mut().enumerate(){*b=((free*44+i)*11+scenario*3)as u8;}hardware.set_word(0,-1);let controls=hardware.take_over(object,(scenario*7)as u8,(scenario*11)as u8);
   for n in[u32::from(eq),scenario as u32,free as u32,hash(&object.bytes),hash(&hardware.bytes),u32::from(controls.fire_rate),u32::from(controls.projectile_speed),u32::from(controls.double_range),u32::from(controls.cooldown),free as u32]{total=(total^n).wrapping_mul(16777619);}count+=1;
  }}assert_eq!(count,320);assert_eq!(total,3903147213,"actual43eee0+581f0 including full44-byte reused OBJECT/W checksums");
 }
}
