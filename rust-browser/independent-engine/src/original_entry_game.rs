//!450db0/450cd0 ordinary hardware conversion. The native actor slot survives;
//!the browser's existing neutral renderer entity receives that slot explicitly.
use super::{Game,Unit,Stats,Crew,stats,original_entry as entry,original_combat,native_math,original_hardware,json};
impl Game{
 ///53130 consumes the unload command for all native hardware. Fixed guns
 ///restore the robot through44c970; vehicles use59830/59720 and phase64.
 ///Graph-less legacy fixtures keep their existing compatibility adapter.
 pub(crate) fn unload_native_hardware(&mut self,index:usize)->bool{
  let Some(carrier)=self.units.get(index).cloned()else{return false;};if !self.map.original{return false;}if carrier.kind=="apc"{return self.unload_native_apc(index);}if self.map.native_road_graph.is_none()||stats(&carrier.kind).cls==0{return false;}if stats(&carrier.kind).cls!=2{return self.unload_native_vehicle(index);}
  self.consume_native_unload_order(index);
  let Some(crew)=carrier.crew.clone()else{return true;};let cell=((carrier.y/32.).floor()as usize).saturating_mul(self.map.width).saturating_add((carrier.x/32.).floor()as usize);if self.map.terrain_codes.get(cell).is_some_and(|c|c&127==16){self.reset_native_idle(index);return true;}
  let hardware=carrier.native_hardware_slot.and_then(|s|self.native_hardware_pool.get(s));let saved=hardware.map(|w|w.attributes);let retained=carrier.transport_actor.and_then(|id|self.units.iter().position(|u|u.id==id));let previous=retained.map(|i|self.units[i].clone());let restored_class=previous.as_ref().map_or_else(||original_combat::runtime_class(&crew.kind)as u8,|u|u.native_class7c);
  // The authored direct gun has no previous driver adapter or GROUP+4
  // profile. 44c910 selects the first available GOG foot profile (Grunt).
  let live=carrier.native_class7c==2&&hardware.is_some_and(|w|w.live_marker==0);
  let group=live.then(||self.fixed_gun_group(carrier.native_slot)).flatten();
  let direct=group.filter(|&(slot,group)|carrier.transport_actor.is_none()&&crew.kind=="grunt"
   &&matches!(self.native_selection.contexts[slot].saved_class8e,Some(2|4|5|6))&&self.native_group_rosters.pool[group].roster[4..8]==[0;4]);
  if let Some((slot,group))=direct{
   // 44c970 stores the selected descriptor in both raw roster fields on
   // clear terrain and writes WORD8d=4, whose high byte is saved class8e.
   let profile=super::original_group_lifecycle::authored_rinfo(2).unwrap();let roster=&mut self.native_group_rosters.pool[group];
   roster.set_active_rinfo(profile);roster.set_cached_rinfo(profile);
   self.record_native_selection_saved_class(slot,0);
  }else if let Some((_,group))=group.filter(|_|retained.is_some()){
   let roster=&mut self.native_group_rosters.pool[group];
   if let Some(profile)=roster.cached_rinfo(){roster.set_active_rinfo(profile);}
  }
  self.park_native_hardware_for_unload(index,&carrier);let object=self.native_object_slot(carrier.id);self.units[index].team=-1;self.units[index].crew=None;self.units[index].cargo.clear();self.units[index].native_slot=None;self.units[index].transport_actor=None;self.units[index].native_hardware=None;self.units[index].native_transport=None;self.units[index].path.clear();self.units[index].dest=None;self.units[index].target=None;self.units[index].order="guard".into();self.units[index].native_order5c=0;self.units[index].native_flags9c&=!32;
  //The native UNIT itself is the robot again. Only the renderer entity needs
  //a separate Web identity; current conversions retain its original identity.
  let ri=if let Some(i)=retained{i}else if let Some((slot,_))=direct{self.push_unit_adapter(&crew.kind,carrier.team,carrier.x,carrier.y,carrier.squad,None,Some(slot),stats(&crew.kind))}else{let n=self.units.len();self.spawn(&crew.kind,carrier.team,carrier.x,carrier.y,carrier.squad,None);if self.units.len()==n{return true;}n};
  let mut robot=previous.unwrap_or_else(||self.units[ri].clone());robot.kind=crew.kind.clone();robot.team=carrier.team;robot.hp=crew.hp;robot.max_hp=stats(&crew.kind).hp;robot.x=carrier.x;robot.y=carrier.y+20.;robot.squad=carrier.squad;robot.native_slot=carrier.native_slot;robot.native_hardware_slot=None;robot.native_hardware=None;robot.native_class7c=restored_class;robot.native_phase35=0;robot.native_frame32=0;robot.native_timer3c=0;robot.native_order5c=0;robot.native_target46=carrier.native_target46;robot.native_trait7a=carrier.native_trait7a;robot.native_cyclea0=carrier.native_cyclea0;robot.native_reload7b=carrier.native_reload7b;robot.native_bucket38=carrier.native_bucket38;robot.native_flags9c=carrier.native_flags9c&!32;robot.native_flags64=carrier.native_flags64;robot.native_mask98=carrier.native_mask98;robot.native_control06=carrier.native_control06;robot.native_saved_control07=carrier.native_saved_control07;robot.native_protectionaa=carrier.native_protectionaa;robot.grenades=carrier.grenades;robot.native_initiative_scan=carrier.native_initiative_scan;if let Some(object)=object{robot.native_initiative_scan.excluded9d=object;}
  robot.native_range4=robot.native_range4.or_else(||super::native_ai_scoring::default_input(&crew.kind).map(|p|p.range.wrapping_mul(2)as i8));robot.native_route_flip.set(carrier.native_route_flip.get());robot.native_route_road_replaced.set(carrier.native_route_road_replaced.get());robot.native_secondary_route.set(false);robot.native_route_pending.set(None);robot.native_transport=None;robot.native_transport_slot=None;robot.transport_actor=None;robot.transport_carrier=None;robot.angle=carrier.angle;robot.aim=carrier.angle;robot.path.clear();robot.original_motion=None;robot.native_walk=None;robot.native_fire=None;robot.native_contact=None;robot.native_cover=None;robot.native_dodge=None;robot.pursuit=None;robot.target=None;robot.dest=None;robot.order="guard".into();robot.moving=false;robot.hold_position=false;robot.explicit_stop=false;robot.initiative=None;robot.march_resume=None;
  if direct.is_some(){robot.native_cell10=carrier.native_cell10;robot.native_footprint18=carrier.native_footprint18;robot.native_permission1c=carrier.native_permission1c;robot.native_terrain_tag63=carrier.native_terrain_tag63;robot.native_pattern20=carrier.native_pattern20;robot.native_field62=carrier.native_field62;robot.native_speed8b=-1;}
  let mut combat=robot.original_combat.take().unwrap_or_else(||original_combat::CombatState::new(&crew.kind));if let Some(current)=carrier.original_combat.as_ref(){combat.aim_skill=current.aim_skill;}if let Some(w)=saved{combat.fire_rate=w.byte(41);combat.projectile_speed=Some(w.byte(42));}robot.original_combat=Some(combat);let id=robot.id;self.units[ri]=robot;if let Some(slot)=carrier.native_slot{self.native_group_rosters.remap_entity(slot,carrier.id,id);self.rebind_native_crane_identity(slot,carrier.id,id);}self.remap_native_pursuit_identity(carrier.id,id);self.maintain_native_group_rosters();self.reset_native_idle(ri);self.sync_native_world();self.events.push(json!({"kind":"unloaded","team":carrier.team,"unitId":carrier.id,"memberIds":[id]}));true
 }
 fn fixed_gun_group(&self,slot:Option<usize>)->Option<(usize,usize)>{
  let slot=slot?;
  let group=usize::from(self.native_group_rosters.members.get(slot)?.as_ref()?.group);let roster=self.native_group_rosters.pool.get(group)?;
  (roster.roster[18]==1&&roster.member(0)==Some(slot as u8)).then_some((slot,group))
 }
 ///Original weapon damage/range come from the shared W record after takeover,
 ///not from the image/profile name. APC Unit4 retains its separate bodyrange.
 pub(crate) fn native_unit_stats(&self,u:&Unit)->Stats{
  let mut d=stats(&u.kind);if !self.map.original{return d;}if let Some(w)=u.native_hardware_slot.and_then(|s|self.native_hardware_pool.get(s)).filter(|w|w.live_marker>=0){if u.kind!="apc"{d.damage=f64::from(w.attributes.byte(6));d.range=f64::from(w.attributes.byte(5))*16.;}}if let Some(range)=u.native_range4{d.range=f64::from(range)*8.;}d
 }
 ///Returntrue only when the authored campaign's ordinary enter state owns
 ///this callback. APC transport and procedural/legacy fixtures stay separate.
 pub(crate) fn update_native_hardware_entry(&mut self,index:usize)->bool{
  if !self.map.original||self.map.native_road_graph.is_none(){return false;}let Some(u)=self.units.get(index).cloned()else{return false;};if stats(&u.kind).cls!=0||u.order!="enter"{return false;}
  let Some(target)=u.target.and_then(|id|self.units.iter().position(|v|v.id==id&&v.team<0&&v.crew.is_none()&&original_combat::alive(true,v.hp)))else{return false;};
  let Some(slot)=u.native_slot else{return false;};let Some(mut m)=self.native_group_rosters.members[slot].as_ref().cloned()else{return false;};if !matches!(m.contact_order,1|3){return false;}
  //452200 owns the finite head and calls50db0 only after51170. Its
  //arrival callback has already spent this UNIT's movement budget.
  let managed=self.owns_native_route(index);
  //The accepted UI claim defers its578b0 route leaf to this callback. The
  //source leaf is59c60→52810: prepend the claimed waypoint to the retained
  //primary list.53730 would incorrectly release that reservation first.
  //Consume only this pending event; finite-pool exhaustion does not invent
  //a successful line or turn it into a fresh global MOVE.
  if managed&&u.native_phase35==0{if let Some(anchor)=self.units[index].native_route_pending.take(){
   let waypoint=[((anchor.0/2.).floor()as i16).wrapping_add(8),((anchor.1/2.).floor()as i16).wrapping_add(8)];self.prepend_native_walk_claim(index,waypoint);
   if let Some(current)=self.native_group_rosters.members[slot].as_ref(){m=current.clone();}
  }}
  if managed&&self.units[index].native_phase35==2{return false;}
  if m.flags31&64!=0||(11..=14).contains(&u.native_phase35){return true;}
  if m.role<0{if let Some(g)=self.native_group_rosters.current_roster{self.native_group_rosters.pool[g].roster[9]&=!1;}}self.units[index].native_flags64&=!64;
  let Some(object)=self.native_group_rosters.contact_target(slot).and_then(|s|self.native_world_scan_objects().into_iter().find(|o|o.slot==s))else{self.finish_order(index);return true;};
  if object.sprite==0x1435{return false;}
  if !managed&&(!self.units[index].path.is_empty()||self.units[index].native_route_pending.get().is_some()){self.move_original(index,self.motion_tick,1,true);}
  if object.flag_delay1f!=0{return true;}
  let p=[(self.units[index].x/2.).floor()as i16,(self.units[index].y/2.).floor()as i16];if !entry::within_entry_radius(p,object.position){return true;}
  if !entry::available(object.definition){if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags31&=!8;m.contact_order=0;}return true;}
  self.complete_native_hardware_entry(index,target,object.slot,object.definition,object.position,object.flags13,m.contact_order==3);true
 }
 fn complete_native_hardware_entry(&mut self,index:usize,target:usize,object:u8,definition:u16,origin:[i16;2],flags13:u8,vehicle:bool){
  let Some(input)=self.native_object_hardware_input(object)else{return;};let driver=self.units[index].clone();let renderer=self.units[target].clone();
  //50db0 clears31:8 before4bba0, so the retired OBJECT reservation isnot
  //released a second time. Its remaining bytes stay in the recycled slot.
  if let Some(slot)=driver.native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags31&=!8;}}
  let point=entry::occupied_origin(definition,origin,vehicle);self.units[index].x=f64::from(point[0])*2.;self.units[index].y=f64::from(point[1])*2.;self.units[index].native_bucket38=super::original_initiative::native::coarse_cell(point);
  if let Some(sector)=self.native_flag_contact(index){self.capture_sector(driver.team,sector);self.finish_native_flag_contact(index);}
  self.apply_native_ambient_object(object,[-1,origin[1]],flags13);
  let Some(hardware)=self.allocate_native_takeover_record(index)else{return;};
  let old=driver.original_combat.clone().unwrap_or_else(||original_combat::CombatState::new(&driver.kind));let previous_speed=old.projectile_speed.unwrap_or_else(||super::native_ai_scoring::default_input(&driver.kind).map_or(15,|i|i.projectile_speed));
  let controls=self.native_hardware_pool[hardware].attributes.take_over(input,old.fire_rate,previous_speed);let profile=self.native_hardware_pool[hardware].attributes.initialize_occupied(definition,vehicle);
  //57ff0@45802a writes this retained native slot before renderer ID remap.
  if vehicle&&definition==35{self.note_native_crane_initializer(index);if let Some(slot)=driver.native_slot{self.record_native_selection_link(slot,Some(0x2084));}}
  if !vehicle{if let Some(slot)=driver.native_slot{self.record_native_selection_saved_class(slot,driver.native_class7c);}}
  let mut order=self.native_hardware_pool[hardware].order;order.selected_target12=255;if vehicle{order.field17=0;}self.native_hardware_pool[hardware].set_order(order);
  let attributes=self.native_hardware_pool[hardware].attributes;let mut state=old;state.fire_rate=controls.fire_rate;state.projectile_speed=Some(controls.projectile_speed);state.movement_control=attributes.byte(7);state.secondary_state=attributes.byte(25)as i8;
  let u=&mut self.units[index];u.kind=profile.kind.into();u.native_family30=None;u.crew=Some(Crew{kind:driver.kind.clone(),hp:driver.hp});u.hp=f64::from(attributes.word(8));u.max_hp=f64::from(attributes.word(10));u.original_combat=Some(state);u.native_range4=Some(controls.double_range as i8);u.native_reload7b=0;if !vehicle{u.native_counter84=-1;}u.native_class7c=profile.class;u.native_phase35=profile.phase;u.native_frame32=0;u.native_timer3c=profile.timer;u.native_order5c=0;u.cooldown=0.;u.angle=native_math::angle(profile.heading);u.aim=u.angle;u.native_hardware=Some(original_hardware::State::new(profile.heading,attributes.byte(25)as i8));u.native_idle=None;u.native_walk=None;u.native_fire=None;u.native_contact=None;u.native_cover=None;u.path.clear();u.dest=None;u.target=None;u.order="guard".into();u.moving=false;u.initiative=None;u.march_resume=None;u.original_motion=None;u.native_route_pending.set(None);u.native_secondary_route.set(false);u.hold_position=true;
  //Each ordinary vehicle initializer ends in41b950. Fixed582c0 doesnot.
  if vehicle{self.refresh_native_collision_tables(index,None);}
  if let Some(slot)=driver.native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags31&=!16;m.contact_order=0;m.flagsb7&=!1;}self.native_group_rosters.record_secondary_path(slot,false);}
  //50db0 clears fixed-gun paths before4bba0;50cd0 clears vehicle paths
  //after the new single-member group has been constructed (@450d6d).
  if !vehicle{self.clear_native_owned_paths(index);}
  if let Some(group)=self.reform_native_hardware_group(index){self.native_group_rosters.pool[group].roster[58]=0;}
  if vehicle{self.clear_native_owned_paths(index);}
  let mut converted=self.units[index].clone();converted.id=renderer.id;converted.fort_id=renderer.fort_id;converted.transport_actor=Some(driver.id);
  self.units[target]=converted;self.units[index]=driver.clone();let retired=&mut self.units[index];retired.hp=-1.;retired.native_slot=None;retired.native_hardware_slot=None;retired.crew=None;retired.cargo.clear();
  if let Some(slot)=driver.native_slot{self.native_group_rosters.remap_entity(slot,driver.id,renderer.id);self.rebind_native_crane_identity(slot,driver.id,renderer.id);}self.remap_native_pursuit_identity(driver.id,renderer.id);self.maintain_native_group_rosters();
  self.events.push(json!({"kind":"crewed","team":driver.team,"type":profile.kind,"unitId":renderer.id,"memberIds":[driver.id]}));
 }
}

#[cfg(test)]mod tests{
 use super::*;
 use super::super::{original_route,original_group_game,Value};
 fn field(kind:&str,definition:u16,equipment:u8)->(Game,usize,usize){
  field_with_driver(kind,definition,equipment,"grunt")
 }
 fn field_with_driver(kind:&str,definition:u16,equipment:u8,driver:&str)->(Game,usize,usize){
  let mut g=Game::new(0,1,33191);g.map.original=true;g.map.width=64;g.map.height=64;g.map.world=0;g.map.tiles=vec![0;4096];g.map.original_tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.map.bridge_base=vec![-1;4096];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
  g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters=original_group_game::State::default();g.ai_teams.clear();g.events.clear();g.next_id=1;g.winner=None;
  g.spawn(kind,-1,640.,640.,0,None);let carrier=g.units[0].id;g.objects.push(json!({"type":definition,"equipmentId":equipment,"x":640,"y":640,"nativeHp":(stats(kind).hp as i16/25),"nativeFlags13":0x70}));g.import_native_world(&Value::Null);
  for n in 0..3{g.spawn(driver,0,640.+n as f64*4.,660.,7,None);}let actor=g.units[1].id;g.units[1].hp=17.;g.units[1].native_reload7b=37;g.units[1].grenades=3;g.units[1].original_combat.as_mut().unwrap().aim_skill=83;g.maintain_native_group_rosters();g.native_group_rosters.observe_unit_context(g.units[1].native_slot.unwrap());g.command(&json!({"kind":"enter","ids":[actor],"target":carrier}));(g,carrier,actor)
 }
 #[test]fn all_twelve_nonapc_definitions_take_the_original_kind_origin_attributes_and_fresh_group(){
  for(kind,definition,eq)in[("cannon",5,0),("medium",8,1),("missile",21,9),("gatling",22,11),("howitzer",23,12),("launcher",24,6),("heavy",26,5),("light",29,8),("jeep",33,13),("crane",35,15),("firetruck",37,14),("apc",58,10)]{
   let(mut g,carrier,actor)=field(kind,definition,eq);let ai=g.units.iter().position(|u|u.id==actor).unwrap();g.units[ai].native_counter84=1234;g.units[ai].native_mask98=3;let slot=g.units[ai].native_slot;let previous_group=g.native_group_rosters.group_slot(0,7).unwrap();
   //Type58 is an admitted alternateOBJECT that takes50be0's default branch.
   //Its unused UI classification is outside this entry callback fixture.
   if definition==58{let object=g.native_object_slot(carrier).unwrap();g.units[ai].order="enter".into();g.units[ai].target=Some(carrier);let slot=slot.unwrap();g.native_group_rosters.record_contact_target(slot,object);let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.contact_order=3;m.flags31|=8;}
   assert!(g.update_native_hardware_entry(ai),"{kind}");let c=g.units.iter().find(|u|u.id==carrier).unwrap();assert_eq!(c.team,0,"{kind}");assert_eq!(c.native_slot,slot);assert_eq!(c.native_mask98,3,"the original actor slot retains98 across the neutral renderer identity change");assert_eq!(c.crew.as_ref().unwrap().hp,17.);assert_eq!(c.grenades,3);assert_eq!(c.original_combat.as_ref().unwrap().aim_skill,83);assert_eq!(c.native_order5c,0);assert_eq!(c.native_reload7b,0);assert_eq!(c.order,"guard");assert!(g.units[ai].native_slot.is_none());
   let source=entry::equipment_attributes(eq).unwrap();let vehicle=![5,21,22,23].contains(&definition);if !vehicle{assert_eq!(c.native_counter84,-1,"582c0 resets84 when taking over every fixed-gun type");}let point=entry::occupied_origin(definition,[320,320],vehicle);assert_eq!((c.x,c.y),(f64::from(point[0])*2.,f64::from(point[1])*2.));let mut attrs=entry::HardwareAttributes::default();attrs.take_over(source,80,15);let profile=attrs.initialize_occupied(definition,vehicle);assert_eq!((c.kind.as_str(),c.native_class7c,c.native_phase35),(profile.kind,profile.class,profile.phase));assert_eq!((c.angle,c.aim),(native_math::angle(6),native_math::angle(6)));assert_eq!(c.original_combat.as_ref().unwrap().projectile_speed,Some(source.byte(25)));assert_ne!(g.native_group_rosters.group_slot(c.team,c.squad),Some(previous_group));assert_eq!(g.native_group_rosters.pool[previous_group].roster[18],2);assert!(g.native_object_slot(carrier).is_none());
  }
 }
 #[test]fn ordinary_entry_preserves_hull_damage_and_live_w_values_drive_primary_combat_stats(){
  let(mut g,carrier,actor)=field("medium",8,1);g.units[0].hp=75.;g.sync_native_world();let ai=g.units.iter().position(|u|u.id==actor).unwrap();assert!(g.update_native_hardware_entry(ai));let ci=g.units.iter().position(|u|u.id==carrier).unwrap();assert_eq!((g.units[ci].hp,g.units[ci].max_hp),(75.,200.));let w=g.units[ci].native_hardware_slot.unwrap();g.native_hardware_pool[w].attributes.bytes[6]=17;g.native_hardware_pool[w].attributes.bytes[5]=4;g.units[ci].native_range4=Some(8);let d=g.native_unit_stats(&g.units[ci]);assert_eq!((d.damage,d.range),(17.,64.));
  g.spawn("medium",1,840.,g.units[ci].y,9,None);let enemy=g.units.last().unwrap().clone();assert!(!g.native_target_in_range(&g.units[ci],&enemy,d.range));assert!(g.native_target_in_range(&g.units[ci],&enemy,stats("medium").range));
 }
 #[test]fn managed_entry_leaves_phase2_to_the_native_walker_then_retires_its_pool_links_on_takeover(){
  for(kind,definition,equipment)in[("medium",8,1),("gatling",22,11)]{let(mut g,carrier,actor)=field(kind,definition,equipment);let ai=g.units.iter().position(|u|u.id==actor).unwrap();let slot=g.units[ai].native_slot.unwrap();let head=g.native_routes.get_mut().pool.allocate_goal([328,338]).unwrap();g.native_routes.get_mut().managed[slot]=true;g.native_routes.get_mut().pool.links[slot].primary=head;g.units[ai].native_phase35=2;g.units[ai].native_route_pending.set(None);g.units[ai].path=std::collections::VecDeque::from([(640.,660.)]);let before=(g.units[ai].x,g.units[ai].y);assert!(!g.update_native_hardware_entry(ai));assert_eq!((g.units[ai].x,g.units[ai].y),before);assert_eq!(g.native_routes.borrow().pool.links[slot].primary,head);g.units[ai].native_phase35=0;assert!(g.update_native_hardware_entry(ai));assert_eq!(g.native_routes.borrow().pool.links[slot].primary,0);assert_eq!(g.native_routes.borrow().pool.links[slot].secondary,0);assert_eq!(g.native_routes.borrow().pool.node(head).unwrap().next,crate::original_route_pool::FREE);assert_eq!(g.units.iter().find(|u|u.id==carrier).unwrap().native_slot,Some(slot));}
 }
 #[test]fn accepted_far_claim_starts_its_deferred_native_prepend_and_eventually_crews_hardware(){
  let(mut g,carrier,actor)=field("jeep",33,13);let ai=g.units.iter().position(|u|u.id==actor).unwrap();let slot=g.units[ai].native_slot.unwrap();g.units[ai].x=320.;g.units[ai].y=640.;for i in 1..g.units.len(){if i!=ai{g.units[i].hp=-1.;}}g.spawn("gatling",1,1800.,1800.,99,None);g.units.last_mut().unwrap().native_reload7b=127;g.ai_teams.clear();g.maintain_native_group_rosters();g.native_route_mode=true;g.native_routes.get_mut().managed[slot]=true;
  let old=g.native_routes.get_mut().pool.allocate_goal([508,328]).unwrap();g.native_routes.get_mut().pool.links[slot].primary=old;g.command(&json!({"kind":"stop","ids":[actor]}));g.command(&json!({"kind":"enter","ids":[actor],"target":carrier}));assert_eq!(g.units[ai].motion_bonus,3,"578b0 contact speed survives the command wrapper");assert!(g.units[ai].path.is_empty());assert!(g.units[ai].native_route_pending.get().is_some());assert_eq!(g.units[ai].native_phase35,0);let contact=g.native_group_rosters.members[slot].as_ref().unwrap().contact_order;
  g.step(0.059);assert_eq!(g.native_route_boundary,None);assert!(g.units[ai].native_route_pending.get().is_none());assert_eq!(g.units[ai].native_phase35,2);assert!(g.units[ai].original_motion.is_some(),"the next callback starts525a0 even when its movement-pattern step is zero");assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().contact_order,contact,"53730 must not erase578b0's reservation");let head=g.native_routes.borrow().pool.links[slot].primary;assert_eq!(g.native_routes.borrow().pool.points(head).last(),Some(&[508,328]),"59c60 retains the pre-existing march tail");
  let mut moved=false;for _ in 0..800{g.step(0.059);assert_eq!(g.native_route_boundary,None);moved|=(g.units[ai].x,g.units[ai].y)!=(320.,640.);if g.units.iter().find(|u|u.id==carrier).unwrap().team==0{break;}}assert!(moved,"the commanded robot must actually approach its hardware");
  let occupied=g.units.iter().find(|u|u.id==carrier).unwrap();assert_eq!(occupied.team,0);assert_eq!(occupied.native_slot,Some(slot));assert!(occupied.crew.is_some());assert_eq!(g.native_routes.borrow().pool.links[slot],crate::original_route_pool::Links::default());
 }
 #[test]fn takeover_remaps_a_pursuing_enemy_to_the_same_native_actor_after_web_identity_transfer(){
  let(mut g,carrier,actor)=field("jeep",33,13);g.spawn("grunt",1,850.,640.,9,None);let enemy=g.units.len()-1;g.units[enemy].order="attack".into();g.units[enemy].target=Some(actor);let ai=g.units.iter().position(|u|u.id==actor).unwrap();assert!(g.update_native_hardware_entry(ai));assert_eq!(g.units[enemy].target,Some(carrier));assert!(g.events.iter().any(|e|e["kind"]=="crewed"&&e["unitId"]==carrier&&e["memberIds"][0]==actor));
 }
 #[test]fn direct_fixed_gun_exit_keeps_the_physical_actor_and_restores_the_cold_grunt_profile(){
  for (kind,definition,equipment,saved_class) in [("cannon",5,0,2),("missile",21,8,4),("gatling",22,11,5),("howitzer",23,12,6)]{
   for mp in [false,true]{
    let(mut g,_,_)=field(kind,definition,equipment);g.multiplayer=mp;g.original_rng.multiplayer=mp;
    g.spawn(kind,0,960.,960.,90,None);let i=g.units.len()-1;g.maintain_native_group_rosters();
    let carrier=g.units[i].id;let slot=g.units[i].native_slot.unwrap();let w=g.units[i].native_hardware_slot.unwrap();
    let generation=g.native_upstream_reactions.generation(slot).unwrap();let group=g.native_group_rosters.group_slot(0,90).unwrap();
    assert!(g.units[i].transport_actor.is_none());assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(saved_class));
    g.native_group_rosters.observe_unit_context(slot);g.units[i].native_order5c=13;g.units[i].native_flags9c|=32;
    assert!(g.unload_native_hardware(i));assert_eq!(g.units[i].team,-1);assert_eq!(g.native_hardware_pool[w].live_marker,-1);
    let robot=g.units.iter().find(|u|u.team==0&&u.native_slot==Some(slot)).unwrap();
    assert_eq!((robot.kind.as_str(),robot.native_class7c,robot.native_phase35,robot.native_range4),("grunt",0,1,Some(30)));
    assert_ne!(robot.id,carrier);assert_eq!((robot.x,robot.y),(960.,980.));
    assert_eq!(g.native_upstream_reactions.generation(slot),Some(generation),"{kind} must not run the allocator again (MP={mp})");
    assert!(g.native_cranes.bound(slot,generation,robot.id).is_some());
    assert_eq!(g.native_group_rosters.group_slot(0,90),Some(group));
    assert_eq!(&g.native_group_rosters.pool[group].roster[..8],&[0x8c,0x9b,0x61,0,0x8c,0x9b,0x61,0]);
    assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(0));
   }
  }
 }
 #[test]fn cannon_takeover_caches_and_unload_restores_the_original_infantry_group_profile(){
  for (kind,gid,range) in [("grunt",2u32,30),("tough",3,30),("sniper",4,36)]{for mp in [false,true]{
   let(mut g,carrier,actor)=field_with_driver("cannon",5,0,kind);g.multiplayer=mp;g.original_rng.multiplayer=mp;
   let ai=g.units.iter().position(|u|u.id==actor).unwrap();let ci=g.units.iter().position(|u|u.id==carrier).unwrap();
   let slot=g.units[ai].native_slot.unwrap();let class=g.units[ai].native_class7c;let old=g.native_group_rosters.group_slot(0,7).unwrap();
   let descriptor=(0x619ae8+82*gid).to_le_bytes();g.native_group_rosters.pool[old].roster[..4].copy_from_slice(&descriptor);
   g.units[ai].native_range4=Some(range);
   assert!(g.update_native_hardware_entry(ai));let squad=g.units[ci].squad;
   let group=g.native_group_rosters.group_slot(0,squad).unwrap();assert_ne!(group,old);
   assert_eq!(&g.native_group_rosters.pool[group].roster[4..8],&descriptor,"mounted {kind} profile (MP={mp})");
   assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(class));
   assert!(g.unload_native_hardware(ci));
   let robot=&g.units[ai];assert_eq!((robot.id,robot.kind.as_str(),robot.native_slot,robot.native_class7c,robot.native_range4),(actor,kind,Some(slot),class,Some(range)));
   assert_eq!(&g.native_group_rosters.pool[group].roster[..4],&descriptor);
   assert_eq!(&g.native_group_rosters.pool[group].roster[4..8],&descriptor);
   assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(class));
  }}
 }
 #[test]fn repeated_unload_and_recrew_of_each_fixed_gun_preserves_the_neutral_origin_and_actor_slot(){
  for(kind,definition,eq)in[("cannon",5,0),("missile",21,9),("gatling",22,11),("howitzer",23,12)]{
   let(mut g,carrier,actor)=field(kind,definition,eq);let ai=g.units.iter().position(|u|u.id==actor).unwrap();let ci=g.units.iter().position(|u|u.id==carrier).unwrap();let slot=g.units[ai].native_slot;g.units[ai].native_range4=Some(36);g.units[ai].native_target46=0x5501;g.units[ai].native_trait7a=-37;g.units[ai].native_cyclea0=3;g.units[ai].original_combat.as_mut().unwrap().fire_rate=87;g.units[ai].original_combat.as_mut().unwrap().projectile_speed=Some(19);
   for _ in 0..4{assert!(g.update_native_hardware_entry(ai),"{kind}");let occupied=(g.units[ci].x,g.units[ci].y);assert_eq!(g.units[ci].native_trait7a,-37);assert_eq!(g.units[ci].native_cyclea0,3);let hardware=g.units[ci].native_hardware_slot.unwrap();let group=g.native_group_rosters.group_slot(0,g.units[ci].squad);g.units[ci].native_flags9c|=32;g.units[ci].native_order5c=13;assert!(g.unload_native_hardware(ci));assert_eq!((g.units[ci].x,g.units[ci].y),(640.,640.),"{kind}");assert_eq!(g.units[ci].team,-1);assert!(g.units[ci].crew.is_none());assert_eq!(g.native_hardware_pool[hardware].live_marker,-1);assert_eq!(g.units[ai].native_slot,slot);assert_eq!((g.units[ai].x,g.units[ai].y),(occupied.0,occupied.1+20.));assert_eq!(g.units[ai].hp,17.);assert_eq!(g.units[ai].grenades,3);assert_eq!(g.units[ai].native_target46,0x5501);assert_eq!(g.units[ai].native_trait7a,-37);assert_eq!(g.units[ai].native_cyclea0,3);assert_eq!(g.units[ai].native_range4,Some(36));assert_eq!(g.units[ai].native_order5c,0);assert_eq!(g.units[ai].native_flags9c&32,0);assert_eq!(g.units[ai].original_combat.as_ref().unwrap().fire_rate,87);assert_eq!(g.units[ai].original_combat.as_ref().unwrap().projectile_speed,Some(19));assert_eq!(g.native_group_rosters.group_slot(0,g.units[ai].squad),group);assert_eq!(g.units[ai].native_initiative_scan.excluded9d,g.native_object_slot(carrier).unwrap());g.command(&json!({"kind":"enter","ids":[actor],"target":carrier}));}
  }
 }
 #[test]fn fixed_gun_unload_on_code16_consumes_the_command_without_creating_a_robot_or_freeing_hardware(){
  let(mut g,carrier,actor)=field("gatling",22,11);let ai=g.units.iter().position(|u|u.id==actor).unwrap();let ci=g.units.iter().position(|u|u.id==carrier).unwrap();assert!(g.update_native_hardware_entry(ai));let cell=(g.units[ci].y/32.).floor()as usize*g.map.width+(g.units[ci].x/32.).floor()as usize;g.map.terrain_codes[cell]=16;g.units[ci].native_order5c=13;g.units[ci].native_flags9c|=32;let hardware=g.units[ci].native_hardware_slot;assert!(g.unload_native_hardware(ci));assert_eq!(g.units[ci].native_hardware_slot,hardware);assert_eq!(g.units[ci].team,0);assert!(g.units[ci].crew.is_some());assert_eq!(g.units[ci].native_order5c,0);assert_eq!(g.units[ci].native_flags9c&32,0);assert!(g.units[ai].hp<0.);assert!(g.native_object_slot(carrier).is_none());
 }
 #[test]fn inactive_driver_identity_survives_real_callbacks_until_hardware_unload(){
  let(mut g,carrier,actor)=field("gatling",22,11);let ai=g.units.iter().position(|u|u.id==actor).unwrap();g.units[ai].native_range4=Some(36);let slot=g.units[ai].native_slot;assert!(g.update_native_hardware_entry(ai));g.spawn("cannon",1,1800.,1800.,90,None);g.units.last_mut().unwrap().cooldown=10000.;for _ in 0..3{g.step(0.059);}let ci=g.units.iter().position(|u|u.id==carrier).unwrap();assert!(g.units.iter().any(|u|u.id==actor&&u.hp<0.&&u.native_slot.is_none()));assert!(g.unload_native_hardware(ci));let robot=g.units.iter().find(|u|u.id==actor).unwrap();assert_eq!(robot.native_slot,slot);assert_eq!(robot.native_range4,Some(36));assert_eq!(robot.hp,17.);
 }
 #[test]fn ordinary_vehicle_unload_parks_through_the_original_object_pool_and_staggers_only_its_driver(){
  let(mut g,carrier,actor)=field("medium",8,1);let ai=g.units.iter().position(|u|u.id==actor).unwrap();let ci=g.units.iter().position(|u|u.id==carrier).unwrap();let slot=g.units[ai].native_slot;assert!(g.update_native_hardware_entry(ai));let hardware=g.units[ci].native_hardware_slot.unwrap();g.native_hardware_pool[hardware].attributes.bytes[39]=18;g.units[ci].native_order5c=13;g.units[ci].native_flags9c|=32;g.multiplayer=true;assert!(g.unload_native_hardware(ci));assert_eq!((g.units[ci].x,g.units[ci].y),(640.,640.));assert_eq!(g.units[ci].team,-1);assert_eq!(g.native_hardware_pool[hardware].live_marker,-1);assert!(g.native_object_slot(carrier).is_some());let group=g.native_transports.iter().find(|p|p.carrier==carrier).unwrap();assert_eq!(group.members.len(),1);assert_eq!(group.members[0].id,actor);assert_eq!(g.units[ai].native_phase35,64);assert_eq!(g.units[ai].native_slot,slot);assert_eq!(g.units[ai].native_range4,Some(36));assert_eq!(g.units[ai].native_order5c,0);assert_eq!(g.units[ai].native_flags9c&32,0);assert_eq!(g.units[ai].hp,17.);assert_eq!(g.units[ai].grenades,3);let native_group=g.native_group_rosters.group_slot(0,g.units[ai].squad);for _ in 0..40{g.update_native_transport(ai);}assert!(g.units[ai].native_transport.is_none());assert_eq!(g.native_group_rosters.group_slot(0,g.units[ai].squad),native_group);
 }
}
