//! 461d10's post-AI same-team parked-vehicle separation. This is distinct
//! from terrain collision, flag capture and the pre-UNIT vehicle snapshot.
use crate::original_path_contact::Vehicle;
const OFFSETS:[[i16;2];8]=[[-32,-32],[0,-32],[32,-32],[-32,0],[32,0],[-32,32],[0,32],[32,32]];
const TERRAIN:[u8;128]=[0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,255,255,255,255,0,0,0,0,255,255,0,255,255,255,0,255,255,0,0,0,0,255,255,255,255,0,255,255,0,0,255,255,255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,0,0,0,0,112,114,101,115,101,116,63,63,46,119,97,108,0,0,0,0,111,98,106,101,99,116,115,49,46,100,97,116,0,0,0,0,98,117,105,108,100,105,110,103,46,100,97,116,0,0,0,0,0,0,0,0,3,0,2,0,2,0,255,255];
fn near(a:[i16;2],b:[i16;2])->bool{let x=i32::from(a[0])-i32::from(b[0]);let y=i32::from(a[1])-i32::from(b[1]);x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y))<64}
pub trait World{
 type Error;
 fn parked(&mut self,slot:u8)->Result<bool,Self::Error>;
 fn workshop(&mut self,point:[i16;2])->Result<bool,Self::Error>;
 fn position(&mut self,slot:u8)->Result<[i16;2],Self::Error>;
 fn terrain(&mut self,cell:i32)->Result<u8,Self::Error>;
 fn random(&mut self)->u32;
 fn line(&mut self,slot:u8,point:[i16;2])->Result<(),Self::Error>;
}
/// The first same-team neighbour alone is considered. 461bb0 deliberately
/// tries all eight candidates, so its last accepted line remains installed.
pub fn separate<W:World>(records:&[Vehicle],count:i8,w:&mut W)->Result<(),W::Error>{
 if count<=0{return Ok(());}let records=&records[..usize::from(count as u8).min(records.len())];
 for actor in records{
  if !w.parked(actor.slot)?{continue;}
  let Some(other)=records.iter().find(|r|r.slot!=actor.slot&&(r.owner^actor.owner)&3==0&&near(r.position,actor.position))else{continue;};
  if !w.parked(other.slot)?||w.workshop(other.position)?{continue;}
  let first=w.random()as usize&7;
  for n in 0..8{
   let origin=w.position(other.slot)?;let d=OFFSETS[(first+n)&7];let candidate=[origin[0].wrapping_add(d[0]),origin[1].wrapping_add(d[1])];
   if records.iter().any(|r|near(r.position,candidate)){continue;}
   let x=i32::from(origin[0])+i32::from(d[0]);let y=i32::from(origin[1])+i32::from(d[1]);
   let cell=((((y*8+64)&!127)+((x+8)>>4))as i16)as i32;
   let mut clear=true;for delta in[0,1,128,129]{if TERRAIN[usize::from(w.terrain(cell+delta)?&127)]!=0{clear=false;break;}}
   if clear{w.line(other.slot,[x as i16,y as i16])?;}
  }
 }
 Ok(())
}
struct GameWorld<'a>(&'a mut crate::Game);
impl World for GameWorld<'_>{
 type Error=crate::original_path_contact::game::Boundary;
 fn parked(&mut self,slot:u8)->Result<bool,Self::Error>{
  use crate::original_path_contact::game::Boundary;
  let u=self.0.units.iter().rfind(|u|u.native_slot==Some(usize::from(slot))).ok_or(Boundary::MissingUnit(usize::from(slot)))?;
  let m=self.0.native_group_rosters.members[usize::from(slot)].as_ref().ok_or(Boundary::MissingMember)?;
  let group=self.0.native_group_rosters.pool.get(usize::from(m.group)).ok_or(Boundary::MissingGroup(i16::from(m.group)))?;
  let order=u.native_hardware_slot.and_then(|s|self.0.native_hardware_pool.get(s)).map_or(self.0.native_hardware_dummy,|w|w.order);
  Ok(group.roster[58]==0&&order.field17&1==0)
 }
 fn workshop(&mut self,p:[i16;2])->Result<bool,Self::Error>{
  let records=self.0.native_world_buildings().ok_or(Self::Error::MissingHardwareProfile)?;
  Ok(crate::original_world_contact::first_building(&records,p,false,false).is_some_and(|(slot,_)|records[slot].kind==0))
 }
 fn position(&mut self,slot:u8)->Result<[i16;2],Self::Error>{self.0.native_raw_unit_position(usize::from(slot)).ok_or(Self::Error::MissingUnit(usize::from(slot)))}
 fn terrain(&mut self,cell:i32)->Result<u8,Self::Error>{
  if !(0..16384).contains(&cell){return Err(Self::Error::MissingTerrain(cell as i16));}let cell=cell as usize;let(x,y)=(cell&127,cell>>7);
  if x<self.0.map.width&&y<self.0.map.height{self.0.map.terrain_codes.get(y*self.0.map.width+x)}else{self.0.map.native_terrain_plane.get(cell)}.copied().ok_or(Self::Error::MissingTerrain(cell as i16))
 }
 fn random(&mut self)->u32{self.0.native_random()}
 fn line(&mut self,slot:u8,p:[i16;2])->Result<(),Self::Error>{let i=self.0.units.iter().rposition(|u|u.native_slot==Some(usize::from(slot))).ok_or(Self::Error::MissingUnit(usize::from(slot)))?;self.0.start_native_walking_direct(i,p);Ok(())}
}
impl crate::Game{
 pub(crate) fn update_native_vehicle_spacing(&mut self){
  if !self.map.original||self.map.native_road_graph.is_none(){return;}
  self.prepare_native_path_contacts();let records=self.native_path_contact.vehicles.clone();let count=self.native_path_contact.count;
  if let Err(e)=separate(&records,count,&mut GameWorld(self)){self.native_route_boundary=Some(e);}
 }
}
#[cfg(test)]mod tests{
 use super::*;use crate::original_path_contact::{Actor,State};
 fn mix(h:&mut u32,n:i64){*h=(*h^n as u32).wrapping_mul(16777619);}
 struct Env{scenario:usize,positions:Vec<[i16;2]>,randoms:u32,calls:u32,lines:u32}
 impl World for Env{type Error=();
  fn parked(&mut self,s:u8)->Result<bool,()>{let n=self.scenario+usize::from(s);Ok(n%7!=0&&n%11!=0)}
  fn workshop(&mut self,p:[i16;2])->Result<bool,()>{let s=self.scenario;let b=crate::original_world_contact::Building{position:[480,480],kind:if s%3==0{0}else{1},hp:if s%5==0{0}else{1000}};Ok(crate::original_world_contact::building_contact(b,p,true,false).is_some()&&b.kind==0)}
  fn position(&mut self,s:u8)->Result<[i16;2],()>{Ok(self.positions[usize::from(s)])}
  fn terrain(&mut self,_:i32)->Result<u8,()>{Ok((self.scenario/8%128)as u8|(self.scenario%2*128)as u8)}
  fn random(&mut self)->u32{let r=self.scenario as u32*7919^self.randoms*2027^0x7731a5c9;self.randoms+=1;r}
  fn line(&mut self,s:u8,p:[i16;2])->Result<(),()>{mix(&mut self.lines,i64::from(s));mix(&mut self.lines,i64::from(p[0]));mix(&mut self.lines,i64::from(p[1]));self.calls+=1;Ok(())}
 }
 #[test]fn post_ai_spacing_matches_complete_native_controller_and_all_eight_candidates(){
  let native=std::env::var("NATIVE_VEHICLE_SPACING_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|r|r.split(',').map(|n|n.parse::<i64>().unwrap()).collect::<Vec<_>>()).collect::<Vec<_>>());let mut total=2166136261u32;
  for s in 0..1024usize{
   let n=if s%32==0{240}else{8};let offsets=[[0,0],[0,0],[7,0],[8,0],[0,7],[32,0],[64,0],[-32,0]];let positions=(0..n).map(|k|if n==240{[100+(k%20)as i16*64,100+(k/20)as i16*64]}else{[500+offsets[k][0],500+offsets[k][1]]}).collect::<Vec<_>>();let mut state=State::default();
   state.prepare((0..n).map(|k|{let owner=if k<2{s%4}else{(s+k)%4};let mut flags=owner as u8|4;if n!=240&&(s+k)%13==0{flags&=!4;}Actor{slot:k as u8,position:positions[k],flags31:flags,class:if n!=240&&(s+k)%17==0{0}else{1},flags71:if n!=240&&(s+k)%19==0{2}else{0},..Default::default()}}));
   let mut cache=2166136261u32;for r in &state.vehicles{for v in[i64::from(r.slot),i64::from(r.position[0]),i64::from(r.position[1]),i64::from(r.owner)]{mix(&mut cache,v);}}
   let mut w=Env{scenario:s,positions,randoms:0,calls:0,lines:2166136261};separate(&state.vehicles,state.count,&mut w).unwrap();let row=[s as i64,i64::from(state.count),i64::from(cache),i64::from(w.randoms),i64::from(w.calls),i64::from(w.lines)];if let Some(rows)=&native{assert_eq!(row.as_slice(),rows[s].as_slice(),"actual461d10 scenario{s}");}for v in row{mix(&mut total,v);}
  }
  if let Some(rows)=native{assert_eq!(rows.len(),1024);}assert_eq!(total,4237801726,"full461d10 plus cached membership and all eight61bb0 candidates; RNG and525a0 installation are explicit boundaries");
 }
 #[test]fn two_parked_tanks_get_the_source_line_without_replacing_their_primary_paths(){
  use crate::{Game,json,original_route::RoadGraph};let mut g=Game::new(0,1,13);g.map.original=true;g.map.native_road_graph=Some(RoadGraph{regions:vec![]});g.units.clear();g.buildings.clear();g.objects.clear();g.walls.clear();g.crates.clear();g.sectors.clear();g.native_group_rosters.clear();g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.native_terrain_plane=vec![0;16384];g.spawn("medium",0,1000.,1000.,1,None);g.spawn("medium",0,1000.,1000.,2,None);g.maintain_native_group_rosters();g.import_native_world(&json!({}));let mut links=Vec::new();for i in 0..2{let slot=g.units[i].native_slot.unwrap();let p=g.native_routes.get_mut().pool.allocate_goal([777,888]).unwrap();g.native_routes.get_mut().pool.links[slot].primary=p;links.push((slot,p));}let rng=g.original_rng.words;g.update_native_vehicle_spacing();assert_ne!(g.original_rng.words,rng);assert!(g.native_route_boundary.is_none(),"{:?}",g.native_route_boundary);for(i,(slot,p))in links.into_iter().enumerate(){assert_eq!(g.native_routes.borrow().pool.links[slot].primary,p);let u=g.units.iter().find(|u|u.native_slot==Some(slot)).unwrap();assert_eq!((u.x,u.y),(1000.,1000.));assert_eq!(u.original_motion.is_some(),i==1,"525a0 sets W17:1 for the second tank; its later outer-record parked gate now fails");assert_eq!(u.native_phase35,9);let w=u.native_hardware_slot.unwrap();assert_eq!(g.native_hardware_pool[w].order.field17&1,u8::from(i==1));}
 }
}
