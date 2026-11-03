//! Native176b0 CPU artillery placement and its174e0/126e0 spatial kernels.
//! Input coordinates are native pixels. The caller supplies the real RNG and
//! physical pools; no nearest-building search or replacement placement grid.
use super::original_world_contact::{Building,BUILDING_SHAPES};
pub const SOCKETS:[[i16;2];6]=[[17,170],[113,1],[113,170],[17,1],[113,49],[17,49]];
pub const SOCKET_REMAP:[usize;6]=[3,1,5,4,0,2];
const MISSION_ORDERS:[&[u8;6];20]=[b"ABCDEF",b"EDAFCB",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"EDAFCB",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD",b"ABEFCD"];
const BLOCKED:&[u8]=&[1,2,3,4,5,6,7,9,11,12,13,14,15,16,17,18,19,20,21,23,24,25,26,27,28,29,32,33,34,35,36,37,38,39,40,41,42,43,44,46,47,48,49,50,51,52,56,57,58,60,61];
pub fn socket_order(level:usize)->Option<[usize;6]>{let order=MISSION_ORDERS.get(level)?;Some(std::array::from_fn(|i|SOCKET_REMAP[usize::from(order[i]-b'A')]))}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Placement{pub position:[i16;2],pub socket:Option<usize>}
pub fn actor_anchor(position:[i16;2],profile:u8)->[i16;2]{let d=if profile==2{12}else{16};[position[0].wrapping_sub(d),position[1].wrapping_sub(d)]}
pub fn initial_gun_state(profile:u8)->Option<(u8,u8)>{Some(match profile{2=>(2,0),4=>(4,50),5=>(5,51),6=>(6,52),_=>return None})}
///13340 only considers active UNIT class2, supplied as actor anchors. Unit
///anchors round with(+7)&FFF0; all four corners use inclusive32px comparisons.
pub fn gun_occupied(position:[i16;2],guns:&[[i16;2]])->bool{guns.iter().any(|g|{let p=[g[0].wrapping_add(7)&!15,g[1].wrapping_add(7)&!15];corners_inside(p,[32,32],position,[32,32],false)})}
fn corners_inside(candidate:[i16;2],size:[i16;2],origin:[i16;2],extent:[i16;2],center:bool)->bool{
 let end=[origin[0].wrapping_add(extent[0]),origin[1].wrapping_add(extent[1])];let inside=|p:[i16;2]|(0..2).all(|i|origin[i]<=p[i]&&p[i]<=end[i]);
 [[0,0],[size[0],0],[0,size[1]],[size[0],size[1]]].into_iter().any(|d|inside([candidate[0].wrapping_add(d[0]),candidate[1].wrapping_add(d[1])]))||center&&inside([candidate[0].wrapping_add(16),candidate[1].wrapping_add(16)])
}
///124f0 ignores HP and type2; it tests candidate corners against each BUILD's
///descriptor30/31 dimensions, rather than ordinary blast/contact rectangles.
pub fn building_occupied(position:[i16;2],buildings:&[Building])->bool{buildings.iter().take(35).any(|b|{
 if b.kind==2{return false;}let Some(shape)=usize::try_from(b.kind).ok().and_then(|k|BUILDING_SHAPES.get(k))else{return false;};
 let size=[i16::from(shape[0])*16,i16::from(shape[1])*16-17+if b.kind==1{8}else{0}];corners_inside(position,[32,32],b.position,size,false)
})}
#[derive(Clone,Copy,Debug)]pub struct ObjectBox{pub position:[i16;2],pub extent:[i8;2]}
///41c790 has no flags13 gate. It tests candidate corners AND the center.
pub fn object_occupied(position:[i16;2],objects:&[ObjectBox])->bool{objects.iter().take(200).any(|o|o.position[0]!=-1&&corners_inside(position,[32,32],o.position,[i16::from(o.extent[0]),i16::from(o.extent[1])],true))}
pub fn flag_occupied(position:[i16;2],flags:&[[i16;2]])->bool{flags.iter().any(|f|{let p=[f[0].wrapping_add(4),f[1].wrapping_add(8)];(0..2).all(|i|position[i].wrapping_sub(8)<=p[i]&&p[i]<=position[i].wrapping_add(8))})}
pub fn terrain_occupied(position:[i16;2],terrain:&[u8])->bool{
 let x=position[0]>>4;let y=position[1]>>4;[(x,y),(x.wrapping_add(1),y),(x,y.wrapping_add(1)),(x.wrapping_add(1),y.wrapping_add(1))].into_iter().any(|(x,y)|{
  if !(0..128).contains(&x)||!(0..128).contains(&y){return true;}terrain.get(y as usize*128+x as usize).is_none_or(|b|BLOCKED.contains(&(b&127)))
 })
}
#[derive(Clone,Copy,Debug,Default)]pub struct View{pub fort_bounds:[i16;2],pub camera:[i16;2]}
///126e0's AI mode. `same_sector` is17400 on the aligned point and the
///temporary61b650 slot0, not a browser territory-tile approximation.
pub fn validate(position:[i16;2],same_sector:bool,product:u8,view:View,buildings:&[Building],guns:&[[i16;2]],objects:&[ObjectBox],flags:&[[i16;2]],terrain:&[u8])->Option<[i16;2]>{
 let position=[position[0]&!15,position[1]&!15];if !same_sector{return None;}
 if(0..2).all(|i|view.fort_bounds[i]<=position[i]&&position[i]<=view.fort_bounds[i].wrapping_add(144)){
  let dx=position[0].wrapping_sub(view.fort_bounds[0]);let dy=position[1].wrapping_sub(view.fort_bounds[1]);let slot=if dx<64{if dy<31{3}else if dy>80{0}else{5}}else if dy<31{1}else if dy<=80{4}else{2};
  let mut socket=[view.fort_bounds[0].wrapping_add(SOCKETS[slot][0]),view.fort_bounds[1].wrapping_add(SOCKETS[slot][1])];
  //The native126e0 parameter is the PRODUCT byte, not the weapon profile.
  if product!=2{socket[0]=socket[0].wrapping_sub(2);socket[1]=socket[1].wrapping_sub(8);}
  if gun_occupied(socket,guns){return None;}return Some([position[0].wrapping_sub(view.camera[0]),position[1].wrapping_sub(view.camera[1])]);
 }
 if building_occupied(position,buildings)||gun_occupied(position,guns)||flag_occupied(position,flags)||object_occupied(position,objects)||terrain_occupied(position,terrain){None}else{Some(position)}
}
///176b0: six fort sockets first, then45/53/54/55 marker scans. A successful
///marker is consumed even if a later marker replaces its candidate. The RNG
///only advances after a successful marker; a socket placement consumes none.
pub fn select(terrain:&mut[u8],rect:[i16;4],fort:Option<([i16;2],[usize;6])>,skill:i16,mut occupied:impl FnMut([i16;2])->bool,mut validate:impl FnMut([i16;2],&[u8])->Option<[i16;2]>,mut rng:impl FnMut()->u32)->Option<Placement>{
 if let Some((base,order))=fort{for slot in order{let p=[base[0].wrapping_add(SOCKETS[slot][0]).wrapping_add(12),base[1].wrapping_add(SOCKETS[slot][1]).wrapping_add(12)];if !occupied(p){return Some(Placement{position:p,socket:Some(slot)});}}}
 //Malformed sectors can make the native signed16 scan wrap into invalid
 //memory. Keep that external boundary explicit rather than clamping a map.
 if terrain.len()<128*128||rect[0]<0||rect[1]<0||rect[2]>128||rect[3]>128||rect[0]>rect[2]||rect[1]>rect[3]{return None;}
 let mut best=None;for marker in [45,53,54,55]{let mut found=None;
  'scan:for y in rect[1]..rect[3]{for x in rect[0]..rect[2]{let index=y as usize*128+x as usize;if terrain[index]&127!=marker{continue;}if let Some(p)=validate([x*16,y*16],terrain){terrain[index]&=128;found=Some(p);break 'scan;}}}
  if let Some(p)=found{best=Some(Placement{position:p,socket:None});if skill<(rng()&127)as i16{return best;}}
 }best
}
#[cfg(test)]mod tests{
 use super::*;
 fn hash(values:impl IntoIterator<Item=u32>)->u32{values.into_iter().fold(2166136261u32,|h,n|(h^n).wrapping_mul(16777619))}
 #[test]fn placement_validation_matches_512_complete_native_executions_without_stubs(){
  let mut values=Vec::new();for s in 0..512{
   let x=(s*13%80+10)*16;let y=(s*7%80+10)*16;let kind=[-1,0,1,2,8][(s>>3)%5];let bd=[-16,0,32][(s>>5)%3];let gd=[-48,-32,-16,0,16,32,48,64][(s>>2)&7];let od=[-16,0,16,32,48][(s>>4)%5];let extent=[-1,0,8,16,32,64,127][(s>>3)%7];
   let mut terrain=vec![0;16384];if s&4!=0{terrain[(y>>4)*128+(x>>4)]=1;}let x=x as i16;let y=y as i16;
   let buildings=[Building{position:[x+bd,y+bd],kind,hp:0}];let guns=if s&32!=0{vec![[x+gd,y+gd]]}else{vec![]};let objects=[ObjectBox{position:[x+od,y+od],extent:[extent,if s&64!=0{8}else{extent}]}];let flags=if s&128!=0{vec![[x-4,y-8]]}else{vec![]};let view=View{fort_bounds:if s&2!=0{[x,y]}else{[-500,-500]},camera:[(s%5*16)as i16,(s%3*16)as i16]};
   let point=[x+7,y+9];let result=validate(point,s&1==0,(s%19)as u8,view,&buildings,&guns,&objects,&flags,&terrain);let p=result.unwrap_or(point);values.extend([s as u32,u32::from(result.is_none()),p[0]as u32,p[1]as u32]);
  }
  assert_eq!(hash(values),1687225946,"ZNativeFactoryPlacementValidationOracle complete126e0/17400/124f0/13340/41cbd0/41c790/12640 executions");
 }
 #[test]fn cpu_sockets_marker_consumption_and_choice_rng_match_128_native_placement_executions(){
  let mut values=Vec::new();for s in 0..128{
   let mut terrain=vec![0;16384];for(x,y,code)in[(10,20,45),(11,20,45),(12,21,53),(13,21,53),(14,22,54),(15,22,54),(16,23,55),(17,23,55)]{terrain[y*128+x]=code|128;}
   let guns=if s&2!=0{SOCKETS.iter().map(|p|[1000+p[0],1000+p[1]]).collect::<Vec<_>>()}else{vec![]};let fort=if s&1!=0{Some(([1000,1000],socket_order(s%20).unwrap()))}else{None};let mut checks=0;let mut random=0;
   let placed=select(&mut terrain,[10,20,18,26],fort,[-1,0,63,127,140][(s>>4)%5],|p|gun_occupied(p,&guns),|p,_|{checks+=1;if s&4!=0||(i32::from(p[0]>>4)+i32::from(p[1]>>4)+s as i32)%3==0{None}else if s&8!=0{Some([p[0]+16,p[1]-16])}else{Some(p)}},||{random+=1;((random*29+s*17)%128)as u32});
   let p=placed.map_or([0,0],|p|p.position);values.extend([s as u32,u32::from(placed.is_some()),p[0]as u32,p[1]as u32,checks,random as u32,u32::from(placed.is_none()),hash(terrain.iter().map(|&b|u32::from(b)))]);
  }
  assert_eq!(hash(values),3748223709,"ZNativeFactoryPlacementOracle real176b0/174e0/13340; validation,RNG,creation are explicit boundaries");
 }
}
