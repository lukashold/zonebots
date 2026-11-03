//!44f0b0/44ef40 grenade contact. Raw OBJECT order, signed16 distances and
//!signed group count are independent of selectable browser entity IDs.
use super::{Game,native_point};
#[derive(Clone,Copy)]pub struct Object{pub position:[i16;2],pub flags13:u8,pub definition:u16}
pub fn select(objects:&[Object],position:[i16;2],class:u8)->Option<usize>{
 let (p,r)=if class==1{(position.map(|v|v.wrapping_sub(8)),24)}else{(position,16)};
 objects.iter().take(200).position(|o|o.position[0]>=0&&o.flags13&128!=0
  &&(0..2).all(|a|p[a].wrapping_sub(o.position[a]).wrapping_abs()<r)
  &&matches!(o.definition,9|10))
}
///44ef40 replaces each occupied roster slot's byte. The first occupied slot
///gets the remainder; the division uses count12, not a recounted roster.
pub fn distribute(definition:u16,count:i8,roster:&[u8;21])->Option<Vec<(u8,u8)>>{
 if count==0{return None;}let total:i16=if definition==10{20}else{10};
 let q=total/i16::from(count);let mut rem=total-q*i16::from(count);let mut out=Vec::new();
 for &slot in roster{if slot==255{continue;}let n=(q as u8).wrapping_add(rem as u8);rem=0;out.push((slot,if (n as i8)<0{127}else{n}));}Some(out)
}
impl Game{
 ///4f1e0 dispatches44f0b0 for terrain36 before the phase handler. Graphless
 ///synthetic missions keep their existing8-grenade crate contract explicitly.
 pub(crate) fn native_grenade_contact(&mut self,index:usize)->bool{
  let Some(u)=self.units.get(index).cloned()else{return false;};
  if u.team<0||!super::super::original_combat::alive(self.map.original,u.hp)||super::super::workshops::hidden(&u){return false;}
  if self.map.native_road_graph.is_none(){
   if super::super::stats(&u.kind).cls!=0{return false;}let mut took=false;
   for c in &mut self.crates{if !c.taken&&(u.x-c.x).hypot(u.y-c.y)<22.{c.taken=true;self.units[index].grenades+=8;took=true;}}
   if took{self.events.push(super::json!({"kind":"grenades","team":u.team}));}return took;
  }
  if !self.map.original{return false;}let p=native_point(u.x,u.y);if p[0]==10000{return false;}
  let center=[p[0].wrapping_add(8),p[1].wrapping_add(8)];let cell=Self::world_cell(center);
  if cell<0||cell as usize>0x4000||self.world_code(cell)&127!=36{return false;}
  let Some(member)=u.native_slot.and_then(|s|self.native_group_rosters.members.get_mut(s)).and_then(Option::as_mut)else{return false;};member.flags69|=4;let group=usize::from(member.group);
  let Some(group)=self.native_group_rosters.pool.get(group)else{return false;};let count=group.roster[18]as i8;let roster:[u8;21]=group.roster[19..40].try_into().unwrap();
  let grenades=roster.iter().filter(|&&s|s!=255).map(|&slot|self.units.iter().find(|v|v.native_slot==Some(usize::from(slot))).map_or(0,|v|v.grenades as i8)).collect::<Vec<_>>();
  if !matches!(u.native_class7c,0|15|16)||super::super::native_ai_scoring::group_can_demolish(&grenades){return false;}
  let Some(state)=&self.native_world else{return false;};let objects=state.objects.iter().map(|o|o.as_ref().filter(|o|o.active).map(|o|Object{position:o.record.position,flags13:o.record.flags13,definition:o.kind}).unwrap_or(Object{position:[-1,0],flags13:0,definition:0})).collect::<Vec<_>>();
  let Some(slot)=select(&objects,p,u.native_class7c)else{return false;};let definition=objects[slot].definition;let Some(writes)=distribute(definition,count,&roster)else{return false;};
  for(slot,n)in writes{if let Some(v)=self.units.iter_mut().find(|v|v.native_slot==Some(usize::from(slot))){v.grenades=usize::from(n);}}
  let Some(o)=self.native_world.as_mut().and_then(|s|s.objects[slot].as_mut())else{return false;};let point=o.record.position;let saved=o.saved[0];let pickup=o.pickup;let render=o.render;o.active=false;o.record.position[0]=-1;
  if let Some(c)=pickup.and_then(|i|self.crates.get_mut(i)){c.taken=true;}if let Some(raw)=render.and_then(|i|self.objects.get_mut(i)){raw["nativeActive"]=super::json!(false);}
  self.set_world_code(Self::world_cell(point),saved);self.map.revision=self.map.revision.wrapping_add(1);self.events.push(super::json!({"kind":"grenades","team":u.team}));true
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn field(graph:bool)->Game{
  use super::super::super::{original_group_game,original_route};let mut g=Game::new(0,1,8751);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.original_tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.territory=vec![0;4096];g.map.bridge_base=vec![-1;4096];g.map.bridge_hp.clear();g.units.clear();g.objects.clear();g.crates.clear();g.buildings.clear();g.sectors.clear();g.walls.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters=original_group_game::State::default();g.next_id=1;g.events.clear();g.ai_teams.clear();g.map.native_road_graph=graph.then(||original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});g
 }
 #[test]fn authored_pickup_replaces_the_real_roster_inventory_and_retires_only_its_object(){
  let mut g=field(true);for _ in 0..3{g.spawn("grunt",0,384.,384.,1,None);}g.maintain_native_group_rosters();g.crates.push(super::super::super::Crate{x:384.,y:384.,taken:false});g.objects.push(super::super::json!({"type":10,"x":384,"y":384,"nativeSlot":0,"nativeHp":0}));g.import_native_world(&serde_json::Value::Null);let words=g.original_rng.words;
  assert_eq!(g.map.terrain_codes[12*64+12],36);assert!(g.native_grenade_contact(0));assert_eq!(g.units.iter().map(|u|u.grenades).collect::<Vec<_>>(),vec![8,6,6]);assert!(g.crates[0].taken);assert_eq!(g.objects[0]["nativeActive"],false);assert_eq!(g.map.terrain_codes[12*64+12],0);assert_eq!(g.original_rng.words,words);assert!(!g.native_grenade_contact(1));
 }
 #[test]fn any_existing_group_grenade_blocks_an_authored_pickup(){
  let mut g=field(true);for _ in 0..2{g.spawn("grunt",0,384.,384.,1,None);}g.maintain_native_group_rosters();g.units[1].grenades=1;g.crates.push(super::super::super::Crate{x:384.,y:384.,taken:false});g.objects.push(super::super::json!({"type":10,"x":384,"y":384,"nativeSlot":0,"nativeHp":0}));g.import_native_world(&serde_json::Value::Null);assert!(!g.native_grenade_contact(0));assert!(!g.crates[0].taken);assert_eq!(g.units.iter().map(|u|u.grenades).collect::<Vec<_>>(),vec![0,1]);
 }
 #[test]fn graphless_compatibility_pickup_keeps_the_eight_grenade_contract(){
  for kind in ["grunt","psycho","sniper","pyro","laser"]{let mut g=field(false);g.spawn(kind,0,384.,384.,1,None);g.crates.push(super::super::super::Crate{x:384.,y:384.,taken:false});g.step(0.1);assert_eq!(g.units[0].grenades,8,"{kind} must pick up before the native idle early return");assert!(g.crates[0].taken);assert!(!g.native_grenade_contact(0));}
 }
 #[test]fn strict_axis_contact_and_group_distribution_keep_native_boundaries(){
  let mut o=[Object{position:[335,335],flags13:128,definition:10}];assert_eq!(select(&o,[320,320],0),Some(0));o[0].position[0]=336;assert_eq!(select(&o,[320,320],0),None);
  let mut roster=[255;21];roster[0]=7;roster[3]=8;roster[7]=9;assert_eq!(distribute(10,3,&roster),Some(vec![(7,8),(8,6),(9,6)]));assert_eq!(distribute(9,3,&roster),Some(vec![(7,4),(8,3),(9,3)]));assert_eq!(distribute(10,0,&roster),None);
 }
 #[test]fn native_selector_and_distribution_match_2048_executed_original_calls(){
  let(cls,delta)=([0,15,16,3,1,2,255,18],[0i16,15,16,-15,-16,23,24,-32768]);let mut all=0x811c9dc5u32;
  for s in 0..2048{let n=1+(s/64)%21;let missing=s%4;let class=cls[s%8];let mut roster=[255;21];let mut values=[0u8;21];for i in 0..21{if i<n{roster[i]=i as u8;}values[i]=if missing==1&&i==0{1}else if missing==2&&i<2{128}else{0};}
   let objects=[Object{position:[320i16.wrapping_add(delta[s/8%8]),320],flags13:if s&128!=0{0}else{128},definition:(9+s/512%2)as u16},Object{position:[320,320],flags13:128,definition:12},Object{position:[320,320],flags13:128,definition:10}];let mut xs=objects.map(|o|o.position[0]);let mut terrain=Vec::new();
   if let Some(slot)=select(&objects,[320,320],class){if matches!(class,0|15|16)&&super::super::super::native_ai_scoring::group_grenade_sum(&values[..n].iter().map(|&v|v as i8).collect::<Vec<_>>())as u8==0{for(slot,n)in distribute(objects[slot].definition,n as i8,&roster).unwrap(){values[usize::from(slot)]=n;}let p=objects[slot].position;terrain.push((((p[1]as u16&0xfff0).wrapping_mul(8)).wrapping_add((p[0]>>4)as u16)as usize,85+slot));xs[slot]=-1;}}
   let mut h=0x811c9dc5u32;for n in values{h=(h^u32::from(n)).wrapping_mul(16777619);}for x in xs{h=(h^u32::from(x as u16)).wrapping_mul(16777619);}for(cell,v)in terrain{h=(h^cell as u32).wrapping_mul(16777619);h=(h^v as u32).wrapping_mul(16777619);}h=(h^21).wrapping_mul(16777619);for byte in h.to_le_bytes(){all=(all^u32::from(byte)).wrapping_mul(16777619);}
  }assert_eq!(all,2128449109);
 }
}
