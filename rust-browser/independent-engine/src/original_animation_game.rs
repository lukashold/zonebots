//! World adapters for verified native idle, movement frames and cover poses.
//! The phase0/1 infantry preamble consumes persistent UNIT/group fields before
//! animation and scans. Unsupported command types keep their existing handler.
use super::*;
use super::original_idle::{State as Idle,Action};
use super::original_reactions as reaction;

///UNIT64:40 and the replacement525a0 line. The primary route and player
///command stay on Unit;8c=2 is a native acquisition counter, not a lifetime.
#[derive(Clone)]
pub struct Dodge{segment:original_motion::Segment,pub counter8c:u8}

///41c090 and the hardware acquisition initializers41ba80..41bb70,
///457ff0/458050/458090/4582c0. Infantry first run phase0's44dd00.
pub(crate) fn initial_phase(kind:&str)->u8{match kind{
 "cannon"=>4,"missile"=>50,"gatling"=>51,"howitzer"=>52,
 "medium"=>9,"launcher"=>53,"heavy"=>54,"light"=>55,"apc"=>56,"jeep"=>58,"crane"=>59,"truck"|"firetruck"=>60,_=>0,
}}
pub(crate) fn fire_phase(mode:original_fire::Mode,current:u8)->u8{match mode{
 original_fire::Mode::Rifle=>5,original_fire::Mode::Rapid=>57,original_fire::Mode::Laser=>71,original_fire::Mode::Flame=>65,original_fire::Mode::Tough=>19,original_fire::Mode::Grenade=>49,original_fire::Mode::Hardware=>current,
}}

fn dodge_bucket(point:[i16;2])->i16{(point[1]>>3&!15).wrapping_add(point[0]>>7)}

impl Game{
 ///44cac0 reset and negative-order continuation. This source event has no
 ///browser idle eligibility check; acquisition does not advance a fire frame.
 pub(crate) fn reset_native_idle(&mut self,index:usize)->bool{
  let Some(u)=self.units.get(index).cloned()else{return false;};let member=u.native_slot.and_then(|slot|self.native_group_rosters.members.get_mut(slot)).and_then(Option::as_mut);let leader=member.as_ref().is_some_and(|m|m.role==-1);if let Some(m)=member{m.flags8a&=!8;}if leader{if let Some(context)=self.native_group_rosters.current_roster.filter(|&s|s<240){self.native_group_rosters.pool[context].roster[59]=255;}}
  if u.native_class7c==2{return true;}if u.native_class7c!=1{let chance=original_combat::CombatState::new(&u.kind).aim_skill as i8;let idle=Idle::enter(original_combat::facing(u.angle),chance,||self.native_random());let v=&mut self.units[index];v.native_frame32=idle.frame as i16;v.native_timer3c=idle.timer;v.native_idle=Some(idle);v.native_phase35=1;v.native_walk=None;v.native_fire=None;}
  if self.units[index].native_order5c<0{self.continue_native_idle_reset(index);}true
 }
 ///Known phase0/1 infantry dispatch. Its preamble precedes the idle counters
 ///and directional scans; unrelated manual attack orders stay with their
 ///existing handlers until their native command fields have been supplied.
 pub(crate) fn update_native_idle_handler(&mut self,index:usize)->bool{
  let u=self.units[index].clone();
  if !self.map.original||stats(&u.kind).cls!=0||!matches!(u.native_phase35,0|1)||!matches!(u.order.as_str(),"guard"|"move")||!u.path.is_empty()||u.native_route_pending.get().is_some()||u.native_fire.is_some()||u.initiative.is_some()||u.workshop.is_some()||u.native_contact.is_some()||u.native_dodge.is_some()||u.native_cover.is_some()||u.native_transport.is_some()||u.transport_carrier.is_some(){return false;}
  if u.native_phase35==0||u.native_idle.is_none(){self.units[index].native_idle=None;self.update_native_idle(index,false);return true;}
  let member=u.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).cloned();
  let context=self.native_group_rosters.current_roster.filter(|&s|s<240);let group=context.map(|s|self.native_group_rosters.pool[s].roster);
  let p=original_idle::Preamble{role:member.as_ref().map_or(0,|m|m.role),flags31:member.as_ref().map_or((u.team as u8&3)|4,|m|m.flags31),flags64:u.native_flags64,aim_skill:u.original_combat.as_ref().map_or(60,|s|s.aim_skill)as i8,valid_group:group.is_some(),group_target:group.and_then(|g|(g[59]!=255).then_some(g[59])),group_3d:group.map_or(0,|g|g[61]as i8),group_3e:group.map_or(0,|g|g[62]as i8),group_3f:group.map_or(0,|g|g[63]as i8),marked_unit:member.as_ref().map_or(-1,|m|m.promotion.target_x as i16)};
  if original_idle::requires_order_sync(&p){if let(Some(group),Some(member))=(group,member.as_ref()){
   let fields=[i16::from_le_bytes([group[44],group[45]]),i16::from_le_bytes([group[46],group[47]]),i16::from_le_bytes([group[48],group[49]])];
   if let Some(order)=original_idle::sync_group_order(p.role,member.promotion.order as i8,member.flags69&128!=0,group[58]as i8,fields){if let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members.get_mut(s)).and_then(Option::as_mut){m.promotion.order=order.command as u8;m.promotion.target_x=order.field50;m.promotion.target_y=order.field54;m.promotion.target_aux=order.field56;}}
  }}
  match original_idle::preamble(&p,||self.native_random()){
   original_idle::Gate::Animate=>self.update_native_idle(index,false),
   original_idle::Gate::Wait=>{},
   original_idle::Gate::ReactGroup=>self.react_native_idle_group(index,p.group_target,context),
   original_idle::Gate::WithdrawFromThreat{target}=>{
    if let Some(threat)=self.units.iter().rev().find(|v|v.native_slot==Some(usize::from(target))).cloned(){if let Some(goal)=original_idle::withdraw_goal(p.role,((u.x/2.).floor()as i16,(u.y/2.).floor()as i16),((threat.x/2.).floor()as i16,(threat.y/2.).floor()as i16)){self.start_native_idle_path(index,[goal.0,goal.1]);if let Some(context)=context{self.native_group_rosters.pool[context].roster[59]=255;}}}
   },
   original_idle::Gate::FollowMarkedUnit{target}=>{if let Some(target)=usize::try_from(target).ok().and_then(|slot|self.units.iter().rev().find(|v|v.native_slot==Some(slot))).cloned(){self.start_native_idle_path(index,[(target.x/2.).floor()as i16,(target.y/2.).floor()as i16]);}},
   original_idle::Gate::FollowGroup=>self.follow_native_idle_group(index),
  }
  true
 }
 fn start_native_idle_path(&mut self,index:usize,point:[i16;2]){
  let goal=(f64::from(point[0])*2.,f64::from(point[1])*2.);let path=self.route_unit(&self.units[index],goal.0,goal.1);if (self.units[index].x,self.units[index].y)!=goal{self.prepare_native_walk(index);}let u=&mut self.units[index];u.path=path;u.dest=Some(goal);u.order="move".into();u.original_motion=None;u.native_idle=None;u.native_fire=None;
 }
 fn react_native_idle_group(&mut self,index:usize,target:Option<u8>,context:Option<usize>){
  let Some(target)=target.and_then(|slot|self.units.iter().rev().find(|v|v.native_slot==Some(usize::from(slot)))).cloned()else{return;};
  let member=target.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref);
  let role=member.map_or(0,|m|m.role);let raw=member.filter(|m|usize::from(m.group)<240).map(|m|self.native_group_rosters.pool[usize::from(m.group)].roster);let roster=raw.map(|r|r[19..40].try_into().unwrap()).unwrap_or([255;21]);
  let selection=original_idle::group_reaction_target(target.native_slot.unwrap_or(255)as u8,original_combat::animation_family(&target.kind),role,original_combat::alive(true,target.hp),&roster,raw.map_or(0,|r|r[18]as i8),||self.native_random());
  let selected=match selection{original_idle::GroupReactionTarget::Target(slot)=>self.units.iter().rev().find(|v|v.native_slot==Some(usize::from(slot))).cloned(),original_idle::GroupReactionTarget::Clear=>{if let Some(context)=context{self.native_group_rosters.pool[context].roster[59]=255;}None},original_idle::GroupReactionTarget::None=>None};
  if let Some(target)=selected{self.react_native_idle_target(index,&target);}
 }
 fn react_native_idle_target(&mut self,index:usize,target:&Unit){
  if target.team==self.units[index].team||!original_combat::alive(true,target.hp){return;}
  if matches!(original_combat::runtime_class(&self.units[index].kind),0|3|15|16){self.advance_native_fire(index,target);if self.units[index].native_fire.is_some(){self.units[index].native_idle=None;self.units[index].native_walk=None;}}
  else{
   //574c0's other weapon classes approach the target through525a0 after
   //59e40. This is a local straight line, not a new global player route.
   let goal=(target.x,target.y);if (self.units[index].x,self.units[index].y)!=goal{self.prepare_native_walk(index);}let p=&mut self.units[index];p.path=VecDeque::from([goal]);p.dest=Some(goal);p.order="move".into();p.native_idle=None;p.original_motion=None;
  }
 }
 pub(crate) fn react_native_dodge(&mut self,shooter:Option<usize>,fallback_team:i32,current:[i16;2],target:[i16;2]){
  if !self.map.original{return;}
  //462fd0 reads the shooter's current slot, including a reused UNIT slot.
  let team=shooter.and_then(|slot|self.units.iter().rev().find(|u|u.native_slot==Some(slot))).map_or(fallback_team,|u|u.team)as u8;
  let origin=dodge_bucket(target);let mut candidates=Vec::new();
  for offset in reaction::PROJECTILE_BUCKET_OFFSETS{
   let bucket=origin.wrapping_add(offset)as u16;if bucket>=256{continue;}
   let mut local=self.units.iter().enumerate().filter(|(_,u)|u.team>=0&&original_combat::alive(true,u.hp)&&!workshops::hidden(u)&&dodge_bucket([(u.x/2.).floor()as i16,(u.y/2.).floor()as i16])as u16==bucket).map(|(i,u)|(u.native_slot.unwrap_or(u.id),i)).collect::<Vec<_>>();
   local.sort_unstable();candidates.extend(local.into_iter().take(32).map(|(_,i)|i));
  }
  for i in candidates{
   let u=self.units[i].clone();let member=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref);
   let input=reaction::DodgeUnit{position:[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],flags31:member.map_or((u.team as u8&3)|4,|m|m.flags31),flags64:u.native_flags64,flags69:member.map_or(0,|m|m.flags69),class7c:u.native_class7c,state35:u.native_phase35,aim02:u.original_combat.as_ref().map_or(60,|c|c.aim_skill)as i8};
   if !reaction::dodge_candidate(input,team,current,target){continue;}
   let destination=reaction::dodge_destination(input.position,target,[self.native_random(),self.native_random()]);
   let segment=original_motion::Segment::new(u.x,u.y,(f64::from(destination[0])*2.,f64::from(destination[1])*2.));
   if input.class7c!=1{self.prepare_native_walk(i);}
   let heading=segment.heading();let p=&mut self.units[i];
   p.native_remaining48=segment.remaining();p.native_dodge=Some(Dodge{segment,counter8c:2});p.native_flags64|=64;if input.class7c!=1{p.native_phase35=2;}p.original_motion=None;p.native_idle=None;p.native_cover=None;p.native_fire=None;p.native_contact=None;
   p.angle=heading;if input.class7c!=1{p.aim=heading;}
   if let Some(slot)=p.native_slot{if let Some(m)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut){if input.class7c!=1{m.promotion.phase=2;}}self.native_group_rosters.record_segment_direction(slot,original_combat::facing(heading),64);}
  }
 }
 pub(crate) fn update_native_dodge(&mut self,index:usize,first_tick:u64,ticks:u64)->bool{
  let Some(mut dodge)=self.units[index].native_dodge.take()else{return false;};
  if !self.map.original||!original_combat::alive(true,self.units[index].hp)||self.units[index].team<0{return false;}
  //The hardware handler continues hatch/APC and body contact updates while
  //its17:1 movement bit is set. Infantry use the ordinary52200 walk frames.
  if stats(&self.units[index].kind).cls==1{self.update_native_hardware(index);}
  let result=self.move_native_local(index,&mut dodge.segment,first_tick,ticks,stats(&self.units[index].kind).cls==0);
  if result==original_motion::LocalResult::Moving{self.units[index].native_dodge=Some(dodge);}
  else{
   //51170 removes40 on arrival or a blocked local attempt.50db0/52810
   //reload the unchanged route; no spare pixels are spent in the new line.
   let p=&mut self.units[index];p.original_motion=None;p.native_flags64&=!64;if let Some(&goal)=p.path.front(){p.native_remaining48=original_motion::Segment::new(p.x,p.y,goal).remaining();}if stats(&p.kind).cls==0{p.native_phase35=if p.path.is_empty(){0}else{2};}
   if let Some(slot)=p.native_slot{if let Some(m)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut){if stats(&p.kind).cls==0{m.promotion.phase=if p.path.is_empty(){0}else{2};}}}
  }
  true
 }
 pub(crate) fn update_native_idle(&mut self,index:usize,busy:bool){
  if !self.map.original{return;}
  let u=self.units[index].clone();
  let eligible=stats(&u.kind).cls==0&&u.team>=0&&original_combat::alive(true,u.hp)&&!busy&&u.path.is_empty()&&u.native_route_pending.get().is_none()&&matches!(u.order.as_str(),"guard"|"move")&&u.initiative.is_none()&&u.workshop.is_none()&&u.transport_carrier.is_none()&&u.native_contact.is_none()&&u.native_cover.is_none()&&u.native_fire.is_none();
  if !eligible{self.units[index].native_idle=None;return;}
  let aim=u.original_combat.as_ref().map(|s|s.aim_skill).unwrap_or(60)as i8;
  let Some(mut idle)=u.native_idle else{
   self.reset_native_idle(index);return;
  };
  let tick=idle.advance(self.motion_tick&1!=0,aim,||self.native_random());
  self.units[index].angle=native_math::angle(idle.direction);self.units[index].aim=self.units[index].angle;
  self.units[index].native_frame32=idle.frame as i16;self.units[index].native_timer3c=idle.timer;
  self.units[index].native_idle=Some(idle);
  if tick.clear_aim{if let Some(slot)=u.native_slot{if let Some(m)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut){m.flags69&=127;}}}
  if let Action::Search{arrived}=tick.action{if self.native_idle_object_scan(index,if arrived{0}else{idle.frame as u8},arrived){return;}
   //44d920 searches for enemies only at its animation search actions. A
   //known in-range target enters the existing native attack-state machine.
   if !arrived{if let Some(target)=self.native_scan(index,0,self.native_unit_stats(&u).range){if self.units[index].native_flags64&0x30==0x10{self.advance_native_fire(index,&target);if self.units[index].native_fire.is_some(){self.units[index].native_idle=None;self.units[index].native_walk=None;}return;}}}
  }
  // The turn-completion micro-step is a local segment, not a replacement
  // player command. Existing legal global destinations remain intact.
  if matches!(tick.action,Action::Search{arrived:true})&&!u.explicit_stop&&u.order=="guard"&&u.dest.is_none()&&u.formation.is_none(){
   let delta=original_idle::SEARCH_STEP[usize::from(idle.desired_direction)];let goal=(u.x+f64::from(delta.0)*2.,u.y+f64::from(delta.1)*2.);
   if self.movement_segment_clear_for(u.x,u.y,goal.0,goal.1,&u.kind){self.prepare_native_walk(index);self.units[index].path=VecDeque::from([goal]);self.units[index].original_motion=None;self.units[index].native_idle=None;return;}
  }
  self.units[index].native_idle=Some(idle);
 }
 fn native_cover_group_gate(&self,index:usize)->reaction::GroupGate{
  let u=&self.units[index];let member=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref);let group=member.map_or(255,|m|m.group);let raw=(group!=255).then(||self.native_group_rosters.pool[usize::from(group)].roster);
  reaction::GroupGate{member:member.map_or(0,|m|m.role),group,flags09:raw.map_or(0,|g|g[9]),timer3d:raw.map_or(0,|g|g[61]),command3a:raw.map_or(0,|g|g[58]),control10:raw.map_or(0,|g|g[16]),count12:raw.map_or(0,|g|g[18]),acquisition_override:false,range04:(self.native_unit_stats(&u).range/8.)as i8}
 }
 pub(crate) fn native_cover_prevented(&mut self,index:usize)->bool{
  let gate=self.native_cover_group_gate(index);if !reaction::group_needs_probe(gate){return true;}
  // Native58500 returns FF in AH when no combat target was acquired. Preserve
  // that distinction; a visible target uses the same quantized scan distance.
  let u=self.units[index].clone();let probe=self.native_scan(index,0,self.native_unit_stats(&u).range).map(|v|{
   let distance=original_hardware::scan_distance(((v.x-u.x)/2.)as i32,((v.y-u.y)/2.)as i32,original_combat::facing(u.angle));u16::from(distance.min(127))
  }).unwrap_or(0xff00);
  reaction::group_prevents_cover(gate,probe)
 }
 pub(crate) fn react_native_projectile(&mut self,point:[i16;2]){
  let origin=reaction::projectile_bucket(point[0]as u16);let mut candidates=Vec::new();
  for offset in reaction::PROJECTILE_BUCKET_OFFSETS{let Some(bucket)=reaction::candidate_bucket(origin,offset)else{continue;};let mut units=self.units.iter().enumerate().filter(|(_,u)|u.team>=0&&original_combat::alive(true,u.hp)&&!workshops::hidden(u)&&(((u.y/2.).floor()as i32>>7)*16+((u.x/2.).floor()as i32>>7))==bucket as i32).map(|(i,u)|(u.native_slot.unwrap_or(u.id),i)).collect::<Vec<_>>();units.sort_unstable();candidates.extend(units.into_iter().take(32).map(|(_,i)|i));}
  for index in candidates.into_iter().take(288){let u=self.units[index].clone();
   if !reaction::projectile_nearby(u.native_class7c,u.native_phase35,point,[(u.x/2.)as i16,(u.y/2.)as i16]){continue;}
   let blocked=self.native_cover_prevented(index);let code=self.native_terrain((u.x/2.)as i32+5,(u.y/2.)as i32+11);
   if let Some(cover)=reaction::begin_cover(original_combat::facing(u.angle)*2,code,blocked){let p=&mut self.units[index];p.native_frame32=cover.frame32;p.native_cover=Some(cover);p.native_phase35=43;p.native_idle=None;p.native_walk=None;p.native_fire=None;p.moving=false;}
  }
 }
 pub(crate) fn update_native_cover(&mut self,index:usize)->bool{
  let Some(mut cover)=self.units[index].native_cover else{return false;};
  let facing=original_combat::facing(self.units[index].angle);let blocked=self.native_cover_prevented(index);
  if reaction::tick_cover(&mut cover,facing*2,(self.motion_tick&1)as u8,blocked){
   let chance=original_combat::CombatState::new(&self.units[index].kind).aim_skill as i8;
   let idle=Idle::enter(facing,chance,||self.native_random());self.units[index].native_frame32=idle.frame as i16;self.units[index].native_timer3c=idle.timer;self.units[index].native_cover=None;self.units[index].native_idle=Some(idle);self.units[index].native_phase35=1;
  }else{self.units[index].native_frame32=cover.frame32;self.units[index].native_cover=Some(cover);}
  self.units[index].moving=false;true
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,751);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.ai_teams.clear();g.winner=None;g.next_id=1;g.spawn("gatling",1,1800.,1200.,9,None);g.units[0].cooldown=10000.;g.units[0].hold_position=true;g}
 #[test]fn animation_initial_phase_and_flags_match_98_executed_native_initializers(){
  let mut hash=2166136261u32;let mut record=|v:&[u32]|{for &n in v{hash=(hash^n).wrapping_mul(16777619);}};
  for team in 0..4{for color in [0,1,2,3,16,32,64,255]{record(&[team,color,initial_phase("grunt")as u32,color|16]);}}
  for(k,kind)in["medium","launcher","heavy","light","apc","jeep","crane","cannon","missile","gatling","howitzer"].iter().enumerate(){for flags in[0,16,17,64,128,255]{record(&[k as u32,flags,u32::from(initial_phase(kind)),flags]);}}
  assert_eq!(hash,818448089,"ZNativeAnimationInitOracle");
 }
 fn idle_robot()->Game{let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.maintain_native_group_rosters();let slot=g.units[1].native_slot.unwrap();g.native_group_rosters.observe_unit_context(slot);g.units[1].native_phase35=1;let mut idle=Idle::enter(0,60,||0);idle.timer=7;g.units[1].native_idle=Some(idle);g}
 #[test]fn raw_idle_preamble_withdraws_without_attack_permission_and_clears_only_group_target(){
  let mut g=idle_robot();let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[59]=0;g.units[1].native_flags64&=!16;let rng=g.original_rng.words;
  assert!(g.update_native_idle_handler(1));assert_eq!(g.units[1].dest,Some((600.,900.)));assert_eq!(g.units[1].native_phase35,2);assert_eq!(g.native_group_rosters.pool[group].roster[59],255);assert_eq!(g.original_rng.words,rng);
 }
 #[test]fn raw_idle_marked_target_and_missing_group_do_not_invent_random_calls(){
  let mut g=idle_robot();let slot=g.units[1].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().promotion.target_x=0;g.units[1].native_flags64|=32;let rng=g.original_rng.words;assert!(g.update_native_idle_handler(1));assert_eq!(g.units[1].dest,Some((1800.,1200.)));assert_eq!(g.original_rng.words,rng);
  let mut g=idle_robot();g.native_group_rosters.current_roster=None;let before=g.units[1].native_idle.as_ref().unwrap().timer;let rng=g.original_rng.words;assert!(g.update_native_idle_handler(1));assert_eq!(g.units[1].native_idle.as_ref().unwrap().timer,before);assert_eq!(g.original_rng.words,rng);
 }
 #[test]fn raw_idle_group_sync_preserves_the_native_50_54_56_fields(){
  let mut g=idle_robot();let slot=g.units[1].native_slot.unwrap();let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.record_group_order(0,1,1,[-123,321],7);g.native_group_rosters.pool[group].roster[62]=1;let m=g.native_group_rosters.members[slot].as_mut().unwrap();m.promotion.order=12;m.flags69|=128;
  assert!(g.update_native_idle_handler(1));let m=g.native_group_rosters.members[slot].as_ref().unwrap();assert_eq!((m.promotion.order,m.promotion.target_x,m.promotion.target_y,m.promotion.target_aux),(1,-123,321,7));
 }
 #[test]fn partial_routes_and_manual_attack_orders_bypass_idle_dispatch(){
  let mut g=idle_robot();g.units[1].native_phase35=0;g.units[1].native_route_pending.set(Some((1400.,1400.)));assert!(!g.update_native_idle_handler(1));assert_eq!(g.units[1].native_phase35,0);
  g.units[1].native_route_pending.set(None);g.units[1].order="attackBuilding".into();assert!(!g.update_native_idle_handler(1));assert_eq!(g.units[1].order,"attackBuilding");
 }
 #[test]fn stopped_robot_idle_matches_the_native_counter_and_rng_sequence(){
  let mut g=field();g.spawn("grunt",0,320.,320.,1,None);let i=1;g.units[i].explicit_stop=true;g.units[i].hold_position=true;
  let mut rng=g.original_rng.clone();let mut expected=original_idle::State::enter(0,60,||rng.next_u32());g.step(0.059);
  assert_eq!(g.original_rng.words,rng.words);
  for tick in 2..=120{expected.advance(tick&1!=0,60,||rng.next_u32());g.step(0.059);let idle=g.units[i].native_idle.as_ref().unwrap();assert_eq!((idle.frame,idle.direction,idle.timer),(expected.frame,expected.direction,expected.timer));assert_eq!(g.original_rng.words,rng.words);assert_eq!((g.units[i].x,g.units[i].y),(320.,320.));}
 }
 #[test]fn nearby_projectile_uses_original_bucket_and_only_an_idle_follower_takes_cover(){
  let mut g=field();g.spawn("grunt",0,14.,128.,1,None);g.spawn("grunt",0,14.,80.,1,None);g.step(0.059);let pos=(g.units[2].x,g.units[2].y);let hp=g.units[2].hp;
  g.react_native_projectile([7,40]);assert!(g.units[1].native_cover.is_none());let cover=g.units[2].native_cover.unwrap();assert_eq!(cover.frame32,0);assert_eq!(reaction::COVER_SPRITE_BASE,2075);
  for _ in 0..14{g.step(0.059);assert_eq!((g.units[2].x,g.units[2].y),pos);assert_eq!(g.units[2].hp,hp);}
  assert!(g.units[2].native_cover.is_none());assert!(g.units[2].native_idle.is_some());
 }
 #[test]fn bonus_timer_and_water_both_block_cover_without_replacing_the_current_order(){
  for water in [false,true]{let mut g=field();g.spawn("grunt",0,14.,128.,1,None);g.spawn("grunt",0,14.,80.,1,None);g.step(0.059);
   if water{g.map.terrain_codes[3*g.map.width]=9;}else{g.units[2].group_bonus_ticks=30;let group=g.native_group_rosters.group_slot(0,1).unwrap();g.native_group_rosters.pool[group].roster[61]=30;}
   g.react_native_projectile([7,40]);assert!(g.units[2].native_cover.is_none());assert_eq!(g.units[2].order,"guard");
  }
 }
 #[test]fn the_shared_native_unit_pool_has_exactly_240_slots(){
  let mut g=field();g.units.clear();for n in 0..240{g.spawn("grunt",(n&1)as i32,320.,320.,n,None);}let next=g.next_id;g.spawn("grunt",0,320.,320.,241,None);assert_eq!(g.units.len(),240);assert_eq!(g.next_id,next);g.units[17].hp=-1.;g.spawn("grunt",0,320.,320.,242,None);assert_eq!(g.units.last().unwrap().native_slot,Some(17));
 }
 #[test]fn incoming_shell_spends_exactly_two_random_values_and_dodges_for_a_segment(){
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.units[1].original_combat.as_mut().unwrap().movement_control=70;
  let mut rng=g.original_rng.clone();let goal=reaction::dodge_destination([500,500],[508,508],[rng.next_u32(),rng.next_u32()]);
  g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert_eq!(g.original_rng.words,rng.words);assert_eq!(g.units[1].native_dodge.as_ref().unwrap().counter8c,2);
  //A second shell cannot retarget an unfinished dodge or consume more RNG.
  g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert_eq!(g.original_rng.words,rng.words);
  for tick in 0..2{assert!(g.update_native_dodge(1,tick,1));}assert!(g.units[1].native_dodge.is_some());
  for tick in 2..100{if !g.update_native_dodge(1,tick,1){break;}}
  assert!(g.units[1].native_dodge.is_none());assert_eq!((g.units[1].x,g.units[1].y),(f64::from(goal[0])*2.,f64::from(goal[1])*2.));assert_eq!(g.units[1].order,"guard");assert_eq!(g.original_rng.words,rng.words);
 }
 #[test]fn hardware_dodge_retains_the_primary_route_and_resumes_from_the_new_anchor(){
  let mut g=field();g.spawn("light",0,1000.,1000.,1,None);let p=&mut g.units[1];p.original_combat.as_mut().unwrap().movement_control=70;p.order="move".into();p.path=VecDeque::from([(1500.,1000.),(1600.,1200.)]);p.dest=Some((1600.,1200.));let path=p.path.clone();
  g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert!(g.units[1].native_dodge.is_some());assert_eq!(g.units[1].path,path);
  for tick in 0..100{if !g.update_native_dodge(1,tick,1){break;}assert_eq!(g.units[1].path,path);}
  let after=(g.units[1].x,g.units[1].y);assert_ne!(after,(1000.,1000.));assert_eq!(g.units[1].order,"move");assert_eq!(g.units[1].dest,Some((1600.,1200.)));
  g.move_original(1,100,1,false);assert_ne!((g.units[1].x,g.units[1].y),after);assert_eq!(g.units[1].path,path);
 }
 #[test]fn walking_infantry_guns_allies_and_promoting_state7_do_not_dodge(){
  for kind in ["grunt","cannon","missile","gatling","howitzer"]{let mut g=field();g.spawn(kind,0,1000.,1000.,1,None);if kind=="grunt"{g.units[1].path.push_back((1400.,1000.));g.units[1].native_phase35=2;}let rng=g.original_rng.words;g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert!(g.units[1].native_dodge.is_none(),"{kind}");assert_eq!(g.original_rng.words,rng);}
  let mut g=field();g.spawn("grunt",1,1000.,1000.,1,None);let rng=g.original_rng.words;g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert!(g.units[1].native_dodge.is_none());assert_eq!(g.original_rng.words,rng);
  g.units[1].team=0;g.units[1].native_phase35=7;g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert!(g.units[1].native_dodge.is_none());
 }
 #[test]fn dodge_uses_full_target_bucket_and_the_current_shooter_slot_owner(){
  let mut g=field();g.spawn("grunt",0,1000.,3000.,1,None);g.react_native_dodge(Some(0),0,[500,1500],[508,1508]);assert!(g.units[1].native_dodge.is_some());
  //The projectile's original team is immaterial after that slot changes side.
  g.units[1].native_dodge=None;g.units[1].native_flags64&=!64;g.units[1].native_phase35=0;g.units[0].team=0;let rng=g.original_rng.words;g.react_native_dodge(Some(0),1,[500,1500],[508,1508]);assert!(g.units[1].native_dodge.is_none());assert_eq!(g.original_rng.words,rng);
 }
 #[test]fn blocked_local_dodge_returns_without_entering_water_or_losing_its_route(){
  let mut g=field();g.spawn("light",0,1000.,1000.,1,None);g.units[1].original_combat.as_mut().unwrap().movement_control=70;g.units[1].order="move".into();g.units[1].path=VecDeque::from([(1500.,1000.)]);let path=g.units[1].path.clone();
  g.react_native_dodge(Some(0),1,[500,500],[508,508]);assert!(g.units[1].native_dodge.is_some());
  for y in 29..35{for x in 29..35{if (x,y)!=(31,31){g.map.tiles[y*g.map.width+x]=3;g.map.terrain_codes[y*g.map.width+x]=9;}}}
  for tick in 0..100{if !g.update_native_dodge(1,tick,1){break;}}
  assert!(g.units[1].native_dodge.is_none());assert_eq!(((g.units[1].x/32.).floor()as i32,(g.units[1].y/32.).floor()as i32),(31,31));assert_eq!(g.units[1].path,path);assert_eq!(g.units[1].order,"move");
 }
 #[test]fn the_real_game_callback_advances_a_dodge_before_idle_and_combat(){
  let mut g=field();g.spawn("grunt",0,1000.,1000.,1,None);g.units[1].original_combat.as_mut().unwrap().movement_control=70;g.units[1].hold_position=true;g.react_native_dodge(Some(0),1,[500,500],[508,508]);
  let before=(g.units[1].x,g.units[1].y);g.step(0.059);assert!(g.units[1].native_dodge.is_some());assert_ne!((g.units[1].x,g.units[1].y),before);assert!(g.units[1].native_idle.is_none());assert!(g.units[1].native_fire.is_none());
 }
}
