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
impl Game {
    fn native_height(&self,x:i32,y:i32)->i32 {
        let code=self.native_terrain(x,y);
        i32::from(data::TERRAIN_HEIGHT.get(code as usize).copied().unwrap_or(0))
    }
    fn native_terrain(&self,x:i32,y:i32)->u8 {
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
        let origin=(x+offset[0],y+offset[1]);
        let elevation=self.native_height(x,y)+1+if stats(&u.kind).cls>0{1}else{0};
        let range=((stats(&u.kind).range/2.)as i32>>3)*8;
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
    /// Point orders share the unit weapon's windup and allocator. No damage is
    /// committed here; the world geometry at the eventual impact decides it.
    pub(crate) fn fire_native_ground(&mut self,index:usize,point:(f64,f64),target:usize,grenade:bool)->bool {
        if grenade&&self.units[index].grenades==0{return false;}
        let mut dummy=self.units[index].clone();dummy.id=target;
        dummy.x=point.0-16.;dummy.y=point.1-16.;dummy.kind=if grenade{"medium"}else{"grunt"}.into();
        if !self.advance_native_fire_choice(index,&dummy,grenade){return false;}
        let u=self.units[index].clone();let mut attack=Attack::from_unit(&u,target,if grenade{50.}else{stats(&u.kind).damage},u.aim);attack.bullet=false;
        let point=self.native_hardware_aim(&u,((point.0/2.).floor()as i32,(point.1/2.).floor()as i32));
        let launched=self.launch_native_heavy_at(index,point,attack,grenade);
        self.units[index].cooldown=self.units[index].original_combat.as_ref().map(|s|s.reload_seconds()).unwrap_or(stats(&u.kind).reload);
        if launched{self.events.push(serde_json::json!({"kind":"shot","type":u.kind,"weapon":if grenade{"grenade"}else{"primary"},"x":u.x,"y":u.y,"team":u.team}));}
        launched
    }
    fn spawn_native_blast(&mut self,x:i32,y:i32,attack:Attack,team:i32,shooter:Option<usize>){
        // Scenery contacts are synchronous even if the visual/delayed pool is full.
        self.native_blast_environment(x+12,y+24,attack.damage,team);
        let Some(id)=(0..50).find(|id|!self.blasts.iter().any(|b|b.id==*id))else{return;};
        let water=matches!(self.native_terrain(x+12,y+32),9|29|38|39|40|42);
        let special=data::TERRAIN_BLAST[self.map.world.min(4)];
        let (sprite,lifetime)=if water{if attack.damage<25{(special[2]as usize,special[3]as u8)}else{(special[0]as usize,special[1]as u8)}}else{(395,12)};
        self.blasts.push(Blast{id,x:f64::from(x)*2.,y:f64::from(y)*2.,team,frame:0,strength:attack.damage,water,sprite,lifetime,attack,_shooter:shooter});
        self.events.push(serde_json::json!({"kind":"explosion","x":(x+12)*2,"y":(y+24)*2,"radius":32}));
    }
    pub(crate) fn update_native_projectiles(&mut self){
        let mut pending=std::mem::take(&mut self.projectiles);pending.sort_by_key(|p|p.id);
        for mut p in pending {
            let impact=p.flight.advance(p.kind,|x,y|self.native_height(x,y));
            let(x,y)=(p.flight.line.x,p.flight.line.y);p.x=f64::from(x)*2.;p.y=f64::from(y)*2.;
            if !impact {
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
        let variant=(self.native_random()&7).max(1)as usize;
        let Some(id)=(0..200).find(|id|!self.fires.iter().any(|f|f.id==*id))else{return;};
        if self.fires.len()+self.objects.len()>=200{return;}
        let lifetime=i32::from(data::FIRE_LIFETIME[self.native_terrain(x-3,y-3)as usize]);
        let base=[0,5836,557,469,5840,5844,5848,5854][variant];
        self.fires.push(Fire{id,x:f64::from(x-3)*2.,y:f64::from(y-3)*2.,team,frame:0,sprite:base,base,lifetime});
    }
    pub(crate) fn update_native_fires(&mut self){
        if self.motion_tick&1!=0{return;}
        let mut pending=std::mem::take(&mut self.fires);pending.sort_by_key(|f|f.id);
        for mut f in pending{
            f.frame=(f.frame+1)%if matches!(f.base,557|5848|5854){6}else{4};f.sprite=f.base+usize::from(f.frame);
            if f.frame==0{
                let _smoke=self.native_random()&7==7;
                let(x,y)=((f.x/2.)as i32,(f.y/2.)as i32);
                let inside=self.buildings.iter().any(|b|f.x>=b.x&&f.x<=b.x+b.w&&f.y>=b.y&&f.y<=b.y+b.h);
                let damage=if inside{10}else{20};
                self.native_blast_environment(x+12,y+24,damage,255);
                self.native_rectangle_contact(x,y,x+24,y+24,255,Attack{target:0,damage,aim:0,facing:0,bullet:false});
                if f.lifetime!=127{f.lifetime-=8;if f.lifetime<0{continue;}}
            }
            self.fires.push(f);
        }
    }
    fn native_blast_environment(&mut self,x:i32,y:i32,strength:u8,team:i32){
        let (col,row)=(x>>4,(y+8)>>4);
        for (c,r) in [(col,row),(col-1,row),(col+1,row),(col,row-1),(col,row+1)]{
            if c<0||r<0||c>=self.map.width as i32||r>=self.map.height as i32{continue;}
            let j=r as usize*self.map.width+c as usize;
            if let Some(&owner)=self.wall_owners.get(j){if self.walls.iter().any(|w|w.id==owner&&w.hp>0.){self.damage_wall(owner,f64::from(strength),team);break;}}
        }
        // Building contacts use the impact position, never a saved target ID.
        for b in self.buildings.iter_mut().filter(|b|b.hp>0.&&b.team!=team){
            if f64::from(x)*2.>=b.x&&f64::from(x)*2.<=b.x+b.w&&f64::from(y)*2.>=b.y&&f64::from(y)*2.<=b.y+b.h{
                b.hp=(b.hp-f64::from(strength)).max(0.);
                if b.hp==0.{self.events.push(serde_json::json!({"kind":"buildingLost","team":b.team}));if b.kind=="fort"{self.winner=Some(team);}}
            }
        }
        let bridge=[(col,row),(col-1,row),(col+1,row),(col,row-1),(col,row+1)].into_iter().find_map(|(c,r)|{
            if c<0||r<0||c>=self.map.width as i32||r>=self.map.height as i32{return None;}
            self.bridge_at_point(f64::from(c*32+16),f64::from(r*32+16))
        });
        if let Some(id)=bridge{
            if self.map.bridge_hp[id]>0.{
                self.map.bridge_hp[id]=(self.map.bridge_hp[id]-f64::from(strength)).max(0.);
                if self.map.bridge_hp[id]==0.{
                    for j in 0..self.map.tiles.len(){if self.map.bridge_base[j]==id as i32{self.map.tiles[j]=3;}}
                    for u in &mut self.units{let(c,r)=((u.x/32.).floor()as usize,(u.y/32.).floor()as usize);if c<self.map.width&&r<self.map.height&&self.map.bridge_base[r*self.map.width+c]==id as i32{u.hp=-1.;}}
                    self.map.revision=self.map.revision.wrapping_add(1);self.events.push(serde_json::json!({"kind":"bridgeDestroyed","team":team}));
                }
            }
        }
    }
}
