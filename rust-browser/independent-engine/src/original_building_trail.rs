//! Fixed shared destruction trails. Numeric native addresses preserve aliases;
//! unmapped accesses stop at a checked boundary, never clamp source counters.
use super::effects::{Particle,bridge::{self,Globals,World,Constructor}};
pub const BASE:u32=0x5ba0a8;
pub const BLOCK:usize=0x2794;
pub const JOB_BASE:u32=0x62d5f8;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Boundary{pub address:u32,pub length:usize}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Job(pub [u8;24]);
impl Job{
 pub fn word(&self,n:usize)->i16{i16::from_le_bytes(self.0[n..n+2].try_into().unwrap())}
 pub fn ptr(&self,n:usize)->u32{u32::from_le_bytes(self.0[n..n+4].try_into().unwrap())}
 pub fn set(&mut self,n:usize,v:i16){self.0[n..n+2].copy_from_slice(&v.to_le_bytes());}
 pub fn set_ptr(&mut self,n:usize,v:u32){self.0[n..n+4].copy_from_slice(&v.to_le_bytes());}
}
#[derive(Clone,Debug)]pub struct State{
 pub arena:Vec<u8>,pub jobs:[Job;64],pub headers:[Option<u32>;35],
 pub error:u8,pub boundary:Option<Boundary>,pub sound_clock:u32,
 pub height_global:u32,pub angle_global:u32,
 pub builds:Vec<[u8;88]>,pub known_builds:[bool;35],
}
impl Default for State{fn default()->Self{Self{arena:vec![0;BLOCK*4],jobs:[Job([0;24]);64],headers:[None;35],error:0,boundary:None,sound_clock:0,height_global:0,angle_global:0,builds:vec![[0;88];35],known_builds:[false;35]}}}
#[derive(Clone,Copy,Debug)]pub struct View{pub camera:[i16;2],pub size:[i16;2],pub clip_min:[i16;2],pub quality:i8,pub world_base:i16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Draw{pub position:[i16;2],pub screen:[i16;2],pub sprite:i16,pub angle:u8,pub height_global:u32}
#[derive(Clone,Debug,Default)]pub struct Playback{pub draws:Vec<Draw>,pub restore:Option<Job>,pub sound:Option<i16>,pub complete:bool}
pub fn header(kind:i8)->Option<(u32,u8)>{Some(match kind{0=>(0x5befd0,10),1=>(0x5bc83c,12),4=>(BASE,23),8=>(0x5c1764,8),_=>return None})}
impl State{
 pub fn mission_reset(&mut self){self.jobs.fill(Job([0;24]));self.headers.fill(None);self.known_builds.fill(false);self.sound_clock=0;self.boundary=None;}
 ///45fcb0 initializes ONLY0x299 dwords, then expands80-byte disk fields.
 ///Remaining bytes of late slots survive the prefix fill until source writes.
 pub fn load_buildings(&mut self,disk:&[[u8;80]]){for n in 0..0x299*4{self.builds[n/88][n%88]=255;}for(slot,d)in disk.iter().take(35).enumerate(){let b=&mut self.builds[slot];b[..22].copy_from_slice(&d[..22]);for(to,from)in[(28,26),(30,28),(54,52),(60,57),(80,73)]{b[to..to+2].copy_from_slice(&d[from..from+2]);}for(to,from)in[(36,34),(40,38),(44,42),(76,69)]{let mut v=i32::from(d[from+3]as i8);for n in(0..3).rev(){v=v.wrapping_mul(256).wrapping_add(i32::from(d[from+n]as i8));}b[to..to+4].copy_from_slice(&v.to_le_bytes());}for(to,from)in[(52,50),(53,51),(56,54),(57,55),(58,56),(62,59),(68,64),(69,65),(70,66),(71,67),(72,68),(82,75),(83,76),(84,77),(85,78)]{b[to]=d[from];}self.known_builds[slot]=true;}}
 pub fn read(&self,address:u32,len:usize)->Result<&[u8],Boundary>{let start=address.wrapping_sub(BASE)as usize;self.arena.get(start..start.checked_add(len).ok_or(Boundary{address,length:len})?).ok_or(Boundary{address,length:len})}
 pub fn write(&mut self,address:u32,data:&[u8])->Result<(),Boundary>{for(i,&b)in data.iter().enumerate(){let a=address.wrapping_add(i as u32);let Some(v)=self.arena.get_mut(a.wrapping_sub(BASE)as usize)else{return Err(Boundary{address:a,length:data.len()-i});};*v=b;}Ok(())}
 pub fn word(&self,a:u32)->Result<i16,Boundary>{Ok(i16::from_le_bytes(self.read(a,2)?.try_into().unwrap()))}
 pub fn put_word(&mut self,a:u32,v:i16)->Result<(),Boundary>{self.write(a,&v.to_le_bytes())}
 pub fn footer(&mut self,p:&Particle)->Result<(),Boundary>{if p.owner&4!=0{let h=u32::from_le_bytes(p.raw_tail[8..12].try_into().unwrap());self.write(h.wrapping_add(10),&[p.raw_tail[1],p.raw_tail[1]])?;}Ok(())}
 pub fn record(&mut self,p:&mut Particle,point:[i16;2])->Result<(),Boundary>{
  if p.owner&4==0||p.flags&4==0||p.sprite==250{return Ok(());}
  if p.owner&8!=0{let ptr=u32::from_le_bytes(p.raw_tail[4..8].try_into().unwrap());
   self.put_word(ptr,point[0])?;self.put_word(ptr.wrapping_add(2),point[1])?;self.put_word(ptr.wrapping_add(4),p.sprite.wrapping_add(i16::from(p.phase)))?;
   self.write(ptr.wrapping_add(6),&[p.angle.wrapping_add(p.spin)as u8,(-i32::from(p.position[2])>>3)as u8,p.raw_tail[1]])?;
   p.raw_tail[1]=p.raw_tail[1].wrapping_add(1);if p.raw_tail[1]as i8>44{self.error=3;}
   let next=ptr.wrapping_add((i32::from(p.raw_tail[0]as i8)*10)as u32);p.raw_tail[4..8].copy_from_slice(&next.to_le_bytes());
  }p.owner^=8;Ok(())
 }
 pub fn free_job(&self)->Option<usize>{self.jobs.iter().position(|j|j.ptr(0)==0)}
 ///44fcc0's arrived BUILD branch. The caller owns the actual59 work/arrival.
 pub fn birth(&mut self,building:u32,position:[i16;2],head:u32,actor:u32)->Result<Option<usize>,Boundary>{let Some(slot)=self.free_job()else{return Ok(None);};let frame=self.read(head.wrapping_add(11),1)?[0];let j=&mut self.jobs[slot];j.set_ptr(0,head);j.set_ptr(4,building);j.set(8,position[0]);j.set(10,position[1]);j.0[12]=frame;j.set_ptr(16,actor);j.set(20,0);Ok(Some(slot))}
 pub fn playback(&mut self,slot:usize,v:View,target_position:Option<[i16;2]>)->Result<Playback,Boundary>{
  let mut out=Playback::default();let j=&mut self.jobs[slot];let h=j.ptr(0);if h==0{return Ok(out);}if j.word(20)!=0{j.set(20,j.word(20).wrapping_sub(1));return Ok(out);}
  j.0[12]=j.0[12].wrapping_sub(1);let job=*j;let frame=i16::from(job.0[12]as i8);let count=(self.word(h)?as u16&127)as usize;
  let from=h.wrapping_add(12).wrapping_add((i32::from(frame)*count as i32*10)as u32);let rows=self.read(from,count*10)?.to_vec();let anchor=[self.word(h.wrapping_add(6))?,self.word(h.wrapping_add(8))?];
  for r in rows.chunks_exact(10){self.angle_global=u32::from(r[6]);self.height_global=(self.height_global&0xffff0000)|u32::from(if v.quality>2{r[7]as i8 as i16 as u16}else{0});let x=i16::from_le_bytes([r[0],r[1]]);let y=i16::from_le_bytes([r[2],r[3]]);let mut sprite=i16::from_le_bytes([r[4],r[5]]);if x==0&&y==0&&sprite==0{continue;}
   let p=[x.wrapping_sub(anchor[0]).wrapping_add(job.word(8)),y.wrapping_sub(anchor[1]).wrapping_add(job.word(10))];let s=[p[0].wrapping_sub(v.camera[0]),p[1].wrapping_sub(v.camera[1])];
   if s[0].wrapping_add(200)>v.clip_min[0]&&i32::from(s[0].wrapping_add(200))-270<i32::from(v.size[0])&&s[1].wrapping_add(200)>v.clip_min[1]&&i32::from(s[1].wrapping_add(200))-270<i32::from(v.size[1]){if sprite<=16{sprite=sprite.wrapping_add(v.world_base);}out.draws.push(Draw{position:p,screen:s,sprite,angle:r[6],height_global:self.height_global});}
  }
  let visible=target_position.is_some_and(|p|{let s=[p[0].wrapping_sub(v.camera[0]),p[1].wrapping_sub(v.camera[1])];s[0]>=-48&&i32::from(s[0])<=i32::from(v.size[0])+32&&s[1]>=-48&&i32::from(s[1])<=i32::from(v.size[1])+32});
  if frame==7&&visible&&self.sound_clock==0{out.sound=Some(44);self.sound_clock=100;}
  if frame==1{if self.word(h)?as u16&128!=0{return Err(Boundary{address:h,length:12});}out.restore=Some(job);out.complete=true;if visible{out.sound=Some(46);self.sound_clock=100;}}Ok(out)
 }
 pub fn complete(&mut self,slot:usize,head:u32)->Result<(),Boundary>{self.jobs[slot].set_ptr(0,0);let end=self.read(head.wrapping_add(10),1)?[0];self.write(head.wrapping_add(11),&[end])}
 pub fn end_reader(&mut self){if self.sound_clock!=0{self.sound_clock=self.sound_clock.wrapping_sub(1);}}
}
///4020f0 is distinct from403090/FallingObject, including its zero-spin fix.
pub fn construct(record:&mut[u8;44],mut rng:impl FnMut()->u32){record[23]=0;record[10]=((rng()&31)as i8-16)as u8;record[11]=((rng()&31)as i8-16)as u8;record[12]=(-4i8).wrapping_sub((rng()&3)as i8)as u8;let spin=(rng()&63)as i16-32;record[28..30].fill(0);record[30..32].copy_from_slice(&(if spin==0{6}else{spin}).to_le_bytes());record[13]=4;record[14]=4;record[18..20].copy_from_slice(&400i16.to_le_bytes());record[20..22].copy_from_slice(&3i16.to_le_bytes());}
pub fn emit(w:&mut impl World,g:&mut Globals,count:u16)->usize{w.before_emit(count,g);if g.suppression!=0{return 0;}let mut made=0;for _ in 0..count{let Some(slot)=w.first_free()else{break;};let mut r=w.load(slot);bridge::common(&mut r,g);g.raw.last_record=Some(bridge::NATIVE_POOL.wrapping_add(slot as u32*44));construct(&mut r,||w.random());w.store(slot,r);made+=1;}made}
pub fn prefix(s:&mut State,w:&mut impl World,g:&mut Globals,kind:i8,p:[i16;2])->Result<usize,Boundary>{let Some((h,stride))=header(kind)else{return Ok(0);};s.put_word(h,i16::from(stride))?;for(a,n)in[(2,p[0]),(4,p[1]),(6,p[0]),(8,p[1])]{s.put_word(h+a,n)?;}let disabled=g.suppression;g.suppression=0;g.raw.byte32=stride;g.raw.payload36=h+12;g.raw.payload40=h;g.context.owner=4;g.context.flags=12;g.context.fade=0;g.raw.constructor=Some(0x4020f0);let mut made=0;let rows=match kind{0=>PARTS_0,1=>PARTS_1,4=>PARTS_4,8=>PARTS_8,_=>unreachable!()};for &[x,y,sprite,max]in rows{g.context.origin=[p[0].wrapping_add(x),p[1].wrapping_add(y)];g.context.sprite=sprite;g.maximum=max as u8;made+=emit(w,g,1);g.raw.payload36=g.raw.payload36.wrapping_add(10);}g.suppression=disabled;Ok(made)}
///Called after live contacts; no saved pre-contact global context is restored.
pub fn expiry_children(w:&mut impl World,g:&mut Globals,source:usize,point:[i16;2])->usize{g.context.origin=[point[0].wrapping_sub(154),point[1].wrapping_sub(80)];g.context.fade=0;g.context.owner=0;g.context.flags=4;g.context.sprite=250;g.raw.constructor=Some(bridge::HIT_CONSTRUCTOR);let made=bridge::emit(w,g,4,Constructor::Hit);w.retire(source);made}
pub fn raw_word(b:&[u8;88],at:usize)->i16{i16::from_le_bytes(b[at..at+2].try_into().unwrap())}
pub fn raw_ptr(b:&[u8;88],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
pub fn raw_set(b:&mut[u8;88],at:usize,value:i16){b[at..at+2].copy_from_slice(&value.to_le_bytes());}
///450260 prefix and descriptor writes. The caller constructs/retires the
///fragment pool before querying417400 and applying the reader timer tail.
pub fn restore_raw(b:&mut[u8;88],multiplayer:bool,mut random:impl FnMut()->u32,mut crt:impl FnMut()->u16){
 raw_set(b,30,raw_word(b,28));for at in[10,60,54]{raw_set(b,at,0);}b[32..36].fill(255);raw_set(b,8,-1);b[17..22].fill(0);b[53]=2;b[7]=255;b[82..86].fill(255);b[6]&=242;b.copy_within(64..68,24);b[5]=8;b[36..40].fill(0);
 match b[4]{0=>{b[69]=5;b[17]=((random()as u8)&51).wrapping_add(136);},4=>{b[16]=0;b[62]=if multiplayer{0}else{crt()as u8};},8=>{b[6]&=223;raw_set(b,10,0);},_=>{}}
}
pub fn restore_tail(b:&mut[u8;88],sector:u32){b[32..36].copy_from_slice(&sector.to_le_bytes());b[57]=0;b[76..80].fill(255);let saved=raw_word(b,80);if saved!=-1{b[76..80].copy_from_slice(&i32::from(saved).wrapping_mul(1700).to_le_bytes());}}
#[cfg(test)]#[path="original_building_trail_tests.rs"]pub(super) mod tests;

const PARTS_0:&[[i16;4]]=&[[-99, -76, 2884, 0], [-99, -60, 2885, 0], [-142, -87, 2905, 0], [-144, -60, 2907, 0], [-154, -76, 2910, 0], [-128, -92, 1210, 11], [-144, -92, 1222, 11], [-144, -60, 1222, 11], [-160, -76, 1222, 11], [-128, -60, 1210, 11]];

const PARTS_1:&[[i16;4]]=&[[-144, -28, 2119, 0], [-146, -44, 2110, 0], [-147, -29, 2115, 0], [-148, -22, 2117, 0], [-138, -37, 2113, 0], [-150, -77, 2099, 7], [-160, -76, 1210, 11], [-152, -60, 1222, 11], [-160, -76, 1210, 11], [-160, -76, 1222, 11], [-160, -28, 1210, 11], [-160, -44, 1222, 11]];

const PARTS_4:&[[i16;4]]=&[[-147, -73, 2875, 0], [-122, -37, 2897, 0], [-146, -45, 2121, 0], [-150, -78, 2091, 7], [-144, -28, 2107, 0], [-134, -8, 2123, 0], [-137, -8, 2123, 0], [-140, -8, 2123, 0], [-143, -8, 2123, 0], [-146, -8, 2123, 0], [-149, -8, 2123, 0], [-126, -62, 2903, 0], [-130, -62, 2903, 0], [-134, -62, 2903, 0], [-138, -62, 2903, 0], [-142, -62, 2903, 0], [-146, -62, 2903, 0], [-160, -76, 1210, 11], [-144, -44, 1222, 11], [-160, -76, 1222, 11], [-160, -76, 1210, 11], [-136, -36, 1210, 11], [-160, -44, 1222, 11]];

const PARTS_8:&[[i16;4]]=&[[-142, -78, 2875, 0], [-142, -78, 2875, 0], [-144, -74, 1488, 0], [-119, -92, 2891, 0], [-144, -70, 2914, 0], [-128, -92, 1222, 11], [-144, -92, 1210, 11], [-144, -60, 1222, 11], [-160, -76, 1210, 11]];
