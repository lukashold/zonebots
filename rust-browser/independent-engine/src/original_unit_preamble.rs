//! 45a080 before 44f1e0: frame bounds, collision-table selection and the
//! enabled single-player 459eb0 idle-pose initializer. Raw tokens name the
//! original tables; they are never dereferenced as host pointers.
#[path="original_unit_preamble_game.rs"] pub mod game;

pub const FRAME_LIMITS:[u8;128]=[1,3,4,3,10,5,4,4,4,255,255,0,0,0,0,3,5,5,0,3,5,3,1,5,20,50,50,50,50,50,50,50,50,50,50,50,50,10,50,50,50,5,255,30,30,5,5,20,2,4,1,12,12,13,15,15,15,2,2,10,10,5,0,10,6,3,4,255,255,255,16,4,10,6,6,0,249,255,250,255,251,255,251,255,0,0,253,255,253,255,255,255,250,255,254,255,246,255,255,255,244,255,253,255,248,255,251,255,11,0,250,255,11,0,250,255,11,0,250,255,11,0,250,255,11,0,250,255];

/// The phase is a signed byte in the executable. Negative phases require
/// bytes before 496d70 and are deliberately not interpreted as valid states.
pub fn clamp_frame(phase:u8,frame:&mut i16)->bool{
 let Some(&maximum)=FRAME_LIMITS.get(usize::from(phase))else{return false;};
 if *frame>i16::from(maximum){*frame=0;}true
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct CollisionTables{pub footprint:u32,pub permission:u32}
/// 41b950's constructor and class3 callback selector. APC saved class is an
/// explicit input; a missing saved value cannot select a made-up footprint.
pub fn collision_tables(class:u8,family:u8,saved_class:Option<u8>,flags31:u8,role:i8)->Option<CollisionTables>{
 let t=usize::from(flags31&3);
 let ordinary=[0x6471a0,0x644990,0x499888,0x499888];
 let tough=[0x63a9d0,0x6499c0,0x49a9a0,0x49a9a0];
 let heavy=[0x6381f0,0x6421b0,0x49a840,0x49a840];
 let light=[0x63f990,0x64c1e0,0x49a8a0,0x49a8a0];
 let crane=[0x64c258,0x63d1b0,0x49a930,0x49a930];
 let foot_permission=[0x632698,0x632458,0x499830,0x499830];
 let vehicle_permission=[0x632770,0x632890,0x49a788,0x49a788];
 let crane_permission=[0x632650,0x6327b8,0x49a7e0,0x49a7e0];
 if class!=1{return Some(CollisionTables{footprint:if class==3&&role<1{tough[t]}else{ordinary[t]},permission:foot_permission[t]});}
 Some(CollisionTables{footprint:match family{32=>if saved_class?==3{heavy[t]}else{light[t]},34=>light[t],36=>crane[t],_=>heavy[t]},permission:if family==36{crane_permission[t]}else{vehicle_permission[t]}})
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Pose{
 pub phase:u8,pub class:u8,pub flags8a:u8,pub role:i8,pub group:u8,
 pub delay:u8,pub cycles:u16,pub timer:u8,pub frame:i16,pub counter82:i16,
 pub heading:u8,pub transition:u16,
}
pub fn pose_terrain_allowed(code:u8)->bool{!matches!(code&127,9|38|39|42)}
/// Exact 459eb0. Its 459460 call always returns AL=1 here: this entry has
/// already required phase1. `prevented` is the lazy 458590 query used only
/// by descriptor3/case1 after its terrain gate.
pub fn idle_pose(p:&mut Pose,multiplayer:bool,quarter_clock:i16,descriptor:u8,terrain:u8,mut random:impl FnMut()->u32,mut prevented:impl FnMut()->bool){
 if multiplayer||p.phase!=1||!matches!(p.class,0|3|15|16|17|18)||p.flags8a&48!=0||p.delay==0||quarter_clock!=0{return;}
 p.delay=p.delay.wrapping_sub(1);if p.delay!=0||p.role== -1{return;}
 p.cycles=((random()as u8&5)|2)as u16;
 if p.role==0{return;}
 let _check=random()&3; // 59460 is a proven no-op at phase1, but draw remains.
 if p.group==255||descriptor==1{return;}
 let variant=random()&3;
 if !pose_terrain_allowed(terrain){return;}
 match descriptor{
  2=>if variant&1==0{p.transition=0x422;p.phase=42;p.frame=0;p.cycles&=255;}else{p.transition=0x426;p.phase=42;p.frame=0;p.counter82=0;p.cycles&=255;p.timer=((random()as u16)%51)as u8+100;},
  3=>match variant{
   0|3=>{p.phase=31;p.frame=0;p.counter82=0;p.cycles=((random()as u8&4)|3)as u16;p.timer=20;p.heading&=6;p.frame=i16::from(p.heading as i8)+i16::from((p.heading as i8)>>1);},
   1=>if !prevented(){p.counter82=-1;p.phase=43;let h=p.heading&14;p.frame=i16::from(h as i8)+i16::from((h as i8)>>1);p.transition=(p.transition&255)|0x400;p.cycles=3;p.timer=4;},
   _=>{p.transition=if(random()as i8)<0{0x41e}else{0x41c};p.phase=42;p.frame=0;p.counter82=0;p.cycles=((random()as u16)%3+5)as u16;p.timer=(random()as u8&15)+2;}
  },
  4=>{p.transition=0x41b;p.phase=42;p.frame=0;p.counter82=0;p.cycles=((random()as u16)%3+5)as u16;p.timer=0;},
  _=>if variant<2{p.transition=0x42d;p.phase=42;p.frame=0;p.cycles=7;p.timer=3;}else{p.transition=0x41d;p.phase=42;p.frame=0;p.counter82=0;p.cycles=((random()as u16)%3+5)as u16;p.timer=(random()as u8&7)+2;}
 }
}

/// 59e60 tests each live roster entry for 8a:20, but passes the original
/// actor to 58ec0 every time. The roster lookup is intentionally lazy: actor
/// removal/promotion can change the subsequent entries in the same group.
pub fn remove_transport_actor(actor:u8,mut roster_member:impl FnMut(usize)->Option<(u8,u8)>,mut remove:impl FnMut(u8)){
 for role in 0..21{if let Some((_,flags8a))=roster_member(role){if flags8a&32!=0{remove(actor);}}}
}

pub trait PoseEnvironment{fn random(&mut self)->u32;fn prevented(&mut self)->bool;fn target_direction(&mut self,relative:u8)->Option<u8>;}
/// The handlers reached by459eb0, including the31→42→41 response. True
/// requests44cac0 after the raw writes; the reset's RNG belongs to its owner.
pub fn tick_pose(p:&mut Pose,callback_mode:u8,terrain:u8,env:&mut impl PoseEnvironment)->Option<bool>{
 if !matches!(p.phase,27|28|29|30|31|34|38|41|43|45){return None;}
 if callback_mode!=0{return Some(false);}
 if p.phase==31&&(p.timer as i8)<=10{let c=p.counter82 as u8;let relative=if c==3{1}else{c}.wrapping_sub(1);if let Some(direction)=env.target_direction(relative){
  if pose_terrain_allowed(terrain)&&!matches!(p.class,1|2){let d=direction&7;p.counter82=i16::from(d);p.transition=0x29|(u16::from([6u8,3,4,5,6,3,4,5][usize::from(d)])<<8);p.frame=0;p.cycles&=255;p.timer=20;p.phase=42;}return Some(false);
 }}
 if env.prevented(){return Some(true);}
 let low_cycles=|p:&mut Pose|{let n=p.cycles as u8&7;if n==0{false}else{p.cycles=(p.cycles&!7)|u16::from(n-1);true}};
 let reset=match p.phase{
  34=>{p.frame=p.frame.wrapping_sub(1)&1;if p.frame==0{if !low_cycles(p){true}else{p.frame=0;false}}else{false}},
  38=>if p.frame!=3{p.frame=p.frame.wrapping_add(1);false}else if p.timer!=0{p.timer=p.timer.wrapping_sub(1);false}else{true},
  28|30=>{if matches!(p.frame,0|1){p.timer=p.timer.wrapping_sub(1);if p.timer!=0{return Some(false);}p.timer=(env.random()as u8&15)+10;}p.frame=p.frame.wrapping_add(1);if p.frame>=5{if !low_cycles(p){true}else{p.frame=0;p.timer=10;false}}else{false}},
  27=>{if p.timer==0{p.frame=p.frame.wrapping_add(1);if p.frame!=9{return Some(false);}p.timer=(env.random()as u8&15)+20;}p.timer=p.timer.wrapping_sub(1);if p.timer==0{if !low_cycles(p){true}else{p.frame=6;false}}else{false}},
  45=>{p.frame=p.frame.wrapping_sub(1)&1;env.random()as u8<=16},
  29=>if p.frame!=3{p.frame=p.frame.wrapping_add(1);false}else{p.timer=p.timer.wrapping_sub(1);if p.timer==0{p.timer=(env.random()as u8&15)+10;if !low_cycles(p){true}else{p.frame=2;false}}else{false}},
  31=>if p.timer!=0{p.timer=p.timer.wrapping_sub(1);false}else{p.heading&=6;let base=(p.heading as i8).wrapping_add((p.heading as i8)>>1);p.counter82=p.counter82.wrapping_add(1)&3;if matches!(p.counter82,0|2){p.timer=20;}p.frame=i16::from(base.wrapping_add(if p.counter82==3{1}else{p.counter82 as i8}));p.counter82==0&&!low_cycles(p)},
  41=>{p.timer=p.timer.wrapping_sub(1);p.frame=p.counter82;p.timer==0},
  43=>{let h=i16::from(p.heading&14);if p.frame!=h+(h>>1)+2{p.frame=p.frame.wrapping_add(1);false}else if p.timer!=0{p.timer=p.timer.wrapping_sub(1);false}else{true}},
  _=>unreachable!(),
 };Some(reset)
}

#[cfg(test)]mod tests{
 use super::*;use crate::{original_combat::OriginalRng,original_group_lifecycle as group};
 fn hash(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn prefix_before_pattern_matches_1024_native_callbacks_with_explicit_mutable_leaves(){
  let mut h=2166136261;
  for s in 0..1024usize{
   let role=[-1,0,1][s%3];let mut phase=[0,1,5,8,19,61,42,77][s/3%8];let class=[0,3,15][s/24%3];let flags31=(s%4)as u8;
   let mut flags8a=(s*29)as u8;let mut flags9c=(s*37)as u8;let mut aa=[0u8,1,2,255][s/7%4];let mut frame=[-32768,-1,0,1,4,5,10,255,256,32767][s%10];let mut counter=[0,1,254,255][s/3%4];let flags69=(s*17)as u8;
   let mut ptr18=0x12345678;let mut ptr1c=0x87654321;let mut context=1;let mut seen=0;let mut trace=2166136261u32;
   let mut groups=std::array::from_fn::<_,3,_>(|g|{let mut row=group::GroupSlot::empty();row.roster[9]=(s*(g+3))as u8;row.roster[63]=(s/3%4)as u8;row.roster[64]=[0,1,127,128,255][s/9%5];row.roster[59]=if s%7==0{255}else{17};row.roster[16]=if s%4==0{0}else{7};row});
   if phase==61&&s%5==0{phase=0;flags8a&=!24;hash(&mut trace,[1,2]);}
   if class==3{let tables=collision_tables(class,4,None,flags31,role).unwrap();ptr18=tables.footprint;ptr1c=tables.permission;}
   assert!(clamp_frame(phase,&mut frame));
   if aa!=0{aa-=1;flags9c&=!20;seen=1;groups[1].roster[9]&=!32;}
   if role<0{counter=group::advance_leader_counter(counter);if group::begin_leader_preamble(&mut groups[1]){hash(&mut trace,[3]);context=2;if s%2==0{phase=5;}}group::finish_leader_preamble(&mut groups[context],phase);}
   hash(&mut h,[s as u32,u32::from(flags69&64==0),context as u32,seen,trace,u32::from(phase),frame as u32,u32::from(flags8a),u32::from(flags9c),u32::from(aa),u32::from(counter),ptr18,ptr1c]);
   for g in groups{hash(&mut h,[9,63,64,59].map(|off|u32::from(g.roster[off])));}
  }
  assert_eq!(h,2625170038,"ZNov04PreambleOracle: real5a080,41b950; recorded cleanup/group-expiry leaves");
 }
 #[test]fn enabled_idle_pose_matches_4096_native_initializations_and_rng_states(){
  let mut h=2166136261;
  for s in 0..4096usize{
   let mut p=Pose{phase:if s%37==0{2}else{1},class:[0,3,15,16,17,18,1,2,4][s/17%9],flags8a:if s%19==0{32}else{0},role:[-1,0,1,7][s/11%4],group:if s%23==0{255}else{0},delay:if s%13==0{2}else{1},cycles:0x9321,timer:55,frame:-12,counter82:1234,heading:(s%8)as u8,transition:0x7134};
   let mut rng=OriginalRng{words:std::array::from_fn(|i|(s*(i+1)+i+1)as u32),multiplayer:s%41==0};let mut queries=0;
   idle_pose(&mut p,s%41==0,if s%43==0{2}else{0},(s%6)as u8,[0,9,38,39,42,128,16,35][s/31%8],||rng.next_u32(),||{queries+=1;s%5==0});
   hash(&mut h,[s as u32,queries,u32::from(p.phase),p.frame as u32,u32::from(p.heading),u32::from(p.delay),u32::from(p.cycles),u32::from(p.timer),p.counter82 as u32,u32::from(p.transition)]);hash(&mut h,rng.words);
  }
  assert_eq!(h,2040364358,"ZNov04IdlePoseOracle: real459eb0/all reachable initializers/RNG;58590 boolean input");
 }
 struct Fixture{scenario:usize,queries:u32,scans:u32,relative:i32,rng:OriginalRng}
 impl PoseEnvironment for Fixture{
  fn random(&mut self)->u32{self.rng.next_u32()}
  fn prevented(&mut self)->bool{self.queries+=1;self.scenario%7==0}
  fn target_direction(&mut self,relative:u8)->Option<u8>{self.scans+=1;self.relative=i32::from(relative);(self.scenario%5==0).then(||crate::native_math::direction((self.scenario as i32%5-2)*40,(self.scenario as i32/5%5-2)*40))}
 }
 #[test]fn reachable_pose_handlers_match_2560_real_callbacks_and_rng(){
  let mut h=2166136261;
  for phase in [27,28,29,30,31,34,38,41,43,45]{for s in 0..256usize{
   let mut p=Pose{phase,class:0,cycles:(s*37)as u16,timer:[0,1,2,10,11,20,127,128,255][s%9],frame:[-1,0,1,2,3,4,5,6,8,9,10,32767][s/3%12],counter82:[-1,0,1,2,3,4,255][s/7%7],heading:(s%8)as u8,transition:0x7134,..Default::default()};
   let mut env=Fixture{scenario:s,queries:0,scans:0,relative:-1,rng:OriginalRng{words:std::array::from_fn(|i|(s*(i+1)+i+1)as u32),multiplayer:false}};
   let reset=tick_pose(&mut p,if s%2==0{0}else{255},if s%7==0{9}else{0},&mut env).unwrap();
   hash(&mut h,[phase as u32,s as u32,env.queries,env.scans,env.relative as u32,u32::from(reset),u32::from(p.phase),p.frame as u32,u32::from(p.heading),u32::from(p.cycles),u32::from(p.timer),p.counter82 as u32,u32::from(p.transition),0x31]);hash(&mut h,env.rng.words);
  }}
  assert_eq!(h,1618884660,"ZNov04PoseHandlerOracle");
 }
 #[test]fn failed_transport_always_passes_the_input_actor_not_inside_candidates(){
  let mut h=2166136261;
  for s in 0..512usize{let actor=32+(s%13)as u8;let mut calls=0;let mut args=2166136261;
   remove_transport_actor(actor,|role|((s+role)%4!=0).then_some((role as u8,(s*17+role*5)as u8)),|slot|{calls+=1;hash(&mut args,[u32::from(slot)]);});hash(&mut h,[s as u32,u32::from(actor),calls,args]);
  }
  assert_eq!(h,270263530,"ZNov04FailedTransportOracle: real59e60, recorded58ec0 arguments");
 }
}
