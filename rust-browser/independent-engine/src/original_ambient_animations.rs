//! The independent150x48 animation records at60cfe0. These records are not
//! UNITs, OBJECTs, heavy projectiles or either transient particle pool.
#[path="original_ambient_animations_game.rs"]mod game;
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Record{pub bytes:[u8;48]}
impl Default for Record{fn default()->Self{let mut r=Self{bytes:[0;48]};r.set_word(0,-1);r}}
impl Record{pub fn byte(&self,i:usize)->u8{self.bytes[i]}pub fn set_byte(&mut self,i:usize,v:u8){self.bytes[i]=v;}pub fn word(&self,i:usize)->i16{i16::from_le_bytes([self.bytes[i],self.bytes[i+1]])}pub fn set_word(&mut self,i:usize,v:i16){self.bytes[i..i+2].copy_from_slice(&v.to_le_bytes());}pub fn active(&self)->bool{self.word(0)!=-1}pub fn position(&self)->[i16;2]{[self.word(0),self.word(2)]}}
#[derive(Clone,Copy,Debug,Default)]pub struct Globals{pub sprite:i16,pub word14:i16,pub word18:i16}
#[derive(Clone,Copy,Debug)]pub struct Draw{pub slot:u8,pub position:[i16;2],pub screen:[i16;2],pub sprite:i16,pub palette:i8,pub shade:u8,pub mode:u8,pub parameter:u8}
#[derive(Clone,Copy,Debug)]pub struct Submission{pub quad:bool,pub palette:Option<u8>,pub y_offset:i8,pub size:i16,pub angle:i16}
///4549f0/54630. A quad can inherit the preceding queue entry's palette;
///None preserves that dependency instead of choosing a team or neutral color.
pub fn submission(mode:u8,palette:i8,shade:u8,parameter:u8)->Submission{
 if mode==0{return Submission{quad:false,palette:Some(match palette as u8{p@0..=3=>p,4..=7=>255,_=>4}),y_offset:shade as i8,size:0,angle:0};}
 let quad=palette<0||mode&16!=0;let masked=palette as u8&7;Submission{quad,palette:if palette<0{None}else if masked<=3{Some(masked)}else if quad{None}else{Some(255)},y_offset:shade as i8,size:if quad{-i16::from(palette)}else{0},angle:if quad&&mode&16!=0{255-i16::from(parameter as i8)}else{0}}
}
pub struct State{pub records:Vec<Record>,pub globals:Globals,pub draws:Vec<Draw>,pub queue:Vec<u8>,pub extra:Vec<u8>,extra_storage:Vec<u8>}
impl Default for State{fn default()->Self{Self{records:vec![Record::default();150],globals:Globals::default(),draws:Vec::new(),queue:Vec::new(),extra:Vec::new(),extra_storage:vec![0;2400]}}}
pub trait World{fn emit(&mut self,state:&mut State,source_slot:usize,position:[i16;2],heading:i16);}
impl State{
 pub fn clear_for_mission(&mut self){for r in &mut self.records{r.set_word(0,-1);}self.draws.clear();self.queue.clear();self.extra.clear();}
 pub fn spawn_water(&mut self,p:[i16;2])->Option<usize>{let slot=self.records.iter().position(|r|!r.active())?;let r=&mut self.records[slot];for offset in[0,4,8]{r.set_word(offset,p[0]);r.set_word(offset+2,p[1]);}r.set_word(32,2);r.set_word(34,2);r.bytes[38..42].fill(0);r.set_word(26,0);r.set_word(20,3034);r.set_word(22,3038);r.set_byte(31,0);Some(slot)}
 ///41dbf0 obtains C rand BEFORE first-free allocation. The caller supplies
 ///that draw (zero in multiplayer), including when this150-slot pool is full.
 pub fn spawn_spark(&mut self,point:[i16;2],random:u16)->Option<usize>{let p=[point[0],point[1].wrapping_add(24)];let slot=self.records.iter().position(|r|!r.active())?;let r=&mut self.records[slot];for o in[0,4,8]{r.set_word(o,p[0]);r.set_word(o+2,p[1]);}r.set_word(32,1);r.set_byte(34,1);r.bytes[38..42].copy_from_slice(&236u32.to_le_bytes());if self.duplicate(slot,p){self.records[slot].set_word(0,-1);return None;}let r=&mut self.records[slot];r.set_word(26,1);r.set_word(20,0xb4d+(random&3)as i16);r.set_word(22,0xb4e+(random&3)as i16);r.set_byte(33,255);r.set_word(35,0xa01);r.set_byte(41,127);r.set_byte(39,40);r.set_word(12,2);r.set_byte(31,0);Some(slot)}
 ///41d4b0 includes inactive records and compares both current coordinates
 ///and words4/6; those words are supplied by the original persistent globals.
 fn duplicate(&self,slot:usize,p:[i16;2])->bool{self.records.iter().enumerate().any(|(i,r)|i!=slot&&r.position()==p&&[r.word(4),r.word(6)]==p)}
 pub fn spawn(&mut self,kind:u8,point:[i16;2],source35:u8)->Option<usize>{
  if ![1,2].contains(&kind){return None;}let slot=self.records.iter().position(|r|!r.active())?;let p=if kind==1{point}else{[point[0].wrapping_sub(8),point[1].wrapping_sub(16)]};let r=&mut self.records[slot];r.set_word(0,p[0]);r.set_word(2,p[1]);r.set_word(4,self.globals.word18);r.set_word(6,self.globals.word14);r.set_word(8,p[0]);r.set_word(10,p[1]);r.set_word(32,1);r.set_byte(34,1);r.bytes[38..42].copy_from_slice(&(if kind==1{226u32}else{236}).to_le_bytes());
  if self.duplicate(slot,p){self.records[slot].set_word(0,-1);return None;}let r=&mut self.records[slot];r.set_word(26,0);
  if kind==1{r.set_word(20,2923);r.set_word(22,2935);r.set_byte(35,source35);let limit=(i16::from(source35 as i8)+16)as u8;if limit!=0{r.set_byte(36,limit);r.set_byte(39,8);}}
  else{r.set_word(20,self.globals.sprite.wrapping_add(25));r.set_word(22,self.globals.sprite.wrapping_add(26));r.set_byte(33,255);r.set_word(35,0x401);r.set_byte(41,127);r.set_byte(39,40);r.set_word(12,1);}
  r.set_byte(31,0);Some(slot)
 }
 fn emit_record(&mut self,slot:usize,point:[i16;2],world:&mut impl World){let heading=i16::from(self.records[slot].byte(30)as i8);world.emit(self,slot,point,heading);}
 fn inherit_globals(&mut self,slot:usize){let r=&self.records[slot];self.globals=Globals{sprite:r.word(20),word14:r.word(6),word18:r.word(4)};}
 pub fn advance(&mut self,camera:[i16;2],world:&mut impl World){self.draws.clear();self.queue.clear();self.extra.clear();
  for slot in 0..self.records.len().min(150){
   if !self.records[slot].active(){continue;}let delay=self.records[slot].word(26);if delay!=0{let r=&mut self.records[slot];r.set_word(26,delay.wrapping_sub(1));if r.byte(40)&4==0{continue;}}
   let r=&self.records[slot];let mut screen=[r.word(0).wrapping_sub(camera[0]),r.word(2).wrapping_sub(camera[1])];let flags=r.byte(39);if flags&8!=0&&r.byte(35)==0{screen[0]=screen[0].wrapping_add(2);}let sprite=r.word(20).wrapping_add(i16::from(r.byte(31)as i8));let mut palette=(flags&3)as i8;let mut mode=0u8;let mut parameter=0;
   self.queue.extend(screen[0].to_le_bytes());self.queue.extend(screen[1].to_le_bytes());
   if flags&12==0{self.queue.extend(sprite.to_le_bytes());self.queue.push(flags&3);self.queue.push(r.byte(38));}
   else{let start=self.extra.len();self.queue.extend(((start/16)as i16).to_le_bytes());self.queue.extend((-1i16).to_le_bytes());let row=&mut self.extra_storage[start..start+16];row[0..2].copy_from_slice(&screen[0].to_le_bytes());row[2..4].copy_from_slice(&screen[1].to_le_bytes());row[4..6].copy_from_slice(&sprite.to_le_bytes());row[6]=flags&3;row[7]=33;row[14]=r.byte(38);if flags&4!=0{parameter=r.byte(37);row[9]=parameter;row[7]=49;}if flags&8!=0{palette=(r.byte(35)as i8).wrapping_neg();row[6]=palette as u8;}mode=row[7];self.extra.extend_from_slice(row);}
   self.draws.push(Draw{slot:slot as u8,position:r.position(),screen,sprite,palette,shade:r.byte(38),mode,parameter});
   if self.records[slot].byte(40)&4!=0{if self.records[slot].word(26)!=0{continue;}if self.records[slot].byte(43)!=0{let r=&mut self.records[slot];r.set_byte(43,r.byte(43).wrapping_sub(1));continue;}}
   let r=&mut self.records[slot];r.set_byte(32,r.byte(32).wrapping_sub(1));if r.byte(32)!=0{continue;}r.set_byte(32,r.byte(34));
   if r.byte(39)&8!=0{r.set_byte(35,r.byte(35).wrapping_add(1));r.set_word(2,r.word(2).wrapping_sub(2));if r.byte(35)==r.byte(36){let point=r.position();r.set_word(0,-1);if r.byte(41)==127{self.inherit_globals(slot);self.records[slot].set_word(24,0);self.records[slot].set_byte(39,0);self.emit_record(slot,point,world);}continue;}}
   let r=&mut self.records[slot];if r.byte(39)&32!=0{r.set_word(24,r.word(24).wrapping_add(1));if r.byte(41)!=127&&i16::from(r.byte(41)as i8)==r.word(24){let point=r.position();self.emit_record(slot,point,world);}}
   let r=&mut self.records[slot];r.set_byte(31,if r.byte(39)&16==0{r.byte(31).wrapping_add(1)}else{r.byte(31).wrapping_sub(1)});if i32::from(r.word(22))!=i32::from(r.word(20))+i32::from(r.byte(31)as i8){continue;}
   if r.byte(33)!=255{if r.byte(33)!=0{r.set_byte(33,r.byte(33).wrapping_sub(1));}if r.byte(33)==0{let point=r.position();r.set_word(0,-1);if r.byte(41)==127{self.inherit_globals(slot);self.records[slot].set_word(24,0);self.records[slot].set_byte(39,0);self.emit_record(slot,point,world);}continue;}}
   let r=&mut self.records[slot];r.set_byte(31,0);if r.byte(40)&4!=0{r.set_word(26,r.word(28));}
  }
 }
 pub fn bytes(&self)->Vec<u8>{self.records.iter().flat_map(|r|r.bytes).collect()}
}
#[cfg(test)]mod tests{
 use super::*;
 fn bytes_hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619))}
 fn values(h:&mut u32,ns:&[u32]){for &n in ns{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn final_submission_matches_all_432_executed_native_draws(){let mut hash=2166136261u32;let mut scenario=0;for mode in[0,33,49]{for palette in[-128,-30,-1,0,1,3,4,7,8,127]{if mode==0&&palette<0{continue;}for offset in[0i8,30,-30,-1]{for parameter in[0,127,128,255]{let draw=submission(mode,palette,offset as u8,parameter);values(&mut hash,&[scenario,u32::from(mode),palette as u32,offset as u32,u32::from(parameter),if draw.quad{2}else{1},500,(600i16+i16::from(draw.y_offset))as u32,1000,u32::from(draw.palette.unwrap_or(2)),draw.size as u32,draw.angle as u32]);scenario+=1;}}}}assert_eq!((scenario,hash),(432,3950909877),"ZNativeAnimationDrawOracle4549f0/54630finalgraphicscalls");}
 #[test]fn constructors_match_all_1024_executed_native_pool_images(){let mut hash=2166136261u32;for kind in 0..2u8{for s in 0..512u32{let mut pool=State::default();for(i,r)in pool.records.iter_mut().enumerate(){for(j,b)in r.bytes.iter_mut().enumerate(){*b=((i*48+j)*17+s as usize)as u8;}let free=[0,1,149,150][s as usize&3];r.set_word(0,if i<free{100+i as i16}else{-1});}let p=[346+(s%9)as i16,420+(s/9%9)as i16];pool.globals=Globals{sprite:(s*17)as i16,word14:if s&4!=0{p[1]}else{612},word18:if s&4!=0{p[0]}else{511}};if s&8!=0{let r=&mut pool.records[1];let point=if kind==0{p}else{[p[0]-8,p[1]-16]};r.set_word(0,point[0]);r.set_word(2,point[1]);r.set_word(4,point[0]);r.set_word(6,point[1]);}pool.spawn(kind+1,p,(s*13)as u8);values(&mut hash,&[u32::from(kind)*512+s,bytes_hash(&pool.bytes())]);}}assert_eq!(hash,2686973327,"ZNativeAmbientAnimationOracle1024native41d500/41d5d0poolimages");}
 #[test]fn repair_sparks_match_all512_native_full_recycled_and_duplicate_pool_images(){let mut h=2166136261u32;for s in 0..512usize{let mut pool=State::default();let free=[0,1,149,150][s%4];for(j,r)in pool.records.iter_mut().enumerate(){for(k,b)in r.bytes.iter_mut().enumerate(){*b=((j*48+k)*17+s)as u8;}r.set_word(0,if j<free{100+j as i16}else{-1});}if s%8==0{let r=&mut pool.records[1];for o in[0,4]{r.set_word(o,346);r.set_word(o+2,444);}}let crt=if s%3==0{0}else{1};let random=if crt==0{0}else{((s*3571+0x1743)&32767)as u16};pool.spawn_spark([346,420],random);values(&mut h,&[s as u32,bytes_hash(&pool.bytes()),crt]);}assert_eq!(h,996811846,"ZNativeWorkshopOracle512actual41dbf0ctors; C rand precedes full-pool and duplicate tests");}
 #[test]fn water_constructor_matches_all_256_executed_native_pool_images(){let mut hash=2166136261u32;for s in 0..256u32{let mut pool=State::default();let free=[0,1,149,150][s as usize&3];for(i,r)in pool.records.iter_mut().enumerate(){for(j,b)in r.bytes.iter_mut().enumerate(){*b=((i*48+j)*17+s as usize)as u8;}r.set_word(0,if i<free{100+i as i16}else{-1});}pool.spawn_water([346+(s%9)as i16,420+(s/9%9)as i16]);values(&mut hash,&[s,bytes_hash(&pool.bytes())]);}assert_eq!(hash,495843603,"ZNativeAmbientWaterOracle256original41dcc0poolimages");}
 struct Trace{events:u32,hash:u32}
 impl World for Trace{fn emit(&mut self,state:&mut State,source:usize,p:[i16;2],heading:i16){self.events+=1;values(&mut self.hash,&[source as u32,p[0]as u32,p[1]as u32,heading as u32,state.records[source].word(18)as u32,state.globals.sprite as u32,state.globals.word14 as u32,state.globals.word18 as u32]);}}
 #[test]fn all_12288_native_animation_callbacks_match_pool_draws_and_emission_order(){let mut hash=2166136261u32;for s in 0..128u32{let mut pool=State::default();pool.globals=Globals{sprite:(s*17)as i16,word14:612,word18:511};pool.spawn(if s&1==0{1}else{2},[346,420],(s*13)as u8);for tick in 0..96u32{let mut trace=Trace{events:0,hash:2166136261};pool.advance([(s%3*12)as i16,(s%5*8)as i16],&mut trace);values(&mut hash,&[s*96+tick,bytes_hash(&pool.bytes()),pool.queue.len()as u32,bytes_hash(&pool.queue),pool.extra.len()as u32,bytes_hash(&pool.extra),trace.events,trace.hash]);}}assert_eq!(hash,2847959541,"ZNativeAmbientAnimationOracle12288real41d6a0callbacks");}
 #[test]fn water_animation_matches_all_1536_executed_native_callbacks(){let mut hash=2166136261u32;for s in 0..64u32{let mut pool=State::default();pool.spawn_water([346+s as i16,420-s as i16]);for tick in 0..24u32{let mut trace=Trace{events:0,hash:2166136261};pool.advance([(s%3*12)as i16,(s%5*8)as i16],&mut trace);values(&mut hash,&[s*24+tick,bytes_hash(&pool.bytes()),pool.queue.len()as u32,bytes_hash(&pool.queue)]);}}assert_eq!(hash,457392357,"ZNativeAmbientWaterOracle1536original41d6a0watercallbacks");}
}
