//! Game adapters for the verified APC roster/portal/animation state machine.
//! Robot IDs, health and squad stay alive while hidden. Hidden coordinates stay
//! at the portal instead of native sentinelX10000, keeping Web snapshots bounded.
use super::*;
use super::original_transport::{self as transport,Boarding,Event,Phase};

impl Game{
 ///41eb30 retires only native slots whose persistent signedB5 names this
 ///OBJECT and whose raw phase62/8a20 marks an occupant. It is direct58ec0.
 pub(crate) fn remove_native_apc_object_occupants(&mut self,object:u8){
  let mut removed=Vec::new();
  for slot in 0..240{if i16::from(self.native_group_rosters.contact_claim(slot))!=i16::from(object){continue;}let Some(i)=self.units.iter().position(|u|u.native_slot==Some(slot))else{continue;};let u=self.units[i].clone();let flags=self.native_group_rosters.members[slot].as_ref().map_or(0,|m|m.flags8a);if u.native_phase35!=62&&flags&32==0{continue;}
   let inside=u.native_phase35==62||u.native_transport.as_ref().is_some_and(|a|a.phase==Phase::Inside);let point=[if inside{10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16];if !self.native_direct_removal(i,point,false,false){continue;}removed.push(u.id);let v=&mut self.units[i];v.native_transport=None;v.native_transport_slot=None;v.transport_carrier=None;
   if let Some(actor)=u.transport_actor{if let Some(a)=self.units.iter_mut().find(|v|v.id==actor){a.hp=-1.;a.native_slot=None;a.native_transport=None;a.native_transport_slot=None;a.transport_carrier=None;removed.push(actor);}}
  }
  for group in &mut self.native_transports{group.members.retain(|m|!removed.contains(&m.id));}self.native_transports.retain(|g|!g.members.is_empty());
 }
 ///58730 removes live native FOLLOWERS in roster order before58ec0 removes
 ///the APC leader. It allocates corpse records directly, including hidden
 ///X10000 passengers, rather than recursively dispatching full death effects.
 pub(crate) fn kill_native_apc_passengers(&mut self,index:usize,suppressed:bool,no_random:bool){
  let Some(carrier)=self.units.get(index).cloned()else{return;};
  let group=carrier.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).map(|m|usize::from(m.group));
  let slots=group.and_then(|g|self.native_group_rosters.pool.get(g)).map(|g|(1..21).filter_map(|role|g.member(role)).map(usize::from).collect::<Vec<_>>()).unwrap_or_default();
  for slot in slots{
   if !self.native_group_rosters.members.get(slot).and_then(Option::as_ref).is_some_and(|m|m.flags31&4!=0){continue;}
   let Some(i)=self.units.iter().position(|u|u.native_slot==Some(slot))else{continue;};let passenger=self.units[i].clone();
   self.native_group_rosters.clear_contact_claim(slot);
   if let Some(member)=self.native_group_rosters.members[slot].as_mut(){member.flags31&=!4;}
   let raw_x=if passenger.native_transport.as_ref().is_some_and(|a|a.phase==Phase::Inside){10000}else{(passenger.x/2.).floor()as i16};
   let point=[raw_x,(passenger.y/2.).floor()as i16];self.native_deaths.allocate(point,passenger.native_flags64,suppressed,no_random,||self.original_rng.next_u32());
   let u=&mut self.units[i];u.hp=-1.;u.native_hardware=None;u.native_transport=None;u.transport_carrier=None;u.native_transport_slot=None;u.path.clear();u.original_motion=None;u.native_route_pending.set(None);
  }
  //The retained browser driver duplicates the transformed native leader; it
  //has no second native removal/corpse allocation of its own.
  if let Some(actor)=carrier.transport_actor.and_then(|id|self.units.iter().position(|u|u.id==id)){
   let u=&mut self.units[actor];u.hp=-1.;u.native_slot=None;u.native_transport_slot=None;u.native_transport=None;u.transport_carrier=None;
  }
  self.native_transports.retain(|g|g.carrier!=carrier.id);self.native_transport_commands.retain(|(id,_)|*id!=carrier.id);self.units[index].transport_actor=None;
 }
 fn native_transport_order_valid(&self,index:usize,command:&Value)->bool{
  let unit=&self.units[index];let kind=command["kind"].as_str().unwrap_or("move");
  let target=command["target"].as_u64().and_then(|id|usize::try_from(id).ok());
  if command.get("target").is_some_and(|v|!v.is_null())&&target.is_none(){return false;}
  match kind{
   "move"|"stop"|"attack"=>true,
   "attackBuilding"=>target.is_some_and(|id|self.buildings.iter().any(|b|b.id==id&&b.team!=unit.team&&b.alive(self.map.original))),
   "attackWall"=>walls::capable(unit)&&target.and_then(|id|self.wall(id)).is_some(),
   "attackBridge"=>command["x"].as_f64().zip(command["y"].as_f64()).and_then(|(x,y)|self.bridge_at_point(x,y)).is_some_and(|id|self.map.bridge_hp[id]>0.&&walls::capable(unit)),
   "enter"=>target.is_some_and(|id|self.units.iter().any(|u|u.id==id&&u.team<0&&u.crew.is_none()&&stats(&u.kind).cls>0&&original_combat::alive(true,u.hp))),
   "board"=>target.is_some_and(|id|self.units.iter().any(|u|u.id==id&&u.team==unit.team&&u.kind=="apc"&&original_combat::alive(true,u.hp)&&self.apc_has_passenger_room(u))),
   _=>false,
  }
 }
 ///00454070/540b0/54160 allow cancellation before anyone is inside, and
 /// postpone a group command while the visible/inside roster is mixed.
 /// Filter before formation preparation so hidden IDs cannot split the squad.
 pub(crate) fn native_transport_command_ids(&mut self,ids:&[usize],team:i32,command:&Value)->Vec<usize>{
  if !self.map.original{return ids.to_vec();}
  let mut accepted=Vec::new();
  for &id in ids{
   let Some(i)=self.units.iter().position(|u|u.id==id&&u.team==team&&original_combat::alive(true,u.hp))else{continue;};
   let Some(carrier)=self.units[i].transport_carrier else{accepted.push(id);continue;};
   if !self.native_transport_order_valid(i,command){continue;}
   let Some(gi)=self.native_transports.iter().position(|g|g.carrier==carrier)else{continue;};
   let group=&self.native_transports[gi];
   // The occupied vehicle is separately addressable; its retained driver and
   // passengers are not independent command recipients while inside.
   if group.occupant.is_some(){continue;}
   let cancellable=group.members.iter().all(|m|matches!(m.animation.phase,Phase::Approach|Phase::Entering));
   if cancellable{
    let group=self.native_transports.remove(gi);
    self.native_transport_commands.retain(|(c,_)|*c!=carrier);
    for m in group.members{if let Some(mi)=self.transport_member_index(m.id){self.clear_transport_member(mi);self.finish_order(mi);}}
    accepted.push(id);
   }else{
    self.native_transport_commands.retain(|(c,_)|*c!=carrier);
    self.native_transport_commands.push((carrier,command.clone()));
   }
  }
  accepted
 }
 pub(crate) fn apc_has_passenger_room(&self,unit:&Unit)->bool{
  if self.map.original{unit.cargo.len()+usize::from(unit.crew.is_some())<transport::CAPACITY}else{unit.cargo.len()<6}
 }
 fn take_native_transport_command(&mut self,carrier:usize,ids:&[usize])->Option<Value>{
  let i=self.native_transport_commands.iter().position(|(c,_)|*c==carrier)?;
  let(_,mut command)=self.native_transport_commands.remove(i);command["ids"]=json!(ids);Some(command)
 }
 ///00459d80 releases the special group routes. Its failed-arrival companion
 ///00459e60 additionally destroys members already marked8a&20 inside.
 pub(crate) fn cancel_native_transport(&mut self,index:usize,kill_inside:bool)->bool{
  let carrier=self.units[index].transport_carrier.unwrap_or(self.units[index].id);
  let Some(gi)=self.native_transports.iter().position(|g|g.carrier==carrier&&g.occupant.is_none())else{return false;};
  let group=self.native_transports.remove(gi);self.native_transport_commands.retain(|(c,_)|*c!=carrier);
  for member in group.members{if let Some(i)=self.transport_member_index(member.id){if kill_inside&&member.animation.phase==Phase::Inside{self.units[i].hp=-1.;if let Some(slot)=self.units[i].native_slot.or(self.units[i].native_transport_slot){self.native_group_rosters.clear_contact_claim(slot);}}self.clear_transport_member(i);self.finish_order(i);}}
  true
 }
 ///59d80 as used by54160: only the actor's own21 roster and raw8a10
 ///members participate. This does not finish or replace the accepted order.
 pub(crate) fn cleanup_native_order_transports(&mut self,index:usize){
  let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return;};let Some(actor)=self.native_group_rosters.members.get(slot).and_then(Option::as_ref)else{return;};let group=usize::from(actor.group);if group>=240{return;}
  let members=(0..21).filter_map(|role|self.native_group_rosters.pool[group].member(role)).map(usize::from).filter(|&slot|self.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.flags8a&16!=0)).collect::<Vec<_>>();let mut cleared=vec![];
  for slot in members{let Some(i)=self.units.iter().position(|u|u.native_slot==Some(slot))else{continue;};let m=self.native_group_rosters.members[slot].as_ref().unwrap().clone();
   if m.role<0&&m.contact_order!=0{if let Some(target)=self.native_group_rosters.contact_target(slot){if self.native_group_apc_target(i,target)==Some(target){self.clear_native_apc_object_contact(target);}}}
   //The52500 call precedes clearing8a18 and releases both finite lists.
   self.clear_native_owned_paths(i);
   self.native_group_rosters.record_secondary_path(slot,false);if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.primary_path=false;m.secondary_path=false;m.flags8a&=!0x18;}
   let u=&mut self.units[i];u.path.clear();u.native_secondary_route.set(false);u.native_route_pending.set(None);u.original_motion=None;u.native_transport=None;u.transport_carrier=None;cleared.push(u.id);
  }
  for group in &mut self.native_transports{group.members.retain(|m|!cleared.contains(&m.id));}self.native_transports.retain(|g|!g.members.is_empty());
 }
 ///59e60 checks the own roster's8a20, but passes the input ACTOR to58ec0.
 ///Subsequent matches call the same retired actor, whose31:4 guard returns.
 ///No follower removal or synthetic familyFF conversion is substituted.
 pub(crate) fn remove_native_failed_transport_actor(&mut self,index:usize)->bool{
  let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return false;};let Some(actor)=self.native_group_rosters.members.get(slot).and_then(Option::as_ref)else{return false;};let group=usize::from(actor.group);if group>=240{return false;}let inside=(0..21).filter_map(|role|self.native_group_rosters.pool[group].member(role)).any(|s|self.native_group_rosters.members[usize::from(s)].as_ref().is_some_and(|m|m.flags8a&32!=0));inside&&self.native_remove_actor(index,false,false)
 }
 fn transport_member_index(&self,id:usize)->Option<usize>{self.units.iter().position(|u|u.id==id)}
 fn clear_transport_member(&mut self,index:usize){
  if let Some(slot)=self.units[index].native_slot.or(self.units[index].native_transport_slot){self.native_group_rosters.record_transport_flags(slot,0,0x18);}
  let u=&mut self.units[index];u.native_transport=None;u.transport_carrier=None;
  if u.native_slot.is_none(){u.native_slot=u.native_transport_slot;}u.native_transport_slot=None;
 }
 fn transport_direction_clear(&self,vehicle:&Unit,kind:&str,direction:u8,entering:bool)->bool{
  let origin=((vehicle.x/2.).floor()as i32,(vehicle.y/2.).floor()as i32);
  if self.map.native_road_graph.is_some(){
   let probe=if entering{[(origin.0 as i16).wrapping_add(8),(origin.1 as i16).wrapping_add(8)]}else{[origin.0 as i16,origin.1 as i16]};let base=((probe[1]as u16&0xfff0).wrapping_mul(8)).wrapping_add((probe[0]>>4)as u16)as i16;
   let blocked=transport::footprint_blocked(base,direction,self.map.world==1,|cell|self.transport_terrain_code(cell));
   return !blocked&&(entering||self.native_first_object_contact([origin.0 as i16,origin.1 as i16]).is_none());
  }
  let p=if entering{transport::entry_portal(origin,direction)}else{transport::exit_portal(transport::parked_origin(origin),direction)};
  self.navigation_point_clear(kind,f64::from(p.0)*2.,f64::from(p.1)*2.)
 }
 fn transport_terrain_code(&self,cell:i16)->u8{let cell=cell as u16 as usize;let(x,y)=(cell&127,cell>>7);if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(0)}else{self.map.native_terrain_plane.get(cell).copied().unwrap_or(0)}}
 fn transport_unload_direction(&self,vehicle:&Unit,kind:&str)->Option<u8>{let heading=original_combat::facing(vehicle.angle);if self.map.native_road_graph.is_some(){let origin=[(vehicle.x/2.).floor()as i16,(vehicle.y/2.).floor()as i16];transport::choose_exit(origin,heading,self.map.world==1,|cell|self.transport_terrain_code(cell),|point|self.native_first_object_contact(point).is_some())}else{transport::choose_direction(heading,|d|self.transport_direction_clear(vehicle,kind,d,false))}}
 fn start_native_transport(&mut self,index:usize)->bool{
  let u=self.units[index].clone();
  if !self.map.original||stats(&u.kind).cls!=0||!matches!(u.order.as_str(),"enter"|"board"){return false;}
  let Some(carrier)=u.target.and_then(|id|self.units.iter().find(|v|v.id==id&&v.kind=="apc"&&v.team<0&&v.crew.is_none()&&original_combat::alive(true,v.hp))).cloned()else{return false;};
  if self.map.native_road_graph.is_some()&&self.native_object_slot(carrier.id).and_then(|slot|self.native_world_scan_objects().into_iter().find(|o|o.slot==slot)).is_none_or(|o|o.sprite!=0x1435){return false;}
  //00450890 measures to object origin+8,+8 and accepts the exact48-native
  // boundary. The browser stores both neutral anchors in doubled pixels.
  let dx=(u.x/2.).floor()as i32-((carrier.x/2.).floor()as i32+8);
  let dy=(u.y/2.).floor()as i32-((carrier.y/2.).floor()as i32+8);
  if dx*dx+dy*dy>48*48{return false;}
  if self.native_transports.iter().any(|g|g.carrier==carrier.id){self.finish_order(index);return true;}
  let Some(direction)=transport::choose_direction(original_combat::facing(u.angle),|d|self.transport_direction_clear(&carrier,&u.kind,d,true))else{self.finish_order(index);return true;};
  let mut members=self.units.iter().filter(|v|v.team==u.team&&v.squad==u.squad&&stats(&v.kind).cls==0&&original_combat::alive(true,v.hp)&&v.transport_carrier.is_none()&&!workshops::hidden(v))
   .map(|v|(v.native_slot.unwrap_or(v.id),v.id)).collect::<Vec<_>>();members.sort_unstable();members.truncate(transport::CAPACITY);let ids=members.into_iter().map(|(_,id)|id).collect::<Vec<_>>();
  let origin=((carrier.x/2.).floor()as i32,(carrier.y/2.).floor()as i32);let Ok(group)=Boarding::new(carrier.id,u.squad,origin,direction,&ids)else{return false;};
  let object_slot=self.native_object_slot(carrier.id);
  if let Some(object)=object_slot{self.set_native_object_sprite_offset(object,(direction*5)as i8);if matches!(direction,0|1|7){if let Some((frame,motion))=self.native_object_animation(object){if frame!=4&&motion!=-1{self.set_native_object_animation(object,frame,1);}}}}
  let p=group.portal();let goal=(f64::from(p.0)*2.,f64::from(p.1)*2.);
  for member in &group.members{if let Some(i)=self.transport_member_index(member.id){
   let path=self.route_unit(&self.units[i],goal.0,goal.1);if let Some((slot,object))=self.units[i].native_slot.zip(object_slot){self.native_group_rosters.record_contact_target(slot,object);self.native_group_rosters.record_transport_flags(slot,0x18,0);if member.id==u.id{self.native_group_rosters.record_contact_claim(slot,object);}}
   let v=&mut self.units[i];
   v.native_transport=Some(member.animation.clone());v.native_transport_slot=v.native_slot;v.transport_carrier=Some(carrier.id);
   v.order="enter".into();v.target=Some(carrier.id);v.dest=Some(goal);v.path=path;v.original_motion=None;v.native_fire=None;v.native_idle=None;v.native_cover=None;v.native_walk=None;v.native_contact=None;v.formation=None;v.motion_bonus=0;v.hold_position=false;v.explicit_stop=false;v.native_phase35=2;
   if member.id!=u.id{v.initiative=None;v.march_resume=None;}
  }}
  self.native_transports.push(group);true
 }
 fn occupy_native_transport(&mut self,group:&Boarding,actor:usize){
  let Some(ai)=self.transport_member_index(actor)else{return;};let Some(ci)=self.transport_member_index(group.carrier)else{return;};
  let driver=self.units[ai].clone();let point=group.vehicle_origin();
  let object=self.native_object_slot(group.carrier);let input=object.and_then(|o|self.native_object_hardware_input(o));let mut source_hardware=None;
  if let Some(input)=input{if let Some(slot)=self.allocate_native_takeover_record(ai){let old=driver.original_combat.as_ref();let rate=old.map_or(80,|c|c.fire_rate);let speed=old.and_then(|c|c.projectile_speed).unwrap_or_else(||super::native_ai_scoring::default_input(&driver.kind).map_or(15,|i|i.projectile_speed));let controls=self.native_hardware_pool[slot].attributes.take_over(input,rate,speed);let range=super::native_ai_scoring::default_input(&driver.kind).map_or(15,|i|i.range);self.native_hardware_pool[slot].attributes.initialize_apc(range);let mut order=self.native_hardware_pool[slot].order;order.selected_target12=255;order.field17=0;self.native_hardware_pool[slot].set_order(order);source_hardware=Some((slot,self.native_hardware_pool[slot].attributes,controls.double_range as i8));}}
  if source_hardware.is_some(){if let Some(slot)=driver.native_slot{self.record_native_selection_saved_class(slot,driver.native_class7c);}}
  if let Some(object)=object{if let Some(o)=self.native_world_scan_objects().into_iter().find(|o|o.slot==object){self.apply_native_ambient_object(object,[-1,o.position[1]],o.flags13);}}
  let cargo=group.members.iter().filter(|m|m.id!=actor).filter_map(|m|self.units.iter().find(|u|u.id==m.id&&original_combat::alive(true,u.hp))).map(|u|Crew{kind:u.kind.clone(),hp:u.hp}).collect::<Vec<_>>();
  self.finish_order(ci);
  let c=&mut self.units[ci];c.x=f64::from(point.0)*2.;c.y=f64::from(point.1)*2.;c.native_bucket38=super::original_initiative::native::coarse_cell([point.0 as i16,point.1 as i16]);c.team=driver.team;c.squad=driver.squad;c.native_slot=driver.native_slot;c.transport_actor=Some(actor);c.native_route_flip.set(driver.native_route_flip.get());c.native_route_road_replaced.set(driver.native_route_road_replaced.get());c.native_secondary_route.set(driver.native_secondary_route.get());
  c.crew=Some(Crew{kind:driver.kind.clone(),hp:driver.hp});c.cargo=cargo;c.grenades=driver.grenades;c.native_hardware=None;c.angle=native_math::angle(group.direction);c.aim=native_math::angle(0);c.cooldown=0.;c.native_reload7b=0;c.explicit_stop=false;c.native_phase35=56;c.native_class7c=1;c.native_flags64=driver.native_flags64;c.native_flags9c=driver.native_flags9c;c.native_control06=driver.native_control06;c.native_order5c=0;c.native_target46=driver.native_target46;c.native_trait7a=driver.native_trait7a;c.native_cyclea0=driver.native_cyclea0;c.native_initiative_scan=driver.native_initiative_scan;
  let mut state=driver.original_combat.clone().unwrap_or_else(||original_combat::CombatState::new(&driver.kind));
  let equipment=original_combat::CombatState::new("apc");state.fire_rate=equipment.fire_rate;state.movement_control=equipment.movement_control;state.projectile_speed=Some(15);state.secondary_state=-1;c.original_combat=Some(state);
  if let Some((slot,attrs,range4))=source_hardware{c.native_hardware_slot=Some(slot);c.native_range4=Some(range4);c.hp=f64::from(attrs.word(8));c.max_hp=f64::from(attrs.word(10));if let Some(s)=&mut c.original_combat{s.fire_rate=attrs.byte(32);s.movement_control=attrs.byte(7);s.projectile_speed=Some(attrs.byte(33));}}
  self.units[ai].native_slot=None;
  self.units[ai].native_hardware_slot=None;
  for member in &group.members{if let Some(slot)=self.units.iter().find(|u|u.id==member.id).and_then(|u|u.native_slot.or(u.native_transport_slot)){self.native_group_rosters.record_transport_flags(slot,0,0x18);self.native_group_rosters.record_contact_target(slot,0);}}
  if driver.initiative==Some(original_initiative::Initiative::Vehicle(group.carrier)){self.resume_march(ci,driver.march_resume);}
  self.events.push(json!({"kind":"crewed","team":driver.team,"type":"apc","unitId":group.carrier,"memberIds":group.members.iter().map(|m|m.id).collect::<Vec<_>>()}));
 }
 /// True while this callback belongs to approach/boarding/inside/emergence.
 pub(crate) fn update_native_transport(&mut self,index:usize)->bool{
  if !self.map.original{return false;}
  if self.units[index].kind=="apc"&&self.units[index].transport_actor.is_some(){return self.native_transports.iter().any(|g|g.carrier==self.units[index].id&&g.occupied_tick==Some(self.motion_tick));}
  if self.units[index].native_transport.is_none()&&!self.start_native_transport(index){return false;}
  let Some(carrier)=self.units[index].transport_carrier else{return false;};
  let Some(gi)=self.native_transports.iter().position(|g|g.carrier==carrier)else{self.clear_transport_member(index);return false;};
  let mut group=self.native_transports.remove(gi);let id=self.units[index].id;
  let phase=group.animation(id).map(|a|a.phase);
  if phase==Some(Phase::Approach){
   let p=group.portal();let goal=(f64::from(p.0)*2.,f64::from(p.1)*2.);
   if (self.units[index].x-goal.0).abs()+(self.units[index].y-goal.1).abs()<=1.{
    self.units[index].x=goal.0;self.units[index].y=goal.1;self.units[index].path.clear();self.units[index].original_motion=None;self.units[index].moving=false;group.arrive(id);self.units[index].native_phase35=61;
    if let Some(slot)=self.units[index].native_slot.or(self.units[index].native_transport_slot){if let Some(object)=self.native_group_rosters.contact_target(slot).or_else(||self.native_object_slot(carrier)){self.native_group_rosters.record_contact_claim(slot,object);}}
   }else{
    // Group approach is locked to the portal until all members are inside.
    // A new ordinary march is resumed by the carrier after the squad boards.
    if self.units[index].order=="move"{self.units[index].march_resume=self.units[index].dest;self.units[index].initiative=Some(original_initiative::Initiative::Vehicle(carrier));}
    self.units[index].order="enter".into();self.units[index].target=Some(carrier);self.units[index].dest=Some(goal);
    let pending=self.units[index].native_route_pending.get().is_some_and(|old|(old.0-goal.0).abs()+(old.1-goal.1).abs()<=1.);
    if self.units[index].path.is_empty()||!pending&&self.units[index].path.back().is_none_or(|old|(old.0-goal.0).abs()+(old.1-goal.1).abs()>1.){self.units[index].path=self.route_unit(&self.units[index],goal.0,goal.1);self.units[index].original_motion=None;}
    self.move_original(index,self.motion_tick,1,true);
   }
   self.units[index].native_transport=group.animation(id).cloned();self.native_transports.insert(gi,group);return true;
  }
  let position=((self.units[index].x/2.).floor()as i32,(self.units[index].y/2.).floor()as i32);
  let event=group.advance(id,position,||self.native_random());self.units[index].native_transport=group.animation(id).cloned();self.units[index].moving=false;
  match event{
   Event::Occupy{member}=>{group.occupied_tick=Some(self.motion_tick);self.occupy_native_transport(&group,member);},
   Event::Hidden{..}=>{self.units[index].path.clear();self.units[index].native_fire=None;self.units[index].native_walk=None;self.units[index].native_phase35=62;if let Some(slot)=self.units[index].native_slot.or(self.units[index].native_transport_slot){self.native_group_rosters.record_transport_inside(slot,true);}},
   Event::Released{goal,..}=>{
    if let Some(slot)=self.units[index].native_slot.or(self.units[index].native_transport_slot){self.native_group_rosters.record_transport_inside(slot,false);if let Some(member)=self.native_group_rosters.members[slot].as_mut(){member.flags69|=4;}}
    self.clear_transport_member(index);let point=(f64::from(goal.0)*2.,f64::from(goal.1)*2.);let path=self.route_unit(&self.units[index],point.0,point.1);
    let u=&mut self.units[index];u.order="move".into();u.target=None;u.dest=Some(point);u.path=path;u.original_motion=None;u.native_fire=None;u.hold_position=false;u.explicit_stop=false;u.native_phase35=2;
    if let Some(state)=u.original_combat.as_mut(){state.movement_control=original_combat::CombatState::new(&u.kind).movement_control;}
    if group.members.iter().all(|m|m.animation.phase==Phase::Released){if let Some(object)=self.native_object_slot(carrier){self.clear_native_apc_object_contact(object);}}
   },Event::None=>{}
  }
  let occupied=matches!(event,Event::Occupy{..});self.native_transports.insert(gi,group);
  if occupied{if let Some(command)=self.take_native_transport_command(carrier,&[carrier]){self.command(&command);}}
  true
 }
 pub(crate) fn maintain_native_transports(&mut self){
  if !self.map.original{return;}
  let groups=std::mem::take(&mut self.native_transports);let mut commands=Vec::new();
  for mut group in groups{
   let carrier=self.units.iter().find(|u|u.id==group.carrier).cloned();
   //00456f20 no longer queries the parked object: destroying it during the
   // staggered exit must not reveal every remaining passenger prematurely.
   let unloading=group.occupant.is_none()&&group.members.iter().any(|m|matches!(m.animation.phase,Phase::Exiting|Phase::Released));
   let valid=unloading||carrier.as_ref().is_some_and(|u|original_combat::alive(true,u.hp)&&if group.occupant.is_some(){u.team>=0&&u.crew.is_some()}else{u.team<0});
   if !valid{
    self.native_transport_commands.retain(|(c,_)|*c!=group.carrier);
    for m in &group.members{if let Some(i)=self.transport_member_index(m.id){if group.occupant.is_some()||m.animation.phase==Phase::Inside{self.units[i].hp=-1.;if let Some(slot)=self.units[i].native_slot.or(self.units[i].native_transport_slot){self.native_group_rosters.clear_contact_claim(slot);}}self.clear_transport_member(i);self.finish_order(i);}}
    continue;
   }
   group.members.retain(|m|self.units.iter().any(|u|u.id==m.id&&original_combat::alive(true,u.hp)));
   if !group.members.is_empty()&&!group.finished(){self.native_transports.push(group);}
   else if let Some(command)=self.take_native_transport_command(group.carrier,&group.members.iter().map(|m|m.id).collect::<Vec<_>>()){commands.push(command);}
  }
  for command in commands{self.command(&command);}
 }
 /// Handles original APC unloading, including legacy imported occupied carriers.
 /// A blocked exit consumes the order and leaves the occupied carrier intact.
 pub(crate) fn unload_native_apc(&mut self,index:usize)->bool{
  if !self.map.original||self.units[index].kind!="apc"{return false;}
  self.unload_native_vehicle(index)
 }
 ///53130 sends every non-fixed vehicle through59830/59720. Ordinary
 ///takeovers own a one-robot group; APCs retain their complete native roster.
 pub(crate) fn unload_native_vehicle(&mut self,index:usize)->bool{
  if !self.map.original||stats(&self.units[index].kind).cls!=1{return false;}
  //53130 consumes13 before59830 searches for an exit; failure still clearsit.
  self.units[index].native_flags9c&=!32;self.units[index].native_order5c=0;
  if let Some(slot)=self.units[index].native_slot{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags9c=self.units[index].native_flags9c;m.flagsb7&=!1;m.promotion.order=0;}}
  if let Some(group)=self.native_group_rosters.current_roster{self.native_group_rosters.pool[group].roster[58]=0;}
  let carrier=self.units[index].clone();let Some(driver)=carrier.crew.clone()else{return true;};
  let restored_range4=carrier.native_hardware_slot.and_then(|s|self.native_hardware_pool.get(s)).map(|w|w.attributes.byte(39).wrapping_mul(2)as i8).unwrap_or_else(||super::native_ai_scoring::default_input(&driver.kind).map_or(30,|i|i.range.wrapping_mul(2)as i8));
  let Some(direction)=self.transport_unload_direction(&carrier,&driver.kind)else{return true;};
  //58af0 frees W while the original native actor slot is still attached.
  //Renderer-only driver allocation below must not hide that live pointer.
  self.park_native_hardware_for_unload(index,&carrier);
  let gi=if let Some(gi)=self.native_transports.iter().position(|g|g.carrier==carrier.id&&g.occupant.is_some()){gi}else{
   // Ordinary conversions retain the original robot as an inactive Web
   //identity. Old saves lack it and need a single renderer allocation.
   self.units[index].native_slot=None;let mut ids=Vec::new();
   if let Some(i)=carrier.transport_actor.and_then(|id|self.transport_member_index(id)){let u=&mut self.units[i];u.hp=driver.hp;u.transport_carrier=Some(carrier.id);u.native_transport_slot=carrier.native_slot;ids.push(u.id);}
   else{for crew in std::iter::once(driver.clone()).chain(carrier.cargo.clone()).take(transport::CAPACITY){self.spawn(&crew.kind,carrier.team,carrier.x,carrier.y,carrier.squad,None);let u=self.units.last_mut().unwrap();u.hp=crew.hp;u.transport_carrier=Some(carrier.id);u.native_transport_slot=u.native_slot;ids.push(u.id);}}
   let origin=((carrier.x/2.).floor()as i32,(carrier.y/2.).floor()as i32);let Ok(mut group)=Boarding::new(carrier.id,carrier.squad,origin,direction,&ids)else{return true;};group.occupant=ids.first().copied();self.native_transports.push(group);self.native_transports.len()-1
  };
  let mut group=self.native_transports.remove(gi);let actor=group.occupant;let driver_class=actor.and_then(|id|self.units.iter().find(|u|u.id==id).map(|u|u.native_class7c)).unwrap_or_else(||original_combat::runtime_class(&driver.kind));
  self.finish_order(index);let parked=((self.units[index].x/2.).floor()as i32,(self.units[index].y/2.).floor()as i32);let portal=group.unload(parked,direction);
  let c=&mut self.units[index];c.x=f64::from(parked.0)*2.;c.y=f64::from(parked.1)*2.;c.hp=f64::from(transport::parked_hull(c.hp as i16));c.team=-1;c.crew=None;c.cargo.clear();c.native_slot=None;c.transport_actor=None;c.native_hardware=None;
  for m in &group.members{if let Some(i)=self.transport_member_index(m.id){
   if let Some(slot)=self.units[i].native_slot.or(self.units[i].native_transport_slot){self.native_group_rosters.record_transport_flags(slot,0,0x18);}
   self.finish_order(i);let u=&mut self.units[i];u.x=f64::from(portal.0)*2.;u.y=f64::from(portal.1)*2.;u.angle=native_math::angle(m.animation.direction);u.aim=u.angle;u.squad=carrier.squad;u.native_transport=Some(m.animation.clone());u.transport_carrier=Some(carrier.id);u.hold_position=false;u.explicit_stop=false;u.native_phase35=64;
   if Some(m.id)==actor{u.native_slot=carrier.native_slot.or(u.native_transport_slot);u.hp=driver.hp;u.grenades=carrier.grenades;u.native_class7c=driver_class;u.native_range4=Some(restored_range4);u.native_route_flip.set(carrier.native_route_flip.get());u.native_route_road_replaced.set(carrier.native_route_road_replaced.get());u.native_secondary_route.set(carrier.native_secondary_route.get());u.native_flags64=carrier.native_flags64;u.native_flags9c=carrier.native_flags9c;u.native_order5c=carrier.native_order5c;u.native_target46=carrier.native_target46;u.native_trait7a=carrier.native_trait7a;u.native_cyclea0=carrier.native_cyclea0;u.native_reload7b=carrier.native_reload7b;u.native_bucket38=carrier.native_bucket38;u.native_initiative_scan=carrier.native_initiative_scan;if let Some(state)=u.original_combat.as_mut(){if let Some(current)=carrier.original_combat.as_ref(){state.aim_skill=current.aim_skill;state.fire_rate=current.fire_rate;state.projectile_speed=current.projectile_speed;}}}
   else if u.native_slot.is_none(){u.native_slot=u.native_transport_slot;}
  }}
  if let Some((slot,actor))=carrier.native_slot.zip(actor){self.native_group_rosters.remap_entity(slot,carrier.id,actor);self.remap_native_pursuit_identity(carrier.id,actor);}
  self.events.push(json!({"kind":"unloaded","team":carrier.team,"unitId":carrier.id,"memberIds":group.members.iter().map(|m|m.id).collect::<Vec<_>>()}));
  self.native_transports.insert(gi,group);self.sync_native_world();if let Some(object)=self.native_object_slot(carrier.id){self.set_native_object_sprite_offset(object,(direction*5)as i8);if matches!(direction,0|1|7){self.set_native_object_animation(object,0,1);}}true
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,712);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;g.next_id=1;g.spawn("gatling",1,1800.,1200.,9,None);g.units[0].hold_position=true;g.units[0].cooldown=10000.;g}
 fn board(g:&mut Game)->(usize,Vec<usize>){g.spawn("apc",-1,640.,640.,2,None);let carrier=g.units.last().unwrap().id;let mut ids=Vec::new();for n in 0..3{g.spawn("grunt",0,640.+n as f64*8.,700.,3,None);let u=g.units.last_mut().unwrap();u.hp=20.+n as f64;ids.push(u.id);}g.command(&json!({"kind":"enter","ids":ids,"target":carrier}));(carrier,ids)}
 #[test]fn all_robots_board_before_ownership_changes_and_unload_restores_their_original_ids(){
  let mut g=field();let(carrier,ids)=board(&mut g);let old=ids.iter().map(|id|g.units.iter().find(|u|u.id==*id).unwrap().hp).collect::<Vec<_>>();let mut entered=false;
  for _ in 0..180{g.step(0.059);let c=g.units.iter().find(|u|u.id==carrier).unwrap();if c.team==0{entered=true;assert_eq!(c.cargo.len(),2);assert_eq!((c.x,c.y),(652.,652.));break;}}
  assert!(entered);assert_eq!(g.units.iter().find(|u|u.id==carrier).unwrap().native_phase35,56);assert!(ids.iter().all(|id|g.units.iter().any(|u|u.id==*id&&workshops::hidden(u)&&u.native_phase35==62)));
  g.command(&json!({"kind":"unload","ids":[carrier]}));let c=g.units.iter().find(|u|u.id==carrier).unwrap();assert_eq!(c.team,-1);assert!(c.crew.is_none());assert!(ids.iter().all(|id|g.units.iter().find(|u|u.id==*id).unwrap().native_phase35==64));
  for _ in 0..180{g.step(0.059);if g.native_transports.is_empty(){break;}}
  assert!(g.native_transports.is_empty());for(id,hp)in ids.iter().zip(old){let u=g.units.iter().find(|u|u.id==*id).unwrap();assert_eq!(u.hp,hp);assert_eq!(u.squad,3);assert!(!workshops::hidden(u));assert!(u.transport_carrier.is_none());}
 }
 #[test]fn destroying_the_occupied_carrier_removes_its_hidden_roster(){
  let mut g=field();let(carrier,ids)=board(&mut g);for _ in 0..180{g.step(0.059);if g.units.iter().any(|u|u.id==carrier&&u.team==0){break;}}
  g.units.iter_mut().find(|u|u.id==carrier).unwrap().hp=-1.;g.step(0.059);assert!(g.native_transports.is_empty());assert!(ids.iter().all(|id|g.units.iter().all(|u|u.id!=*id)));
 }
 fn occupied(g:&mut Game,carrier:usize){for _ in 0..240{g.step(0.059);if g.units.iter().any(|u|u.id==carrier&&u.team==0){return;}}panic!("squad never boarded APC");}
 fn authored_exit_field()->(Game,usize){let mut g=field();g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});g.map.native_terrain_plane=vec![0;16384];g.map.world=0;g.spawn("apc",0,640.,640.,2,None);let i=g.units.len()-1;g.import_native_world(&Value::Null);g.maintain_native_group_rosters();(g,i)}
 fn set_exit_code(g:&mut Game,cell:i16,code:u8){let cell=cell as u16 as usize;let(x,y)=(cell&127,cell>>7);g.map.native_terrain_plane[cell]=code;g.map.terrain_codes[y*g.map.width+x]=code;}
 #[test]fn authored_exit_probes_native_two_by_two_and_diagonal_third_column_instead_of_web_building_rectangles(){
  for direction in 0..8{for(probe,offset)in[0i16,1,128,129,2,130].into_iter().enumerate(){let(mut g,i)=authored_exit_field();let base=2580i16.wrapping_add(transport::FOOTPRINT_CELL_OFFSETS[direction]);set_exit_code(&mut g,base.wrapping_add(offset),32);assert_eq!(g.transport_direction_clear(&g.units[i],"grunt",direction as u8,false),probe>=4&&direction&1==0,"direction{direction},probe{probe}");}}
  let(mut g,i)=authored_exit_field();let cell=2580+transport::FOOTPRINT_CELL_OFFSETS[0];for code in[9,38,39,42]{set_exit_code(&mut g,cell,code);g.map.world=0;assert!(g.transport_direction_clear(&g.units[i],"grunt",0,false));g.map.world=1;assert!(!g.transport_direction_clear(&g.units[i],"grunt",0,false));}
  let(mut g,i)=authored_exit_field();let mut building=Game::new(0,1,712).buildings[0].clone();building.x=400.;building.y=400.;building.w=600.;building.h=600.;building.hp=100.;g.buildings.push(building);assert!(g.transport_direction_clear(&g.units[i],"grunt",0,false),"native457b60 reads BUILD terrain codes, not a rendered building rectangle");
 }
 #[test]fn authored_exit_uses_the_first_actual_apc_object_at_the_vehicle_origin_and_blocks_every_direction(){
  let(mut g,i)=authored_exit_field();g.objects=vec![json!({"type":31,"x":640,"y":640,"nativeFlags13":0x70}),json!({"type":31,"x":640,"y":640,"nativeFlags13":0x70})];g.import_native_world(&Value::Null);let first=g.native_first_object_contact([320,320]).unwrap();let matches=g.native_world_scan_objects().into_iter().filter(|o|o.sprite==0x1435).map(|o|o.slot).collect::<Vec<_>>();assert_eq!(matches.len(),2);assert_eq!(first,matches[0]);for direction in 0..8{assert!(!g.transport_direction_clear(&g.units[i],"grunt",direction,false));}assert!(g.unload_native_vehicle(i));assert_eq!(g.units[i].team,0,"the blocker consumes13 but leaves the vehicle occupied");
  g.apply_native_ambient_object(matches[0],[-1,320],0x70);assert_eq!(g.native_first_object_contact([320,320]),Some(matches[1]));g.apply_native_ambient_object(matches[1],[-1,320],0x70);assert!(g.native_first_object_contact([320,320]).is_none());assert!(g.transport_direction_clear(&g.units[i],"grunt",0,false));
 }
 #[test]fn native_apc_passenger_death_allocates_hidden_corpses_before_leader_removal(){
  let mut g=field();let(carrier,ids)=board(&mut g);occupied(&mut g,carrier);let ci=g.transport_member_index(carrier).unwrap();g.native_deaths=Default::default();
  let group=g.native_group_rosters.group_slot(0,3).unwrap();let count=g.native_group_rosters.pool[group].roster[18];
  let mut rng=g.original_rng.clone();rng.next_u32();rng.next_u32();g.kill_native_apc_passengers(ci,false,false);
  assert_eq!(g.original_rng.words,rng.words);assert_eq!(g.native_group_rosters.pool[group].roster[18],count,"58730 clears active bits without rewriting the roster");
  let corpses=g.native_deaths.records.iter().filter(|r|r.sprite!=-1).collect::<Vec<_>>();assert_eq!(corpses.len(),2);assert!(corpses.iter().all(|r|r.position[0]==10000));
  assert!(ids.iter().all(|id|g.units[g.transport_member_index(*id).unwrap()].hp<0.));assert!(g.native_transports.is_empty());assert_eq!(g.units[ci].team,0,"leader58ec0 is a separate subsequent action");
 }
 #[test]fn apc_object_destruction_uses_persistent_signed_claim_and_inside_gate_without_harming_approaching_robots(){
  let mut g=field();let(carrier,ids)=board(&mut g);let leader=g.transport_member_index(ids[0]).unwrap();assert!(g.start_native_transport(leader));let object=g.native_object_slot(carrier).unwrap();
  for(rank,id)in ids.iter().enumerate(){let i=g.transport_member_index(*id).unwrap();let slot=g.units[i].native_slot.unwrap();g.native_group_rosters.record_contact_claim(slot,if rank==2{object.wrapping_add(1)}else{object});if rank!=1{g.units[i].native_phase35=62;g.units[i].native_transport.as_mut().unwrap().phase=Phase::Inside;g.units[i].native_transport.as_mut().unwrap().visible=false;g.native_group_rosters.record_transport_inside(slot,true);}}
  g.native_deaths=Default::default();g.remove_native_apc_object_occupants(object);assert!(g.units[g.transport_member_index(ids[0]).unwrap()].hp<0.);assert!(ids[1..].iter().all(|id|original_combat::alive(true,g.units[g.transport_member_index(*id).unwrap()].hp)));let corpses=g.native_deaths.records.iter().filter(|r|r.sprite!=-1).collect::<Vec<_>>();assert_eq!(corpses.len(),1);assert_eq!(corpses[0].position[0],10000);
  let i=g.transport_member_index(ids[2]).unwrap();let slot=g.units[i].native_slot.unwrap();g.native_group_rosters.record_contact_claim(slot,130);g.remove_native_apc_object_occupants(130);assert!(original_combat::alive(true,g.units[i].hp),"41eb30 compares sign-extendedB5 against the unsignedOBJECT index");
 }
 #[test]fn unstarted_boarding_cancels_as_one_squad_without_losing_slots(){
  let mut g=field();let(carrier,ids)=board(&mut g);let i=g.transport_member_index(ids[0]).unwrap();assert!(g.start_native_transport(i));assert_eq!(g.native_transports.len(),1);
  g.command(&json!({"kind":"move","ids":ids,"x":1000.,"y":800.}));assert!(g.native_transports.is_empty());
  for id in &ids{let u=&g.units[g.transport_member_index(*id).unwrap()];assert!(u.native_transport.is_none());assert!(u.transport_carrier.is_none());assert!(u.native_slot.is_some());assert_eq!(u.squad,3);}
  assert_eq!(g.units[g.transport_member_index(carrier).unwrap()].team,-1);assert_eq!(g.units[i].dest,Some((1000.,800.)));
 }
 #[test]fn accepted_order_cleanup_uses_raw_transport_bits_without_finishing_the_new_command(){
  let mut g=field();let(_carrier,ids)=board(&mut g);let leader=g.transport_member_index(ids[0]).unwrap();assert!(g.start_native_transport(leader));let untouched=g.units[g.transport_member_index(ids[2]).unwrap()].native_slot.unwrap();g.native_group_rosters.members[untouched].as_mut().unwrap().flags8a&=!16;
  for &id in &ids{let i=g.transport_member_index(id).unwrap();g.units[i].native_order5c=4;g.units[i].order="stop".into();g.units[i].dest=Some((1000.,800.));let slot=g.units[i].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().contact_order=0;}
  let leader_slot=g.units[leader].native_slot.unwrap();let node=g.native_routes.get_mut().pool.allocate_goal([512,512]).unwrap();g.native_routes.get_mut().managed[leader_slot]=true;g.native_routes.get_mut().pool.links[leader_slot].primary=node;
  g.cleanup_native_order_transports(leader);assert_eq!(g.native_routes.borrow().pool.links[leader_slot].primary,0);assert_eq!(g.native_routes.borrow().pool.node(node).unwrap().next,crate::original_route_pool::FREE,"59d80's52500 frees the owned original node, not just its Web path mirror");
  for id in ids.iter().take(2){let u=&g.units[g.transport_member_index(*id).unwrap()];let m=g.native_group_rosters.members[u.native_slot.unwrap()].as_ref().unwrap();assert_eq!((u.native_order5c,u.order.as_str(),u.dest),(4,"stop",Some((1000.,800.))));assert!(u.native_transport.is_none());assert_eq!(m.flags8a&24,0);}
  let u=&g.units[g.transport_member_index(ids[2]).unwrap()];assert!(u.native_transport.is_some());assert_eq!(g.native_group_rosters.members[untouched].as_ref().unwrap().flags8a&24,8);
 }
 #[test]fn failed_transport_removes_the_input_actor_when_a_follower_has_inside20(){
  let mut g=field();let(_carrier,ids)=board(&mut g);let actor=g.transport_member_index(ids[0]).unwrap();assert!(!g.remove_native_failed_transport_actor(actor));let follower=g.transport_member_index(ids[1]).unwrap();let slot=g.units[follower].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().flags8a|=32;g.native_deaths=Default::default();assert!(g.remove_native_failed_transport_actor(actor));assert!(g.units[actor].hp<0.);for id in &ids[1..]{assert!(g.units[g.transport_member_index(*id).unwrap()].hp>0.,"59e60 passes the input actor, not the roster candidate, to58ec0");}assert_eq!(g.native_deaths.records.iter().filter(|r|r.sprite!=-1).count(),1);assert!(!g.remove_native_failed_transport_actor(actor),"inactive31:4 guard prevents duplicate removals");
 }
 #[test]fn invalid_commands_cannot_cancel_or_overwrite_a_transport_order(){
  let mut g=field();let(_carrier,ids)=board(&mut g);let i=g.transport_member_index(ids[0]).unwrap();g.start_native_transport(i);
  for kind in ["unknownOrder","repair","repairBridge","repairVehicle","unload","attackWall"]{g.command(&json!({"kind":kind,"ids":ids,"target":9999,"x":900.,"y":700.}));assert_eq!(g.native_transports.len(),1);assert!(g.native_transport_commands.is_empty());assert_eq!(g.units[i].order,"enter");}
 }
 #[test]fn failed_special_arrival_releases_routes_and_removes_only_already_inside_members(){
  let mut g=field();let(_carrier,ids)=board(&mut g);let i=g.transport_member_index(ids[0]).unwrap();g.start_native_transport(i);
  let animation=&mut g.native_transports[0].members[0].animation;animation.phase=Phase::Inside;animation.visible=false;g.units[i].native_transport=Some(animation.clone());
  let follower=g.transport_member_index(ids[1]).unwrap();assert!(g.cancel_native_transport(follower,true));assert!(g.native_transports.is_empty());assert_eq!(g.units[i].hp,-1.);
  for id in ids.iter().skip(1){let u=&g.units[g.transport_member_index(*id).unwrap()];assert!(original_combat::alive(true,u.hp));assert!(u.transport_carrier.is_none());assert_eq!(u.order,"guard");}
 }
 #[test]fn approach_begins_at_the_exact_native_boundary_from_object_center(){
  for(distance,starts)in [(48,true),(49,false)]{let mut g=field();let(_carrier,ids)=board(&mut g);let i=g.transport_member_index(ids[0]).unwrap();g.units[i].x=656.+f64::from(distance)*2.;g.units[i].y=656.;assert_eq!(g.start_native_transport(i),starts);}
 }
 #[test]fn mixed_boarding_defers_the_latest_command_and_hidden_ids_cannot_command_the_carrier(){
  let mut g=field();let(carrier,ids)=board(&mut g);let i=g.transport_member_index(ids[0]).unwrap();g.start_native_transport(i);
  let p=g.native_transports[0].portal();g.units[i].x=f64::from(p.0)*2.;g.units[i].y=f64::from(p.1)*2.;
  for _ in 0..16{g.update_native_transport(i);if workshops::hidden(&g.units[i]){break;}}assert!(workshops::hidden(&g.units[i]));
  g.command(&json!({"kind":"move","ids":ids,"x":1000.,"y":800.}));g.command(&json!({"kind":"move","ids":ids,"x":1200.,"y":900.}));assert_eq!(g.native_transport_commands.len(),1);
  occupied(&mut g,carrier);let ci=g.transport_member_index(carrier).unwrap();assert_eq!(g.units[ci].dest,Some((1200.,900.)));assert!(g.native_transport_commands.is_empty());
  g.command(&json!({"kind":"stop","ids":ids}));assert_eq!(g.units[ci].order,"move");assert!(ids.iter().all(|id|g.units[g.transport_member_index(*id).unwrap()].squad==3));
 }
 #[test]fn emergence_defers_commands_until_all_original_members_are_released(){
  let mut g=field();let(carrier,ids)=board(&mut g);occupied(&mut g,carrier);g.command(&json!({"kind":"unload","ids":[carrier]}));
  g.command(&json!({"kind":"stop","ids":[ids[0]]}));assert_eq!(g.native_transport_commands.len(),1);
  for _ in 0..180{g.step(0.059);if g.native_transports.is_empty(){break;}}
  assert!(g.native_transports.is_empty());assert!(g.native_transport_commands.is_empty());for id in ids{let u=&g.units[g.transport_member_index(id).unwrap()];assert!(u.hold_position);assert!(u.native_transport.is_none());}
 }
 #[test]fn destroying_the_parked_apc_during_exit_does_not_interrupt_the_stagger(){
  let mut g=field();let(carrier,ids)=board(&mut g);occupied(&mut g,carrier);g.command(&json!({"kind":"unload","ids":[carrier]}));
  g.units.iter_mut().find(|u|u.id==carrier).unwrap().hp=-1.;g.step(0.059);assert_eq!(g.native_transports.len(),1);
  assert!(workshops::hidden(&g.units[g.transport_member_index(ids[2]).unwrap()]));for _ in 0..180{g.step(0.059);if g.native_transports.is_empty(){break;}}
  assert!(g.native_transports.is_empty());assert!(ids.iter().all(|id|g.units.iter().any(|u|u.id==*id&&original_combat::alive(true,u.hp)&&!workshops::hidden(u))));
 }
 #[test]fn original_capacity_is_21_total_and_procedural_capacity_stays_six_passengers(){
  let mut g=field();g.spawn("apc",0,640.,640.,2,None);let i=g.units.len()-1;
  g.units[i].cargo=vec![Crew{kind:"grunt".into(),hp:30.};19];assert!(g.apc_has_passenger_room(&g.units[i]));
  g.units[i].cargo.push(Crew{kind:"grunt".into(),hp:30.});assert!(!g.apc_has_passenger_room(&g.units[i]));
  g.map.original=false;g.units[i].cargo.truncate(5);assert!(g.apc_has_passenger_room(&g.units[i]));g.units[i].cargo.push(Crew{kind:"grunt".into(),hp:30.});assert!(!g.apc_has_passenger_room(&g.units[i]));
 }
 #[test]fn all_21_native_roster_members_survive_a_full_boarding_and_staggered_exit(){
  let mut g=field();g.spawn("apc",-1,640.,640.,2,None);let carrier=g.units.last().unwrap().id;let mut ids=Vec::new();
  for rank in 0..21{g.spawn("grunt",0,640.+f64::from(rank%4)*4.,700.+f64::from(rank/4)*4.,3,None);ids.push(g.units.last().unwrap().id);}
  g.command(&json!({"kind":"enter","ids":ids,"target":carrier}));occupied(&mut g,carrier);let ci=g.transport_member_index(carrier).unwrap();assert_eq!(g.units[ci].cargo.len(),20);
  assert_eq!(g.native_transports[0].members.len(),21);assert!(ids.iter().all(|id|workshops::hidden(&g.units[g.transport_member_index(*id).unwrap()])));
  g.command(&json!({"kind":"unload","ids":[carrier]}));for _ in 0..200{g.step(0.059);if g.native_transports.is_empty(){break;}}
  assert!(g.native_transports.is_empty());assert!(ids.iter().all(|id|{let u=&g.units[g.transport_member_index(*id).unwrap()];!workshops::hidden(u)&&u.native_slot.is_some()&&u.squad==3}));
 }
 #[test]fn apc_sniper_hits_only_hull_and_unload_preserves_robot_health_grenades_and_native_profiles(){
  let mut g=field();let(carrier,ids)=board(&mut g);let di=g.transport_member_index(ids[0]).unwrap();g.units[di].grenades=7;g.units[di].native_target46=0x5503;g.units[di].native_trait7a=-37;g.units[di].native_cyclea0=3;g.units[di].native_reload7b=37;g.units[di].original_combat.as_mut().unwrap().aim_skill=37;
  let old_hp=ids.iter().map(|id|g.units[g.transport_member_index(*id).unwrap()].hp).collect::<Vec<_>>();occupied(&mut g,carrier);let ci=g.transport_member_index(carrier).unwrap();
  assert_eq!(g.units[ci].native_target46,0x5503);assert_eq!(g.units[ci].native_trait7a,-37);assert_eq!(g.units[ci].native_cyclea0,3);assert_eq!(g.units[ci].native_reload7b,0);assert_eq!(g.units[ci].grenades,7);assert_eq!(g.units[ci].original_combat.as_ref().unwrap().aim_skill,37);assert_eq!(g.units[ci].original_combat.as_ref().unwrap().fire_rate,80);
  let slots=g.units.iter().filter_map(|u|u.native_slot).collect::<Vec<_>>();assert_eq!(slots.len(),4);assert_eq!(slots.iter().copied().collect::<std::collections::HashSet<_>>().len(),slots.len());
  let hull=g.units[ci].hp;let driver_hp=g.units[ci].crew.as_ref().unwrap().hp;
  assert!(!original_combat::apply_attack_with_mode(&mut g.units[ci],original_combat::Attack{target:carrier,damage:10,aim:120,facing:0,bullet:true},1,false,||0));
  assert!(g.units[ci].hp<hull);assert_eq!(g.units[ci].crew.as_ref().unwrap().hp,driver_hp);assert!(ids.iter().zip(&old_hp).all(|(id,hp)|g.units[g.transport_member_index(*id).unwrap()].hp==*hp));
  let state=g.units[ci].original_combat.as_mut().unwrap();state.aim_skill=29;state.fire_rate=63;g.units[ci].grenades=4;g.units[ci].native_target46=0x5507;g.units[ci].native_cyclea0=2;g.command(&json!({"kind":"unload","ids":[carrier]}));
  let d=&g.units[g.transport_member_index(ids[0]).unwrap()];assert_eq!(d.native_target46,0x5507);assert_eq!(d.native_trait7a,-37);assert_eq!(d.native_cyclea0,2);assert_eq!(d.grenades,4);assert_eq!(d.original_combat.as_ref().unwrap().aim_skill,29);assert_eq!(d.original_combat.as_ref().unwrap().fire_rate,63);assert_eq!(d.original_combat.as_ref().unwrap().projectile_speed,Some(15));
  for _ in 0..180{g.step(0.059);if g.native_transports.is_empty(){break;}}
  assert!(ids.iter().zip(&old_hp).all(|(id,hp)|g.units[g.transport_member_index(*id).unwrap()].hp==*hp));
 }
}
