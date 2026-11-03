//! Original100-slot ambient-target records. These include walking wildlife and
//! other template-driven world animations, separately from OBJECT200 and the
//!500x44 transient-effect pool. Raw fields preserve recycled-slot semantics.
pub const CAPACITY:usize=100;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Record{pub bytes:[u8;62]}
impl Default for Record{fn default()->Self{let mut r=Self{bytes:[0;62]};r.set_word(0,-1);r}}
impl Record{
 pub fn byte(&self,offset:usize)->u8{self.bytes[offset]}
 pub fn set_byte(&mut self,offset:usize,value:u8){self.bytes[offset]=value;}
 pub fn word(&self,offset:usize)->i16{i16::from_le_bytes([self.bytes[offset],self.bytes[offset+1]])}
 pub fn set_word(&mut self,offset:usize,value:i16){self.bytes[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
 pub fn active(&self)->bool{self.word(0)!=-1}
 pub fn position(&self)->[i16;2]{[self.word(0),self.word(2)]}
}
#[derive(Clone,Copy,Debug)]
pub struct Spawn{
 pub position:[i16;2],pub template:u8,
 ///Constructor stack arguments4..8 retain raw values until their writers
 ///and event users establish complete meanings.
 pub arg4:u8,pub arg5:u8,pub arg6:i8,pub arg7:i16,pub arg8:i16,
 pub heading:u8,pub delay:i16,pub arg11:u8,
}
#[derive(Clone,Copy)]struct Template{sprite:i16,stride:i8,frames:u8,field14:i16,field16:i16,field18:i16,flag36:u8,phase:u8,field6:i16}
const TEMPLATES:[Template;52]=[
 Template{sprite:2162,stride:10,frames:0,field14:-1,field16:0,field18:-1,flag36:0,phase:4,field6:0},
 Template{sprite:3535,stride:14,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:4,field6:0},
 Template{sprite:2181,stride:4,frames:8,field14:2181,field16:2245,field18:2249,flag36:1,phase:0,field6:-2287},
 Template{sprite:2213,stride:4,frames:6,field14:2213,field16:2247,field18:2265,flag36:1,phase:0,field6:2288},
 Template{sprite:4422,stride:5,frames:0,field14:-1,field16:0,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:3419,stride:4,frames:8,field14:3419,field16:3515,field18:3519,flag36:0,phase:1,field6:-2289},
 Template{sprite:2282,stride:5,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:2172,stride:7,frames:0,field14:-1,field16:0,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:3660,stride:4,frames:8,field14:3660,field16:3708,field18:3692,flag36:1,phase:0,field6:-2293},
 Template{sprite:2325,stride:4,frames:8,field14:2325,field16:2373,field18:2357,flag36:1,phase:0,field6:2291},
 Template{sprite:3610,stride:4,frames:8,field14:3610,field16:3658,field18:3642,flag36:1,phase:0,field6:2294},
 Template{sprite:3451,stride:8,frames:6,field14:3451,field16:3517,field18:-1,flag36:0,phase:0,field6:2290},
 Template{sprite:2285,stride:1,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:2285,stride:4,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:2286,stride:4,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:3770,stride:4,frames:8,field14:3770,field16:3818,field18:3802,flag36:0,phase:0,field6:2295},
 Template{sprite:3820,stride:4,frames:8,field14:3820,field16:3868,field18:3852,flag36:1,phase:0,field6:2296},
 Template{sprite:3922,stride:4,frames:8,field14:3922,field16:3970,field18:3954,flag36:0,phase:0,field6:2297},
 Template{sprite:3972,stride:4,frames:8,field14:3972,field16:4020,field18:4004,flag36:1,phase:2,field6:2298},
 Template{sprite:4026,stride:4,frames:8,field14:4026,field16:4058,field18:4060,flag36:1,phase:0,field6:2299},
 Template{sprite:4076,stride:4,frames:8,field14:4076,field16:4124,field18:4108,flag36:0,phase:0,field6:2300},
 Template{sprite:4176,stride:9,frames:0,field14:-1,field16:0,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4185,stride:9,frames:0,field14:-1,field16:1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4194,stride:9,frames:0,field14:-1,field16:2,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4203,stride:9,frames:0,field14:-1,field16:3,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4212,stride:9,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:4221,stride:13,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:4234,stride:12,frames:0,field14:-1,field16:0,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4246,stride:12,frames:0,field14:-1,field16:1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4258,stride:12,frames:0,field14:-1,field16:2,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4270,stride:12,frames:0,field14:-1,field16:3,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4212,stride:9,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:4494,stride:4,frames:6,field14:4494,field16:4692,field18:4526,flag36:0,phase:0,field6:-2301},
 Template{sprite:4542,stride:4,frames:8,field14:4542,field16:4590,field18:4574,flag36:1,phase:0,field6:-2302},
 Template{sprite:4592,stride:4,frames:8,field14:4592,field16:-1,field18:4624,flag36:0,phase:0,field6:2303},
 Template{sprite:4291,stride:1,frames:6,field14:-1,field16:-1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4299,stride:1,frames:6,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:2375,stride:4,frames:6,field14:2375,field16:2407,field18:-1,flag36:0,phase:0,field6:-2292},
 Template{sprite:3521,stride:14,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:4,field6:0},
 Template{sprite:3507,stride:14,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:4,field6:0},
 Template{sprite:3493,stride:14,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:4,field6:0},
 Template{sprite:4642,stride:4,frames:0,field14:-1,field16:4690,field18:4674,flag36:0,phase:0,field6:4642},
 Template{sprite:4126,stride:4,frames:0,field14:-1,field16:4174,field18:4158,flag36:1,phase:0,field6:4126},
 Template{sprite:4300,stride:1,frames:6,field14:-1,field16:-1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:4308,stride:1,frames:6,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:6139,stride:4,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:5,field6:0},
 Template{sprite:6229,stride:1,frames:1,field14:-1,field16:-1,field18:-1,flag36:0,phase:0,field6:0},
 Template{sprite:6171,stride:4,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:5,field6:0},
 Template{sprite:6211,stride:6,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:5,field6:0},
 Template{sprite:6203,stride:1,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:5,field6:0},
 Template{sprite:6287,stride:5,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:6,field6:0},
 Template{sprite:6212,stride:5,frames:0,field14:-1,field16:-1,field18:-1,flag36:0,phase:5,field6:0},
];
///405c60 writes selected fields without clearing the rest of the reused62-byte
///record. A disabled constructor performs no allocation or writes.
pub fn initialize(record:&mut Record,spawn:Spawn){
 let template=TEMPLATES[usize::from(spawn.template)];
 let x=spawn.position[0].wrapping_abs();let y=spawn.position[1].wrapping_abs();
 record.set_word(0,x);record.set_word(4,x);record.set_word(8,x);record.set_word(38,spawn.delay);record.set_byte(51,spawn.arg11);
 record.set_byte(54,0);record.set_word(16,0);record.set_word(52,0);record.set_word(2,y);record.set_word(10,y);
 record.set_word(28,template.field14);record.set_word(22,template.field16);record.set_word(24,template.field18);
 record.set_byte(54,template.flag36);record.set_byte(60,template.phase);record.set_word(6,template.field6);record.set_byte(30,template.frames);
 record.set_byte(44,spawn.heading);record.set_byte(58,spawn.heading);record.set_byte(31,template.stride as u8);
 let sprite=template.sprite.wrapping_add(i16::from(spawn.heading>>5).wrapping_mul(i16::from(template.stride)));
 record.set_word(20,sprite);record.set_byte(57,0);record.set_byte(46,1);record.set_word(26,template.sprite);record.set_word(34,sprite.wrapping_add(i16::from(template.stride)));
 if spawn.position[1]<0{record.set_byte(52,16);}record.set_word(36,0);record.set_byte(55,spawn.arg6 as u8);
 if spawn.arg6>=0{record.set_byte(52,record.byte(52)|32);record.set_word(18,spawn.arg7);record.set_word(36,0);}
 record.set_byte(46,spawn.arg5);record.set_byte(48,spawn.arg5);record.set_byte(45,0);record.set_byte(56,16);record.set_byte(47,spawn.arg4);record.set_word(49,0);
 if spawn.arg8!=-1{record.set_byte(52,record.byte(52)|68);}
}
pub struct State{pub records:Vec<Record>,pub enabled:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Fragment{pub position:[i16;2],pub sprite:i16}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Blocker{pub bytes:[u8;10]}
impl Default for Blocker{fn default()->Self{let mut r=Self{bytes:[0;10]};r.bytes[..2].copy_from_slice(&(-1i16).to_le_bytes());r}}
///41cab0 returns the first active50-pool record strictly within8 native pixels.
pub fn blocker_at(blockers:&[Blocker],point:[i16;2])->Option<usize>{
 blockers.iter().position(|b|{let p=b.position();let d=[i32::from(point[0])-i32::from(p[0]),i32::from(point[1])-i32::from(p[1])];p[0]!=-1&&d[0]*d[0]+d[1]*d[1]<64})
}
impl Blocker{pub fn position(&self)->[i16;2]{[i16::from_le_bytes([self.bytes[0],self.bytes[1]]),i16::from_le_bytes([self.bytes[2],self.bytes[3]])]}}
#[derive(Clone,Copy,Debug)]pub struct SceneObject{pub slot:u8,pub position:[i16;2],pub sprite:u16,pub flags13:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct LavaSeed{pub position:[i16;2],pub template:u8,pub size:u8,pub interval:u8}
pub trait StartupWorld{
 fn random(&mut self)->u32;
 fn native_world(&self)->u8;
 fn terrain_cell(&self,cell:i16)->u8;
 fn set_terrain_cell(&mut self,cell:i16,code:u8);
 ///405350 is a separate32x60 native world pool, active in Volcan only.
 fn lava_seed(&mut self,seed:LavaSeed);
}
impl Default for State{fn default()->Self{Self{records:vec![Record::default();CAPACITY],enabled:true}}}
impl State{
 ///405e70 consumes its20 four-word prefix even outside Volcan. The seeded
 ///animated objects, source flags and50 blockers retain original slot order.
 pub fn initialize_scene(&mut self,objects:&mut[SceneObject],blockers:&mut[Blocker],world:&mut impl StartupWorld)->Result<(),&'static str>{
  for _ in 0..20{
   let interval=(world.random()&63)as u8;let template=(world.random()&1)as u8;let y=(world.random()&1023)as i16;let x=(world.random()&1023)as i16;
   let size=if world.native_world()==2{1}else{2};world.lava_seed(LavaSeed{position:[x,y],template,size,interval});
  }
  let mut order=(0..objects.len()).collect::<Vec<_>>();order.sort_by_key(|&i|objects[i].slot);
  for index in order{
   let object=&mut objects[index];if object.position[0]==-1{continue;}
   let cell=(object.position[1]&!15).wrapping_mul(8).wrapping_add(object.position[0]>>4);
   if matches!(object.sprite,2935..=2939){
    object.flags13&=127;
    let Some(blocker)=blockers.iter_mut().find(|b|b.position()[0]==-1)else{return Err("Native50-record blocker pool exhausted");};
    blocker.bytes[0..2].copy_from_slice(&object.position[0].to_le_bytes());blocker.bytes[2..4].copy_from_slice(&object.position[1].to_le_bytes());blocker.bytes[5]=object.slot;
    let mut remaining=6;let mut mask=1u16;
    while remaining!=0{
     if remaining<3&&world.random()as u16&mask!=0{remaining-=1;continue;}
     mask=mask.wrapping_mul(2).wrapping_add(1);let delay=((world.random()&65535)%11+32)as i16;let template=[2,3,5,11][(world.random()&3)as usize];
     let old=world.terrain_cell(cell);world.set_terrain_cell(cell,(old&128)+17);
     self.spawn(Spawn{position:object.position,template,arg4:255,arg5:2,arg6:-1,arg7:0,arg8:0,heading:128,delay,arg11:6});remaining-=1;
    }
   }else if object.sprite==2285{
    let random=world.random();let position=object.position.map(|n|n&!15);object.position[0]=-1;
    if world.terrain_cell(cell)&127==0{self.spawn(Spawn{position,template:6,arg4:0,arg5:2,arg6:127,arg7:3,arg8:-1,heading:0,delay:((random&65535)%61+20)as i16,arg11:0});}
   }
  }
  Ok(())
 }
 pub fn spawn(&mut self,spawn:Spawn)->Option<u8>{
  if !self.enabled||usize::from(spawn.template)>=TEMPLATES.len(){return None;}
  let slot=self.records.iter().position(|r|!r.active())?;initialize(&mut self.records[slot],spawn);Some(slot as u8)
 }
 ///404ec0 area damage visits every native slot;404f80 bullet contact stops at
 ///the first qualifying slot. Their square bounds are strict25 and16 pixels.
 pub fn damage(&mut self,position:[i16;2],bullet:bool,mut fragment:impl FnMut(Fragment))->bool{
  let mut found=false;
  for record in &mut self.records{
   if !record.active()||record.word(38)!=0||record.byte(53)&48!=0||record.byte(60)==6||record.byte(30)==0||bullet&&record.byte(30)==255{continue;}
   let p=record.position();let limit=if bullet{16}else{25};
   if !(0..2).all(|i|p[i].wrapping_sub(position[i]).wrapping_abs()<limit){continue;}
   record.set_byte(45,0);record.set_word(52,0x1000);record.set_byte(57,0);record.set_word(0,-1);
   fragment(Fragment{position:p,sprite:record.word(20).wrapping_abs()});found=true;if bullet{return true;}
  }
  found
 }
 ///41cb40 reacts to a retired terrain object at its aligned original home.
 pub fn clear_cell(&mut self,position:[i16;2]){
  let home=position.map(|n|n&!15);
  for record in &mut self.records{if record.active()&&[record.word(8),record.word(10)]==home{record.set_byte(52,record.byte(52)&126);record.set_word(16,-1);if record.word(38)!=0{record.set_word(0,-1);}}}
 }
}


#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Actor{pub position:[i16;2],pub class:u8,pub phase:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Draw{pub position:[i16;2],pub sprite:i16,pub palette:u8,pub shade:u8,pub rotation:Option<u8>,pub flags52:u8,pub flags53:u8}
pub const STEPS:[[i16;2];8]=[[0,-1],[1,-1],[1,0],[1,1],[0,1],[-1,1],[-1,0],[-1,-1]];
const TERRAIN:[u8;67]=[0,255,255,255,255,255,255,255,0,255,255,255,255,255,255,255,255,0,255,255,0,0,0,255,255,255,255,255,255,255,0,255,255,255,255,255,0,255,255,255,255,255,255,255,0,0,255,255,255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,0,0,0];
///World queries are the exact external pools used by405820. Actions receive
///the mutable record because a native first-free allocation may reuse it.
pub trait World{
 fn random(&mut self)->u32;
 fn terrain(&self,position:[i16;2])->u8;
 fn nearby_actors(&self,position:[i16;2])->Vec<Actor>;
 fn blocker(&self,position:[i16;2])->bool;
 fn hazard_near(&self,position:[i16;2])->bool;
 fn secondary_at(&mut self,record:&mut Record,position:[i16;2],heading:u8);
 fn water_contact(&mut self,record:&mut Record);
 fn delayed_mark(&mut self,record:&mut Record,delay:u16);
}
impl Record{
 fn flag(&self,offset:usize,bits:u8)->bool{self.byte(offset)&bits!=0}
 fn bit(&mut self,offset:usize,bits:u8,set:bool){self.set_byte(offset,if set{self.byte(offset)|bits}else{self.byte(offset)&!bits});}
 ///405040 emits a draw from the pre-update state. Its only persistent write
 ///is Y-offset33=16 when53:20 is set; clipping does not suppress that write.
 pub fn draw(&mut self)->Draw{
  let frame=i16::from(self.byte(45)as i8);
  let sprite=if self.flag(53,16){self.word(28).wrapping_add(frame)}
   else if self.flag(53,32){self.set_byte(51,16);self.word(22)}
   else if self.flag(53,64){self.word(24).wrapping_add(frame).wrapping_add(i16::from((self.byte(44)>>4)&12))}
   else{self.word(20).wrapping_add(frame)};
  Draw{position:self.position(),sprite,palette:if self.flag(52,12){0}else{self.byte(52)&3},shade:self.byte(51),rotation:self.flag(52,4).then_some(self.byte(50)),flags52:self.byte(52),flags53:self.byte(53)}
 }
 fn extinguish(&mut self,world:&mut impl World){
  if self.byte(60)==1{
   let delay=((world.random()&65535)%33+32)as u16;world.delayed_mark(self,delay);let position=self.position();let heading=self.byte(44);self.set_word(0,-1);world.secondary_at(self,position,heading);
  }else{
   self.set_byte(52,self.byte(52)&126);let delay=((world.random()&127)+160)as i16;self.set_word(38,delay);self.set_word(16,delay&15);self.set_byte(44,128);self.set_byte(58,128);
  }
 }
 ///405820, called once per active record in pool-slot order. It performs no
 ///independent wall-clock integration. The returned draw precedes all motion.
 pub fn advance(&mut self,world:&mut impl World)->Option<Draw>{
  if !self.active(){return None;}
  let p=self.position();self.set_word(42,((p[1]>>3)&!15).wrapping_add(p[0]>>7));
  let delay=self.word(38);if delay!=0{self.set_word(38,delay.wrapping_sub(1));return None;}
  let draw=self.draw();self.set_byte(32,0);if self.flag(53,32){return Some(draw);}
  if matches!(self.byte(60),0|1|2){
   let p=self.position();if world.nearby_actors(p).iter().any(|a|(a.class==1||a.phase==18)&&(0..2).all(|i|p[i].wrapping_sub(8)<=a.position[i]&&a.position[i]<=p[i].wrapping_add(16))){
    self.bit(53,32,true);self.set_word(45,0x7f00);
   }
  }
  self.set_byte(46,self.byte(46).wrapping_sub(1));if self.byte(46)!=0{return Some(draw);}
  let mut period=self.byte(48);let special=self.flag(53,64);
  if !special{if self.flag(52,128){period=(period as i8>>1)as u8;}}
  else if matches!(self.byte(45),0|2){period=period.wrapping_shl(3);self.set_byte(45,self.byte(45).wrapping_add(1)&3);}
  self.set_byte(46,period);
  let mut direct_turn=false;
  if !special{
   let sprite=self.word(26).wrapping_add(i16::from(self.byte(44)>>5).wrapping_mul(i16::from(self.byte(31)as i8)));
   self.set_word(20,sprite);self.set_byte(50,0);self.bit(53,1,false);self.set_word(34,sprite.wrapping_add(i16::from(self.byte(31)as i8)));self.set_byte(44,self.byte(58));
   if self.byte(57)!=0{self.set_byte(57,self.byte(57).wrapping_sub(1));if self.byte(57)!=0{return Some(draw);}direct_turn=true;}
   else{
    if self.flag(52,32){self.set_word(36,self.word(36).wrapping_add(1));if self.byte(55)!=127&&i16::from(self.byte(55)as i8)==self.word(36){let position=self.position();let heading=self.byte(44);world.secondary_at(self,position,heading);}}
    if !self.flag(52,16){
     self.set_byte(45,self.byte(45).wrapping_add(1));
     if !self.flag(53,16){
      if self.word(20).wrapping_add(i16::from(self.byte(45)as i8))==self.word(34){
       let loops=self.byte(47)as i8;if loops!=-1{
        if loops==0{let position=self.position();let heading=self.byte(44);self.set_word(0,-1);if self.byte(55)==127&&self.word(18)!=0{world.secondary_at(self,position,heading);}return Some(draw);}
        self.set_byte(47,self.byte(47).wrapping_sub(1));
       }
       self.set_byte(45,0);
      }
     }else if self.byte(45)==self.byte(30){self.set_word(0,-1);return Some(draw);}
    }
    if self.flag(52,16){
     self.set_byte(45,self.byte(45).wrapping_sub(1));
     if i32::from(self.byte(45)as i8)!=-i32::from(self.byte(31)as i8){let position=self.position();let heading=self.byte(44);self.set_word(0,-1);if self.byte(55)==127&&self.word(18)!=0{world.secondary_at(self,position,heading);}return Some(draw);}
    }
   }
  }
  if !direct_turn{
   if self.byte(60)!=4&&!self.flag(52,64){return Some(draw);}
   if self.byte(60)==4||!special{
    let step=STEPS[usize::from(self.byte(44)>>5)];let multiplier=if self.flag(54,1){2}else{1};
    self.set_word(0,self.word(0).wrapping_add(step[0]*multiplier));self.set_word(2,self.word(2).wrapping_add(step[1]*multiplier));
   }
   if self.byte(60)==2&&self.byte(56)==16&&matches!(world.terrain(self.position())&127,9|38){world.water_contact(self);self.set_word(0,-1);return Some(draw);}
   self.set_byte(56,self.byte(56).wrapping_sub(1));if self.byte(56)==0{self.set_byte(56,16);}
   else if !self.flag(53,64)&&self.flag(54,1){self.set_byte(56,self.byte(56).wrapping_sub(1));if self.byte(56)==0{self.set_byte(56,16);}}
   if self.byte(60)!=1&&world.blocker(self.position()){self.extinguish(world);return Some(draw);}
   if self.byte(56)!=16{return Some(draw);}
  }
  let mut turn:i16;
  loop{
   if !direct_turn{
   self.bit(53,64,false);self.set_byte(45,0);
   if self.flag(53,1){
    let dx=self.word(8).wrapping_sub(self.word(0));let dy=self.word(10).wrapping_sub(self.word(2));
    if dx==0&&dy==0{self.extinguish(world);return Some(draw);}
    let ax=dx.wrapping_abs();let ay=dy.wrapping_abs();let desired=if ax==ay{if dx<0{if dy<0{224}else{160}}else if dy<0{32}else{96}}
     else if ay<=ax{if dx<0{64}else{192}}else if dy<0{0}else{128};
    self.set_byte(58,desired);turn=i16::from(desired as i8)-i16::from(self.byte(44));
    self.bit(53,2,turn<0);if turn<0{turn=turn.wrapping_neg();}break;
   }
   let watch=self.word(16);let near=if watch==0{!matches!(self.byte(60),5|6)&&!self.flag(52,1)&&!world.nearby_actors(self.position()).is_empty()}
    else{self.set_word(16,watch.wrapping_sub(1));false};
   if near{self.bit(52,129,true);continue;}
   if (self.byte(56)as i8)<16&&self.word(6)<0&&!self.flag(52,1)&&world.hazard_near(self.position()){self.bit(52,129,true);continue;}
   }
   if self.byte(60)==5{return Some(draw);}
   let r=world.random()as u16;let choice=r&3;turn=if choice==0{0}else{(choice as i16-2)*32};
   if choice==2{self.bit(53,2,false);break;}
   if self.word(24)==-1{self.bit(53,2,false);turn=32;break;}
   let r=world.random()as u16;
   if !self.flag(53,1)||r&15!=0||matches!(self.byte(45),0|2)||self.byte(44)&63==0{
    self.bit(53,64,true);let r=world.random();self.set_byte(56,((((r&65535)%3+2)&6)+1)as u8);let r=world.random();self.set_byte(45,(r&2)as u8);return Some(draw);
   }
   break;
  }
  loop{
   if turn==0{turn=32-(world.random()as u16&64)as i16;}
   self.set_byte(32,self.byte(32).wrapping_add(1));if self.byte(32)==10{self.set_word(0,-1);return Some(draw);}
   let delta=if self.flag(53,2){(turn as i8).wrapping_neg()}else{turn as i8};let heading=self.byte(44).wrapping_add(delta as u8);
   self.set_byte(58,heading);self.set_byte(44,heading);let step=STEPS[usize::from(heading>>5)];
   let p=[self.word(0).wrapping_add(step[0]*16),self.word(2).wrapping_add(step[1]*16)];
   if !self.flag(52,1)&&world.blocker(p)&&heading>32&&heading<224{continue;}
   let code=world.terrain(p)&127;
   if (self.byte(60)==2&&matches!(code,9|38))||TERRAIN.get(usize::from(code))==Some(&0){
    if p[0].wrapping_sub(self.word(8)).wrapping_abs()<80&&p[1].wrapping_sub(self.word(10)).wrapping_abs()<80{return Some(draw);}
   }
  }
 }
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_1536_native_graphics_submissions_match_sprite_palette_signed_offset_and_rotation(){
  let mut hash=2166136261u32;let mut count=0;
  for flags52 in[0,1,2,3,4,8,12,68]{for flags53 in[0,16,32,64]{for frame in[-3i8,0,2]{for offset in[0,6,16,255]{for angle in[0,127,128,255]{
   let mut r=Record::default();r.set_word(0,500);r.set_word(2,600);for(o,v)in[(20,100),(22,200),(24,300),(28,400)]{r.set_word(o,v);}r.set_byte(44,96);r.set_byte(45,frame as u8);r.set_byte(50,angle);r.set_byte(51,offset);r.set_byte(52,flags52);r.set_byte(53,flags53);
   let d=r.draw();let tag=if d.rotation.is_some(){2}else{1};let y=d.position[1].wrapping_add(i16::from(d.shade as i8));let rotation=d.rotation.map_or(0,|a|255-i32::from(a as i8));
   for n in[i32::from(flags52),i32::from(flags53),i32::from(frame),i32::from(offset),i32::from(angle),tag,i32::from(d.position[0]),i32::from(y),i32::from(d.sprite),i32::from(d.palette),0,rotation]{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}
  assert_eq!(count,1536);assert_eq!(hash,2016245045,"405040→4549f0→54630 final native submission oracle");
 }
 struct Flat{rng:crate::original_combat::OriginalRng,mode:u8,water:u32,mark:u32}
 impl World for Flat{
  fn random(&mut self)->u32{self.rng.next_u32()}
  fn terrain(&self,_:[i16;2])->u8{if self.mode==10{9}else{0}}
  fn nearby_actors(&self,_:[i16;2])->Vec<Actor>{if self.mode<3{vec![Actor{position:[500,600],class:if self.mode==0{1}else{0},phase:if self.mode==1{18}else{2}}]}else{vec![]}}
  fn blocker(&self,p:[i16;2])->bool{self.mode==11&&(0..50).any(|j|p==[496+j%9,596+j/9])}
  fn hazard_near(&self,_:[i16;2])->bool{false}
  fn secondary_at(&mut self,r:&mut Record,_:[i16;2],_:u8){assert_eq!(r.word(18),0);}
  fn water_contact(&mut self,_:&mut Record){self.water+=1;}
  fn delayed_mark(&mut self,_:&mut Record,_:u16){self.mark+=1;}
 }
 #[test]fn all_4992_complete_native_ambient_callbacks_match_record_bytes_and_rng(){
  let mut hash=2166136261u32;let mut rows=vec![];
  let mut details=vec![];
  for template in 0..52{for mode in 0..4{
   let mut r=Record::default();initialize(&mut r,Spawn{position:[500,600],template,arg4:127,arg5:2,arg6:-1,arg7:0,arg8:if mode&1==0{0}else{-1},heading:mode*64,delay:0,arg11:0});
   let mut world=Flat{rng:crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:false},mode:255,water:0,mark:0};
   for tick in 1..=24{r.advance(&mut world);let mut record_hash=2166136261u32;for b in r.bytes{record_hash=(record_hash^u32::from(b)).wrapping_mul(16777619);}
    let mut row=vec![u32::from(template),u32::from(mode),tick,record_hash];row.extend(world.rng.words);for &n in &row{hash=(hash^n).wrapping_mul(16777619);}rows.push(row.iter().map(u32::to_string).collect::<Vec<_>>().join(","));
    if template==2&&mode==0{row.extend(r.bytes.map(u32::from));details.push(row.iter().map(u32::to_string).collect::<Vec<_>>().join(","));}
   }
  }}
  if let Ok(path)=std::env::var("AMBIENT_TRACE"){std::fs::write(path,rows.join("\n")).unwrap();}
  if let Ok(path)=std::env::var("AMBIENT_DETAIL"){std::fs::write(path,details.join("\n")).unwrap();}
  assert_eq!(rows.len(),4992);assert_eq!(hash,2514899352);
 }
 #[test]fn all_832_native_constructor_cases_preserve_the_exact_recycled_record_bytes(){
  let mut hash=2166136261u32;let mut count=0;
  for template in 0..52{for mode in 0..16{
   let free=mode%5;let mut state=State::default();state.enabled=mode&8==0;
   for(slot,record)in state.records.iter_mut().enumerate(){for(j,b)in record.bytes.iter_mut().enumerate(){*b=((slot*62+j)*13+mode*7)as u8;}record.set_word(0,if slot<free{32+slot as i16}else{-1});}
   let input=Spawn{position:[if mode&4==0{300}else{-300},if mode&2==0{400}else{-400}],template,arg4:(mode*9+1)as u8,arg5:(mode*7+1)as u8,arg6:if mode&1==0{-1}else{3},arg7:(mode*5+2)as i16,arg8:if mode&2==0{-1}else{0},heading:(mode*37)as u8,delay:(mode*7)as i16-20,arg11:(mode*11)as u8};
   assert_eq!(state.spawn(input),state.enabled.then_some(free as u8));let mut record_hash=2166136261u32;for b in state.records[free].bytes{record_hash=(record_hash^u32::from(b)).wrapping_mul(16777619);}
   for n in[u32::from(template),mode as u32,free as u32,record_hash]{hash=(hash^n).wrapping_mul(16777619);}count+=1;
  }}
  assert_eq!(count,832);assert_eq!(hash,1488429325);
 }
 #[test]fn all_14976_native_contact_callbacks_match_crushing_water_delays_and_special_frames(){
  let mut hash=2166136261u32;let mut rows=vec![];
  for template in 0..52{for mode in 0..12{
   let mut r=Record::default();initialize(&mut r,Spawn{position:[500,600],template,arg4:127,arg5:2,arg6:-1,arg7:0,arg8:if mode&1==0{0}else{-1},heading:0,delay:0,arg11:0});
   match mode{3|4=>{r.set_byte(57,if mode==3{1}else{2});r.set_byte(45,1);r.set_byte(53,3);},5=>r.set_word(38,2),6=>r.bit(52,16,true),7=>r.set_byte(47,0),8=>r.set_byte(53,16),9=>{r.bit(52,32,true);r.set_byte(55,1);},_=>{}}
   let mut world=Flat{rng:crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:false},mode,water:0,mark:0};
   for tick in 1..=24{r.advance(&mut world);let mut record_hash=2166136261u32;for b in r.bytes{record_hash=(record_hash^u32::from(b)).wrapping_mul(16777619);}
    let mut row=vec![u32::from(template),u32::from(mode),tick,record_hash];row.extend(world.rng.words);row.extend([world.water,world.mark]);for &n in &row{hash=(hash^n).wrapping_mul(16777619);}rows.push(row.iter().map(u32::to_string).collect::<Vec<_>>().join(","));
   }
  }}
  if let Ok(path)=std::env::var("AMBIENT_CONTACT_TRACE"){std::fs::write(path,rows.join("\n")).unwrap();}
  assert_eq!(rows.len(),14976);assert_eq!(hash,429329161);
 }
 fn bytes_hash<'a>(bytes:impl IntoIterator<Item=&'a u8>)->u32{bytes.into_iter().fold(2166136261u32,|h,&n|(h^u32::from(n)).wrapping_mul(16777619))}
 struct SceneWorld{rng:crate::original_combat::OriginalRng,world:u8,plane:Vec<u8>,lava:Vec<LavaSeed>}
 impl StartupWorld for SceneWorld{
  fn random(&mut self)->u32{self.rng.next_u32()}
  fn native_world(&self)->u8{self.world}
  fn terrain_cell(&self,cell:i16)->u8{self.plane[cell as usize]}
  fn set_terrain_cell(&mut self,cell:i16,code:u8){self.plane[cell as usize]=code;}
  fn lava_seed(&mut self,seed:LavaSeed){self.lava.push(seed);}
 }
 #[test]fn all_96_complete_native_scene_initializations_match_four_pools_terrain_and_rng(){
  let mut hash=2166136261u32;let mut rows=vec![];
  for scenario in 0..16{for world in [0,2,3]{for multi in 0..2{
   let mut state=State::default();let mut blockers=vec![Blocker::default();50];let mut objects=(0..200).map(|slot|SceneObject{slot:slot as u8,position:[-1,0],sprite:0,flags13:0}).collect::<Vec<_>>();
   for i in 0..24{if(i+scenario)%7==0{continue;}objects[i]=SceneObject{slot:i as u8,position:[(400+(i*31+scenario*17)%1000)as i16,(400+(i*67+scenario*13)%1000)as i16],sprite:if i%3==0{2935+(i%5)as u16}else if i%3==1{2285}else{7000},flags13:128|((i+scenario)&127)as u8};}
   let mut env=SceneWorld{rng:crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:multi!=0},world,plane:(0..16384).map(|j|(((j+scenario)&1)*128+if scenario&2!=0&&j%11==0{10}else{0})as u8).collect(),lava:vec![]};
   state.initialize_scene(&mut objects,&mut blockers,&mut env).unwrap();
   let records_hash=bytes_hash(state.records.iter().flat_map(|r|r.bytes.iter()));let blocker_hash=bytes_hash(blockers.iter().flat_map(|b|b.bytes.iter()));let terrain_hash=bytes_hash(&env.plane);
   let mut object_hash=2166136261u32;for o in objects{for n in[u32::from(o.position[0]as u16),u32::from(o.position[1]as u16),u32::from(o.flags13)]{object_hash=(object_hash^n).wrapping_mul(16777619);}}
   let mut event_hash=2166136261u32;for seed in &env.lava{for n in[u32::from(seed.position[0]as u16),u32::from(seed.position[1]as u16),u32::from(seed.template),u32::from(seed.size),u32::from(seed.size),u32::from(seed.interval)]{event_hash=(event_hash^n).wrapping_mul(16777619);}}
   let mut row=vec![scenario as u32,u32::from(world),multi,records_hash,blocker_hash,terrain_hash,object_hash,env.lava.len()as u32,event_hash];row.extend(env.rng.words);for &n in &row{hash=(hash^n).wrapping_mul(16777619);}rows.push(row.iter().map(u32::to_string).collect::<Vec<_>>().join(","));
  }}}
  if let Ok(path)=std::env::var("AMBIENT_SCENE_TRACE"){std::fs::write(path,rows.join("\n")).unwrap();}
  assert_eq!(rows.len(),96);assert_eq!(hash,3493876672);
 }
 #[test]fn all_384_native_damage_and_terrain_retirements_match_all100_slots_and_fragment_order(){
  let mut hash=2166136261u32;
  for operation in 0..3{for scenario in 0..128{
   let mut state=State::default();for(i,r)in state.records.iter_mut().enumerate(){for(j,b)in r.bytes.iter_mut().enumerate(){*b=((i*62+j)*11+scenario*7)as u8;}
    let x=(500+(i*17+scenario*13)%65)as i16-32;let y=(600+(i*29+scenario*7)%65)as i16-32;
    r.set_word(0,if(i+scenario)%7==0{-1}else{x});r.set_word(2,y);r.set_word(8,(if i%3==0{500}else{x})&!15);r.set_word(10,(if i%3==0{600}else{y})&!15);r.set_word(38,if(i+scenario)%5==0{2}else{0});r.set_byte(53,if(i+scenario)%11==0{16}else{0});r.set_byte(60,if(i+scenario)%13==0{6}else{0});r.set_byte(30,if(i+scenario)%17==0{0}else if(i+scenario)%19==0{255}else{4});r.set_word(20,(if i%2==0{-1}else{1})*(2000+i as i16*3));
   }
   let mut fragments=vec![];let found=if operation==2{state.clear_cell([500,600]);false}else{state.damage([500,600],operation==1,|f|fragments.push(f))};
   let record_hash=bytes_hash(state.records.iter().flat_map(|r|r.bytes.iter()));let mut event_hash=2166136261u32;for f in &fragments{for n in[u32::from(f.position[0]as u16),u32::from(f.position[1]as u16),u32::from(f.sprite as u16)]{event_hash=(event_hash^n).wrapping_mul(16777619);}}
   for n in[operation,scenario as u32,u32::from(operation==1&&found),record_hash,fragments.len()as u32,event_hash]{hash=(hash^n).wrapping_mul(16777619);}
  }}
  assert_eq!(hash,3018359808);
 }
}
