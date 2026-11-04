//!Game binding for the separate native removal sprite pool.
use super::{Game,Unit,json,original_combat,original_death::{self,Action,RemovalSource}};
impl Game{
 ///58ec0 without the caller's contact explosions/audio or41eb30's familyFF
 ///override. An already inactive slot returns before conversion and RNG.
 pub(crate) fn native_remove_actor(&mut self,index:usize,suppressed:bool,forced:bool)->bool{
  let Some(before)=self.units.get(index).cloned()else{return false;};
  if !before.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref).is_some_and(|m|m.flags31&4!=0){return false;}
  self.native_remove_actor_from(index,&before,suppressed,forced,false);self.units[index].hp=-1.;true
 }
 fn native_remove_actor_from(&mut self,index:usize,before:&Unit,suppressed:bool,forced:bool,crew_only:bool){
  let family=original_combat::animation_family(&before.kind);let mut point=[(before.x/2.).floor()as i16,(before.y/2.).floor()as i16];
  if before.native_transport.as_ref().is_some_and(|t|t.phase==super::original_transport::Phase::Inside){point[0]=10000;}
  self.remember_original_group_order(index,before);self.release_original_claim(index);
  if [32,59].contains(&family){self.kill_native_apc_passengers(index,suppressed,forced);}
  if let Some(slot)=before.native_slot{self.remember_native_point_target(slot,point,before.native_class7c);}
  self.native_removed_hardware(index,before,suppressed,crew_only);
  //58660 releases occupancy only for an extant W pointer, before freeing it.
  //58af0 may already have replaced UNIT8 with-1, retaining the outer dummycase.
  if self.units[index].native_hardware_slot.is_some(){self.release_native_removal_occupancy(index);}self.release_native_hardware_record(index);
  if let Some(m)=self.units[index].native_slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.flags31&=!4;}
  self.native_deaths.allocate(point,before.native_flags64,suppressed,forced,||self.original_rng.next_u32());
  //58ec0 repeats5ab40 after58660, now with inactive31:4 (no path restart).
  self.release_native_removal_occupancy(index);self.remove_original_group_member(index);self.clear_native_owned_paths(index);
  let u=&mut self.units[index];u.native_slot=None;u.native_dodge=None;u.native_cover=None;u.native_idle=None;u.native_walk=None;u.native_fire=None;u.path.clear();u.native_route_pending.set(None);u.dest=None;u.target=None;u.moving=false;
 }
 ///41eb30 writes UNIT.family30=FF before calling58ec0 directly. That skips
 ///58af0's OBJECT conversion and emits no contact-death explosions. The
 ///caller supplies the real native position, including Inside sentinel10000.
 pub(crate) fn native_direct_removal(&mut self,index:usize,point:[i16;2],suppressed:bool,forced:bool)->bool{
  let Some(before)=self.units.get(index).cloned()else{return false;};let active=before.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref).is_some_and(|m|m.flags31&4!=0);if !active{return false;}
  if let Some(slot)=before.native_slot{self.remember_native_point_target(slot,point,before.native_class7c);}self.remember_original_group_order(index,&before);self.release_original_claim(index);if self.units[index].native_hardware_slot.is_some(){self.release_native_removal_occupancy(index);}self.release_native_hardware_record(index);if let Some(m)=self.units[index].native_slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.flags31&=!4;}self.native_deaths.allocate(point,before.native_flags64,suppressed,forced,||self.original_rng.next_u32());self.release_native_removal_occupancy(index);self.remove_original_group_member(index);
  let u=&mut self.units[index];u.hp=-1.;u.native_slot=None;u.native_dodge=None;u.native_cover=None;u.native_idle=None;u.native_walk=None;u.native_fire=None;u.path.clear();u.native_route_pending.set(None);u.dest=None;u.target=None;u.moving=false;true
 }
 ///Run before the group promotion callback:58660 allocates its independent
 ///removal sprite (and consumes RNG) before4bba0 can choose another leader.
 pub(crate) fn native_contact_removal(&mut self,index:usize,before:&Unit,source:RemovalSource,crew_only:bool){
  if !self.map.original||index>=self.units.len(){return;}
  let class=before.native_class7c;let family=original_combat::animation_family(&before.kind);let point=[(before.x/2.).floor()as i16,(before.y/2.).floor()as i16];let owner=before.native_flags64;
  let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));let audible=original_death::sound_visible(point,camera,size);
  let actions=original_death::removal_actions(source,class,family,point,owner,crew_only,audible);let mut removed=false;
  for action in actions{match action{
   Action::Remove{suppressed}=>{if removed{continue;}removed=true;
    self.native_remove_actor_from(index,before,suppressed,false,crew_only);
   },
   Action::Emit{kind,count,sprite,maximum,origin,owner}=>self.native_removal_emission(kind,origin,count,sprite,maximum,owner),
   Action::Sound(id)=>self.native_removal_sound(point,id),
  }}
 }
 pub(crate) fn update_native_deaths(&mut self){if self.map.original{let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));self.native_deaths.advance(camera,size,self.motion_tick&1!=0);}}
 pub(crate) fn native_death_visuals(&self)->Vec<serde_json::Value>{self.native_deaths.draws.iter().map(|d|json!({"id":d.slot,"sprite":d.sprite,"x":i32::from(d.position[0])*2,"y":i32::from(d.position[1])*2,"team":d.color&3,"nativeDrawX":d.screen[0],"nativeDrawY":d.screen[1],"scale":1.,"rotation":0.})).collect()}
}
