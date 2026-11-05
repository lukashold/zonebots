//! Game bindings for the native CPU artillery completion event. The five
//! placement globals are player selection state; CPU40c0a0 only borrows slot0.
use super::{Game,original_factory_placement as native,original_combat};
use std::cell::Cell;
#[derive(Clone,Copy,Debug)]struct Pending{product:i8,sector:Option<usize>}
pub struct State{pending:[Cell<Pending>;5],view:Cell<native::View>}
impl Default for State{fn default()->Self{Self{pending:std::array::from_fn(|i|Cell::new(Pending{product:if i<4{-1}else{0},sector:None})),view:Cell::new(native::View::default())}}}
impl State{
 ///460430 writes only the four player bytes at5ba0a4; pointer/view/scratch
 ///fields are retained until their own writers run (cold processBSS iszero).
 pub fn clear_for_mission(&mut self){for p in self.pending.iter().take(4){let mut old=p.get();old.product=-1;p.set(old);}}
 pub fn pending_sector_count(&self,sector:usize)->i16{self.pending.iter().filter(|p|{let p=p.get();p.product>=0&&p.sector==Some(sector)}).count()as i16}
 pub fn pending_team_count(&self,team:usize)->i16{self.pending.get(team).is_some_and(|p|p.get().product>=0)as i16}
}
#[cfg(test)]mod tests{
 use super::*;
 fn blank()->Game{let mut g=Game::new(0,1,719);g.map.original=true;g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters.clear();g}
 fn field()->Game{
  let mut g=blank();let template=Game::new(0,1,731).buildings[0].clone();g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.native_road_graph=Some(super::super::original_route::RoadGraph{regions:vec![]});g.map.native_level_record=0;g.original_ai=Some(super::super::original_ai::Clock::new(0));g.ai_teams=vec![1];
  g.sectors=vec![super::super::Sector{id:0,x:144.,y:272.,owner:1,home:true,counts_for_production:true},super::super::Sector{id:1,x:144.,y:1800.,owner:0,home:true,counts_for_production:true}];
  for(id,kind,team,sector,base)in[(0,"robot",1,0,[200,200]),(1,"fort",1,0,[100,100]),(2,"fort",0,1,[100,900])]{let mut b=template.clone();b.id=id;b.kind=kind.into();b.team=team;b.sector=sector;b.x=f64::from(base[0])*2.;b.y=f64::from(base[1])*2.;b.raw_base=(kind=="fort").then_some((f64::from(base[0]),f64::from(base[1])));b.hp=1500.;b.max_hp=1500.;b.product=Some("grunt".into());g.buildings.push(b);}
  let v=super::super::json!({"nativeAi":{"decisionControl":0,"reservePercent":25,"populationWeight":1,"initialSectorTasks":[]},"sectors":[{"id":0,"rect":[0,0,128,32],"nativeSlot":0,"rawFlagTile":[4,8],"nativeFlags":0,"nativePopulation":0,"owner":1},{"id":1,"rect":[0,32,128,128],"nativeSlot":1,"rawFlagTile":[4,56],"nativeFlags":0,"nativePopulation":0,"owner":0}],"buildings":[]});g.native_strategy=super::super::original_strategy::State::from_mission(&v);g.import_native_world(&super::super::Value::Null);g
 }
 #[test]fn completed_cpu_gun_uses_first_mission_socket_and_restores_the_human_pending_selection(){
  let mut g=field();g.record_original_gun_selection(0,Some("howitzer"),Some(1));assert_eq!(g.complete_original_cpu_gun(0,"gatling"),Some(true));let u=g.units.last().unwrap();assert_eq!((u.x,u.y),(226.,194.));assert_eq!(u.fort_id,Some(1));assert_eq!((u.native_class7c,u.native_phase35),(5,51));assert_eq!(g.native_pending_sector_guns(1),1);assert!(g.buildings[0].ready_guns.is_empty());g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.formation_layout(1,g.units[0].squad),Some(0));
 }
 #[test]fn native_cpu_completion_on_a_map_without_a_valid_position_is_consumed_without_ready_stock(){
  let mut g=field();g.buildings[0].sector=1;g.buildings[0].product=Some("cannon".into());g.buildings[0].remaining_work=Some(1.);g.buildings[0].completion_ticks=0;g.sectors[1].owner=1;g.buildings[1].product=None;g.buildings[2].product=None;
  assert_eq!(g.complete_original_cpu_gun(0,"cannon"),Some(false));g.update_original_production(1./17.);assert!(g.units.is_empty());assert!(g.buildings[0].ready_guns.is_empty());assert_eq!(g.buildings[0].completion_ticks,68);
 }
 #[test]fn cpu_gun_creation_matches_64_real_native_allocator_and_constructor_records(){
  let mut hash=2166136261u32;for scenario in 0..64{let mut g=blank();let product=[0,8,10,14][scenario%4];let kind=super::super::original_stats::PRODUCTS[product].0;let index=g.spawn_original_cpu_gun(kind,[100+scenario as i16*17,200+scenario as i16*13],None).unwrap();g.maintain_native_group_rosters();let u=&g.units[index];let group=g.native_group_rosters.group_slot(1,u.squad).unwrap();let row=&g.native_group_rosters.pool[group].roster;
   for n in[scenario as u32,product as u32,u.native_slot.unwrap()as u32,(u.x/2.)as i16 as u32,(u.y/2.)as i16 as u32,u32::from(u.native_class7c),u32::from(u.native_phase35),group as u32,u32::from(row[8]),u32::from(row[60])]{hash=(hash^n).wrapping_mul(16777619);}
  }assert_eq!(hash,1443340261,"ZNativeFactoryGunCreationOracle actual17610 and all real group/unit allocators");
 }
 #[test]fn direct_cpu_gun_placement_preserves_allocator_mask_fields_in_a_reused_roster(){
  let mut g=blank();g.native_group_rosters.pool[120].roster[8]=3;g.native_group_rosters.pool[120].roster[60]=2;g.spawn_original_cpu_gun("gatling",[500,500],None).unwrap();g.maintain_native_group_rosters();assert_eq!((g.native_group_rosters.pool[120].roster[8],g.native_group_rosters.pool[120].roster[60]),(3,2));
 }
 #[test]fn placement_state_counts_selections_and_preserves_native_unwritten_globals_at_mission_reset(){
  let mut g=blank();g.record_original_gun_selection(0,Some("cannon"),Some(7));g.record_original_gun_selection(1,Some("missile"),Some(7));assert_eq!(g.native_pending_sector_guns(7),2);g.record_native_gun_placement_view([100,200],[32,48]);g.record_original_gun_selection(0,None,None);assert_eq!(g.native_pending_sector_guns(7),1);g.native_factory_placement.clear_for_mission();assert_eq!(g.native_pending_sector_guns(7),0);assert_eq!(g.native_factory_placement.pending[0].get().sector,Some(7));assert_eq!(g.native_factory_placement.view.get().camera,[32,48]);
 }
}
fn product(kind:&str)->Option<u8>{super::original_stats::PRODUCTS.iter().position(|p|p.0==kind).and_then(|i|u8::try_from(i).ok())}
impl Game{
 ///Actual player selection/cancellation event. A mere ready stock item does
 ///not create a5ba0a4 placement entry. Source45e940 clears only the byte.
 pub(crate) fn record_original_gun_selection(&self,team:usize,kind:Option<&str>,sector:Option<usize>){
  if team>=4{return;}let p=&self.native_factory_placement.pending[team];let old=p.get();p.set(Pending{product:kind.and_then(product).map(|p|p as i8).unwrap_or(-1),sector:if kind.is_some(){sector}else{old.sector}});
 }
 ///12f60's last preview globals. The base is the human d58 fort, and camera
 ///is622cb8, not a replacement CPU-fort rectangle. No preview meanscoldzero.
 pub(crate) fn record_native_gun_placement_view(&self,fort_base:[i16;2],camera:[i16;2]){self.native_factory_placement.view.set(native::View{fort_bounds:fort_base,camera});}
 pub(crate) fn native_pending_sector_guns(&self,sector:usize)->i16{self.native_factory_placement.pending_sector_count(sector)}
 pub(crate) fn native_sector_gun_count(&self,team:i32,sector:usize)->Option<i16>{
  let state=self.native_strategy.as_ref()?;let live=self.units.iter().filter(|u|u.native_slot.is_some()&&u.team==team&&original_combat::alive(true,u.hp)&&u.native_class7c==2&&state.sector_at(u.x,u.y)==Some(sector)).count()as i16;
  let neutral=self.native_ai_world_objects()?.iter().filter(|o|o.position[0]>=0&&o.sector==Some(sector)&&matches!(o.definition,Some(5|21|22|23))).count()as i16;Some(live.wrapping_add(neutral).wrapping_add(self.native_pending_sector_guns(sector)))
 }
 ///40c0a0→176b0/17610. Some(false) means the original tried and discarded
 ///the product: CPU completion never falls back to the human ready-stock queue.
 pub(crate) fn complete_original_cpu_gun(&mut self,factory:usize,kind:&str)->Option<bool>{
  if !self.map.original||!super::gun_placement::is_gun(kind){return None;}
  let b=self.buildings.iter().find(|b|b.id==factory)?.clone();if b.team!=1||!self.ai_teams.contains(&1){return None;}
  let sector=self.native_factory_sector(b.sector)?;let product=product(kind)?;let skill=self.original_ai.as_ref()?.control;let level=self.native_factory_level_record_index()?;
  let buildings=self.native_world_buildings()?;let objects=self.native_world_scan_objects().iter().map(|o|native::ObjectBox{position:o.position,extent:[o.extent20,o.extent21]}).collect::<Vec<_>>();
  let guns=self.units.iter().filter(|u|u.native_slot.is_some()&&u.team>=0&&original_combat::alive(true,u.hp)&&u.native_class7c==2).map(|u|[(u.x/2.)as i16,(u.y/2.)as i16]).collect::<Vec<_>>();
  let sectors=self.sectors.iter().filter_map(|s|self.native_factory_sector(s.id)).collect::<Vec<_>>();
  let lookup=|p:[i16;2]|{let(x,y)=(p[0]>>4,p[1]>>4);[(0,0),(1,1)].into_iter().find_map(|(dx,dy)|sectors.iter().find(|s|s.valid&&s.rect[0]<=x+dx&&x+dx<s.rect[2]&&s.rect[1]<=y+dy&&y+dy<s.rect[3]).map(|s|s.id))};
  let flags=sectors.iter().filter(|s|s.flags&3==0).map(|s|[i16::from(s.flag[0]as i8)<<4,i16::from(s.flag[1]as i8)<<4]).collect::<Vec<_>>();
  let fort=self.buildings.iter().filter(|b|b.kind=="fort").filter_map(|b|b.raw_base.map(|p|(b.id,[p.0 as i16,p.1 as i16]))).min_by_key(|(_,p)|(p[1],p[0]));
  let fort_attempt=if sector.flags&1!=0{let(_,base)=fort?;Some((base,native::socket_order(level)?))}else{None};
  let mut terrain=if self.map.native_terrain_plane.len()==128*128{self.map.native_terrain_plane.clone()}else if self.map.width==128&&self.map.height==128&&self.map.terrain_codes.len()==128*128{self.map.terrain_codes.clone()}else{return None;};
  //The visible map may be smaller; its live overlays take precedence over
  //the128x128 padding, matching the route adapter's native terrain accessor.
  for y in 0..self.map.height{for x in 0..self.map.width{if let Some(&code)=self.map.terrain_codes.get(y*self.map.width+x){terrain[y*128+x]=code;}}}
  let old=self.native_factory_placement.pending[0].get();self.native_factory_placement.pending[0].set(Pending{product:product as i8,sector:old.sector});let view=self.native_factory_placement.view.get();
  let placed=native::select(&mut terrain,sector.rect,fort_attempt,skill,|p|native::gun_occupied(p,&guns),|p,terrain|native::validate(p,lookup([p[0]&!15,p[1]&!15])==Some(sector.id),product,view,&buildings,&guns,&objects,&flags,terrain),||self.original_rng.next_u32());
  for y in 0..self.map.height{for x in 0..self.map.width{if let Some(code)=self.map.terrain_codes.get_mut(y*self.map.width+x){*code=terrain[y*128+x];}}}self.map.native_terrain_plane=terrain;
  //40c0a0 saves/restores slot0 AND its61b650 sector even on failedplacement.
  self.native_factory_placement.pending[0].set(old);
  let Some(placed)=placed else{return Some(false);};let unit=self.spawn_original_cpu_gun(kind,placed.position,if placed.socket.is_some(){fort.map(|(id,_)|id)}else{None})?;let anchor=[(self.units[unit].x/2.)as i16,(self.units[unit].y/2.)as i16];
  self.events.push(super::json!({"kind":"placed","team":1,"type":kind,"factory":factory,"x":f64::from(anchor[0])*2.,"y":f64::from(anchor[1])*2.}));Some(true)
 }
 fn spawn_original_cpu_gun(&mut self,kind:&str,position:[i16;2],fort:Option<usize>)->Option<usize>{
  let profile=super::native_ai_scoring::default_input(kind)?.profile;let(class,phase)=native::initial_gun_state(profile)?;let anchor=native::actor_anchor(position,profile);let squad=self.next_id;let index=self.units.len();
  self.spawn(kind,1,f64::from(anchor[0])*2.,f64::from(anchor[1])*2.,squad,None);let unit=self.units.get_mut(index).filter(|u|u.squad==squad)?;unit.fort_id=fort;unit.cooldown=0.;unit.native_class7c=class;unit.native_phase35=phase;
  //4a69f3=1 bypasses formation initialization: preserve the allocator's stale
  //mask fields (fresh native pool0/0), unlike an ordinary factoryexitFF mask.
  self.native_group_rosters.record_direct_placement_group(1,squad);
  if let Some(gid)=product(kind){self.native_group_rosters.record_constructed_descriptor(1,squad,None,usize::from(gid));}
  Some(index)
 }
}
