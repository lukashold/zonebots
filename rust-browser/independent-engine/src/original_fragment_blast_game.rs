//! Exact ownerless zero-strength entry used by41f610. Generic projectile
//! explosions retain their own adapter and are not silently reclassified.
use crate::{Game,original_fragment_blast as native};
use super::effects;
impl Game {
 pub(crate) fn native_fragment_zero_blast(&mut self,p:[i16;2]) {
    if !native::admitted(p,[(self.map.width*16)as i16,(self.map.height*16)as i16]){return;}
    self.native_blast_sound_prefix(p);
    //OBJECT, BUILD, forts and blockers are synchronous, even if all50 delayed
    //records are occupied. Strength zero still performs their source queries.
    self.native_blast_environment(i32::from(p[0]),i32::from(p[1]),0,255,false);
    let(cell,leaf)=native::terrain_leaf(native::sample(p),false,|c|self.world_code(c));
    let mut class=self.native_blast_terrain_class;let world=self.map.world;
    native::alter_terrain(world,leaf,&mut class,&mut Terrain(self));self.native_blast_terrain_class=class;
    let water=matches!(self.world_code(cell)&127,9|29|42|39|38|40);
    //Allocation is the native first-free50-slot operation, without running the
    //environment twice. UNIT contact occurs only when420be0 reaches frame3.
    if self.allocate_native_fragment_zero_blast(p,water)&&water {self.native_fragment_water_particles(p);}
 }
 ///Shared420200 sound prefix. 41f870's separate bridge caller remains an
 ///unported writer of the same cycle/gate; no second isolated cycle is used.
 pub(crate) fn native_blast_sound_prefix(&mut self,p:[i16;2]){
    if !native::admitted(p,[(self.map.width*16)as i16,(self.map.height*16)as i16]){return;}
    let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));
    if let Some(id)=native::sound(p,camera,size,self.native_blast_sound_gate,self.native_blast_quiet(),self.native_blast_sound_cycle){
        self.emit_native_sample(id,crate::original_audio::Gate::Enabled);
        self.native_blast_sound_cycle=self.native_blast_sound_cycle.wrapping_add(1)&7;self.native_blast_sound_gate=true;
    }
 }
 fn native_fragment_water_particles(&mut self,p:[i16;2]){
    let sprites=native::water_sprites(self.map.world);let mut groups=vec![(sprites[0],7,8),(sprites[1],7,8)];if self.map.world==0{groups.push((3537,10,5));}
    for(sprite,maximum,expiry)in groups{for _ in 0..4{let Some(slot)=self.native_world.as_ref().and_then(|s|s.effects.iter().position(|p|p.as_ref().is_none_or(|p|!p.active)))else{break;};
        let seed=native::water_seed(||self.native_random());let s=self.native_world.as_mut().unwrap();s.effect_owner=0;s.effect_maximum=maximum;
        s.effects[slot]=Some(effects::Particle{position:[0;3],origin:[p[0].wrapping_sub(148),p[1].wrapping_sub(68)],velocity:seed.velocity,deceleration:1,deceleration_left:3,tint:s.effect_tint,fade_left:0,fade_interval:0,lifetime:500,flags:12,phase:seed.phase,maximum_phase:maximum,sprite,angle:0,spin:seed.spin,active:true,expiry,owner:0});
    }}
 }
 ///401560/402ce0's final retirement is represented exactly. The intervening
 ///42230(count12,tint9) emitter remains an explicit cosmetic leaf boundary;
 ///its unported particle allocations/RNG are not replaced by guessed effects.
 pub(super) fn retire_native_fragment_water_particle(&mut self,slot:usize){if let Some(s)=&mut self.native_world{s.effect_tint=9;s.effect_owner=0;if let Some(p)=s.effects.get_mut(slot).and_then(Option::as_mut){p.active=false;p.position[0]=-1000;}}}
}
struct Terrain<'a>(&'a mut Game);
impl native::Terrain for Terrain<'_>{
 fn tile(&self,c:i16)->u8{if c<0{return 0;}let c=c as usize;let(x,y)=(c&127,c>>7);if x<self.0.map.width&&y<self.0.map.height{self.0.map.original_tiles.get(y*self.0.map.width+x).copied().unwrap_or(0)as u8}else{self.0.map.native_tile_plane.get(c).copied().unwrap_or(0)}}
 fn terrain(&self,c:i16)->u8{self.0.world_code(c)}
 fn set(&mut self,c:i16,w:u16){if !(0..16384).contains(&c){return;}self.0.set_world_code(c,(w>>8)as u8|22);let c=c as usize;if let Some(t)=self.0.map.native_tile_plane.get_mut(c){*t=w as u8;}let(x,y)=(c&127,c>>7);if x<self.0.map.width&&y<self.0.map.height{if let Some(t)=self.0.map.original_tiles.get_mut(y*self.0.map.width+x){*t=(w&255)|((w&0x8000)>>7);}}self.0.map.revision=self.0.map.revision.wrapping_add(1);}
 fn random(&mut self)->u32{self.0.native_random()}
}
#[cfg(test)]mod tests{
 use super::*;use crate::{json,original_combat::Attack};
 fn field()->Game{let mut g=Game::new(0,0,41610);g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![66;4096];g.map.native_terrain_plane=vec![66;16384];g.map.original_tiles=vec![0;4096];g.map.native_tile_plane=vec![0;16384];g.map.territory=vec![0;4096];g.map.bridge_base=vec![-1;4096];g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();g.native_world=None;g.native_strategy=None;g.native_group_rosters=Default::default();g.ai_teams.clear();g.import_native_world(&json!({}));g.native_feedback.sound_enabled=1;g.native_feedback.ui_clock=100;g.native_view=Some(([0,0],[640,600]));g}
 #[test]fn zero_strength_fragment_allocates_ownerless_record_and_reaches_real_frame_three_easy_damage(){
    for kind in["grunt","medium"]{for team in 0..2{let mut g=field();g.spawn(kind,team,1024.,1040.,1,None);g.maintain_native_group_rosters();g.units[0].native_flags64=(team as u8)|16;g.units[0].native_mask98=1;g.units[0].native_flags9c=0;let hp=g.units[0].hp;g.native_fragment_zero_blast([500,500]);assert_eq!(g.blasts.len(),1);assert_eq!((g.blasts[0].team,g.blasts[0].strength,g.blasts[0].frame),(255,0,0));assert_eq!((g.units[0].hp,g.units[0].native_flags64&4,g.units[0].native_mask98),(hp,0,1));
        for frame in 1..=3{g.motion_tick=frame*2;g.update_native_blasts();assert_eq!(g.blasts[0].frame,frame as u8);if frame<3{assert_eq!(g.units[0].hp,hp);assert_eq!(g.units[0].native_flags64&4,0);}}
        assert_eq!(g.units[0].native_flags64&4,4);assert_eq!(g.units[0].native_mask98,7);if team==0{assert_eq!(g.units[0].hp,hp-1.);}else{assert_eq!(g.units[0].hp,hp);}
    }}
 }
 #[test]fn full_delayed_pool_keeps_synchronous_zero_strength_terrain_and_sound(){let mut g=field();for _ in 0..50{assert!(g.allocate_native_fragment_zero_blast([100,100],false));}let c=native::sample([500,500])+128;g.set_world_code(c,0);let j=(c as usize>>7)*64+(c as usize&127);g.map.original_tiles[j]=105;let mut rng=g.original_rng.clone();let expected=native::words(0)[(rng.next_u32()&7)as usize];g.native_fragment_zero_blast([500,500]);assert_eq!(g.blasts.len(),50);assert_eq!(g.map.original_tiles[j],(expected&255)|((expected&0x8000)>>7));assert_eq!(g.world_code(c),((expected>>8)as u8)|22);assert_eq!(g.original_rng.words,rng.words);assert_eq!(g.native_blast_sound_cycle,1);assert!(g.native_blast_sound_gate);}
 #[test]fn fragment_and_normal_explosion_share_the_source_sound_gate_and_next_callback_reset(){let mut g=field();let a=Attack{target:0,damage:1,aim:0,facing:0,bullet:false};g.spawn_native_blast(100,100,a,0,None);g.native_fragment_zero_blast([200,200]);assert_eq!(g.native_blast_sound_cycle,1);assert_eq!(g.events.iter().filter(|e|e["kind"]=="nativeSample").count(),1);assert!(g.events.iter().filter(|e|e["kind"]=="explosion").all(|e|e["silent"]==true));
    g.spawn("grunt",0,100.,100.,1,None);g.spawn("grunt",1,1600.,1600.,2,None);for u in &mut g.units{u.native_flags9c|=4;}g.step(0.06);assert!(!g.native_blast_sound_gate,"435b07 resets before producer/UNIT callbacks");g.native_fragment_zero_blast([300,300]);assert_eq!(g.native_blast_sound_cycle,2);assert_eq!(g.events.iter().filter(|e|e["kind"]=="nativeSample").count(),2,"source callback reset admits the second descriptor");assert_eq!(g.events.iter().filter(|e|e["kind"]=="nativeSample").last().unwrap()["sampleId"],2);}
 #[test]fn water_child_constructors_spend_rng_only_for_successful_effect_slots_and_retire_without_inventing_the_unported_emitter(){let mut g=field();for c in &mut g.map.terrain_codes{*c=9;}g.map.native_terrain_plane.fill(9);let mut rng=g.original_rng.clone();for _ in 0..12*5{rng.next_u32();}g.native_fragment_zero_blast([500,500]);assert_eq!(g.original_rng.words,rng.words);assert!(g.blasts[0].water);let s=g.native_world.as_ref().unwrap();assert_eq!(s.effects.iter().flatten().filter(|p|p.active).count(),12);assert_eq!(s.effects[0].as_ref().unwrap().expiry,8);assert_eq!(s.effects[8].as_ref().unwrap().expiry,5);let before=g.original_rng.words;g.retire_native_fragment_water_particle(0);assert!(!g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap().active);assert_eq!(g.native_world.as_ref().unwrap().effect_tint,9);assert_eq!(g.original_rng.words,before,"42230/count12 remains an explicit unexecuted cosmetic/RNG boundary");
    let mut g=field();let mut e=g.native_world.as_ref().unwrap().effects.clone();let template=effects::initialize(effects::Kind::Hit,[0,0],0,||0);for p in &mut e{*p=Some(template.clone());}g.native_world.as_mut().unwrap().effects=e;g.map.terrain_codes.fill(9);g.map.native_terrain_plane.fill(9);let before=g.original_rng.words;g.native_fragment_zero_blast([500,500]);assert_eq!(g.original_rng.words,before);assert_eq!(g.blasts.len(),1);
 }
}
