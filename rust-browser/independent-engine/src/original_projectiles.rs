//! Heavy-projectile pool (50 slots), native integer flight, and independent
//! blast pool (50 slots). A release, an impact, and a damage pulse are different
//! callbacks. Numeric rules were checked against the locally owned executable.
use serde::Serialize;
use super::{Game,Unit,stats,workshops,native_math::{self,Line},native_weapon_data as data,
    original_combat::{self,Attack},original_fire::Mode};

const CURVES:[&[i32];5]=[&[16],&[16,12,16],&[16,12,8,12,16],
    &[16,12,8,4,8,12,16],&[16,12,8,4,0,4,8,12,16]];
const BLAST_BUCKETS:[i32;9]=[-17,-16,-15,-1,0,1,15,16,17];

#[derive(Clone,Copy,PartialEq,Eq,Serialize,Debug)]
#[serde(rename_all="camelCase")]
pub enum Kind {Shell,Grenade,Rocket,Flame,Howitzer}
#[derive(Clone)]
struct Flight {
    line:Line,origin:(i32,i32),target:(i32,i32),range:i32,speed:i32,
    acceleration:i32,elevation:i32,curve:usize,segment:i32,segment_length:i32,phase:usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field()->Game{
        let mut g=Game::new(0,1,7291);g.map.original=true;g.map.world=0;
        g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
        g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.walls.clear();g.objects.clear();
        g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.next_id=1;g.winner=None;g
    }
    #[test]
    fn phase65_build_release_keeps_the_original_zero_length_rejection(){
        let mut g=field();g.spawn("pyro",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);
        let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=32;
        assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,500],600)));
        g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=32;g.units[0].native_order5c=-5;g.units[0].native_phase35=65;g.units[0].native_frame32=0;g.units[0].native_timer3c=1;
        for _ in 0..8{g.fire_native_ground(0,(1000.,1200.),900,false);}
        assert_eq!(g.units[0].native_phase35,65);assert!(g.units[0].native_fire.is_none());
        assert!(g.units[0].cooldown>0.,"the real windup reached a release attempt");
        assert!(g.projectiles.is_empty(),"56840 uses the same raw54/56 origin and selected target, so464a20 rejects the shot");
    }
    #[test]
    fn ground_fire_waits_for_the_real_phase_selector_without_acquiring_a_surrogate_enemy(){
        let mut g=field();g.spawn("pyro",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);let group=g.native_group_rosters.current_roster.unwrap();g.native_group_rosters.pool[group].roster[59]=255;let rng=g.original_rng.words;let activity=g.units[0].group_activity_ticks;let count=g.units.len();
        for _ in 0..20{assert!(!g.fire_native_ground(0,(1000.,1200.),900,false));}
        assert_eq!(g.units.len(),count);assert_eq!(g.units[0].native_phase35,0);assert!(g.units[0].native_fire.is_none());assert_eq!(g.native_group_rosters.pool[group].roster[59],255);assert_eq!(g.units[0].group_activity_ticks,activity);assert_eq!(g.original_rng.words,rng);assert!(g.native_attack_reports.is_empty());assert!(g.projectiles.is_empty());
    }
    #[test]
    fn ground_flame_cycle_uses_persistent_a0_without_an_old_fire_state_or_enemy_reports(){
        let mut g=field();g.spawn("pyro",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,500],600)));let slot=g.units[0].native_slot.unwrap();g.native_group_rosters.members[slot].as_mut().unwrap().flags69|=32;g.units[0].native_order5c=-5;g.units[0].native_phase35=65;g.units[0].native_frame32=2;g.units[0].native_timer3c=0;g.units[0].native_cyclea0=3;g.units[0].original_combat.as_mut().unwrap().fire_rate=80;let rng=g.original_rng.words;
        assert_eq!(g.update_native_ground_flame(0),Some(false));assert_eq!((g.units[0].native_frame32,g.units[0].native_timer3c,g.units[0].native_cyclea0),(1,127,0));assert!(g.units[0].native_fire.is_none());assert_eq!(g.units[0].native_order5c,-5);assert_eq!(g.original_rng.words,rng);assert!(g.native_attack_reports.is_empty());
        g.units[0].native_frame32=2;g.units[0].native_timer3c=0;g.units[0].native_cyclea0=2;assert_eq!(g.update_native_ground_flame(0),Some(false));assert_eq!((g.units[0].native_frame32,g.units[0].native_timer3c,g.units[0].native_cyclea0),(0,1,3));
    }
    #[test]
    fn ground_flame_lost_goal_clears_group_order_then_paths_and_runs_the_real_idle_reset(){
        let mut g=field();g.spawn("pyro",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,500],600)));let group=g.native_group_rosters.current_roster.unwrap();g.units[0].native_order5c=-5;g.units[0].native_phase35=65;g.units[0].native_frame32=2;g.units[0].native_timer3c=0;g.units[0].native_cyclea0=1;g.units[0].path=std::collections::VecDeque::from([(1200.,1200.)]);g.map.terrain_codes.fill(0);g.map.native_terrain_plane.fill(0);let mut rng=g.original_rng.clone();for _ in 0..3{rng.next_u32();}
        assert_eq!(g.update_native_ground_flame(0),Some(false));assert_eq!(g.units[0].native_cyclea0,2);assert_eq!(g.units[0].native_order5c,0);assert_eq!(g.native_group_rosters.pool[group].roster[58],0);assert_eq!(g.units[0].native_phase35,1);assert!(g.units[0].path.is_empty());assert!(g.units[0].native_idle.is_some());assert_eq!(g.original_rng.words,rng.words);assert!(g.projectiles.is_empty());
    }
    #[test]
    fn real_ground_grenade_phase_releases_on_ninth_callback_and_finishes_after_last_grenade(){
        let mut g=field();g.spawn("grunt",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,500],600)));g.units[0].native_order5c=0;g.units[0].native_phase35=49;g.units[0].native_frame32=0;g.units[0].native_timer3c=3;g.units[0].grenades=1;
        for _ in 0..8{assert_eq!(g.update_native_ground_fire(0),Some(false));}assert_eq!(g.update_native_ground_fire(0),Some(true));assert_eq!(g.units[0].grenades,0);assert_eq!(g.projectiles.len(),1);assert_eq!(g.projectiles[0].kind,Kind::Grenade);assert_eq!((g.units[0].native_frame32,g.units[0].native_timer3c),(3,3));for _ in 0..3{assert_eq!(g.update_native_ground_fire(0),Some(false));}assert_eq!(g.units[0].native_phase35,1);assert!(g.units[0].native_fire.is_none());assert_eq!(g.units.len(),1);assert!(g.native_attack_reports.is_empty());
    }
    #[test]
    fn real_ground_rocket_phase_refreshes_the_raw_attack_after_its_animation(){
        let mut g=field();g.spawn("tough",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,500],600)));g.units[0].native_order5c=-5;g.units[0].native_phase35=19;g.units[0].native_frame32=0;g.units[0].native_timer3c=20;
        for _ in 0..21{assert_eq!(g.update_native_ground_fire(0),Some(false));}assert_eq!(g.update_native_ground_fire(0),Some(true));assert_eq!(g.projectiles.len(),1);assert_eq!(g.projectiles[0].kind,Kind::Rocket);for _ in 0..6{assert_eq!(g.update_native_ground_fire(0),Some(false));}assert_eq!((g.units[0].native_frame32,g.units[0].native_timer3c,g.units[0].native_order5c),(0,-2,5));let slot=g.units[0].native_slot.unwrap();assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.order,5);assert!(g.native_attack_reports.is_empty());
    }
    #[test]
    fn original_ground_entry_selects_flame_then_the_following_four_callbacks_attempt_release(){
        let mut g=field();g.map.native_road_graph=Some(crate::original_route::RoadGraph{regions:vec![crate::original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});g.map.native_terrain_plane=vec![0;16384];g.spawn("pyro",0,1000.,1000.,10,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);g.import_native_world(&serde_json::Value::Null);let cell=(500/16)*g.map.width+520/16;g.map.terrain_codes[cell]=8;g.map.native_terrain_plane[(500/16)*128+520/16]=8;g.units[0].native_range4=Some(127);g.units[0].native_cyclea0=3;assert!(g.record_native_order_event(0,crate::original_orders_game::Event::classified(5,[3456,520],500)));assert_eq!(g.dispatch_native_attack_order(0),Some(crate::original_attack_order::AttackOrderResult::Ground([520,500])));
        assert!(g.update_native_attack_phase(0));assert_eq!((g.units[0].native_phase35,g.units[0].native_frame32,g.units[0].native_timer3c,g.units[0].native_cyclea0),(65,0,1,3));assert!(g.units[0].native_fire.is_none());assert_eq!(g.units[0].cooldown,0.);for _ in 0..3{assert_eq!(g.update_native_ground_fire(0),Some(false));assert_eq!(g.units[0].cooldown,0.);}assert_eq!(g.update_native_ground_fire(0),Some(false));assert!(g.units[0].cooldown>0.);assert!(g.projectiles.is_empty());assert!(g.native_attack_reports.is_empty());assert_eq!(g.units.len(),1);
    }
    #[test]
    fn all_193_steps_match_executed_x86_flight_in_eight_octants_and_long_axis(){
        let mut hash=2166136261u32;let mut count=0;
        for rocket in [0,1]{for (dx,dy) in [(60,22),(22,60),(-60,22),(-22,60),(60,-22),(22,-60),(-60,-22),(-22,-60),(160,0)]{
            let kind=if rocket==1{Kind::Rocket}else{Kind::Shell};
            let mut f=Flight::new((500,500),(500+dx,500+dy),800,if rocket==1{10}else{8},1,kind);
            for tick in 1..=100{
                let impact=f.advance(kind,|_,_|0);count+=1;
                for n in [rocket,dx,dy,tick,f.line.x,f.line.y,f.line.remaining,f.phase as i32,f.acceleration,impact as i32]{hash=(hash^(n as u32)).wrapping_mul(16777619);}
                if impact{break;}
            }
        }}
        assert_eq!(count,193);assert_eq!(hash,3593183724,"Ghidra ZNativeProjectileOracle numeric trace");
    }
    #[test]
    fn height_contact_and_inclusive_range_do_not_follow_the_target_id(){
        let mut f=Flight::new((100,100),(300,100),20,10,1,Kind::Rocket);
        assert!(!f.advance(Kind::Rocket,|_,_|1));assert!(f.advance(Kind::Rocket,|_,_|2));
        let mut f=Flight::new((100,100),(300,100),20,10,1,Kind::Shell);
        assert!(!f.advance(Kind::Shell,|_,_|0));assert!(f.advance(Kind::Shell,|_,_|0));assert_eq!(f.line.x,120);
    }
    #[test]
    fn blast_pulses_on_the_third_alternate_callback_inclusive_rectangle_and_faction_filter(){
        let mut g=field();g.spawn("pyro",1,512.,512.,1,None);g.spawn("pyro",0,512.,512.,2,None);
        g.spawn("pyro",1,500.+26.*2.-16.,500.+18.*2.-16.,3,None);
        let a=Attack{target:0,damage:20,aim:0,facing:0,bullet:false};
        g.spawn_native_blast(250-12,250-24,a,0,None);
        for tick in 1..=5{g.motion_tick=tick;g.update_native_blasts();assert_eq!(g.units[0].hp,80.);}
        g.motion_tick=6;g.update_native_blasts();assert_eq!(g.units[0].hp,60.);assert_eq!(g.units[1].hp,80.);assert_eq!(g.units[2].hp,60.);
        for tick in 7..=24{g.motion_tick=tick;g.update_native_blasts();}assert!(g.blasts.is_empty());assert_eq!(g.units[0].hp,60.);
    }
    #[test]
    fn heavy_shell_makes_main_strength_and_two_separate_offset_blasts(){
        let mut g=field();g.spawn("heavy",0,320.,320.,1,None);g.spawn("heavy",1,480.,320.,2,None);
        let a=Attack::from_unit(&g.units[0],g.units[1].id,120.,0.);let target=g.units[1].clone();
        assert!(g.launch_native_heavy(0,&target,a,false));
        for _ in 0..100{g.update_native_projectiles();if g.projectiles.is_empty(){break;}}
        assert_eq!(g.blasts.iter().map(|b|b.strength).collect::<Vec<_>>(),[120,35,10]);assert!(g.blasts.iter().all(|b|b.frame==0));
        assert_eq!(g.units[1].hp,300.);assert!(g.blasts[0].x!=g.blasts[1].x||g.blasts[0].y!=g.blasts[1].y);
    }
    #[test]
    fn fifty_projectile_slots_are_shared_and_failed_allocation_does_not_spend_inventory(){
        let mut g=field();g.spawn("grunt",0,320.,320.,1,None);g.spawn("medium",1,480.,320.,2,None);
        g.units[0].grenades=51;let a=Attack{target:2,damage:50,aim:60,facing:2,bullet:false};let target=g.units[1].clone();
        for _ in 0..50{assert!(g.launch_native_heavy(0,&target,a,true));}assert_eq!(g.units[0].grenades,1);
        assert!(!g.launch_native_heavy(0,&target,a,true));assert_eq!(g.units[0].grenades,1);
        g.projectiles.remove(19);assert!(g.launch_native_heavy(0,&target,a,true));assert_eq!(g.projectiles.last().unwrap().id,19);assert_eq!(g.units[0].grenades,0);
    }
    #[test]
    fn missile_and_mobile_launcher_keep_their_distinct_barrels_and_salvo_sizes(){
        for(kind,expected)in [("missile",2),("launcher",3)]{
            let mut g=field();g.spawn(kind,0,320.,320.,1,None);g.spawn("medium",1,480.,320.,2,None);
            let a=Attack::from_unit(&g.units[0],2,stats(kind).damage,0.);let target=g.units[1].clone();g.launch_native_heavy(0,&target,a,false);
            assert_eq!(g.projectiles.len(),expected);assert!(g.projectiles.windows(2).all(|v|(v[0].x,v[0].y)!=(v[1].x,v[1].y)));
        }
    }
}
impl Flight {
    fn new(origin:(i32,i32),target:(i32,i32),range:i32,speed:i32,elevation:i32,kind:Kind)->Self {
        let line=Line::new(origin.0,origin.1,target.0,target.1);
        let d=(i64::from(target.0-origin.0).pow(2)+i64::from(target.1-origin.1).pow(2))as u64;
        // 00464e80 converts the square root to float before subtracting one.
        let curve=((((d>>7)as f64).sqrt()as f32-1.)as i32).clamp(0,4)as usize;
        let segment_length=line.major_length()/CURVES[curve].len()as i32;
        Self{line,origin,target,range,speed,acceleration:if matches!(kind,Kind::Rocket|Kind::Flame){1}else{0},
            elevation,curve,segment:segment_length,segment_length,phase:0}
    }
    /// Return true on contact. Range is inclusive, unlike the bullet pool.
    fn advance(&mut self,kind:Kind,height:impl FnOnce(i32,i32)->i32)->bool {
        let pixels=if self.acceleration>0{self.speed.min(self.acceleration)}else{self.speed};
        let moved=self.line.advance(pixels);self.segment-=moved;
        if self.line.remaining==0{return true;}
        let d=i64::from(self.line.x-self.origin.0).pow(2)+i64::from(self.line.y-self.origin.1).pow(2);
        if d>=i64::from(self.range).pow(2){return true;}
        let clearance=if matches!(kind,Kind::Rocket|Kind::Flame) {
            self.phase=(self.phase+1)&3;self.acceleration=(self.acceleration+1).min(15);self.elevation
        }else{
            if self.segment<0{self.phase+=1;self.segment=self.segment_length;}
            self.elevation+((16-CURVES[self.curve][self.phase.min(CURVES[self.curve].len()-1)])>>1)
        };
        height(self.line.x,self.line.y)>clearance
    }
    fn size(&self,kind:Kind)->i32 {
        if matches!(kind,Kind::Rocket|Kind::Flame){return 1;}
        let traveled=i64::from(self.line.x-self.origin.0).pow(2)+i64::from(self.line.y-self.origin.1).pow(2);
        let remaining=i64::from(self.line.x-self.target.0).pow(2)+i64::from(self.line.y-self.target.1).pow(2);
        let remaining=remaining.min(i64::from(self.range).pow(2)-traveled).max(0);
        let n=native_math::isqrt((traveled.min(remaining)as u64)>>8);
        match kind{Kind::Grenade=>n>>1,Kind::Howitzer=>n*2,_=>n}
    }
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Projectile {
    pub id:usize,pub x:f64,pub y:f64,pub team:i32,pub kind:Kind,pub sprite:usize,
    pub angle:f64,pub size:i32,
    #[serde(skip)] flight:Flight,
    #[serde(skip)] attack:Attack,
    #[serde(skip)] shooter:Option<usize>,
}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Blast {
    pub id:usize,pub x:f64,pub y:f64,pub team:i32,pub frame:u8,pub strength:u8,pub water:bool,pub sprite:usize,
    #[serde(skip)] lifetime:u8,
    #[serde(skip)] attack:Attack,
    #[serde(skip)] _shooter:Option<usize>,
}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Fire {
    pub id:usize,pub x:f64,pub y:f64,pub team:i32,pub frame:u8,pub sprite:usize,
    #[serde(skip)] base:usize,
    #[serde(skip)] lifetime:i32,
}
impl Fire{pub(crate) fn sync_native_object(&mut self,frame:u8,lifetime:i8){self.frame=frame;self.sprite=self.base+usize::from(frame);self.lifetime=i32::from(lifetime);}}
impl Game {
    #[cfg(test)]pub(crate) fn native_hardware_shot_debug(&self)->Vec<(Kind,[i16;2],[i16;2],i32,u8)>{self.projectiles.iter().map(|p|(p.kind,[p.flight.origin.0 as i16,p.flight.origin.1 as i16],[p.flight.target.0 as i16,p.flight.target.1 as i16],p.flight.speed,p.flight.elevation as u8)).collect()}
    fn native_height(&self,x:i32,y:i32)->i32 {
        let code=self.native_terrain(x,y);
        i32::from(data::TERRAIN_HEIGHT.get(code as usize).copied().unwrap_or(0))
    }
    pub(crate) fn native_terrain(&self,x:i32,y:i32)->u8 {
        if x<0||y<0{return 0;}
        let (c,r)=((x/16)as usize,(y/16)as usize);
        if c>=self.map.width||r>=self.map.height{return 0;}
        self.map.terrain_codes.get(r*self.map.width+c).copied().unwrap_or(0)&127
    }
    fn native_hardware_aim(&mut self,u:&Unit,point:(i32,i32))->(i32,i32) {
        let(mut x,mut y)=point;let sx=(u.x/2.).floor()as i32;let sy=(u.y/2.).floor()as i32;
        if native_math::isqrt(((i64::from(sx-x).pow(2)+i64::from(sy-y).pow(2))as u64)>>6)>6 {
            let aim=i32::from(u.original_combat.as_ref().map(|s|s.aim_skill).unwrap_or(60));
            let amplitude=((128-(aim+(aim>>2)).min(127))>>2).max(1);
            let radius=(self.native_random()&65535)as i32/(65535/amplitude).max(1);
            let angle=(self.native_random()&511)as i32;
            x+=(native_math::sine(angle)*radius)>>10;y+=(native_math::sine(angle+128)*radius)>>10;
        }
        (x,y)
    }
    pub(crate) fn launch_native_heavy(&mut self,index:usize,target:&Unit,mut attack:Attack,grenade:bool)->bool {
        let u=self.units[index].clone();
        let mode=u.native_fire.as_ref().map(|s|s.mode).unwrap_or(Mode::Hardware);
        let mut point=if mode==Mode::Hardware {
            self.native_hardware_aim(&u,((target.x/2.).floor()as i32+12,(target.y/2.).floor()as i32+8))
        }else{u.native_fire.as_ref().and_then(|s|s.frozen_aim).unwrap_or_else(||self.native_scatter(&u,target,mode))};
        if mode!=Mode::Hardware {if grenade{point.0+=8;point.1+=8;}else{point.1+=12;}}
        attack.bullet=false;
        self.launch_native_heavy_at(index,point,attack,grenade)
    }
    fn launch_native_heavy_at(&mut self,index:usize,point:(i32,i32),attack:Attack,grenade:bool)->bool {
        let u=self.units[index].clone();
        let aim=original_combat::facing(u.aim)as usize;let body=original_combat::facing(u.angle)as usize;
        let (x,y)=((u.x/2.).floor()as i32,(u.y/2.).floor()as i32);
        let(kind,speed,offset)=if grenade {
            let speed=match u.kind.as_str(){"grunt"|"psycho"=>3,"sniper"=>5,"pyro"|"laser"=>6,_=>1};
            (Kind::Grenade,speed,data::GRENADE_MUZZLE[aim])
        }else{match u.kind.as_str(){
            "tough"=>(Kind::Rocket,10,data::TOUGH_MUZZLE[aim]),
            "pyro"=>(Kind::Flame,13,[data::FLAME_MUZZLE[aim][0]-8,data::FLAME_MUZZLE[aim][1]-8]),
            "missile"=>(Kind::Rocket,5,data::MISSILE_A[aim]),
            "launcher"=>(Kind::Rocket,4,[data::LAUNCHER_BODY[body][0]+data::LAUNCHER_A[aim][0],data::LAUNCHER_BODY[body][1]+data::LAUNCHER_A[aim][1]]),
            "howitzer"=>(Kind::Howitzer,5,data::GUN_MUZZLE[aim]),
            "cannon"=>(Kind::Shell,10,data::GUN_MUZZLE[aim]),
            "light"|"medium"|"heavy"=>{
                let extra=match u.kind.as_str(){"medium"=>data::MEDIUM_EXTRA[aim],"heavy"=>data::HEAVY_EXTRA[aim],_=>[0,0]};
                (Kind::Shell,match u.kind.as_str(){"heavy"=>8,"light"=>12,_=>10},
                    [data::GUN_MUZZLE[aim][0]+data::TANK_TURRET[aim][0]+6+extra[0],
                     data::GUN_MUZZLE[aim][1]+data::TANK_TURRET[aim][1]-9+extra[1]])
            },
            "firetruck"=>(Kind::Flame,8,[16,8]),
            _=>return false,
        }};
        let speed=if grenade{speed}else{u.original_combat.as_ref().and_then(|c|c.projectile_speed).map(i32::from).unwrap_or(speed)};
        let origin=(x+offset[0],y+offset[1]);
        let elevation=self.native_height(x,y)+1+if stats(&u.kind).cls>0{1}else{0};
        let range=((self.native_unit_stats(&u).range/2.)as i32>>3)*8;
        let mut launches=vec![(origin,point)];
        if !grenade&&u.kind=="missile"{
            launches[0].1=(point.0+data::MISSILE_TARGET_A[aim][0],point.1+data::MISSILE_TARGET_A[aim][1]);
            launches.push(((x+data::MISSILE_B[aim][0],y+data::MISSILE_B[aim][1]),
                (point.0+data::MISSILE_TARGET_B[aim][0],point.1+data::MISSILE_TARGET_B[aim][1])));
        }else if !grenade&&u.kind=="launcher"{
            for extra in [data::LAUNCHER_B[aim],data::LAUNCHER_C[aim]]{
                // 0045a770 adds the X barrel offset to BOTH target axes.
                // Subsequent target resolver calls still consume their scatter.
                let _=self.native_hardware_aim(&u,point);
                launches.push(((origin.0+extra[0],origin.1+extra[1]),(point.0+extra[0],point.1+extra[0])));
            }
        }
        let mut launched=false;
        for (origin,target) in launches {
            if origin==target{continue;}
            let Some(id)=(0..50).find(|id|!self.projectiles.iter().any(|p|p.id==*id))else{break;};
            let flight=Flight::new(origin,target,range,speed,elevation,kind);
            self.projectiles.push(Projectile{id,x:f64::from(origin.0)*2.,y:f64::from(origin.1)*2.,team:u.team,
                kind,sprite:if kind==Kind::Grenade{1336}else if kind==Kind::Flame{6046}else{1634},
                angle:native_math::projectile_angle(target.0-origin.0,target.1-origin.1),size:0,
                flight,attack,shooter:u.native_slot});launched=true;
        }
        if launched&&grenade{self.units[index].grenades=self.units[index].grenades.saturating_sub(1);}
        launched
    }
    pub(crate) fn emit_native_projectile(&mut self,u:&Unit,origin:(i32,i32),target:(i32,i32),kind:Kind,speed:i32,range:i32,attack:Attack)->bool{
        if origin==target{return false;}
        let Some(id)=(0..50).find(|id|!self.projectiles.iter().any(|p|p.id==*id))else{return false;};
        let elevation=self.native_height((u.x/2.).floor()as i32,(u.y/2.).floor()as i32)+1;
        self.projectiles.push(Projectile{id,x:f64::from(origin.0)*2.,y:f64::from(origin.1)*2.,team:u.team,kind,
            sprite:if kind==Kind::Flame{6046}else{1622},angle:native_math::projectile_angle(target.0-origin.0,target.1-origin.1),size:1,
            flight:Flight::new(origin,target,range,speed,elevation,kind),attack,shooter:u.native_slot});true
    }
    ///464e80 / forced4648a0 from45c5c0. The already selected raw handler
    ///owns release timing; this keeps the shared50-slot allocation and flight.
    pub(crate) fn emit_native_hardware_point_projectile(&mut self,index:usize,shot:super::original_hardware_point::Shot)->Result<bool,super::original_path_contact::game::Boundary>{
        self.emit_native_hardware_shot(index,shot,i32::from(shot.speed))
    }
    fn emit_native_hardware_shot(&mut self,index:usize,shot:super::original_hardware_point::Shot,speed:i32)->Result<bool,super::original_path_contact::game::Boundary>{
        use super::original_path_contact::game::Boundary;
        let u=self.units.get(index).ok_or(Boundary::MissingActor)?.clone();let origin=(i32::from(shot.origin[0]),i32::from(shot.origin[1]));let target=(i32::from(shot.target[0]),i32::from(shot.target[1]));if origin==target||speed==0{return Ok(false);}
        let attributes=u.native_hardware_slot.and_then(|s|self.native_hardware_pool.get(s)).ok_or(Boundary::MissingHardwareProfile)?.attributes;
        self.native_projectile_warning(index,shot.target);
        //The original sound is attempted before the projectile-pool search.
        let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));if shot.sound>=0&&self.native_feedback.sound_enabled!=0&&!super::original_feedback::outside_view(shot.origin,camera,size){self.emit_native_sample(i16::from(shot.sound),super::original_audio::Gate::Direct);}
        let Some(id)=(0..50).find(|id|!self.projectiles.iter().any(|p|p.id==*id))else{return Ok(false);};let kind=if shot.rocket{Kind::Rocket}else{Kind::Shell};let range=i32::from(attributes.byte(5))*8;let cell=u.native_cell10 as u16 as usize;let (cx,cy)=(cell&127,cell>>7);let code=if cx<self.map.width&&cy<self.map.height{self.map.terrain_codes.get(cy*self.map.width+cx).copied().unwrap_or(0)}else{self.map.native_terrain_plane.get(cell).copied().unwrap_or(0)}&127;let elevation=i32::from(data::TERRAIN_HEIGHT[usize::from(code)])+2;
        let attack=Attack{target:usize::MAX,damage:attributes.byte(6),aim:u.original_combat.as_ref().map_or(0,|c|c.aim_skill),facing:original_combat::facing(u.aim),bullet:false};self.projectiles.push(Projectile{id,x:f64::from(origin.0)*2.,y:f64::from(origin.1)*2.,team:u.team,kind,sprite:if shot.rocket{1622}else{1634},angle:native_math::projectile_angle(target.0-origin.0,target.1-origin.1),size:0,flight:Flight::new(origin,target,range,speed,elevation,kind),attack,shooter:u.native_slot});Ok(true)
    }
    ///44c220/45ae80 update only the projectile returned by a successful
    ///allocation. A full later barrel cannot mutate the previous shot.
    pub(crate) fn emit_native_hardware_caller_projectile(&mut self,index:usize,shot:super::original_hardware_point::Shot,howitzer:bool,unsigned_speed:bool)->Result<bool,super::original_path_contact::game::Boundary>{let made=self.emit_native_hardware_shot(index,shot,if unsigned_speed{i32::from(shot.speed as u8)}else{i32::from(shot.speed)})?;if made{let p=self.projectiles.last_mut().unwrap();if howitzer{p.kind=Kind::Howitzer;}}Ok(made)}
    /// Point orders share the unit weapon's windup and allocator. No damage is
    /// committed here; the world geometry at the eventual impact decides it.
    pub(crate) fn fire_native_ground(&mut self,index:usize,point:(f64,f64),target:usize,grenade:bool)->bool {
        if !grenade&&self.units[index].kind=="apc"{return self.fire_native_apc_point(index,((point.0/2.).floor()as i32,(point.1/2.).floor()as i32),target);}

        if let Some(made)=self.update_native_ground_flame(index){return made;}
        //Synthetic/CLI maps lack the authored457020/44a040 caller. Preserve
        //their directed explosive-fire contract with the verified19/49 seeds;
        //this is an explicit web adapter, not an original class3 ground entry.
        if self.map.native_road_graph.is_none()&&stats(&self.units[index].kind).cls==0{
            let phase=if grenade&&self.units[index].grenades>0{Some((49,3))}else if !grenade&&self.units[index].kind=="tough"{Some((19,20))}else{None};
            if let Some((phase,timer))=phase{if self.units[index].native_phase35!=phase||phase==19&&self.units[index].native_order5c!= -5{
                let p=[(point.0/2.).floor()as i16,(point.1/2.).floor()as i16];let u=&mut self.units[index];u.native_phase35=phase;u.native_frame32=0;u.native_timer3c=timer;u.native_order5c=-5;u.native_idle=None;u.native_walk=None;u.native_fire=None;u.angle=native_math::angle(native_math::direction(i32::from(p[0])-(u.x/2.).floor()as i32,i32::from(p[1])-(u.y/2.).floor()as i32));u.aim=u.angle;if let Some(m)=u.native_slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.promotion.order=(-5i8)as u8;m.promotion.target_x=3456;m.promotion.target_y=p[0];m.promotion.target_aux=p[1];}return false;
            }}
        }
        if !self.advance_native_ground_fire_choice(index,target,point,grenade){return false;}
        if grenade&&self.units[index].grenades==0{return false;}
        let u=self.units[index].clone();let mut attack=Attack::from_unit(&u,target,if grenade{50.}else{self.native_unit_stats(&u).damage},u.aim);attack.bullet=false;
        let point=self.native_hardware_aim(&u,((point.0/2.).floor()as i32,(point.1/2.).floor()as i32));
        let launched=self.launch_native_heavy_at(index,point,attack,grenade);
        self.units[index].cooldown=if matches!(self.units[index].native_phase35,19|49){f64::from(100u8.wrapping_sub(self.units[index].original_combat.as_ref().map_or(80,|s|s.fire_rate)))/17.}else{self.units[index].original_combat.as_ref().map(|s|s.reload_seconds()).unwrap_or(stats(&u.kind).reload)};
        if launched{self.events.push(serde_json::json!({"kind":"shot","type":u.kind,"weapon":if grenade{"grenade"}else{"primary"},"x":u.x,"y":u.y,"team":u.team}));}
        launched
    }
    pub(crate) fn spawn_native_blast(&mut self,x:i32,y:i32,attack:Attack,team:i32,shooter:Option<usize>){
        self.native_blast_sound_prefix([x as i16,y as i16]);
        // Scenery contacts are synchronous even if the visual/delayed pool is full.
        self.native_blast_environment(x,y,attack.damage,team,false);
        let Some(id)=(0..50).find(|id|!self.blasts.iter().any(|b|b.id==*id))else{return;};
        let water=matches!(self.native_terrain(x+12,y+32),9|29|38|39|40|42);
        let special=data::TERRAIN_BLAST[self.map.world.min(4)];
        let (sprite,lifetime)=if water{if attack.damage<25{(special[2]as usize,special[3]as u8)}else{(special[0]as usize,special[1]as u8)}}else{(395,12)};
        self.blasts.push(Blast{id,x:f64::from(x)*2.,y:f64::from(y)*2.,team,frame:0,strength:attack.damage,water,sprite,lifetime,attack,_shooter:shooter});
        self.events.push(serde_json::json!({"kind":"explosion","x":(x+12)*2,"y":(y+24)*2,"radius":32,"silent":true}));
    }
    ///41f9d0/420200 tail for the41f610 ownerless zero-strength call. Sound and
    ///environment were already executed by its caller; no second event/sample.
    pub(crate) fn allocate_native_fragment_zero_blast(&mut self,p:[i16;2],water:bool)->bool {
        let Some(id)=(0..50).find(|id|!self.blasts.iter().any(|b|b.id==*id))else{return false;};
        let special=data::TERRAIN_BLAST[self.map.world.min(4)];let(sprite,lifetime)=if water{(special[2]as usize,special[3]as u8)}else{(395,12)};
        self.blasts.push(Blast{id,x:f64::from(p[0])*2.,y:f64::from(p[1])*2.,team:255,frame:0,strength:0,water,sprite,lifetime,attack:Attack{target:0,damage:0,aim:0,facing:0,bullet:false},_shooter:None});true
    }
    pub(crate) fn update_native_projectiles(&mut self){
        let mut pending=std::mem::take(&mut self.projectiles);pending.sort_by_key(|p|p.id);
        for mut p in pending {
            self.react_native_projectile([p.flight.line.x as i16,p.flight.line.y as i16]);
            let impact=p.flight.advance(p.kind,|x,y|self.native_height(x,y));
            let(x,y)=(p.flight.line.x,p.flight.line.y);p.x=f64::from(x)*2.;p.y=f64::from(y)*2.;
            if !impact {
                self.react_native_dodge(p.shooter,p.team,[p.flight.line.x as i16,p.flight.line.y as i16],[p.flight.target.0 as i16,p.flight.target.1 as i16]);
                p.size=p.flight.size(p.kind);p.sprite=match p.kind{Kind::Grenade=>1336+((self.motion_tick+1)/2)as usize%4,
                    Kind::Rocket=>1622+p.flight.phase,Kind::Flame=>6046+p.flight.phase,_=>1634};
                self.projectiles.push(p);continue;
            }
            if p.kind==Kind::Flame{self.spawn_native_fire(x,y,p.team,p.shooter);continue;}
            self.spawn_native_blast(x-12,y-24,p.attack,p.team,p.shooter);
            if p.attack.damage>50&&!matches!(self.native_terrain(x,y+8),9|42|38|40|39|29){
                for excess in [i32::from(p.attack.damage)-50,i32::from(p.attack.damage)-100]{
                    if excess<0{continue;}
                    let bx=x-12+(self.native_random()&31)as i32-16;
                    let by=y-24+(self.native_random()&31)as i32-16;
                    let mut attack=p.attack;attack.damage=(excess>>1)as u8;
                    if bx> -33{self.spawn_native_blast(bx,by,attack,p.team,p.shooter);}
                }
            }
        }
    }
    pub(crate) fn update_native_blasts(&mut self){
        if self.motion_tick&1!=0{return;}
        let mut pending=std::mem::take(&mut self.blasts);pending.sort_by_key(|b|b.id);
        for mut b in pending {
            b.frame+=1;
            if b.frame==3{self.native_blast_contact(&b);}
            if b.frame<b.lifetime{self.blasts.push(b);}
        }
    }
    fn native_blast_contact(&mut self,b:&Blast){
        let(left,top)=((b.x/2.)as i32-6,(b.y/2.)as i32+10);
        self.native_ambient_area_contact([left as i16,top as i16]);
        self.native_rectangle_contact(left,top,left+44,top+32,b.team,b.attack);
    }
    fn native_rectangle_contact(&mut self,left:i32,top:i32,right:i32,bottom:i32,team:i32,attack:Attack){
        let coarse=(top>>7)*16+(left>>7);
        for offset in BLAST_BUCKETS {
            let cell=coarse+offset;if !(0..256).contains(&cell){continue;}
            let mut candidates=self.units.iter().enumerate().filter(|(_,u)|original_combat::alive(true,u.hp)&&!workshops::hidden(u)
                &&(((u.y/2.).floor()as i32>>7)*16+((u.x/2.).floor()as i32>>7))==cell)
                .map(|(i,u)|(u.native_slot.unwrap_or(u.id),i)).collect::<Vec<_>>();
            candidates.sort_unstable();
            // Buckets are built before faction/rectangle filtering in the EXE.
            for (_,i) in candidates.into_iter().take(32){
                let u=&self.units[i];if u.team<0||u.team==team{continue;}
                let center=if stats(&u.kind).cls==1{16}else{8};
                let(x,y)=((u.x/2.).floor()as i32+center,(u.y/2.).floor()as i32+center);
                if x>=left&&x<=right&&y>=top&&y<=bottom{
                    let mut attack=attack;attack.target=u.id;self.apply_native_contact(attack);
                }
            }
        }
    }
    fn spawn_native_fire(&mut self,x:i32,y:i32,team:i32,_shooter:Option<usize>){
        if self.map.original&&self.native_world.is_none(){self.sync_native_world();}
        let variant=(self.native_random()&7).max(1)as usize;
        let id=if self.native_world.is_some(){self.reserve_native_world_fire_slot()}else{(0..200).find(|id|!self.fires.iter().any(|f|f.id==*id))};let Some(id)=id else{return;};
        let lifetime=i32::from(data::FIRE_LIFETIME[self.native_terrain(x-3,y-3)as usize]);
        let base=[0,5836,557,469,5840,5844,5848,5854][variant];
        self.fires.push(Fire{id,x:f64::from(x-3)*2.,y:f64::from(y-3)*2.,team,frame:0,sprite:base,base,lifetime});
        self.configure_native_world_fire_slot(id,[(x-3)as i16,(y-3)as i16],base as u16,0,lifetime as i8);
    }
    pub(crate) fn update_native_fires(&mut self){self.update_native_world_objects();}

}
