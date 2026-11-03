//! Native selection/dispersal bridge. The browser supplies selection events
//! explicitly; a command's flat entity list is not silently made into a row.
use super::{Game,original_combat,original_orders_game::{self as orders,Event},original_selection as native,original_world_contact as world};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct PreparedMove{pub index:usize,pub event:Event,pub anchor:[i16;2]}
impl Game{
 ///41b210 zeroes the UNIT and438740 writes its initial ABB icon. Call only
 ///for a fresh native UNIT allocation, never a group split or identity rebind.
 pub(crate) fn initialize_native_selection_context(&mut self,slot:usize){if let Some(c)=self.native_selection.contexts.get_mut(slot){*c=native::SourceContext{icon_a8:Some(0xabb),link28:Some(0),saved_class8e:Some(0)};}}
 pub(crate) fn record_native_selection_saved_class(&mut self,slot:usize,class:u8){if let Some(c)=self.native_selection.contexts.get_mut(slot){c.saved_class8e=Some(class);}}
 pub(crate) fn record_native_selection_link(&mut self,slot:usize,link:Option<u32>){if let Some(c)=self.native_selection.contexts.get_mut(slot){c.link28=link;}}
 ///438740 propagates the raw icon across all21 slots only for a leader.
 pub(crate) fn record_native_selection_icon(&mut self,index:usize,icon:u16){let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return;};let mut slots=vec![slot];if let Some(m)=self.native_group_rosters.members[slot].as_ref().filter(|m|m.role== -1){if let Some(g)=self.native_group_rosters.pool.get(usize::from(m.group)){slots.extend((0..21).filter_map(|role|g.member(role)).map(usize::from));}}for slot in slots{if let Some(c)=self.native_selection.contexts.get_mut(slot){c.icon_a8=Some(icon);}}}
 ///4416b0's move-click classifier with live BUILD/APC/terrain and explicit
 ///raw flag/fort lists. `click` has already passed438950's camera transform.
 ///No offset is added here: a native UI click is already a waypoint.
 pub(crate) fn classify_native_move_click(&self,index:usize,click:[i16;2],flag_mode:bool,forts:[[i16;2];4],flags:&[[i16;2]])->Option<Event>{
  self.classify_native_move_click_known_forts(index,click,flag_mode,forts.map(Some),flags)
 }
 fn classify_native_move_click_known_forts(&self,index:usize,click:[i16;2],flag_mode:bool,forts:[Option<[i16;2]>;4],flags:&[[i16;2]])->Option<Event>{
  let unavailable=std::cell::Cell::new(false);
  if !self.map.original{return None;}let unit=self.units.get(index)?;let buildings=self.native_world_buildings()?;let objects=self.native_world_scan_objects();let actor=[(unit.x/2.).floor()as i16,(unit.y/2.).floor()as i16];let point=native::move_click_destination(unit.native_class7c,actor,click,flag_mode,|query,p|match query{
   native::ClickQuery::Flag=>native::flag_click(*p,flags).is_some_and(|at|{*p=at;true}),
   native::ClickQuery::Apc=>objects.iter().find(|o|o.flags13&0x50!=0&&o.position[0]!= -1&&(0..2).all(|axis|o.position[axis]<=p[axis]&&i32::from(p[axis])-i32::from(o.extent20)<=i32::from(o.position[axis]))).filter(|o|o.definition==31).is_some_and(|o|{*p=o.position;true}),
   native::ClickQuery::Building=>world::first_building(&buildings,*p,false,false).is_some_and(|(_,at)|{*p=at;true}),
   native::ClickQuery::Snap=>self.native_selection_terrain_code(*p).and_then(|code|{let i=match code&127{20=>1,21=>0,60=>2,61=>3,_=>return None};let Some(base)=forts[i]else{unavailable.set(true);return None;};Some([base[0].wrapping_add(80),base[1].wrapping_add(64)])}).is_some_and(|at|{*p=at;true}),
  },|p|!orders::footprint_clear(p,false,|cell|self.original_sector_path_allowed(unit,cell)))?;(!unavailable.get()).then_some(Event::classified(1,point.point,0))
 }
 pub(crate) fn native_selection_terrain_code(&self,point:[i16;2])->Option<u8>{let cell=(point[1]&!15).wrapping_mul(8).wrapping_add(point[0]>>4);if !(0..16384).contains(&i32::from(cell)){return None;}let(x,y)=(cell as usize&127,cell as usize>>7);if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x)}else{self.map.native_terrain_plane.get(y*128+x)}.copied()}
 ///The source37ac0 selection operation is a separate validated UI event.
 ///After that,43a9f0 broadcasts secondary leaders in row order and the caller
 ///finishes its primary actor last. Followers do not receive new raw5c here.
 pub(crate) fn native_human_move_actors(&mut self,primary:usize)->Option<Vec<usize>>{let mut actors=self.native_selection_broadcast_candidates(primary)?;actors.push(primary);Some(actors)}
 ///Concrete single-player4416b0→45f070→43a600 preparation. Classification
 ///runs before any selection mask/roster mutation; the caller records the
 ///returned event only after its own accepted-command validation succeeds.
 ///The anchor is raw525a0 geometry; convert it to browser scale exactlyonce.
 pub(crate) fn prepare_native_human_move(&mut self,index:usize,click:[i16;2],flag_mode:bool)->Option<(Event,[i16;2])>{
  let event=self.classify_native_human_move(index,click,flag_mode)?;let event=self.disperse_native_human_move(index,event)?;Some((event,[event.waypoint[0].wrapping_sub(8),event.waypoint[1].wrapping_sub(8)]))
 }
 pub(crate) fn classify_native_human_move(&self,index:usize,click:[i16;2],flag_mode:bool)->Option<Event>{let bases=self.native_fort_bases()?;let flags=self.native_world_scan_objects().iter().filter(|o|o.flag_sector.is_some()).map(|o|o.position).collect::<Vec<_>>();self.classify_native_move_click_known_forts(index,click,flag_mode,[Some(bases[0]),Some(bases[1]),None,None],&flags)}
 pub(crate) fn disperse_native_human_move(&mut self,index:usize,event:Event)->Option<Event>{let bases=self.native_fort_bases()?;let bounds=[((self.map.width&!1)as i16).wrapping_mul(16),((self.map.height&!1)as i16).wrapping_mul(16)];self.disperse_native_selection_order(index,event,bounds,bases[0])}
 ///A validated37ac0 selection event has already supplied the native rows.
 ///Classification happens once, before any mutation;43a9f0 then broadcasts
 ///one shared waypoint to secondary leaders and finishes the primary last.
 pub(crate) fn prepare_native_human_move_batch(&mut self,primary:usize,click:[i16;2],flag_mode:bool)->Option<Vec<PreparedMove>>{
  let event=self.classify_native_human_move(primary,click,flag_mode)?;let actors=self.native_human_move_actors(primary)?;let mut result=vec![];for index in actors{let event=self.disperse_native_human_move(index,event)?;result.push(PreparedMove{index,event,anchor:[event.waypoint[0].wrapping_sub(8),event.waypoint[1].wrapping_sub(8)]});}Some(result)
 }
 ///37ac0 consumes selected group slots in the UI's selection order. It first
 ///removes their previous row membership, then invokes43ab30 on each leader.
 ///The returned list records failed eligibility rather than inventing a row.
 pub(crate) fn select_native_groups(&mut self,groups:&[u8])->Vec<u8>{
  if !self.map.original||groups.is_empty(){return vec![];}let mut inputs=vec![];let mut rejected=vec![];
  for &group in groups{let Some(row)=self.native_group_rosters.pool.get(usize::from(group))else{rejected.push(group);continue;};let Some(leader)=row.member(0).and_then(|slot|self.units.iter().find(|u|u.native_slot==Some(usize::from(slot))))else{rejected.push(group);continue;};let mut first=None;let mut uniform=true;for role in 0..21{if let Some(slot)=row.member(role){let Some(m)=self.native_group_rosters.members[usize::from(slot)].as_ref()else{uniform=false;break;};let bit=m.flags8a&32!=0;if first.is_some_and(|old|old!=bit){uniform=false;break;}first=Some(bit);}}let class=leader.native_class7c;let family=original_combat::animation_family(&leader.kind);if class==2||family==36||!uniform{rejected.push(group);}inputs.push((group,class,family,uniform));}
  self.native_selection.select(&inputs);rejected
 }
 ///43aaf0 followed by43a9f0's roster ordering, before43a830 reaction handling.
 ///No fresh alive/visibility/ownership filtering occurs at this pool stage.
 pub(crate) fn native_selection_broadcast_candidates(&mut self,index:usize)->Option<Vec<usize>>{
  let slot=self.units.get(index)?.native_slot?;let m=self.native_group_rosters.members.get(slot)?.as_ref()?;let(group,role)=(m.group,m.role);self.native_selection.promote(group,role);let groups=self.native_selection.broadcast_groups(group,role);let mut result=vec![];for group in groups{let Some(leader)=self.native_group_rosters.pool.get(usize::from(group)).and_then(|r|r.member(0))else{continue;};let Some(i)=self.units.iter().position(|u|u.native_slot==Some(usize::from(leader)))else{continue;};let u=&self.units[i];if i!=index&&u.native_class7c!=2&&original_combat::animation_family(&u.kind)!=36{result.push(i);}}Some(result)
 }
 ///43a600 on a classified event. `bounds` is622d50/52 and `fort` is the
 ///fixed622d54 base; neither is reconstructed from a faction's currentowner.
 ///41c290 uses live BUILDslot order/metadata;43a520 uses the first APC OBJECT
 ///hit in forced-extent mode and the unit's actual terrain permission table.
 pub(crate) fn disperse_native_selection_order(&mut self,index:usize,event:Event,bounds:[i16;2],fort:[i16;2])->Option<Event>{
  if !self.map.original||event.opcode!=1{return Some(event);}let unit=self.units.get(index)?.clone();let slot=unit.native_slot?;let group=self.native_group_rosters.members.get(slot)?.as_ref()?.group;let buildings=self.native_world_buildings()?;let objects=self.native_world_scan_objects();let class=unit.native_class7c;let family=original_combat::animation_family(&unit.kind);let mut state=std::mem::take(&mut self.native_selection);
  let dispersed=state.disperse(group,class,event.opcode,event.waypoint,fort,|point|{let(hit,adjusted)=world::first_building(&buildings,*point,false,false)?;*point=adjusted;Some(buildings[hit].kind)},|point|native::destination_blocked(class,family,point,bounds,|point|objects.iter().find(|o|o.flags13&0x50!=0&&o.position[0]!= -1&&(0..2).all(|axis|o.position[axis]<=point[axis]&&i32::from(point[axis])-i32::from(o.extent20)<=i32::from(o.position[axis]))).is_some_and(|o|o.definition==31),|point,wide|!orders::footprint_clear(point,wide,|cell|self.original_sector_path_allowed(&unit,cell))));
  self.native_selection=state;Some(Event{waypoint:dispersed.point,..event})
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,733);g.map.original=true;g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.native_group_rosters.clear();g.map.width=64;g.map.height=64;g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];for(i,squad)in[7,9,11].into_iter().enumerate(){g.spawn("grunt",0,100.+i as f64*40.,100.,squad,None);}g.maintain_native_group_rosters();g.import_native_world(&super::super::Value::Null);g}
 #[test]fn explicit_selection_rows_and_original_offsets_reach_live_order_geometry(){let mut g=field();let groups=(0..3).map(|i|g.native_group_rosters.group_slot(0,g.units[i].squad).unwrap()as u8).collect::<Vec<_>>();assert!(g.select_native_groups(&groups).is_empty());let event=Event::classified(1,[200,300],0);let moved=g.disperse_native_selection_order(1,event,[1024,1024],[1000,1000]).unwrap();assert_eq!(moved.waypoint,[200,316]);let moved=g.disperse_native_selection_order(1,event,[1024,1024],[1000,1000]).unwrap();assert_eq!(moved.waypoint,[184,300]);assert_eq!(g.disperse_native_selection_order(2,event,[1024,1024],[1000,1000]).unwrap(),event,"primary header maps to member0 and skips dispersal");}
 #[test]fn live_move_click_uses_raw_flag_positions_and_source_fort_snap(){let mut g=field();let forts=[[100,100],[100,800],[0,0],[0,0]];let rawflag=[96,160];assert_eq!(g.classify_native_move_click(0,[100,168],true,forts,&[rawflag]).unwrap().waypoint,rawflag);g.map.terrain_codes[10*g.map.width+10]=20;assert_eq!(g.classify_native_move_click(0,[164,164],false,forts,&[]).unwrap().waypoint,[180,864]);}
 #[test]fn mixed_transport_members_are_rejected_at_the_source_selection_gate(){let mut g=field();g.spawn("grunt",0,140.,140.,7,None);g.maintain_native_group_rosters();let group=g.native_group_rosters.group_slot(0,7).unwrap();let slot=g.units.last().unwrap().native_slot.unwrap();g.native_group_rosters.record_transport_inside(slot,true);assert_eq!(g.select_native_groups(&[group as u8]),vec![group as u8]);assert!(g.native_selection.find(group as u8).is_none());}
 fn fort_field()->Game{let mut g=field();let template=Game::new(0,1,739).buildings.iter().find(|b|b.kind=="fort").unwrap().clone();for(i,base)in[[800.,32.],[800.,800.]].into_iter().enumerate(){let mut b=template.clone();b.id=500+i;b.team=1-i as i32;b.raw_base=Some((base[0],base[1]));b.x=base[0]*2.;b.y=base[1]*2.;g.buildings.push(b);}g.import_native_world(&super::super::Value::Null);g}
 #[test]fn human_move_batch_classifies_once_then_orders_secondary_leaders_before_primary(){let mut g=fort_field();let groups=(0..3).map(|i|g.native_group_rosters.group_slot(0,g.units[i].squad).unwrap()as u8).collect::<Vec<_>>();g.select_native_groups(&groups);let events=g.prepare_native_human_move_batch(2,[200,300],false).unwrap();assert_eq!(events.iter().map(|e|e.index).collect::<Vec<_>>(),vec![0,1,2]);assert_eq!(events.iter().map(|e|e.event.waypoint).collect::<Vec<_>>(),vec![[200,300],[200,316],[184,300]]);assert_eq!(events[1].anchor,[192,308]);assert!(g.units.iter().all(|u|u.native_order5c==0),"preparation does not write an unaccepted raw order");}
 #[test]fn rejected_native_click_leaves_selection_mask_and_rosters_untouched(){let mut g=fort_field();let groups=(0..3).map(|i|g.native_group_rosters.group_slot(0,g.units[i].squad).unwrap()as u8).collect::<Vec<_>>();g.select_native_groups(&groups);g.map.terrain_codes.fill(127);g.map.native_terrain_plane.fill(127);g.native_selection.used[8]=1;let before=g.native_selection.rows.clone();let roster=g.native_group_rosters.pool.iter().map(|r|r.roster).collect::<Vec<_>>();assert!(g.prepare_native_human_move_batch(2,[200,300],false).is_none());assert_eq!(g.native_selection.rows,before);assert_eq!(g.native_selection.used[8],1);assert_eq!(g.native_group_rosters.pool.iter().map(|r|r.roster).collect::<Vec<_>>(),roster);}
}
