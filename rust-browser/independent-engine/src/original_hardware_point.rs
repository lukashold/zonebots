//! Native69:20 hardware point-fire controls; source callbacks remain distinct
//! from the finite projectile pool and movement contact worker.
use crate::Game;
use crate::native_weapon_data as data;
#[path="original_hardware_target.rs"]pub(crate) mod target;
#[path="original_hardware_target_game.rs"]mod target_game;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Shot{pub origin:[i16;2],pub target:[i16;2],pub rocket:bool,pub sound:i8,pub speed:i8}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct State{pub family:u8,pub flags69:u8,pub reload7b:i8,pub timer3d:u8,pub position:[i16;2],pub point:[i16;2],pub body:u8,pub turret:u8,pub rate:u8,pub speed:i8,pub w22:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{Absent,Waiting,AimTarget,FireApc,Fire}
pub fn select(s:&mut State)->Action{if s.flags69&32==0{return Action::Absent;}s.w22=102;s.timer3d=4;if s.flags69&128!=0{return Action::AimTarget;}if s.reload7b!=0{return Action::Waiting;}if s.family==32{return Action::FireApc;}s.turret=crate::native_math::direction(i32::from(s.point[0])-i32::from(s.position[0]),i32::from(s.point[1])-i32::from(s.position[1]));s.reload7b=100u8.wrapping_sub(s.rate)as i8;Action::Fire}
fn add(p:[i16;2],d:[i32;2])->[i16;2]{[p[0].wrapping_add(d[0]as i16),p[1].wrapping_add(d[1]as i16)]}
const TRUCK_BODY:[[i32;2];8]=[[0,4],[-1,3],[-3,4],[0,1],[1,0],[4,1],[6,4],[4,4]];
const TRUCK_EXTRA:[[i32;2];8]=[[-3,-19],[7,-16],[12,-11],[7,-5],[1,-3],[-7,-5],[-12,-9],[-10,-15]];
///5b4f0 +5b730 or forced5a770. The latter applies each barrel's X
///offset to BOTH target axes, and skips all449470 scatter calls.
pub fn shots(s:State)->Vec<Shot>{let body=usize::from(s.body&7);let aim=usize::from(s.turret&7);if s.family==24{let origin=add(add(s.position,data::LAUNCHER_BODY[body]),data::LAUNCHER_A[aim]);let mut out=vec![Shot{origin,target:s.point,rocket:true,sound:32,speed:s.speed}];for d in[data::LAUNCHER_B[aim],data::LAUNCHER_C[aim]]{out.push(Shot{origin:add(origin,d),target:add(s.point,[d[0],d[0]]),rocket:true,sound:-1,speed:s.speed});}return out;}let origin=if s.family==38{add(add(s.position,TRUCK_BODY[body]),TRUCK_EXTRA[aim])}else if s.family==34{add(add(s.position,data::JEEP_BODY[body]),data::JEEP_MUZZLE[aim])}else{let mut p=add(add(s.position,data::GUN_MUZZLE[aim]),data::TANK_TURRET[aim]);p=add(p,[6,-9]);match s.family{6=>add(p,data::MEDIUM_EXTRA[aim]),25=>add(p,data::HEAVY_EXTRA[aim]),_=>p}};vec![Shot{origin,target:s.point,rocket:false,sound:match s.family{6=>22,25=>18,30=>21,_=> -1},speed:s.speed}]}
///45a970 after a real vehicle movement/contact attempt.525a0 initializes
///A2/A3 toFF/0; a blocked facing is held for exactly six source callbacks.
pub fn restore_contact_facing(a2:&mut i8,a3:&mut i8,facing:&mut u8,changed:bool){if changed{*a3=6;return;}if *a3!=0{*a3=a3.wrapping_sub(1);if *a3==0{if *a2!= -1{*facing=*a2 as u8;}*a2= -1;}}}
#[path="original_hardware_point_game.rs"]mod game;
impl Game{
 pub(crate) fn native_hardware_ready(&self,u:&crate::Unit)->bool{if self.map.original&&u.native_slot.is_some(){u.native_reload7b==0}else{u.cooldown<=0.}}
 pub(crate) fn set_native_hardware_reload(&mut self,index:usize,value:u8){self.units[index].native_reload7b=value as i8;self.units[index].cooldown=f64::from(value)*0.059;}
 ///45a080's69:20 tail precedes55f10. `original_slot` is captured before
 ///the callback, so a transport identity change still resolves its raw UNIT.
 pub(crate) fn finish_native_unit_callback(&mut self,index:usize,original_slot:Option<usize>){
  let Some(slot)=original_slot else{return;};let index=self.units.iter().rposition(|u|u.native_slot==Some(slot)).unwrap_or(index);if self.native_group_rosters.members.get(slot).and_then(Option::as_ref).is_some_and(|m|m.flags69&32!=0)&&self.units.get(index).is_some_and(|u|u.native_slot==Some(slot)&&u.native_class7c==1&&u.native_common_callback!=Some(self.motion_tick)){if let Err(boundary)=self.update_native_hardware_point(index){self.native_route_boundary=Some(boundary);}}
  let Some(member)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut)else{return;};if member.flags69&32==0{return;}member.flags69&=!32;self.sync_native_group_order(index);
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,n:i64){*h=(*h^n as u32).wrapping_mul(16777619);}
 #[test]fn raw_contact_direction_hold_matches2048_native_callbacks(){let mut hash=2166136261;for s in 0..2048usize{let mut a2=[-1,0,7,127,-128,-1][s%6];let mut a3=[-128,-1,0,1,6,127][s/6%6];let mut facing=(s*19)as u8;restore_contact_facing(&mut a2,&mut a3,&mut facing,s/36%2!=0);for n in[s as i64,i64::from(a2),i64::from(a3),i64::from(facing)]{mix(&mut hash,n);}}assert_eq!(hash,2347042240,"full45a970, no stubs");}
 #[test]fn point_gates_geometry_reload_and_restore_match8192_executed_original_callbacks(){let mut hash=2166136261;let delta=[[0,-70],[70,-70],[70,0],[70,70],[0,70],[-70,70],[-70,0],[-70,-70]];for s in 0..8192usize{let d=delta[s/80%8];let mut state=State{family:[6,25,30,24,32,34,36,38][s%8],flags69:32+(s/8%2*128)as u8,reload7b:[0,1,127,-128,-1][s/16%5],timer3d:7,position:[500,600],point:[500+d[0],600+d[1]],body:(s/640%8)as u8,turret:3,rate:(s*17)as u8,speed:(s*13)as i8,w22:37};let action=select(&mut state);let(mut sh,mut sync,mut aim,mut apc)=(2166136261,0,0,0);match action{Action::Fire=>{for shot in shots(state){for n in[i64::from(shot.rocket),i64::from(shot.origin[0]),i64::from(shot.origin[1]),i64::from(shot.target[0]),i64::from(shot.target[1]),i64::from(shot.speed),i64::from(shot.sound),i64::from(s%3==0)]{mix(&mut sh,n);}}sync=2;},Action::FireApc=>{state.reload7b=(s*13)as i8;apc=1;sync=1;},Action::AimTarget=>aim=1,Action::Waiting=>sync=1,Action::Absent=>{}}for n in[s as i64,i64::from(state.reload7b),i64::from(state.timer3d),i64::from(state.turret),i64::from(state.w22),111,222,sync,aim,apc,i64::from(sh)]{mix(&mut hash,n);}}assert_eq!(hash,1208336219,"actual45c5c0/5b4f0/5b730/5b7b0/forced5a770; unrelated prelude/target/APC/allocator calls are explicit recorded boundaries");}
}
