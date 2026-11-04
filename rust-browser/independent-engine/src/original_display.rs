//!545b0's shared eight-byte queue and4549f0's plain-record palette dispatch.
//!The comparison-exchange traversal is deliberately not a stable Y sort.
#[path="original_display_game.rs"]mod game;
#[path="original_scene_display_game.rs"]mod scene;
#[path="original_display_hardware.rs"]pub(crate) mod hardware;
#[path="original_display_terrain.rs"]pub(crate) mod terrain;
#[path="original_display_terrain_game.rs"]mod terrain_game;
///UNIT submissions are retained from their own callback. A later OBJECT,
///ambient hit or slot retirement must not rewrite the already queued record.
#[derive(Clone,Copy)]pub(crate) struct FlagSeed{pub sector:usize,pub position:[i16;2]}
pub struct State{pub(crate) units:Option<Vec<(Record,serde_json::Value)>>,pub(crate) buildings_before:Option<Vec<serde_json::Value>>,pub(crate) entry69:[u8;240],pub(crate) flags:crate::original_flag_display::Pool,pub(crate) flag_seeds:Vec<FlagSeed>,pub(crate) flags_initialized:bool,pub(crate) flags_before:Option<Vec<(Record,serde_json::Value)>>,pub(crate) radar_before:Option<Vec<(Record,serde_json::Value)>>,pub(crate) fort_visible:[u8;2],pub(crate) scene_mode:i32}
impl Default for State{fn default()->Self{Self{units:None,buildings_before:None,entry69:[0;240],flags:Default::default(),flag_seeds:vec![],flags_initialized:false,flags_before:None,radar_before:None,fort_visible:[255;2],scene_mode:0}}}
impl State{pub fn clear_for_mission(&mut self){self.units=None;self.buildings_before=None;self.entry69.fill(0);self.flags.retire();self.flag_seeds.clear();self.flags_initialized=false;self.flags_before=None;self.radar_before=None;self.fort_visible=[255;2];self.scene_mode=0;}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Record{pub x:i16,pub y:i16,pub sprite:i16,pub palette:u8,pub offset:i8}
impl Record{pub fn bytes(self)->[u8;8]{let mut b=[0;8];b[..2].copy_from_slice(&self.x.to_le_bytes());b[2..4].copy_from_slice(&self.y.to_le_bytes());b[4..6].copy_from_slice(&self.sprite.to_le_bytes());b[6]=self.palette;b[7]=self.offset as u8;b}}
///The final slot is compared to every earlier slot, then the boundary moves
///backward. Equal-Y records can reverse through swaps with a third record.
pub fn sort<T>(records:&mut[T],mut y:impl FnMut(&T)->i16){for end in(1..records.len()).rev(){for i in 0..end{if y(&records[end])<y(&records[i]){records.swap(end,i);}}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Palette{pub current:u8,pub uploaded:u8}
impl Palette{
 ///40bb50, followed by4549f0's white flag and conditional46bde0 upload.
 ///FF inherits the current color; FF/FF records belong to the extra dispatcher.
 pub fn plain(&mut self,request:u8,force_neutral:bool)->bool{if request!=255{let p=if force_neutral{8}else{request&15};self.current=match p{p@0..=3=>p,4..=7=>255,_=>4};}if self.current==self.uploaded{return false;}self.uploaded=self.current;true}
}
#[derive(Clone,Copy,Debug)]pub struct ObjectInput{pub position:[i16;2],pub base:i16,pub animation:i8,pub sprite_offset:i8,pub flags:u8,pub depth:i8}
///43f260's primary record. Flags480 are produced separately by41d110;
///5215 is deliberately absent. Animation and sprite offsets are signedbytes.
pub fn object(input:ObjectInput)->Option<Record>{let[x,y]=input.position;if x<=0||input.base==5215{return None;}let sprite=input.base.wrapping_add(i16::from(input.animation)).wrapping_add(i16::from(input.sprite_offset));if sprite==480{return None;}let colored=input.flags&32!=0;Some(Record{x,y:if colored{y}else{y.wrapping_sub(i16::from(input.depth))},sprite,palette:if colored{input.flags>>1&15}else{255},offset:if colored{0}else{input.depth}})}
///55f10's normal sprite branch. Specialized hardware/fade compositions own
///their extra rows, but use the same common eight-byte sorting header.
pub fn infantry(position:[i16;2],phase:u8,frame:i16,heading:i8,elevation:i8,flags64:u8)->Option<Record>{if matches!(phase,18|68){return None;}let &[base,stride]=PHASES.get(usize::from(phase))?;Some(Record{x:position[0],y:position[1].wrapping_add(i16::from(elevation)),sprite:base.wrapping_add(i16::from(heading).wrapping_mul(stride)).wrapping_add(frame),palette:flags64&7,offset:elevation.wrapping_neg()})}
#[derive(Clone,Copy,Debug)]pub struct ObjectBinding{pub slot:u8,pub position:[i16;2],pub record:Option<Record>,pub unit:Option<usize>,pub object:Option<usize>,pub flag:Option<usize>,pub pickup:Option<usize>,pub fire:Option<usize>}
#[cfg(test)]mod tests{
 use super::*;fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
 #[test]fn all_512_executed_native_queue_sorts_and_palette_switch_sequences_match(){
  let mut hash=2166136261u32;
  for s in 0..512usize{let count=if s==0{0}else{1+s%17};let mode=s%3;let forced=s/3%2!=0;let mut palette=Palette{current:[0,4,255][s/6%3],uploaded:[0,1,4,255][s/18%4]};let mut records=(0..count).map(|i|{let y=match s%4{0=>(i%4)as i16,1=>(count-i)as i16,2=>(i*7313+s*37)as i16,_=>-100};let p=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,255][(s+i*7)%17];let mut offset=[-128,-1,0,16,127][(s+i)%5];if p==255&&offset== -1{offset=0;}Record{x:100+i as i16,y,sprite:1000+i as i16,palette:p,offset}}).collect::<Vec<_>>();sort(&mut records,|r|r.y);let mut bytes=2166136261u32;mix(&mut bytes,records.iter().flat_map(|r|r.bytes()).map(u32::from));let(mut trace,mut uploads,mut draws)=(2166136261u32,0,0);for r in records{if palette.plain(r.palette,forced){uploads+=1;mix(&mut trace,[0,u32::from(palette.current)]);}draws+=1;mix(&mut trace,[if mode==0{1}else{2},r.x as u32,r.y.wrapping_add(i16::from(r.offset))as u32,r.sprite as u32,u32::from(palette.current)]);}mix(&mut hash,[s as u32,count as u32,bytes,u32::from(palette.current),u32::from(palette.uploaded),trace,uploads,draws]);}
  assert_eq!(hash,131070041,"real545b0/4549f0/40bb50; final graphics/texture/upload submissions are explicit boundaries");
 }
 #[test]fn equal_depths_follow_the_native_comparison_exchange_order(){let mut rows=[('A',1),('B',1),('C',0)];sort(&mut rows,|r|r.1);assert_eq!(rows,[('C',0),('B',1),('A',1)]);}
 #[test]fn all_4480_executed_normal_unit_queue_records_match_raw_animation_and_elevation(){
  let mut hash=2166136261u32;let mut scenario=0u32;
  for phase in[0,1,2,3,4,5,6,7,8,16,17,19,20,21,22,23,24,25,26,27,28,29,30,33,42,49,61,62,63,64,65,66,72,73,74]{for heading in 0..8i8{for elevation in[-128,-32,0,127i8]{for frame in[-1,0,2,32767i16]{let flags=scenario.wrapping_mul(37)as u8;let record=infantry([500,600],phase,frame,heading,elevation,flags).unwrap();let mut bytes=2166136261u32;mix(&mut bytes,record.bytes().map(u32::from));mix(&mut hash,[scenario,u32::from(phase),heading as u32,elevation as u32,frame as u32,u32::from(flags),8,bytes,u32::from(flags&!4)]);scenario+=1;}}}}
  assert_eq!(scenario,4480);assert_eq!(hash,3040814069,"actual55f10, normal sprite branch; terrain clips and specialized hardware are separate producers");
 }
 #[test]fn all_9984_executed_object_primary_queue_records_match_signed_offsets_and_palettes(){
  let mut hash=2166136261u32;let mut scenario=0u32;
  for kind in 0..78u16{let base=crate::original_world_contact::object_metadata(kind).unwrap().0 as i16;let depth=OBJECT_DEPTH[usize::from(kind)];for animation in[-1,0,4,127i8]{for offset in[-128,0,30,127i8]{for flags in[0,32,34,38,40,48,62,112u8]{let record=object(ObjectInput{position:[500,600],base,animation,sprite_offset:offset,flags,depth});let mut bytes=2166136261u32;if let Some(r)=record{mix(&mut bytes,r.bytes().map(u32::from));}mix(&mut hash,[scenario,u32::from(kind),base as u32,depth as u32,animation as u32,offset as u32,u32::from(flags),if record.is_some(){8}else{0},bytes]);scenario+=1;}}}}
  assert_eq!(scenario,9984);assert_eq!(hash,3811278845,"actual43f260 primary rows; shadow submission, lifetime admission and119 extra wreck-overlay cases are explicit boundaries");
 }
 #[test]fn queue_elevation_moves_only_depth_and_cancels_at_actual_draw_position(){let r=infantry([500,600],23,0,0,-32,16).unwrap();assert_eq!((r.y,r.offset,r.y+i16::from(r.offset)),(568,32,600));let r=object(ObjectInput{position:[500,600],base:352,animation:0,sprite_offset:0,flags:112,depth:-18}).unwrap();assert_eq!((r.y,r.palette,r.offset),(600,8,0));}
}
pub const OBJECT_DEPTH:[i8;78]=[0,0,0,0,0,-18,-25,-14,-30,127,127,0,0,0,0,0,0,0,0,0,0,-31,-23,-32,-32,-29,-29,-32,0,-30,-26,0,-29,-31,-29,-29,-32,0,0,-32,-32,-32,-20,-16,-27,-28,-29,-26,-29,-31,-31,-29,-29,-29,-31,-31,-29,-24,0,0,-27,-16,-25,-30,-30,-24,-32,0,0,0,0,-8,-8,-8,-8,-8,-8,-8];
pub const PHASES:[[i16;2];75]=[[66,1],[184,3],[74,4],[184,3],[330,1],[127,5],[119,1],[74,4],[74,4],[488,3],[488,3],[66,0],[66,0],[66,0],[66,0],[208,0],[119,1],[115,1],[66,0],[226,3],[127,5],[226,3],[66,1],[315,0],[319,0],[2467,0],[304,0],[2445,0],[2457,0],[2496,0],[2462,0],[2500,0],[1953,0],[1959,0],[1963,0],[2022,0],[2037,0],[2024,0],[2029,0],[2033,0],[2057,0],[2067,0],[66,1],[2075,0],[2087,0],[208,0],[316,0],[365,0],[2554,2],[2522,4],[1643,1],[1659,2],[5767,2],[1845,3],[5305,3],[5719,3],[5655,2],[2570,2],[2570,1],[2570,1],[2570,1],[5217,5],[5217,0],[2570,1],[5257,6],[6022,3],[2522,4],[5937,0],[6237,0],[5950,0],[6279,0],[1340,3],[6022,3],[6022,3],[1340,3]];
