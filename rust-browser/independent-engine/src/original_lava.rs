//! Separate32x60 volcanic animation/hazard pool60c860.405350 construction
//!retains untouched bytes of recycled records; it consumes no random numbers.
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Record{pub bytes:[u8;60]}
impl Default for Record{fn default()->Self{let mut r=Self{bytes:[0;60]};r.set_word(0,-1);r}}
impl Record{
 pub fn byte(&self,i:usize)->u8{self.bytes[i]}
 pub fn set_byte(&mut self,i:usize,n:u8){self.bytes[i]=n;}
 pub fn word(&self,i:usize)->i16{i16::from_le_bytes([self.bytes[i],self.bytes[i+1]])}
 pub fn set_word(&mut self,i:usize,n:i16){self.bytes[i..i+2].copy_from_slice(&n.to_le_bytes());}
 pub fn active(&self)->bool{self.word(0)!=-1}
 pub fn position(&self)->[i16;2]{[self.word(0),self.word(2)]}
}
#[derive(Clone,Copy,Debug)]pub struct Draw{pub slot:usize,pub position:[i16;2],pub sprite:i16,pub palette:i8,pub mode:u8,pub parameter:u8,pub offset:i8}
#[derive(Clone,Copy,Debug)]pub struct View{pub camera:[i16;2],pub lower:[i16;2],pub upper:[i16;2]}
///Only406880's pre+96 audio stage. No record/World/allocator is duplicated.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct SoundChance{pub point:[i16;2]}
///Lava uses signed16 subtraction, lower-128 and inclusive upper (no+32).
pub fn visible(point:[i16;2],view:View)->bool{let screen=[point[0].wrapping_sub(view.camera[0]),point[1].wrapping_sub(view.camera[1])];(0..2).all(|i|i32::from(screen[i])>=i32::from(view.lower[i])-128&&screen[i]<=view.upper[i])}

pub trait World{
 fn random(&mut self)->u32;fn cosmetic_random(&mut self)->u32;fn multiplayer(&self)->bool;
 fn view(&self)->View;fn bounds(&self)->[i16;2];fn native_world(&self)->u8;fn resource_present(&self)->bool;
 fn terrain(&self,cell:i16)->u8;fn scare(&mut self,point:[i16;2]);fn burst(&mut self,point:[i16;2]);fn sound(&mut self,chance:SoundChance);
}
pub struct State{pub records:[Record;32],pub draws:Vec<Draw>}
impl Default for State{fn default()->Self{Self{records:std::array::from_fn(|_|Record::default()),draws:Vec::new()}}}
const VELOCITIES:[[i16;2];32]=[[0,-3],[1,-3],[2,-3],[2,-3],[3,-3],[3,-2],[3,-2],[3,-1],[3,0],[3,1],[3,2],[3,2],[3,3],[2,3],[2,3],[1,3],[0,3],[-1,3],[-2,3],[-2,3],[-3,3],[-3,2],[-3,2],[-3,1],[-3,0],[-3,-1],[-3,-2],[-3,-2],[-3,-3],[-2,-3],[-2,-3],[-1,-3]];
const EDGE:[[i16;2];4]=[[0,0],[48,32],[0,32],[-48,32]];
const SLIDE:[[i16;2];4]=[[0,1],[1,1],[0,1],[-1,1]];
fn directional_frame(r:&Record)->u8{((r.byte(40)as i8>>4)&12)as u8}
fn enter_edge(r:&mut Record,w:&mut impl World)->bool{
 if r.byte(49)&8==0{return false;}
 let count=i16::from(r.byte(55)as i8);let speed=if count>7{3}else{if count>=4{2}else{1}};let d=SLIDE[usize::from(r.byte(40)>>6)];
 r.set_word(0,r.word(0).wrapping_add(d[0].wrapping_mul(2*speed-1)));r.set_word(2,r.word(2).wrapping_add(d[1].wrapping_mul(speed-1)));r.set_byte(41,r.byte(41).wrapping_add(1));r.set_byte(47,r.byte(47).wrapping_add(1));
 let first=directional_frame(r);if i16::from(r.byte(41)as i8)==i16::from(first)+4{r.set_byte(41,first);r.set_byte(55,r.byte(55).wrapping_sub(1));if r.byte(55)==0{r.set_word(2,r.word(2).wrapping_add(1));r.set_byte(49,r.byte(49)&!8|32);r.set_byte(55,(w.random()as u8&12)|3);}}
 true
}
fn wait_edge(r:&mut Record,w:&mut impl World)->bool{
 if r.byte(49)&32==0{return false;}r.set_byte(41,r.byte(41).wrapping_add(1));let end=directional_frame(r).wrapping_add(3);
 if i16::from(r.byte(41)as i8)==i16::from(end){r.set_byte(41,(w.random()as u8&2).wrapping_sub(3).wrapping_add(end));r.set_byte(55,r.byte(55).wrapping_sub(1));if r.byte(55)==0{r.set_byte(49,r.byte(49)&!32|16);if r.byte(40)==64{r.set_word(0,r.word(0).wrapping_add(2));}if r.byte(40)==192{r.set_word(0,r.word(0).wrapping_sub(2));}r.set_byte(55,r.byte(56));r.set_byte(41,end);}}
 true
}
fn leave_edge(r:&mut Record)->bool{
 if r.byte(49)&16==0{return false;}let speed=if 9-i16::from(r.byte(55)as i8)>4{3}else{2};let d=SLIDE[usize::from(r.byte(40)>>6)];r.set_word(0,r.word(0).wrapping_add(d[0].wrapping_mul(speed)));r.set_word(2,r.word(2).wrapping_sub(d[1].wrapping_mul(speed-1)));r.set_byte(41,r.byte(41).wrapping_sub(1));r.set_byte(47,r.byte(47).wrapping_sub(1));
 if i16::from(r.byte(41)as i8)==i16::from(directional_frame(r)){r.set_byte(41,directional_frame(r).wrapping_add(3));r.set_byte(55,r.byte(55).wrapping_sub(1));if r.byte(55)==0{if r.byte(40)==0{r.set_word(2,r.word(2).wrapping_add(i16::from(r.byte(47)as i8)));}r.set_byte(49,r.byte(49)&!16);r.set_byte(55,20);r.set_word(41,256);return false;}}
 true
}
fn boundary_turn(r:&mut Record,bounds:[i16;2])->bool{
 let point=r.position();let desired=if point[0]<65{64}else if i32::from(bounds[0].wrapping_sub(64))<=i32::from(point[0])+32{192}else if point[1]<65||i32::from(bounds[1].wrapping_sub(64))<=i32::from(point[1])+32{128}else{return false;};r.set_byte(49,r.byte(49)|3);r.set_byte(53,desired);true
}
///406880 through its external emission boundaries. A true result requests
///the native first-free405350 respawn, which may reuse this same record.
fn advance_record(r:&mut Record,w:&mut impl World,draws:&mut Vec<Draw>)->bool{
 if r.word(24)!=0{r.set_word(24,r.word(24).wrapping_sub(1));return false;}if r.word(28)<0{r.set_word(28,0);}
 let mut point=r.position();let flags=r.byte(49);let sprite=if r.byte(48)&1==0&&flags&64==0{
  if flags&24==0{if flags&32==0{r.word(16).wrapping_add(i16::from(r.byte(41)as i8))}else{point[0]=point[0].wrapping_add(4);point[1]=point[1].wrapping_add(4);r.word(34).wrapping_add(i16::from(r.byte(41)as i8))}}
  else{point[1]=point[1].wrapping_add(4);if matches!(r.byte(40),64|192){point[0]=point[0].wrapping_add(4);}r.word(32).wrapping_add(i16::from(r.byte(41)as i8))}
 }else{r.word(8).wrapping_add(i16::from(r.byte(40)>>6))};
 let view=w.view();let multiplayer=w.multiplayer();
 //MP's rand branch is identically0. Lift only this audio opportunity before
 //canonical culling; its listener evaluates the source window separately.
 //SP keeps its original one visible CRT draw, including when master is off.
 if multiplayer{w.sound(SoundChance{point});}
 if visible(point,view){
  if !multiplayer&&w.cosmetic_random()&127==0{w.sound(SoundChance{point});}
  let mut mode=0;let mut palette=(r.byte(48)&3)as i8;let mut parameter=0;let mut offset=(r.byte(47)as i8).wrapping_sub(64);
  if r.byte(48)&12!=0{mode=33;palette=0;point[1]=point[1].wrapping_add(96);offset=(r.byte(47)as i8).wrapping_sub(96);if r.byte(48)&4!=0{mode=49;parameter=r.byte(46);}if r.word(28)!=0{palette=((r.word(28)as u16>>1)as i8).wrapping_neg();}}
  draws.push(Draw{slot:0,position:point,sprite,palette,mode,parameter,offset});
 }
 r.set_byte(42,r.byte(42).wrapping_sub(1));if r.byte(42)as i8>0{return false;}r.set_byte(42,if flags&32!=0{r.byte(44).wrapping_mul(2)}else{r.byte(44)});
 if flags&128!=0{r.set_byte(46,r.byte(46).wrapping_add(16));r.set_word(28,r.word(28).wrapping_sub(2));if r.word(28)>=0{return false;}w.burst(r.position());r.set_word(0,-1);return true;}
 if flags&56==0{r.set_byte(47,-(r.word(28)>>1)as i8 as u8);}else if enter_edge(r,w)||wait_edge(r,w)||leave_edge(r){return false;}
 w.scare(r.position());let heading=r.byte(40);r.set_byte(46,heading&63);let base=r.word(18).wrapping_add(i16::from(heading>>6).wrapping_mul(i16::from(r.byte(39)as i8)));let end=base.wrapping_add(i16::from(r.byte(39)as i8));r.set_word(16,base);r.set_word(20,end);
 if r.byte(49)&1!=0{if heading&248==r.byte(53){r.set_byte(49,r.byte(49)&!1);}else{r.set_byte(40,if r.byte(49)&2==0{heading.wrapping_add(2)}else{heading.wrapping_sub(2)});}}
 let flags=r.byte(48);if flags&1==0{let increment=if flags&2!=0{true}else{r.set_byte(51,r.byte(51)^1);r.byte(51)!=0};if increment&&r.byte(49)&64==0{r.set_byte(41,r.byte(41).wrapping_add(1));}}
 if i32::from(base)+i32::from(r.byte(41)as i8)==i32::from(end){r.set_byte(41,0);}
 let target=r.word(30);if flags&2==0{if flags&1==0{if r.byte(49)&64!=0{r.set_word(4,r.word(4).wrapping_sub(1));if r.word(4)==0{r.set_word(4,4);let h=r.word(28).wrapping_sub(1).max(target);r.set_word(28,h);if h==target{r.set_byte(49,r.byte(49)&!64);}}}}else{let h=r.word(28).wrapping_sub(1).max(target);r.set_word(28,h);if h==target{r.set_byte(48,flags&!1);}}}else{let h=r.word(28).wrapping_add(1).min(target);r.set_word(28,h);if h==target{r.set_byte(48,flags&!2);}}
 if r.byte(48)&64==0{return false;}
 let velocity=VELOCITIES[usize::from((r.byte(40)as i8>>2)as u8&62)/2];let height=i32::from(r.word(28))+34;for i in 0..2{r.set_word(i*2,r.word(i*2).wrapping_add((i32::from(velocity[i])*height/34)as i16));}
 if r.byte(48)&3==0{if r.byte(49)&1!=0{return false;}if !boundary_turn(r,w.bounds())&&(w.random()&65535)%513<17{let turn=(((w.random()&48)as i16)-32)*2;r.set_byte(53,r.byte(40).wrapping_add(turn as u8)&192);r.set_byte(40,r.byte(40)&240);r.set_byte(49,r.byte(49)&!2|if turn<0{3}else{1});}}
 if r.byte(49)&1!=0||r.byte(48)&3!=0{return false;}
 if w.random()&511<17{r.set_word(30,r.word(28).wrapping_add(((w.random()&65535)%17)as i16).wrapping_add(4).min(34));r.set_byte(49,r.byte(49)&!64);r.set_byte(48,r.byte(48)&!1|2);return false;}
 if r.word(28)!=0{if r.byte(41)as i8==0{let random=w.random()&511;if random<17{r.set_byte(41,0);r.set_word(30,r.word(28).wrapping_sub(((w.random()&65535)%9)as i16+12).max(0));r.set_byte(49,r.byte(49)&!64);r.set_byte(48,r.byte(48)&!2|1);return false;}if random<33{r.set_byte(41,3);r.set_word(30,r.word(28).wrapping_sub(((w.random()&65535)%19)as i16+2).max(0));r.set_byte(49,r.byte(49)|64);r.set_byte(48,r.byte(48)&!3);r.set_word(4,4);return false;}}if r.word(28)!=0{return false;}}
 if r.byte(49)&49!=0{return false;}if r.byte(55)as i8!=0{r.set_byte(55,r.byte(55).wrapping_sub(1));return false;}
 if r.byte(48)&3==0&&r.byte(49)&64==0&&r.word(32)!=-1&&r.byte(40)&63==0{let d=EDGE[usize::from(r.byte(40)>>6)];let x=r.word(0).wrapping_add(d[0]);let y=r.word(2).wrapping_add(d[1]);let cell=y.wrapping_mul(8)&!127;let cell=cell.wrapping_add(x>>4);if [0,1,128,129].into_iter().all(|d|w.terrain(cell.wrapping_add(d))&127==2){r.set_word(55,0x606);if w.random()&127<64{r.set_byte(49,r.byte(49)|8);r.set_byte(41,directional_frame(r));}}}
 false
}

#[cfg(test)]mod tests{
 use super::*;use crate::original_combat::OriginalRng;
 struct Fixture{scenario:u32,rng:OriginalRng}
 impl World for Fixture{
  fn random(&mut self)->u32{self.rng.next_u32()}fn cosmetic_random(&mut self)->u32{panic!("offscreen calls must not reach C rand")}
  fn multiplayer(&self)->bool{self.rng.multiplayer}fn view(&self)->View{View{camera:[10000,10000],lower:[0,0],upper:[320,200]}}
  fn bounds(&self)->[i16;2]{[1024,768]}fn native_world(&self)->u8{3}fn resource_present(&self)->bool{true}
  fn terrain(&self,cell:i16)->u8{if cell>=0&&self.scenario&1!=0{2}else{0}}
  fn scare(&mut self,_:[i16;2]){self.random();}fn burst(&mut self,_:[i16;2]){}fn sound(&mut self,c:SoundChance){assert!(!visible(c.point,self.view()),"offscreen reference has no audible native window");}
 }
 #[test]fn all_16384_complete_native_volcanic_pool_steps_match_bytes_and_rng(){
  let mut hash=2166136261u32;let mut count=0;
  for scenario in 0..128u32{for multi in 0..2{
   let mut p=State::default();let slot=scenario as usize&3;for i in 0..slot{p.records[i].set_word(0,400+i as i16);p.records[i].set_word(24,30000);}
   p.spawn([100+scenario as i16*3,200+scenario as i16*2],(scenario&1)as u8,2,2,3,true);
   let r=&mut p.records[slot];r.set_byte(49,[0,8,16,32,64,128,1,3][scenario as usize%8]);r.set_byte(48,[64,68,65,66,72,64,68,68][scenario as usize/8%8]);r.set_byte(40,(scenario*17)as u8);r.set_byte(41,(scenario%6)as u8);r.set_byte(42,(scenario%4)as u8);r.set_byte(44,(2+scenario%3)as u8);let height=[0,1,3,12][scenario as usize/4%4];r.set_word(28,height);r.set_word(30,(height+3)%35);r.set_word(4,4);r.set_byte(55,(scenario%12)as u8);r.set_byte(56,6);r.set_word(24,if scenario%11==0{2}else{0});
   let mut world=Fixture{scenario,rng:OriginalRng{words:[1+scenario,2,3,4,5],multiplayer:multi!=0}};
   for tick in 1..=64{p.advance(&mut world);assert!(p.draws.is_empty());let bytes=p.records.iter().flat_map(|r|r.bytes).fold(2166136261u32,|h,b|(h^u32::from(b)).wrapping_mul(16777619));let mut row=vec![scenario,multi,tick,bytes];row.extend(world.rng.words);for n in row{hash=(hash^n).wrapping_mul(16777619);}count+=1;}
  }}
  assert_eq!(count,16384);assert_eq!(hash,2365353223,"ZNativeLavaOracle executes406f80/406880 and all five helpers; offscreen, inactive UNITs, disabled external particle pool");
 }
 #[test]fn recycled_constructor_and_visible_native_draw_queues_match_original(){
  let mut init_hash=2166136261u32;for scenario in 0..512u32{let mut pool=State::default();for(slot,r)in pool.records.iter_mut().enumerate(){for(j,b)in r.bytes.iter_mut().enumerate(){*b=((slot*60+j)as u32*17+scenario*13)as u8;}r.set_word(0,if slot<scenario as usize%33{100+slot as i16}else{-1});}pool.spawn([500-scenario as i16,600+scenario as i16],(scenario>>4&1)as u8,(scenario*7)as u8,(scenario*11)as u8,(scenario%5)as u8,scenario>>3&1!=0);let h=pool.records.iter().flat_map(|r|r.bytes).fold(2166136261u32,|h,b|(h^u32::from(b)).wrapping_mul(16777619));for n in [scenario,h]{init_hash=(init_hash^n).wrapping_mul(16777619);}}
  let mut draw_hash=2166136261u32;for scenario in 0..1024u32{let mut r=Record::default();for(i,v)in[(0,150),(2,160),(8,6014),(16,4318),(32,4338),(34,4354)]{r.set_word(i,v);}r.set_byte(40,(scenario*17)as u8);r.set_byte(41,(scenario%6)as u8);r.set_byte(42,2);r.set_byte(47,(scenario*13)as u8);r.set_byte(46,(scenario*7)as u8);r.set_byte(48,[0,1,2,3,4,8,12,68][scenario as usize%8]);r.set_byte(49,[0,8,16,32,64][scenario as usize/8%5]);r.set_word(28,[-1,0,2,34,256][scenario as usize/40%5]);let mut world=Fixture{scenario,rng:OriginalRng{words:[1+scenario,2,3,4,5],multiplayer:true}};let mut draws=vec![];
   //The proof viewport is onscreen; only sound's external sink is isolated.
   struct Onscreen<'a>(&'a mut Fixture);impl World for Onscreen<'_>{fn random(&mut self)->u32{self.0.random()}fn cosmetic_random(&mut self)->u32{panic!("multiplayer skips C rand")}fn multiplayer(&self)->bool{true}fn view(&self)->View{View{camera:[12,20],lower:[0,0],upper:[320,200]}}fn bounds(&self)->[i16;2]{[1024,768]}fn native_world(&self)->u8{3}fn resource_present(&self)->bool{true}fn terrain(&self,c:i16)->u8{self.0.terrain(c)}fn scare(&mut self,_:[i16;2]){panic!("first frame only")}fn burst(&mut self,_:[i16;2]){panic!("first frame only")}fn sound(&mut self,_:SoundChance){}}
   advance_record(&mut r,&mut Onscreen(&mut world),&mut draws);assert_eq!(draws.len(),1);let d=draws[0];for n in [scenario,d.position[0]as u32,d.position[1]as u32,d.sprite as u32,d.palette as i32 as u32,u32::from(d.mode),u32::from(d.parameter),d.offset as i32 as u32]{draw_hash=(draw_hash^n).wrapping_mul(16777619);}
  }
  assert_eq!(init_hash,803941086,"ZNativeLavaInitDrawOracle512 source405350 pool images");assert_eq!(draw_hash,2845882592,"ZNativeLavaInitDrawOracle1024 visible406880 queue records");
 }

}
impl State{
 pub fn clear_for_mission(&mut self){for r in &mut self.records{r.set_word(0,-1);}self.draws.clear();}
 pub fn advance(&mut self,w:&mut impl World){self.draws.clear();if w.native_world()!=3||!w.resource_present(){return;}for slot in 0..32{if !self.records[slot].active(){continue;}let mut record=self.records[slot].clone();let first_draw=self.draws.len();let respawn=advance_record(&mut record,w,&mut self.draws);for draw in &mut self.draws[first_draw..]{draw.slot=slot;}self.records[slot]=record;if respawn{let size=if w.native_world()==2{1}else{2};self.spawn([256,-64],0,size,128,w.native_world(),w.resource_present());self.records[slot].set_word(24,100);}}}
 pub fn spawn(&mut self,point:[i16;2],template:u8,size:u8,interval:u8,native_world:u8,resource_present:bool)->Option<usize>{
  if native_world!=3||!resource_present||template>1{return None;}
  let slot=self.records.iter().position(|r|!r.active())?;let r=&mut self.records[slot];
  //Both recovered10-byte rows at4770fc are identical.
  for(i,n)in[(0,point[0]),(2,point[1]),(8,6014),(16,4318),(18,4318),(20,4323),(24,0),(28,0),(32,4338),(34,4354),(45,0),(48,68)]{r.set_word(i,n);}
  for(i,n)in[(39,5),(40,interval),(41,0),(42,size),(43,255),(44,size),(47,0),(51,0),(55,8)]{r.set_byte(i,n);}
  Some(slot)
 }
 pub fn near(&self,point:[i16;2])->bool{self.records.iter().filter(|r|r.active()).any(|r|{let p=r.position();let dx=i32::from(point[0])-i32::from(p[0]);let dy=i32::from(point[1])-i32::from(p[1]);dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))<4096})}
}
