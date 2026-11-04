//! Actual44e520/45a5c0 permission-table transactions and5aa60 occupancy.
//! Table addresses are stable native tokens, never host memory addresses.
use std::collections::BTreeMap;
use crate::original_collision::Contact;
#[path="original_dynamic_collision_game.rs"]pub mod game;
#[path="original_friendly_contact.rs"]pub mod friendly;
include!("original_dynamic_collision_data.rs");

#[derive(Clone,Debug,PartialEq,Eq)]pub enum Error{MissingMaskByte(u32),MissingTerrain(i16)}
#[derive(Clone,Default)]pub struct Masks{pub world:Option<u8>,bytes:BTreeMap<u32,u8>,pub flag586972:u8,pub flag4d02b7:u8}
impl Masks{
 pub fn initialize(&mut self,world:u8){self.bytes.clear();for(token,worlds)in INITIAL_MASKS{for(i,&value)in worlds[usize::from(world==1)].iter().enumerate(){self.bytes.insert(token.wrapping_add(i as u32),value);}}self.world=Some(world);}
 pub fn read(&self,address:u32)->Result<u8,Error>{self.bytes.get(&address).copied().ok_or(Error::MissingMaskByte(address))}
 pub fn write(&mut self,address:u32,value:u8)->Result<(),Error>{let target=self.bytes.get_mut(&address).ok_or(Error::MissingMaskByte(address))?;*target=value;Ok(())}
 pub fn value(&self,token:u32,code:u8)->Result<u8,Error>{self.read(token.wrapping_add(u32::from(code&127)))}
}
///41b426 stores the current footprint-table value, clamping signed negatives
///to zero. It is independent of the actor's team/color byte.
pub fn initial_terrain_tag(masks:&Masks,footprint:u32,terrain:u8)->Result<i8,Error>{let value=masks.value(footprint,terrain)? as i8;Ok(value.max(0))}
///5a2e0: the returned boolean is the NORTH branch, including unknown tokens.
///South/sideways returnfalse. It is not a generic 'pointer changed' result.
pub fn select_permission(token:&mut u32,heading:u8)->bool{
 const BASE:[u32;9]=[0x499830,0x632698,0x632458,0x49a7e0,0x632650,0x6327b8,0x49a788,0x632770,0x632890];
 const NORTH:[u32;9]=[0x637c88,0x632608,0x6325c0,0x637d18,0x6324a0,0x6323c8,0x637d60,0x632728,0x632530];
 const SOUTH:[u32;9]=[0x499b08,0x6324e8,0x6326e0,0x637da8,0x632848,0x632410,0x637cd0,0x632800,0x632578];
 let north=matches!(heading,0|1|7);if north||matches!(heading,3|4|5){if let Some(i)=BASE.iter().position(|&p|p==*token){*token=if north{NORTH[i]}else{SOUTH[i]};}}north
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct WrapperActor{
 pub position:[i16;2],pub permission:u32,pub footprint:u32,pub terrain_tag:i8,
 pub heading:u8,pub pausebc:u8,pub counter67:i8,
}
///The lazy BUILD query is called only in44e520's NORTH/group-command1 branch.
///A workshop allows66 only for the duration of this single collision query.
pub fn wrapped(actor:&mut WrapperActor,masks:&mut Masks,attempted:[i16;2],vehicle:bool,group_command:i8,mut workshop:impl FnMut()->bool,mut terrain:impl FnMut(i16)->Result<u8,Error>)->Result<(bool,Contact),Error>{
 let original=actor.permission;masks.flag586972=0;masks.flag4d02b7=0;let north=select_permission(&mut actor.permission,actor.heading);
 let(permission66,footprint66)=if vehicle{(masks.value(actor.permission,66)?,masks.value(actor.footprint,66)?)}else{(0,0)};
 if vehicle&&north{let value=if group_command==1&&workshop(){0}else{255};masks.write(actor.permission.wrapping_add(66),value)?;masks.write(actor.footprint.wrapping_add(66),value)?;}
 let addresses=[0,45,53,54,55].map(|offset|actor.permission.wrapping_add_signed(i32::from(actor.terrain_tag)+offset));for address in addresses{masks.write(address,0)?;}
 let mut contact=Contact::new((actor.position[0],actor.position[1]),(attempted[0],attempted[1]));contact.pause=actor.pausebc;let mut error=None;
 let hit=contact.wrapped((attempted[0].wrapping_sub(actor.position[0])as i8,attempted[1].wrapping_sub(actor.position[1])as i8),vehicle,|cell|match terrain(cell).and_then(|code|masks.value(actor.permission,code)){Ok(value)=>value as i8,Err(e)=>{error.get_or_insert(e);-1}});
 actor.pausebc=contact.pause;
 //The original writesFF, rather than restoring these five old values.
 for address in addresses{masks.write(address,255)?;}
 if vehicle{masks.write(actor.footprint.wrapping_add(66),footprint66)?;masks.write(actor.permission.wrapping_add(66),permission66)?;if !hit{actor.counter67=0;}}
 actor.permission=original;if let Some(e)=error{return Err(e);}Ok((hit,contact))
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Occupant{
 pub slot:u8,pub position:[i16;2],pub class:u8,pub phase:u8,pub flags31:u8,
 pub flags71:u8,pub peer6f:u8,pub peer70:u8,pub wait72:i16,
}
pub fn occupancy_cells(position:[i16;2])->[i16;4]{let cell=(position[1]&!15).wrapping_mul(8).wrapping_add(position[0]>>4);[cell,cell.wrapping_add(1),cell.wrapping_add(128),cell.wrapping_add(129)]}
///5aa60. It does not set71:1; only a real contact/yield caller does that.
pub fn occupy(actor:&mut Occupant,hardware:&mut[u8;44],terrain:&mut impl Terrain)->Result<(),Error>{
 if actor.class!=1{actor.phase=22;actor.wait72=17;return Ok(());}
 let cells=occupancy_cells(actor.position);for(i,&cell)in cells.iter().enumerate(){hardware[28+i]=terrain.read(cell)?&127;}
 hardware[23]|=4;actor.flags71|=2;
 for cell in cells{let old=terrain.read(cell)?;terrain.write(cell,(old&128)+19)?;}actor.wait72=17;Ok(())
}
pub trait Terrain{fn read(&mut self,cell:i16)->Result<u8,Error>;fn write(&mut self,cell:i16,value:u8)->Result<(),Error>;}
///5ab40's prefix. The caller owns the subsequent52810/52960 loop, which
///must execute only for active class1 actors after this exact restoration.
pub fn release(actor:&mut Occupant,hardware:&mut[u8;44],terrain:&mut impl Terrain,mut clear_peer:impl FnMut(u8))->Result<bool,Error>{
 if actor.flags71&1==0{return Ok(false);}actor.flags71&=!3;if actor.peer70!=255{clear_peer(actor.peer70);actor.peer70=255;}
 if actor.class!=1{return Ok(false);}if hardware[23]&4!=0{hardware[23]&=!4;for(i,cell)in occupancy_cells(actor.position).into_iter().enumerate(){let old=terrain.read(cell)?;terrain.write(cell,(old&128)|hardware[28+i])?;}}
 Ok(actor.flags31&4!=0)
}
pub fn near_signed_word(a:[i16;2],b:[i16;2])->bool{(0..2).all(|i|{let delta=b[i].wrapping_sub(a[i]);let distance=if delta<0{delta.wrapping_neg()}else{delta};distance<32})}
///5b470. A non-hardware peer queries all21 active members of its own group.
pub fn can_release(actor:&mut Occupant,peer:Occupant,mut near_peer_group:impl FnMut()->bool)->bool{
 if actor.flags71&1==0{return false;}if peer.class==1{if peer.flags31&4!=0{actor.wait72=actor.wait72.wrapping_sub(1);if actor.wait72!=0&&near_signed_word(actor.position,peer.position){return false;}}}else if near_peer_group(){return false;}true
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Handshake{Blocked,Align([i16;2]),Park}
///5ba10's exact alignment/2x2 footprint admission. Callers retain the two
///separate route leaves59c60/52810; their failure does not suppress laterwrites.
pub fn handshake(actor:Occupant,peer:Occupant,mut footprint:impl FnMut(i16)->Result<u8,Error>)->Result<Handshake,Error>{
 let mut x=actor.position[0].wrapping_sub(7);let mut y=actor.position[1].wrapping_sub(4);let misaligned=(x|y)&15!=0;
 if misaligned{x&=!15;if x>peer.position[0]{x=x.wrapping_add(16);}y&=!15;if y>peer.position[1]{y=y.wrapping_add(16);}}
 for cell in occupancy_cells([x,y]){if footprint(cell)?==255{return Ok(Handshake::Blocked);}}
 Ok(if misaligned{Handshake::Align([x.wrapping_add(15),y.wrapping_add(12)])}else{Handshake::Park})
}
pub fn record_handshake(actor:&mut Occupant,peer:&mut Occupant){actor.flags71|=1;actor.peer70=peer.slot;peer.peer6f=actor.slot;}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,ns:impl IntoIterator<Item=u32>){for n in ns{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn wrappers_and_all_restored_table_bytes_match_2048_actual_native_calls(){
  let mut h=2166136261;let dirs=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)];
  for s in 0..2048usize{let vehicle=s/2%3!=0;let team=s/6%3;let(permission,footprint)=if !vehicle{([0x632698,0x632458,0x499830][team],[0x6471a0,0x644990,0x499888][team])}else if s/2%3==2{([0x632650,0x6327b8,0x49a7e0][team],[0x64c258,0x63d1b0,0x49a930][team])}else{([0x632770,0x632890,0x49a788][team],[0x63f990,0x64c1e0,0x49a8a0][team])};let(ax,ay)=(512+[0,1,7,8,9,15][s/3%6],512+[0,1,7,8,9,15][s/5%6]);let(dx,dy)=dirs[s%8];
   let mut actor=WrapperActor{position:[ax-dx,ay-dy],permission,footprint,terrain_tag:[0,1,2,11][s/13%4],heading:(s/18%8)as u8,pausebc:[0,1,31,32,33][s/29%5],counter67:(s*17)as i8};let mut masks=Masks::default();masks.initialize((s%2)as u8);masks.flag586972=77;masks.flag4d02b7=88;let mut queries=0;
   let(hit,c)=wrapped(&mut actor,&mut masks,[ax,ay],vehicle,if s%3==0{1}else{5},||{queries+=1;s%7!=0&&s%5==0},|cell|Ok(((cell as u16 as usize+s/7)%67)as u8|(s%2*128)as u8)).unwrap();
   let mut table_hash=2166136261;for(token,_)in INITIAL_MASKS{for j in 0..72{hash(&mut table_hash,[u32::from(masks.read(token+j).unwrap())]);}}
   hash(&mut h,[s as u32,u32::from(hit),queries,c.position.0 as u32,c.position.1 as u32,actor.permission,u32::from(actor.pausebc),actor.counter67 as u32,c.x.unwrap_or(-1)as u32,c.y.unwrap_or(-1)as u32,u32::from(c.axes),u32::from(masks.flag586972),u32::from(masks.flag4d02b7),table_hash]);
  }
  assert_eq!(h,165255187,"ZNov04DynamicWrapperOracle; only41c290 BUILD query is a recorded input");
 }

 struct Plane(Vec<u8>);
 impl Terrain for Plane{fn read(&mut self,c:i16)->Result<u8,Error>{self.0.get(c as u16 as usize).copied().ok_or(Error::MissingTerrain(c))}fn write(&mut self,c:i16,v:u8)->Result<(),Error>{*self.0.get_mut(c as u16 as usize).ok_or(Error::MissingTerrain(c))?=v;Ok(())}}
 fn release_fixture(actor:&mut Occupant,peer:&mut Occupant,hw:&mut[u8;44],terrain:&mut Plane,s:usize,resume:&mut u32,checks:&mut u32,trace:&mut u32){
  let run=release(actor,hw,terrain,|slot|{assert_eq!(slot,1);peer.peer6f=255;}).unwrap();
  if run{loop{*resume+=1;hash(trace,[2]);if *resume>s as u32%3{hw[23]=(hw[23]&!2)|1;break;}*checks+=1;hash(trace,[3]);if *checks>(s/3%3)as u32{break;}}}
 }
 #[test]fn occupancy_handshake_restore_and_wait_match_1024_native_callbacks(){
  let mut h=2166136261;
  for mode in 0..4u32{for s in 0..256usize{
   let mut terrain=Plane((0..16384).map(|j|(((j*3+s*17)%67)|(s%2*128))as u8).collect());let mut hw=std::array::from_fn(|j|(s*19+j*7)as u8);
   let position=[519+(s%3)as i16,516+(s/3%3)as i16];let mut actor=Occupant{slot:0,position,class:(s/2%2)as u8,phase:9,flags31:if s%5==0{0}else{4},flags71:(s*17)as u8,peer6f:(s*23)as u8,peer70:if mode==2{1}else if s%7==0{255}else{1},wait72:[0,1,2,17,-1,-32768][s/3%6]};
   let delta=[-33,-32,-31,0,31,32,33,-32768];let dx=delta[s/5%8];let dy=delta[s/7%8];let mut peer=Occupant{slot:1,position:[position[0].wrapping_add(dx),position[1].wrapping_add(dy)],class:if s%3==0{1}else{0},flags31:if s%7==0{0}else{4},peer6f:(s*13)as u8,..Default::default()};
   let(mut resume,mut checks,mut trace,mut result)=(0u32,0u32,2166136261u32,0u32);
   match mode{
    0=>occupy(&mut actor,&mut hw,&mut terrain).unwrap(),
    1=>release_fixture(&mut actor,&mut peer,&mut hw,&mut terrain,s,&mut resume,&mut checks,&mut trace),
    2=>{let near=s%13!=0&&(0..21usize).any(|j|(s+j)%4!=0&&(s+j)%5!=0&&near_signed_word(position,[position[0].wrapping_add(dx).wrapping_add((j%3)as i16*31-31),position[1].wrapping_add(dy).wrapping_add((j%5)as i16*16-32)]));if can_release(&mut actor,peer,||near){result=1;release_fixture(&mut actor,&mut peer,&mut hw,&mut terrain,s,&mut resume,&mut checks,&mut trace);}},
    _=>{let plan=handshake(actor,peer,|c|{let code=terrain.read(c)?&127;Ok(if(s+usize::from(code))%11==0{255}else{code%5})}).unwrap();if plan!=Handshake::Blocked{if let Handshake::Align(p)=plan{hash(&mut trace,[1,p[0]as u32,p[1]as u32]);resume+=1;hash(&mut trace,[2]);hw[23]=(hw[23]&!2)|1;}record_handshake(&mut actor,&mut peer);if plan==Handshake::Park{occupy(&mut actor,&mut hw,&mut terrain).unwrap();}}}
   }
   let mut hw_hash=2166136261;hash(&mut hw_hash,hw.into_iter().map(u32::from));let mut terrain_hash=2166136261;hash(&mut terrain_hash,terrain.0.into_iter().map(u32::from));
   hash(&mut h,[mode,s as u32,result,resume,checks,trace,u32::from(actor.phase),u32::from(actor.flags71),u32::from(actor.peer6f),u32::from(actor.peer70),actor.wait72 as u32,u32::from(peer.peer6f),2,hw_hash,terrain_hash]);
  }}
  assert_eq!(h,2596835704,"ZNov04OccupancyOracleV2: real5aa60/5ab40/5b470/5b3c0/5ba10; recorded route leaves");
 }

 #[test]fn directional_caller_retains_corrected_unit_and_distinct_proposed_globals_at518e0(){
  let mut h=2166136261;
  for s in 0..1024usize{let vehicle=s/2%3!=0;let team=s/6%3;let(permission,footprint)=if !vehicle{([0x632698,0x632458,0x499830][team],[0x6471a0,0x644990,0x499888][team])}else if s/2%3==2{([0x632650,0x6327b8,0x49a7e0][team],[0x64c258,0x63d1b0,0x49a930][team])}else{([0x632770,0x632890,0x49a788][team],[0x63f990,0x64c1e0,0x49a8a0][team])};let initial=[512+[0,1,7,8,9,15][s/3%6],512+[0,1,7,8,9,15][s/5%6]];
   let mut actor=WrapperActor{position:initial,permission,footprint,terrain_tag:[0,1,2,11][s/13%4],heading:(s/18%8)as u8,pausebc:[0,1,31,32,33][s/29%5],counter67:(s*17)as i8};let mut masks=Masks::default();masks.initialize((s%2)as u8);let(mut proposed,mut error,mut remaining,mut counter66)=(initial,2i16,4i16,(s*13)as u8);let mut c=Contact::new((initial[0],initial[1]),(initial[0],initial[1]));let mut hit=false;let step=(s/4%4)as i16;
   if step!=0{for _ in 0..step{error=error.wrapping_sub((s%4)as i16);if error<0{error=error.wrapping_add(4);proposed[1]=proposed[1].wrapping_add(if s%2==0{1}else{-1});}proposed[0]=proposed[0].wrapping_sub(1);}
    (hit,c)=wrapped(&mut actor,&mut masks,proposed,vehicle,if s%3==0{1}else{5},||s%7!=0&&s%5==0,|cell|Ok(((cell as u16 as usize+s/7)%67)as u8|(s%2*128)as u8)).unwrap();if !hit{remaining-=step;counter66=0;}
   }
   hash(&mut h,[s as u32,u32::from(hit),c.position.0 as u32,c.position.1 as u32,remaining as u32,error as u32,proposed[0]as u32,proposed[1]as u32,c.x.unwrap_or(-1)as u32,c.y.unwrap_or(-1)as u32,u32::from(c.axes),u32::from(counter66),u32::from(actor.pausebc),actor.counter67 as u32]);
  }
  assert_eq!(h,2305097233,"real51da0→44e520/45a5c0→collision; only518e0 final handler and BUILDquery recorded");
 }
}
