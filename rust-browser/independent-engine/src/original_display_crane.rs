//!55f10 ->455820 crane composition. Work86's six44fa40 records are separate
//!and are already submitted before this compound three-layer record.
//!Unknown warm9a, signed W21 outside0..7, and negative glScissor heights are
//!explicit adapter boundaries. This does not assert GPU pixel parity.
use super::{Record,hardware::{Clip,Draw,Layer}};
use crate::original_world_contact::{Building,first_building};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{MissingWork9a,InvalidW21(i8),MissingWorld,MissingForts,MissingHardware,NegativeScissor,UnsupportedPalette(u8)}
impl Boundary{pub fn label(self)->&'static str{match self{Self::MissingWork9a=>"missingWork9a",Self::InvalidW21(_)=>"invalidW21",Self::MissingWorld=>"missingWorld",Self::MissingForts=>"missingForts",Self::MissingHardware=>"missingHardware",Self::NegativeScissor=>"negativeScissor",Self::UnsupportedPalette(_)=>"unsupportedPalette"}}}
#[derive(Clone,Copy,Debug)]pub struct Input{pub position:[i16;2],pub heading:u8,pub frame:i16,pub turret:i8,pub work:i8,pub flags64:u8,pub mask98:u8,pub elevation:i8}
const TURRET:[[i16;2];8]=[[-7,-6],[-5,-5],[0,-3],[-3,-1],[-6,-2],[-10,-1],[-12,-3],[-8,-5]];
const ARM:[[i16;2];8]=[[17,4],[27,6],[25,14],[22,22],[13,23],[2,20],[2,16],[10,4]];
const BUILD_CLIP:[[i16;2];9]=[[50,-1],[66,-1],[49,-1],[49,-1],[67,-1],[49,-1],[49,-1],[49,-1],[32,-1]];
fn add(a:[i16;2],b:[i16;2])->[i16;2]{[a[0].wrapping_add(b[0]),a[1].wrapping_add(b[1])]}
pub fn special_terrain(code:u8)->bool{matches!(code&127,30|31|44|62|63|64|65|66|48|50|47|57|58|56)}
///55f10 probes all ruined BUILD slots before any live BUILD slot. Type2
///ruins have no live-query allow gate. A failed pair does not move UNIT XY.
pub fn select_clip(code:u8,point:[i16;2],buildings:&[Building],forts:Option<[[i16;2];2]>,allow_type2:bool)->Result<Option<Clip>,Boundary>{
 if !special_terrain(code){return Ok(None);}
 let hit=first_building(buildings,point,allow_type2,true).or_else(||first_building(buildings,point,allow_type2,false));
 if let Some((slot,_))=hit{let b=buildings[slot];let below=b.position[1].wrapping_sub(point[1])<0;return Ok(Some(Clip{mode:if below{4}else{8},line:b.position[1].wrapping_add(BUILD_CLIP[b.kind as usize][usize::from(!below)])}));}
 let [top,bottom]=forts.ok_or(Boundary::MissingForts)?;
 if point[0]>=top[0]&&point[0]<=top[0].wrapping_add(160)&&point[1]>=top[1]&&point[1]<=top[1].wrapping_add(256){return Ok(Some(Clip{mode:4,line:top[1].wrapping_add(145)}));}
 if point[0]>=bottom[0]&&point[0]<=bottom[0].wrapping_add(160)&&point[1]>=bottom[1].wrapping_sub(56)&&point[1]<=bottom[1].wrapping_add(160){return Ok(Some(Clip{mode:8,line:bottom[1].wrapping_add(16)}));}Ok(None)
}
pub fn draw(i:Input,clip:Option<Clip>,extra_index:i16)->Result<Draw,Boundary>{
 let turret=usize::try_from(i.turret).ok().filter(|t|*t<8).ok_or(Boundary::InvalidW21(i.turret))?;
 let heading=i.heading&7;let base=add(i.position,[-6,i16::from(i.elevation).wrapping_sub(6)]);
 let palette=i.flags64&i.mask98;let mode=clip.map_or(0,|c|c.mode);
 let part=|position,sprite|Layer{position,sprite,palette,flags:mode,clip,parameter8:None};
 let body=part(base,5794i16.wrapping_add(i16::from(heading)*3).wrapping_add(i.frame));
 let turret_origin=add(add(base,TURRET[turret]),[6,-3]);let top=part(turret_origin,5818+i16::from(i.turret));
 let phase=if i.work<0{0}else{i.work as u8&63};let arm_origin=if (3..=5).contains(&heading){add(add(base,TURRET[turret]),ARM[turret])}else{add(turret_origin,ARM[turret])};
 let arm=part(arm_origin,5826+i16::from(phase));let mut layers=if (3..=5).contains(&heading){vec![arm,body,top]}else{vec![body,top,arm]};layers[2].flags|=1;
 Ok(Draw{header:Record{x:i.position[0].wrapping_sub(6),y:i.position[1].wrapping_add(i16::from(i.elevation)).wrapping_add(8),sprite:extra_index,palette:255,offset:-1}.into(),layers,flags64:i.flags64&!4,mask98:if i.flags64&4!=0{7}else{i.mask98}})
}
///The original writes the unused warm4d1912 word even without clipping.
///Structured nativeLayers does not serialize that unused state. This numeric
///packer accepts caller-owned warm storage/line for byte-level source checks.
pub fn packed_layers(draw:&Draw,previous:&[[u8;16]],unclipped_line:i16)->Vec<u8>{draw.layers.iter().enumerate().flat_map(|(n,l)|{let mut b=l.bytes_with_previous(previous.get(n).copied().unwrap_or([0;16]));if l.clip.is_none(){b[10..12].copy_from_slice(&unclipped_line.to_le_bytes());}b}).collect()}

#[cfg(test)]#[path="original_display_crane_tests.rs"]mod tests;
