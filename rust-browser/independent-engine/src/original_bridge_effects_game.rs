//!Bridge prefix and expiry11 bindings. Both use the existing raw44 FX pool.
use crate::{Game,original_combat::OriginalRng,original_world_contact::Bridge};
use super::{State,effects::{Particle,bridge as native,spray::{Context,Pool}}};
fn globals(s:&State)->native::Globals{native::Globals{context:Context{origin:s.effect_origin,tint:s.effect_tint,flags:s.effect_flags,fade:s.effect_fade,owner:s.effect_owner,sprite:s.effect_sprite},maximum:s.effect_maximum as u8,suppression:s.effect_suppression,raw:s.bridge_fx}}
fn commit(s:&mut State,g:native::Globals){s.effect_origin=g.context.origin;s.effect_tint=g.context.tint;s.effect_flags=g.context.flags;s.effect_fade=g.context.fade;s.effect_owner=g.context.owner;s.effect_sprite=g.context.sprite;s.effect_maximum=g.maximum as i8;s.effect_suppression=g.suppression;s.bridge_fx=g.raw;}
struct World<'a>{state:&'a mut State,rng:&'a mut OriginalRng,limit:usize}
impl Pool for World<'_>{
 fn first_free(&self)->Option<usize>{self.state.effects.iter().take(self.limit).position(|p|p.as_ref().is_none_or(|p|!p.active||p.position[0]==-1000))}
 fn load(&mut self,slot:usize)->[u8;44]{self.state.effects[slot].as_ref().map_or([0;44],Particle::bytes)}
 fn store(&mut self,slot:usize,record:[u8;44]){self.state.put_effect(slot,Particle::from_bytes(record));}
 fn random(&mut self)->u32{self.rng.next_u32()}
}
impl native::World for World<'_>{fn retire(&mut self,slot:usize){if let Some(p)=&mut self.state.effects[slot]{p.active=false;p.position[0]=-1000;}}}
impl Game{
 pub(super) fn native_bridge_particle_prefix(&mut self,state:&mut State,bridge:Bridge,orientation:i16,half:bool){
  let Some(sprites)=native::engine_sprites(self.map.world)else{return;};self.original_rng.multiplayer=self.multiplayer;let mut g=globals(state);let limit=state.effects.len();native::prefix(&mut World{state,rng:&mut self.original_rng,limit},&mut g,bridge.position,bridge.size,orientation,half,sprites.prefix);commit(state,g);
 }
 pub(super) fn finish_native_bridge_expiry(&mut self,slot:usize){
  let Some(sprites)=native::engine_sprites(self.map.world)else{return;};let Some(state)=&mut self.native_world else{return;};if state.effects.get(slot).and_then(Option::as_ref).is_none(){return;}self.original_rng.multiplayer=self.multiplayer;let mut g=globals(state);let projection=state.last_projection;let limit=state.effects.len();native::expiry(&mut World{state,rng:&mut self.original_rng,limit},&mut g,slot,projection,sprites.children);commit(state,g);
 }
}
#[cfg(test)]#[path="original_bridge_effects_game_tests.rs"]mod tests;
