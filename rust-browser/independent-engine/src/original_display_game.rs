//!Shared queue producers in435b90..435e1c order. BUILD base graphics precede
//!the queue;500FX/bullets/blasts/projectiles remain later native overlays.
use super::{Record,sort,infantry};
use crate::{Game,Unit,json,original_combat,workshops};
use serde_json::Value;
fn point(x:f64,y:f64)->[i16;2]{[(x/2.).floor()as i16,(y/2.).floor()as i16]}
pub(super) fn plain(r:Record,source:&str,id:usize)->Value{json!({"kind":"sprite","source":source,"id":id,"queueY":i32::from(r.y)*2,"palette":r.palette,"sprite":r.sprite,"x":i32::from(r.x)*2,"y":i32::from(r.y.wrapping_add(i16::from(r.offset)))*2,"scale":1.,"rotation":0.})}
fn effect_rows(queue:&mut Vec<(Record,Value)>,source:&str,values:Vec<Value>){for f in values{let p=point(f["x"].as_f64().unwrap_or(0.),f["queueY"].as_f64().or_else(||f["y"].as_f64()).unwrap_or(0.));let record=Record{x:p[0],y:p[1],sprite:f["sprite"].as_i64().unwrap_or(0)as i16,palette:255,offset:0};queue.push((record,json!({"kind":"effect","source":source,"id":f["id"],"queueY":i32::from(p[1])*2,"draw":f})));}}
impl Game{
 ///5a080's69:40 branch returns before55f10. Remember eligibility at UNIT
 ///entry; a handler may mutate other raw flags before the final draw call.
 pub(crate) fn native_unit_display_slot(&mut self,index:usize)->Option<usize>{
  if !self.map.original{return None;}let u=self.units.get(index)?;let slot=u.native_slot?;let m=self.native_group_rosters.members.get(slot)?.as_ref()?;
  self.native_display.entry69[slot]=m.flags69;
  (m.flags31&4!=0&&m.flags69&64==0).then_some(slot)
 }
 pub(crate) fn begin_native_display_frame(&mut self){if self.map.original{self.native_display.units=Some(Vec::new());}}
 fn native_regular_hardware_picture(&self,u:&Unit)->Option<super::hardware::Draw>{
  let cell=u.native_cell10 as u16 as usize;let(x,y)=(cell&127,cell>>7);let terrain=if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(127)}else{self.map.native_terrain_plane.get(cell).copied().unwrap_or(127)}&127;
  //55f10's BUILD clipping path is a separate query/caller boundary. Do not
  //silently claim those clipped compositions as the ordinary layer branch.
  if matches!(terrain,30|31|44|62|63|64|65|66)||u.native_flags64&u.native_mask98>=128{return None;}
  let w=u.native_hardware_slot.and_then(|slot|self.native_hardware_pool.get(slot)).map(|s|s.attributes);let h=u.native_hardware.as_ref();
  super::hardware::draw(super::hardware::Input{position:point(u.x,u.y),family:original_combat::animation_family(&u.kind),class:u.native_class7c,phase:u.native_phase35,frame:u.native_frame32,heading:original_combat::facing(u.angle),turret:h.map_or(original_combat::facing(u.aim),|h|h.turret),hp:w.map_or(u.hp as i16,|w|w.word(8)),maximum:w.map_or(u.max_hp as i16,|w|w.word(10)),hatch_body:h.map_or_else(||w.map_or(0,|w|w.byte(4)),|h|h.hatch_body),hatch_frame:h.map_or_else(||u.original_combat.as_ref().map_or(-1,|c|c.secondary_state),|h|h.hatch_frame),flags64:u.native_flags64,mask98:u.native_mask98,elevation:u.native_elevation76,counter84:u.native_counter84})
 }
 fn native_unit_display_row(&self,u:&Unit,slot:usize)->Option<(Record,Value)>{
  if workshops::hidden(u)||matches!(u.native_phase35,18|68){return None;}let p=point(u.x,u.y);if p[0]==10000{return None;}
  let flags69=self.native_group_rosters.members.get(slot).and_then(Option::as_ref).map_or(self.native_display.entry69[slot],|m|m.flags69);
  //55f10 checks visibility and signed screen bounds before selecting the
  //normal record or clearing its one-shot64:4 palette flag.
  if flags69&4==0{return None;}let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));let screen=[p[0].wrapping_sub(camera[0]),p[1].wrapping_sub(camera[1])];if screen[0]<=-32||screen[0]>=size[0]||screen[1]<=-48||screen[1]>=size[1]{return None;}
  let hardware=matches!(u.native_class7c,1|2);let native=if hardware{self.native_regular_hardware_picture(u)}else{None};let header=if let Some(n)=&native{n.header}else if u.native_class7c==1{Record{x:p[0],y:p[1].wrapping_add(i16::from(u.native_elevation76)).wrapping_add(8),sprite:0,palette:u.native_flags64&7,offset:0}}else{infantry(p,u.native_phase35,u.native_frame32,original_combat::facing(u.angle)as i8,u.native_elevation76,u.native_flags64)?};let plain_hardware=native.as_ref().is_some_and(|n|n.layers.is_empty());
  let mut row=json!({"kind":"unit","id":u.id,"nativeSlot":slot,"queueY":i32::from(header.y)*2,"palette":header.palette,"sprite":if hardware&&!plain_hardware{None}else{Some(header.sprite)},"x":i32::from(if plain_hardware{header.x}else{p[0]})*2,"y":i32::from(if plain_hardware{header.y.wrapping_add(i16::from(header.offset))}else{p[1]})*2});
  if let Some(native)=&native{if !native.layers.is_empty(){row["nativeLayers"]=json!(native.layers.iter().map(|l|json!({"sprite":l.sprite,"x":i32::from(l.position[0])*2,"y":i32::from(l.position[1])*2,"palette":l.palette,"flags":l.flags})).collect::<Vec<_>>());}}
  if hardware{
   //Only fields consumed by the existing hardware artwork adapter are kept.
   //No path, roster, passenger, target, fire state or complete UNIT is copied.
   let cell=(u.y/32.).floor()as usize*self.map.width+(u.x/32.).floor()as usize;
   row["time"]=json!(self.time);row["art"]=json!({"type":u.kind,"x":i32::from(p[0])*2,"y":i32::from(p[1])*2,"team":u.team,"angle":u.angle,"aim":u.aim,"moving":u.moving,"nativeMoveFrame":u.native_move_frame,"nativeTerrainCode":self.map.terrain_codes.get(cell).copied().unwrap_or(0)&127,"hp":u.hp,"maxHp":u.max_hp,"cooldown":u.cooldown,"originalCombat":{"fireRate":u.original_combat.as_ref().map_or(80,|c|c.fire_rate)},"nativeHardware":u.native_hardware.as_ref().map(|h|json!({"hatchFrame":h.hatch_frame,"hatchBody":h.hatch_body}))});
  }
  Some((header,row))
 }
 ///Actual55f10 submission point after this UNIT's handler. Resolve a changed
 ///renderer ID by the same native slot, then keep its record through later
 ///ambient contacts, deaths, identity changes and frontend snapshots.
 pub(crate) fn capture_native_unit_display(&mut self,index:usize,original_slot:Option<usize>){
  let Some(slot)=original_slot else{return;};if !self.map.original||self.native_display.units.as_ref().is_some_and(|rows|rows.iter().any(|(_,v)|v["nativeSlot"].as_u64()==Some(slot as u64))){return;}let actor=self.units.iter().rposition(|u|u.native_slot==Some(slot)).unwrap_or(index);let Some(u)=self.units.get(actor)else{return;};
  let row=self.native_unit_display_row(u,slot);if row.is_some(){let white=self.units[actor].native_flags64&4!=0;self.units[actor].native_flags64&=!4;if white&&row.as_ref().is_some_and(|(_,v)|v.get("nativeLayers").is_some()){self.units[actor].native_mask98=7;}}
  if let(Some(rows),Some(row))=(&mut self.native_display.units,row){rows.push(row);}
 }
 pub(crate) fn native_display_queue(&self)->Vec<Value>{
  if !self.map.original{return vec![];}let mut pre=Vec::new();let mut queue=Vec::new();
  //460a00 precedes BUILD drawing. Its queued exhaust and direct ground
  //marks use different paths even though they share the500x6 allocator.
  for d in &self.native_smoke20.tracks.draws{let r=Record{x:d.position[0],y:d.position[1],sprite:d.sprite,palette:255,offset:0};if d.queued{queue.push((r,plain(r,"track",d.slot)));}else{let mut v=plain(r,"track",d.slot);v["stage"]=json!("pre");pre.push(v);}}
  pre.extend(self.native_display.buildings_before.clone().unwrap_or_else(||self.project_native_building_display()));
  queue.extend(self.native_display.radar_before.clone().unwrap_or_else(||self.project_native_radar_display()));
  let objects=self.native_display_objects();
  if let Some(rows)=&self.native_display.units{queue.extend(rows.iter().cloned());}else{
   //Before the first UNIT callback only, expose the imported initial picture.
   let mut units=self.units.iter().filter(|u|u.native_slot.is_some()&&u.team>=0&&original_combat::alive(true,u.hp)).collect::<Vec<_>>();units.sort_by_key(|u|u.native_slot);for u in units{if let Some(row)=self.native_unit_display_row(u,u.native_slot.unwrap()){queue.push(row);}}
  }
  //Main435d32 calls41d110 after5a240's UNITs,419fb0 and461d10.
  //Insertion order matters even at equal depth because545b0 is unstable.
  queue.extend(self.native_display.flags_before.clone().unwrap_or_else(||self.project_native_flag_display()));
  for d in &self.native_deaths.draws{let r=Record{x:d.position[0],y:d.position[1],sprite:d.sprite,palette:d.color,offset:0};queue.push((r,plain(r,"death",usize::from(d.slot))));}
  for o in objects{if let Some(r)=o.record{queue.push((r,plain(r,"object",usize::from(o.slot))));}}
  for d in &self.native_smoke20.draws{let r=Record{x:d.position[0],y:d.position[1],sprite:d.sprite,palette:255,offset:0};queue.push((r,plain(r,"smoke",usize::from(d.slot))));}
  effect_rows(&mut queue,"chunk",self.native_ambient_animation_visuals());effect_rows(&mut queue,"ambient",self.native_ambient_visuals());effect_rows(&mut queue,"lava",self.native_lava_visuals());
  for d in &self.native_bullet_impacts20.draws{let r=Record{x:d.position[0],y:d.position[1],sprite:d.sprite,palette:255,offset:0};queue.push((r,plain(r,"groundImpact",usize::from(d.slot))));}
  sort(&mut queue,|r|r.0.y);pre.extend(queue.into_iter().map(|(_,mut value)|{value["stage"]=json!("queue");value}));pre
 }
}
#[cfg(test)]mod tests{
 use super::*;use crate::{original_group_game,original_death};
 fn field()->Game{let mut g=Game::new(0,1,36739);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![0;4096];g.map.native_terrain_plane=vec![0;16384];g.units.clear();g.buildings.clear();g.objects.clear();g.sectors.clear();g.crates.clear();g.walls.clear();g.native_world=None;g.native_group_rosters=original_group_game::State::default();g.native_deaths=Default::default();g.native_smoke20=Default::default();g.native_bullet_impacts20=Default::default();g.native_ambient=None;g.native_ambient_animations=Default::default();g.native_lava=Default::default();g.next_id=1;g}
 #[test]fn native_primary_rows_share_depth_and_do_not_draw_a_neutral_object_as_a_second_unit(){
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.spawn("grunt",1,1000.,1000.,2,None);g.spawn("medium",-1,800.,1000.,3,None);let first=g.units[0].id;g.units[0].native_elevation76=-32;g.units[0].native_phase35=23;g.units[0].native_flags64|=4;g.objects=vec![json!({"type":8,"equipmentId":1,"nativeSlot":0,"nativeSector":0,"x":800,"y":1000,"hp":20})];g.import_native_world(&Value::Null);g.maintain_native_group_rosters();g.native_deaths.draws.push(original_death::Draw{slot:0,position:[500,500],screen:[500,500],sprite:66,color:1});
  let rows=g.native_display_queue();assert_eq!(rows.iter().filter(|r|r["kind"]=="unit").count(),2);let u=rows.iter().find(|r|r["kind"]=="unit"&&r["id"]==first).unwrap();assert_eq!(u["queueY"],936);assert_eq!(u["y"],1000);assert_eq!(u["palette"],4);assert_eq!(u["sprite"],315);assert_eq!(g.units[0].team,0);let object=rows.iter().find(|r|r["source"]=="object").unwrap();assert_eq!(object["x"],800);assert_eq!(object["y"],1000);assert_eq!(object["palette"],8);assert_eq!(rows.iter().filter(|r|r["source"]=="object").count(),1);assert!(rows.windows(2).all(|p|p[0]["queueY"].as_i64()<=p[1]["queueY"].as_i64()));
 }
 #[test]fn direct_buildings_precede_common_queue_and_raw_phase65_needs_no_legacy_fire_state(){
  let template=Game::new(0,1,3).buildings[0].clone();let mut g=field();let mut b=template;b.id=900;b.kind="robot".into();b.x=1000.;b.y=1000.;g.buildings.push(b);g.spawn("pyro",0,1000.,1000.,1,None);g.units[0].native_phase35=65;g.units[0].native_frame32=2;g.units[0].angle=crate::native_math::angle(3);g.import_native_world(&Value::Null);g.maintain_native_group_rosters();let rows=g.native_display_queue();assert_eq!(rows[0]["stage"],"pre");assert_eq!(rows[0]["kind"],"building");assert_eq!(rows[0]["id"],900);let unit=rows.iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(unit["sprite"],6033);assert!(g.units[0].native_fire.is_none());
 }
 #[test]fn flag_cloth_and_shadow_use_the_actual_global_frame_and_distinct_queue_depths(){
  let mut sector=Game::new(0,1,3).sectors[0].clone();sector.id=7;sector.owner=1;let mut g=field();g.sectors.push(sector);g.import_native_world(&json!({"sectors":[{"id":7,"nativeSlot":0,"rawFlagTile":[20,21]}]}));g.native_flag_frame=3;g.motion_tick=0;let rows=g.native_display_queue();let cloth=rows.iter().find(|r|r["source"]=="flag").unwrap();let shadow=rows.iter().find(|r|r["source"]=="flagShadow").unwrap();assert_eq!(cloth["sprite"],483);assert_eq!(shadow["sprite"],487);assert_eq!((cloth["queueY"].as_i64(),shadow["queueY"].as_i64()),(Some(736),Some(640)));assert_eq!(cloth["y"],672);assert_eq!(shadow["y"],672);assert_eq!(cloth["palette"],1);assert_eq!(shadow["palette"],1);
 }
 #[test]fn queued_robot_picture_survives_later_hits_motion_and_renderer_retirement_for_this_frame_only(){
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.maintain_native_group_rosters();g.units[0].native_phase35=23;g.units[0].native_flags64|=4;let id=g.units[0].id;let slot=g.units[0].native_slot.unwrap();g.begin_native_display_frame();let token=g.native_unit_display_slot(0);assert_eq!(token,Some(slot));g.capture_native_unit_display(0,token);assert_eq!(g.units[0].native_flags64&4,0);
  let before=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(before["palette"],4);assert_eq!(before["sprite"],315);g.units[0].x=1400.;g.units[0].native_phase35=19;g.units[0].hp=-1.;g.units[0].native_slot=None;g.native_group_rosters.members[slot]=None;g.units.clear();let after=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(before,after);assert_eq!(after["id"],id);assert_eq!(after["nativeSlot"],slot);assert!(after.get("art").is_none());g.begin_native_display_frame();assert!(!g.native_display_queue().iter().any(|r|r["kind"]=="unit"));
 }
 #[test]fn draw_submission_obeys_frozen_visibility_and_signed_viewport_gates_before_clearing_white_flash(){
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.maintain_native_group_rosters();let slot=g.units[0].native_slot.unwrap();g.native_view=Some(([0,0],[1024,1024]));g.units[0].native_flags64|=4;g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=64;g.begin_native_display_frame();let frozen=g.native_unit_display_slot(0);assert_eq!(frozen,None);g.capture_native_unit_display(0,frozen);assert_eq!(g.units[0].native_flags64&4,4);
  g.native_group_rosters.members[slot].as_mut().unwrap().flags69=0;let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);assert_eq!(g.units[0].native_flags64&4,4);g.native_group_rosters.members[slot].as_mut().unwrap().flags69=4;g.units[0].x=-64.;let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);assert_eq!(g.units[0].native_flags64&4,4);g.units[0].x=-62.;let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);assert_eq!(g.units[0].native_flags64&4,0);assert_eq!(g.native_display.units.as_ref().unwrap().len(),1);
 }
 #[test]fn hardware_submission_keeps_only_the_art_inputs_at_its_own_callback(){
  let mut g=field();g.time=1.25;g.spawn("medium",1,1000.,1000.,1,None);g.maintain_native_group_rosters();g.units[0].native_hardware=Some(crate::original_hardware::State::new(3,-1));g.begin_native_display_frame();let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);let before=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(before["time"],1.25);assert_eq!(before["art"]["nativeHardware"]["hatchFrame"],-1);assert_eq!(before["art"]["nativeTerrainCode"],0);for key in["path","target","cargo","nativeFire","squad","order"]{assert!(before["art"].get(key).is_none());}
  g.time=8.;g.units[0].hp=1.;g.units[0].angle=crate::native_math::angle(7);g.units[0].aim=crate::native_math::angle(4);g.units[0].native_hardware.as_mut().unwrap().hatch_frame=4;g.units[0].native_move_frame=3;let cell=31*g.map.width+31;g.map.terrain_codes[cell]=7;let after=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(before,after);
 }
 #[test]fn tank_capture_retains_masked_body_unmasked_hatch_and_raw_track_frame_then_resets_once(){
  let mut g=field();g.spawn("medium",1,1000.,1000.,1,None);g.maintain_native_group_rosters();g.units[0].native_phase35=9;g.units[0].native_frame32=2;g.units[0].native_move_frame=0;g.units[0].native_flags64=21;g.units[0].native_mask98=3;g.units[0].native_hardware=Some(crate::original_hardware::State::new(0,-1));g.units[0].angle=crate::native_math::angle(0);g.begin_native_display_frame();let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);let row=g.native_display_queue().into_iter().find(|v|v["kind"]=="unit").unwrap();let layers=row["nativeLayers"].as_array().unwrap();assert_eq!(layers.len(),4);assert_eq!(layers[0]["sprite"],490);assert_eq!(layers[0]["palette"],1);assert_eq!(layers[3]["palette"],5);assert_eq!(layers[3]["flags"],1);assert_eq!((g.units[0].native_flags64,g.units[0].native_mask98),(17,7));g.units[0].native_frame32=0;g.units[0].native_hardware.as_mut().unwrap().turret=7;assert_eq!(g.native_display_queue().into_iter().find(|v|v["kind"]=="unit").unwrap(),row,"later state cannot rewrite already submitted layers");
 }
 #[test]fn gun_source_pose_uses_phase_frame_and_84_offsets_instead_of_weapon_cooldown(){
  let mut g=field();g.spawn("gatling",1,1000.,1000.,1,None);g.maintain_native_group_rosters();g.units[0].native_class7c=2;g.units[0].native_phase35=51;g.units[0].native_frame32=1;g.units[0].native_counter84=-1;g.units[0].angle=crate::native_math::angle(0);g.units[0].cooldown=999.;g.begin_native_display_frame();let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);let row=g.native_display_queue().into_iter().find(|v|v["kind"]=="unit").unwrap();assert_eq!(row["sprite"],1660);assert_eq!((row["x"].as_i64(),row["y"].as_i64()),(Some(1002),Some(996)));g.units[0].native_counter84=3;g.units[0].cooldown=0.;g.begin_native_display_frame();let token=g.native_unit_display_slot(0);g.capture_native_unit_display(0,token);assert_eq!(g.native_display_queue().into_iter().find(|v|v["kind"]=="unit").unwrap()["sprite"],2419);
 }
 #[test]fn special_build_clipping_stays_an_explicit_artwork_boundary(){let mut g=field();g.spawn("medium",0,640.,640.,1,None);g.maintain_native_group_rosters();g.units[0].native_cell10=0;g.map.terrain_codes[0]=31;let rows=g.native_display_queue();let row=rows.iter().find(|r|r["kind"]=="unit").unwrap();assert!(row.get("nativeLayers").is_none());assert!(row.get("art").is_some());}
 #[test]fn real_unit_loop_captures_after_an_early_handler_break_and_does_not_clear_flash_on_snapshot(){
  let mut g=field();g.ai_teams.clear();g.spawn("grunt",0,1000.,1000.,1,None);g.maintain_native_group_rosters();g.units[0].native_flags64|=4;g.units[0].native_phase35=33;g.units[0].native_frame32=0;g.units[0].native_timer3c=5;g.step(0.059);let row=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(row["palette"],4);assert_eq!(g.units[0].native_flags64&4,0);g.units[0].native_flags64|=4;let again=g.native_display_queue().into_iter().find(|r|r["kind"]=="unit").unwrap();assert_eq!(row,again);assert_eq!(g.units[0].native_flags64&4,4);
 }
 #[test]fn all_800_executed_callback_tail_visibility_retirement_and_retained_record_cases_match(){
  fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
  fn records_hash(g:&Game)->u32{let mut h=2166136261u32;for(r,_)in g.native_display.units.as_ref().unwrap(){mix(&mut h,r.bytes().map(u32::from));}h}
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.maintain_native_group_rosters();let slot=g.units[0].native_slot.unwrap();let base=g.native_group_rosters.members[slot].as_ref().unwrap().clone();g.native_view=Some(([0,0],[1024,1024]));let mut hash=2166136261u32;let mut scenario=0u32;
  for x in[-32,-31,100,1023,1024i16]{for y in[-48,-47,100,1023,1024i16]{for flags69 in[0,4,64,68u8]{for remove in[false,true]{for flags64 in[0,4,5,255u8]{
   let mut m=base.clone();m.flags31=4;m.flags69=flags69;g.native_group_rosters.members[slot]=Some(m);let u=&mut g.units[0];u.native_slot=Some(slot);u.hp=40.;u.x=f64::from(x)*2.;u.y=f64::from(y)*2.;u.native_phase35=1;u.native_frame32=0;u.angle=crate::native_math::angle(3);u.native_flags64=flags64;
   g.begin_native_display_frame();let token=g.native_unit_display_slot(0);let calls=u32::from(token.is_some());if calls!=0&&remove{g.units[0].hp=-1.;g.units[0].native_slot=None;g.native_group_rosters.members[slot]=None;}g.capture_native_unit_display(0,token);let bytes=records_hash(&g);let size=g.native_display.units.as_ref().unwrap().len()as u32*8;let after=g.units[0].native_flags64;g.units[0].x=1400.;g.units[0].y=1400.;g.units[0].native_phase35=18;if let Some(m)=g.native_group_rosters.members[slot].as_mut(){m.flags69=0;}let retained=records_hash(&g);mix(&mut hash,[scenario,x as u32,y as u32,u32::from(flags69),u32::from(remove),u32::from(flags64),calls,size,bytes,u32::from(after),retained]);scenario+=1;
  }}}}}
  assert_eq!(scenario,800);assert_eq!(hash,2839048781,"real5a080→55f10; world/order/handler bodies are explicit boundaries, including same-handler58ec0-style raw slot retirement");
 }
}
