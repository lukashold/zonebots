//! APC squad transport:00450890/00450740/00450b30, entry00456c00/00456d50,
//! unload00459830/00459720 and exit00456f20. Coordinates are native pixels.
//! The caller retains robot identity, crew health, squad and live route geometry.
use serde::{Serialize,Deserialize};

pub const CAPACITY:usize=21; // Native group roster includes the driving robot.
pub type Point=(i32,i32);
pub const ENTRY_PORTALS:[Point;8]=[(7,16),(-2,19),(-16,5),(0,0),(7,-1),(16,-2),(20,6),(17,16)];
pub const EXIT_PORTALS:[Point;8]=[(0,0),(16,-12),(30,0),(16,16),(7,15),(-1,16),(-14,12),(-14,-12)];
pub const RELEASE_WALK:[Point;8]=[(0,-16),(15,15),(16,0),(15,15),(0,16),(-15,15),(-16,0),(-15,-15)];
const ENTRY_FRAMES:[u8;8]=[5,3,1,1,1,1,1,3];
const EXIT_FRAMES:[u8;8]=[2,2,2,4,6,4,2,2];
fn add(a:Point,b:Point)->Point{((a.0 as i16).wrapping_add(b.0 as i16)as i32,(a.1 as i16).wrapping_add(b.1 as i16)as i32)}
pub fn entry_portal(origin:Point,direction:u8)->Point{add(origin,ENTRY_PORTALS[usize::from(direction&7)])}
pub fn exit_portal(origin:Point,vehicle_direction:u8)->Point{add(origin,EXIT_PORTALS[usize::from((vehicle_direction&7)^4)])}
pub fn occupied_origin(parked:Point)->Point{add(parked,(6,6))}
///00458af0 maps family32 to objecttype31, which skips0044de00's alignment.
pub fn parked_origin(occupied:Point)->Point{occupied}
///00459830 tries the current body direction, then the next seven clockwise.
/// Terrain footprints and object collisions remain the caller's live queries.
pub fn choose_direction(mut direction:u8,mut clear:impl FnMut(u8)->bool)->Option<u8>{for _ in 0..8{direction&=7;if clear(direction){return Some(direction);}direction=direction.wrapping_add(1);}None}
///497094 selects the emergence side, behind the vehicle's current heading.
pub const FOOTPRINT_CELL_OFFSETS:[i16;8]=[256,254,-2,-258,-256,-254,2,258];
//457b60 reads the unmodified499888 path table, including its byte-addressable
//tail. These are semantic blocked bits, not the physical/team collision mask.
const FOOTPRINT_BLOCKED:[u8;128]=[
 0,1,1,1,1,1,1,0,0,0,0,1,1,1,1,1,
 1,1,1,1,0,0,0,0,1,1,0,0,1,1,0,1,
 1,1,1,0,0,0,0,0,1,0,0,1,0,0,1,1,
 1,1,1,0,1,0,0,0,1,1,1,0,0,0,0,0,
 0,0,0,0,0,0,0,0,1,0,1,1,0,0,0,0,
 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
 1,0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,
 0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,
];
///457b60: a2×2 terrain footprint; odd directions also test the third column.
///The cells are evaluated in this exact short-circuit order.433b30 mutates
///four path-table entries in native Volcan world3 before these queries run.
pub fn footprint_blocked(base:i16,direction:u8,volcan:bool,mut terrain:impl FnMut(i16)->u8)->bool{
 let base=base.wrapping_add(FOOTPRINT_CELL_OFFSETS[usize::from(direction&7)]);
 for offset in [0i16,1,128,129,2,130].into_iter().take(if direction&1!=0{6}else{4}){let code=terrain(base.wrapping_add(offset))&127;if FOOTPRINT_BLOCKED[usize::from(code)]!=0||volcan&&matches!(code,9|38|39|42){return true;}}false
}
///59830 calls41c600 at the unchanged vehicle origin after every successful
///terrain test. Its caller owns OBJECT pool order and one-shot query controls.
pub fn choose_exit(origin:[i16;2],direction:u8,volcan:bool,mut terrain:impl FnMut(i16)->u8,mut object_blocked:impl FnMut([i16;2])->bool)->Option<u8>{
 let base=((origin[1]as u16&0xfff0).wrapping_mul(8)).wrapping_add((origin[0]>>4)as u16)as i16;
 choose_direction(direction,|d|!footprint_blocked(base,d,volcan,&mut terrain)&&!object_blocked(origin))
}
///00458af0 serializes hull health to the native object's25-HP byte units.
pub fn parked_hull(hp:i16)->i16{let q=hp/25;i16::from(if q==0||hp<0{1}else{q as u8})*25}

#[derive(Clone,Copy,PartialEq,Eq,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Phase{Approach,Entering,Inside,Exiting,Released}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Animation{
 pub phase:Phase,pub direction:u8,pub frame:u8,pub visible:bool,
 timer:i8,
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Transition{None,Hidden,Released{goal:Point}}
impl Animation{
 pub fn approach(direction:u8)->Self{Self{phase:Phase::Approach,direction:direction&7,frame:0,visible:true,timer:0}}
 pub fn arrive(&mut self)->bool{if self.phase!=Phase::Approach{return false;}self.phase=Phase::Entering;self.frame=0;self.timer=2;true}
 pub fn exiting(direction:u8,rank:usize,was_visible:bool)->Self{Self{phase:Phase::Exiting,direction:direction&7,frame:0,visible:was_visible,timer:(5+rank.min(20)*6)as i8}}
 pub(crate) fn pose(&self)->(i16,i8){(i16::from(self.frame),self.timer)}
 pub fn sprite(&self)->Option<usize>{match self.phase{Phase::Entering=>Some(5217+usize::from(self.direction)*5+usize::from(self.frame)),Phase::Exiting=>Some(5257+usize::from(self.direction)*6+usize::from(self.frame)),_=>None}}
 pub fn hidden(&self)->bool{!self.visible}
 pub fn advance(&mut self,position:Point,mut random:impl FnMut()->u32)->Transition{
  match self.phase{
   Phase::Entering|Phase::Exiting=>{
    self.timer=self.timer.wrapping_sub(1);if self.timer>=0{return Transition::None;}
    self.frame=self.frame.wrapping_add(1);self.timer=1;
    if self.phase==Phase::Entering&&self.frame==ENTRY_FRAMES[usize::from(self.direction)]{
     self.phase=Phase::Inside;self.visible=false;return Transition::Hidden;
    }
    if self.phase==Phase::Exiting&&self.frame==EXIT_FRAMES[usize::from(self.direction)]+1{
     self.phase=Phase::Released;self.visible=true;self.frame=0;
     let jitter=((random()&15)as i32-8,(random()&15)as i32-8);
     return Transition::Released{goal:add(add(position,RELEASE_WALK[usize::from(self.direction)]),jitter)};
    }
   },_=>{}
  }
  Transition::None
 }
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Member{pub id:usize,pub animation:Animation}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Boarding{
 pub carrier:usize,pub squad:usize,pub origin:Point,pub direction:u8,
 pub members:Vec<Member>,pub occupant:Option<usize>,
 #[serde(skip)] pub occupied_tick:Option<u64>,
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Event{None,Hidden{member:usize},Occupy{member:usize},Released{member:usize,goal:Point}}
impl Boarding{
 pub fn new(carrier:usize,squad:usize,origin:Point,direction:u8,ids:&[usize])->Result<Self,&'static str>{
  if ids.is_empty()||ids.len()>CAPACITY{return Err("An APC requires one native squad of1..21 robots");}
  if ids.iter().enumerate().any(|(i,id)|ids[..i].contains(id)){return Err("Duplicate robot in APC squad");}
  Ok(Self{carrier,squad,origin,direction:direction&7,members:ids.iter().map(|id|Member{id:*id,animation:Animation::approach(direction)}).collect(),occupant:None,occupied_tick:None})
 }
 pub fn portal(&self)->Point{entry_portal(self.origin,self.direction)}
 pub fn vehicle_origin(&self)->Point{occupied_origin(self.origin)}
 pub fn arrive(&mut self,id:usize)->bool{self.members.iter_mut().find(|m|m.id==id).is_some_and(|m|m.animation.arrive())}
 pub fn all_inside(&self)->bool{!self.members.is_empty()&&self.members.iter().all(|m|m.animation.phase==Phase::Inside)}
 /// Call once per robot callback in native unit-slot order. A robot that just
 /// became hidden cannot occupy the vehicle until its next callback.
 pub fn advance(&mut self,id:usize,position:Point,random:impl FnMut()->u32)->Event{
  let Some(index)=self.members.iter().position(|m|m.id==id)else{return Event::None;};
  if self.members[index].animation.phase==Phase::Inside&&self.occupant.is_none()&&self.all_inside(){let leader=self.members[0].id;self.occupant=Some(leader);return Event::Occupy{member:leader};}
  match self.members[index].animation.advance(position,random){Transition::Hidden=>Event::Hidden{member:id},Transition::Released{goal}=>Event::Released{member:id,goal},Transition::None=>Event::None}
 }
 /// Returns the shared emergence anchor, given the parked OBJECT origin;
 /// every member keeps its identity and squad and receives a staggered timer.
 pub fn unload(&mut self,origin:Point,direction:u8)->Point{
  self.origin=origin;self.direction=direction&7;let facing=self.direction^4;let portal=exit_portal(self.origin,self.direction);
  for(rank,member)in self.members.iter_mut().enumerate(){member.animation=Animation::exiting(facing,rank,self.occupant==Some(member.id));}
  self.occupant=None;self.occupied_tick=None;portal
 }
 /// Convenience entry point when the caller has the occupied carrier anchor.
 pub fn unload_vehicle(&mut self,occupied:Point,direction:u8)->Point{self.unload(parked_origin(occupied),direction)}
 pub fn remove(&mut self,id:usize){self.members.retain(|m|m.id!=id);}
 pub fn animation(&self,id:usize)->Option<&Animation>{self.members.iter().find(|m|m.id==id).map(|m|&m.animation)}
 pub fn finished(&self)->bool{self.members.iter().all(|m|m.animation.phase==Phase::Released)}
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,ns:&[i32]){for n in ns{*h=(*h^*n as u32).wrapping_mul(16777619);}}
 #[test]fn all_entry_animation_callbacks_match_the_original_transport_handler(){
  let mut h=2166136261;
  for direction in 0..8{let mut a=Animation::approach(direction);a.arrive();for tick in 1..=24{
   let event=a.advance((500,600),||0);hash(&mut h,&[i32::from(direction),tick,if a.phase==Phase::Inside{62}else{61},i32::from(a.frame),i32::from(a.timer),i32::from(a.visible),if a.phase==Phase::Inside{10000}else{500}]);if event==Transition::Hidden{break;}
  }}
  assert_eq!(h,1655704517);
 }
 #[test]fn all_3264_exit_callbacks_and_random_release_goals_match_the_original(){
  let mut h=2166136261;
  for direction in 0..8{for rank in [0,1,5,20]{for multi in 0..2{
   let mut a=Animation::exiting(direction,rank,false);let mut rng=crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:multi!=0};
   for tick in 1..=160{
    let event=a.advance((500,600),||rng.next_u32());let(release,x,y)=if let Transition::Released{goal}=event{(true,goal.0,goal.1)}else{(false,0,0)};
    hash(&mut h,&[i32::from(direction),rank as i32,multi,tick,i32::from(a.frame),i32::from(a.timer),i32::from(a.visible),i32::from(release),x,y]);if release{break;}
   }
  }}}
  assert_eq!(h,1072479629);
 }
 #[test]fn all_224_native_unload_placements_keep_the_roster_order_and_stagger(){
  let mut h=2166136261;
  for direction in 0..8{for count in [1,6,21]{
   let ids=(0..count).collect::<Vec<_>>();let mut b=Boarding::new(80,3,(500,600),direction,&ids).unwrap();let portal=b.unload((500,600),direction);
   for(rank,m)in b.members.iter().enumerate(){hash(&mut h,&[i32::from(direction),count as i32,rank as i32,i32::from(m.animation.direction),64,i32::from(m.animation.frame),i32::from(m.animation.timer),portal.0,portal.1]);}
  }}
  assert_eq!(h,4210857405);
 }
 #[test]fn all_62_completion_cases_match_the_native_all_passengers_inside_gate(){
  let mut h=2166136261;
  for count in [1,6,21]{for inside in 0..=count{for occupied in 0..2{
   let ids=(0..count).collect::<Vec<_>>();let mut b=Boarding::new(80,3,(500,600),0,&ids).unwrap();
   for m in b.members.iter_mut().take(inside){m.animation.phase=Phase::Inside;}
   b.occupant=if occupied!=0{Some(0)}else{None};let transform=matches!(b.advance(0,(500,600),||0),Event::Occupy{..});
   hash(&mut h,&[count as i32,inside as i32,occupied,i32::from(transform)]);
  }}}
  assert_eq!(h,1012469471);
 }
 #[test]fn group_waits_for_every_live_robot_and_does_not_duplicate_occupancy(){
  let mut b=Boarding::new(80,3,(500,600),0,&[11,12,13]).unwrap();assert_eq!(b.portal(),(507,616));
  for id in [11,12]{assert!(b.arrive(id));for _ in 0..11{b.advance(id,b.portal(),||0);}}
  assert!(!b.all_inside());assert_eq!(b.advance(11,b.portal(),||0),Event::None);
  b.arrive(13);for _ in 0..11{b.advance(13,b.portal(),||0);}assert!(b.all_inside());assert_eq!(b.advance(11,b.portal(),||0),Event::Occupy{member:11});assert_eq!(b.advance(12,b.portal(),||0),Event::None);
  assert_eq!(b.unload((500,600),0),(507,615));assert!(b.members.iter().all(|m|m.animation.direction==4));assert_eq!(b.members.iter().map(|m|m.animation.timer).collect::<Vec<_>>(),vec![5,11,17]);
  assert!(Boarding::new(1,1,(0,0),0,&vec![2;22]).is_err());assert!(Boarding::new(1,1,(0,0),0,&[2,2]).is_err());
 }
 #[test]fn the_last_passenger_callback_transforms_the_squad_leader(){
  let mut b=Boarding::new(80,3,(500,600),0,&[11,12,13]).unwrap();
  for m in &mut b.members{m.animation.phase=Phase::Inside;m.animation.visible=false;}
  assert_eq!(b.advance(13,b.portal(),||0),Event::Occupy{member:11});assert_eq!(b.occupant,Some(11));
 }
 #[test]fn clockwise_exit_search_and_native_health_serialization_are_bounded(){
  let mut tried=Vec::new();assert_eq!(choose_direction(6,|d|{tried.push(d);d==1}),Some(1));assert_eq!(tried,vec![6,7,0,1]);assert_eq!(choose_direction(2,|_|false),None);
  for(hp,parked)in [(0,25),(1,25),(24,25),(25,25),(49,25),(70,50),(200,200),(299,275),(300,300)]{assert_eq!(parked_hull(hp),parked);}
 }
 #[test]fn all_24576_native_exit_footprints_match_world_masks_probe_width_and_terrain_high_bits(){
  let mut h=2166136261u32;let mut count=0;
  for world in 0..2{for direction in 0..8{for(probe,offset)in[0i16,1,128,129,2,130].into_iter().enumerate(){for code in 0..128u8{for high in 0..2{
   let cell=4128+FOOTPRINT_CELL_OFFSETS[direction]+offset;let blocked=footprint_blocked(4128,direction as u8,world==1,|p|if p==cell{code|(high<<7)}else{10});for n in[world,direction as i32,probe as i32,i32::from(code),i32::from(high),i32::from(blocked)]{h=(h^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}
  assert_eq!((count,h),(24576,3304508549),"complete457b60 after real433b30 world initialization");
 }
 #[test]fn all_2048_complete_native_exit_searches_match_first_object_and_one_shot_query_controls(){
  use crate::original_world_contact as world;use std::cell::Cell;
  let mut h=2166136261u32;let mut count=0;
  for volcanic in 0..2{for scenario in 0..128usize{for start in 0..8u8{
   let(x,y)=(500+(scenario&15)as i16,600+((scenario>>3)&15)as i16);let base=(((y as u16&0xfff0)*8).wrapping_add((x>>4)as u16))as usize;let mut map=vec![10u8;16384];let pattern=scenario%8;
   if pattern!=0{for row in -3..=4{for col in -3..=4{let n=(scenario as i32*17+row*11+col*7)&31;if n<pattern as i32{map[((base as i32+row*128+col)&16383)as usize]=[9,31,32,40,38,62,63][n as usize%7];}}}}
   let deltas=[-25i16,-24,-16,-8,-1,0,1,8,16,24,25];let objects=(0..4).map(|i|world::Object{position:[x+deltas[(scenario+i*3)%11],y+deltas[(scenario/3+i*5)%11]],flags13:[0,16,64,80][(scenario+i)%4],metadata_flags18:(scenario*13+i*17)as u8,metadata_extent20:(scenario*43+i*7)as i8}).collect::<Vec<_>>();let metadata=Cell::new((scenario>>6)&1!=0);let object_calls=Cell::new(0usize);
   let selected=choose_exit([x,y],start,volcanic==1,|p|map[p as u16 as usize],|p|{object_calls.set(object_calls.get()+1);let hit=world::first_object(&objects,p,(scenario>>5)&1!=0,metadata.get()).is_some();metadata.set(false);hit});let terrain_calls=selected.map_or(8,|d|i32::from(d.wrapping_sub(start)&7)+1);
   for n in[volcanic,scenario as i32,i32::from(start),i32::from(selected.is_some()),i32::from(selected.unwrap_or(start)),terrain_calls,object_calls.get()as i32,i32::from(metadata.get())]{h=(h^n as u32).wrapping_mul(16777619);}count+=1;
  }}}
  assert_eq!((count,h),(2048,3874696961),"complete59830 with real457b60 and41c600; tail59720 is outside this availability oracle");
 }
 #[test]fn all_117_native_parking_conversions_keep_apc_anchors_and_quantize_hull_health(){
  let mut h=2166136261;
  for x in [492,499,500,503,504,505,510,511,512,518,519,520,527]{for hp in [0,1,24,25,49,70,200,299,300]{
   let p=parked_origin((x,600));hash(&mut h,&[x,600,i32::from(hp),p.0,p.1,i32::from(parked_hull(hp))]);
  }}
  assert_eq!(h,2693706809);assert_eq!(occupied_origin((500,600)),(506,606));
 }
}
