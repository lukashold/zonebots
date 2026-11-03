//! Vehicle/robot contact0045c210 and knockdown0045b9c0/0045a6b0/0045a740.
//! Evasive segments retain the existing live terrain collision adapter.
use super::{Game,stats,original_combat};
use serde::{Serialize,Deserialize};
use std::collections::VecDeque;
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct State{
 pub phase:u8,pub frame:u8,pub side:bool,
 #[serde(skip)] path:VecDeque<(f64,f64)>,
 #[serde(skip)] dest:Option<(f64,f64)>,
}
impl State{
 fn advance(&mut self,nearby:bool,even:bool)->bool{
  if self.phase==0{if !nearby{self.phase=1;}}
  else if self.phase==1&&even{self.frame+=1;return self.frame>10;}
  false
 }
}
fn action(dx:i32,dy:i32,bridge:bool,protected:bool,hp:i8)->u8{
 if !protected&&(-8..=16).contains(&dx)&&(-8..=16).contains(&dy){return if hp<=10{1}else{2};}
 let margin=if bridge{18}else{23};let right=if bridge{26}else{31};let bottom=if bridge{22}else{27};
 if(-margin..=right).contains(&dx)&&(-margin..=bottom).contains(&dy){3}else{0}
}
impl Game{
 pub(crate) fn update_native_vehicle_contacts(&mut self,index:usize){
  if !self.map.original||self.motion_tick&1!=0||stats(&self.units[index].kind).cls!=1{return;}
  let u=self.units[index].clone();if u.team<0{return;}
  let(x,y)=((u.x/2.).floor()as i32,(u.y/2.).floor()as i32);
  let col=((x+8)>>4)as usize;let row=((y+8)>>4)as usize;
  let bridge=col<self.map.width&&row<self.map.height&&matches!(self.map.terrain_codes.get(row*self.map.width+col),Some(7|51));
  let coarse=(y>>7)*16+(x>>7);let mut candidates=Vec::new();
  for delta in [0,-1,1,-17,-16,-15,15,16,17]{
   let cell=coarse+delta;if !(0..256).contains(&cell){continue;}
   let mut bucket=self.units.iter().enumerate().filter(|(_,v)|v.team>=0&&original_combat::alive(true,v.hp)&&!super::workshops::hidden(v)
    &&(((v.y/2.).floor()as i32>>7)*16+((v.x/2.).floor()as i32>>7))==cell).map(|(i,v)|(v.native_slot.unwrap_or(v.id),i)).collect::<Vec<_>>();
   bucket.sort_unstable();candidates.extend(bucket.into_iter().take(32).map(|(_,i)|i));
  }
  for i in candidates{
   let v=self.units[i].clone();if v.team==u.team||stats(&v.kind).cls!=0||v.native_contact.as_ref().is_some_and(|s|s.phase<2){continue;}
   let(vx,vy)=((v.x/2.).floor()as i32,(v.y/2.).floor()as i32);
   let protected=self.buildings.iter().filter(|b|b.kind=="fort"&&b.team>=0).any(|b|{
    // Campaign rawBase is already in native pixels (unlike building x/y).
    let base=b.raw_base.unwrap_or((b.x/2.,b.y/2.));let(bx,by)=(base.0 as i32,base.1 as i32);
    vx>=bx&&vx<=bx+160&&vy>=by&&vy<=by+270
   });
   match action(vx-x,vy-y,bridge,protected,v.hp as i8){
    1=>{self.activate_native_combat_groups(i,u.id);self.units[i].hp=-1.;self.remove_original_group_member(i);self.finish_order(i);},
    2=>{
     self.activate_native_combat_groups(i,u.id);
     let side=(original_combat::facing(v.angle).wrapping_add(2)&4)!=0;
     let next=(v.hp as i8).wrapping_sub(10);self.units[i].hp=f64::from(if next<0{10}else{next});
     let p=&mut self.units[i];p.native_contact=Some(State{phase:0,frame:0,side,path:VecDeque::new(),dest:None});p.native_cover=None;p.native_idle=None;
     p.order="guard".into();p.target=None;p.dest=None;p.path.clear();p.original_motion=None;p.native_walk=None;p.native_fire=None;p.formation=None;p.motion_bonus=0;p.moving=false;
     p.angle=super::native_math::angle(0);p.aim=p.angle;p.native_frame32=0;p.native_phase35=if side{46}else{23};
    },
    3=>{
     let goal=(f64::from(vx*2-x-4)*2.,f64::from(vy*2-y-2)*2.);
     if !self.movement_segment_clear_unit(&v,goal){continue;}
     let old=v.native_contact.unwrap_or(State{phase:2,frame:0,side:false,path:v.path.clone(),dest:v.dest});
     self.units[i].native_contact=Some(old);self.units[i].native_phase35=2;self.units[i].path=VecDeque::from([goal]);self.units[i].dest=Some(goal);self.units[i].original_motion=None;self.units[i].native_fire=None;self.units[i].motion_bonus=4;
    },_=>{}
   }
  }
 }
 pub(crate) fn update_native_contact_state(&mut self,index:usize)->bool{
  if !self.map.original{return false;}
  let u=self.units[index].clone();let Some(mut state)=u.native_contact else{return false;};
  if state.phase==2{
   if self.units[index].path.is_empty(){let p=&mut self.units[index];p.native_contact=None;p.native_phase35=0;p.path=state.path;p.dest=state.dest;p.original_motion=None;p.motion_bonus=0;return false;}
   self.move_original(index,self.motion_tick,1,true);return true;
  }
  self.units[index].moving=false;
  //0045a9f0 caches this list before any unit handler moves a hull.
  let nearby=state.phase==0&&self.native_vehicle_positions.iter().any(|&(x,y)|
    (x-(u.x/2.).floor()as i32).abs()<24&&(y-(u.y/2.).floor()as i32).abs()<24);
  if state.advance(nearby,self.motion_tick&1==0){
    //0044cac0 starts idle with two random counters.
    self.native_random();self.native_random();self.units[index].native_contact=None;self.units[index].native_phase35=0;return true;
  }
  self.units[index].native_phase35=if state.side{if state.phase==0{46}else{47}}else if state.phase==0{23}else{24};
  self.units[index].native_frame32=i16::from(state.frame);self.units[index].native_contact=Some(state);true
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_338_inner_outer_cases_match_the_executed_native_vehicle_contacts(){
  let mut h=2166136261u32;
  for bridge in 0..2{for dx in [-24,-23,-19,-18,-9,-8,0,16,17,26,27,31,32]{for dy in [-24,-23,-19,-18,-9,-8,0,16,17,22,23,27,28]{
   let a=action(dx,dy,bridge!=0,false,30);let(x,y)=if a==3{(500+dx*2-4,500+dy*2-2)}else{(0,0)};let hp=if a==2{20}else{30};
   for n in [bridge,dx,dy,i32::from(a),x,y,hp]{h=(h^n as u32).wrapping_mul(16777619);}
  }}}
  assert_eq!(h,2894185418);
 }
 #[test]fn all_624_knockdown_and_recovery_steps_match_the_native_state_handlers(){
  let mut h=2166136261u32;
  for facing in 0..8{for hp in [11,20,100]{
   let mut s=State{phase:0,frame:0,side:(facing+2)&4!=0,path:VecDeque::new(),dest:None};
   for tick in 1..=28{
    let release=s.advance(tick<4,tick&1==0);let native=if s.phase==0{if s.side{46}else{23}}else if s.side{47}else{24};
    for n in [facing,hp,tick,native,i32::from(s.frame),if s.phase==0{-32}else{0},hp-10,i32::from(release)]{h=(h^n as u32).wrapping_mul(16777619);}
    if release{break;}
   }
  }}
  assert_eq!(h,1837382229);
 }
 fn field()->Game{
  let mut g=Game::new(0,1,712);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
  g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;g.next_id=1;g
 }
 #[test]fn a_robot_waits_under_the_hull_then_stands_up_and_weak_robots_are_run_over(){
  for hp in [10.,30.]{
   let mut g=field();g.spawn("medium",0,1000.,1000.,1,None);g.spawn("grunt",1,1032.,1000.,2,None);let robot=g.units[1].id;
   g.units[1].hp=hp;for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}
   g.step(0.059);assert!(g.units[1].native_contact.is_none());g.step(0.059);
   if hp==10.{assert!(g.units.iter().all(|u|u.id!=robot||u.hp<0.));continue;}
   assert_eq!(g.units[1].hp,20.);assert_eq!(g.units[1].native_contact.as_ref().unwrap().phase,0);
   for _ in 0..30{g.step(0.059);}assert_eq!(g.units[1].hp,20.);assert_eq!(g.units[1].native_contact.as_ref().unwrap().phase,0);
   g.units[0].x=1300.;g.step(0.059);assert_eq!(g.units[1].native_contact.as_ref().unwrap().phase,1);
   for _ in 0..23{g.step(0.059);}assert!(g.units[1].native_contact.is_none());assert_eq!(g.units[1].hp,20.);
  }
 }
 #[test]fn evasive_contact_restores_the_existing_march_and_respects_faction_and_fort_protection(){
  assert_eq!(action(0,0,false,true,30),3);
  for team in [0,1]{let mut g=field();g.spawn("medium",0,1000.,1000.,1,None);g.spawn("grunt",team,1040.,1000.,2,None);g.spawn("gatling",1,1800.,1200.,3,None);
   for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}let id=g.units[1].id;
   g.command(&crate::json!({"team":team,"kind":"move","ids":[id],"x":1400.,"y":1000.}));g.step(0.059);g.step(0.059);
   assert_eq!(g.units[1].native_contact.is_some(),team==1);
   for _ in 0..80{g.step(0.059);}assert!(g.units[1].native_contact.is_none());assert_eq!(g.units[1].dest,Some((1400.,1000.)));assert!(g.units[1].x>1064.);
  }
 }
 #[test]fn fort_raw_base_is_native_pixels_and_protects_against_crushing(){
  let mut fort=Game::new(0,1,2).buildings.into_iter().find(|b|b.kind=="fort").unwrap();fort.raw_base=Some((500.,500.));fort.team=0;
  let mut g=field();g.buildings.push(fort);g.spawn("medium",0,1000.,1000.,1,None);g.spawn("grunt",1,1032.,1000.,2,None);for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}let hp=g.units[1].hp;
  g.step(0.059);g.step(0.059);assert_eq!(g.units[1].hp,hp);assert_ne!(g.units[1].native_contact.as_ref().map(|s|s.phase),Some(0));
 }
}
