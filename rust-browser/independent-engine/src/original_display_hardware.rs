//!55f10→455c30,455110,455290/454ff0 and the class2 gun84 branch.
//!The eight-byte header and16-byte regular-tank layers are retained together.
//!55f10's live-BUILD/fort clipping is selected before tank composition.
use super::{Record,PHASES};
#[derive(Clone,Copy,Debug)]pub struct Input{pub position:[i16;2],pub family:u8,pub class:u8,pub phase:u8,pub frame:i16,pub heading:u8,pub turret:u8,pub hp:i16,pub maximum:i16,pub hatch_body:u8,pub hatch_frame:i8,pub flags64:u8,pub mask98:u8,pub elevation:i8,pub counter84:i16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Clip{pub mode:u8,pub line:i16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Layer{pub position:[i16;2],pub sprite:i16,pub palette:u8,pub flags:u8,pub clip:Option<Clip>}
impl Layer{
 pub fn bytes(self)->[u8;16]{let mut b=[0;16];b[..2].copy_from_slice(&self.position[0].to_le_bytes());b[2..4].copy_from_slice(&self.position[1].to_le_bytes());b[4..6].copy_from_slice(&self.sprite.to_le_bytes());b[6]=self.palette;b[7]=self.flags;if let Some(c)=self.clip{let at=if c.mode==4{10}else{12};b[at..at+2].copy_from_slice(&c.line.to_le_bytes());}b}
 ///454630's native scissor before resolution scaling. Positions are in the
 ///same coordinate space as the layer; line is an absolute native world Y.
 pub fn rectangle(self,camera_y:i16)->Option<[i32;4]>{let c=self.clip?;let y=if c.mode==4{i32::from(c.line)-i32::from(camera_y)+2}else{i32::from(self.position[1])};Some([i32::from(self.position[0]),y,256,if c.mode==4{256}else{i32::from(c.line)-i32::from(camera_y)-y+2}])}
}
#[derive(Clone,Debug)]pub struct Draw{pub header:Record,pub layers:Vec<Layer>,pub flags64:u8,pub mask98:u8}
fn add(a:[i16;2],b:[i16;2])->[i16;2]{[a[0].wrapping_add(b[0]),a[1].wrapping_add(b[1])]}
fn layer(position:[i16;2],sprite:i16,palette:u8)->Layer{Layer{position,sprite,palette,flags:0,clip:None}}
pub fn special_terrain(code:u8)->bool{matches!(code&127,30|31|44|62|63|64|65|66)}
const BUILD_CLIP:[[i16;2];9]=[[49,-1],[65,-1],[48,0],[48,0],[67,0],[48,0],[48,0],[48,0],[32,0]];
///55f10 ->41c290, then4544d0. The two identical failed live queries do not
///move the actor:41c290 writes -1 into the result pointer on every failure.
///Type2 admission is explicit because its flag belongs to the query caller.
pub fn building_clip(point:[i16;2],buildings:&[crate::original_world_contact::Building],allow_type2:bool)->Option<Clip>{
 if let Some((slot,_))=crate::original_world_contact::first_building(buildings,point,allow_type2,false){let b=buildings[slot];let below=b.position[1].wrapping_sub(point[1])<0;return Some(Clip{mode:if below{4}else{8},line:b.position[1].wrapping_add(BUILD_CLIP[b.kind as usize][usize::from(!below)])});}
 None
}
pub fn select_clip(code:u8,point:[i16;2],buildings:&[crate::original_world_contact::Building],forts:[[i16;2];2],allow_type2:bool)->Option<Clip>{
 if !special_terrain(code){return None;}
 if let Some(c)=building_clip(point,buildings,allow_type2){return Some(c);}
 let [top,bottom]=forts;
 if point[0]>=top[0]&&point[0]<=top[0].wrapping_add(160)&&point[1]>=top[1]&&point[1]<=top[1].wrapping_add(256){return Some(Clip{mode:4,line:top[1].wrapping_add(145)});}
 if point[0]>=bottom[0]&&point[0]<=bottom[0].wrapping_add(160)&&point[1]>=bottom[1].wrapping_sub(56)&&point[1]<=bottom[1].wrapping_add(160){return Some(Clip{mode:8,line:bottom[1].wrapping_add(16)});}
 None
}
pub fn draw_clipped(i:Input,clip:Option<Clip>)->Option<Draw>{let mut d=draw(i)?;if let Some(c)=clip{if d.layers.is_empty(){return None;}for l in &mut d.layers{l.flags|=c.mode;l.clip=Some(c);}}Some(d)}
///Only confirmed regular tank families6/25/30 and live class2 gun records.
///None retains the existing artwork adapter for other hardware families.
pub fn draw(i:Input)->Option<Draw>{
 if i.heading>7||i.turret>7||matches!(i.phase,18|68){return None;}let p=[i.position[0],i.position[1].wrapping_add(i16::from(i.elevation))];
 if i.class==2{return gun(i,p);}
 if i.class!=1||!matches!(i.family,6|25|30){return None;}
 let heading=usize::from(i.heading&3);let turret=usize::from(i.turret);let anchor=add(p,[-6,if i.family==6{0}else{-6}]);let palette=i.flags64&i.mask98;
 let(hull,body_offset,turret_base,turret_offset)=match i.family{6=>(if i.hp<i.maximum>>2{500i16}else{488},MEDIUM_BODY[heading],513,MEDIUM_TURRET[turret]),25=>(5305,HEAVY_BODY[heading],5329,HEAVY_TURRET[turret]),30=>(if i.hp<i.maximum>>2{5747}else{5719},LIGHT_BODY[heading],5731,LIGHT_TURRET[turret]),_=>unreachable!()};
 let turret_position=add(add(anchor,[6,-3]),turret_offset);let mut layers=vec![layer(add(anchor,body_offset),hull.wrapping_add((heading as i16).wrapping_mul(3)).wrapping_add(i.frame),palette),layer(turret_position,turret_base+i16::from(i.turret),palette)];
 if i.family==6{layers.push(layer(add(turret_position,MEDIUM_BARREL[turret]),521+i16::from(i.turret),palette));}
 if i.hatch_body!=255{
  let closed_offset=match i.family{6=>MEDIUM_HATCH[turret],25=>HEAVY_HATCH[turret],30=>LIGHT_HATCH[turret],_=>unreachable!()};let closed=layer(add(anchor,closed_offset),533+i16::from(i.turret)*3+i16::from(i.hatch_body&15),i.flags64&7);
  if i.hatch_frame== -1{layers.push(closed);}else{let early=i.hatch_frame<3;let crew_offset=match(i.family,early){(6,true)=>[6,-6],(6,false)=>MEDIUM_CREW[turret],(25,true)=>HEAVY_CREW_EARLY[turret],(25,false)=>HEAVY_CREW_LATE[turret],(30,_)=>LIGHT_CREW[turret],_=>unreachable!()};let crew=layer(add(anchor,crew_offset),if early{1464+i16::from(i.hatch_frame)}else{1469+i16::from(i.hatch_frame)+i16::from(i.turret)*2},i.flags64&7);if matches!(i.turret,0|1|2|6|7){layers.extend([crew,closed]);}else{layers.extend([closed,crew]);}}
 }
 layers.last_mut()?.flags|=1;
 Some(Draw{header:Record{x:p[0].wrapping_sub(6),y:p[1].wrapping_add(8),sprite:0,palette:255,offset:-1},layers,flags64:i.flags64&!4,mask98:if i.flags64&4!=0{7}else{i.mask98}})
}
fn gun(i:Input,mut p:[i16;2])->Option<Draw>{let d=usize::from(i.heading);let shifted=match i.phase{50=>{if i.counter84!= -1{p=add(p,MISSILE_FLASH[d]);}true},51=>{p=add(p,GATLING_FLASH[d]);true},52=>{p=add(p,HOWITZER_FLASH[d]);true},_=>false};let sprite=if i.counter84== -1{let &[base,stride]=PHASES.get(usize::from(i.phase))?;base.wrapping_add(i16::from(i.heading).wrapping_mul(stride)).wrapping_add(i.frame)}else if i.counter84<3{p=add(p,[12,if shifted{12}else{8}]);2412i16.wrapping_add(i.counter84)}else{let base=match i.phase{50=>2420i16,51=>2416,52=>2424,_=>2412};base.wrapping_add(i.counter84)};Some(Draw{header:Record{x:p[0],y:p[1],sprite,palette:i.flags64&7,offset:i.elevation.wrapping_neg()},layers:vec![],flags64:i.flags64&!4,mask98:i.mask98})}
const MEDIUM_BODY:[[i16;2];4]=[[0,0],[-2,-7],[-2,0],[0,-7]];
const MEDIUM_TURRET:[[i16;2];8]=[[0,0],[-2,0],[-3,-3],[-2,-3],[0,-3],[0,-3],[0,-3],[-1,0]];
const MEDIUM_BARREL:[[i16;2];8]=[[7,-6],[20,0],[19,3],[13,10],[6,11],[-2,9],[-6,3],[-2,-4]];
const HEAVY_BODY:[[i16;2];4]=[[0,0];4];
const HEAVY_TURRET:[[i16;2];8]=[[-7,0],[-6,0],[-2,2],[-6,1],[-7,2],[-8,1],[-12,2],[-8,0]];
const LIGHT_BODY:[[i16;2];4]=[[-1,0],[-4,0],[-5,0],[-5,0]];
const LIGHT_TURRET:[[i16;2];8]=[[-9,1],[-9,0],[-9,0],[-9,0],[-9,3],[-10,2],[-9,2],[-10,2]];
const MEDIUM_HATCH:[[i16;2];8]=[[11,-6];8];
const HEAVY_HATCH:[[i16;2];8]=[[15,6],[13,9],[8,8],[8,5],[8,3],[9,0],[14,2],[15,3]];
const LIGHT_HATCH:[[i16;2];8]=[[9,4],[8,3],[8,2],[8,2],[9,3],[9,2],[9,2],[10,3]];
const HEAVY_CREW_LATE:[[i16;2];8]=[[11,0],[11,6],[8,8],[7,4],[6,4],[5,1],[8,2],[11,0]];
const HEAVY_CREW_EARLY:[[i16;2];8]=[[11,4],[8,8],[4,7],[4,6],[3,3],[4,0],[10,2],[11,4]];
const LIGHT_CREW:[[i16;2];8]=[[7,0],[8,-3],[9,1],[8,1],[5,3],[5,2],[1,2],[3,0]];
const MEDIUM_CREW:[[i16;2];8]=[[7,-12],[11,-10],[12,-6],[10,-6],[7,-6],[5,-6],[1,-6],[3,-10]];
const MISSILE_FLASH:[[i16;2];8]=[[1,0],[1,0],[0,-1],[0,-1],[0,-1],[1,0],[2,-1],[1,0]];
const GATLING_FLASH:[[i16;2];8]=[[1,-2],[2,-1],[5,-1],[2,0],[1,-1],[0,1],[-4,-1],[0,-3]];
const HOWITZER_FLASH:[[i16;2];8]=[[1,-6],[1,-6],[4,-6],[1,-3],[1,-3],[1,-3],[-1,-6],[1,-6]];
#[cfg(test)]mod tests{
 use super::*;fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}fn bytes(data:impl IntoIterator<Item=u8>)->u32{let mut h=2166136261;mix(&mut h,data.into_iter().map(u32::from));h}
 #[test]fn executed_regular_hardware_draws_match_headers_layers_white_mask_resets_and_palette_dispatch(){
  let csv=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../work/parity-audit/native-hardware-display.csv");let csv=std::fs::read_to_string(csv).ok();let rows=csv.as_ref().map(|s|s.lines().map(|line|line.split(',').collect::<Vec<_>>()).collect::<Vec<_>>());let mut all=2166136261u32;
  for s in 0..16384usize{let which=s%7;let input=Input{position:[500,600],family:[6,25,30,5,21,22,23][which],class:if which<3{1}else{2},phase:[9,9,9,0,50,51,52][which],heading:(s/7%8)as u8,turret:(s/56%8)as u8,frame:(s/448%3)as i16,hatch_body:[0,1,2,255][s/1344%4],hatch_frame:[-1,0,1,2,3,4][s*11%6],flags64:[0,1,2,3,4,5,6,7,16,17,20,21][s/2%12],mask98:[0,1,3,7][s/5%4],hp:[24,25,100][s/9%3],maximum:100,counter84:[-1,0,1,2,3,4,6][s/13%7],elevation:[-32,0,7][s/17%3]};let initial=(s/23%4)as u8;let out=draw(input).unwrap();let header=out.header.bytes();let extra=out.layers.iter().flat_map(|r|r.bytes()).collect::<Vec<_>>();let(mut trace,mut uploads,mut count)=(2166136261u32,0u32,0u32);let mut palette=super::super::Palette{current:initial,uploaded:initial};let records=if out.layers.is_empty(){vec![(out.header.x,out.header.y.wrapping_add(i16::from(out.header.offset)),out.header.sprite,out.header.palette)]}else{out.layers.iter().map(|l|(l.position[0],l.position[1],l.sprite,l.palette)).collect()};for(x,y,sprite,pal)in records{if palette.plain(pal,false){uploads+=1;mix(&mut trace,[0,u32::from(palette.current)]);}mix(&mut trace,[1,x as u32,y as u32,sprite as u32,u32::from(palette.current)]);count+=1;}
   let values=[s as u32,u32::from(input.family),u32::from(input.phase),u32::from(input.heading),u32::from(input.turret),input.frame as u32,u32::from(input.hatch_body),input.hatch_frame as u32,u32::from(input.flags64),u32::from(input.mask98),input.hp as u32,input.counter84 as u32,input.elevation as u32,u32::from(initial),bytes(header),bytes(extra.iter().copied()),u32::from(out.flags64),u32::from(out.mask98),27,trace,count,uploads,u32::from(palette.current)];mix(&mut all,values);
   if let Some(rows)=&rows{let r=&rows[s];let hex=|b:&[u8]|b.iter().map(|v|format!("{v:02x}")).collect::<String>();assert_eq!(r[14],hex(&header),"header scenario{s}");assert_eq!(r[15],hex(&extra),"layers scenario{s}");assert_eq!([r[16],r[17],r[18],r[19],r[20],r[21],r[22]].map(|v|v.parse::<u32>().unwrap()),[u32::from(out.flags64),u32::from(out.mask98),27,trace,count,uploads,u32::from(palette.current)],"dispatch scenario{s}");}
  }
  assert_eq!(all,3583215544,"actual55f10→tank/gun records→4549f0/454630 palette/raster calls; texture/upload/final raster endpoints are boundaries");
 }
 #[test]fn executed_build_fort_clipping_matches_every_layer_and_raster_scissor_then_disables_it(){
  use crate::original_world_contact::Building;
  let csv=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../work/parity-audit/native-hardware-clip.csv")).ok();let rows=csv.as_ref().map(|c|c.lines().filter(|l|!l.starts_with("TABLE,")).map(|l|l.split(',').collect::<Vec<_>>()).collect::<Vec<_>>());let mut all=2166136261u32;let(mut upper,mut lower,mut negative)=(0,0,0);
  for s in 0..8192usize{
   let family=[6,25,30][s%3];let phase=[9,30,31,44,62,63,64,65,66][s/3%9];let terrain=[0,30,31,44,62,63,64,65,66][s/27%9];let mode=s/243%12;let kind=if mode<9{mode as i8}else{-1};let bx=500-[0,16,79,80,81][s/5%5];let by=600-[-1,0,1,16,48,64,65,96][s/7%8];let hp=[0,1,-1][s/11%3];let allow=s/13%2;let slot=if s/17%2==0{0}else{17};let second=s/19%2;let mut buildings=vec![Building{position:[0,0],kind:-1,hp:0};35];if kind>=0{buildings[slot]=Building{position:[bx,by],kind,hp};}if second!=0{buildings[34]=Building{position:[480,580],kind:1,hp:1};}
   let input=Input{position:[460,540],family,class:1,phase,heading:(s/23%8)as u8,turret:(s/29%8)as u8,frame:(s/31%3)as i16,hatch_body:[0,1,2,255][s/37%4],hatch_frame:[-1,0,1,2,3,4][s/41%6],flags64:[0,1,2,3,4,5,6,7][s/43%8],mask98:[0,1,3,7][s/47%4],hp:24,maximum:100,elevation:[-32,0,7][s/53%3],counter84:-1};let forts=[if mode==9{[500,600]}else{[2000,2000]},if mode==10{[500,600]}else{[3000,3000]}];let clip=select_clip(terrain,[500,600],&buildings,forts,allow!=0);if let Some(c)=clip{if c.mode==4{upper+=1;}else{lower+=1;}}
   let out=draw_clipped(input,clip).unwrap();let header=out.header.bytes();let extra=out.layers.iter().flat_map(|l|l.bytes()).collect::<Vec<_>>();let mut palette=super::super::Palette{current:0,uploaded:0};let mut trace=2166136261;for l in &out.layers{palette.plain(l.palette,false);let r=l.rectangle(60).unwrap_or([0,0,-1,0]);if r[3]<0{negative+=1;}mix(&mut trace,[l.position[0]as u32,l.position[1]as u32,l.sprite as u32,u32::from(palette.current),r[0]as u32,r[1]as u32,r[2]as u32,r[3]as u32]);}
   let values=[s as u32,u32::from(family),u32::from(phase),u32::from(terrain),mode as u32,kind as u32,bx as u32,by as u32,hp as u32,allow as u32,slot as u32,second as u32,u32::from(input.heading),u32::from(input.turret),input.frame as u32,u32::from(input.hatch_body),input.hatch_frame as u32,u32::from(input.flags64),u32::from(input.mask98),input.elevation as u32,bytes(header),bytes(extra.iter().copied()),clip.map_or(0,|c|u32::from(c.mode)),clip.map_or(0,|c|c.line as u32),u32::from(out.flags64),u32::from(out.mask98),trace,out.layers.len()as u32,u32::MAX];mix(&mut all,values);
   if let Some(rows)=&rows{let r=&rows[s];let hex=|b:&[u8]|b.iter().map(|v|format!("{v:02x}")).collect::<String>();assert_eq!(r[20],hex(&header),"header{s}");assert_eq!(r[21],hex(&extra),"extra{s}");assert_eq!([r[22],r[23],r[24],r[25],r[26],r[27],r[28]].map(|n|n.parse::<i64>().unwrap()as u32),values[22..],"clip/dispatch{s}");}
  }
  assert!(upper>100&&lower>100&&negative>0,"both clip directions and the explicit negative-GL-height boundary must be exercised");assert_eq!(all,2503140282,"8192 original55f10 BUILD/fort layers and454630 scissor arguments, including width=-1 reset");
 }
}
