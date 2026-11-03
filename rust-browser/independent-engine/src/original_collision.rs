//! Native terrain-contact dispatcher0045a450/0044e3a0 and probes004503f0,
//!00450470,0044e0a0,0044e130. Coordinates and permission values are raw.
//! The caller owns dynamic-object permission overlays (0045a2e0).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Contact {
 pub position:(i16,i16),
 pub attempted:(i16,i16),
 pub x:Option<i16>,pub y:Option<i16>,pub axes:u8,pub pause:u8,
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_8064_native_dispatches_match_axis_contacts_and_coordinate_corrections(){
  let mut hash=2166136261u32;let mut rows=0;let directions=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)];
  let offsets=[-128,-127,-126,0,1,2,128,129,130,256,257,258];
  for vehicle in 0..2{for fx in [0,1,7,8,9,15]{for fy in [0,1,7,8,9,15]{for(d,&(dx,dy))in directions.iter().enumerate(){for pattern in 0..14{
   let attempted=(512+fx,512+fy);let mut c=Contact::new((attempted.0-i16::from(dx),attempted.1-i16::from(dy)),attempted);
   let hit=c.dispatch((dx,dy),vehicle==1,|cell|if pattern==13{1}else if pattern<12&&i32::from(cell)==4128+offsets[pattern]{if (i32::from(fx)+i32::from(fy)+d as i32)&1==0{-1}else{1}}else{0});
   for n in [vehicle,i32::from(fx),i32::from(fy),d as i32,pattern as i32,i32::from(hit),i32::from(c.position.0),i32::from(c.position.1),i32::from(c.x.unwrap_or(-1)),i32::from(c.y.unwrap_or(-1)),i32::from(c.axes)]{hash=(hash^n as u32).wrapping_mul(16777619);}rows+=1;
  }}}}}
  assert_eq!(rows,8064);assert_eq!(hash,2820084749);
 }
 #[test]fn native_pause_decrements_low_five_bits_without_contacts(){
  for vehicle in [false,true]{for pause in [1,31,32,33,63,64,255]{let mut c=Contact::new((0,0),(512,512));c.pause=pause;
   assert!(!c.dispatch((1,-1),vehicle,|_|0));assert_eq!(c.pause,if pause&31!=0{pause-1}else{pause});assert_eq!((c.x,c.y,c.axes),(None,None,0));
  }}
 }
 #[test]fn all_120_native_segment_walks_match_pixel_dda_and_vehicle_wrapper(){
  let starts=[(400,480),(464,512),(560,300),(600,700),(544,530)];let goals=[32+32*128,45+30*128,32+15*128,20+45*128];let mut h=2166136261u32;let mut rows=0;
  for vehicle in 0..2{for mode in 0..3{for(si,&start)in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){
   let end=((goal&127)*16+if vehicle==1{7}else{0},(goal>>7)*16+if vehicle==1{0}else{-2});let c=trace(start,end,vehicle==1,|cell,_|{let(x,y)=(cell&127,cell>>7);if mode!=0&&(30..=34).contains(&x)&&(20..=40).contains(&y){if mode==1{-1}else{1}}else{0}});
   for n in [vehicle,mode,si as i32,gi as i32,i32::from(c.axes!=0),i32::from(c.position.0),i32::from(c.position.1),i32::from(c.x.unwrap_or(-1)),i32::from(c.y.unwrap_or(-1)),i32::from(c.axes)]{h=(h^n as u32).wrapping_mul(16777619);}rows+=1;
  }}}}
  assert_eq!(rows,120);assert_eq!(h,35763167);
 }
 #[test]fn all_72_native_collision_masks_match_team_and_direction_switches(){
  let mut h=2166136261u32;
  for(k,kind)in[MaskKind::Foot,MaskKind::Vehicle,MaskKind::Crane].into_iter().enumerate(){for team in 0..3{for direction in 0..8{
   for n in[k as i32,i32::from(team),i32::from(direction)]{h=(h^n as u32).wrapping_mul(16777619);}
   for code in 0..67{h=(h^i32::from(permission(code,kind,team,direction))as u32).wrapping_mul(16777619);}
  }}}
  assert_eq!(h,3395159258);
 }
 #[test]fn all_144_initialized_world_collision_masks_match_the_complete_native_constructor(){
  let mut h=2166136261u32;
  for world in 0..2{for(k,kind)in[MaskKind::Foot,MaskKind::Vehicle,MaskKind::Crane].into_iter().enumerate(){for team in 0..3{for direction in 0..8{
   for n in[world,k as i32,i32::from(team),i32::from(direction)]{h=(h^n as u32).wrapping_mul(16777619);}
   for code in 0..67{h=(h^i32::from(world_permission(code,kind,team,direction,world==1))as u32).wrapping_mul(16777619);}
  }}}}
  assert_eq!(h,585093049,"complete433b30→45f6a0→1b950→5a2e0 native execution");
 }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum MaskKind{Foot,Vehicle,Crane}
const FOOT:[i8;67]=[-1,-1,-1,-1,-1,-1,-1,0,0,0,0,-1,-1,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,-1,-1,0,0,-1,-1,0,-1,-1,0,0,0,0,0,0,0,-1,0,0,-1,0,-1,-1,-1,-1,-1,-1,0,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,0,0,0,0];
const VEHICLE:[i8;67]=[-1,-1,-1,-1,-1,-1,-1,0,0,-1,0,-1,-1,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,-1,-1,0,0,-1,-1,0,-1,-1,-1,0,0,0,0,-1,-1,-1,0,-1,-1,0,-1,-1,-1,-1,-1,-1,0,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,0,0,0,0];
const CRANE:[i8;67]=[-1,-1,-1,-1,-1,-1,-1,0,0,-1,0,-1,-1,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,-1,-1,0,0,-1,-1,0,-1,-1,-1,0,0,0,0,-1,-1,-1,0,-1,-1,0,-1,-1,0,0,0,0,0,-1,-1,-1,-1,0,0,0,-1,0,0,0,0,0,0,0];
///0041b950 ->0045a2e0 with0045f6a0 initialization, plus the wrapper's
/// own-team overrides. Volcan's world initialization is a separate overlay.
pub fn permission(code:u8,kind:MaskKind,team:u8,octant:u8)->i8{
 let code=usize::from(code&127);let mut v=match kind{MaskKind::Foot=>FOOT.get(code),MaskKind::Vehicle=>VEHICLE.get(code),MaskKind::Crane=>CRANE.get(code)}.copied().unwrap_or(-1);
 let north=matches!(octant,0|1|7);let south=matches!(octant,3|4|5);
 if kind==MaskKind::Foot&&code==31&&south{v=0;}
 if code==66&&(north||kind==MaskKind::Foot&&team<2){v=-1;}
 if code==62&&team==0&&!north{v=-1;}
 if code==63&&team==1&&match kind{MaskKind::Vehicle=>!north&&!south,_=>!south}{v=-1;}
 if [0,45,53,54,55].into_iter().any(|d|code==usize::from(team)+d){v=0;}
 v
}
///433b30 mutates the physical infantry defaults before45f6a0 copies them.
///The separate path mask always blocks40; physical contact allows it outside
///Volcan. The role-dependent Tough path mask is not written by this overlay.
pub fn world_permission(code:u8,kind:MaskKind,team:u8,octant:u8,volcan:bool)->i8{
 if kind==MaskKind::Foot&&matches!(code&127,9|40){if volcan{-1}else{0}}else{permission(code,kind,team,octant)}
}
impl Contact{
 pub fn new(position:(i16,i16),attempted:(i16,i16))->Self{Self{position,attempted,x:None,y:None,axes:0,pause:0}}
 fn x_probe(&mut self,base:i16,direction:i8,vehicle:bool,permission:&mut impl FnMut(i16)->i8){
  let mut cell=base;
  let blocked=if vehicle{
   if permission(cell)<0{true}else if self.attempted.1&8==0&&{cell=base.wrapping_sub(128);permission(cell)<0}{true}
   else if self.attempted.1&15!=0{cell=base.wrapping_add(128);permission(cell)!=0}else{false}
  }else if self.attempted.1&8==0&&permission(cell)<0{true}
   else if self.attempted.1&15!=0{cell=base.wrapping_add(128);permission(cell)!=0}else{false};
  if blocked{if direction>=0{self.position.0=self.position.0.wrapping_add(8);}self.position.0&=!15;self.axes=1;self.x=Some(cell);}
 }
 fn y_probe(&mut self,base:i16,direction:i8,vehicle:bool,permission:&mut impl FnMut(i16)->i8){
  let mut cell=base;
  let blocked=if permission(cell)<0{true}
   else if vehicle&&{cell=base.wrapping_add(1);permission(cell)<0}{true}
   else if self.attempted.0&15!=0{cell=base.wrapping_add(if vehicle{2}else{1});permission(cell)!=0}else{false};
  if blocked{self.position.1=if direction<0{(self.position.1&!7)|8}else{self.position.1.wrapping_add(8)&!15};self.axes|=2;self.y=Some(cell);}
 }
 /// Direction values are the signed accumulated X/Y displacement, rather
 /// than an octant. Diagonal dispatchers restore both corrected coordinates.
 pub fn dispatch(&mut self,direction:(i8,i8),vehicle:bool,mut permission:impl FnMut(i16)->i8)->bool{
  self.axes=0;self.x=None;self.y=None;
  if self.pause&31!=0{self.pause=(self.pause&!31)|self.pause.wrapping_sub(1)&31;return false;}
  let(dx,dy)=direction;let(x,y)=self.attempted;
  let base=((y as u16&!15).wrapping_shl(3)).wrapping_add((x>>4)as u16)as i16;
  let old=self.position;
  if dx!=0&&(if dx<0{x&8!=0}else{x&8==0&&x&7!=0}){
   self.x_probe(base.wrapping_add(if dx<0{if vehicle{128}else{0}}else if vehicle{130}else{1}),dx,vehicle,&mut permission);
  }
  if dy!=0&&y&8==0&&(dy<0||y&7!=0){
   self.y_probe(base.wrapping_add(if dy<0{0}else if vehicle{256}else{128}),dy,vehicle,&mut permission);
  }
  if dx!=0&&dy!=0{self.position=old;}
  self.axes!=0
 }
 ///0044e520 shifts the vehicle test origin by(-7,-8), including the stored
 /// current position. Infantry0045a5c0 uses its position without this shift.
 /// Dynamic object/team overrides must already be reflected by permission.
 pub fn wrapped(&mut self,direction:(i8,i8),vehicle:bool,permission:impl FnMut(i16)->i8)->bool{
  if vehicle{self.position=(self.position.0.wrapping_sub(7),self.position.1.wrapping_sub(8));self.attempted=(self.attempted.0.wrapping_sub(7),self.attempted.1.wrapping_sub(8));}
  let hit=self.dispatch(direction,vehicle,permission);
  if vehicle{self.position=(self.position.0.wrapping_add(7),self.position.1.wrapping_add(8));self.attempted=(self.attempted.0.wrapping_add(7),self.attempted.1.wrapping_add(8));}
  hit
 }
}

/// One-pixel dummy walk used by0043bfb0 to validate native contour shortcuts.
/// The permission callback receives the movement octant, after004525a0.
/// Collision response of the dummy retains the corrected old foot position;
/// the vehicle dummy retains the attempted position instead (004518e0).
pub fn trace(start:(i16,i16),goal:(i16,i16),vehicle:bool,permission:impl FnMut(i16,u8)->i8)->Contact{trace_impl(start,goal,vehicle,true,permission)}
pub fn trace_before_response(start:(i16,i16),goal:(i16,i16),vehicle:bool,permission:impl FnMut(i16,u8)->i8)->Contact{trace_impl(start,goal,vehicle,false,permission)}
fn trace_impl(start:(i16,i16),goal:(i16,i16),vehicle:bool,dummy_response:bool,mut permission:impl FnMut(i16,u8)->i8)->Contact{
 let(mut x,mut y)=(i32::from(start.0),i32::from(start.1));let dx=i32::from(goal.0)-x;let dy=i32::from(goal.1)-y;let(ax,ay)=(dx.abs(),dy.abs());let sx=if dx>0{1}else{-1};let sy=if dy<0{-1}else{1};let major=ax.max(ay);let mut error=major>>1;
 let octant=if dx>0{if ax<ay{if dy<0{if ax<((-dy)>>1){0}else{1}}else if ax<(dy>>1){4}else{3}}else if dy<0{if (ax>>1)<(-dy){1}else{2}}else if(ax>>1)<dy{3}else{2}}
  else if ax<ay{if dy<0{if ax<((-dy)>>1){0}else{7}}else if ax<(dy>>1){4}else{5}}else if dy<0{if (ax>>1)<=(-dy){7}else{6}}else if(ax>>1)<=dy{5}else{6};
 let mut result=Contact::new(start,start);
 for _ in 0..major{
  let old=(x as i16,y as i16);let(mut vx,mut vy)=(0,0);
  if ax>=ay{x+=sx;vx=sx as i8;error-=ay;if error<0{error+=ax;y+=sy;vy=sy as i8;}}
  else{y+=sy;vy=sy as i8;error-=ax;if error<0{error+=ay;x+=sx;vx=sx as i8;}}
  result=Contact::new(old,(x as i16,y as i16));
  if result.wrapped((vx,vy),vehicle,|cell|permission(cell,octant)){
   if vehicle&&dummy_response{result.position=result.attempted;}return result;
  }
  result.position=result.attempted;
 }
 result
}
