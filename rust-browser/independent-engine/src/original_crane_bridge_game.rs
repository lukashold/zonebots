//! Native bridge repair after the admitted browser order boundary. Neither
//!438cf0's warm UI local nor45ef00's original networking is synthesized here.
use crate::{Game,Value,json,original_combat,original_orders_game::Event};
use super::{State,BridgeSlot,crane::Crane,crane_bridge as bridge,trail};
impl BridgeSlot{
 pub(super)fn raw(&self)->[u8;32]{let mut r=self.raw_tail;for(at,n)in[(0,self.record.position[0]),(2,self.record.position[1]),(4,self.record.size[0]),(6,self.record.size[1]),(8,self.record.hp),(10,self.record.maximum),(12,self.orientation)]{bridge::set(&mut r,at,n);}r[15]=self.record.flags15;r}
 pub(super)fn store(&mut self,r:[u8;32]){self.raw_tail=r;self.record.position=[bridge::word(&r,0),bridge::word(&r,2)];self.record.size=[bridge::word(&r,4),bridge::word(&r,6)];self.record.hp=bridge::word(&r,8);self.record.maximum=bridge::word(&r,10);self.orientation=bridge::word(&r,12);self.record.flags15=r[15];}
}
impl State{
 ///45fef0/097b0 known records, including12 inactive/warm slots. This stays in
 ///the existing BridgeSlot pool; raw_tail owns only fields absent in Bridge.
 pub(super)fn load_native_bridge_records(&mut self,previous:Option<&Self>,v:&Value){
  let Some(rows)=v["nativeBridgeDisk26"].as_array().filter(|a|a.len()==12)else{return;};
  let Some(disk)=rows.iter().map(|a|{let a=a.as_array().filter(|a|a.len()==26)?;let mut d=[0u8;26];for(i,n)in a.iter().enumerate(){d[i]=u8::try_from(n.as_u64()?).ok()?;}Some(d)}).collect::<Option<Vec<_>>>()else{return;};
  for(slot,d)in disk.iter().enumerate(){let mut r=previous.and_then(|s|s.bridges.iter().find(|b|b.native_slot==slot)).map_or([0;32],BridgeSlot::raw);bridge::load(&mut r,d);if bridge::word(&r,0)!=-1{let max=bridge::word(&r,10);if max==0{bridge::set(&mut r,8,-1);bridge::set(&mut r,10,600);r[15]=2;}else{bridge::set(&mut r,8,max);}}
   if let Some(b)=self.bridges.iter_mut().find(|b|b.native_slot==slot){b.store(r);}else{let mut b=BridgeSlot{record:crate::original_world_contact::Bridge{position:[-1,0],size:[0;2],hp:0,maximum:0,flags15:0},binding:usize::MAX,native_slot:slot,orientation:0,retired_road:None,restored_terrain:vec![],raw_tail:r,native_work_owned:false};b.store(r);self.bridges.push(b);}
  }
 }
 pub(super)fn load_native_bridge_art(&mut self,v:&Value){
  self.bridge_repair_ready=false;self.bridge_crane_frames.clear();
  let Some(rows)=v["nativeBridgePlaybackHeaders"].as_array().filter(|a|a.len()==8)else{return;};
  let Some(headers)=rows.iter().enumerate().map(|(i,r)|{if r["pointer"].as_u64()?!=u64::from(bridge::HEADERS[i]){return None;}let bytes=r["bytes"].as_array().filter(|a|a.len()==1280)?.iter().map(|n|u8::try_from(n.as_u64()?).ok()).collect::<Option<Vec<_>>>()?;Some((bridge::HEADERS[i],bytes))}).collect::<Option<Vec<_>>>()else{return;};
  let Some(dirs)=v["nativeCraneBridgeFrames"].as_array().filter(|a|a.len()==4)else{return;};let Some(frames)=dirs.iter().enumerate().map(|(dir,d)|d.as_array().filter(|a|a.len()==12)?.iter().map(|f|f.as_array().filter(|a|a.len()==if dir&1==0{6}else{4})?.iter().map(|r|{let a=r.as_array().filter(|a|a.len()==4)?;let mut row=[0i16;4];for(i,n)in a.iter().enumerate(){row[i]=i16::try_from(n.as_i64()?).ok()?;}Some(row)}).collect::<Option<Vec<_>>>()).collect::<Option<Vec<_>>>()).collect::<Option<Vec<_>>>()else{return;};
  for(head,bytes)in headers{if let Some((_,old))=self.trail.bridge_headers.iter_mut().find(|(p,_)|*p==head){if old.len()<1280{old.resize(1280,0);}old[..1280].copy_from_slice(&bytes);}else{self.trail.bridge_headers.push((head,bytes));}}
  self.bridge_crane_frames=frames;self.bridge_repair_ready=v["nativeBridgeDisk26"].as_array().is_some_and(|a|a.len()==12);
 }
}
impl Game{
 pub(crate)fn owns_native_bridge_repair(&self,index:usize)->bool{let Some(u)=self.units.get(index)else{return false;};let Some(slot)=u.native_slot else{return false;};self.map.original&&self.map.native_road_graph.is_some()&&self.native_world.as_ref().is_some_and(|s|s.bridge_repair_ready&&s.cranes[slot].as_ref().is_some_and(|c|c.entity==u.id&&c.target.is_some())&&self.native_group_rosters.members[slot].as_ref().is_some_and(|m|m.flags8a&64!=0))}
 ///The browser has admitted a repair command. Below this explicit boundary
 ///we write the source link/work marker and lower MOVE1 waypoint; the old
 ///438cf0 local/45ef00 nativeMP latency are NOT replaced with guessed values.
 pub(crate)fn begin_native_bridge_after_browser_acceptance(&mut self,index:usize,click:[i16;2])->Option<bool>{
  if !self.map.original||self.map.native_road_graph.is_none(){return None;}let u=self.units.get(index)?.clone();let slot=u.native_slot?;let s=self.native_world.as_ref()?;if !s.bridge_repair_ready{return None;}if u.kind!="crane"{return Some(false);}
  let Some(target)=s.bridges.iter().enumerate().filter(|(_,b)|b.native_slot<12&&bridge::contains_destroyed(&b.raw(),click)).min_by_key(|(_,b)|b.native_slot).map(|(i,_)|i)else{return Some(false);};
  let r=s.bridges[target].raw();let mut c=s.cranes[slot].clone().filter(|c|c.entity==u.id).unwrap_or_default();c.entity=u.id;c.target=Some(target);c.work9a=255;c.work9a_known=true;let point=bridge::admitted_waypoint(&r);
  self.native_selection.contexts[slot].link28=Some(bridge::BASE+s.bridges[target].native_slot as u32*32);
  if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags8a|=64;}let s=self.native_world.as_mut().unwrap();s.cranes[slot]=Some(c);s.bridges[target].native_work_owned=true;
  self.native_group_rosters.set_current_roster(u.team,u.squad);self.record_native_order_event(index,Event::classified(1,point,0));let u=&mut self.units[index];u.order="repairBridge".into();u.target=None;u.dest=Some((f64::from(point[0]-8)*2.,f64::from(point[1]-8)*2.));u.hold_position=false;u.explicit_stop=false;Some(true)
 }
 pub(super)fn native_bridge_crane_contact(&mut self,index:usize,c:&mut Crane,point:[i16;2],code:u8){let Some(target)=c.target else{return;};let Some(s)=self.native_world.as_ref()else{return;};let Some(b)=s.bridges.get(target)else{return;};let first=s.bridges.iter().filter(|b|bridge::contains_destroyed(&b.raw(),point)).min_by_key(|b|b.native_slot).map(|b|b.native_slot);if first!=Some(b.native_slot){return;}if code==47&&c.work9a==255{c.work9a=1;return;}if code!=50{return;}if c.frame86==-1&&c.work9a!=0{if b.record.flags15&2==0||b.record.flags15&8!=0{c.work9a=0;return;}c.approach_b1=Some(original_combat::facing(self.units[index].angle)>>1&3);c.frame86=0;c.saved=point;self.native_world.as_mut().unwrap().bridges[target].record.flags15|=8;}else if c.work9a==254&&c.frame86==11{c.frame86=75;}}
 pub(super)fn progress_native_bridge_crane(&mut self,target:usize,rate:u8)->bool{let Some(s)=&mut self.native_world else{return false;};let b=&mut s.bridges[target];let mut r=b.raw();let done=bridge::progress(&mut r,rate);b.store(r);if let Some(v)=self.map.bridge_hp.get_mut(b.binding){*v=f64::from(b.record.hp.max(0));}done}
 pub(super)fn native_bridge_retreat(&mut self,index:usize,c:&Crane)->Option<[i16;2]>{let target=c.target?;let Some(direction)=c.approach_b1 else{self.native_world.as_mut()?.trail.boundary=Some(trail::Boundary{address:0x5e30c8+(self.units[index].native_slot?as u32)*192+0xb1,length:1});return None;};let b=&self.native_world.as_ref()?.bridges[target];Some(bridge::retreat(&b.raw(),[(self.units[index].x/2.).floor()as i16,(self.units[index].y/2.).floor()as i16],direction))}
 pub(super)fn create_native_bridge_jobs(&mut self,index:usize,c:&Crane){let Some(target)=c.target else{return;};let Some(slot)=self.units[index].native_slot else{return;};let Some(mut s)=self.native_world.take()else{return;};let raw=s.bridges[target].raw();for(cell,code)in bridge::restore_terrain(&raw,|cell|self.world_code(cell)){self.set_world_code(cell,code);}if let Some((region,bounds))=s.bridges[target].retired_road{if let Some(r)=self.map.native_road_graph.as_mut().and_then(|g|g.regions.get_mut(region)){r.bounds=bounds;}}self.map.revision=self.map.revision.wrapping_add(1);
  let mut raw=raw;self.original_rng.multiplayer=self.multiplayer;let pointer=bridge::BASE+s.bridges[target].native_slot as u32*32;let result=bridge::jobs(&mut s.trail,&mut raw,pointer,0x5e30c8+slot as u32*192,||self.original_rng.next_u32());s.bridges[target].store(raw);if let Err(e)=result{s.trail.boundary=Some(e);}self.native_world=Some(s);
 }
 ///58ec0's family36 prefix, before58af0/group removal. The known existing
 ///Crane binding owns28/8a; no dead-unit/WebID fallback is introduced.
 pub(crate)fn release_native_crane_target_for_removal(&mut self,index:usize){let Some(u)=self.units.get(index)else{return;};if u.kind!="crane"{return;}let Some(slot)=u.native_slot else{return;};let Some(m)=self.native_group_rosters.members[slot].as_ref().filter(|m|m.flags31&4!=0)else{return;};let bridge_mode=m.flags8a&64!=0;let claim=u.native_initiative_scan.flags71&16!=0;let Some(s)=&mut self.native_world else{return;};let Some(c)=s.cranes[slot].as_ref().filter(|c|c.entity==u.id)else{return;};let Some(target)=c.target else{return;};let mask=if claim{!12u8}else{!8u8};if bridge_mode{if let Some(b)=s.bridges.get_mut(target){b.record.flags15&=mask;}}else if let Some(b)=s.buildings.get_mut(target){b.display.flags6&=mask;if s.trail.known_builds[target]{s.trail.builds[target][6]&=mask;}}}
 pub(super)fn finish_native_bridge_job(&mut self,pointer:u32)->bool{let Some(s)=&mut self.native_world else{return false;};let slot=pointer.wrapping_sub(bridge::BASE)as usize/32;let Some(b)=s.bridges.iter_mut().find(|b|b.native_slot==slot)else{s.trail.boundary=Some(trail::Boundary{address:pointer,length:32});return false;};let mut raw=b.raw();let final_job=bridge::finish_job(&mut raw);b.store(raw);if final_job{b.native_work_owned=false;if let Some(v)=self.map.bridge_hp.get_mut(b.binding){*v=f64::from(b.record.hp.max(0));}for(k,&id)in self.map.bridge_base.iter().enumerate(){if id==b.binding as i32{self.map.tiles[k]=5;}}self.map.revision=self.map.revision.wrapping_add(1);self.events.push(json!({"kind":"bridgeRepaired","id":b.binding}));}let point=b.record.position;
  if final_job{self.emit_native_late_build_audio(crate::original_audio::late_build::Stage::BridgeComplete,point);}true
 }
}

#[cfg(test)]#[path="original_crane_bridge_game_tests.rs"]mod tests;
