//!53290 attack-command acceptance and48250's post-walk control flow.
//!Native spatial queries and called handlers are explicit inputs, never dummy
//!units. Coordinates are signed native pixels; raw50 retains its32-bit sentinel.
#[path="original_attack_order_game.rs"]mod game;
pub(crate) use game::{Result as AttackOrderResult,WalkSnapshot};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct State{
 pub class:u8,pub family:u8,pub role:i8,pub group:u8,pub position:[i16;2],
 pub order:i8,pub target:i32,pub point:[i16;2],pub phase:u8,pub frame:i16,pub timer:i8,
 pub flags31:u8,pub flags64:u8,pub flags69:u8,pub flags71:u8,pub flags9c:u8,pub flagsb7:u8,
 pub contact5d:u8,pub object36:i16,pub contact8c:u8,pub motion61:u8,pub field62:u8,
 pub counter68:i8,pub range4:i8,pub aim2:i8,
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Context{pub command:i8,pub arguments:[i16;3],pub flags9:u8,pub counter3d:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Live{pub point:[i16;2],pub class:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Effect{ReleaseObject(i16),ClearPaths,Route([i16;2]),MoveOrder,ResetIdle,FailedGroupOrder,StopReaction,Feedback,RefreshCounter(Option<Live>),HardwareTarget([i16;2])}
pub trait World{
 fn live_target(&mut self,state:&State)->Option<Live>;
 fn ground_allowed(&mut self,state:&State)->bool;
 fn grenade_total(&mut self)->u8;
 fn in_range_and_visible(&mut self,state:&State)->bool;
 fn random(&mut self)->u32;
 ///Called in source order. Path/animation handlers may update raw state.
 fn effect(&mut self,state:&mut State,context:&mut Context,effect:Effect);
}
pub fn squared(point:[i16;2])->i32{let x=i32::from(point[0]);let y=i32::from(point[1]);x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y))}
///5351e uses two float32 squares, each scaled by1/256, then a float64
///sqrt converted back tofloat32 before truncation and multiplication by5.
pub fn ground_in_range(position:[i16;2],target:[i16;2],range:i8)->bool{
 let x=(i32::from(position[0])-i32::from(target[0]))as f32;
 let y=(i32::from(position[1])-i32::from(target[1]))as f32;
 let distance=((y*y*(1f32/256.)+x*x*(1f32/256.))as f64).sqrt()as f32;
 (distance as i32).wrapping_mul(5)<i32::from(range)
}
pub fn hardware_phase(family:u8)->u8{match family{24=>53,25=>54,30=>55,32=>56,34=>58,36=>59,38=>60,_=>9}}
///432320 prepares both UNIT endpoints with their separate class offsets and
///terrain-cell heights, then runs the shared source432250 ray kernel.
pub fn unit_visible(a:[i16;2],aclass:u8,acell:i16,b:[i16;2],bclass:u8,bcell:i16,building_mask:bool,mut terrain:impl FnMut(i16)->u8)->bool{let at=a.map(|n|n.wrapping_add(if aclass==1{8}else{16})&!15);let bt=b.map(|n|n.wrapping_add(if bclass==1{8}else{16})&!15);let ah=DEFAULT_RAY[usize::from(terrain(acell)&127)]as i8;let bh=DEFAULT_RAY[usize::from(terrain(bcell)&127)]as i8;let table=if building_mask{&BUILDING_RAY}else{&DEFAULT_RAY};crate::original_initiative::native::clear_ray([at[0],at[1],i16::from(ah)],[bt[0],bt[1],i16::from(bh)],|cell|table[usize::from(terrain(cell)&127)]as i8)}
pub fn ground_ray(a:[i16;2],b:[i16;2],building_mask:bool,mut terrain:impl FnMut(i16)->u8)->bool{let a=a.map(|n|n.wrapping_add(8));let b=b.map(|n|n.wrapping_add(8));let table=if building_mask{&BUILDING_RAY}else{&DEFAULT_RAY};crate::original_initiative::native::clear_ray([a[0],a[1],0],[b[0],b[1],0],|cell|table[usize::from(terrain(cell)&127)]as i8)}
///Entire53290 control flow.52500/4490b0/53730/reset callbacks remain explicit
///owned handlers. A leader can resynchronize a different group order and retry.
pub fn accept(state:&mut State,context:&mut Context,world:&mut impl World)->u8{
 if state.class==2{return state.phase;}
 loop{
  state.flags9c&=!32;
  if state.flags31&8!=0{state.flags31&=!8;state.contact5d=0;world.effect(state,context,Effect::ReleaseObject(state.object36));}
  context.flags9&=!8;context.counter3d=0;
  let ground=state.target==3456;
  let live;
  let mut point;
  if ground{
   world.effect(state,context,Effect::ClearPaths);point=state.point;live=None;
   if !world.ground_allowed(state){if invalid(state,context,world){continue;}return state.phase;}
   if world.grenade_total()==0&&matches!(state.class,0|16){state.target=i32::from(point[0]);state.point=[point[1],2];state.order=1;world.effect(state,context,Effect::MoveOrder);return state.phase;}
  }else{
   live=world.live_target(state);let Some(target)=live else{if invalid(state,context,world){continue;}return state.phase;};
   world.effect(state,context,Effect::ClearPaths);state.flags71&=!32;if state.point[1]!=0{state.flags71|=32;}point=target.point;
  }
  if squared(point)<1024{
   let(dx,dy)=loop{let dy=(world.random()as u16&7)as i16-4;let dx=(world.random()as u16&7)as i16-4;if dx!=0||dy!=0{break(dx,dy);}};
   point=[point[0].wrapping_add(dx),point[1].wrapping_add(dy)];if state.flags9c&16==0{state.contact8c=0;}state.counter68=0;
   let preserve=(state.phase==8).then_some((state.frame as i8 as i16,state.timer));world.effect(state,context,Effect::Route(point));if let Some((frame,timer))=preserve{state.frame=frame;state.timer=timer;}
  }else{
   let in_range=if state.class==15||!ground{world.in_range_and_visible(state)}else{ground_in_range(state.position,point,state.range4)};
   if !in_range{state.counter68=0;world.effect(state,context,Effect::Route(point));}
  }
  state.order=state.order.wrapping_neg();world.effect(state,context,Effect::RefreshCounter(live));
  if state.class!=1{state.motion61=1;return 8;}state.flags64|=8;world.effect(state,context,Effect::HardwareTarget(state.point));return hardware_phase(state.family);
 }
}
fn invalid(state:&mut State,context:&mut Context,world:&mut impl World)->bool{
 if context.command==state.order&&i32::from(context.arguments[0])==state.target&&context.arguments[1]==state.point[0]&&context.arguments[2]==state.point[1]{
  state.order=1;context.command=1;let x=state.position[0].wrapping_add(8);let y=state.position[1].wrapping_add(8);state.target=i32::from(x);state.point[0]=y;context.arguments[0]=x;context.arguments[1]=y;
  world.effect(state,context,Effect::FailedGroupOrder);world.effect(state,context,Effect::ClearPaths);world.effect(state,context,Effect::ResetIdle);world.effect(state,context,Effect::StopReaction);world.effect(state,context,Effect::Feedback);return false;
 }
 state.order=0;
 if state.role>=0{world.effect(state,context,Effect::ClearPaths);world.effect(state,context,Effect::ResetIdle);world.effect(state,context,Effect::Feedback);return false;}
 state.flags69&=!128;
 if state.role== -1&&context.command!=0&&(state.order==12||state.flags69&128==0){state.order=context.command;state.target=i32::from(context.arguments[0]);state.point=[context.arguments[1],context.arguments[2]];}
 state.order==5&&state.class!=2
}
///48250 after48060. The original stalled-position test compares oldY to NEWX.
///`walk_phase` is48060's actual resulting phase before restoring old_phase.
pub fn after_walk(state:&mut State,old_position:[i16;2],old_phase:u8,walk_phase:u8,handler_marked:bool){
 state.phase=old_phase;
 if matches!(walk_phase,4|9|51|52|53|54|55|56|58|59|60|63){state.phase=walk_phase;return;}
 if state.field62!=0&&!handler_marked&&old_position[0]==state.position[0]&&old_position[1]==state.position[0]&&state.flags31&0x60==0{state.order=5;return;}
 if state.flags31&64!=0{state.flags64|=8;return;}
 if state.order!=12&&!matches!(walk_phase,5|57|65|19){state.phase=8;if state.flags31&32==0{state.counter68=state.counter68.wrapping_sub(1);if state.counter68<0{state.counter68=0;state.order=-5;}}}
}
///4490b0's exact branch gate. Its queued path uses native waypoints; the
///short direct525a0 branch receives an anchor without subtracting8.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Route{None,Queued,ShortDirect}
pub fn route_kind(s:&State)->Route{if s.role>=0||s.flags69&128!=0||s.counter68==0{if s.role<0&&(s.flags71&16!=0||s.flags69&128==0){Route::Queued}else{Route::ShortDirect}}else{Route::None}}
///449a90/4325e0. The source permission pointer survives a successful BUILD
///query; a miss preserves its previous table instead of restoring the default.
pub fn visible(position:[i16;2],target:Option<[i16;2]>,class:u8,ground:bool,range74:u8,building_mask:&mut bool,mut building:impl FnMut()->bool,mut terrain:impl FnMut(i16)->u8)->bool{
 let Some(target)=target else{return false;};if ground&&class==15{return false;}
 let dx=i32::from(position[0])-i32::from(target[0]);let dy=i32::from(position[1])-i32::from(target[1]);
 if dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))>=i32::from(range74).wrapping_mul(i32::from(range74)).wrapping_mul(49){return false;}
 if ground&&building(){*building_mask=true;}
 let table=if *building_mask{&BUILDING_RAY}else{&DEFAULT_RAY};
 let(a,b)=if ground{(position.map(|n|n.wrapping_add(8)),target.map(|n|n.wrapping_add(8)))}else{(target,position)};
 crate::original_initiative::native::clear_ray([a[0],a[1],0],[b[0],b[1],0],|cell|table[usize::from(terrain(cell)&127)]as i8)
}
const DEFAULT_RAY:[u8;128]=[0,3,4,5,6,7,8,0,0,0,0,1,2,3,4,5,1,1,1,1,1,1,0,0,1,1,1,1,1,0,0,1,1,1,1,0,0,1,0,1,0,0,1,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,152,184,73,0,124,177,73,0,176,189,73,0,88,190,73,0,8,178,73,0,255,255,255,255,255,255,255,0,0,0,0,255,255,255,255,255,255,255,255,255,0,0,0,0,255,255,0,0,255,255,0,0,255,0,0,0,0,0,0,0];
const BUILDING_RAY:[u8;128]=[0,3,4,5,6,7,8,0,0,0,0,1,2,3,4,5,0,0,1,0,0,0,0,0,1,1,0,0,1,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,72,154,100,0,224,249,99,0,56,130,99,0,240,73,100,0,0,0,1,0,2,0,16,0,17,0,18,0,32,0,33,0,34,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0];
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn unit_ray_matches1024_native432320_calls_with_stale_endpoint_cells(){let mut h=2166136261u32;for n in 0..1024usize{let mut terrain=vec![0u8;16384];for y in 0..128{terrain[y*128+34]=19;terrain[y*128+35]=24;}for x in 0..128{terrain[36*128+x]=if n%3==0{17}else{0};}let acell=n*31%16384;let bcell=(n*47+3)%16384;terrain[acell]=[0,17,19,24,35,38,66,127][n/16%8];terrain[bcell]=[0,17,19,24,35,38,66,127][n/128%8];let visible=unit_visible([500+(n%17)as i16,600+(n%11)as i16],[0,1,2,15][n%4],acell as i16,[500+((n%13)as i16-6)*30,600+((n/13%11)as i16-5)*30],[0,1,2,16][n/4%4],bcell as i16,n%2!=0,|c|terrain[c as u16 as usize]);if let Ok(path)=std::env::var("NATIVE_UNIT_RAY_ORACLE"){let text=std::fs::read_to_string(path).unwrap();let v:serde_json::Value=serde_json::from_str(text.lines().nth(n).unwrap()).unwrap();assert_eq!(u8::from(visible),v["visible"].as_u64().unwrap()as u8,"scenario{n}");}for v in[n as u32,u32::from(visible)]{h=(h^v).wrapping_mul(16777619);}}assert_eq!(h,2420957529,"actual432320 and432250, no ray stub");}
 #[test]fn post_walk_control_matches1024_native48250_executions(){let phases=[2,4,5,9,19,51,52,53,54,55,56,57,58,59,60,63,65,7,1];let mut h=2166136261u32;for scenario in 0..1024usize{let x=400+(scenario%16)as i16;let y=if scenario%3==0{x}else{500+(scenario%11)as i16};let old_phase=if(scenario*13)as u8&128!=0{2}else if scenario%2==0{8}else{7};let mut s=State{position:[x+(scenario/4%3)as i16-1,y+(scenario/12%3)as i16-1],phase:phases[scenario%phases.len()],field62:(scenario/3%2)as u8,flags31:[4,36,68,100][scenario/19%4],flags64:(scenario*7)as u8,order:if scenario%7==0{12}else{-5},counter68:(scenario*17)as i8,..Default::default()};after_walk(&mut s,[x,y],old_phase,phases[scenario%phases.len()],scenario&2!=0);for n in[scenario as u32,u32::from(s.phase),s.order as u32,s.counter68 as u32,u32::from(s.flags64)]{h=(h^n).wrapping_mul(16777619);}}assert_eq!(h,965164670,"actual48250;48060's result/position/marker are explicitinputs");}
 #[test]fn attack_visibility_matches512_complete_native449a90_ray_executions(){let mut h=2166136261u32;for s in 0..512usize{let target=[500+(s%13)as i16*30-180,600+(s/13%11)as i16*30-150];let class=[0,1,2,15][s%4];let ground=s/4%2!=0;let mut mask=s%2!=0;let mut queries=0;let valid=ground||s/16%2==0&&s/32%2==0;let yes=visible([500,600],valid.then_some(target),class,ground,[0,1,15,30,127,255][s/8%6],&mut mask,||{queries+=1;s%3==0},|cell|{let i=cell as u16 as usize;let(x,y)=(i&127,i>>7);if y==36{if s%3==0{17}else{0}}else if x==34{19}else if x==35{24}else{0}});for n in[s as u32,u32::from(yes),u32::from(mask),queries]{h=(h^n).wrapping_mul(16777619);}}assert_eq!(h,678313511,"actual48470/4325e0/432250, both original128-byte masks; BUILDhit explicitinput");}
 fn word(s:usize)->(State,Context){let class=[0,1,2,3,15,16,4,6][s%8];let ground=s/8%2!=0;let near=s/16%2!=0;let point=if near{[[0,0],[12,6],[-12,-6],[31,1]][s%4]}else{[300+(s%23)as i16*13,400+(s%19)as i16*17]};let raw=if ground{[point[0],point[1]]}else{[1900+s as i16,(s&1)as i16]};(State{class,family:[4,24,25,30,32,34,36,38][s%8],role:[-1,0,1][s/32%3],group:0,position:[50+(s%10)as i16*11,60+(s%7)as i16*13],range4:[-128,0,30,60,127][s%5],aim2:[-128,-1,0,40,60,120,127][s%7],flags31:4|if s&4!=0{8}else{0},flags9c:(s*11)as u8,flags71:(s*7)as u8,flags64:(s*3)as u8,flags69:(s*17)as u8,flagsb7:(s*19)as u8,order:5,contact5d:3,object36:0,target:if ground{3456}else{1},point:raw,phase:if s%3==0{8}else{1},frame:0x1290+s as i16,timer:(s*13)as i8,counter68:(s*23)as i8,contact8c:(s*29)as u8,..Default::default()},Context{flags9:(s*13)as u8,counter3d:(s*17)as u8,command:if s/384%2==0{5}else{1},arguments:[if s%3==0{123}else if ground{3456}else{1},raw[0],raw[1]]})}
 struct Env{scenario:usize,object13:u8,draws:usize,trace:u32}
 impl Env{fn n(&mut self,n:i32){self.trace=(self.trace^n as u32).wrapping_mul(16777619);}fn point(&self)->[i16;2]{let s=self.scenario;if s/16%2!=0{[[0,0],[12,6],[-12,-6],[31,1]][s%4]}else{[300+(s%23)as i16*13,400+(s%19)as i16*17]}}}
 impl World for Env{
  fn live_target(&mut self,state:&State)->Option<Live>{let s=self.scenario;(state.class!=1&&state.target==1&&s/96%2==0&&s/192%2==0).then_some(Live{point:self.point(),class:(s%4)as u8})}
  fn ground_allowed(&mut self,_:&State)->bool{self.n(4);if self.scenario/24%3==0{return true;}self.n(5);if self.scenario/24%3==1{return true;}self.n(6);self.scenario/48%2==0}
  fn grenade_total(&mut self)->u8{self.n(7);(self.scenario/12%2)as u8}
  fn in_range_and_visible(&mut self,_:&State)->bool{self.n(2);self.scenario&1!=0}
  fn random(&mut self)->u32{self.n(13);let r=if self.draws<2&&self.scenario%5==0{4}else{self.scenario*11+self.draws*3};self.draws+=1;r as u32}
  fn effect(&mut self,s:&mut State,c:&mut Context,e:Effect){match e{
   Effect::ReleaseObject(_)=>self.object13&=!1,Effect::ClearPaths=>self.n(1),Effect::Route(p)=>{self.n(i32::from(p[0]));self.n(i32::from(p[1]));self.n(3);s.phase=2;s.frame=333;s.timer=77;},Effect::MoveOrder=>{self.n(8);s.order=-1;s.phase=2;},Effect::ResetIdle=>{self.n(9);s.phase=1;},Effect::StopReaction=>{self.n(10);s.phase=42;},Effect::Feedback=>self.n(11),Effect::FailedGroupOrder=>{s.order=0;if s.role== -1&&s.group!=255{s.flagsb7&=!1;c.command=0;self.n(12);}},Effect::RefreshCounter(live)=>s.counter68=crate::original_pursuit::refresh_counter(s.target==3456,live.map(|l|l.class),true,s.aim2,s.position,live.map_or([0;2],|l|l.point)),Effect::HardwareTarget(_)=>{}
  }}
 }
 #[test]fn attack_acceptance_matches_768_executed_native_branch_records(){let mut h=2166136261u32;for scenario in 0..768{let(mut s,mut c)=word(scenario);let mut e=Env{scenario,object13:(scenario*13)as u8,draws:0,trace:2166136261};let result=accept(&mut s,&mut c,&mut e);let values=[scenario as u32,s.order as u32,u32::from(s.flags31),u32::from(s.flags9c),u32::from(s.flags71),u32::from(s.flags64),u32::from(s.flags69),u32::from(s.flagsb7),u32::from(s.contact5d),u32::from(s.contact8c),u32::from(s.motion61),s.counter68 as u32,u32::from(s.phase),s.frame as u32,s.timer as u32,s.target as u32,s.point[0]as u32,s.point[1]as u32,u32::from(c.flags9),u32::from(c.counter3d),c.command as u32,c.arguments[0]as u32,c.arguments[1]as u32,c.arguments[2]as u32,u32::from(e.object13),e.draws as u32,e.trace,u32::from(result)];if let Ok(path)=std::env::var("NATIVE_ATTACK_ORACLE"){let text=std::fs::read_to_string(path).unwrap();let v:serde_json::Value=serde_json::from_str(text.lines().nth(scenario).unwrap()).unwrap();let mut expected=vec![scenario as u32];for i in 0..17{expected.push(v[format!("v{i}")].as_i64().unwrap()as u32);}for key in["g9","g3d","gc","gx","gy","gm","object13","draws","trace","result"]{expected.push(v[key].as_i64().unwrap()as u32);}assert_eq!(values.as_slice(),expected.as_slice(),"scenario{scenario}");}for n in values{h=(h^n).wrapping_mul(16777619);}}assert_eq!(h,2482801987,"ZNativeAttackOrderOracle; query/path/idle callbacks are recorded boundaries");}
}
