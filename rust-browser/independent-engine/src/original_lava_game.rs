use super::{Game,json,original_ambient::LavaSeed,original_lava as lava,original_reactions as reaction,original_combat};
impl Game{
 pub(crate) fn native_cosmetic_random(&mut self)->u16{self.native_cosmetic_rng=self.native_cosmetic_rng.wrapping_mul(214013).wrapping_add(2531011);((self.native_cosmetic_rng>>16)&32767)as u16}

 pub(crate) fn spawn_native_lava(&mut self,seed:LavaSeed){if !self.map.original{return;}let native_world=self.map.wall_tile_lookup["nativeWorld"].as_u64().map_or([0,3,1,4,2][self.map.world.min(4)],|n|n as u8);self.native_lava.spawn(seed.position,seed.template,seed.size,seed.interval,native_world,true);}
 pub(crate) fn native_lava_near(&self,point:[i16;2])->bool{self.native_lava.near(point)}
 pub(crate) fn update_native_lava(&mut self){if !self.map.original{return;}let mut state=std::mem::take(&mut self.native_lava);state.advance(self);self.native_lava=state;}
 pub(crate) fn native_lava_visuals(&self)->Vec<serde_json::Value>{self.native_lava.draws.iter().map(|d|{let style=super::original_ambient_animations::submission(d.mode,d.palette,d.offset as u8,d.parameter);let quad=style.quad;json!({"id":d.slot,"sprite":d.sprite,"x":i32::from(d.position[0])*2,"y":i32::from(d.position[1].wrapping_add(i16::from(d.offset)))*2,"queueY":i32::from(d.position[1])*2,"team":style.palette.filter(|&p|p<4),"nativeInheritedPalette":style.palette.is_none(),"nativeWhitePalette":style.palette==Some(255),"nativeMode":d.mode,"nativePalette":d.palette,"nativeParameter":d.parameter,"scale":if quad{f64::from((-f32::from(d.palette)+16.)*0.03125)}else{1.},"rotation":if d.mode==49{f64::from((255.-f32::from(d.parameter as i8))*2.*3.1415925f32/256.)}else{0.}})}).collect()}
 fn scare_native_lava(&mut self,point:[i16;2]){
  if self.native_random()&127>=5{return;}
  let mut candidates=self.units.iter().enumerate().filter_map(|(i,u)|u.native_slot.map(|slot|(slot,i))).collect::<Vec<_>>();candidates.sort_unstable();
  for(slot,i)in candidates{let u=self.units[i].clone();let flags=self.native_group_rosters.members.get(slot).and_then(Option::as_ref).map_or(0,|m|m.flags31);let inside=u.native_transport.as_ref().is_some_and(|t|t.phase==super::original_transport::Phase::Inside);let p=[if inside{10000}else{(u.x/2.).floor()as i16},(u.y/2.).floor()as i16];let dx=i32::from(p[0])-i32::from(point[0]);let dy=i32::from(p[1])-i32::from(point[1]);if flags&4==0||matches!(u.native_class7c,1|2)||!matches!(u.native_phase35,0|1)||dx*dx+dy*dy>=416{continue;}
   let prevented=self.native_cover_prevented(i);let terrain=self.native_terrain(i32::from(p[0])+5,i32::from(p[1])+11);if let Some(cover)=reaction::begin_cover(original_combat::facing(u.angle)*2,terrain,prevented){let v=&mut self.units[i];v.native_frame32=cover.frame32;v.native_cover=Some(cover);v.native_phase35=43;v.native_idle=None;v.native_walk=None;v.native_fire=None;v.moving=false;}
  }
 }
}
impl lava::World for Game{
 fn random(&mut self)->u32{self.native_random()}
 fn cosmetic_random(&mut self)->u32{u32::from(self.native_cosmetic_random())}
 fn multiplayer(&self)->bool{self.multiplayer}
 fn view(&self)->lava::View{let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));lava::View{camera,lower:[0,0],upper:size}}
 fn bounds(&self)->[i16;2]{[(self.map.width*16)as i16,(self.map.height*16)as i16]}
 fn native_world(&self)->u8{self.map.wall_tile_lookup["nativeWorld"].as_u64().map_or([0,3,1,4,2][self.map.world.min(4)],|n|n as u8)}
 fn resource_present(&self)->bool{true}
 fn terrain(&self,cell:i16)->u8{let i=cell as u16 as usize;let(x,y)=(i&127,i>>7);let raw=self.map.native_terrain_plane.get(i).copied().unwrap_or(127);if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(127)&127|raw&128}else{raw}}
 fn scare(&mut self,point:[i16;2]){self.scare_native_lava(point);}
 fn burst(&mut self,point:[i16;2]){self.native_lava_burst(point);}
 fn sound(&mut self,chance:lava::SoundChance){self.emit_native_lava_audio(chance);}
}

#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,17);g.map.original=true;g.map.world=1;g.units.clear();g.native_view=Some(([0,0],[320,200]));g.native_lava=Default::default();g.spawn_native_lava(LavaSeed{position:[150,160],template:0,size:2,interval:128});g}
 #[test]fn visible_volcanic_actor_uses_separate_crt_random_stream(){let mut g=field();let rng=g.original_rng.words;g.update_native_lava();assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_cosmetic_rng,2745024);assert_eq!(g.native_lava_visuals().len(),1);let mut g=field();g.multiplayer=true;let rng=g.original_rng.words;g.update_native_lava();assert_eq!(g.native_cosmetic_rng,1);assert_eq!(g.original_rng.words,rng);assert_eq!(g.native_lava_visuals().len(),1);}
 #[test]fn offscreen_volcanic_actor_does_not_consume_cosmetic_random(){let mut g=field();g.native_view=Some(([10000,10000],[320,200]));g.update_native_lava();assert_eq!(g.native_cosmetic_rng,1);assert!(g.native_lava_visuals().is_empty());}

}
