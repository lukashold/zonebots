//!The current5a080 callback after its phase61 cleanup removed its UNIT.
//!This is not a fresh UNIT handler and never restores31:4 or membership.
#[path="original_retired_unit_preamble_game.rs"]mod game;
use crate::{original_display::{self as display,Record},original_unit_preamble as pre};
pub const PATTERNS:[u32;13]=[0x49acc8,0x49b8e4,0x49badc,0x49b994,0x49bdc0,0x49b808,0x49b85c,0x49be20,0x49b924,0x49b0d4,0x49ae7c,0x49a824,0x49ba58];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{Family(u8),Class(u8),Membership,Paths,Order(i8),AliasByte24,Pattern(u32),Terrain(u8),SlotReuse,CurrentRoster,TransportFlags(u8)}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Unit{
 pub position:[i16;2],pub family:u8,pub class:u8,pub role:i8,pub group:u8,pub flags31:u8,pub flags69:u8,pub flags8a:u8,pub flags9c:u8,pub flags64:u8,
 pub phase:u8,pub frame:i16,pub protection:u8,pub order:i8,pub reload:u8,pub heading:u8,pub elevation:i8,pub cell:i32,pub footprint:u32,pub permission:u32,pub pattern:u32,pub step62:u8,pub word90:i16,
 pub control06:u8,pub flags71:u8,pub bonus8c:i8,pub speed8b:i8,pub pattern_rewritten:bool,
 pub variant3c:u8,pub timer3d:u8,pub frame40:u8,pub timer41:u8,pub delay7d:u8,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Prefix{pub clear_alias20:bool,pub protected_seen:bool,pub frozen:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Completion{pub slot:usize,pub context:u32,pub prefix:Unit,pub unit:Unit,pub frozen:bool,pub picture:Option<Record>}
impl Unit{
 pub fn from_bytes(b:&[u8;192])->Self{let w=|p|i16::from_le_bytes([b[p],b[p+1]]);let d=|p|u32::from_le_bytes(b[p..p+4].try_into().unwrap());Self{position:[w(44),w(46)],family:b[48],class:b[124],role:b[94]as i8,group:b[95],flags31:b[49],flags69:b[105],flags8a:b[138],flags9c:b[156],flags64:b[100],phase:b[53],frame:w(50),protection:b[170],order:b[92]as i8,reload:b[123],heading:b[57],elevation:b[118]as i8,cell:d(16)as i32,footprint:d(24),permission:d(28),pattern:d(32),step62:b[98],word90:w(144),control06:b[6],flags71:b[113],bonus8c:b[140]as i8,speed8b:b[139]as i8,pattern_rewritten:false,variant3c:b[60],timer3d:b[61],frame40:b[64],timer41:b[65],delay7d:b[125]}}
 ///Only the fields written by this continuation; unrelated rawbytes survive.
 pub fn apply(self,b:&mut[u8;192]){b[24..28].copy_from_slice(&self.footprint.to_le_bytes());b[28..32].copy_from_slice(&self.permission.to_le_bytes());b[50..52].copy_from_slice(&self.frame.to_le_bytes());b[53]=self.phase;b[170]=self.protection;b[156]=self.flags9c;b[105]=self.flags69;b[123]=self.reload;b[16..20].copy_from_slice(&self.cell.to_le_bytes());b[98]=self.step62;b[144..146].copy_from_slice(&self.word90.to_le_bytes());b[60]=self.variant3c;b[61]=self.timer3d;b[64]=self.frame40;b[65]=self.timer41;b[125]=self.delay7d;b[100]=self.flags64;if self.pattern_rewritten{for o in[32,172,184]{b[o..o+4].copy_from_slice(&self.pattern.to_le_bytes());}b[139]=self.speed8b as u8;}}
}
///This prefix is unconditional even for a later unsupported order/frozenreturn.
pub fn prefix(u:&mut Unit)->Prefix{if u.class==3{if let Some(t)=pre::collision_tables(u.class,u.family,None,u.flags31,u.role){u.footprint=t.footprint;u.permission=t.permission;}}pre::clamp_frame(u.phase,&mut u.frame);let protected=u.protection!=0;if protected{u.protection=u.protection.wrapping_sub(1);u.flags9c&=0xeb;}Prefix{clear_alias20:protected,protected_seen:protected,frozen:u.flags69&64!=0}}
///Bounded4f1e0→54160(5c0)→44dd00→55f10. The caller supplies an actual
///known pattern step, not a virtual-zero dereference of a nullUNIT20.
pub fn finish(u:&mut Unit,terrain:u8,mut pattern_step:impl FnMut(u32)->Option<u8>,mut random:impl FnMut()->u32)->Result<(),Boundary>{
 if u.family!=255{return Err(Boundary::Family(u.family));}if !matches!(u.class,0|3){return Err(Boundary::Class(u.class));}if u.role!=0||u.group!=255||u.flags31&4!=0{return Err(Boundary::Membership);}if u.order!=0{return Err(Boundary::Order(u.order));}if u.flags8a!=0{return Err(Boundary::TransportFlags(u.flags8a));}
 if u.flags69&64!=0{return Ok(());}
 refresh_position_pattern(u,terrain,&mut pattern_step)?;
 if u.reload!=0{u.reload=u.reload.wrapping_sub(1);} //54160 returns0 for5c0.
 if u.flags31&64!=0{return Err(Boundary::Paths);}u.phase=1;u.frame=1;u.timer3d=8;u.variant3c=(random()&3)as u8;u.timer41=0;u.frame40=u.heading;u.delay7d=((random()as u16)%53+12)as u8;u.flags69&=!32;Ok(())
}
///The nonvehicle, role0, terrain0 portion of4f1e0/451d40.
pub fn refresh_position_pattern(u:&mut Unit,terrain:u8,mut pattern_step:impl FnMut(u32)->Option<u8>)->Result<(),Boundary>{
 let x=i32::from(u.position[0]);let y=i32::from(u.position[1]);let cell=((y*8+64)&0x7ff80)+((x+8)>>4);if x==10000{u.cell=0;return Err(Boundary::Terrain(255));}u.flags69|=4;if !(0..=16384).contains(&cell){u.cell=0;return Err(Boundary::Terrain(255));}u.cell=cell;
 if terrain&127!=0{return Err(Boundary::Terrain(terrain));}u.word90=0;
 //4f4c0/4f509..64 after removal: role0 omits oldGROUP bonus, UNIT71:20
 //selects modifier10, unsigned UNIT6/10 plus signed UNIT8c is clamped0..12.
 let wanted=(i32::from(u.control06)/10+i32::from(u.flags71&32!=0)+i32::from(u.bonus8c)).clamp(0,12)as i8;if u.speed8b!=wanted{u.speed8b=wanted;u.pattern=PATTERNS[wanted as usize];u.pattern_rewritten=true;}
 u.step62=pattern_step(u.pattern).ok_or(Boundary::Pattern(u.pattern))?;
Ok(())}
pub fn picture(u:&mut Unit,camera:[i16;2],size:[i16;2])->Option<Record>{if u.flags69&4==0||u.flags69&64!=0{return None;}let p=[u.position[0].wrapping_sub(camera[0]),u.position[1].wrapping_sub(camera[1])];if p[0]<=-32||p[0]>=size[0]||p[1]<=-48||p[1]>=size[1]{return None;}let r=display::infantry(p,u.phase,u.frame,u.heading as i8,u.elevation,u.flags64)?;u.flags64&=!4;Some(r)}

#[cfg(test)]mod tests{
 use super::*;use crate::original_combat::OriginalRng;
 pub(super) fn fixture(s:usize,pattern:u32)->[u8;192]{let mut b=[0u8;192];b[3]=60;b[4]=15;for(o,n)in[(8,if s/8==0{u32::MAX}else{0x59e6c2}),(16,2580),(24,0x6471a0),(28,0x632698),(32,pattern),(172,pattern),(184,pattern)]{b[o..o+4].copy_from_slice(&n.to_le_bytes());}for(o,n)in[(44,320i16),(46,320),(50,100)]{b[o..o+2].copy_from_slice(&n.to_le_bytes());}b[48]=255;b[95]=255;b[100]=16;b[105]=4+if s/4%2==0{0}else{64};b[123]=3;b[124]=if s/2%2==0{0}else{3};b[125]=4;b[156]=255;b[170]=1;b[181]=255;b}
 pub(super) fn rng(s:usize)->OriginalRng{OriginalRng{words:std::array::from_fn(|i|0x173510a7u32.wrapping_add(s as u32*7919).wrapping_add((i as u32).wrapping_mul(0x9e3779b9))),multiplayer:false}}
 fn run_native_callbacks(multiplayer:bool,oracle:&str){let native=std::env::var(oracle).ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<crate::Value>(s).unwrap()).collect::<Vec<_>>());let decode=|s:&str|s.as_bytes().chunks_exact(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect::<Vec<_>>();let mut hash=2166136261u32;
 for s in 0..16{let mut bytes=fixture(s,PATTERNS[0]);let mut r=Unit::from_bytes(&bytes);let mut random=rng(s);random.multiplayer=multiplayer;random.next_u32();if let Some(rs)=&native{let before=rs.iter().find(|r|r["scenario"]==s&&r["stage"]=="after-cleanup").unwrap();assert_eq!(bytes.as_slice(),decode(before["unit"].as_str().unwrap()),"aftercleanup{s}");assert_eq!(random.words,std::array::from_fn(|i|before[format!("r{i}")].as_u64().unwrap()as u32));}
 let pf=prefix(&mut r);assert!(pf.clear_alias20&&pf.protected_seen);let alias=255u8&0xdf;finish(&mut r,0,|p|(p==PATTERNS[0]).then_some(1),||random.next_u32()).unwrap();let draw=picture(&mut r,[0,0],[1024,1024]);r.apply(&mut bytes);let picture=draw.map_or_else(Vec::new,|r|r.bytes().to_vec());if let Some(rs)=&native{let after=rs.iter().find(|r|r["scenario"]==s&&r["stage"]=="after").unwrap();assert_eq!(bytes.as_slice(),decode(after["unit"].as_str().unwrap()),"full192bytes{s}");assert_eq!(picture,decode(after["queue"].as_str().unwrap()));assert_eq!(alias,decode(after["aliasFF"].as_str().unwrap())[9]);assert_eq!(random.words,std::array::from_fn(|i|after[format!("r{i}")].as_u64().unwrap()as u32));}for n in [s as u32].into_iter().chain(bytes.into_iter().map(u32::from)).chain(picture.into_iter().map(u32::from)).chain([0x610dc0]).chain(random.words){hash=(hash^n).wrapping_mul(16777619);}}
 assert_eq!(hash,if multiplayer{1057529485}else{1672440905},"16 real5a080 bodies; corrected actual UNIT20 patterns, all192 retained bytes/5RNGwords/finalqueue");}
 #[test]fn actual_retired_callback_with_real_patterns_matches_all16_native_final_records_and_rng(){run_native_callbacks(false,"NATIVE_RETIRED_PREAMBLE_ORACLE");}
 #[test]fn actual_mp_callback_matches16_native_records_without_advancing_rng_words(){run_native_callbacks(true,"NATIVE_RETIRED_PREAMBLE_MP_ORACLE");}
 #[test]fn virtual_zero_pattern_is_a_boundary_after_known_prefix(){let mut r=Unit::from_bytes(&fixture(0,0));let pf=prefix(&mut r);assert!(pf.protected_seen);let before=r;assert_eq!(finish(&mut r,0,|_|None,||panic!("no guessed pattern read may consume RNG")),Err(Boundary::Pattern(0)));assert_eq!((r.phase,r.frame,r.protection,r.flags9c),(0,0,0,235));assert_eq!(r.reload,before.reload);}
 #[test]fn real4f1e0_pattern_selection_matches128_raw_native_records_including_warm_pointers(){let native=std::env::var("NATIVE_RETIRED_PATTERN_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|s|serde_json::from_str::<crate::Value>(s).unwrap()).collect::<Vec<_>>());let decode=|s:&str|s.as_bytes().chunks_exact(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect::<Vec<_>>();let mut hash=2166136261u32;for s in 0..128{let control=[0u8,10,60,255][s%4];let bonus=[-1i8,0,3,127][s/4%4];let flag=(s/16%2*32)as u8;let wanted=(i32::from(control)/10+i32::from(flag!=0)+i32::from(bonus)).clamp(0,12)as u8;let mut b=[0u8;192];b[6]=control;b[44..46].copy_from_slice(&320i16.to_le_bytes());b[46..48].copy_from_slice(&320i16.to_le_bytes());b[48]=255;b[95]=255;b[139]=if s/32%2==0{wanted}else{255};b[140]=bonus as u8;b[113]=flag;b[32..36].copy_from_slice(&PATTERNS[s/64%2].to_le_bytes());b[172..176].copy_from_slice(&0x12345678u32.to_le_bytes());b[184..188].copy_from_slice(&0x87654321u32.to_le_bytes());b[144..146].copy_from_slice(&0x1234i16.to_le_bytes());let before=b;let mut r=Unit::from_bytes(&b);refresh_position_pattern(&mut r,0,|p|PATTERNS.iter().position(|&token|token==p).and_then(|i|crate::original_motion::known_pattern_step(i,0))).unwrap();r.apply(&mut b);if let Some(rs)=&native{assert_eq!(before.as_slice(),decode(rs[s]["before"].as_str().unwrap()));assert_eq!(b.as_slice(),decode(rs[s]["after"].as_str().unwrap()),"whole192patternfixture{s}");}for n in[s as u32].into_iter().chain(before.into_iter().map(u32::from)).chain(b.into_iter().map(u32::from)){hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,2170189549,"128actual4f1e0/451d40 calls, no selector/step stubs");}

}
