//! BUILD type0:40cd20/40d050 and44ebf0 repair admission. Coordinates and
//! arithmetic retain the native signed byte/word widths. Drawing, damage
//! fragments and command dispatch are explicit effects of the callback.
//! The41f610 leaf is not implemented here: its50x18 damage-fragment pool also
//! performs CRT random draws and420200(point,0) environment callbacks. The
//! callback oracle isolates that leaf and does not prove its terrain effects.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct State{pub bytes:[u8;88]}
impl Default for State{fn default()->Self{Self{bytes:[0;88]}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Fault{ZeroMaximum,ZeroFactor,MissingOccupant(i16)}
#[derive(Clone,Copy,Debug)]pub struct Input{pub owner:i8,pub team_rate:i8,pub camera:[i16;2],pub size:[i16;2],pub mode:u8}
#[derive(Clone,Copy,Debug)]pub struct Unit{pub flags31:u8,pub hp:i16,pub maximum:i16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Effect{OpenTerrain,OwnerOverlay,Draw{offset:[i16;2],sprite:i16},Spark([i16;2]),Clock(Option<i32>),Fragments(i16),Complete{slot:i16,full_heal:bool}}
#[derive(Clone,Debug,Default)]pub struct Output{pub effects:Vec<Effect>,pub unit_hp:Option<i16>,pub rate:Option<u8>}
impl State{
 pub fn word(&self,o:usize)->i16{i16::from_le_bytes([self.bytes[o],self.bytes[o+1]])}
 pub fn dword(&self,o:usize)->i32{i32::from_le_bytes(self.bytes[o..o+4].try_into().unwrap())}
 pub fn set_word(&mut self,o:usize,n:i16){self.bytes[o..o+2].copy_from_slice(&n.to_le_bytes());}
 pub fn set_dword(&mut self,o:usize,n:i32){self.bytes[o..o+4].copy_from_slice(&n.to_le_bytes());}
 pub fn position(&self)->[i16;2]{[self.word(0),self.word(2)]}
 pub fn remaining(&self)->i32{self.dword(36)}
 pub fn occupant(&self)->i16{self.word(8)}
 ///Only common40b070 and type0's source-written fields are reset. The raw
 ///clock3e and byte10 survive this constructor and are reset at admission.
 pub fn construct(&mut self,flags6:u8,hp:i16,sparks:u8){self.bytes[4]=0;self.bytes[5]=8;self.bytes[6]=flags6&224;self.bytes[7]=255;self.set_word(8,-1);self.set_word(10,0);self.bytes[17]=sparks;self.bytes[18..22].fill(0);self.set_word(28,hp);self.set_dword(36,0);self.bytes[53]=2;self.set_word(54,0);self.bytes[57]=0;self.bytes[58]=32;self.set_word(60,0);self.bytes[69]=5;}
 pub fn rate(&self,team:i8)->Result<u8,Fault>{let max=i32::from(self.word(28));if max==0{return Err(Fault::ZeroMaximum);}Ok((((i32::from(self.word(30))*100/max)*i32::from(team))/100)as u8)}
 pub fn eta(&self,team:i8)->Result<i32,Fault>{let rate=i32::from(self.rate(team)?);Ok(if rate==0{0}else{self.remaining()/(rate*17)})}
 ///44ebf0: quotient zero alone becomes1; negative missing hull is not
 ///clamped. Factor is signed622dba and supplied by its real caller.
 pub fn enter(&mut self,slot:i16,flags64:u8,hp:i16,maximum:i16,factor:i16)->Result<i32,Fault>{if factor==0{return Err(Fault::ZeroFactor);}let mut q=maximum.wrapping_sub(hp)/5;if q==0{q=1;}let work=(i32::from(q)+1)*884/i32::from(factor);self.set_word(8,slot);self.bytes[5]=flags64&3;self.bytes[62]=0;self.set_dword(36,work);self.set_word(10,0);Ok(work)}
 pub fn advance(&mut self,i:Input,unit:Option<Unit>,mut random:impl FnMut()->u32)->Result<Output,Fault>{
  let mut out=Output::default();let p=self.position();let hp=self.word(30);let maximum=self.word(28);let owned=i.owner<4;
  if owned&&self.bytes[69]==5{self.bytes[69]=4;out.effects.push(Effect::OpenTerrain);}
  self.bytes[62]=self.bytes[62].wrapping_add(1);
  let screen=[p[0].wrapping_sub(i.camera[0]),p[1].wrapping_sub(i.camera[1])];let visible=screen[0]<=i.size[0]&&screen[1]<=i.size[1]&&screen[0]>-80&&screen[1]>-64;
  if visible{
   //40d050's visible ruin path returns immediately, including owned ruins.
   if hp==0{return Ok(out);}
   let threshold=maximum.wrapping_sub((maximum/5).wrapping_mul(3));
   for(o,offset,base)in[(18,[18,5],0xb59i16),(19,[16,32],0xb5b),(20,[6,16],0xb5e)]{out.effects.push(Effect::Draw{offset,sprite:base.wrapping_add(i16::from(self.bytes[o]))});}
   self.bytes[18]=u8::from(owned&&hp>threshold);
   if hp<=threshold{self.bytes[19]=0;}else if i.mode!=0{self.bytes[19]=self.bytes[19].wrapping_add(1);if self.bytes[19]==3{self.bytes[19]=0;}}
   self.bytes[20]=u8::from(owned&&hp>threshold);
   if self.bytes[20]!=0&&self.bytes[69]==5{self.bytes[69]=4;out.effects.push(Effect::OpenTerrain);}
   if owned{out.effects.push(Effect::OwnerOverlay);}
   let stage=self.bytes[69]as i8;if stage!=0{if stage!=1{out.effects.push(Effect::Draw{offset:[16,49],sprite:0x767i16.wrapping_sub(i16::from(stage))});}if stage!=5&&i.mode==0{self.bytes[69]=self.bytes[69].wrapping_sub(1);}}
  }
  if hp==0{out.effects.push(Effect::Fragments(-2));return Ok(out);}
  out.effects.push(Effect::Fragments(maximum.wrapping_sub(hp)));
  if !owned||self.remaining()==0{return Ok(out);}
  let rate=self.rate(i.team_rate)?;out.effects.push(Effect::Clock(if rate==0{None}else{Some(self.remaining()/(i32::from(rate)*17))}));
  //40cf70 runs even offscreen; its spark ctor consumes the separate C rand.
  if i.mode==0{let mut b=self.bytes[17];if b&15==0{out.effects.push(Effect::Spark([p[0].wrapping_add(58),p[1].wrapping_sub(1)]));b=((random()as u8&3).wrapping_add(b&240)).wrapping_add(8);}else{b=b.wrapping_sub(1);}self.bytes[17]=b;if b&240!=0{self.bytes[17]=b.wrapping_sub(16);}else{out.effects.push(Effect::Spark([p[0].wrapping_add(56),p[1].wrapping_add(7)]));self.bytes[17]=(random()as u8&3).wrapping_add(8).wrapping_mul(16).wrapping_add(self.bytes[17]&240);}}
  out.effects.push(Effect::Draw{offset:[32,0],sprite:0xb60i16.wrapping_add(i16::from(self.bytes[21]))});if i.mode!=0{self.bytes[21]=self.bytes[21].wrapping_sub(1)&1;}
  out.rate=Some(rate);if rate==0{return Ok(out);}let mut cancel=false;
  if (self.bytes[62]as i8)>=17{self.bytes[62]=0;let slot=self.occupant();if slot== -1{self.set_word(8,-1);self.set_dword(36,0);return Ok(out);}let u=unit.ok_or(Fault::MissingOccupant(slot))?;
   if(i.owner as u8^u.flags31)&3==0{if u.flags31&4==0{self.set_word(8,-1);self.set_dword(36,0);return Ok(out);}let eta=self.eta(i.team_rate)?;if u.hp<u.maximum&&eta!=0{let addition=((i32::from(u.maximum.wrapping_sub(u.hp))/eta)+1)as i16;out.unit_hp=Some(u.hp.wrapping_add(addition.max(1)));}}
   else{cancel=true;}
  }
  if !cancel{self.set_dword(36,self.remaining().wrapping_sub(i32::from(rate)));if self.remaining()>0{return Ok(out);}}
  let slot=self.occupant();let u=unit.ok_or(Fault::MissingOccupant(slot))?;self.set_dword(36,0);self.set_word(8,-1);if !cancel{out.unit_hp=Some(u.maximum);}out.effects.push(Effect::Complete{slot,full_heal:!cancel});out.effects.push(Effect::Fragments(-2));Ok(out)
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Admission{Absent,Redirect{hold:[i16;2],goal:[i16;2]},Repair}
///44ebf0 compares CP owner&7 against UNIT31&3 before freezing an actor.
pub fn admission(found:bool,owner:u8,flags31:u8,position:[i16;2])->Admission{if !found{Admission::Absent}else if owner&7!=flags31&3{Admission::Redirect{hold:[position[0].wrapping_add(8),position[1].wrapping_add(8)],goal:[position[0].wrapping_add(4),position[1].wrapping_add(56)]}}else{Admission::Repair}}
///40a790 counts only valid CP records. The caller already filters signed
///left<0 and supplies owners from the source64-slot pool.
pub fn team_rates(owners:&[i8])->[i8;4]{let total=owners.len().min(64)as i32;std::array::from_fn(|team|if total==0{0}else{(1+((owners.iter().take(64).filter(|&&o|o==team as i8).count()as i32*100/total)>>2))as i8})}
///17b30;622300's separate cold initialization is8.
pub fn initialized_factor(raw_mode:i16)->i16{let s=match raw_mode{2=>18,3=>22,_=>23};s-(s>>3)-(s>>2)}
///44eba0 scans all35 slots without testing BUILD kind, owner or HP.
pub fn entry_building(positions:&[[i16;2]],unit_cell:i32)->Option<usize>{positions.iter().take(35).position(|p|{let cell=(p[1]as u16&0xfff0).wrapping_mul(8).wrapping_add((p[0]>>4)as u16).wrapping_add(0x181)as i16;i32::from(cell)==unit_cell})}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,n|(h^u32::from(*n)).wrapping_mul(16777619))}
 fn mix(h:&mut u32,ns:&[u32]){for &n in ns{*h=(*h^n).wrapping_mul(16777619);}}
 fn fixture(s:usize)->State{let mut b=State::default();b.set_word(0,400);b.set_word(2,500);b.bytes[5]=(s*13)as u8;b.set_word(8,0);b.set_word(10,(s*117)as i16);b.bytes[17]=(s*17)as u8;for j in 18..22{b.bytes[j]=(s*31+j*7)as u8;}b.set_word(28,[1,5,100,1000,32767,-1,-32768][s/7%7]);b.set_word(30,[0,1,40,100,1000,-1,32767,-32768][s/49%8]);b.set_dword(32,0x1910000);b.set_dword(36,[0,1,25,26,27,884,100000,-1][s/3%8]);b.bytes[58]=32;b.bytes[62]=[0,15,16,17,126,127,128,255][s/11%8];b.bytes[69]=[0,1,2,4,5,128,255][s/37%7];b}
 #[test]fn callbacks_match_all4096_native_build_images_and_repair_sideeffects(){let mut h=2166136261;for s in 0..4096usize{let mut b=fixture(s);let u=Unit{flags31:[0,4,5,7][s/13%4],hp:[0,1,99,100,32767,-1,-32768][s/17%7],maximum:100};let mut rng=0;let out=b.advance(Input{owner:[0,1,3,4,7,-1][s%6],team_rate:[0,1,26,51,127,-1,-128][s/5%7],camera:if s%3==0{[1000,1000]}else{[0,0]},size:[800,600],mode:(s%2)as u8},Some(u),||{let r=(s as u32*7919)^(rng*2027)^0x7731a5c9;rng+=1;r}).unwrap();let mut stamps=0;let mut sparks=0;let mut complete=0;let mut fragments=0;for a in out.effects{match a{Effect::OpenTerrain=>stamps+=1,Effect::Spark(_)=>sparks+=1,Effect::Complete{..}=>complete+=1,Effect::Fragments(_)=>fragments+=1,_=>{}}}mix(&mut h,&[s as u32,hash(&b.bytes),out.unit_hp.unwrap_or(u.hp)as u32,rng,stamps,sparks,complete,complete,fragments]);}assert_eq!(h,1307332844,"ZNativeWorkshopOracle4096actual40d050calls, graphics/damage/command leaves intercepted");}
 #[test]fn admission_matches_all1024_native_build_unit_hardware_and_group_images(){let mut h=2166136261;for s in 0..1024usize{let mut b=fixture(s);let mut u=[0u8;192];let mut w=[0u8;44];let mut g=[0u8;72];u[8..12].copy_from_slice(&0x1900000i32.to_le_bytes());let cell=4378+if s%11==0{1}else{0};u[16..20].copy_from_slice(&(cell as i32).to_le_bytes());u[44..46].copy_from_slice(&424i16.to_le_bytes());u[46..48].copy_from_slice(&548i16.to_le_bytes());u[49]=[0,4,5,7][s/13%4];u[100]=(s*19)as u8;u[105]=96;u[2]=11;u[3]=71;u[92]=7;u[94]=if s%3==0{255}else{0};u[124]=1;w[7]=2;w[12]=33;let hp=[0,1,99,100,32767,-1,-32768][s/17%7];w[8..10].copy_from_slice(&i16::to_le_bytes(hp));w[10..12].copy_from_slice(&100i16.to_le_bytes());let mut selected=(s%2)as u32;let mut dispatch=0;let mut positions=[[0i16;2];35];positions[0]=[400,500];let found=entry_building(&positions,cell).is_some();match admission(found,[0,1,3,4,7,255][s%6],u[49],[424,548]){Admission::Absent=>{},Admission::Redirect{hold,..}=>{u[92]=1;u[80..84].copy_from_slice(&i32::from(hold[0]).to_le_bytes());u[84..86].copy_from_slice(&hold[1].to_le_bytes());g[58]=1;g[44..46].copy_from_slice(&hold[0].to_le_bytes());g[46..48].copy_from_slice(&hold[1].to_le_bytes());u[183]&=224;u[156]|=16;dispatch=1;},Admission::Repair=>{b.enter(0,u[100],hp,100,[1,12,17,-1][s/23%4]).unwrap();u[105]|=64;w[7]=w[12];u[2]=u[3];if selected==1{selected=0;}}}mix(&mut h,&[s as u32,hash(&b.bytes),hash(&u),hash(&w),hash(&g),dispatch,selected]);}assert_eq!(h,664895437,"ZNativeWorkshopOracle1024actual44ebf0/44eba0calls, command/audio leaves intercepted");}
 #[test]fn team_rate_snapshot_matches_all1024_native64slot_counts(){let mut h=2166136261;for s in 0..1024usize{let owners=(0..64).filter(|&j|(s+j*3)%7!=0).map(|j|[0,1,2,3,4,-1][(s+j*7)%6]).collect::<Vec<_>>();mix(&mut h,&[s as u32,hash(&team_rates(&owners).map(|n|n as u8))]);}assert_eq!(h,47686517,"ZNativeWorkshopGlobalsOracle40a7901024actual64CPscans");assert_eq!(team_rates(&[]),[0;4]);}
 #[test]fn production_factor_matches_all8_native_initializer_branches(){for(mode,factor)in[(-32768,16),(-1,16),(0,16),(1,16),(2,12),(3,15),(4,16),(32767,16)]{assert_eq!(initialized_factor(mode),factor);}assert_eq!(crate::Game::new(0,1,8).native_work_factor,8,"462300 cold initialization precedes17b30 campaignmode2");}
 #[test]fn admission_preserves_negative_missing_hull_and_wraps_difference_before_dividing(){let mut b=State::default();assert_eq!(b.enter(3,255,99,100,12),Ok(147));assert_eq!(b.enter(3,3,150,100,12),Ok(-663));assert_eq!(b.enter(3,0,-32768,32767,12),Ok(147));assert_eq!(b.enter(3,0,99,100,0),Err(Fault::ZeroFactor));}
 #[test]fn occupied_and_invalidated_slots_wait_for_the_signed17_tick_test(){let mut b=fixture(0);b.set_word(28,100);b.set_word(30,100);b.set_dword(36,10000);b.bytes[62]=127;b.set_word(8,-1);let i=Input{owner:0,team_rate:26,camera:[1000,1000],size:[800,600],mode:1};let o=b.advance(i,None,||0).unwrap();assert_eq!(b.remaining(),9974);assert_eq!(b.bytes[62],128);assert!(!o.effects.iter().any(|a|matches!(a,Effect::Complete{..})));b.bytes[62]=16;b.advance(i,None,||0).unwrap();assert_eq!(b.remaining(),0);assert_eq!(b.occupant(),-1);}
}
