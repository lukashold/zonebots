//! Native follower geometry0044b6d0/0044d430, with all five native masks.
//! Followers target the leader's current position plus a rotated native offset;
//! they do not receive arbitrary12x14 browser grid destinations.
use super::{Game,stats,original_combat,native_math};
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap,HashSet,VecDeque};

const LAYOUTS:[[[i32;2];20];5]=[[[-16,16],[16,16],[-32,32],[0,32],[32,32],[-48,48],[-16,48],[16,48],[48,48],[-16,32],[16,32],[-32,48],[32,48],[-48,64],[-32,64],[-16,64],[0,64],[16,64],[32,64],[48,64]],[[-16,16],[0,16],[16,16],[-16,32],[0,32],[16,32],[-16,48],[0,48],[16,48],[-16,64],[0,64],[16,64],[-16,80],[0,80],[16,80],[-16,96],[0,96],[16,96],[-16,112],[16,112]],[[-32,48],[32,48],[-32,-48],[32,-48],[0,48],[0,-48],[-32,0],[32,0],[-16,48],[16,48],[-16,-48],[16,-48],[-32,32],[32,32],[-32,-32],[32,-32],[-32,16],[32,16],[0,0],[32,-16]],[[-16,0],[16,0],[-32,0],[32,0],[-48,0],[48,0],[-64,0],[64,0],[-80,0],[80,0],[-96,0],[96,0],[-112,0],[112,0],[-96,-64],[96,-64],[-96,64],[96,64],[0,-112],[0,112]],[[-32,48],[32,48],[-32,-48],[32,-48],[0,48],[0,-48],[-32,0],[32,0],[-16,48],[16,48],[-16,-48],[16,-48],[-32,32],[32,32],[-32,-32],[32,-32],[-32,16],[32,16],[-32,-16],[32,-16]]];
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Follow {
 #[serde(default)] pub layout:u8,
 pub leader:usize,pub slot:usize,pub command_goal:(f64,f64),pub direction:u8,
 #[serde(skip)] last_goal:Option<(f64,f64)>,
 #[serde(skip)] next_plan:f64,
 #[serde(skip)] revision:u32,
}
impl Follow{
 pub(crate) fn new(leader:usize,slot:usize,layout:u8,command_goal:(f64,f64),direction:u8,revision:u32)->Self{Self{leader,slot,layout,command_goal,direction,last_goal:None,next_plan:0.,revision}}
}
pub(crate) fn raw_offset(slot:usize,layout:u8)->[i8;2]{let role=slot.clamp(1,20);if layout==255{return[0,(role*10)as i8];}let p=LAYOUTS[usize::from(layout.min(4))][role-1];[p[0]as i8,p[1]as i8]}
#[cfg(test)]
pub fn offset(slot:usize,direction:u8)->(f64,f64) {offset_layout(slot,direction,0)}
pub fn offset_layout(slot:usize,direction:u8,layout:u8)->(f64,f64) {
 let raw=raw_offset(slot,layout);let p=[i32::from(raw[0]),i32::from(raw[1])];let a=-(i32::from(direction))*64;
 let s=native_math::sine(a);let c=native_math::sine(a+128);
 (f64::from((p[1]*s>>10)+(p[0]*c>>10))*2.,f64::from((p[1]*c>>10)-(s*p[0]>>10))*2.)
}
impl Game {
 pub(crate) fn prepare_native_formations(&mut self,ids:&[usize],team:i32,goal:(f64,f64)) {
  let chosen:HashSet<_>=ids.iter().copied().collect();let mut squads:BTreeMap<usize,Vec<usize>>=BTreeMap::new();
  for(i,u)in self.units.iter().enumerate(){if u.team==team&&original_combat::alive(true,u.hp)&&stats(&u.kind).cls==0&&chosen.contains(&u.id){squads.entry(u.squad).or_default().push(i);}}
  let mut fresh=self.units.iter().map(|u|u.squad).max().unwrap_or(0)+1;
  for(squad,mut members)in squads {
   members.sort_by_key(|&i|{let u=&self.units[i];let role=u.native_slot.and_then(|slot|self.native_group_rosters.role(slot));(match role{Some(-1)=>0,Some(r)if r>0=>r as usize,_=>usize::MAX},u.native_slot.unwrap_or(u.id))});
   let all=self.units.iter().filter(|u|u.squad==squad&&u.team==team&&stats(&u.kind).cls==0&&original_combat::alive(true,u.hp)).count();
   if members.len()<all {for &i in &members{self.units[i].squad=fresh;}fresh+=1;}
   let lead=members[0];let leader=self.units[lead].id;
   let direction=if (goal.0-self.units[lead].x).abs()+(goal.1-self.units[lead].y).abs()<1.{original_combat::facing(self.units[lead].angle)}
       else{native_math::direction(((goal.0-self.units[lead].x)/2.)as i32,((goal.1-self.units[lead].y)/2.)as i32)};
   for(rank,&i)in members.iter().enumerate(){
    self.units[i].formation=if rank==0||rank>20{None}else{Some(Follow{leader,slot:rank,layout:self.units[i].formation_layout,command_goal:goal,direction,last_goal:None,next_plan:0.,revision:self.map.revision})};
    self.units[i].motion_bonus=0;
   }
  }
 }
 pub(crate) fn native_formation_destination(&self,index:usize,goal:(f64,f64))->(f64,f64) {
  self.units[index].formation.as_ref().map(|f|{let d=offset_layout(f.slot,f.direction,f.layout);(goal.0+d.0,goal.1+d.1)}).unwrap_or(goal)
 }
 pub(crate) fn update_native_follower(&mut self,index:usize) {
  let u=self.units[index].clone();let Some(mut f)=u.formation.clone()else{return;};
  if let Some(layout)=self.native_group_rosters.formation_layout(u.team,u.squad){f.layout=layout;}
  if u.order!="move"||u.initiative.is_some()||u.hold_position||u.workshop.is_some(){self.units[index].motion_bonus=if u.initiative.is_some(){3}else{0};return;}
  let leader=self.units.iter().find(|p|p.id==f.leader&&p.squad==u.squad&&p.team==u.team&&original_combat::alive(true,p.hp)).cloned();
  let leader=if let Some(leader)=leader{leader}else{
   let promoted=self.units.iter().filter(|p|p.squad==u.squad&&p.team==u.team&&original_combat::alive(true,p.hp)&&stats(&p.kind).cls==0).min_by_key(|p|p.native_slot.unwrap_or(p.id)).cloned();
   let Some(promoted)=promoted else{return;};
   if promoted.id==u.id {
    let path=self.route_unit(&u,f.command_goal.0,f.command_goal.1);let p=&mut self.units[index];
    p.formation=None;p.motion_bonus=0;p.dest=Some(f.command_goal);p.path=path;p.original_motion=None;return;
   }
   f.leader=promoted.id;promoted
  };
  if leader.moving{f.direction=original_combat::facing(leader.angle);}
  let delta=offset_layout(f.slot,f.direction,f.layout);let goal=(leader.x+delta.0,leader.y+delta.1);
  let endpoint=(f.command_goal.0+delta.0,f.command_goal.1+delta.1);
  if (u.x-goal.0).abs()+(u.y-goal.1).abs()<1. {
   let p=&mut self.units[index];p.path.clear();p.original_motion=None;p.motion_bonus=0;p.dest=Some(endpoint);p.formation=Some(f);return;
  }
  let direct=self.movement_segment_clear_unit(&u,goal);
  let changed=f.last_goal.is_none_or(|old|(old.0-goal.0).abs()+(old.1-goal.1).abs()>=if direct{2.}else{32.});
  // Native followers update their local segment continuously. The browser's
  // global route adapter must not run a whole-map search for every leader pixel.
  // Obstructed paths are reused, with bounded retry and immediate map revision.
  if direct&&(changed||u.path.is_empty())||!direct&&(self.time>=f.next_plan&&(changed||u.path.is_empty())||f.revision!=self.map.revision){
   let path=if direct{VecDeque::from([goal])}
       else{self.route_unit(&u,goal.0,goal.1)};
   let p=&mut self.units[index];p.path=path;p.original_motion=None;f.last_goal=Some(goal);f.next_plan=self.time+0.5;f.revision=self.map.revision;
  }
  let p=&mut self.units[index];p.dest=Some(endpoint);p.motion_bonus=if p.path.is_empty(){0}else{4};p.formation=Some(f);
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 use crate::json;
 #[test]
 fn all_five_layouts_match_executed_native_mask_generation_including_the_empty_slot(){
  let mut hash=2166136261u32;
  for(layout,slots)in LAYOUTS.iter().enumerate(){for(slot,p)in slots.iter().enumerate(){for n in [layout as i32,slot as i32+1,p[0],p[1]]{hash=(hash^n as u32).wrapping_mul(16777619);}}}
  assert_eq!(hash,2104783681,"ZNativeAnimationOracle,100 original follower positions");
  assert_eq!(LAYOUTS[2][18],[0,0]);
 }
 #[test]
 fn formation_choice_changes_followers_without_changing_the_leader_goal(){
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();
  g.command(&json!({"kind":"move","ids":ids,"x":800.,"y":600.,"formation":1}));
  assert_eq!(g.units[0].dest,Some((800.,600.)));assert_eq!(g.units[1].formation.as_ref().unwrap().layout,1);
  assert_eq!(g.units[2].dest,Some((768.,600.)));
 }
 fn field()->Game {
  let mut g=Game::new(0,1,891);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
  g.units.clear();g.buildings.clear();g.crates.clear();g.sectors.clear();g.walls.clear();g.objects.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.next_id=1;g.winner=None;
  for x in [320.,352.,384.]{g.spawn("grunt",0,x,600.,1,None);}
  g.spawn("gatling",1,1500.,1000.,2,None);for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}g.maintain_native_group_rosters();g
 }
 fn march(g:&mut Game,ids:&[usize]) {g.command(&json!({"kind":"move","ids":ids,"x":800.,"y":600.}));}
 #[test]
 fn native_rotation_keeps_signed_shift_and_sincos_quantization() {
  assert_eq!(offset(1,0),(-32.,30.));assert_eq!(offset(2,0),(30.,30.));
  assert_eq!(offset(1,2),(-32.,-32.));assert_eq!(offset(2,2),(-32.,32.));
  assert_eq!(offset(1,4),(32.,-32.));assert_eq!(offset(1,6),(30.,32.));
 }
 #[test]
 fn one_squad_marches_as_leader_and_rotated_followers_then_settles_in_formation() {
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();march(&mut g,&ids);
  assert_eq!(g.units[0].dest,Some((800.,600.)));assert_eq!(g.units[1].dest,Some((768.,568.)));assert_eq!(g.units[2].dest,Some((768.,632.)));
  let mut boosted=false;
  for _ in 0..700{g.step(0.059);boosted|=g.units.iter().any(|u|u.motion_bonus==4);}
  assert!(boosted);assert_eq!((g.units[0].x,g.units[0].y),(800.,600.));
  assert_eq!((g.units[1].x,g.units[1].y),(768.,568.));assert_eq!((g.units[2].x,g.units[2].y),(768.,632.));
  assert!(g.units[..3].iter().all(|u|u.path.is_empty()));
 }
 #[test]
 fn an_individual_move_detaches_the_selected_robot_and_does_not_drag_its_old_squad() {
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();march(&mut g,&ids);g.step(0.059);
  let old_squad=g.units[0].squad;g.command(&json!({"kind":"move","ids":[ids[2]],"x":400.,"y":800.}));
  assert_ne!(g.units[2].squad,old_squad);assert!(g.units[2].formation.is_none());assert_eq!(g.units[2].dest,Some((400.,800.)));
  assert_eq!(g.units[0].dest,Some((800.,600.)));assert_eq!(g.units[1].formation.as_ref().unwrap().leader,ids[0]);
 }
 #[test]
 fn loss_of_the_leader_promotes_a_survivor_and_preserves_the_original_goal() {
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();march(&mut g,&ids);g.maintain_native_group_rosters();let follower_slot=g.units[2].native_slot.unwrap();g.units[0].hp=-1.;g.remove_original_group_member(0);
  //4b990 promotes the first occupied role without compacting the other roles.
  //4bba0 marks40 only when a follower was removed; combat TTL is zero here.
  let group=g.native_group_rosters.group_slot(0,1).unwrap();assert_eq!(g.native_group_rosters.pool[group].member(1),None);assert_eq!(g.native_group_rosters.role(follower_slot),Some(2));
  for _ in 0..700{g.step(0.059);}
  let leader=g.units.iter().find(|u|u.id==ids[1]).unwrap();assert!(leader.formation.is_none());assert_eq!((leader.x,leader.y),(800.,600.));
  let follower=g.units.iter().find(|u|u.id==ids[2]).unwrap();assert_eq!(follower.formation.as_ref().unwrap().leader,ids[1]);
  assert_eq!((follower.x,follower.y),(768.,632.));
 }
 #[test]
 fn explicit_stop_cancels_following_without_later_catch_up() {
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();march(&mut g,&ids);for _ in 0..10{g.step(0.059);}
  g.command(&json!({"kind":"stop","ids":ids}));let positions=g.units[..3].iter().map(|u|(u.x,u.y)).collect::<Vec<_>>();
  for _ in 0..100{g.step(0.059);}assert_eq!(positions,g.units[..3].iter().map(|u|(u.x,u.y)).collect::<Vec<_>>());
  assert!(g.units[..3].iter().all(|u|u.formation.is_none()&&u.explicit_stop));
 }
}
