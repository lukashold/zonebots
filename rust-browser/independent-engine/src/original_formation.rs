//! Native follower geometry0044b6d0/0044d430, using the default V layout.
//! Followers target the leader's current position plus a rotated native offset;
//! they do not receive arbitrary12x14 browser grid destinations.
use super::{Game,stats,original_combat,native_math};
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap,HashSet,VecDeque};

const V:[[i32;2];20]=[[-16,16],[16,16],[-32,32],[0,32],[32,32],[-48,48],[-16,48],[16,48],[48,48],[-16,32],[16,32],[-32,48],[32,48],[-48,64],[-32,64],[-16,64],[0,64],[16,64],[32,64],[48,64]];
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Follow {
 pub leader:usize,pub slot:usize,pub command_goal:(f64,f64),pub direction:u8,
 #[serde(skip)] last_goal:Option<(f64,f64)>,
 #[serde(skip)] next_plan:f64,
 #[serde(skip)] revision:u32,
}
pub fn offset(slot:usize,direction:u8)->(f64,f64) {
 let p=V[slot.saturating_sub(1).min(19)];let a=-(i32::from(direction))*64;
 let s=native_math::sine(a);let c=native_math::sine(a+128);
 (f64::from((p[1]*s>>10)+(p[0]*c>>10))*2.,f64::from((p[1]*c>>10)-(s*p[0]>>10))*2.)
}
impl Game {
 pub(crate) fn prepare_native_formations(&mut self,ids:&[usize],team:i32,goal:(f64,f64)) {
  let chosen:HashSet<_>=ids.iter().copied().collect();let mut squads:BTreeMap<usize,Vec<usize>>=BTreeMap::new();
  for(i,u)in self.units.iter().enumerate(){if u.team==team&&original_combat::alive(true,u.hp)&&stats(&u.kind).cls==0&&chosen.contains(&u.id){squads.entry(u.squad).or_default().push(i);}}
  let mut fresh=self.units.iter().map(|u|u.squad).max().unwrap_or(0)+1;
  for(squad,mut members)in squads {
   members.sort_by_key(|&i|self.units[i].native_slot.unwrap_or(self.units[i].id));
   let all=self.units.iter().filter(|u|u.squad==squad&&u.team==team&&stats(&u.kind).cls==0&&original_combat::alive(true,u.hp)).count();
   if members.len()<all {for &i in &members{self.units[i].squad=fresh;}fresh+=1;}
   let lead=members[0];let leader=self.units[lead].id;
   let direction=if (goal.0-self.units[lead].x).abs()+(goal.1-self.units[lead].y).abs()<1.{original_combat::facing(self.units[lead].angle)}
       else{native_math::direction(((goal.0-self.units[lead].x)/2.)as i32,((goal.1-self.units[lead].y)/2.)as i32)};
   for(rank,&i)in members.iter().enumerate(){
    self.units[i].formation=if rank==0||rank>20{None}else{Some(Follow{leader,slot:rank,command_goal:goal,direction,last_goal:None,next_plan:0.,revision:self.map.revision})};
    self.units[i].motion_bonus=0;
   }
  }
 }
 pub(crate) fn native_formation_destination(&self,index:usize,goal:(f64,f64))->(f64,f64) {
  self.units[index].formation.as_ref().map(|f|{let d=offset(f.slot,f.direction);(goal.0+d.0,goal.1+d.1)}).unwrap_or(goal)
 }
 pub(crate) fn update_native_follower(&mut self,index:usize) {
  let u=self.units[index].clone();let Some(mut f)=u.formation.clone()else{self.units[index].motion_bonus=0;return;};
  if u.order!="move"||u.initiative.is_some()||u.hold_position||u.workshop.is_some(){self.units[index].motion_bonus=0;return;}
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
  let delta=offset(f.slot,f.direction);let goal=(leader.x+delta.0,leader.y+delta.1);
  let endpoint=(f.command_goal.0+delta.0,f.command_goal.1+delta.1);
  if (u.x-goal.0).abs()+(u.y-goal.1).abs()<1. {
   let p=&mut self.units[index];p.path.clear();p.original_motion=None;p.motion_bonus=0;p.dest=Some(endpoint);p.formation=Some(f);return;
  }
  let direct=self.movement_segment_clear_for(u.x,u.y,goal.0,goal.1,&u.kind);
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
 fn field()->Game {
  let mut g=Game::new(0,1,891);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
  g.units.clear();g.buildings.clear();g.crates.clear();g.sectors.clear();g.walls.clear();g.objects.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.next_id=1;g.winner=None;
  for x in [320.,352.,384.]{g.spawn("grunt",0,x,600.,1,None);}
  g.spawn("gatling",1,1500.,1000.,2,None);for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}g
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
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();march(&mut g,&ids);g.units[0].hp=-1.;
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
