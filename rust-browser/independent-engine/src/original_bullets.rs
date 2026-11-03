//! Native bullet pool00464af0 and tick004634a0. Positions and step lengths are
//! original integer pixels; snapshots expose the doubled browser coordinates.
use super::{Game,Unit,Fx,stats,workshops,original_tank_origin_offset,
    original_combat::{self,Attack},original_fire::Mode,native_math::{self,Line},native_weapon_data as data};
use serde::Serialize;

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Bullet {
    pub id:usize,pub x:f64,pub y:f64,pub team:i32,pub laser:bool,pub phase:u8,pub angle:f64,
    #[serde(skip)] line:Line,
    #[serde(skip)] origin:(i32,i32),
    #[serde(skip)] maximum_range:i32,
    #[serde(skip)] speed:i32,
    #[serde(skip)] attack:Attack,
}
impl Game {
    pub(crate) fn native_random(&mut self)->u32 {
        self.original_rng.multiplayer=self.multiplayer;self.original_rng.next_u32()
    }
    pub(crate) fn native_scatter(&mut self,shooter:&Unit,target:&Unit,mode:Mode)->(i32,i32) {
        let (sx,sy)=((shooter.x/2.).floor()as i32,(shooter.y/2.).floor()as i32);
        let mut tx=(target.x/2.).floor()as i32;let mut ty=(target.y/2.).floor()as i32;
        if original_combat::runtime_class(&target.kind)==2{tx+=8;}
        let aim=i32::from(shooter.original_combat.as_ref().map(|s|s.aim_skill).unwrap_or(60));
        let effective=(aim+(aim>>2)).min(127);
        let distance_squared=i64::from(sx-tx).pow(2)+i64::from(sy-ty).pow(2);
        let mut amplitude=if matches!(mode,Mode::Rifle|Mode::Rapid) {
            let d=native_math::isqrt((distance_squared as u64)>>8);
            let range_control=((stats(&shooter.kind).range/2.)as i32>>3)*2;
            if range_control<(d>>1)+d*2 {(127-effective)>>3}else{0}
        }else{(128-effective)>>2};
        if native_math::isqrt((distance_squared as u64)>>6)>6 {
            amplitude=amplitude.max(1);let divisor=(65535/amplitude).max(1);
            let radius=(self.native_random()&65535)as i32/divisor;
            let angle=(self.native_random()&511)as i32;
            tx+=(native_math::sine(angle)*radius)>>10;
            ty+=(native_math::sine(angle+128)*radius)>>10;
        }
        (tx,ty)
    }
    pub(crate) fn refresh_native_aim(&mut self,index:usize,target:&Unit) {
        let shooter=self.units[index].clone();let mode=shooter.native_fire.as_ref().unwrap().mode;
        let mut frozen=None;
        if mode!=Mode::Hardware {
            let aim=shooter.original_combat.as_ref().map(|s|s.aim_skill).unwrap_or(60);
            if !original_combat::aim_roll_hits(aim as i8,false,self.native_random()) {
                let point=self.native_scatter(&shooter,target,mode);
                // Native negation of target slot0 remains nonnegative, so its
                // release does the second current-position scatter as well.
                if target.native_slot!=Some(0){frozen=Some(point);}
            }
        }
        self.units[index].native_fire.as_mut().unwrap().frozen_aim=frozen;
    }
    pub(crate) fn launch_native_bullet(&mut self,index:usize,target:&Unit,attack:Attack)->bool {
        let Some(id)=(0..200).find(|id|!self.bullets.iter().any(|b|b.id==*id))else{return false;};
        let u=self.units[index].clone();
        let mode=u.native_fire.as_ref().map(|s|s.mode).unwrap_or(Mode::Hardware);
        let point=u.native_fire.as_ref().and_then(|s|s.frozen_aim).unwrap_or_else(||self.native_scatter(&u,target,mode));
        let body=usize::from(original_combat::facing(u.angle));let aim=usize::from(original_combat::facing(u.aim));
        let (x,y)=((u.x/2.).floor()as i32,(u.y/2.).floor()as i32);
        let (sx,sy)=match u.kind.as_str() {
            "laser"=>(x+8,y),
            "jeep"=>(x+data::JEEP_BODY[body][0]+data::JEEP_MUZZLE[aim][0],y+data::JEEP_BODY[body][1]+data::JEEP_MUZZLE[aim][1]),
            "gatling"=>(x+data::GUN_MUZZLE[aim][0],y+data::GUN_MUZZLE[aim][1]),
            _=>(x+8,y+8),
        };
        let(tx,ty)=(point.0+8,point.1+8);
        if(sx,sy)==(tx,ty){return false;}
        let range=((stats(&u.kind).range/2.)as i32>>3)+1;
        self.bullets.push(Bullet{id,x:f64::from(sx)*2.,y:f64::from(sy)*2.,team:u.team,laser:u.kind=="laser",phase:0,angle:native_math::projectile_angle(tx-sx,ty-sy),
            line:Line::new(sx,sy,tx,ty),origin:(sx,sy),maximum_range:range*8,speed:15,attack});true
    }
    fn bullet_contact(&mut self,b:&Bullet)->bool {
        let (x,y)=(b.line.x,b.line.y);
        if x<0||y<0||x>=self.map.width as i32*16||y>=self.map.height as i32*16{return true;}
        let j=(y/16)as usize*self.map.width+(x/16)as usize;
        let code=self.map.terrain_codes.get(j).copied().unwrap_or(0)&127;
        if data::BULLET_BLOCK.get(usize::from(code))==Some(&255){return true;}
        // Destructible object footprints are an additional native contact pass,
        // separate from the terrain flags. Bullets stop without demolishing stone.
        if self.walls.iter().any(|w|w.hp>0.&&b.x>=w.x&&b.x<=w.x+w.w&&b.y>=w.y&&b.y<=w.y+w.h){return true;}
        let coarse=((y-8)>>7)*16+((x-8)>>7);
        // 00499f5c: the original nine coarse spatial buckets, in scan order.
        for offset in COARSE_OFFSETS {
            let cell=coarse+i32::from(offset);if !(0..256).contains(&cell){continue;}
            let mut candidates=self.units.iter().enumerate().filter(|(_,u)|u.team>=0&&u.team!=b.team
                &&original_combat::alive(true,u.hp)&&!workshops::hidden(u)
                &&(((u.y/2.).floor()as i32>>7)*16+((u.x/2.).floor()as i32>>7))==cell)
                .map(|(i,u)|(u.native_slot.unwrap_or(u.id),i)).collect::<Vec<_>>();
            candidates.sort_unstable();
            for (_,i) in candidates {
                let u=&self.units[i];let extent=if original_combat::runtime_class(&u.kind)==1{16}else{8};
                if ((u.x/2.).floor()as i32-x+8).abs()<=extent&&((u.y/2.).floor()as i32-y+8).abs()<=extent {
                    let mut attack=b.attack;attack.target=u.id;self.apply_native_contact(attack);return true;
                }
            }
        }
        false
    }
    pub(crate) fn update_native_bullets(&mut self) {
        let mut pending=std::mem::take(&mut self.bullets);pending.sort_by_key(|b|b.id);
        for mut b in pending {
            b.line.advance(b.speed);b.x=f64::from(b.line.x)*2.;b.y=f64::from(b.line.y)*2.;
            if b.line.remaining==0 {
                // 004634a0 checks the endpoint, extends the same ray16 pixels,
                // checks once more, and then retires the slot unconditionally.
                self.bullet_contact(&b);b.line.extend(16);b.line.advance(b.speed);
                b.x=f64::from(b.line.x)*2.;b.y=f64::from(b.line.y)*2.;self.bullet_contact(&b);continue;
            }
            let d=i64::from(b.line.x-b.origin.0).pow(2)+i64::from(b.line.y-b.origin.1).pow(2);
            if d>i64::from(b.maximum_range).pow(2)||self.bullet_contact(&b){continue;}
            b.phase^=1;self.bullets.push(b);
        }
    }
    pub(crate) fn apply_native_contact(&mut self,attack:Attack) {
        if let Some(i)=self.units.iter().position(|u|u.id==attack.target&&original_combat::alive(true,u.hp)&&!workshops::hidden(u)) {
            let mut rng=self.original_rng.clone();rng.multiplayer=self.multiplayer;
            let occupied=self.units[i].team>=0&&self.units[i].crew.is_some();
            let crew_killed=original_combat::apply_attack_with_mode(&mut self.units[i],attack,self.difficulty,self.multiplayer,||rng.next_u32());
            if crew_killed&&occupied {
                let(dx,dy)=original_tank_origin_offset(&self.units[i].kind,false);
                self.units[i].x+=dx;self.units[i].y+=dy;self.units[i].native_slot=None;
            }
            self.original_rng=rng;
            if self.units[i].hp<0. {
                let u=&self.units[i];let radius=if stats(&u.kind).cls==0{12.}else{30.};
                self.effects.push(Fx{x:u.x,y:u.y,age:0.,life:0.8,kind:"explosion".into(),radius,x2:0.,y2:0.});
                self.events.push(serde_json::json!({"kind":"explosion","x":u.x,"y":u.y,"radius":radius}));
            }
        }
    }
}

// Filled and independently checked by the local EXE table validator.
const COARSE_OFFSETS:[i16;9]=[0,-1,1,-17,-16,-15,15,16,17];
#[cfg(test)]
mod tests {
 use super::*;
 use crate::json;
 fn field()->Game {
  let mut g=Game::new(0,1,7291);g.map.original=true;g.map.world=0;
  g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
  g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.walls.clear();g.objects.clear();
  g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.next_id=1;g.winner=None;g
 }
 fn arena()->Game {
  let mut g=field();g.spawn("sniper",0,320.,320.,1,None);g.spawn("sniper",1,560.,320.,2,None);
  for u in &mut g.units{u.hold_position=true;u.cooldown=1000.;}g.units[0].cooldown=0.;
  let(a,b)=(g.units[0].id,g.units[1].id);g.command(&json!({"kind":"attack","ids":[a],"target":b}));g
 }
 fn ticks(g:&mut Game,n:usize){for _ in 0..n{g.step(0.059);}}
 #[test]
 fn rifle_windup_then_real_flight_then_contact_are_distinct_native_events() {
  let mut g=arena();ticks(&mut g,12);assert_eq!(g.units[1].hp,50.);assert!(g.bullets.is_empty());
  ticks(&mut g,1);assert_eq!(g.units[1].hp,50.);assert_eq!(g.bullets.len(),1);assert_eq!(g.bullets[0].x,366.);
  ticks(&mut g,6);assert_eq!(g.units[1].hp,50.);ticks(&mut g,1);assert_eq!(g.units[1].hp,40.);assert!(g.bullets.is_empty());
 }
 #[test]
 fn a_relocated_target_is_not_hit_by_a_homing_id_after_release() {
  let mut g=arena();ticks(&mut g,13);g.units[1].y=700.;g.units[0].cooldown=1000.;ticks(&mut g,15);
  assert_eq!(g.units[1].hp,50.);assert!(g.bullets.is_empty());
 }
 #[test]
 fn flying_bullet_contacts_an_intervening_enemy_before_the_assigned_target() {
  let mut g=arena();g.spawn("sniper",1,480.,320.,3,None);g.units[2].cooldown=1000.;g.units[2].hold_position=true;
  ticks(&mut g,20);assert_eq!(g.units[1].hp,50.);assert_eq!(g.units[2].hp,40.);
 }
 #[test]
 fn a_new_terrain_obstruction_stops_a_bullet_already_in_flight() {
  let mut g=arena();ticks(&mut g,13);let j=10*g.map.width+14;g.map.tiles[j]=2;g.map.terrain_codes[j]=19;
  ticks(&mut g,15);assert_eq!(g.units[1].hp,50.);assert!(g.bullets.is_empty());
 }
 #[test]
 fn bullet_survives_its_shooter_and_keeps_its_original_faction_and_attributes() {
  let mut g=arena();g.spawn("gatling",0,1500.,1000.,3,None);ticks(&mut g,13);g.units[0].hp=-1.;ticks(&mut g,7);
  assert_eq!(g.units.iter().find(|u|u.team==1).unwrap().hp,40.);
 }
 #[test]
 fn pool_is_shared_by_both_parties_and_has_exactly_two_hundred_slots() {
  let mut g=arena();let u=g.units[0].clone();let v=g.units[1].clone();
  for i in 0..200{assert!(g.launch_native_bullet(i%2,if i%2==0{&v}else{&u},Attack::from_unit(if i%2==0{&u}else{&v},1,10.,0.)));}
  assert_eq!(g.bullets.len(),200);assert!(!g.launch_native_bullet(0,&v,Attack::from_unit(&u,1,10.,0.)));
  g.bullets.remove(73);assert!(g.launch_native_bullet(0,&v,Attack::from_unit(&u,1,10.,0.)));assert_eq!(g.bullets.last().unwrap().id,73);
 }
 #[test]
 fn native_scheduler_does_not_process_any_callback_before_the_59ms_threshold() {
  let mut g=arena();g.step(0.058);assert_eq!(g.time,0.);assert!(g.units[0].native_fire.is_none());
  g.step(0.001);assert_eq!(g.time,1./17.);assert_eq!(g.units[0].native_fire.as_ref().unwrap().frame,0);
  g.step(0.059);assert_eq!(g.time,2./17.);assert_eq!(g.units[0].native_fire.as_ref().unwrap().frame,0);
 }
}
