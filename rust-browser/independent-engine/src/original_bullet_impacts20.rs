//! Native20x8 cosmetic contact pool59e622, constructor462e00/tick464d90.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Record{pub position:[i16;2],pub frame:i16,pub flags:u8,pub stale:u8}
impl Default for Record{fn default()->Self{Self{position:[-1,0],frame:0,flags:0,stale:0}}}
#[derive(Clone,Copy,Debug)]pub struct Draw{pub slot:u8,pub position:[i16;2],pub screen:[i16;2],pub sprite:i16}
#[derive(Clone,Copy,Debug)]pub struct Profile{pub land_sprite:i16,pub land_frames:u8,pub wet_sprite:i16,pub wet_frames:u8}
impl Profile{pub fn for_world(native_world:u8)->Self{let row=[[460,9,1936,9],[3079,6,3085,8],[3018,6,3010,8],[2679,6,2673,6],[3151,6,3143,8]][usize::from(native_world.min(4))];Self{land_sprite:row[0],land_frames:row[1]as u8,wet_sprite:row[2],wet_frames:row[3]as u8}}}
pub struct State{pub records:[Record;20],pub draws:Vec<Draw>}
impl Default for State{fn default()->Self{Self{records:[Record::default();20],draws:vec![]}}}
impl State{
 pub fn clear_for_mission(&mut self){for r in &mut self.records{r.position[0]=-1;}self.draws.clear();}
 pub fn spawn(&mut self,point:[i16;2],terrain:u8)->Option<usize>{let slot=self.records.iter().position(|r|r.position[0]==-1)?;let r=&mut self.records[slot];r.flags=0;let code=terrain&127;if[9,42,38,40,39].contains(&code){r.flags=1;}else if code!=22&&code>6{return None;}r.position=point;r.frame=0;Some(slot)}
 pub fn advance(&mut self,camera:[i16;2],size:[i16;2],profile:Profile,pulse:bool){self.draws.clear();for(slot,r)in self.records.iter_mut().enumerate(){if r.position[0]==-1{continue;}let screen=[r.position[0].wrapping_sub(camera[0]),r.position[1].wrapping_sub(camera[1])];let wet=r.flags&1!=0;if (0..2).all(|i|screen[i]>-8&&screen[i]<size[i]){self.draws.push(Draw{slot:slot as u8,position:r.position,screen,sprite:r.frame.wrapping_add(if wet{profile.wet_sprite}else{profile.land_sprite})});}if pulse{r.frame=r.frame.wrapping_add(1);if i16::from(if wet{profile.wet_frames}else{profile.land_frames})<=r.frame{r.position[0]=-1;}}}}
 pub fn bytes(&self)->[u8;160]{let mut out=[0;160];for(i,r)in self.records.iter().enumerate(){out[i*8..i*8+2].copy_from_slice(&r.position[0].to_le_bytes());out[i*8+2..i*8+4].copy_from_slice(&r.position[1].to_le_bytes());out[i*8+4..i*8+6].copy_from_slice(&r.frame.to_le_bytes());out[i*8+6]=r.flags;out[i*8+7]=r.stale;}out}
}
