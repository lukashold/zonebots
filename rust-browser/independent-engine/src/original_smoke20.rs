//! Native20x6 cosmetic pool5a112e, advanced by461970. This is distinct
//! from OBJECT fires and from the500x44 world-particle allocator.
#[path="original_cosmetics20_game.rs"]mod game;
#[path="original_tracks.rs"]pub(crate) mod tracks;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Record{pub position:[i16;2],pub frame:u8,pub kind:u8}
impl Default for Record{fn default()->Self{Self{position:[-1,0],frame:0,kind:0}}}
#[derive(Clone,Copy,Debug)]pub struct Draw{pub slot:u8,pub position:[i16;2],pub screen:[i16;2],pub sprite:i16}
pub struct State{pub records:[Record;20],pub draws:Vec<Draw>,pub tracks:tracks::State}
impl Default for State{fn default()->Self{Self{records:[Record::default();20],draws:vec![],tracks:Default::default()}}}
const MOVEMENT:[[i16;2];8]=[[0,-1],[-1,0],[-2,-1],[-2,-1],[0,-2],[1,-2],[2,-2],[1,0]];
impl State{
 pub fn clear_for_mission(&mut self){for r in &mut self.records{r.position[0]=-1;}self.draws.clear();self.tracks.clear_for_mission();}
 ///461b00 and43f100's inline version initialize the same three words.
 pub fn spawn(&mut self,point:[i16;2])->Option<usize>{let slot=self.records.iter().position(|r|r.position[0]==-1)?;self.records[slot]=Record{position:point,frame:4,kind:0};Some(slot)}
 pub fn advance(&mut self,camera:[i16;2],pulse:bool){self.draws.clear();for(slot,r)in self.records.iter_mut().enumerate(){if r.position[0]==-1{continue;}let x=r.position[0].wrapping_sub(camera[0]);let negative_y=r.position[1]<0;let y=if r.kind&128==0&&negative_y{r.position[1].wrapping_neg().wrapping_sub(camera[1])}else{r.position[1].wrapping_sub(camera[1])};let sprite=if r.kind&128!=0{i16::from(r.frame&127)+1806}else if negative_y{((i16::from(r.kind as i8)+317)*4).wrapping_add(i16::from(r.frame as i8))}else{(i16::from(r.kind as i8)*7+1408).wrapping_add(i16::from(r.frame as i8))};self.draws.push(Draw{slot:slot as u8,position:[r.position[0],y.wrapping_add(camera[1])],screen:[x,y],sprite});
  if r.kind&128!=0{let d=MOVEMENT[usize::from(r.kind&7)];r.position[0]=r.position[0].wrapping_add(d[0]);r.position[1]=r.position[1].wrapping_add(d[1]);if pulse{r.frame=r.frame.wrapping_add(1);if r.frame==6{r.position[0]=-1;}}}
  else if pulse{r.frame=r.frame.wrapping_add(1);if negative_y{if r.frame==4{r.frame=5;r.position[1]=r.position[1].wrapping_neg();}}else if r.frame==7{r.position[0]=-1;}}
 }}
 pub fn bytes(&self)->[u8;120]{let mut out=[0;120];for(i,r)in self.records.iter().enumerate(){out[i*6..i*6+2].copy_from_slice(&r.position[0].to_le_bytes());out[i*6+2..i*6+4].copy_from_slice(&r.position[1].to_le_bytes());out[i*6+4]=r.frame;out[i*6+5]=r.kind;}out}
}
#[cfg(test)]mod tests{
 use super::*;use super::super::original_bullet_impacts20 as impact;
 fn bytes_hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619))}fn values(h:&mut u32,ns:&[u32]){for &n in ns{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn both_constructors_match_1024_executed_native_pool_images(){let mut hash=2166136261u32;for kind in 0..2u32{for s in 0..512u32{let free=[0,1,19,20][s as usize&3];let bytes=if kind==0{let mut state=State::default();for(i,r)in state.records.iter_mut().enumerate(){let b=(0..6).map(|j|((i*6+j)*17+s as usize)as u8).collect::<Vec<_>>();r.position=[if i<free{100+i as i16}else{-1},i16::from_le_bytes([b[2],b[3]])];r.frame=b[4];r.kind=b[5];}state.spawn([346,420]);state.bytes().to_vec()}else{let mut state=impact::State::default();for(i,r)in state.records.iter_mut().enumerate(){let b=(0..8).map(|j|((i*8+j)*17+s as usize)as u8).collect::<Vec<_>>();r.position=[if i<free{100+i as i16}else{-1},i16::from_le_bytes([b[2],b[3]])];r.frame=i16::from_le_bytes([b[4],b[5]]);r.flags=b[6];r.stale=b[7];}state.spawn([346,420],(s&127)as u8);state.bytes().to_vec()};values(&mut hash,&[kind*512+s,bytes_hash(&bytes)]);}}assert_eq!(hash,4134284773,"ZNativeCosmetic20Oracle1024first-free smoke/ground-impactconstructors");}
 #[test]fn both_pools_match_8192_executed_native_callbacks(){let mut hash=2166136261u32;let points=[[500i16,600],[-9,-600],[0,-32768],[32767,32767]];for kind in 0..2u32{for s in 0..128u32{let mut smoke=State::default();let mut ground=impact::State::default();for i in 0..3{let p=points[(s as usize+i)%4];let p=[p[0].wrapping_add(i as i16*8),p[1]];if kind==0{smoke.records[i]=Record{position:p,frame:(s*13+i as u32*37)as u8,kind:(s as u8%8)|((s as u8>>3&1)*128)};}else{ground.records[i]=impact::Record{position:p,frame:(s*17+i as u32*13)as i16,flags:s as u8&1,stale:(s*7)as u8};}}for tick in 0..32u32{let camera=[(s%3*12)as i16,(s%5*8)as i16];let(bytes,draws)=if kind==0{smoke.advance(camera,tick&1==0);(smoke.bytes().to_vec(),smoke.draws.iter().map(|d|(d.screen,d.sprite)).collect::<Vec<_>>())}else{ground.advance(camera,[320,200],impact::Profile::for_world((s%5)as u8),tick&1==0);(ground.bytes().to_vec(),ground.draws.iter().map(|d|(d.screen,d.sprite)).collect())};let mut draw_bytes=Vec::new();for(p,sprite)in draws{for n in[p[0],p[1],sprite,255]{draw_bytes.extend(n.to_le_bytes());}}values(&mut hash,&[kind*4096+s*32+tick,bytes_hash(&bytes),draw_bytes.len()as u32,bytes_hash(&draw_bytes)]);}}}assert_eq!(hash,3939176939,"ZNativeCosmetic20Oracle8192real461970/464d90callbacks");}
}
