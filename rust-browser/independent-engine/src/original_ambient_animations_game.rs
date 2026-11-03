//! Game binding for the one original150-record animation queue.
use super::{State,World};
use super::super::{Game,json,original_ambient_emitters as emitter};
impl Game{
 pub(crate) fn spawn_native_ambient_chunk(&mut self,kind:u8,position:[i16;2],source35:u8){self.native_ambient_animations.spawn(kind,position,source35);}
 pub(crate) fn spawn_native_ambient_water(&mut self,position:[i16;2]){self.native_ambient_animations.spawn_water(position);}
 pub(crate) fn update_native_ambient_animations(&mut self){if !self.map.original{return;}let mut state=std::mem::take(&mut self.native_ambient_animations);state.advance(self.native_view.map_or([0,0],|v|v.0),self);self.native_ambient_animations=state;}
 pub(crate) fn native_ambient_animation_visuals(&self)->Vec<serde_json::Value>{let camera=self.native_view.map_or([0,0],|v|v.0);self.native_ambient_animations.draws.iter().map(|d|{let style=super::submission(d.mode,d.palette,d.shade,d.parameter);json!({"id":d.slot,"sprite":d.sprite,"x":i32::from(camera[0].wrapping_add(d.screen[0]))*2,"y":i32::from(camera[1].wrapping_add(d.screen[1]).wrapping_add(i16::from(style.y_offset)))*2,"team":style.palette.filter(|&p|p<4),"nativeInheritedPalette":style.palette.is_none(),"nativeWhitePalette":style.palette==Some(255),"queueY":i32::from(camera[1].wrapping_add(d.screen[1]))*2,"nativeDrawX":d.screen[0],"nativeDrawY":d.screen[1],"nativeMode":d.mode,"nativePalette":d.palette,"nativeShade":d.shade,"nativeParameter":d.parameter,"scale":if style.quad{f64::from((f32::from(style.size)+16.)*0.03125)}else{1.},"rotation":f64::from((f32::from(style.angle)*2.*3.1415925f32)*0.00390625)})}).collect()}
}
impl World for Game{
 fn emit(&mut self,state:&mut State,source_slot:usize,position:[i16;2],heading:i16){let Some(source)=state.records.get(source_slot)else{return;};let kind=source.word(18);let source35=source.byte(35);let actions=emitter::plan(kind,position,heading as u8,source35,||self.original_rng.next_u32());for action in actions{match action{emitter::Action::Spawn(s)=>{if let Some(targets)=&mut self.native_ambient{targets.pool.spawn(s);}},emitter::Action::Chunk{kind,position,source35}=>{state.spawn(kind,position,source35);}}}}
}
