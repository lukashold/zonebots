//!41f520's four native slots remain separate from compact BUILD ordinals.
use crate::{Game,original_building_fragments as native,original_display::Record};
use super::fort_fragments;
impl Game{
 pub(super) fn initialize_native_fort_fragments(&mut self){let Some(w)=&mut self.native_world else{return;};let Some(t)=&w.fort_fragments.templates else{return;};for(slot,fort)in w.forts.iter().take(2).enumerate(){if let Some(b)=self.buildings.iter().find(|b|b.id==fort.binding){fort_fragments::construct(&mut w.fort_fragments.pools[slot],fort.base,t,b.hp as i16);}}}
 pub(crate) fn native_fort_fragment_draws(&self)->Vec<(Record,usize,usize)>{self.native_world.as_ref().map_or_else(Vec::new,|w|w.fort_fragments.draws.clone())}
 ///Main435dfa follows ambient100/lava and precedes407af0 and the sort.
 pub(crate) fn advance_native_fort_fragments(&mut self){if !self.map.original||self.native_world.as_ref().is_none_or(|w|w.fort_fragments.templates.is_none()){return;}let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));let view=native::View{camera,size,multiplayer:self.multiplayer};for slot in 0..4{let Some(w)=&mut self.native_world else{return;};let damage=w.forts.get(slot).and_then(|f|self.buildings.iter().find(|b|b.id==f.binding)).map_or(0,|b|(b.max_hp as i16).wrapping_sub(b.hp as i16));let mut pool=w.fort_fragments.pools[slot].clone();let mut env=Env{game:self,camera,slot,record:0,contact:false};pool.advance(damage,view,&mut env);if let Some(w)=&mut env.game.native_world{w.fort_fragments.pools[slot]=pool;}}}
}
struct Env<'a>{game:&'a mut Game,camera:[i16;2],slot:usize,record:usize,contact:bool}
impl native::World for Env<'_>{
 fn contact_mode(&mut self,v:bool){self.contact=v;}
 fn select_record(&mut self,v:usize){self.record=v;}
 fn submit(&mut self,mut r:Record){r.x=r.x.wrapping_add(self.camera[0]);r.y=r.y.wrapping_add(self.camera[1]);if let Some(w)=&mut self.game.native_world{w.fort_fragments.draws.push((r,self.slot*50+self.record,self.slot));}}
 fn environment(&mut self,p:[i16;2]){debug_assert!(self.contact);self.game.native_fragment_zero_blast(p);}
 fn spark(&mut self,p:[i16;2]){let r=if self.game.multiplayer{0}else{self.game.native_cosmetic_random()};self.game.native_ambient_animations.spawn_spark(p,r);}
 fn crt(&mut self)->u16{self.game.native_cosmetic_random()}
}
#[cfg(test)]#[path="original_fort_fragments_game_tests.rs"]mod tests;
