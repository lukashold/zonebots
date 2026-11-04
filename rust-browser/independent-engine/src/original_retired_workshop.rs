//!40d050 continuation for a removed UNIT whose4bba0 left role0/groupFF
//!and whose real52500 path heads are empty. This is an alias into OBJECT25,
//!not a valid GROUP slot, and never activates a UNIT or OBJECT.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Unit {
 pub position:[i16;2],pub flags31:u8,pub role:i8,pub group:u8,
 pub flags69:u8,pub flags9c:u8,pub primary:bool,pub secondary:bool,
 pub order:i8,pub target_x:i32,pub target_y:i16,pub metadata:i16,
 pub icon:u16,pub protection:u8,pub counter6e:u8,pub fieldb3:u8,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{Multiplayer,Membership,Paths,Active}
///All five bytes share the actual OBJECT record:24/25,26/27,38 decimal.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Alias{pub word18:i16,pub word1a:i16,pub byte26:u8}
impl Alias{
 pub fn apply(self,object:&mut[u8;44]){object[24..26].copy_from_slice(&self.word18.to_le_bytes());object[26..28].copy_from_slice(&self.word1a.to_le_bytes());object[38]=self.byte26;}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Command{
 pub actor:u32,pub roster:u32,pub arguments:[i16;3],pub opcode:i8,
 ///Forced45ef00 uses no MP queue.4d1202 is cleared;4d02c3 returns0.
 pub dispatch_state:u8,pub force_dispatch:u8,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Output{pub alias:Alias,pub command:Command}
///52500 uses the retained pointer even if its W marker is free or reused.
pub fn clear_hardware_motion(field17:&mut u8){*field17&=!1;}
pub fn complete(unit:&mut Unit,slot:usize,multiplayer:bool)->Result<Output,Boundary>{
 if multiplayer{return Err(Boundary::Multiplayer);}if unit.flags31&4!=0{return Err(Boundary::Active);}if unit.role!=0||unit.group!=255{return Err(Boundary::Membership);}if unit.primary||unit.secondary{return Err(Boundary::Paths);}
 let [x,y]=unit.position;let center=[x.wrapping_add(8),y.wrapping_add(8)];let exit=[center[0],y.wrapping_add(66)];
 //40d050's prefix,45a9b0→53ff0→53f60,52500,438740,45ef00→45f070.
 unit.flags69&=!64;unit.order=1;unit.target_x=i32::from(exit[0]);unit.target_y=exit[1];unit.metadata=4;unit.flags31&=!32;unit.flags69&=0xc7;unit.flags9c|=0x34;unit.icon=0xabb;unit.protection=102;unit.counter6e=0;unit.fieldb3=255;
 Ok(Output{alias:Alias{word18:center[0],word1a:center[1],byte26:1},command:Command{actor:0x5e30c8+slot as u32*192,roster:0x6133b8,arguments:[exit[0],exit[1],4],opcode:1,dispatch_state:0,force_dispatch:0}})
}
impl Unit{
 pub fn from_raw(b:&[u8;192])->Self{let w=|o|i16::from_le_bytes([b[o],b[o+1]]);Self{position:[w(44),w(46)],flags31:b[49],role:b[94]as i8,group:b[95],flags69:b[105],flags9c:b[156],primary:b[20..24]!=[0;4],secondary:b[12..16]!=[0;4],order:b[92]as i8,target_x:i32::from_le_bytes(b[80..84].try_into().unwrap()),target_y:w(84),metadata:w(86),icon:w(168)as u16,protection:b[170],counter6e:b[110],fieldb3:b[179]}}
 ///Only the source-written continuation fields. Every other byte survives.
 pub fn apply(self,b:&mut[u8;192]){b[49]=self.flags31;b[92]=self.order as u8;b[105]=self.flags69;b[156]=self.flags9c;b[80..84].copy_from_slice(&self.target_x.to_le_bytes());b[84..86].copy_from_slice(&self.target_y.to_le_bytes());b[86..88].copy_from_slice(&self.metadata.to_le_bytes());b[168..170].copy_from_slice(&self.icon.to_le_bytes());b[170]=self.protection;b[110]=self.counter6e;b[179]=self.fieldb3;}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn aliases_have_exact_word_word_byte_widths_and_leave_saved_and_lifetime(){let mut o=std::array::from_fn(|i|i as u8+128);let old=o;Alias{word18:432,word1a:556,byte26:1}.apply(&mut o);assert_eq!(&o[20..24],&old[20..24]);assert_eq!(&o[36..38],&old[36..38]);assert_eq!(&o[24..28],&[176,1,44,2]);for i in 0..44{if !(24..28).contains(&i)&&i!=38{assert_eq!(o[i],old[i]);}}}
 #[test]fn unsupported_inputs_are_transactional(){for mode in 0..4{let mut u=Unit{group:255,..Default::default()};match mode{0=>{},1=>u.group=0,2=>u.primary=true,_=>u.flags31=4};let before=u;assert!(complete(&mut u,0,mode==0).is_err());assert_eq!(u,before);}}
 #[test]fn complete_matches_native_warm_unit_object_and_shared_w_deltas(){let native=std::env::var("NATIVE_RETIRED_COMPLETION_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).collect::<Vec<_>>());let mut hash=2166136261u32;let h=|b:&[u8]|b.iter().fold(2166136261u32,|a,n|(a^u32::from(*n)).wrapping_mul(16777619));let decode=|s:&str|s.as_bytes().chunks_exact(2).map(|c|u8::from_str_radix(std::str::from_utf8(c).unwrap(),16).unwrap()).collect::<Vec<_>>();
 for s in 0..64usize{let mut b=[0u8;192];b[3]=71;b[8..12].copy_from_slice(&(if s%2==0{0x59e6c2u32}else{u32::MAX}).to_le_bytes());b[16..20].copy_from_slice(&4378u32.to_le_bytes());b[44..46].copy_from_slice(&[424i16,-1,32767,-32768][s%4].to_le_bytes());b[46..48].copy_from_slice(&[548i16,32767,-32768,-100][s/4%4].to_le_bytes());b[48]=6;b[49]=(s as u8).wrapping_mul(13)&0xfb;b[94]=0;b[95]=255;b[100]=17;b[105]=(s*17)as u8;b[124]=1;b[156]=(s*31)as u8;
 //Oracle setup uses the real original slot and the shared W/dummy, before40d050.
 if let Some(rs)=&native{let before=decode(rs[s*2]["unit"].as_str().unwrap());assert_eq!(b.as_slice(),before,"before scenario{s}");}
 let mut u=Unit::from_raw(&b);let out=complete(&mut u,0,false).unwrap();u.apply(&mut b);let mut objects=[0u8;132];for j in 0..3{objects[j*44..j*44+2].copy_from_slice(&1600i16.to_le_bytes());}for j in 22..44{objects[44+j]=(0xa0+j)as u8;}out.alias.apply((&mut objects[44..88]).try_into().unwrap());let mut w=(s*29)as u8;let mut d=(s*23)as u8;if s%2==0{clear_hardware_motion(&mut w);}else{clear_hardware_motion(&mut d);}
 if let Some(rs)=&native{let r=&rs[s*2+1];assert_eq!(b.as_slice(),decode(r["unit"].as_str().unwrap()),"UNIT{s}");assert_eq!(objects.as_slice(),decode(r["objects24to26"].as_str().unwrap()),"OBJECT{s}");assert_eq!(u64::from(w),r["hardware17"].as_u64().unwrap());assert_eq!(u64::from(d),r["dummy17"].as_u64().unwrap());assert_eq!(r["random"],0);assert_eq!(r["scratchX"],out.command.arguments[0]);assert_eq!(r["scratchY"],out.command.arguments[1]);}
 for n in[s as u32,h(&b),h(&objects),u32::from(w),u32::from(d),out.command.actor,out.command.roster,out.command.arguments[0]as u32,out.command.arguments[1]as u32]{hash=(hash^n).wrapping_mul(16777619);}}
 assert_eq!(hash,3417514213,"ZRound43RetiredCompletionOracle64 real40d050 continuations, source golden");}
}
