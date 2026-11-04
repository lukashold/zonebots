//!58660/563c0's separate15-record removal animation pool. UNIT slots are
//!released immediately; these records never keep a selectable UNIT alive.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Record{pub sprite:i16,pub flags:u16,pub position:[i16;2]}
impl Default for Record{fn default()->Self{Self{sprite:-1,flags:0,position:[0,0]}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Draw{pub slot:u8,pub position:[i16;2],pub screen:[i16;2],pub sprite:i16,pub color:u8}
pub struct State{pub records:[Record;15],pub draws:Vec<Draw>}
impl Default for State{fn default()->Self{Self{records:[Record::default();15],draws:Vec::new()}}}
//004971e8: sprite base and packed remaining-frame count/color flags.
pub const REMOVAL_FRAMES:[[u16;2];17]=[[422,2560],[432,2560],[442,2560],[452,2048],[1965,2304],[1965,2304],[1974,2304],[1983,2304],[1992,2304],[2001,1792],[2008,1792],[2015,1792],[422,2560],[432,2560],[442,2560],[452,2048],[167,4352]];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RemovalSource{Bullet,Blast,Direct,Collapse}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EmitterKind{Destroyed,Fragment,Poof}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action{Remove{suppressed:bool},Sound(u8),Emit{kind:EmitterKind,count:u8,sprite:i16,maximum:i8,origin:[i16;2],owner:u8}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ConversionInput{pub family:u8,pub class:u8,pub hull:i16,pub position:[i16;2],pub owner:u8,pub suppressed:bool,pub crew_only:bool,pub removing:bool,pub forced_wreck:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Conversion{pub family_after:u8,pub object:Option<Wreck>,pub recursive_poof:bool,pub jeep_contact:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Wreck{pub kind:u16,pub position:[i16;2],pub flags13:u8,pub wreck:bool,pub align:bool}
///58af0 family30 after conversion. crew_only is the actual462a70 context:
///4a6f7e=1 around41e6e0; it is independent from5849e7 suppression.
///Sharing this decision keeps the captured removed UNIT and the OBJECT caller
///on the same branch without a second allocation or RNG pass.
const CONVERTIBLE_FAMILIES:[u8;12]=[5,6,21,22,23,24,25,30,32,34,36,38];
pub fn conversion_family(family:u8,class:u8,suppressed:bool,crew_only:bool)->u8{
 if CONVERTIBLE_FAMILIES.contains(&family)&&suppressed&&!(crew_only&&matches!(class,1|2)){family}else{255}
}
///58af0's own numerical offsets. The extra−4 X precedes both the usable
///hardware and destroyed-hardware branches; APC family32 skips both offsets.
pub fn conversion(input:ConversionInput)->Conversion{
 let mut result=Conversion{family_after:255,object:None,recursive_poof:false,jeep_contact:false};let f=input.family;if f==255{result.family_after=255;return result;}if f==4{return result;}
 let known=CONVERTIBLE_FAMILIES.contains(&f);
 result.family_after=conversion_family(f,input.class,input.suppressed,input.crew_only);
 if result.family_after!=255{return result;}
 let offset=match f{6=>[-4,-6],21=>[4,0],22=>[0,-1],23=>[3,-6],25=>[-2,-4],30=>[-7,-7],34=>[0,-7],_=>[0,0]};
 let point=if f==32{input.position}else{[input.position[0].wrapping_add(offset[0]).wrapping_sub(if known{4}else{0}),input.position[1].wrapping_add(offset[1])]};
 let recoverable=known&&!input.forced_wreck&&input.hull>=0;
 let kind=if recoverable{match f{6=>8,25=>26,30=>29,32=>31,34=>33,36=>35,38=>37,_=>u16::from(f)}}else{match f{5=>65,21=>63,22=>64,23=>66,24=>27,_=>u16::from(f)}};
 result.recursive_poof=recoverable&&!input.removing;result.jeep_contact=!recoverable&&f==34;
 let wreck=!recoverable;result.object=Some(Wreck{kind,position:point,flags13:((input.owner&3|24)*2)&!1|64,wreck,align:wreck&&[6,25,27,30,32,34,36,38].contains(&kind)});result
}
///58af0 serializes the remaining hull in25-point units, reserving zero for
///other meanings. Signed negative hulls still leave an OBJECT with one HP.
pub fn object_hull(hull:i16)->u8{let value=(i32::from(hull)/25)as u8;if hull<0||value==0{1}else{value}}
///43f200's word24 uses all16 bits. Zero means no expiry; every other bit
///pattern decrements with wrapping arithmetic and expires only on reaching0.
pub fn advance_object_lifetime(value:&mut i16)->bool{if *value==0{return false;}*value=value.wrapping_sub(1);*value==0}
const WRECK_TERRAIN:[u8;128]=[0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,255,255,255,255,255,255,0,0,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,0,255,255,255,255,255,255,255,0,100,109,108,111,115,101,50,0,99,104,95,111,46,100,98,103,0,0,0,0,0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,255,255,255,255,255,255,0,0,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255];
const FAMILY_EXTENT:[i8;64]=[0,0,0,0,0,1,2,0,2,0,0,0,0,0,0,0,0,0,0,0,0,3,10,11,12,4,4,0,0,5,5,6,6,7,7,8,8,9,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,13,13,0,0,0,0];
const INSERTION_TERRAIN:[u8;128]=[0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,255,255,255,255,255,255,0,0,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,0,255,255,255,255,255,255,255,0,100,101,115,101,114,116,46,112,97,108,0,0,0,0,0,0,100,101,115,101,114,116,46,98,108,107,0,0,0,0,0,0,52,135,73,0,16,139,73,0,180,139,73,0,204,118,73,0,254,255,255,255,0,0,0,0,0,0,0,0];
pub fn insertion_allowed(kind:u16,terrain:[u8;4])->bool{[5,21,22,23].contains(&kind)||terrain.into_iter().all(|c|INSERTION_TERRAIN[usize::from(c&127)]==0)}
///44dd70's96 records are the+8 packed waypoints of the same136-byte MAP road
///records used by43b6f0. The native loop includes unused/zero waypoints.
pub fn road_waypoint_overlap(point:[i16;2],waypoints:impl IntoIterator<Item=u16>)->bool{waypoints.into_iter().take(96).any(|packed|{let center=[((packed&127)*16+8)as i16,((packed as i16>>3)&!15).wrapping_add(8)];(0..2).all(|a|point[a]<=center[a]&&i32::from(center[a])-32<=i32::from(point[a]))})}
#[derive(Clone,Copy,Debug)]pub struct PlacementUnit{pub position:[i16;2],pub family:u8,pub flags31:u8,pub phase35:u8,pub self_unit:bool}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Placement{pub position:[i16;2],pub saved:Option<[u8;4]>,pub terrain:[u8;4],pub world_probe:bool,pub contacts:Vec<([i16;2],u8)>}
///44de00 queries the independent96-record world pool only after terrain and
///UNIT overlaps pass. The callback result is an explicit input, not a guessed
///replacement by ordinary BUILD or sector rectangles.
pub fn place_wreck(point:[i16;2],terrain:[u8;4],hardware_saved:Option<[u8;4]>,units:&[PlacementUnit],world_overlap:bool)->Placement{
 let position=[point[0].wrapping_add(7)&!15,point[1].wrapping_add(7)&!15];let mut result=Placement{position,saved:None,terrain,world_probe:false,contacts:Vec::new()};let mut collision=false;let mut strength=0;
 for code in terrain{strength=code&127;if WRECK_TERRAIN[usize::from(strength)]!=0{collision=true;break;}}
 if !collision{collision=units.iter().take(240).any(|u|{if u.flags31&4==0||u.phase35==18||u.self_unit{return false;}let extent=(i32::from(*FAMILY_EXTENT.get(usize::from(u.family)).unwrap_or(&0))+2)*8;i32::from(position[0])<=i32::from(u.position[0])+extent&&i32::from(position[1])<=i32::from(u.position[1])+extent&&i32::from(u.position[0])<=i32::from(position[0])+16&&i32::from(u.position[1])<=i32::from(position[1])+16});}
 if !collision{result.world_probe=true;collision=world_overlap;}
 if !collision{result.saved=Some(hardware_saved.unwrap_or(terrain));result.terrain=terrain.map(|c|(c&0x9c)|28);}else{for delta in[[-12,-20],[12,-20],[12,4],[-12,4]]{result.contacts.push(([position[0].wrapping_add(delta[0]),position[1].wrapping_add(delta[1])],strength));}}
 result
}
fn emit(actions:&mut Vec<Action>,kind:EmitterKind,count:u8,sprite:i16,maximum:i8,point:[i16;2],owner:u8){actions.push(Action::Emit{kind,count,sprite,maximum,origin:[point[0].wrapping_sub(152),point[1].wrapping_sub(84)],owner:owner&3});}
pub fn sound_visible(point:[i16;2],camera:[i16;2],size:[i16;2])->bool{let p=[point[0].wrapping_sub(camera[0]),point[1].wrapping_sub(camera[1])];i32::from(p[0])>=-48&&i32::from(p[0])<=i32::from(size[0])+32&&i32::from(p[1])>=-48&&i32::from(p[1])<=i32::from(size[1])+32}
///41ef80's ordered allocation requests. The caller carries the shared audio
///gate; suppress_sound corresponds only to4a6f3b, not particle suppression.
pub fn breakup(family:u8,point:[i16;2],owner:u8,audible:bool)->Vec<Action>{let mut out=Vec::new();if audible{out.push(Action::Sound(6));}emit(&mut out,EmitterKind::Destroyed,4,557,5,point,owner);let sprite=match family{6|38=>1558,24=>1869,25=>5337,30=>5739,32=>5679,36=>5818,_=>return out};emit(&mut out,EmitterKind::Fragment,1,sprite,7,point,owner);out}
///41e6e0 keeps removal before the1903 particle and vehicle debris. This is
///not an alternative death pose held in UNIT.phase35.
pub fn exploded(class:u8,family:u8,point:[i16;2],owner:u8,crew_only:bool,audible:bool)->Vec<Action>{
 let mut out=vec![Action::Remove{suppressed:true}];emit(&mut out,EmitterKind::Poof,1,1903,7,point,owner);
 if class!=1||crew_only{return out;}
 let parts:&[(i16,i8,u8)]=match family{
  6=>&[(1558,7,1),(512,0,1),(521,7,1),(533,7,1)],
  24=>&[(1869,7,1),(1845,0,1)],25=>&[(5337,7,1),(5317,0,1),(533,7,1)],
  30=>&[(5739,7,1),(5747,0,1),(533,7,1)],32=>&[(5679,7,1),(5688,0,1)],
  34=>&[(5697,3,4),(5687,0,1)],36=>&[(5818,7,1),(5696,0,1)],38=>return out,
  _=>{out.push(Action::Remove{suppressed:true});emit(&mut out,EmitterKind::Poof,1,1903,7,point,owner);return out;}
 };
 if audible{out.push(Action::Sound(6));}
 for &(sprite,maximum,count)in parts{emit(&mut out,EmitterKind::Fragment,count,sprite,maximum,point,owner);}out
}
///Source callsites462a70/208f0/1f2d0/59d10 select distinct removal chains.
pub fn removal_actions(source:RemovalSource,class:u8,family:u8,point:[i16;2],owner:u8,crew_only:bool,audible:bool)->Vec<Action>{
 if crew_only{return exploded(class,family,point,owner,true,false);}
 let plain=source==RemovalSource::Bullet&&[0,3,15,16].contains(&class);
 let broken=match source{RemovalSource::Bullet=>!plain,RemovalSource::Blast=>[1,2].contains(&class),RemovalSource::Direct=>class==1,RemovalSource::Collapse=>false};
 if plain{return vec![Action::Remove{suppressed:false}];}
 if !broken{return exploded(class,family,point,owner,false,audible);}
 let mut out=breakup(family,point,owner,audible&&!(source==RemovalSource::Blast&&class==2));out.push(Action::Remove{suppressed:false});
 if source==RemovalSource::Blast&&class==2{out.push(Action::Emit{kind:EmitterKind::Destroyed,count:4,sprite:557,maximum:5,origin:[point[0].wrapping_sub(154),point[1].wrapping_sub(80)],owner:owner&3});}out
}
impl State{
 ///The caller supplies native5849e7 and4d029f separately. Suppression and
 ///a full pool consume no RNG; forced removal uses table row16 directly.
 pub fn allocate(&mut self,position:[i16;2],flags64:u8,suppressed:bool,forced:bool,mut random:impl FnMut()->u32)->Option<usize>{
  if suppressed{return None;}let slot=self.records.iter().position(|r|r.sprite==-1)?;let choice=if forced{16}else{(random()&15)as usize};let row=REMOVAL_FRAMES[choice];self.records[slot]=Record{sprite:row[0]as i16,flags:row[1]|u16::from(flags64&3),position};Some(slot)
 }
 ///563c0 draws before the shared alternate-callback frame increment. Unlike
 ///4028a0 particles, these animations advance even outside the viewport.
 pub fn advance(&mut self,camera:[i16;2],viewport:[i16;2],pulse:bool){
  self.draws.clear();for(slot,r)in self.records.iter_mut().enumerate(){if r.sprite==-1{continue;}let screen=[r.position[0].wrapping_sub(camera[0]),r.position[1].wrapping_sub(camera[1])];
   if screen[0]>-16&&screen[0]<viewport[0]&&screen[1]>-16&&screen[1]<viewport[1]{self.draws.push(Draw{slot:slot as u8,position:r.position,screen,sprite:r.sprite,color:r.flags as u8});}
   if pulse{r.sprite=r.sprite.wrapping_add(1);r.flags=r.flags.wrapping_sub(256);if r.flags&0xff00==0{r.sprite=-1;}}
  }
 }
 pub fn bytes(&self)->[u8;120]{let mut out=[0;120];for(i,r)in self.records.iter().enumerate(){for(j,v)in[r.sprite as u16,r.flags,r.position[0]as u16,r.position[1]as u16].into_iter().enumerate(){out[i*8+j*2..i*8+j*2+2].copy_from_slice(&v.to_le_bytes());}}out}
}

#[cfg(test)]mod tests{
 use super::*;
 fn bytes_hash(bytes:&[u8])->u32{bytes.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619))}
 fn values(hash:&mut u32,ns:&[u32]){for &n in ns{*hash=(*hash^n).wrapping_mul(16777619);}}
 #[test]fn wreck_lifetime_matches_all_1024_executed_native_records(){let mut total=2166136261u32;for s in 0..1024u32{let mut life=if s<512{s as i16-256}else{s.wrapping_mul(83)as i16};let expired=advance_object_lifetime(&mut life);let mut eh=2166136261u32;if expired{values(&mut eh,&[347,421,80,1,1]);}values(&mut total,&[s,life as u32,if expired{u32::MAX}else{346},u32::from(expired),eh]);}assert_eq!(total,689294762,"ZNativeWreckLifetimeOracle:1024 signed/wrapping43f200 inputs");}
 #[test]fn wreck_placement_matches_all_2048_executed_native_records(){
  let mut total=2166136261u32;let codes=[0,7,8,10,23,24,28,35,60,68,127];let delta=[-65,-33,-17,-16,-1,0,16,17,32,33,65];
  for s in 0..2048u32{let point=[346+(s%9)as i16-4,420+(s/9%9)as i16-4];let aligned=point.map(|c|c.wrapping_add(7)&!15);let terrain=std::array::from_fn(|i|codes[(s as usize+i*3)%codes.len()]|(s as u8&128));let unit=PlacementUnit{position:[aligned[0]+delta[s as usize%11],aligned[1]+delta[s as usize/11%11]],family:(s%39)as u8,flags31:(s as u8&1)*4,phase35:if s&2!=0{18}else{0},self_unit:false};let plan=place_wreck(point,terrain,(s&4!=0).then_some([120,121,122,123]),&[unit],s>>3&1!=0);
   let mut object=[0u8;44];object[0..2].copy_from_slice(&plan.position[0].to_le_bytes());object[2..4].copy_from_slice(&plan.position[1].to_le_bytes());object[20..24].copy_from_slice(&plan.saved.unwrap_or([240,241,242,243]));let mut events=0;let mut eh=2166136261u32;if plan.world_probe{events+=1;values(&mut eh,&[0,plan.position[0]as u32,plan.position[1]as u32]);}for &(p,strength)in &plan.contacts{events+=1;values(&mut eh,&[1,p[0]as u32,p[1]as u32,u32::from(strength),1]);}
   values(&mut total,&[s,bytes_hash(&object),bytes_hash(&plan.terrain),events,eh,0]);
  }assert_eq!(total,165218311,"ZNativeWreckPlacementOracle:2048 original44de00 terrain, UNIT and world-probe cases");
 }
 #[test]fn captured_family_matches_64_actual_removal_caller_results(){
  //ZV45RemovedShooterCrewOracle:32 direct58ec0 and32 crew-only41e6e0.
  //Only family30 is compared here, not the whole native damage/removal world.
  let mut h=2166136261u32;
  for s in 0..64u32{let crew=s>=32;let class=if crew||s/2%2!=0{1}else{0};let family=if class==1{6}else{4};let after=conversion_family(family,class,crew||s&8!=0,crew);values(&mut h,&[s,u32::from(after)]);}
  assert_eq!(h,986074885,"64 real caller final family30 values");
  if let Some(path)=std::env::var_os("NATIVE_REMOVAL_CREW_ORACLE"){
   let text=std::fs::read_to_string(path).unwrap();let rows=text.lines().map(|l|serde_json::from_str::<serde_json::Value>(l).unwrap()).collect::<Vec<_>>();assert_eq!(rows.len(),128);
   for scenario in 0..64u64{let row=|stage:&str|rows.iter().find(|r|r["scenario"].as_u64()==Some(scenario)&&r["stage"]==stage).unwrap();let decode=|v:&serde_json::Value|{let h=v["unit"].as_str().unwrap();(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect::<Vec<_>>()};let before=decode(row("before"));let after=decode(row("after"));let crew=scenario>=32;assert_eq!(after[48],conversion_family(before[48],before[124],crew||scenario&8!=0,crew),"scenario{scenario}");assert_eq!(&before[44..48],&after[44..48]);}
  }
 }
 #[test]fn conversion_matches_all_1664_executed_native_records(){
  let mut total=2166136261u32;let families=[4,5,6,21,22,23,24,25,30,32,34,36,38];let hulls=[-300,-1,0,1,24,25,125];
  for s in 0..1664u32{let hull=hulls[s as usize/13%7];let class=(s/91%4)as u8;let owner=s as u8;let spec=conversion(ConversionInput{family:families[s as usize%13],class,hull,position:[346,420],owner,suppressed:s>>2&1!=0,crew_only:s>>1&1!=0,removing:s>>3&1!=0,forced_wreck:s>>4&1!=0});
   let mut object=[0u8;44];for(i,b)in object.iter_mut().enumerate(){*b=(i as u32*17+s)as u8;}let mut hardware=[0u8;64];for(i,b)in hardware.iter_mut().enumerate(){*b=(i as u32*11+s)as u8;}hardware[8..10].copy_from_slice(&hull.to_le_bytes());hardware[10..12].copy_from_slice(&300i16.to_le_bytes());let mut event_hash=2166136261u32;let mut events=0;let mut calls=0;let mut freed=false;
   if spec.recursive_poof{values(&mut event_hash,&[0]);events+=1;}if spec.jeep_contact{let p=spec.object.unwrap().position;values(&mut event_hash,&[2,p[0]as u32,p[1]as u32]);events+=1;}
   if let Some(w)=spec.object{let[x,y]=w.position;values(&mut event_hash,&[1,x as u32,y as u32,u32::from(w.kind)]);events+=1;
    if s>>5&1==0{
     object[0..2].copy_from_slice(&x.to_le_bytes());object[2..4].copy_from_slice(&y.to_le_bytes());object[4..8].copy_from_slice(&(0x498f38+u32::from(w.kind)*26).to_le_bytes());object[8..12].copy_from_slice(&(-1i32).to_le_bytes());object[12]=(((y>>3)&240)+(x>>7))as u8;object[13]=w.flags13;object[18..20].copy_from_slice(&0x0f00u16.to_le_bytes());object[36..38].copy_from_slice(&0u16.to_le_bytes());object[33..35].copy_from_slice(&0u16.to_le_bytes());
     for(to,from)in[(14,5),(17,6),(16,7),(24,32),(25,33),(30,38),(29,37),(27,35),(28,36)]{object[to]=hardware[from];}if w.kind==31{object[17]=0;}object[40]=12;object[15]=object_hull(hull);freed=true;
     if w.wreck{object[19]=0;object[36..38].copy_from_slice(&0x120u16.to_le_bytes());object[38]=((s.wrapping_mul(7919)^0x7731a5c9)&3)as u8;calls+=1;if w.align{values(&mut event_hash,&[3]);events+=1;}}
    }
   }
   values(&mut total,&[s,events,event_hash,calls,bytes_hash(&object),u32::from(spec.family_after),u32::from(freed)]);
  }assert_eq!(total,2231136454,"ZNativeWreckOracle:1664 original58af0 conversions, allocation/placement callbacks recorded separately");
 }
 #[test]fn death_emitters_match_all_1248_executed_native_sequences(){
  let mut total=2166136261u32;let families=[4,5,6,21,22,23,24,25,30,32,34,36,38];let points=[[160,100],[-49,-49],[-48,-48],[352,232],[353,233],[32767,-32768]];
  for function in 0..2u32{for s in 0..624u32{let family=families[s as usize%13];let class=(s/13%4)as u8;let point=points[s as usize/52%6];let visible=sound_visible(point,[0,0],[320,200]);let gate=s&1!=0;let enabled=s>>1&1!=0;let suppress=s>>2&1!=0;let crew=s>>3&1!=0;
   let has_sound=if function==0{!suppress}else{class==1&&!crew&&[6,24,25,30,32,34,36].contains(&family)};
   let audible=visible&&!gate&&enabled&&has_sound;let actions=if function==0{breakup(family,point,s as u8,audible)}else{exploded(class,family,point,s as u8,crew,audible)};
   let mut event_hash=2166136261u32;for a in &actions{match *a{Action::Remove{suppressed}=>values(&mut event_hash,&[0,u32::from(suppressed),u32::from(suppressed)]),Action::Sound(id)=>values(&mut event_hash,&[2,u32::from(id)]),Action::Emit{kind,count,sprite,maximum,origin,owner}=>values(&mut event_hash,&[1,u32::from(count),origin[0]as u32,origin[1]as u32,12,sprite as u32,maximum as u32,match kind{EmitterKind::Destroyed=>0x4017b0,EmitterKind::Fragment=>0x403200,EmitterKind::Poof=>0x402770},u32::from(owner),u32::from(kind==EmitterKind::Fragment)])}}
   values(&mut total,&[function*624+s,actions.len()as u32,event_hash,u32::from(gate||visible&&has_sound),0,0]);
  }}assert_eq!(total,3123041500,"ZNativeDeathPlanOracle:1248 original41ef80/41e6e0 sequences");
 }
 #[test]fn allocation_and_draw_lifetime_match_all_1536_executed_native_records(){
  let mut hash=2166136261u32;
  for s in 0..256u32{let mut pool=State::default();for(slot,r)in pool.records.iter_mut().enumerate(){*r=Record{sprite:if slot<(s as usize>>3)%16{1000+slot as i16}else{-1},flags:(slot*319)as u16,position:[500+slot as i16,600-slot as i16]};}
   let mut calls=0;if s&1!=0{pool.allocate([500,600],s.wrapping_mul(13)as u8,s&2!=0,s&4!=0,||{calls+=1;s.wrapping_mul(2027)^0x7731a5c9});}
   values(&mut hash,&[s,calls,0,bytes_hash(&pool.bytes())]);
  }
  let points=[[160i16,100],[-16,0],[-15,0],[319,199],[320,200],[32767,-32768],[-32768,32767],[500,500]];let cameras=[[0i16,0],[16,16],[-16,-16],[32760,-32760]];
  for s in 0..32usize{let mut pool=State::default();let row=REMOVAL_FRAMES[s%17];pool.records[0]=Record{sprite:row[0]as i16,flags:row[1]|(s&3)as u16,position:points[s%8]};for tick in 1..=40{pool.advance(cameras[s/8],[320,200],tick&1!=0);let mut draw_bytes=Vec::new();for d in &pool.draws{for n in[d.screen[0]as u16,d.screen[1]as u16,d.sprite as u16,u16::from(d.color)]{draw_bytes.extend(n.to_le_bytes());}}
   values(&mut hash,&[s as u32,tick,bytes_hash(&pool.bytes()),pool.draws.len()as u32,bytes_hash(&draw_bytes)]);
  }}
  assert_eq!(hash,1284791982,"ZNativeRemovalOracle:256allocations+1280lifecyclerecords");
 }
 #[test]fn no_unit_slot_is_reserved_by_a_removal_animation_and_the_pool_is_exactly_fifteen(){
  let mut state=State::default();for slot in 0..15{assert_eq!(state.allocate([0,0],1,false,false,||0),Some(slot));}assert_eq!(state.allocate([0,0],1,false,false,||panic!("a full15-record pool must not consume RNG")),None);
  for _ in 0..10{state.advance([1000,1000],[320,200],true);}assert!(state.records.iter().all(|r|r.sprite==-1));assert_eq!(state.allocate([0,0],1,false,true,||panic!("forced row16 must not consume RNG")),Some(0));assert_eq!(state.records[0].sprite,167);
 }
}
