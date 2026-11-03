//! Adapter between verified native strategic kernels and the browser world.
//! The kernels retain their original numerical rules. Live routing, restricted
//! terrain capability and the remaining task-dispatch stages stay explicit.
use super::{Game,Unit,stats,original_combat,original_ai,native_ai_scoring as score,native_ai_tasks as task};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};

pub struct State{
 sectors:Vec<task::Sector>,population:Vec<(usize,i8)>,building_values:Vec<(usize,i32)>,
 initial:Vec<u8>,control:i16,reserve:i8,population_weight:i8,
 groups:BTreeMap<i32,Vec<task::Group>>,
 retries:BTreeMap<i32,Vec<i8>>,
 scoring_flags:Vec<(usize,u8)>,priorities:BTreeMap<i32,BTreeMap<usize,i32>>,
 distance_bias:bool,
 factory_budgets:BTreeMap<i32,(f32,f32)>,
 local_retries:BTreeMap<i32,Vec<i8>>,
 frontier_retries:BTreeMap<i32,Vec<i8>>,
 building_meta:BTreeMap<usize,(i8,[i8;2])>,
 objects:Vec<task::WorldObject>,
 object_bindings:BTreeSet<usize>,
 terrain_plane:Vec<u8>,
 object_restriction_cleared:BTreeSet<usize>,
 keep_object_restriction:bool,
 presence:BTreeMap<i32,BTreeSet<usize>>,
}
impl State{
 pub fn from_mission(v:&Value)->Option<Self>{
  let settings=&v["nativeAi"];let tasks=settings["initialSectorTasks"].as_array()?;
  let mut sectors=Vec::new();let mut population=Vec::new();let mut scoring_flags=Vec::new();
  for s in v["sectors"].as_array()?{
   if s["countsForProduction"]==false{continue;}
   let r=s["rect"].as_array()?;if r.len()!=4{return None;}
   let rect=[r[0].as_i64()? as i16,r[1].as_i64()? as i16,r[2].as_i64()? as i16,r[3].as_i64()? as i16];
   let id=s["id"].as_u64()? as usize;let native_slot=s["nativeSlot"].as_u64()? as usize;
   let flag=[s["rawFlagTile"][0].as_u64()? as u8,s["rawFlagTile"][1].as_u64()? as u8];
   let flags=s["nativeFlags"].as_u64()? as u8&0xe0;
   sectors.push(task::Sector{id,native_slot,valid:true,rect,owner:s["owner"].as_i64().unwrap_or(-1)as i8,flag,flags,human_strength:0,priority:s["nativePrioritySeed"].as_i64().unwrap_or(0)as i32,rank:0});
   population.push((id,s["nativePopulation"].as_i64().unwrap_or(0)as i8));
   scoring_flags.push((id,s["nativeScoringFlags"].as_u64().unwrap_or(0)as u8));
  }
  sectors.sort_by_key(|s|s.native_slot);
  if let Some(plane)=settings["terrainPlane"].as_array().filter(|p|p.len()==128*128){
   let codes=plane.iter().map(|n|n.as_u64().unwrap_or(0)as u8).collect::<Vec<_>>();
   let lookup=sectors.clone();
   for s in &mut sectors{
    let enclosed=score::enclosed_infantry_area(&codes,[s.flag[0]as i8,s.flag[1]as i8],|x,y|{
     let(x,y)=(x>>4,y>>4);[(0,0),(1,1)].into_iter().find_map(|(dx,dy)|lookup.iter().find(|s|s.valid&&x+dx>=s.rect[0]&&x+dx<s.rect[2]&&y+dy>=s.rect[1]&&y+dy<s.rect[3]).map(|s|s.id))
    });
    s.flags=(s.flags&!64)|if enclosed{64}else{0};
   }
  }
  let mut initial=Vec::new();for t in tasks{initial.push(t["sector"].as_u64().map(|n|(n+1)as u8).unwrap_or(0));}
  let building_values=v["buildings"].as_array()?.iter().filter_map(|b|Some((b["id"].as_u64()? as usize,
   if b["kind"]=="fort"{b["stars"].as_i64().unwrap_or(0)as i32*1000}else{b["rawTechnology"].as_i64().unwrap_or(0)as i32}))).collect();
  let building_meta=v["buildings"].as_array()?.iter().filter_map(|b|Some((b["id"].as_u64()? as usize,(b["rawType"].as_i64()? as i8,[b["rawAttackOffset"][0].as_i64()? as i8,b["rawAttackOffset"][1].as_i64()? as i8])))).collect();
  let terrain_plane=settings["terrainPlane"].as_array().filter(|p|p.len()==128*128).map(|p|p.iter().map(|v|v.as_u64().unwrap_or(0)as u8).collect::<Vec<_>>()).unwrap_or_default();
  let mut state=Self{sectors,population,building_values,initial,control:settings["decisionControl"].as_i64()? as i8 as i16,
   reserve:settings["reservePercent"].as_i64()? as i8,population_weight:settings["populationWeight"].as_i64()? as i8,groups:BTreeMap::new(),retries:BTreeMap::new(),scoring_flags,priorities:BTreeMap::new(),distance_bias:true,factory_budgets:BTreeMap::new(),local_retries:BTreeMap::new(),frontier_retries:BTreeMap::new(),building_meta,objects:Vec::new(),object_bindings:BTreeSet::new(),terrain_plane,object_restriction_cleared:BTreeSet::new(),keep_object_restriction:settings["levelRecordIndex"]==18,presence:BTreeMap::new()};
  for(index,o)in v["originalObjects"].as_array().into_iter().flatten().enumerate(){
   let Some(definition)=o["type"].as_i64().filter(|t|*t!=37)else{continue;};
   let position=[(o["x"].as_f64().unwrap_or(0.)/2.)as i16,(o["y"].as_f64().unwrap_or(0.)/2.)as i16];
   let bound=v["spawns"].as_array().and_then(|spawns|spawns.iter().position(|u|u["source"]=="OBJECT_equipment"&&u["sourceOffset"]==o["sourceOffset"])).map(|i|i+1);
   let id=bound.unwrap_or(65536+index);if bound.is_some(){state.object_bindings.insert(id);}
   let mut flags13=if matches!(definition,5|21|22|23|35|26|29|8|31|24|33|37){0x70}else if matches!(definition,11|12|17){0}else{0x80};
   if o["templateFlags"].as_u64().unwrap_or(0)&0x14!=0{flags13|=64;}
   let flags26=if state.terrain_plane.len()==128*128&&score::enclosed_object_area(&state.terrain_plane,position,|x,y|state.sector_at(f64::from(x)*2.,f64::from(y)*2.)){1}else{0};
   state.objects.push(task::WorldObject{id,position,definition:Some(definition as i16),flags13,flags26,sector:state.sector_at(f64::from(position[0])*2.,f64::from(position[1])*2.)});
  }
  //17a50 initializes presence once; ordinary captures retain that sector flag.
  for team in [0,1]{let buildings=v["buildings"].as_array()?.iter().filter_map(|b|Some(task::BuildingRecord{id:b["id"].as_u64()? as usize,position:[(b["x"].as_f64()?/2.)as i16,(b["y"].as_f64()?/2.)as i16],kind:b["rawType"].as_i64()? as i8,owner:if b["team"].as_i64()?==team{1}else{0},sector:Some(b["sector"].as_u64()? as usize),hit_points:b["hp"].as_i64()? as i16,attack_offset:[0,0]})).collect::<Vec<_>>();state.presence.insert(team as i32,state.sectors.iter().filter(|s|task::available_hardware(s.id,&buildings,&state.objects).is_some()).map(|s|s.id).collect());}
  Some(state)
 }
 pub(crate) fn begin_callback(&mut self){for groups in self.groups.values_mut(){for group in groups{group.flags&=!1;}}}
 pub fn captured(&mut self,team:i32,sector:usize){self.distance_bias=score::distance_bias_after_capture(self.distance_bias,team as u8,0,true);if !self.keep_object_restriction{self.object_restriction_cleared.insert(sector);}for groups in self.groups.values_mut(){task::reset_captured_task_phases(groups,sector);}}
 fn buildings(&self,g:&Game,team:i32)->Vec<task::BuildingRecord>{if let Some(buildings)=g.native_ai_world_buildings(team){return buildings;}g.buildings.iter().filter_map(|b|{let(kind,offset)=self.building_meta.get(&b.id)?;Some(task::BuildingRecord{id:b.id,position:[(b.x/2.)as i16,(b.y/2.)as i16],kind:*kind,owner:if b.team==team{1}else if b.team>=0{0}else{4},sector:Some(b.sector),hit_points:b.hp as i16,attack_offset:*offset})}).collect()}
 fn world_objects(&self,g:&Game)->Vec<task::WorldObject>{
  if let Some(objects)=g.native_ai_world_objects(){return objects;}
  let mut objects=self.objects.iter().filter(|o|!self.object_bindings.contains(&o.id)).cloned().collect::<Vec<_>>();
  for u in g.units.iter().filter(|u|u.team<0&&original_combat::alive(true,u.hp)&&stats(&u.kind).cls>0){
   let definition=match u.kind.as_str(){"cannon"=>5,"medium"=>8,"missile"=>21,"gatling"=>22,"howitzer"=>23,"launcher"=>24,"heavy"=>26,"light"=>29,"apc"=>31,"jeep"=>33,"crane"=>35,"firetruck"=>37,_=>continue};
   let position=[(u.x/2.)as i16,(u.y/2.)as i16];let sector=self.sector_at(u.x,u.y);
   let original=self.objects.iter().find(|o|o.id==u.id&&o.position==position);
   let flags26=original.map(|o|o.flags26).unwrap_or_else(||if self.terrain_plane.len()==128*128&&score::enclosed_object_area(&self.terrain_plane,position,|x,y|self.sector_at(f64::from(x)*2.,f64::from(y)*2.)){1}else{0});
   let reserved=g.units.iter().any(|v|v.order=="enter"&&v.target==Some(u.id));objects.push(task::WorldObject{id:u.id,position,definition:Some(definition),flags13:0x70|u8::from(reserved),flags26,sector});
  }
  for o in &mut objects{if o.sector.is_some_and(|s|self.object_restriction_cleared.contains(&s)){o.flags26&=!1;}}
  // OBJECT slots, not spatial distance, decide the first available pickup.
  objects.sort_by_key(|o|self.objects.iter().position(|v|v.id==o.id).unwrap_or(self.objects.len()+o.id));objects
 }
 pub(crate) fn sector_at(&self,x:f64,y:f64)->Option<usize>{
  let x=(x/32.).floor()as i16;let y=(y/32.).floor()as i16;
  for (dx,dy) in [(0,0),(1,1)]{if let Some(s)=self.sectors.iter().find(|s|s.valid&&x+dx>=s.rect[0]&&x+dx<s.rect[2]&&y+dy>=s.rect[1]&&y+dy<s.rect[3]){return Some(s.id);}}
  None
 }
 pub(crate) fn sector_pointer(&self,x:f64,y:f64)->Option<u32>{let id=self.sector_at(x,y)?;self.sectors.iter().find(|s|s.id==id).map(|s|0x5c3f0c+s.native_slot as u32*30)}
 fn unit_score(u:&Unit)->i32{
  let Some(mut input)=score::default_input(&u.kind)else{return 0;};
  input.grenades=u.grenades as i8;input.traits[4]=if stats(&u.kind).cls==0{u.hp as i8}else{u.crew.as_ref().map(|c|c.hp as i8).unwrap_or(0)};
  if let Some(c)=&u.original_combat{
   input.traits[1]=c.aim_skill as i8;input.traits[5]=c.base_aim(&u.kind)as i8;
   if stats(&u.kind).cls==0{input.fire_rate=c.fire_rate;}
   else{input.hardware.movement=c.movement_control;input.hardware.fire_rate=c.fire_rate;}
  }
  input.hardware.hp=u.hp as i16;
  let mut value=score::unit_score(&input);
  // Native boarding retains the real passenger units. Legacy Web carriers
  // only have Crew summaries, so their fallback is counted here instead.
  if u.transport_actor.is_none(){for member in &u.cargo{if let Some(mut cargo)=score::default_input(&member.kind){cargo.traits[4]=member.hp as i8;value=value.wrapping_add(score::unit_score(&cargo));}}}
  value
 }
 fn refresh_groups(&self,g:&Game,team:i32,groups:&mut Vec<task::Group>){
  let mut squads:BTreeMap<usize,Vec<&Unit>>=BTreeMap::new();
  for u in g.units.iter().filter(|u|u.team==team&&original_combat::alive(true,u.hp)&&!driver_duplicate(u,&g.units)){squads.entry(u.squad).or_default().push(u);}
  for old in groups.iter_mut(){old.active=false;old.leader=None;}
  let mut ordered=squads.into_iter().collect::<Vec<_>>();ordered.sort_by_key(|(_,members)|members.iter().map(|u|u.native_slot.unwrap_or(u.id)).min().unwrap());
  for(id,mut members)in ordered{
   let native_leader=g.native_group_rosters.leader_slot(team,id);
   members.sort_by_key(|u|(u.native_slot!=native_leader,super::workshops::hidden(u),u.native_slot.unwrap_or(u.id)));let lead=members[0];
   let current=self.sector_at(lead.x,lead.y);
   let value=members.iter().fold(0i32,|sum,u|sum.wrapping_add(Self::unit_score(u)));
   let value=original_ai::perturb(value,self.control,&g.original_rng);
   let grenades=score::group_can_demolish(&members.iter().map(|u|u.grenades as i8).collect::<Vec<_>>());
   let leader=task::Leader{native_slot:lead.native_slot.unwrap_or(lead.id),team:1,runtime_class:lead.native_class7c,family:original_combat::animation_family(&lead.kind),
    passenger_class:lead.crew.as_ref().map(|c|original_combat::runtime_class(&c.kind)).unwrap_or(0),position:[(lead.x/2.).floor()as i16,(lead.y/2.).floor()as i16],
    accepts_orders:g.original_ai_accepts_orders(lead),laser_can_cross:grenades};
   let found=groups.iter().position(|old|old.id==id).or_else(||groups.iter().position(|old|!old.active&&!squads_contains(&g.units,team,old.id)));
   let i=if let Some(i)=found{i}else{groups.push(task::Group{id,active:false,flags:0,task:-1,current_sector:None,via_sector:None,assigned_sector:None,value:0,leader:None});groups.len()-1};
   if groups[i].id!=id{groups[i]=task::Group{id,active:false,flags:0,task:-1,current_sector:None,via_sector:None,assigned_sector:None,value:0,leader:None};}
   let p=&mut groups[i];p.active=true;p.value=value;p.leader=Some(leader);task::refresh_group(p,current);
  }
  groups.sort_by_key(|group|g.native_group_rosters.group_slot(team,group.id).unwrap_or(usize::MAX));
 }
 fn strengths(&self,g:&Game)->BTreeMap<(usize,i32),i16>{
  let mut result=BTreeMap::new();
  for u in g.units.iter().filter(|u|u.team>=0&&original_combat::alive(true,u.hp)&&!super::workshops::hidden(u)&&!matches!(u.kind.as_str(),"crane"|"truck"|"firetruck")){
   if let Some(leader)=g.native_group_rosters.leader_slot(u.team,u.squad){if u.native_slot!=Some(leader){continue;}}
   else if g.units.iter().any(|v|v.team==u.team&&v.squad==u.squad&&original_combat::alive(true,v.hp)&&!super::workshops::hidden(v)&&v.native_slot.unwrap_or(v.id)<u.native_slot.unwrap_or(u.id)){continue;}
   let Some(sid)=self.sector_at(u.x,u.y)else{continue;};
   let sum=g.units.iter().filter(|v|v.team==u.team&&v.squad==u.squad&&original_combat::alive(true,v.hp)&&!driver_duplicate(v,&g.units)).fold(0i32,|sum,v|sum.wrapping_add(Self::unit_score(v)));
   let mut value=original_ai::perturb(sum,self.control,&g.original_rng);
   if self.scoring_flags.iter().find(|(id,_)|*id==sid).is_none_or(|(_,f)|f&1==0){value=original_ai::perturb(value,self.control,&g.original_rng);}
   let entry=result.entry((sid,u.team)).or_insert(0i16);*entry=entry.wrapping_add(value.min(65535)as i16);
  }
  result
 }
 fn sectors_for(&self,g:&Game,team:i32,strengths:&BTreeMap<(usize,i32),i16>)->Vec<task::Sector>{
  let mut sectors=self.sectors.clone();
  for s in &mut sectors{
   let owner=g.sectors.iter().find(|p|p.id==s.id).map(|p|p.owner).unwrap_or(-1);
   s.owner=if owner==team{1}else if owner>=0{0}else{4};s.flags&=!19;if self.presence.get(&team).is_some_and(|p|p.contains(&s.id)){s.flags|=16;}s.human_strength=0;
   if let Some(bases)=g.native_fort_bases(){for (i,base) in bases.iter().enumerate(){if self.sector_at(f64::from(base[0])*2.,f64::from(base[1])*2.)==Some(s.id){s.flags|=if (i==0)==(team==1){1}else{2};}}}else{for b in g.buildings.iter().filter(|b|b.sector==s.id&&b.kind=="fort"){s.flags|=if b.team==team{1}else{2};}}
  }
  for s in &mut sectors{s.human_strength=*strengths.get(&(s.id,1-team)).unwrap_or(&0);}
  let origin=sectors.iter().find(|s|s.flags&1!=0).map(|s|(s.flag[0]as i8,s.flag[1]as i8)).unwrap_or((0,0));
  for s in &mut sectors{
   let update=score::should_update_sector_priority(s.rect[0],g.original_ai.as_ref().is_some_and(|c|c.mode),self.scoring_flags.iter().find(|(id,_)|*id==s.id).map(|(_,f)|*f).unwrap_or(0));
   if !update{if let Some(value)=self.priorities.get(&team).and_then(|p|p.get(&s.id)){s.priority=*value;}s.rank=score::distance_priority(s.priority.max(0),(s.flag[0]as i8,s.flag[1]as i8),origin,self.distance_bias);continue;}
   let population=self.population.iter().find(|(id,_)|*id==s.id).map(|(_,p)|*p).unwrap_or(0);
   s.priority=score::sector_priority(population,self.population_weight,s.human_strength,&[],&[]);
   for b in g.buildings.iter().filter(|b|b.sector==s.id){if let Some((_,v))=self.building_values.iter().find(|(id,_)|*id==b.id){s.priority=s.priority.wrapping_add(original_ai::perturb(*v,self.control,&g.original_rng));}}
   for u in g.units.iter().filter(|u|u.team<0&&original_combat::alive(true,u.hp)&&self.sector_at(u.x,u.y)==Some(s.id)){
    let Some(mut input)=score::default_input(&u.kind)else{continue;};input.hardware.hp=(u.hp/25.)as i16;
    let object_type=match u.kind.as_str(){"cannon"=>5,"medium"=>8,"missile"=>21,"gatling"=>22,"howitzer"=>23,"launcher"=>24,"heavy"=>26,"light"=>29,"apc"=>31,"jeep"=>33,"crane"=>35,_=>continue};
    s.priority=s.priority.wrapping_add(original_ai::perturb(score::object_score(object_type,input.hardware),self.control,&g.original_rng));
   }
   s.rank=score::distance_priority(s.priority.max(0),(s.flag[0]as i8,s.flag[1]as i8),origin,self.distance_bias);
  }
  sectors
 }
}
fn squads_contains(units:&[Unit],team:i32,squad:usize)->bool{units.iter().any(|u|u.team==team&&u.squad==squad&&original_combat::alive(true,u.hp))}
fn driver_duplicate(u:&Unit,units:&[Unit])->bool{u.transport_carrier.is_some_and(|id|units.iter().any(|c|c.id==id&&c.transport_actor==Some(u.id)))}

impl Game{
 pub(crate) fn native_factory_level_record_index(&self)->Option<usize>{(self.map.original&&self.map.native_road_graph.is_some()).then(||usize::try_from(self.map.native_level_record).ok()).flatten()}
 pub(crate) fn native_factory_sector(&self,id:usize)->Option<task::Sector>{
  let state=self.native_strategy.as_ref()?;let mut sector=state.sectors.iter().find(|s|s.id==id)?.clone();let owner=self.sectors.iter().find(|s|s.id==id)?.owner;sector.owner=if owner==1{1}else if owner>=0{0}else{4};sector.flags&=!19;
  if state.presence.get(&1).is_some_and(|ids|ids.contains(&id)){sector.flags|=16;}
  if let Some(bases)=self.native_fort_bases(){for (i,base) in bases.iter().enumerate(){if state.sector_at(f64::from(base[0])*2.,f64::from(base[1])*2.)==Some(id){sector.flags|=1<<i;}}}else{for b in self.buildings.iter().filter(|b|b.kind=="fort"&&b.sector==id){sector.flags|=if b.team==1{1}else{2};}}
  if let Some(priority)=state.priorities.get(&1).and_then(|p|p.get(&id)){sector.priority=*priority;}
  Some(sector)
 }
 pub(crate) fn native_factory_technology(&self,id:usize)->Option<i32>{self.native_strategy.as_ref()?.building_values.iter().find(|(b,_)|*b==id).map(|(_,v)|*v)}
 ///150a0's exact authored BUILD impact point, available only with imported
 ///native descriptor metadata. UI target-only requests can use this fallback.
 pub(crate) fn native_building_target_event(&self,id:usize)->Option<super::original_orders_game::Event>{let state=self.native_strategy.as_ref()?;let(_,offset)=state.building_meta.get(&id)?;let b=self.buildings.iter().find(|b|b.id==id)?;let base=b.raw_base.unwrap_or((b.x/2.,b.y/2.));Some(super::original_orders_game::building_target_event([(base.0 as i16).wrapping_add(i16::from(offset[0])),(base.1 as i16).wrapping_add(i16::from(offset[1]))]))}
 pub(crate) fn native_factory_strategy_snapshot(&self,team:i32,sector:usize)->Option<(f32,f32,i32,i32,i32)>{
  let state=self.native_strategy.as_ref()?;let &(attack,reserve)=state.factory_budgets.get(&team)?;let priority=*state.priorities.get(&team)?.get(&sector)?;
  let strengths=state.strengths(self);let human=strengths.iter().filter(|((_,owner),_)|*owner!=team).fold(0i32,|sum,(_,&v)|sum.wrapping_add(i32::from(v)));let cpu=strengths.iter().filter(|((_,owner),_)|*owner==team).fold(0i32,|sum,(_,&v)|sum.wrapping_add(i32::from(v)));
  Some((attack,reserve,priority,human,cpu))
 }
 pub(crate) fn update_original_strategy(&mut self,initial:bool){
  self.maintain_native_group_rosters();
  let Some(mut state)=self.native_strategy.take()else{return;};
  for team in self.ai_teams.clone(){
   let mut groups=state.groups.remove(&team).unwrap_or_default();state.refresh_groups(self,team,&mut groups);
   let strengths=state.strengths(self);let sectors=state.sectors_for(self,team,&strengths);
   let buildings=state.buildings(self,team);let objects=state.world_objects(self);
   state.priorities.insert(team,sectors.iter().map(|s|(s.id,s.priority)).collect());
   if initial{
    if team==1{for order in task::initial_tasks(&mut groups,&state.initial,&sectors){self.issue_original_sector_order(team,&order,&sectors);}}
    //17b30 invokes16e10 before15d60. Group valuation is fresh; the
    //pressure globals retain their previous process/mission values.
    state.refresh_groups(self,team,&mut groups);
    let values=groups.iter().map(|g|if !g.active||g.leader.as_ref().is_none_or(|l|matches!(l.family,36|38)||l.runtime_class==2&&g.current_sector.and_then(|id|sectors.iter().find(|s|s.id==id)).is_none_or(|s|s.flags&1==0)){-1}else{g.value}).collect::<Vec<_>>();
    let total=values.iter().filter(|&&v|v!=-1).fold(0i32,|sum,&v|sum.wrapping_add(v));let(surplus,deficit)=self.native_pressure.get(&team).copied().unwrap_or((0.,0.));
    let(budget,attack)=score::reserve_budget(total,state.reserve,surplus,deficit);state.factory_budgets.insert(team,(attack,budget));task::select_budget_groups(&mut groups,&values,budget);
    let fresh_strengths=state.strengths(self);let fresh_sectors=state.sectors_for(self,team,&fresh_strengths);state.priorities.insert(team,fresh_sectors.iter().map(|s|(s.id,s.priority)).collect());
   }else{
    let values=groups.iter().map(|g|if !g.active||g.leader.as_ref().is_none_or(|l|matches!(l.family,36|38)||l.runtime_class==2&&g.current_sector.and_then(|id|sectors.iter().find(|s|s.id==id)).is_none_or(|s|s.flags&1==0)){-1}else{g.value}).collect::<Vec<_>>();
    let total=values.iter().filter(|&&v|v!=-1).fold(0i32,|sum,&v|sum.wrapping_add(v));
    let pressure=sectors.iter().map(|s|score::PressureSector{present:s.valid,flags:s.flags,human:s.human_strength,cpu:*strengths.get(&(s.id,team)).unwrap_or(&0)}).collect::<Vec<_>>();
    let(surplus,deficit)=score::territory_pressure(&pressure,state.reserve);self.native_pressure.insert(team,(surplus,deficit));
    let(budget,attack)=score::reserve_budget(total,state.reserve,surplus,deficit);state.factory_budgets.insert(team,(attack,budget));
    let released=task::select_budget_groups(&mut groups,&values,budget);
    self.apply_original_strategy_changes(team,task::release_budget_groups(&mut groups,&released,&sectors,sectors.iter().find(|s|s.flags&1!=0).map(|s|s.id)),&sectors);
    for order in task::allocate_attacks(&mut groups,&sectors,false){self.issue_original_sector_order(team,&order,&sectors);}
    self.apply_original_strategy_changes(team,task::reinforce_attacks(&mut groups,&sectors,false),&sectors);
    let requirements=sectors.iter().map(|s|(s.id,score::defense_priority(s.priority,budget,attack))).collect::<Vec<_>>();
    task::cancel_unneeded_defenses(&mut groups,&sectors,&requirements);
    let cpu_fort=self.original_cpu_fort(team);
    //15000 interleaves arrival, personal capture and assault for each group.
    // Apply resulting paths immediately: later native slots observe them.
    for native_slot in 0..240{
     let Some(index)=groups.iter().position(|g|self.native_group_rosters.group_slot(team,g.id)==Some(native_slot))else{continue;};
     let ttls=groups.iter().map(|g|(g.id,self.units.iter().filter(|u|u.team==team&&u.squad==g.id).map(|u|u.group_activity_ticks).max().unwrap_or(0))).collect::<Vec<_>>();
     match task::capture_arrival_for_group(&mut groups,index,&sectors,&ttls){
      Some(task::Arrival::Captured{sector,..})=>{self.issue_original_hardware_capture(team,index,&mut groups,&sectors,&buildings,&objects,cpu_fort);if let Some(s)=sectors.iter().find(|s|s.id==sector){let required=requirements.iter().find(|(id,_)|*id==sector).map(|(_,r)|*r).unwrap_or(0);for order in task::defend_captured_sector(&mut groups,s,required){self.issue_original_sector_order(team,&order,&sectors);}}},
      Some(task::Arrival::Capture{group,member,sector})=>{if let Some(s)=sectors.iter().find(|s|s.id==sector){let position=if s.flags&2!=0{[cpu_fort[0].wrapping_add(80),cpu_fort[1].wrapping_add(72)]}else{[(i16::from(s.flag[0]as i8)<<4).wrapping_add(4),(i16::from(s.flag[1]as i8)<<4).wrapping_add(4)]};if s.flags&2==0{self.issue_original_move_event(team,group,position,1);}else{self.issue_original_personal_capture(team,&task::PersonalCapture{group,member,position,personal:false,preserve_path:false},sector);self.issue_original_personal_capture(team,&task::PersonalCapture{group,member,position:[position[0],position[1].wrapping_sub(120)],personal:false,preserve_path:true},sector);}}},
      None=>{},
     }
     let runtime=self.original_capture_runtime(team,&groups,cpu_fort);
     for request in task::personal_capture_for_group(&mut groups,index,&sectors,&runtime,cpu_fort){let sector=groups[index].current_sector.unwrap_or(0);self.issue_original_personal_capture(team,&request,sector);}
     let controls=self.original_assault_controls(team,&groups);
     self.apply_original_strategy_changes(team,task::advance_assault_for_group(&mut groups,index,&sectors,&controls,cpu_fort),&sectors);
     self.issue_original_hardware_capture(team,index,&mut groups,&sectors,&buildings,&objects,cpu_fort);
    }
    let queries=groups.iter().filter_map(|g|{let s=g.current_sector.and_then(|id|sectors.iter().find(|s|s.id==id))?;let x=(i16::from(s.flag[0]as i8).wrapping_sub(4)).wrapping_mul(16);let y=(i16::from(s.flag[1]as i8).wrapping_sub(4)).wrapping_mul(16);
     if let Some(units)=self.original_rectangle_units(team,[x,y,x.wrapping_add(128),y.wrapping_add(128)],&state){return Some((g.id,units));}
     //Graph-less compatibility fixtures have no456a90 native bucket snapshot.
     let units=self.units.iter().filter(|u|u.team>=0&&original_combat::alive(true,u.hp)&&!super::workshops::hidden(u)).filter_map(|u|{let slot=u.native_slot?;let member=self.native_group_rosters.members[slot].as_ref()?;let pos=[(u.x/2.)as i16,(u.y/2.)as i16];let offset=if u.native_class7c==1{16}else{8};let center=[i32::from(pos[0])+offset,i32::from(pos[1])+offset];(center[0]>=i32::from(x)&&center[0]<=i32::from(x)+128&&center[1]>=i32::from(y)&&center[1]<=i32::from(y)+128).then_some(task::RuntimeUnit{native_slot:slot,position:pos,flags31:(member.flags31&!3)|if u.team==team{1}else{0},flags8a:member.flags8a,sector:state.sector_at(u.x,u.y)})}).collect::<Vec<_>>();Some((g.id,units))}).collect::<Vec<_>>();
    for attack in task::attack_target_buildings(&mut groups,&sectors,&buildings,&queries){let ids=self.original_strategy_ids(team,attack.group);self.command(&serde_json::json!({"team":team,"kind":"attackBuilding","ids":ids,"target":attack.building,"nativeEvent":false}));if let Some(index)=self.units.iter().position(|u|u.team==team&&u.native_slot==Some(attack.member)){self.record_native_order_event(index,super::original_orders_game::building_target_event(attack.position));}}
    task::retire_missing_hardware(&mut groups,&objects);
    let mut retries=self.original_task_counters(team,&groups,20);
    let runtime=self.original_group_runtime(team,&groups);
    let can_retry=groups.iter().map(|g|runtime.iter().find(|r|r.group==g.id).is_some_and(|r|r.native_order==0&&!r.pending)&&g.leader.as_ref().and_then(|l|self.units.iter().find(|u|u.native_slot==Some(l.native_slot)&&u.team==team)).is_some_and(|u|u.path.is_empty())).collect::<Vec<_>>();
    task::retry_stalled_tasks(&mut groups,&mut retries,&can_retry);self.store_original_task_counters(team,&groups,20,&retries);
    let mut personal=self.units.iter().filter(|u|u.team==team).filter_map(|u|{let slot=u.native_slot?;let member=self.native_group_rosters.members[slot].as_ref()?;Some(task::RuntimeUnit{native_slot:slot,position:[(u.x/2.)as i16,(u.y/2.)as i16],flags31:member.flags31,flags8a:member.flags8a,sector:state.sector_at(u.x,u.y)})}).collect::<Vec<_>>();
    for slot in task::finish_personal_routes(&mut personal,&sectors){self.rejoin_original_member(slot);}
    for order in task::assign_defense(&mut groups,&sectors,&requirements,false){self.issue_original_sector_order(team,&order,&sectors);}
    let runtime=self.original_group_runtime(team,&groups);
    let support=task::support_hardware_orders(&mut groups,&sectors,&runtime,&objects,&state.scoring_flags,||self.native_random());
    self.apply_original_strategy_changes(team,task::Changes{sectors:Vec::new(),direct:Vec::new(),sequence:support.into_iter().map(task::Movement::Direct).collect()},&sectors);
    let runtime=self.original_group_runtime(team,&groups);let mut frontier=self.original_task_counters(team,&groups,18);
    let orders=task::assign_idle_frontiers(&mut groups,&sectors,&runtime,&mut frontier,||self.native_random());self.store_original_task_counters(team,&groups,18,&frontier);
    self.apply_original_strategy_changes(team,task::Changes{sectors:Vec::new(),direct:Vec::new(),sequence:orders.into_iter().map(task::Movement::Direct).collect()},&sectors);
   }
   self.native_group_rosters.store_task_groups(team,&groups,&sectors);
   state.groups.insert(team,groups);
  }
  self.native_strategy=Some(state);
 }
 fn original_task_counters(&self,team:i32,groups:&[task::Group],offset:usize)->Vec<i8>{groups.iter().map(|g|self.native_group_rosters.group_slot(team,g.id).map(|slot|self.native_group_rosters.pool[slot].strategy[offset]as i8).unwrap_or(0)).collect()}
 fn original_query_unit(&self,slot:usize)->Option<&Unit>{self.units.iter().rev().find(|u|u.native_slot==Some(slot)).or_else(||self.native_cached_unit_id(slot).and_then(|id|self.units.iter().find(|u|u.id==id)))}
 ///56640 uses the pre-UNIT-loop cache but live raw positions/classes. A body
 ///retired after456a90 still resolves through the cached native-slot identity.
 fn original_rectangle_units(&self,team:i32,rect:[i16;4],state:&State)->Option<Vec<task::RuntimeUnit>>{
  let buckets=self.native_cached_unit_buckets()?;let slots=task::rectangle_units(buckets,rect,|slot|self.original_query_unit(slot).map(|u|([(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],u.native_class7c)));
  Some(slots.into_iter().filter_map(|slot|{let u=self.original_query_unit(slot)?;let member=self.native_group_rosters.members.get(slot).and_then(Option::as_ref);let flags=member.map_or(u.team as u8&3,|m|m.flags31);Some(task::RuntimeUnit{native_slot:slot,position:[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],flags31:(flags&!3)|if u.team==team{1}else{0},flags8a:member.map_or(0,|m|m.flags8a),sector:state.sector_at(u.x,u.y)})}).collect())
 }
 fn store_original_task_counters(&mut self,team:i32,groups:&[task::Group],offset:usize,counters:&[i8]){for(g,&value)in groups.iter().zip(counters){if let Some(slot)=self.native_group_rosters.group_slot(team,g.id){self.native_group_rosters.pool[slot].strategy[offset]=value as u8;}}}
 fn issue_original_hardware_capture(&mut self,team:i32,index:usize,groups:&mut Vec<task::Group>,sectors:&[task::Sector],buildings:&[task::BuildingRecord],objects:&[task::WorldObject],fort:[i16;2]){
  let runtime=self.original_group_runtime(team,groups);let captures=self.original_capture_runtime(team,groups,fort);
  let Some(request)=task::hardware_capture_for_group(groups,index,sectors,&runtime,&captures,buildings,objects)else{return;};
  let target=self.native_object_unit(request.object).or_else(||self.native_world.is_none().then_some(request.object));let Some(target)=target else{return;};
  let Some((mut selected,slot))=self.split_original_hardware_member(team,&request,groups,sectors)else{return;};
  let Some(unit)=self.units.iter().find(|u|u.native_slot==Some(slot)&&u.team==team).cloned()else{return;};
  self.command(&serde_json::json!({"team":team,"kind":"enter","ids":[unit.id],"target":target}));
  let current=sectors.iter().find(|s|{let(x,y)=((unit.x/32.)as i16,(unit.y/32.)as i16);s.valid&&[(0,0),(1,1)].into_iter().any(|(dx,dy)|x+dx>=s.rect[0]&&x+dx<s.rect[2]&&y+dy>=s.rect[1]&&y+dy<s.rect[3])}).map(|s|s.id);
  let role=self.native_group_rosters.role(slot).unwrap_or(0);let flags=(self.native_group_rosters.members[slot].as_ref().map(|m|m.flags31).unwrap_or(5)&!3)|1;
  task::commit_hardware_capture(&mut selected,current,role,flags);
  if let Some(old)=groups.iter_mut().find(|g|g.id==selected.id){*old=selected;}else{groups.push(selected);groups.sort_by_key(|g|self.native_group_rosters.group_slot(team,g.id).unwrap_or(usize::MAX));}
 }
 fn original_cpu_fort(&self,team:i32)->[i16;2]{if let Some(bases)=self.native_fort_bases(){return bases[usize::from(team==1)];}self.buildings.iter().find(|b|b.kind=="fort"&&b.team>=0&&b.team!=team).map(|b|{let p=b.raw_base.unwrap_or((b.x/2.,b.y/2.));[p.0 as i16,p.1 as i16]}).unwrap_or([0,0])}
 fn original_strategy_ids(&self,team:i32,group:usize)->Vec<usize>{self.units.iter().filter(|u|u.team==team&&u.squad==group&&original_combat::alive(true,u.hp)&&!super::workshops::hidden(u)).map(|u|u.id).collect()}
 fn original_ai_flags9c(&self,u:&Unit)->u8{u.native_flags9c}
 fn original_ai_accepts_orders(&self,u:&Unit)->bool{
  self.original_ai_flags9c(u)&0x14==0&&u.native_protectionaa==0
 }
 ///54070||540b0 consumes global632194, including every occupied roster slot.
 ///Missing global context is an explicit unavailable gate, never the queried
 ///squad substituted because it happens to contain the candidate leader.
 pub(crate) fn original_global_transport_uniform(&self)->bool{
  let Some(group)=self.native_group_rosters.current_roster.and_then(|s|self.native_group_rosters.pool.get(s))else{return false;};let mut first=None;
  for role in 0..21{if let Some(slot)=group.member(role){let Some(member)=self.native_group_rosters.members[usize::from(slot)].as_ref()else{return false;};let bit=member.flags8a&32!=0;if first.is_some_and(|first|first!=bit){return false;}first=Some(bit);}}true
 }
 fn original_group_runtime(&self,team:i32,groups:&[task::Group])->Vec<task::GroupRuntime>{groups.iter().filter_map(|g|{let l=g.leader.as_ref()?;self.units.iter().find(|u|u.team==team&&u.native_slot==Some(l.native_slot))?;let slot=self.native_group_rosters.group_slot(team,g.id)?;let row=&self.native_group_rosters.pool[slot].roster;let member=self.native_group_rosters.members[l.native_slot].as_ref()?;
  //1a0d0 scans the native20-byte command-record pool, not workshops or a
  //personal flag marker. That pool is not represented by synchronous Web
  //orders yet; its actual pending state is an explicit adapter boundary.
  Some(task::GroupRuntime{group:g.id,member_count:row[18]as i8,combat_ttl:row[16],native_order:row[58]as i8,pending:false,leader_role:member.role,leader_flags31:member.flags31})}).collect()}
 fn original_assault_controls(&self,team:i32,groups:&[task::Group])->Vec<task::AssaultControl>{groups.iter().map(|g|{
  let uniform=self.original_global_transport_uniform();
  let clear=g.leader.as_ref().and_then(|l|self.units.iter().find(|u|u.team==team&&u.native_slot==Some(l.native_slot))).is_some_and(|u|self.original_ai_flags9c(u)&4==0);
  task::AssaultControl{group:g.id,can_capture:uniform&&clear,can_mobilize:uniform&&clear}
 }).collect()}
 fn original_capture_runtime(&self,team:i32,groups:&[task::Group],fort:[i16;2])->Vec<task::CaptureRuntime>{
  let uniform=self.original_global_transport_uniform();
  groups.iter().map(|g|{
   let mut members=self.units.iter().filter(|u|u.team==team&&u.squad==g.id&&original_combat::alive(true,u.hp)&&!driver_duplicate(u,&self.units)).collect::<Vec<_>>();members.sort_by_key(|u|u.native_slot.unwrap_or(u.id));
   let leader=g.leader.as_ref().and_then(|l|members.iter().find(|u|u.native_slot.unwrap_or(u.id)==l.native_slot)).copied();
   let coord=|p:(f64,f64)|[((p.0/2.).floor()as i16).wrapping_add(8),((p.1/2.).floor()as i16).wrapping_add(8)];
   let row=self.native_group_rosters.group_slot(team,g.id).map(|slot|&self.native_group_rosters.pool[slot]);
   let entering=task::entering_cpu_fort(row.map_or(0,|r|r.roster[58]as i8),row.map(|r|[r.word(44),r.word(46)]),leader.and_then(|u|u.path.front().copied()).map(coord),fort);
   let slots=self.native_group_rosters.roster(team,g.id);
   let roster=slots.into_iter().map(|slot|slot.and_then(|slot|members.iter().find(|u|u.native_slot==Some(slot)).and_then(|u|self.native_group_rosters.members[slot].as_ref().map(|m|task::CaptureMember{native_slot:slot,role:m.role,personal_capture:m.flags8a&4!=0,special_path:m.flags31&64!=0,can_start_path:uniform,direct_blocked:self.original_ai_flags9c(u)&4!=0})))).collect();
   task::CaptureRuntime{group:g.id,roster,leader_overlay:leader.and_then(|u|u.native_slot).and_then(|slot|self.native_group_rosters.members[slot].as_ref()).is_some_and(|m|m.flags31&32!=0),entering_cpu_fort:entering}
  }).collect()
 }
 fn apply_original_strategy_changes(&mut self,team:i32,changes:task::Changes,sectors:&[task::Sector]){
  for movement in changes.sequence{match movement{
   task::Movement::Sector(order)=>self.issue_original_sector_order(team,&order,sectors),
   task::Movement::Direct(order)=>{
    if order.dispatch==task::DirectDispatch::Order{self.issue_original_move_event(team,order.group,order.position,order.metadata);}
    else if let Some(slot)=self.native_group_rosters.leader_slot(team,order.group){if let Some(index)=self.units.iter().position(|u|u.team==team&&u.native_slot==Some(slot)){self.replace_original_ai_path(index,order.position,order.append);}}
   },
  }}
 }
 ///45ef00 actor dispatch. The Web command adapter supplies selection and
 ///formation routing; native45f070 writes remain attached to the real leader.
 fn issue_original_move_event(&mut self,team:i32,group:usize,waypoint:[i16;2],metadata:i16){
  let Some(slot)=self.native_group_rosters.leader_slot(team,group)else{return;};let Some(index)=self.units.iter().position(|u|u.team==team&&u.native_slot==Some(slot))else{return;};
  let ids=self.original_strategy_ids(team,group);let goal=(f64::from(waypoint[0].wrapping_sub(8))*2.,f64::from(waypoint[1].wrapping_sub(8))*2.);
  self.command(&serde_json::json!({"team":team,"kind":"move","ids":ids,"x":goal.0,"y":goal.1,"nativeEvent":false}));self.record_native_order_event(index,super::original_orders_game::Event::classified(1,waypoint,metadata));
 }
 ///59c60→52810 for14bd0/14c60/13740. This never creates a45ef00 event.
 ///The linked primary/local paths currently share the Web path vector; that
 ///representation cannot reproduce exhausted native path-pool allocations.
 fn replace_original_ai_path(&mut self,index:usize,waypoint:[i16;2],append:bool){
  let Some(slot)=self.units[index].native_slot else{return;};self.native_group_rosters.record_physical_contact(slot,false);self.native_group_rosters.record_secondary_path(slot,false);self.units[index].native_secondary_route.set(false);
  let goal=(f64::from(waypoint[0].wrapping_sub(8))*2.,f64::from(waypoint[1].wrapping_sub(8))*2.);let mut start=self.units[index].clone();if append{if let Some(p)=start.path.back(){start.x=p.0;start.y=p.1;}}
  let path=self.route_unit(&start,goal.0,goal.1);if append{self.units[index].path.extend(path);}else{self.units[index].path=path;}
  let secondary=self.units[index].native_secondary_route.get();self.native_group_rosters.record_secondary_path(slot,secondary);
  let head=self.units[index].path.front().map(|p|[((p.0/2.).floor()as i16).wrapping_add(8),((p.1/2.).floor()as i16).wrapping_add(8)]);
  let Some(member)=self.native_group_rosters.members[slot].as_mut()else{return;};member.primary_path=!self.units[index].path.is_empty()&&!secondary;
  let u=&mut self.units[index];let mut contact8c=u.motion_bonus as u8;let started=super::original_orders_game::start_path(member.flags31,u.native_flags9c,&mut u.native_flags64,&mut u.native_initiative_scan.flags71,&mut contact8c,&mut member.promotion.motion,if secondary{None}else{head},if secondary{head}else{None});u.motion_bonus=contact8c as i8;
  if u.motion_bonus==0&&member.flags31&8==0&&u.native_flags9c&16==0{member.contact_order=0;}
  let Some(anchor)=started else{return;};let nonzero=anchor!=[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16];u.original_motion=None;u.order="move".into();u.target=None;u.dest=Some(goal);u.hold_position=false;u.explicit_stop=false;
  if nonzero{u.native_fire=None;u.native_idle=None;u.native_cover=None;self.prepare_native_walk(index);}
 }
 fn issue_original_personal_capture(&mut self,team:i32,request:&task::PersonalCapture,sector:usize){
  let Some(index)=self.units.iter().position(|u|u.team==team&&u.native_slot.unwrap_or(u.id)==request.member&&original_combat::alive(true,u.hp)&&!super::workshops::hidden(u)&&u.transport_carrier.is_none())else{return;};
  let previous=self.units[index].dest;self.replace_original_ai_path(index,request.position,request.preserve_path);let p=&mut self.units[index];
  if request.personal&&!request.preserve_path{p.march_resume=previous;p.initiative=Some(super::original_initiative::Initiative::Flag(sector));p.motion_bonus=3;}
  if request.personal&&!request.preserve_path{if let Some(slot)=self.units[index].native_slot{self.native_group_rosters.record_personal_capture(slot,true);}}
 }
 pub(crate) fn update_original_tactics(&mut self){
  if !self.map.original||self.motion_tick&1!=0{return;}
  let Some(mut state)=self.native_strategy.take()else{return;};
  for team in self.ai_teams.clone(){
   let mut groups=state.groups.remove(&team).unwrap_or_default();state.refresh_groups(self,team,&mut groups);
   //The even-callback pass reads cached priorities; it does not consume the
   //strategic score-noise RNG or recompute the allocation budget.
   let mut sectors=state.sectors.clone();for s in &mut sectors{let owner=self.sectors.iter().find(|p|p.id==s.id).map(|p|p.owner).unwrap_or(-1);s.owner=if owner==team{1}else if owner>=0{0}else{4};s.flags&=!3;if let Some(bases)=self.native_fort_bases(){for(i,base)in bases.iter().enumerate(){if state.sector_at(f64::from(base[0])*2.,f64::from(base[1])*2.)==Some(s.id){s.flags|=1<<i;}}}else{for b in self.buildings.iter().filter(|b|b.kind=="fort"&&b.sector==s.id){s.flags|=if b.team==team{1}else{2};}}}
   let fort=self.original_cpu_fort(team);let runtime=self.original_capture_runtime(team,&groups,fort);
   for request in task::tactical_fort_captures(&mut groups,&sectors,&runtime,fort){let sid=groups.iter().find(|g|g.id==request.group).and_then(|g|g.current_sector).unwrap_or(0);self.issue_original_personal_capture(team,&request,sid);}
   let mut counters=self.original_task_counters(team,&groups,19);
   for goal in task::tactical_vehicle_goals(&groups,&sectors,&mut counters){if let Some(index)=groups.iter().find(|g|g.id==goal.group).and_then(|g|g.leader.as_ref()).and_then(|l|self.units.iter().position(|u|u.team==team&&u.native_slot.unwrap_or(u.id)==l.native_slot&&!super::workshops::hidden(u)&&u.transport_carrier.is_none()&&u.workshop.is_none())){let path=self.route_unit(&self.units[index],f64::from(goal.position[0])*2.,f64::from(goal.position[1])*2.);self.units[index].path=path;self.units[index].original_motion=None;}}
   self.store_original_task_counters(team,&groups,19,&counters);self.native_group_rosters.store_task_groups(team,&groups,&sectors);
   state.groups.insert(team,groups);
  }
  self.native_strategy=Some(state);
 }
 fn issue_original_sector_order(&mut self,team:i32,order:&task::Order,sectors:&[task::Sector]){
  let Some(s)=sectors.iter().find(|s|s.id==order.sector)else{return;};
  let Some(native_leader)=self.native_group_rosters.leader_slot(team,order.group)else{return;};let Some(lead)=self.units.iter().find(|u|u.team==team&&u.native_slot==Some(native_leader)&&original_combat::alive(true,u.hp)).cloned()else{return;};
  if lead.native_class7c==2{return;}
  if s.flags&32!=0&&lead.native_class7c==1||s.flags&64!=0&&matches!(lead.native_class7c,0|15|16)&&!score::group_can_demolish(&self.units.iter().filter(|u|u.team==team&&u.squad==order.group&&original_combat::alive(true,u.hp)).map(|u|u.grenades as i8).collect::<Vec<_>>()){
   self.issue_original_move_event(team,order.group,[(lead.x/2.)as i16,(lead.y/2.)as i16],0);return;
  }
  let mut origin=[i16::from(s.flag[0]as i8)<<4,i16::from(s.flag[1]as i8)<<4];
  if let Some(bases)=self.native_fort_bases(){if s.flags&1!=0{origin=[bases[0][0].wrapping_add(80),bases[0][1].wrapping_add(220)];}if s.flags&2!=0{origin=[bases[1][0].wrapping_add(80),bases[1][1].wrapping_sub(80)];}}
  else if s.flags&3!=0{if let Some(b)=self.buildings.iter().find(|b|b.kind=="fort"&&b.sector==s.id){let base=b.raw_base.unwrap_or((b.x/2.,b.y/2.));origin=[(base.0 as i16).wrapping_add(80),(base.1 as i16).wrapping_add(if s.flags&2!=0{-80}else{220})];}}
  let random=self.native_random();let waypoint=super::original_orders_game::approach_point(origin,original_combat::animation_family(&lead.kind),random,|cell|self.original_sector_path_allowed(&lead,cell),|point|self.original_sector_goal_reserved(point,sectors));
  if let Some(waypoint)=waypoint{self.issue_original_move_event(team,order.group,waypoint,order.metadata);}
 }
 ///437790/437830 inspect the four footprint corners with unit+18's path
 ///permission table. This is separate from physical collision or BUILD boxes.
 fn original_sector_approach_clear(&self,unit:&Unit,point:[i16;2])->bool{
  super::original_orders_game::footprint_clear(point,matches!(original_combat::animation_family(&unit.kind),6|25|30|24|32|34|36|38),|cell|self.original_sector_path_allowed(unit,cell))
 }
 pub(crate) fn original_sector_path_allowed(&self,unit:&Unit,cell:i16)->bool{
  if !(0..16384).contains(&i32::from(cell)){return false;}let(x,y)=(cell as usize&127,cell as usize>>7);let code=if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x)}else{self.map.native_terrain_plane.get(y*128+x)}.copied().unwrap_or(127);let tough=unit.kind=="tough"&&unit.native_slot.and_then(|s|self.native_group_rosters.role(s)).is_some_and(|role|role<1);if tough{super::original_route::tough_leader_passable(code)}else{super::original_terrain::passable(code,self.map.world,super::original_terrain::profile(&unit.kind))}
 }
 ///19240 rejects the aligned goal of an active moving leader in a real sector.
 fn original_sector_goal_reserved(&self,point:[i16;2],sectors:&[task::Sector])->bool{
  self.native_group_rosters.pool.iter().any(|row|{if row.strategy[21]==0{return false;}let Some(slot)=row.member(0)else{return false;};let Some(member)=self.native_group_rosters.members[usize::from(slot)].as_ref()else{return false;};let Some(unit)=self.units.iter().find(|u|u.native_slot==Some(usize::from(slot)))else{return false;};let goal=[row.word(44),row.word(46)];let actual_sector=[(0,0),(1,1)].into_iter().any(|(dx,dy)|sectors.iter().any(|s|s.valid&&(goal[0]>>4)+dx>=s.rect[0]&&(goal[0]>>4)+dx<s.rect[2]&&(goal[1]>>4)+dy>=s.rect[1]&&(goal[1]>>4)+dy<s.rect[3]));super::original_orders_game::reserved_goal(point,super::original_orders_game::GoalReservation{active:true,leader_exists:true,role:member.role,class:unit.native_class7c,flags31:member.flags31,phase:unit.native_phase35,command:row.roster[58]as i8,goal,valid_sector:actual_sector})})
 }
}

#[cfg(test)]mod tests{
 use super::*;use super::super::json;
 fn controls()->Value{json!({"nativeAi":{"decisionControl":0,"reservePercent":25,"populationWeight":1,"initialSectorTasks":[{"sector":1}]},"sectors":[
  {"id":0,"rect":[0,0,10,20],"nativeSlot":0,"rawFlagTile":[4,8],"nativeFlags":0,"nativePopulation":0,"owner":1},
  {"id":1,"rect":[10,0,30,20],"nativeSlot":1,"rawFlagTile":[20,8],"nativeFlags":0,"nativePopulation":0,"owner":0}],"buildings":[]})}
 fn field()->Game{let mut g=Game::new(0,1,915);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.buildings.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.ai_teams=vec![1];g.spawn("grunt",1,160.,256.,101,None);g.native_strategy=State::from_mission(&controls());g}
 #[test]fn scripted_initial_cpu_task_uses_its_own_sector_and_is_not_nearest_flag(){
  let mut g=field();g.update_original_strategy(true);let state=g.native_strategy.as_ref().unwrap();let group=&state.groups[&1][0];
  assert_eq!(group.assigned_sector,Some(1));assert_eq!(group.task,0);assert_eq!(group.flags&8,8);assert_eq!(group.flags&16,0,"17b30 runs16e10 after scripted tasks: one oversized group does not fit the25% reserve budget");
  let dest=g.units[0].dest.unwrap();assert!(dest.0>=528.&&dest.0<=720.,"Native48px sector approach offsets surround CPUPLR flag");assert!(!g.units[0].path.is_empty());
 }
 #[test]fn distance_bias_is_sticky_after_an_opponent_capture_and_resets_with_mission(){
  let mut s=State::from_mission(&controls()).unwrap();assert!(s.distance_bias);s.captured(0,0);assert!(s.distance_bias);s.captured(1,0);assert!(!s.distance_bias);s.captured(0,0);assert!(!s.distance_bias);assert!(State::from_mission(&controls()).unwrap().distance_bias);
 }
 #[test]fn cp_decision_control_is_sign_extended_from_the_source_byte(){let mut v=controls();v["nativeAi"]["decisionControl"]=json!(200);assert_eq!(State::from_mission(&v).unwrap().control,-56);}
 fn current_groups(g:&mut Game)->Vec<task::Group>{g.maintain_native_group_rosters();let state=g.native_strategy.take().unwrap();let mut groups=vec![];state.refresh_groups(g,1,&mut groups);g.native_strategy=Some(state);groups}
 #[test]fn ai_order_acceptance_reads_native_flags_and_protection_without_browser_stop_or_initiative_guesses(){
  let mut g=field();g.units[0].explicit_stop=true;g.units[0].initiative=Some(super::super::original_initiative::Initiative::Flag(0));let groups=current_groups(&mut g);assert!(groups[0].leader.as_ref().unwrap().accepts_orders);
  g.units[0].native_flags9c=4;let groups=current_groups(&mut g);assert!(!groups[0].leader.as_ref().unwrap().accepts_orders);g.units[0].native_flags9c=0;g.units[0].native_protectionaa=1;let groups=current_groups(&mut g);assert!(!groups[0].leader.as_ref().unwrap().accepts_orders);g.advance_native_order_preamble(0);let groups=current_groups(&mut g);assert!(groups[0].leader.as_ref().unwrap().accepts_orders);
 }
 #[test]fn capture_transport_uniformity_reads_the_global_roster_and_each_selected_members_raw_flags(){
  let mut g=field();g.spawn("grunt",1,200.,256.,102,None);g.spawn("grunt",1,232.,256.,102,None);let groups=current_groups(&mut g);g.native_group_rosters.set_current_roster(1,102);let last=g.units[2].native_slot.unwrap();g.native_group_rosters.record_transport_inside(last,true);
  let controls=g.original_assault_controls(1,&groups);assert!(!controls.iter().find(|c|c.group==101).unwrap().can_capture,"global632194 differs from the requested squad");
  g.native_group_rosters.record_transport_inside(last,false);let actor=g.units[0].native_slot.unwrap();g.native_group_rosters.record_physical_contact(actor,true);g.units[0].native_flags9c=4;let runtime=g.original_capture_runtime(1,&groups,[0,0]);let member=runtime.iter().find(|r|r.group==101).unwrap().roster[0].unwrap();assert!(member.can_start_path);assert!(member.special_path);assert!(member.direct_blocked);
 }
 #[test]fn strategic_order_runtime_uses_roster_command_count_and_sparse_member_role(){
  let mut g=field();let groups=current_groups(&mut g);let slot=g.units[0].native_slot.unwrap();let roster=g.native_group_rosters.group_slot(1,101).unwrap();g.native_group_rosters.pool[roster].roster[18]=7;g.native_group_rosters.pool[roster].roster[58]=3;g.units[0].native_order5c=0;g.units[0].order="guard".into();g.native_group_rosters.members[slot].as_mut().unwrap().flags31|=64;
  let runtime=g.original_group_runtime(1,&groups);assert_eq!(runtime[0].member_count,7);assert_eq!(runtime[0].native_order,3);assert_eq!(runtime[0].leader_role,-1);assert_ne!(runtime[0].leader_flags31&64,0);
 }
 #[test]fn hidden_apc_roster_is_scored_once_with_its_actual_damaged_passengers(){
  let mut g=field();g.spawn("apc",1,160.,256.,101,Some(super::super::Crew{kind:"grunt".into(),hp:20.}));let ci=g.units.len()-1;let actor=g.units[0].id;let carrier=g.units[ci].id;
  g.units[ci].transport_actor=Some(actor);g.units[ci].native_slot=g.units[0].native_slot;g.units[0].native_slot=None;g.units[0].transport_carrier=Some(carrier);g.units[0].native_transport=Some(super::super::original_transport::Animation::exiting(0,0,false));
  g.spawn("tough",1,160.,256.,101,None);let pi=g.units.len()-1;g.units[pi].transport_carrier=Some(carrier);g.units[pi].native_transport=Some(super::super::original_transport::Animation::exiting(0,0,false));g.units[ci].cargo=vec![super::super::Crew{kind:"tough".into(),hp:100.}];
  let state=g.native_strategy.take().unwrap();let mut groups=vec![];state.refresh_groups(&g,1,&mut groups);let first=groups[0].value;assert_eq!(groups[0].leader.as_ref().unwrap().native_slot,g.units[ci].native_slot.unwrap());
  g.units[ci].cargo[0].hp=0.;state.refresh_groups(&g,1,&mut groups);assert_eq!(groups[0].value,first,"Crew summary must not duplicate the retained passenger");
  g.units[pi].hp=0.;state.refresh_groups(&g,1,&mut groups);assert!(groups[0].value<first,"Native AI must observe the real passenger damage");
 }
 #[test]fn local_hardware_nudge_runs_only_on_even_native_callbacks(){
  let mut g=field();for s in &mut g.sectors{s.owner=0;}g.units.clear();g.spawn("light",1,700.,256.,101,None);g.motion_tick=1;g.update_original_tactics();assert!(g.units[0].path.is_empty());g.motion_tick=2;g.update_original_tactics();assert!(!g.units[0].path.is_empty());assert_eq!(g.units[0].order,"guard");
 }
 #[test]fn accepted_sector_move_and_direct_assault_have_distinct_native_writes(){
  let mut g=field();g.spawn("grunt",1,192.,256.,101,None);g.maintain_native_group_rosters();let slot=g.units[0].native_slot.unwrap();let group=g.native_group_rosters.group_slot(1,101).unwrap();
  g.issue_original_move_event(1,101,[200,300],1);assert_eq!(g.native_order_arguments,[200,300,1]);assert_eq!(g.units[0].native_order5c,1);assert_eq!(g.native_group_rosters.pool[group].word(48),1);
  let follower=g.units[1].path.clone();g.units[0].native_order5c=-5;g.units[0].native_flags9c=0x20;g.units[0].native_phase35=5;g.units[0].native_initiative_scan.flags71=64;g.units[0].native_flags64|=128;
  g.replace_original_ai_path(0,[240,340],false);assert_eq!(g.units[0].dest,Some((464.,664.)));assert_eq!(g.units[0].native_order5c,-5);assert_eq!(g.units[0].native_flags9c,32);assert_eq!(g.native_order_arguments,[200,300,1]);assert_eq!(g.native_group_rosters.pool[group].word(44),200);assert_eq!(g.units[0].native_initiative_scan.flags71&64,0);assert_eq!(g.units[0].native_phase35,2);assert_eq!(g.units[1].path,follower,"14bd0/14c60 only replace the native leader path");assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.motion,0);
 }
 #[test]fn source_sector_approach_checks_footprint_and_reserves_active_leader_goals(){let mut g=field();g.maintain_native_group_rosters();let leader=g.units[0].clone();assert!(g.original_sector_approach_clear(&leader,[160,128]));g.map.terrain_codes[8*g.map.width+10]=40;assert!(!g.original_sector_approach_clear(&leader,[160,128]));g.map.terrain_codes.fill(0);let row=g.native_group_rosters.group_slot(1,101).unwrap();g.native_group_rosters.pool[row].strategy[21]=1;g.native_group_rosters.record_group_order(1,101,1,[160,128],0);g.units[0].native_phase35=2;let sectors=g.native_strategy.as_ref().unwrap().sectors.clone();assert!(g.original_sector_goal_reserved([167,135],&sectors));g.units[0].native_phase35=1;assert!(!g.original_sector_goal_reserved([167,135],&sectors));}
 #[test]fn ai_rectangle_uses_cached_slots_with_live_positions_and_preserves_retired_identity(){let mut g=field();g.maintain_native_group_rosters();g.native_ambient=Some(super::super::original_ambient_game::State::default());g.prepare_native_ambient_queries();let slot=g.units[0].native_slot.unwrap();let state=g.native_strategy.take().unwrap();assert_eq!(g.original_rectangle_units(1,[64,120,128,160],&state).unwrap().len(),1);g.units[0].x=900.;g.units[0].y=900.;assert!(g.original_rectangle_units(1,[440,440,480,480],&state).unwrap().is_empty(),"565a0 does not rebuild bucket membership from current positions");g.units[0].x=160.;g.units[0].y=256.;g.units[0].hp=-1.;g.remove_original_group_member(0);g.units[0].native_slot=None;assert_eq!(g.original_rectangle_units(1,[64,120,128,160],&state).unwrap()[0].native_slot,slot,"cached pointer still addresses inactive rawUNIT fields");g.spawn("grunt",0,160.,256.,104,None);g.maintain_native_group_rosters();assert_eq!(g.units[1].native_slot,Some(slot));assert_eq!(g.original_rectangle_units(1,[64,120,128,160],&state).unwrap()[0].flags31&3,0,"a reused native slot resolves the new occupant before old cached Web identity");}
 #[test]fn building_target_only_fallback_requires_and_uses_actual_descriptor_offsets(){let mut g=field();let mut b=Game::new(0,1,1).buildings[0].clone();b.id=44;b.raw_base=Some((300.,400.));g.buildings.push(b);g.native_strategy.as_mut().unwrap().building_meta.insert(44,(0,[-7,23]));assert_eq!(g.native_building_target_event(44),Some(super::super::original_orders_game::Event::classified(5,[3456,293],423)));g.native_strategy.as_mut().unwrap().building_meta.clear();assert_eq!(g.native_building_target_event(44),None);}
}
