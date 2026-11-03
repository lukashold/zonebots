//! Normal native robot idle:0044cac0/0044dd00 initialization and the local
//! animation/search section of0044d920. World object/enemy/follower queries are
//! explicit actions, so callers do not invent a free-running visual loop.
use serde::{Serialize,Deserialize};
#[path="original_reset.rs"]pub(crate) mod reset;

///004959bc: after a completed look, an empty scan requests a four-pixel
/// native step. These are native anchor deltas, not browser pixels.
pub const SEARCH_STEP:[(i32,i32);8]=[(0,-4),(4,-4),(4,0),(4,4),(0,4),(-4,4),(-4,0),(-4,-4)];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action{
 Wait,
 /// Not arrived: object scan0044d3a0, then enemy scan00458500, then follow.
 /// Arrived: object scan0044d3a0, 100-record target scan0044d140, then SEARCH_STEP.
 Search{arrived:bool},
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Tick{pub action:Action,pub clear_aim:bool}
/// Raw state1 preamble inputs. Native group offsets3d/3e/3f are kept explicit.
///00458620 synchronizes a leader's group order; it does not refresh targets.
#[derive(Clone,Copy,Debug)]
pub struct Preamble{
 pub role:i8,pub flags31:u8,pub flags64:u8,pub aim_skill:i8,
 pub valid_group:bool,pub group_target:Option<u8>,
 pub group_3d:i8,pub group_3e:i8,pub group_3f:i8,
 /// Native unit+50 slot, used only when flags64&20 and !(flags31&8).
 pub marked_unit:i16,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Gate{
 Wait,
 Animate,
 ///0044a540, with global004d029e=1 for this invocation.
 ReactGroup,
 ///0044ca50 with the group's raw target slot. Only role-1 actually moves:
 /// it withdraws by half the vector away from that threat, then clears3b.
 WithdrawFromThreat{target:u8},
 ///00459c60(unitOrigin+8,+8), unit8c=4, then00452810.
 FollowMarkedUnit{target:i16},
 ///0044d570. Its route/follower query remains an external boundary.
 FollowGroup,
}
pub fn requires_order_sync(p:&Preamble)->bool{p.role<=0&&p.flags64&0x20==0&&p.valid_group}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct NativeOrder{pub command:i8,pub field50:i32,pub field54:i16,pub field56:i16}
///00458620's raw field copy, with no route/attack semantics inferred.
pub fn sync_group_order(role:i8,current_command:i8,aim_locked:bool,group_command:i8,fields:[i16;3])->Option<NativeOrder>{
 if group_command!=0&&role==-1&&(current_command==12||!aim_locked){Some(NativeOrder{command:group_command,field50:i32::from(fields[0]),field54:fields[1],field56:fields[2]})}else{None}
}
///0044ca50 uses signed arithmetic shifts, so negative odd deltas round down.
pub fn withdraw_goal(role:i8,position:(i16,i16),threat:(i16,i16))->Option<(i16,i16)>{
 if role!=-1{return None;}
 Some(((i32::from(position.0)+((i32::from(position.0)-i32::from(threat.0))>>1))as i16,(i32::from(position.1)+((i32::from(position.1)-i32::from(threat.1))>>1))as i16))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GroupReactionTarget{Clear,Target(u8),None}
///44a540 chooses a live roster ROLE by ordinal, preserving holes. An APC
///or independent target bypasses this random choice. Invalid group counts
///are a caller boundary; the original IDIV would trap for zero.
pub fn group_reaction_target(target:u8,family:u8,role:i8,active:bool,roster:&[u8;21],count:i8,mut random:impl FnMut()->u32)->GroupReactionTarget{
 if family==32||role==0{return if active{GroupReactionTarget::Target(target)}else{GroupReactionTarget::Clear};}
 if count<=0{return GroupReactionTarget::None;}
 let ordinal=(random()&65535)as usize%count as usize;
 roster.iter().copied().filter(|&slot|slot!=255).nth(ordinal).map(GroupReactionTarget::Target).unwrap_or(GroupReactionTarget::None)
}
pub fn preamble(p:&Preamble,mut random:impl FnMut()->u32)->Gate{
 let can_attack=p.flags64&0x10!=0;
 if p.role<=0{
  if p.flags64&0x20!=0{
   return if p.flags31&8==0{Gate::FollowMarkedUnit{target:p.marked_unit}}else{Gate::Animate};
  }
  if !p.valid_group{return Gate::Wait;}
  if let Some(target)=p.group_target{
   if !can_attack{return Gate::WithdrawFromThreat{target};}
   if p.group_3e==0&&((random()&127)as i8)<p.aim_skill{return Gate::ReactGroup;}
  }
 }else{
  if p.group_3d!=0{return Gate::FollowGroup;}
  if p.group_target.is_some()&&(p.group_3f!=0||can_attack)&&((random()&127)as i8)<p.aim_skill{return Gate::ReactGroup;}
 }
 Gate::Animate
}
#[derive(Clone,Copy,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct State{
 pub frame:u16,pub direction:u8,pub desired_direction:u8,pub turn_step:i8,
 pub timer:i8,pub look_counter:i8,pub search_timer:u8,
 /// Unit+3e initially copies the robot's original aim. It is independent
 /// from the current damaged aim byte passed to advance().
 pub look_chance:i8,
}
impl State{
 /// Two original RNG calls, in their original order.0044cac0 does this for
 /// classes0/3/15/16; class1 vehicles and class2 fixed guns do not initialize it.
 pub fn enter(direction:u8,look_chance:i8,mut random:impl FnMut()->u32)->Self{
  let timer=(random()&3)as i8;let search_timer=((random()&0xffff)%53+12)as u8;
  Self{frame:1,direction:direction&7,desired_direction:direction&7,turn_step:0,timer,look_counter:8,search_timer,look_chance}
 }
 pub fn sprite(&self)->usize{184+usize::from(self.direction)*3+usize::from(self.frame)}
 /// Must be called once per native59ms callback, after the handler's external
 /// combat/leader preamble has allowed idle processing. A turn only advances
 /// when global00622dc0 is nonzero. No RNG is consumed while waiting to turn.
 pub fn advance(&mut self,turn_pulse:bool,aim_skill:i8,mut random:impl FnMut()->u32)->Tick{
  let mut clear_aim=false;
  if self.direction==self.desired_direction{
   self.timer=self.timer.wrapping_sub(1);if self.timer>0{return Tick{action:Action::Wait,clear_aim};}
   self.look_counter=self.look_counter.wrapping_sub(1);self.timer=7;
   if self.look_counter<0{
    clear_aim=true;self.look_counter=7;
    if (random()&127)as i8<=self.look_chance{
     self.desired_direction=if ((random()&127)as i8)<aim_skill||random()&1==0{self.direction.wrapping_sub(3)&7}else{(random()&7)as u8};
     self.turn_step=((random()&2)as i8)-1;
    }
   }
   let r=random()as u16;self.frame=((r&0x101)>>8)+(r&1);
  }else{
   if !turn_pulse{return Tick{action:Action::Wait,clear_aim};}
   self.direction=self.direction.wrapping_add(self.turn_step as u8)&7;self.frame=0;
   if self.direction==self.desired_direction{return Tick{action:Action::Search{arrived:true},clear_aim};}
  }
  Tick{action:Action::Search{arrived:false},clear_aim}
 }
}

///004616f0 is called when hardware changes anchor, not on a visual wall clock.
/// Its track/wheel frame counter does not consume deterministic gameplay RNG;
/// native dust/sound effects separately use CRT rand(). frozen31 and frozen71
/// are the native flag31&40 and flag71&1 early exits.
pub fn hardware_moved_frame(kind:&str,direction:u8,frame:i16,frozen31:bool,frozen71:bool)->i16{
 if frozen31||frozen71{return frame;}
 if matches!(kind,"jeep"|"truck"|"firetruck"){return frame.wrapping_add(1)&3;}
 if matches!(kind,"apc"|"crane"|"launcher")||!matches!(direction&7,0|1|2|7){let next=frame.wrapping_add(1);if next==3{0}else{next}}
 else{let next=frame.wrapping_sub(1);if next<0{2}else{next}}
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,ns:&[i32]){for n in ns{*h=(*h^*n as u32).wrapping_mul(16777619);}}
 #[test]fn all_256_group_reaction_targets_match_executed_native_sparse_rosters(){
  let mut h=2166136261;
  for s in 0..256{let roster=std::array::from_fn(|r|if(r+s as usize)%4==0{255}else{r as u8+9});let count=roster.iter().filter(|&&id|id!=255).count()as i8;let mut calls=0;
   let selected=group_reaction_target(7,if s&1!=0{32}else{4},if s&2!=0{0}else{1},s&4!=0,&roster,count,||{calls+=1;(s*2027^0x7731a5c9)as u32});
   let(target,group)=match selected{GroupReactionTarget::Target(id)=>(i32::from(id),7),GroupReactionTarget::Clear=>(-1,255),GroupReactionTarget::None=>(-1,7)};
   hash(&mut h,&[s,target,calls,group]);
  }
  assert_eq!(h,3791172574,"ZNativeIdleTargetOracle");
 }
 #[test]fn all_7344_normal_idle_callbacks_and_rng_states_match_native_execution(){
  let mut h=2166136261;let mut rows=0;
  for aim in [0i8,40,60,120]{for chance in [0i8,60,127]{for direction in [0,3,7]{for multi in 0..2{
   let mut rng=crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:multi!=0};let mut s=State::enter(direction,chance,||rng.next_u32());let mut flags=128u8;
   for tick in 0..=240{
    let action=if tick==0{Action::Wait}else{let a=s.advance(tick&1!=0,aim,||rng.next_u32());if a.clear_aim{flags&=127;}a.action};
    let(search,wx,wy)=match action{Action::Wait=>(0,0,0),Action::Search{arrived:false}=>(13,0,0),Action::Search{arrived:true}=>{let p=SEARCH_STEP[usize::from(s.desired_direction)];(19,500+p.0,600+p.1)}};
    hash(&mut h,&[i32::from(aim),i32::from(chance),i32::from(direction),multi,tick,1,i32::from(s.frame),i32::from(s.direction),i32::from(s.timer),i32::from(s.look_counter),i32::from(s.desired_direction),i32::from(s.turn_step),i32::from(s.search_timer),i32::from(flags),search,wx,wy]);for word in rng.words{hash(&mut h,&[word as i32]);}rows+=1;if search&16!=0{break;}
   }
  }}}}
  assert_eq!(rows,7344);assert_eq!(h,2021575908);
 }
 #[test]fn a_turn_waits_for_the_shared_pulse_and_search_steps_are_native_anchor_deltas(){
  let mut rng=||0u32;let mut s=State::enter(0,60,&mut rng);s.desired_direction=5;s.turn_step=-1;
  assert_eq!(s.advance(false,60,||panic!("turn wait must not consume RNG")).action,Action::Wait);assert_eq!(s.direction,0);
  assert_eq!(s.advance(true,60,||panic!("turn must not consume RNG")).action,Action::Search{arrived:false});assert_eq!(s.direction,7);assert_eq!(s.sprite(),205);assert_eq!(SEARCH_STEP[5],(-4,4));
 }
 #[test]fn all_4608_hardware_movement_callbacks_match_native_track_and_wheel_counters(){
  let mut h=2166136261;
  for(family,kind)in [(6,"medium"),(25,"heavy"),(30,"light"),(24,"launcher"),(32,"apc"),(34,"jeep"),(36,"crane"),(38,"truck")]{for direction in 0..8{for start in 0..4{for freeze in 0..3{
   let mut frame=start;for tick in 1..=6{frame=hardware_moved_frame(kind,direction,frame,freeze==1,freeze==2);hash(&mut h,&[family,i32::from(direction),i32::from(start),freeze,tick,i32::from(frame)]);}
  }}}}
  assert_eq!(h,4177501061);
 }
 #[test]fn all_3456_preamble_gates_and_rng_call_counts_match_native_execution(){
  let mut h=2166136261;let mut rows=0;
  for role in [-1,0,1]{for flags in [0,16,32,48]{for attached in [0,8]{for target in [255,7]{for suppressed in 0..2{for hold in 0..2{for force in 0..2{for aim in [0,60,120]{for roll in [0u32,60,127]{
   let p=Preamble{role,flags31:attached,flags64:flags,aim_skill:aim,valid_group:true,group_target:(target!=255).then_some(target),group_3d:hold,group_3e:suppressed,group_3f:force,marked_unit:7};
   let mut calls=0;let gate=preamble(&p,||{calls+=1;roll});let(code,wx,wy)=match gate{Gate::Wait=>(0,0,0),Gate::Animate=>(1,0,0),Gate::ReactGroup=>(2,0,0),Gate::WithdrawFromThreat{..}=>(3,0,0),Gate::FollowMarkedUnit{..}=>(4,508,608),Gate::FollowGroup=>(5,0,0)};
   hash(&mut h,&[i32::from(role),i32::from(flags),i32::from(attached),i32::from(target),i32::from(suppressed),i32::from(hold),i32::from(force),i32::from(aim),roll as i32,code,calls,i32::from(requires_order_sync(&p)),wx,wy]);rows+=1;
  }}}}}}}}}
  assert_eq!(rows,3456);assert_eq!(h,2798926901);
 }
 #[test]fn all_108_group_order_syncs_match_native_raw_field_copies(){
  let mut h=2166136261;
  for role in [-1,0,1]{for command in [0,12]{for locked in [0,128]{for group in [0,1,12]{for coordinate in [-1,0,32767]{
   let actual=sync_group_order(role,command,locked!=0,group,[coordinate,123,456]).unwrap_or(NativeOrder{command,field50:88,field54:99,field56:111});
   hash(&mut h,&[i32::from(role),i32::from(command),locked,i32::from(group),i32::from(coordinate),i32::from(actual.command),actual.field50,i32::from(actual.field54),i32::from(actual.field56)]);
  }}}}}
  assert_eq!(h,2076192394);
 }
 #[test]fn all_36_withdrawal_boundaries_match_native_signed_shifts_and_coordinate_wrapping(){
  let mut h=2166136261;
  for role in [-1,0,1]{for p in [(500,600),(1,1),(-1,-1),(32760,-32760)]{for t in [(600,700),(499,599),(-120,90)]{
   let goal=withdraw_goal(role,p,t);let(gx,gy)=goal.unwrap_or((0,0));hash(&mut h,&[i32::from(role),i32::from(p.0),i32::from(p.1),i32::from(t.0),i32::from(t.1),i32::from(goal.is_some()),i32::from(gx),i32::from(gy),if goal.is_some(){255}else{7},if goal.is_some(){4}else{0}]);
  }}}
  assert_eq!(h,2116145379);
 }
}
