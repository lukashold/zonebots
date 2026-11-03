//! Shared group counters0044b280,004538f0,0044a900. A personal initiative
//! bonus and the group's temporary motion bonus are separate native fields.
use super::Game;
use std::collections::BTreeMap;
impl Game{
 pub(crate) fn update_native_groups(&mut self){
  if !self.map.original{return;}
  let mut groups=BTreeMap::new();
  for u in &self.units{let counters=groups.entry((u.team,u.squad)).or_insert((0u8,0u8));counters.0=counters.0.max(u.group_activity_ticks);counters.1=counters.1.max(u.group_bonus_ticks);}
  for (activity,bonus) in groups.values_mut(){*activity=activity.saturating_sub(1);*bonus=if *activity==0{0}else{bonus.saturating_sub(1)};}
  //0045a080 prepares the movement index before the state handler can start a
  // new order. A counter written by004538f0 affects the following callback.
  for u in &mut self.units{let(a,b)=groups[&(u.team,u.squad)];u.group_activity_ticks=a;u.group_bonus_ticks=b;u.native_group_motion_bonus=if b>0{3}else{0};}
 }
 pub(crate) fn start_native_group_order(&mut self,index:usize){
  if !self.map.original||!self.units[index].native_group_order_pending{return;}
  let u=self.units[index].clone();let ai=self.ai_teams.contains(&u.team);
  let bonus=if u.group_activity_ticks==0{30}else{120};
  for member in self.units.iter_mut().filter(|v|v.team==u.team&&v.squad==u.squad){
   member.native_group_order_pending=false;if !ai{member.group_bonus_ticks=bonus;}
  }
 }
 pub(crate) fn activate_native_combat_groups(&mut self,index:usize,target:usize){
  if !self.map.original{return;}
  let source=(self.units[index].team,self.units[index].squad);
  let other=self.units.iter().find(|u|u.id==target).map(|u|(u.team,u.squad));
  for u in &mut self.units{let group=(u.team,u.squad);if group==source||Some(group)==other{u.group_activity_ticks=75;}}
 }
}
#[cfg(test)]mod tests{
 use super::*;
 use crate::{json,original_combat};
 fn field()->Game{let mut g=Game::new(0,1,1);g.map.original=true;g.units.clear();g.ai_teams.clear();g.spawn("grunt",0,320.,320.,1,None);g.spawn("grunt",0,352.,320.,1,None);g.spawn("light",1,640.,320.,2,None);g}
 #[test]fn group_counters_expire_together_and_combat_refresh_does_not_restart_motion_bonus(){
  let mut g=field();g.activate_native_combat_groups(0,g.units[2].id);
  for u in &mut g.units[..2]{u.group_bonus_ticks=120;}
  for _ in 0..74{g.update_native_groups();}
  assert!(g.units[..2].iter().all(|u|u.group_activity_ticks==1&&u.group_bonus_ticks==46));
  g.activate_native_combat_groups(0,g.units[2].id);g.update_native_groups();
  assert!(g.units[..2].iter().all(|u|u.group_activity_ticks==74&&u.group_bonus_ticks==45));
  for _ in 0..74{g.update_native_groups();}assert!(g.units.iter().all(|u|u.group_activity_ticks==0&&u.group_bonus_ticks==0));
 }
 #[test]fn a_new_path_uses_the_native_30_or_120_counter_and_ai_has_no_player_bonus(){
  for active in [false,true]{let mut g=field();if active{g.activate_native_combat_groups(0,g.units[2].id);}
   let ids=g.units[..2].iter().map(|u|u.id).collect::<Vec<_>>();g.command(&json!({"kind":"move","ids":ids,"x":500.,"y":320.}));
   assert_eq!(g.units[0].group_bonus_ticks,0);g.start_native_group_order(0);
   assert!(g.units[..2].iter().all(|u|u.group_bonus_ticks==if active{120}else{30}));
   g.update_native_groups();assert_eq!(g.units[0].group_bonus_ticks,if active{119}else{0});
  }
  let mut g=field();g.ai_teams.push(0);g.units[0].native_group_order_pending=true;g.start_native_group_order(0);assert_eq!(g.units[0].group_bonus_ticks,0);
  assert!(original_combat::alive(true,g.units[0].hp));
 }
}
