//! Ownership adapter for the verified producer exit orders. Unit coordinates
//! remain native anchors scaled by two; each new robot keeps its own exit path.
use super::{Game,stats,original_factory::{self as native,Producer,FactoryExitState}};
impl Game{
 ///40b070's one fresh-mission constructor pass. Call after importing the35
 ///native BUILD slots and before ambient initialization, never on world sync.
 ///Both random streams remain separate and follow native slot order.
 pub(crate) fn initialize_native_factory_constructors(&mut self,v:&super::Value)->Option<usize>{
  if !self.map.original||self.map.native_road_graph.is_none(){return None;}let raw=v["nativeWorldBuildings"].as_array().filter(|a|a.len()==35)?;let records=self.native_world_buildings()?;if records.len()!=35{return None;}let mut metadata=Vec::new();for(slot,b)in records.iter().enumerate(){if b.kind<0{continue;}if b.kind>8{return None;}let r=raw.iter().find(|r|r["nativeSlot"].as_u64()==Some(slot as u64))?;let flags=u8::try_from(r["flags6"].as_u64()?).ok()?;metadata.push((slot,*b,flags,r["buildingId"].as_u64().map(|n|n as usize)));}
  let mut rng=self.original_rng.clone();rng.multiplayer=self.multiplayer;let mut crt=self.native_cosmetic_rng;let mut changes=Vec::new();for(slot,b,flags,id)in metadata{let fields=native::constructor_fields(b.kind,flags,b.hp,self.multiplayer,||rng.next_u32(),||{crt=crt.wrapping_mul(214013).wrapping_add(2531011);((crt>>16)&32767)as u16})?;changes.push((slot,b.kind,id,fields));}
  self.original_rng=rng;self.native_cosmetic_rng=crt;let count=changes.len();for(slot,kind,id,fields)in changes{self.set_native_building_constructor(slot,fields);if let Some(id)=id{self.native_factory_flags6.insert(id,fields.flags6);if let Some(b)=self.buildings.iter_mut().find(|b|b.id==id){if let Some(maximum)=fields.maximum1c{b.max_hp=f64::from(maximum);}if matches!(kind,1|2|4){self.native_factory_work.insert(id,native::Work{remaining:0,total:0,credit:0,animation:0,hold:0});b.remaining_work=None;b.progress=0.;b.completion_ticks=0;}}}}Some(count)
 }
 pub(crate) fn update_original_production(&mut self,dt:f64){
  self.production_clock+=dt*17.;let ticks=self.production_clock.floor()as usize;self.production_clock-=ticks as f64;
  //Compatibility entry for direct producer/CLI calls. Main's real BUILD
  //stage calls these two halves around its own retained artwork row.
  for _ in 0..ticks{for id in self.native_factory_draw_order(){if let Some(i)=self.buildings.iter().position(|b|b.id==id){self.prepare_native_factory_draw(i);self.finish_native_factory_draw(i);}}}
 }
 pub(crate) fn native_factory_draw_order(&self)->Vec<usize>{let mut order=self.native_display_building_order();for b in &self.buildings{if !order.contains(&b.id){order.push(b.id);}}order}
 ///44ea30→411860: the changed CP initializes only an empty, live producer
 ///with no ready stock. Existing BUILD7/work are never technology-rechecked.
 ///The default product follows the user's continuous-robot preference.
 pub(crate) fn initialize_native_capture_defaults(&mut self,sector:usize){
  for index in 0..self.buildings.len(){let b=&self.buildings[index];if b.sector!=sector||b.team<0||!matches!(b.kind.as_str(),"fort"|"robot"|"vehicle")||!b.alive(true)||b.product.is_some()||!b.ready_guns.is_empty(){continue;}let Some(kind)=self.default_product(b)else{continue;};let id=b.id;let mut work=self.native_factory_work.get(&id).cloned().unwrap_or(native::Work{remaining:0,total:0,credit:0,animation:0,hold:b.completion_ticks as i16});let duration=super::original_stats::PRODUCTS.iter().find(|p|p.0==kind).map(|p|p.5).unwrap_or(1);native::initialize_work(&mut work,duration,12);work.credit=0;self.buildings[index].product=Some(kind);self.store_native_factory_work(index,work);}
 }
 ///40ad60's word36 prefix, then the per-BUILD owner/HP/default-selection
 ///and40b1c0 fort mirror before40ab90 draws the remaining work. Work is
 ///NOT spent here: that happens after the clock submission in the same
 ///40b2a0/40c490 callback, never in40b180 (which only counts buildings).
 pub(crate) fn prepare_native_factory_draw(&mut self,index:usize){
   let b=&mut self.buildings[index];b.completion_ticks=super::original_building_display::completion_at_draw(b.completion_ticks as i16)as u32;let id=b.id;
   if let Some(work)=self.native_factory_work.get_mut(&id){work.hold=b.completion_ticks as i16;if work.hold!=0{work.animation=0;}}
   if !matches!(b.kind.as_str(),"robot"|"vehicle"|"fort")||b.team<0||!b.alive(true)||self.native_work_health(&self.buildings[index]).0==0.{return;}
   if self.buildings[index].kind=="fort"{self.sync_native_fort_producer_health(id);}
   if self.buildings[index].completion_ticks!=0{return;}
   //Keep the user's continuous-production default and valid selections.
   //The single BUILD is selected here; later BUILD slots have not run yet.
   if self.buildings[index].product.is_none(){
    let product=self.default_product(&self.buildings[index]);if product!=self.buildings[index].product{self.buildings[index].product=product;self.buildings[index].remaining_work=None;self.buildings[index].progress=0.;}
   }
   let Some(kind)=self.buildings[index].product.clone()else{return;};
   let total_work=super::original_stats::workload(&kind);
   let mut work=self.native_factory_work.remove(&id).unwrap_or(native::Work{remaining:total_work,total:total_work,credit:0,animation:0,hold:0});
   //The command adapter marks an explicit new product withremainingNone.
   //CPU's completion-time selector leaves that field present, retaining the
   //old40a8a0 work until its next original initialization event.
   if self.buildings[index].remaining_work.is_none(){let duration=super::original_stats::PRODUCTS.iter().find(|p|p.0==kind).map(|p|p.5).unwrap_or(1);native::initialize_work(&mut work,duration,12);work.credit=0;}
   work.remaining=self.buildings[index].remaining_work.map(|v|v as i32).unwrap_or(work.remaining);work.hold=self.buildings[index].completion_ticks as i16;
   self.store_native_factory_work(index,work);
 }
 fn store_native_factory_work(&mut self,index:usize,work:native::Work){let b=&mut self.buildings[index];b.remaining_work=Some(f64::from(work.remaining));b.progress=(1.-f64::from(work.remaining)/f64::from(work.total.max(1))).clamp(0.,1.);b.completion_ticks=work.hold as u16 as u32;self.native_factory_work.insert(b.id,work);}
 pub(crate) fn finish_native_factory_draw(&mut self,index:usize){
    let b=&self.buildings[index];if !matches!(b.kind.as_str(),"robot"|"vehicle"|"fort")||b.team<0||!b.alive(true)||self.native_work_health(b).0==0.{return;}
    let Some(kind)=b.product.clone()else{return;};let(id,team)=(b.id,b.team);let total=super::original_stats::workload(&kind);let mut work=self.native_factory_work.get(&id).cloned().unwrap_or(native::Work{remaining:b.remaining_work.map(|v|v as i32).unwrap_or(total),total,credit:0,animation:0,hold:b.completion_ticks as i16});work.hold=b.completion_ticks as i16;if let Some(remaining)=b.remaining_work{work.remaining=remaining as i32;}
    let duration=super::original_stats::PRODUCTS.iter().find(|p|p.0==kind).map(|p|p.5).unwrap_or(1);
    let (health,maximum)=self.native_work_health(&self.buildings[index]);
    let owned=self.sectors.iter().filter(|s|s.owner==team&&s.counts_for_production).count();let sectors=self.sectors.iter().filter(|s|s.counts_for_production).count();let rate=super::original_stats::rate(health,maximum,owned,sectors);
    let free=240-(0..240).filter(|&slot|self.units.iter().any(|u|u.team>=0&&super::original_combat::alive(true,u.hp)&&u.native_slot==Some(slot))).count();
    let count=self.units.iter().filter(|u|u.team==team&&u.native_slot.is_some()&&super::original_combat::alive(true,u.hp)).count()as i8;
    let queued=(self.buildings.iter().filter(|b|b.team==team&&self.native_work_health(b).0!=0.).map(|b|b.ready_guns.len()).sum::<usize>()as i16).wrapping_add(self.native_factory_placement.pending_team_count(team as usize));
    let gun=super::gun_placement::is_gun(&kind);let blocked=native::team_capacity_blocked(queued,count,120,free)||gun&&(self.native_sector_gun_count(team,self.buildings[index].sector).is_some_and(|n|n>3)||self.map.native_road_graph.is_none()&&self.buildings[index].ready_guns.len()>=super::gun_placement::READY_GUN_CAPACITY);
    if native::spend_work(&mut work,rate,blocked){
     native::complete_work(&mut work,duration,12);
     if gun{if self.complete_original_cpu_gun(id,&kind).is_none(){if self.map.native_road_graph.is_some(){self.buildings[index].ready_guns.insert(0,kind.clone());self.buildings[index].ready_guns.truncate(super::gun_placement::READY_GUN_CAPACITY);}else{self.buildings[index].ready_guns.push(kind.clone());}self.events.push(super::json!({"kind":"ready","team":team,"type":kind,"factory":id}));}}
     else{self.spawn_native_factory_group(id,&kind,team);}
     let pending=self.native_pending_sector_guns(self.buildings[index].sector);if let Some(product)=self.native_cpu_factory_product(index,false,pending){self.buildings[index].product=Some(product);}
    }
    self.store_native_factory_work(index,work);
 }
 pub(crate) fn spawn_native_factory_group(&mut self,factory:usize,kind:&str,team:i32){
  let Some(b)=self.buildings.iter().find(|b|b.id==factory).cloned()else{return;};
  let producer=match b.kind.as_str(){"fort"=>Producer::Fort,"robot"=>Producer::Robot,"vehicle"=>Producer::Vehicle,_=>return};
  let base=self.native_producer_position(&b);
  let squad=self.next_id;let layout=if matches!(kind,"grunt"|"tough"|"sniper"|"psycho"|"heavy"|"medium"|"light"|"launcher"|"missile"|"gatling"|"howitzer"|"firetruck"){1}else{0};
  let first=self.units.len();
  for role in 0..stats(kind).members{let exit=native::exit_geometry(producer,base,team as u8,role);
   self.spawn(kind,team,f64::from(exit.spawn[0])*2.,f64::from(exit.spawn[1])*2.,squad,None);
   let Some(u)=self.units.last_mut().filter(|u|u.squad==squad)else{continue;};
   u.formation_layout=layout;let state=FactoryExitState::new(exit.destination);u.native_flags9c=state.flags9c;u.native_order5c=state.order;u.native_factory_exit=Some(state);u.order="move".into();u.dest=Some((f64::from(exit.destination[0]-8)*2.,f64::from(exit.destination[1]-8)*2.));u.native_group_order_pending=true;
  }
  if self.units.len()>first{self.native_group_rosters.record_produced_group(team,squad,layout);if let Some(product)=super::original_stats::PRODUCTS.iter().find(|p|p.0==kind){self.initialize_native_group_descriptor(team,squad,product.1);}self.events.push(super::json!({"kind":"ready","team":team,"type":kind}));}
 }
 pub(crate) fn update_native_factory_exit(&mut self,index:usize){
  if !self.map.original{return;}
  let Some(mut exit)=self.units[index].native_factory_exit.take()else{return;};
  exit.flags9c=self.units[index].native_flags9c;exit.order=self.units[index].native_order5c;exit.protection=self.units[index].native_protectionaa;
  let unit=self.units[index].clone();let role=unit.native_slot.and_then(|slot|self.native_group_rosters.role(slot)).unwrap_or(0);let leader=role==-1;
  let group=self.native_group_rosters.group_slot(unit.team,unit.squad);
  let(accepted,set_group)=exit.begin_command(leader);
  if set_group{if let Some(slot)=group{self.native_group_rosters.pool[slot].roster[9]|=0x20;}}
  if accepted{let goal=(f64::from(exit.destination[0]-8)*2.,f64::from(exit.destination[1]-8)*2.);let path=self.route_unit(&unit,goal.0,goal.1);
   if !path.is_empty()||(unit.x-goal.0).abs()+(unit.y-goal.1).abs()<1.{
    exit.path_started(leader);let u=&mut self.units[index];u.path=path;u.original_motion=None;u.dest=Some(goal);u.hold_position=false;
    if leader{if let Some(slot)=group{let g=&mut self.native_group_rosters.pool[slot];g.roster[8]=g.roster[0x3c];g.roster[9]&=!0x20;}}
   }
  }
  self.units[index].native_flags9c=exit.flags9c;self.units[index].native_order5c=exit.order;self.units[index].native_protectionaa=exit.protection;
  if exit.protection>0||exit.blocks_formation(!self.units[index].path.is_empty()||self.units[index].native_route_pending.get().is_some()){self.units[index].native_factory_exit=Some(exit);}
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,5299);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.buildings.truncate(1);let b=&mut g.buildings[0];b.id=0;b.kind="robot".into();b.team=0;b.x=300.;b.y=200.;b.hp=1000.;b.max_hp=1000.;b.product=Some("grunt".into());b.remaining_work=Some(1.);b.completion_ticks=0;g.ai_teams.clear();g.native_world=None;g.native_strategy=None;g.native_factory_work.clear();g.native_group_rosters.clear();g.sectors.push(super::super::Sector{id:0,x:800.,y:800.,owner:0,home:true,counts_for_production:true});g}
 #[test]fn robot_completion_has_original_vertical_spawns_and_individual_exit_orders(){
  let mut g=field();g.update_original_production(1./17.);assert_eq!(g.units.len(),3,"remaining {:?},clock {},hp {},max {},hold {}",g.buildings[0].remaining_work,g.production_clock,g.buildings[0].hp,g.buildings[0].max_hp,g.buildings[0].completion_ticks);
  for(role,u)in g.units.iter().enumerate(){assert_eq!((u.x,u.y),(364.,300.-role as f64*8.));assert_eq!(u.native_factory_exit.as_ref().unwrap().destination,[190,198-role as i16*4]);assert_eq!(u.dest,Some((364.,380.-role as f64*8.)));}
  g.maintain_native_group_rosters();let squad=g.units[0].squad;assert_eq!(g.native_group_rosters.formation_layout(0,squad),Some(255));
  g.update_native_factory_exit(0);assert!(!g.units[0].path.is_empty());assert_eq!(g.native_group_rosters.formation_layout(0,squad),Some(1));assert_eq!(g.units[0].native_factory_exit.as_ref().unwrap().protection,68);
  g.advance_native_order_preamble(0);g.update_native_factory_exit(0);assert_eq!(g.units[0].native_factory_exit.as_ref().unwrap().flags9c&0x14,0);assert_eq!(g.units[0].native_factory_exit.as_ref().unwrap().protection,67);
 }
 #[test]fn later_factories_observe_slots_consumed_by_earlier_completions(){
  let mut g=field();let mut b=g.buildings[0].clone();b.id=1;g.buildings.push(b);for i in 0..232{g.spawn("grunt",(i&1)as i32,1200.,900.,i+100,None);}
  g.update_original_production(1./17.);assert_eq!(g.units.len(),235);assert_eq!(g.buildings[0].completion_ticks,68);assert_eq!(g.buildings[1].completion_ticks,0);assert_eq!(g.buildings[1].remaining_work,Some(1.));
 }
 #[test]fn native_human_ready_stock_keeps_the_newest_four_products_and_discards_the_oldest(){
  let mut g=field();g.map.native_road_graph=Some(super::super::original_route::RoadGraph{regions:vec![]});g.buildings[0].product=Some("cannon".into());g.buildings[0].ready_guns=["cannon","missile","howitzer","gatling"].into_iter().map(str::to_string).collect();
  g.update_original_production(1./17.);assert_eq!(g.buildings[0].ready_guns,["cannon","cannon","missile","howitzer"]);assert_eq!(g.buildings[0].completion_ticks,68);
  g.buildings[0].completion_ticks=0;g.buildings[0].remaining_work=Some(10000.);assert!(g.remaining_seconds(&g.buildings[0]).is_some_and(|n|n>0),"40ab90 draws before capacity checks; four native stock slots still have a production clock");
 }
 #[test]fn preserved_cpu_work_and_explicit_product_initialization_have_different_native_timing(){
  let mut g=field();g.buildings[0].product=Some("psycho".into());g.buildings[0].remaining_work=Some(1000.);g.native_factory_work.insert(0,native::Work{remaining:1000,total:1000,credit:0,animation:0,hold:0});g.update_original_production(1./17.);assert_eq!(g.native_factory_work[&0].total,1000,"automatic product changes retain the old completion-time40a8a0 total");
  g.buildings[0].remaining_work=None;g.update_original_production(1./17.);assert_eq!(g.native_factory_work[&0].total,super::super::original_stats::workload("psycho"));
 }
 #[test]fn fort_production_initializes_a_zero_disk_proxy_and_refreshes_global_health_on_its_callback(){
  let mut g=field();let b=&mut g.buildings[0];b.kind="fort".into();b.raw_base=Some((150.,100.));b.hp=9999.;b.max_hp=9999.;b.remaining_work=Some(1000.);
  g.import_native_world(&super::super::json!({"nativeWorldBuildings":[{"nativeSlot":4,"kind":2,"x":150,"y":116,"hp":0,"maxHp":0,"buildingId":0,"fortProxy":true}]}));assert_eq!(g.native_world_buildings().unwrap()[4].hp,9999,"40b070→40b250→40b1c0 runs after45fa10 creates the proxy");assert_eq!(g.native_work_health(&g.buildings[0]),(9999.,9999.));
  g.update_original_production(1./17.);let remaining=g.buildings[0].remaining_work.unwrap();assert!(remaining<1000.&&remaining>0.);g.buildings[0].hp=5000.;assert_eq!(g.native_work_health(&g.buildings[0]),(9999.,9999.),"global contact does not eagerly overwrite the producer mirror");g.update_original_production(1./17.);assert_eq!(g.native_world_buildings().unwrap()[4].hp,5000);assert!(g.buildings[0].remaining_work.unwrap()<remaining);
 }
 #[test]fn fresh_constructor_pass_uses_build_slot_order_and_separate_rng_streams_in_both_modes(){for multiplayer in [false,true]{let mut g=field();g.multiplayer=multiplayer;g.map.native_road_graph=Some(super::super::original_route::RoadGraph{regions:vec![]});let mut records=(0..35).map(|slot|super::super::json!({"nativeSlot":slot,"kind":-1,"flags6":255,"x":-1,"y":-1,"hp":0,"maxHp":0})).collect::<Vec<_>>();for(id,kind,rawkind,hp)in[(0,"robot",4,1000),(1,"repair",0,500),(2,"fort",2,9999),(3,"robot",4,0)]{let mut b=g.buildings[0].clone();b.id=id;b.kind=kind.into();b.hp=hp as f64;b.max_hp=hp as f64;b.raw_base=(kind=="fort").then_some((400.,400.));if id==0{g.buildings[0]=b;}else{g.buildings.push(b);}records[id]=super::super::json!({"nativeSlot":id,"buildingId":id,"kind":rawkind,"flags6":255,"x":100+id*100,"y":100,"hp":hp,"maxHp":hp,"fortProxy":kind=="fort"});}let input=super::super::json!({"nativeWorldBuildings":records});g.import_native_world(&input);g.native_cosmetic_rng=1;let mut expected=g.original_rng.clone();expected.multiplayer=multiplayer;expected.next_u32();let mut crt=1u32;let mut values=vec![];for _ in 0..3{if multiplayer{values.push(0);}else{crt=crt.wrapping_mul(214013).wrapping_add(2531011);values.push(((crt>>16)&32767)as u8);}}assert_eq!(g.initialize_native_factory_constructors(&input),Some(4));assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.native_cosmetic_rng,crt);for(slot,value)in [0,2,3].into_iter().zip(values){assert_eq!(g.native_building_constructor(slot).unwrap().clock3e,Some(value));}assert_eq!(g.native_building_constructor(1).unwrap().door3a,32);assert_eq!(g.native_factory_flags6.get(&0),Some(&0xe0));assert_eq!(g.native_factory_work[&0].remaining,0);assert_eq!(g.buildings[0].remaining_work,None,"the user's selected default is initialized by its next production callback");}}
 #[test]fn real_build_stage_draws_before_completion_and_decrements_the_new_hold_once_per_later_callback(){
  let mut g=field();g.capture_native_building_display();let row=g.native_display_queue().into_iter().find(|r|r["kind"]=="building").unwrap();assert_eq!(row["art"]["completionTicks"],0);assert_eq!(g.buildings[0].completion_ticks,68);assert_eq!(g.units.len(),3);g.capture_native_building_display();assert_eq!(g.buildings[0].completion_ticks,67);assert_eq!(g.native_display_queue().into_iter().find(|r|r["kind"]=="building").unwrap()["art"]["completionTicks"],67);g.update_original_production(0.);assert_eq!(g.buildings[0].completion_ticks,67);assert_eq!(g.units.len(),3);
 }
 #[test]fn held_cold_work_and_existing_product_survive_lower_technology(){
  let mut g=field();g.buildings[0].product=Some("psycho".into());g.buildings[0].remaining_work=Some(777.);g.buildings[0].completion_ticks=2;g.buildings[0].stars=1;g.capture_native_building_display();assert_eq!(g.buildings[0].product.as_deref(),Some("psycho"));assert_eq!(g.buildings[0].remaining_work,Some(777.));assert_eq!(g.buildings[0].completion_ticks,1);assert_eq!(g.native_factory_work[&0].animation,0);g.capture_native_building_display();assert_eq!(g.buildings[0].remaining_work,Some(751.));assert_eq!(g.buildings[0].completion_ticks,0);
 }
 #[test]fn held_warm_work_reads_the_actual_changed_build24_instead_of_a_stale_shadow(){
  let mut g=field();g.capture_native_building_display();let total=g.native_factory_work[&0].total;g.buildings[0].remaining_work=Some(f64::from(total/2));g.capture_native_building_display();assert_eq!(g.buildings[0].completion_ticks,67);assert_eq!(g.buildings[0].remaining_work,Some(f64::from(total/2)));assert_eq!(g.native_factory_work[&0].remaining,total/2);assert_eq!(g.native_factory_work[&0].total,total);
 }
 #[test]fn capture_initializes_only_empty_unstocked_live_producers_and_consumes_credit_once(){
  let mut g=field();g.buildings[0].product=None;g.buildings[0].remaining_work=None;g.buildings[0].sector=7;g.native_factory_work.insert(0,native::Work{remaining:0,total:1000,credit:123,animation:17,hold:0});for id in 1..5{let mut b=g.buildings[0].clone();b.id=id;if id==1{b.product=Some("psycho".into());b.remaining_work=Some(777.);}if id==2{b.ready_guns.push("cannon".into());}if id==3{b.hp=0.;}if id==4{b.kind="radar".into();}g.buildings.push(b);}g.initialize_native_capture_defaults(7);assert_eq!(g.buildings[0].product.as_deref(),Some("grunt"));assert_eq!(g.buildings[0].remaining_work,Some(f64::from(super::super::original_stats::workload("grunt")-123)));assert_eq!(g.native_factory_work[&0].credit,0);assert_eq!(g.buildings[1].product.as_deref(),Some("psycho"));assert_eq!(g.buildings[1].remaining_work,Some(777.));for b in &g.buildings[2..]{assert_eq!(b.product,None);}let before=g.buildings[0].remaining_work;g.initialize_native_capture_defaults(7);assert_eq!(g.buildings[0].remaining_work,before);
 }
 #[test]fn native_build_order_interleaves_radar_and_first_factory_slots_before_later_capacity_checks(){
  let mut g=field();let mut first=g.buildings[0].clone();first.id=1;first.x=600.;let mut radar=first.clone();radar.id=2;radar.x=900.;radar.kind="radar".into();radar.product=None;g.buildings.push(first);g.buildings.push(radar);let mut raw=(0..35).map(|slot|super::super::json!({"nativeSlot":slot,"kind":-1,"x":-1,"y":-1,"hp":0,"maxHp":0})).collect::<Vec<_>>();for(slot,id,kind,x)in[(0,1,4,300),(1,2,8,450),(2,0,4,150)]{raw[slot]=super::super::json!({"nativeSlot":slot,"buildingId":id,"kind":kind,"x":x,"y":100,"hp":1000,"maxHp":1000,"flags6":0,"nativeWord0A":0});}g.import_native_world(&super::super::json!({"nativeWorldBuildings":raw}));for id in 0..232{g.spawn("grunt",(id&1)as i32,1200.,900.,100+id,None);}g.native_callback_mode=255;g.capture_native_building_display();assert_eq!(g.buildings.iter().find(|b|b.id==1).unwrap().completion_ticks,68);assert_eq!(g.buildings[0].completion_ticks,0);assert_eq!(g.buildings[0].remaining_work,Some(1.));assert_eq!(g.units.len(),235);let rows=g.native_display_queue();assert_eq!(rows.iter().filter(|r|r["kind"]=="building").map(|r|r["id"].as_u64().unwrap()).collect::<Vec<_>>(),[1,2,0]);assert_eq!(rows.iter().find(|r|r["source"]=="radarDish").unwrap()["sprite"],1488);assert_eq!(g.native_radar_raw_state(1).unwrap().word0a,1);
 }
}
