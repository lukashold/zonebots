//! Native turret and hatch counters:0045b050/0044c3d0/0045b610.
//! Aiming, opening a hatch, and releasing its secondary weapon are distinct
//! transitions. The renderer consumes these states instead of guessing from time.
use serde::{Serialize,Deserialize};
use super::{Game,Unit,stats,native_math,original_combat::{self,Attack}};

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct State {
    pub turret:u8,pub hatch_frame:i8,pub hatch_body:u8,pub hatch_motion:i8,
    #[serde(skip)] aim_timer:i8,
    #[serde(skip)] hatch_timer:i8,
    #[serde(skip)] body_motion:i8,
    #[serde(skip)] debounce:i8,
    #[serde(skip)] scan:u8,
    #[serde(skip)] scan_flip:bool,
    #[serde(skip)] apc_target:Option<usize>,
    #[serde(skip)] pub(crate) passenger_phase:[u8;21],
}
impl State {
    pub(crate)fn mirror_common_hatch(&mut self,body:u8,frame:i8,motion:i8,timer:i8,debounce:i8){self.hatch_body=body&15;self.hatch_frame=frame;self.hatch_motion=motion;self.hatch_timer=timer;self.debounce=debounce;}

    ///The native target is a stable UNIT slot; the Web renderer ID can change
    ///when that same actor enters or leaves a neutral hardware descriptor.
    pub(crate) fn remap_target(&mut self,previous:usize,current:usize){if self.apc_target==Some(previous){self.apc_target=Some(current);}}
    pub fn new(turret:u8,hatch_frame:i8)->Self{Self{turret,hatch_frame,hatch_body:0,hatch_motion:0,
        aim_timer:15,hatch_timer:0,body_motion:0,debounce:0,scan:0,scan_flip:false,apc_target:None,passenger_phase:[0;21]}}
    ///58500 owns raw71:8 and77. Keep the old adapter's flip mirror in step
    ///when this source entry point is used; no second scan cursor is created.
    pub(crate) fn raw_scan_direction(&mut self,flags71:&mut u8,class:u8,facing:u8,relative:u8)->u8{
        if matches!(class,1|2){*flags71^=8;self.scan_flip=*flags71&8!=0;if self.scan_flip{self.scan=self.scan.wrapping_add(1)&7;return self.scan;}}
        facing.wrapping_add(relative)&7
    }
    #[cfg(test)]pub(crate) fn seed_raw_scan(&mut self,cursor:u8){self.scan=cursor;}
    #[cfg(test)]pub(crate) fn raw_scan_cursor(&self)->u8{self.scan}
    pub(crate) fn raw_scan_cursor_value(&self)->u8{self.scan}
    pub(crate) fn store_raw_scan_cursor(&mut self,n:u8){self.scan=n;}
    pub fn aim(&mut self,direction:u8,odd:bool,ready:bool)->bool{
        self.aim_timer=self.aim_timer.wrapping_sub(1);
        if self.aim_timer>=0{return false;}
        self.aim_timer=0;
        if self.turret==direction{return ready;}
        if odd{
            self.turret=if direction.wrapping_sub(self.turret)&4==0{self.turret.wrapping_add(1)&7}else{self.turret.wrapping_sub(1)&7};
            self.aim_timer=2;
        }
        false
    }
    fn open(&mut self){
        if self.hatch_motion<1&&self.hatch_frame<3{self.hatch_motion=1;self.hatch_timer=5;self.body_motion=1;}
    }
    fn close(&mut self){if self.hatch_motion>=0&&self.hatch_frame!=-1{self.hatch_motion=-1;self.hatch_timer=25;}}
    fn hatch(&mut self,eligible:bool,rate:u8)->bool{
        self.hatch_body=(i16::from(self.hatch_body)+i16::from(self.body_motion)).clamp(0,2)as u8;
        if self.debounce!=0{self.debounce=self.debounce.wrapping_sub(1);}
        if self.hatch_motion==0{
            if self.hatch_frame>3{self.hatch_frame-=1;self.hatch_timer=100i8.wrapping_sub(rate as i8);}
            else if self.hatch_frame==3{
                self.hatch_timer=self.hatch_timer.wrapping_sub(1);
                if self.hatch_timer<0&&eligible{self.hatch_frame+=1;return true;}
            }
        }else if self.hatch_motion<0{
            self.hatch_frame=self.hatch_frame.wrapping_sub(1);
            if self.hatch_frame<0{self.hatch_motion=self.hatch_motion.wrapping_add(1);self.body_motion=-1;}
        }else{
            self.hatch_timer=self.hatch_timer.wrapping_sub(1);
            if self.hatch_timer<0{
                self.hatch_frame=self.hatch_frame.wrapping_add(1);
                if self.hatch_frame>2{self.hatch_motion=0;self.hatch_timer=100i8.wrapping_sub(rate as i8);}
            }
        }
        false
    }
    fn apc_ports(&mut self,passengers:&[(bool,bool)],mask:u8,direction:usize)->Vec<(usize,usize,[i32;2])>{
        let mut ports=Vec::new();let mut seat=0;
        for (member,&(alive,pyro)) in passengers.iter().take(21).enumerate(){
            if !alive{continue;}
            self.passenger_phase[member]=(self.passenger_phase[member]+1)&15;
            if pyro&&self.passenger_phase[member]>=10{continue;}
            while seat<12&&mask&(1<<(seat%4))==0{seat+=1;}
            if seat>=12{break;}
            let side=seat%4;
            let table=match side{0=>super::native_weapon_data::APC_FRONT,1=>super::native_weapon_data::APC_RIGHT,2=>super::native_weapon_data::APC_BACK,_=>super::native_weapon_data::APC_LEFT};
            ports.push((member,side,table[direction*3+seat/4]));seat+=1;if seat==12{break;}
        }
        ports
    }
    fn apc_manual_members(&mut self,passengers:&[(bool,bool)])->Vec<usize>{
        let mut members=Vec::new();
        for (member,&(alive,pyro)) in passengers.iter().take(21).enumerate(){
            if !alive{continue;}
            self.passenger_phase[member]=(self.passenger_phase[member]+1)&15;
            if !pyro||self.passenger_phase[member]<4{members.push(member);}
        }
        members
    }
}
fn apc_reload(rate:u8,manual:bool)->u8{if manual{100u8.wrapping_sub(rate)}else{150u8.wrapping_sub(rate>>1).wrapping_sub(rate)}}
fn apc_manual_unit_target(kind:&str)->bool{kind=="tough"}
fn apc_shooter(carrier:&Unit,kind:&str)->Unit{
    //004581f0/00458090 retain the carrier's hardware fire rate, projectile
    // speed15 and damaged aim. Boarding changes the passenger weapon family.
    let mut shooter=carrier.clone();shooter.kind=kind.into();shooter.native_fire=None;shooter
}
pub fn has_hatch(kind:&str)->bool{matches!(kind,"light"|"medium"|"heavy")}
impl Game {
    pub(crate) fn ensure_native_hardware(&mut self,index:usize){
        if self.units[index].native_hardware.is_none(){
            let u=&self.units[index];let frame=u.original_combat.as_ref().map(|s|s.secondary_state).unwrap_or(-1);
            self.units[index].native_hardware=Some(State::new(original_combat::facing(u.aim),frame));
        }
    }
    pub(crate) fn aim_native_hardware(&mut self,index:usize,target:&Unit)->bool{
        self.ensure_native_hardware(index);
        if self.units[index].kind=="apc"{return false;}
        let u=&self.units[index];
        let direction=native_math::direction((target.x/2.).floor()as i32+12-((u.x/2.).floor()as i32+14),
            (target.y/2.).floor()as i32+8-((u.y/2.).floor()as i32+12));
        let ready=self.native_hardware_ready(u);let h=self.units[index].native_hardware.as_mut().unwrap();
        let release=h.aim(direction,self.motion_tick&1!=0,ready);
        self.units[index].aim=native_math::angle(h.turret);
        if stats(&self.units[index].kind).cls==2{self.units[index].angle=self.units[index].aim;}
        release
    }
    pub(crate) fn native_scan(&mut self,index:usize,relative:i32,range:f64)->Option<Unit>{
        self.ensure_native_hardware(index);
        let u=self.units[index].clone();let h=self.units[index].native_hardware.as_mut().unwrap();
        if stats(&u.kind).cls>0{h.scan_flip=!h.scan_flip;}
        let direction=if stats(&u.kind).cls>0&&h.scan_flip{h.scan=(h.scan+1)&7;h.scan}else{(i32::from(original_combat::facing(u.angle))+relative).rem_euclid(8)as u8};
        let coarse=((u.y/2.).floor()as i32>>7)*16+((u.x/2.).floor()as i32>>7);
        let mut best=None;let mut minimum=255;
        for offset in super::native_weapon_data::SCAN_CELLS[usize::from(direction)]{
            let cell=coarse+offset;if !(0..256).contains(&cell){continue;}
            let mut bucket=self.units.iter().filter(|v|v.team>=0&&original_combat::alive(true,v.hp)&&!super::workshops::hidden(v)
                &&(((v.y/2.).floor()as i32>>7)*16+((v.x/2.).floor()as i32>>7))==cell).collect::<Vec<_>>();
            bucket.sort_by_key(|v|v.native_slot.unwrap_or(v.id));
            for v in bucket.into_iter().take(32){
                if v.team==u.team{continue;}
                let dx=(v.x/2.).floor()as i32-(u.x/2.).floor()as i32;
                let dy=(v.y/2.).floor()as i32-(u.y/2.).floor()as i32;
                let distance=scan_distance(dx,dy,direction);
                if distance<minimum&&f64::from(distance)*16.<=range&&self.unit_target_los(&u,v){minimum=distance;best=Some(v.clone());}
            }
        }
        best
    }
    pub(crate) fn native_target_in_range(&self,u:&Unit,v:&Unit,range:f64)->bool{
        let dx=(v.x/2.).floor()as i32-(u.x/2.).floor()as i32;
        let dy=(v.y/2.).floor()as i32-(u.y/2.).floor()as i32;
        f64::from(scan_distance(dx,dy,original_combat::facing(u.angle)))*16.<=range
    }
    fn update_native_apc(&mut self,index:usize){
        let Some(driver)=self.units[index].crew.clone()else{return;};
        let u=self.units[index].clone();
        //0045ac40 accepts an explicit unit/group target only for Tough crews.
        // The ordinary hardware turn continues during reload; automatic side
        // fire may still run afterwards when the main target is not released.
        if u.order=="attack"&&apc_manual_unit_target(&driver.kind){
            let target=u.target.and_then(|id|self.units.iter().find(|v|v.id==id&&v.team>=0&&v.team!=u.team&&original_combat::alive(true,v.hp)&&!super::workshops::hidden(v))).cloned();
            if let Some(target)=target.filter(|v|self.native_target_in_range(&u,v,self.native_unit_stats(&u).range)&&self.unit_target_los(&u,v)){
                if self.units[index].native_hardware.as_ref().unwrap().apc_target!=Some(target.id){
                    self.units[index].native_hardware.as_mut().unwrap().apc_target=Some(target.id);
                    self.activate_native_combat_groups(index,target.id);self.observe_native_attack(index,target.id);
                }
                let point=((target.x/2.).floor()as i32+12,(target.y/2.).floor()as i32+8);
                let direction=native_math::direction(point.0-((u.x/2.).floor()as i32+14),point.1-((u.y/2.).floor()as i32+12));
                let ready=self.native_hardware_ready(&u);let h=self.units[index].native_hardware.as_mut().unwrap();let release=h.aim(direction,self.motion_tick&1!=0,ready);
                self.units[index].aim=native_math::angle(h.turret);
                if release{self.fire_native_apc_point(index,point,target.id);}
            }
        }else{self.units[index].native_hardware.as_mut().unwrap().apc_target=None;}
        self.run_native_apc_auto_tail_legacy(index,u,driver);
    }
    ///Existing447c20 approximation only. Owned56 calls this after its one
    ///Common; the old UI-target prefix stays outside this shared entry.
    pub(crate)fn update_native_apc_auto_tail_legacy(&mut self,index:usize){
        self.ensure_native_hardware(index);
        let Some(driver)=self.units[index].crew.clone()else{return;};
        let u=self.units[index].clone();
        self.run_native_apc_auto_tail_legacy(index,u,driver);
    }
    fn run_native_apc_auto_tail_legacy(&mut self,index:usize,u:Unit,driver:super::Crew){
        if !self.native_hardware_ready(&self.units[index]){return;}
        let mut targets:[Option<Unit>;4]=[None,None,None,None];
        for side in [1,3,0,2]{
            //004476f0 first scans without a weapon-range cutoff, then checks
            // exact squared distance against hardware+5 (the passenger range).
            targets[side]=self.native_scan(index,(side*2)as i32,f64::INFINITY).filter(|v|{
                let dx=(u.x/2.).floor()as i64-(v.x/2.).floor()as i64;let dy=(u.y/2.).floor()as i64-(v.y/2.).floor()as i64;
                dx*dx+dy*dy<=((stats(&driver.kind).range/2.)as i64).pow(2)
            });
        }
        if targets.iter().all(Option::is_none){return;}
        let rate=u.original_combat.as_ref().map(|s|s.fire_rate).unwrap_or(80);
        self.set_native_hardware_reload(index,apc_reload(rate,false));
        let weapon_kind=driver.kind.clone();
        let passengers=std::iter::once(driver).chain(u.cargo.clone()).take(21).collect::<Vec<_>>();
        let mask=targets.iter().enumerate().fold(0,|m,(side,t)|m|if t.is_some(){1<<side}else{0});
        let direction=usize::from(original_combat::facing(u.angle));
        // The native carrier owns one weapon class (+8e) and damage field
        // (+65) for the whole roster, even if a compatibility save mixes crews.
        let active=passengers.iter().map(|p|(p.hp>=0.,weapon_kind=="pyro")).collect::<Vec<_>>();
        let ports=self.units[index].native_hardware.as_mut().unwrap().apc_ports(&active,mask,direction);
        for (_member,side,offset) in ports{
            let target=targets[side].as_ref().unwrap();
            let mut origin=((u.x/2.).floor()as i32+offset[0]-6,(u.y/2.).floor()as i32+offset[1]-6);
            let shooter=apc_shooter(&u,&weapon_kind);
            let point=self.native_scatter(&shooter,target,super::original_fire::Mode::Hardware);let point=(point.0+8,point.1+8);
            let mut attack=Attack::from_unit(&shooter,target.id,stats(&weapon_kind).damage,(target.y-u.y).atan2(target.x-u.x));
            let range=(((stats(&weapon_kind).range/2.)as i32>>3)+1)*8;
            let emitted=match weapon_kind.as_str(){
                "tough"|"pyro"=>{
                    let kind=if weapon_kind=="tough"{super::original_projectiles::Kind::Rocket}else{
                        let o=super::native_weapon_data::APC_FLAME[direction];origin.0+=o[0];origin.1+=o[1];super::original_projectiles::Kind::Flame};
                    attack.bullet=false;self.emit_native_projectile(&shooter,origin,point,kind,15,range-8,attack)
                },
                _=>{attack.bullet=true;if weapon_kind=="laser"{origin.1-=7;}self.emit_native_bullet(&shooter,origin,point,range,weapon_kind=="laser",attack)},
            };
            if emitted{self.events.push(serde_json::json!({"kind":"shot","type":weapon_kind,"weapon":"passenger","carrier":u.id,"x":origin.0*2,"y":origin.1*2,"team":u.team}));}
        }
    }
    ///00447da0: forced Tough/Pyro point fire uses the carrier anchor for every
    /// passenger, independent two-axis jitter, and the separate manual phase gate.
    /// The browser currently exposes directed unit fire for Tough crews only.
    pub(crate) fn fire_native_apc_point(&mut self,index:usize,point:(i32,i32),target:usize)->bool{
        if self.units[index].kind!="apc"||!self.native_hardware_ready(&self.units[index]){return false;}
        let u=self.units[index].clone();let Some(driver)=u.crew.as_ref()else{return false;};
        if !matches!(driver.kind.as_str(),"tough"|"pyro"){return false;}
        if self.native_hardware_common.apc_scope==1{if let Some(slot)=u.native_slot{self.native_common_legacy.push((slot as u8,crate::original_hardware_common::LegacyCall::ApcManual447da0));}}
        self.ensure_native_hardware(index);
        let passengers=std::iter::once(driver.clone()).chain(u.cargo.clone()).take(21).collect::<Vec<_>>();
        let active=passengers.iter().map(|p|(p.hp>=0.,driver.kind=="pyro")).collect::<Vec<_>>();
        let members=self.units[index].native_hardware.as_mut().unwrap().apc_manual_members(&active);
        let origin=((u.x/2.).floor()as i32,(u.y/2.).floor()as i32);
        let shooter=apc_shooter(&u,&driver.kind);let mut emitted=false;
        for _member in members{
            let x=(point.0 as i16).wrapping_add((self.native_random()&15)as i16-8)as i32;
            let y=(point.1 as i16).wrapping_add((self.native_random()&15)as i16-8)as i32;
            let mut attack=Attack::from_unit(&shooter,target,stats(&driver.kind).damage,native_math::angle(native_math::direction(x-origin.0,y-origin.1)));attack.bullet=false;
            let kind=if driver.kind=="tough"{super::original_projectiles::Kind::Rocket}else{super::original_projectiles::Kind::Flame};
            let range=((stats(&driver.kind).range/2.)as i32>>3)*8;
            if self.emit_native_projectile(&shooter,origin,(x,y),kind,15,range,attack){
                emitted=true;self.events.push(serde_json::json!({"kind":"shot","type":driver.kind,"weapon":"passenger","carrier":u.id,"x":origin.0*2,"y":origin.1*2,"team":u.team}));
            }
        }
        self.set_native_hardware_reload(index,apc_reload(u.original_combat.as_ref().map(|s|s.fire_rate).unwrap_or(80),true));
        emitted
    }
    pub(crate) fn update_native_hardware(&mut self,index:usize){
        if !self.map.original||self.units[index].team<0||stats(&self.units[index].kind).cls==0{return;}
        self.ensure_native_hardware(index);
        if self.units[index].native_phase35==9{self.units[index].native_elevation76=0;}
        self.update_native_vehicle_contacts(index);
        if let Err(e)=self.update_native_friendly_contacts(index){self.native_route_boundary=Some(e);}
        if self.units[index].kind=="apc"{let point=self.units[index].native_slot.and_then(|s|self.native_group_rosters.members[s].as_ref()).is_some_and(|m|m.flags69&32!=0);if !point{self.update_native_apc(index);}return;}
        if !has_hatch(&self.units[index].kind){return;}
        let u=self.units[index].clone();
        // The native hatch only scans while its firing countdown is negative.
        // Its candidate need not be infantry; armor still decides bullet damage.
        let needs_target={let h=self.units[index].native_hardware.as_ref().unwrap();h.hatch_motion==0&&h.hatch_frame==3&&h.hatch_timer<=0};
        let target=if needs_target{
            let body=self.units[index].angle;self.units[index].angle=self.units[index].aim;
            let target=self.native_scan(index,0,self.native_unit_stats(&u).range);
            self.units[index].angle=body;target
        }else{None};
        let fire=self.units[index].native_hardware.as_mut().unwrap().hatch(target.is_some(),80);
        self.units[index].original_combat.as_mut().unwrap().secondary_state=self.units[index].native_hardware.as_ref().unwrap().hatch_frame;
        if fire{if let Some(target)=target{self.fire_native_secondary(index,&target);}}
    }
    ///0045b880: the group recovery counter closes a vulnerable crew while at
    ///rest. During group recovery, failed mutual aim checks may expose the crew.
    pub(crate) fn observe_native_attack(&mut self,attacker:usize,target:usize){
        let Some(t)=self.units.iter().position(|u|u.id==target&&has_hatch(&u.kind)&&u.team>=0)else{return;};
        self.ensure_native_hardware(t);
        if self.units[t].group_bonus_ticks==0{self.units[t].native_hardware.as_mut().unwrap().close();return;}
        if self.units[t].native_hardware.as_ref().unwrap().debounce!=0{return;}
        self.units[t].native_hardware.as_mut().unwrap().debounce=-120;
        let aim=|u:&Unit|i32::from(u.original_combat.as_ref().map(|s|s.aim_skill).unwrap_or(60));
        let a=aim(&self.units[attacker]);let b=aim(&self.units[t]);
        let first=a+(a>>2)>=(self.native_random()&127)as i32;
        let close=first&&{let adjusted=(b-30).max(0);adjusted+(adjusted>>2)>=(self.native_random()&127)as i32};
        let h=self.units[t].native_hardware.as_mut().unwrap();if close{h.close();}else{h.open();}
    }
    pub(crate) fn fire_native_secondary(&mut self,index:usize,target:&Unit){
        self.activate_native_combat_groups(index,target.id);
        let mut shooter=self.units[index].clone();
        if let Some(s)=shooter.original_combat.as_mut(){s.aim_skill>>=2;}
        let point=self.native_scatter(&shooter,target,super::original_fire::Mode::Hardware);
        let origin=((shooter.x/2.).floor()as i32+8,(shooter.y/2.).floor()as i32+8);
        let point=(point.0+8,point.1+8);
        let mut attack=Attack::from_unit(&shooter,target.id,4.,(target.y-shooter.y).atan2(target.x-shooter.x));attack.bullet=true;
        let range=(((self.native_unit_stats(&shooter).range/2.)as i32>>3)+1)*8;
        if self.emit_native_bullet(&shooter,origin,point,range,false,attack){
            self.events.push(serde_json::json!({"kind":"shot","type":shooter.kind,"weapon":"secondary","x":shooter.x,"y":shooter.y,"team":shooter.team}));
        }
    }
}

/// The original directional predicate's return is discarded by0044ce00. Its
/// signed X rewrite still affects quantization; do not replace this with hypot.
pub(crate) fn scan_distance(dx:i32,dy:i32,direction:u8)->u8{
    let x=match direction{0|4=>dx.abs(),6=>-dx,_=>dx}>>3;let y=dy>>3;
    native_math::isqrt((i64::from(x).pow(2)+i64::from(y).pow(2))as u64)as u8
}

#[cfg(test)]
mod tests{
    use super::*;
    fn hash(h:&mut u32,values:&[i32]){for v in values{*h=(*h^*v as u32).wrapping_mul(16777619);}}
    #[test]
    fn turret_matches_384_emulated_callbacks(){
        let mut h=2166136261;
        for direction in 0..8{
            let mut state=State::new(0,-1);let mut reload=0;
            for tick in 1..=48{
                if reload>0{reload-=1;}
                let shot=state.aim(direction,tick&1!=0,reload==0);
                if shot{reload=20;}
                hash(&mut h,&[i32::from(direction),tick,i32::from(state.turret),i32::from(state.aim_timer),i32::from(shot)]);
            }
        }
        assert_eq!(h,415109158);
    }
    #[test]
    fn hatches_match_256_emulated_callbacks(){
        let mut h=2166136261;
        for scenario in 0..4{
            let mut state=State::new(0,if scenario==2{3}else{-1});
            state.hatch_motion=if scenario==2{-1}else{1};state.hatch_timer=if scenario==2{25}else{5};
            for tick in 1..=64{
                let shot=state.hatch(scenario!=1,80);
                hash(&mut h,&[scenario,tick,i32::from(state.hatch_frame),i32::from(state.hatch_motion),i32::from(state.hatch_timer),i32::from(shot)]);
            }
        }
        assert_eq!(h,409407460);
    }
    #[test]
    fn apc_ports_match_1512_emitted_muzzles_and_360_native_bursts(){
        let mut shots=2166136261;let mut bursts=2166136261;
        for direction in 0..8{for mask in 1..16{for passengers in [1,6,21]{
            let mut state=State::new(0,-1);let ports=state.apc_ports(&vec![(true,false);passengers],mask,direction);
            for (shot,&(_,side,offset))in ports.iter().enumerate(){hash(&mut shots,&[direction as i32,i32::from(mask),passengers as i32,shot as i32,side as i32,500+offset[0]-6,600+offset[1]-6]);}
            hash(&mut bursts,&[direction as i32,i32::from(mask),passengers as i32,ports.len()as i32,150-(80>>1)-80]);
        }}}
        assert_eq!(shots,703959899);assert_eq!(bursts,2168408253);
    }
    #[test]
    fn apc_pyro_gate_matches_all_sixteen_native_passenger_phases(){
        let mut h=2166136261;
        for phase in 0..16{let mut state=State::new(0,-1);state.passenger_phase[0]=phase;
            let ports=state.apc_ports(&[(true,true)],15,0);
            hash(&mut h,&[i32::from(phase),i32::from(state.passenger_phase[0]),ports.len()as i32,150-(90>>1)-90]);
        }
        assert_eq!(h,2805808201);
    }
    #[test]
    fn apc_manual_bursts_match_2240_native_projectiles_and_384_reload_phase_states(){
        let mut shots=2166136261;let mut bursts=2166136261;
        for class in [3,15]{for multi in 0..2{for passengers in [1,6,21]{for phase in 0..16{for facing in [0,3]{
            let mut state=State::new(0,-1);state.passenger_phase.fill(phase);
            let mut rng=original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:multi!=0};
            let members=state.apc_manual_members(&vec![(true,class==15);passengers]);
            for (shot,_)in members.iter().enumerate(){
                let x=800+(rng.next_u32()&15)as i32-8;let y=900+(rng.next_u32()&15)as i32-8;
                hash(&mut shots,&[class,multi,passengers as i32,i32::from(phase),facing,shot as i32,500,600,x,y]);
            }
            hash(&mut bursts,&[class,multi,passengers as i32,i32::from(phase),facing,members.len()as i32,i32::from(apc_reload(70,true)),i32::from(state.passenger_phase[0])]);
        }}}}}
        assert_eq!(shots,370539461);assert_eq!(bursts,60003957);
    }
    #[test]
    fn apc_unit_group_target_gate_matches_the_original_twelve_marker_cases(){
        let mut h=2166136261;
        for (class,kind)in [(0,"grunt"),(3,"tough"),(15,"pyro"),(16,"laser")]{for marker in [-1234,-4321,100]{
            let allowed=apc_manual_unit_target(kind)||!matches!(marker,-1234|-4321);
            hash(&mut h,&[class,marker,i32::from(allowed)]);
        }}
        assert_eq!(h,3695306749);
    }
    #[test]
    fn all_784_directional_distances_match_the_original_x86_scanner(){
        let mut h=2166136261;
        for direction in 0..8{for dx in [-257,-129,-65,-17,-8,-1,0,1,7,8,17,65,129,257]{for dy in [-129,-17,-1,0,1,17,129]{
            hash(&mut h,&[i32::from(direction),dx,dy,1,i32::from(scan_distance(dx,dy,direction))]);
        }}}
        assert_eq!(h,3638738723);
    }
    fn field()->Game{
        let mut g=Game::new(0,1,491);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
        g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;g.next_id=1;g
    }
    #[test]
    fn game_waits_for_the_turret_and_keeps_its_heading_during_a_march(){
        let mut g=field();g.spawn("medium",0,320.,320.,1,None);g.spawn("gatling",1,512.,320.,2,None);
        for u in &mut g.units{u.hold_position=true;}g.units[1].cooldown=10000.;g.units[1].native_reload7b=127;
        let id=g.units[0].id;let mut first=None;
        for tick in 1..=24{g.step(0.059);if g.events.iter().any(|e|e["kind"]=="shot"&&e["team"]==0){first=Some(tick);break;}}
        assert_eq!(first,Some(24));assert_eq!(g.units[0].native_hardware.as_ref().unwrap().turret,2);
        g.units[1].x=1500.;g.units[1].y=1200.;g.command(&crate::json!({"kind":"move","ids":[id],"x":320.,"y":600.}));
        for _ in 0..10{g.step(0.059);}assert!(g.units[0].y>320.);
        assert_eq!(original_combat::facing(g.units[0].angle),4);assert_eq!(original_combat::facing(g.units[0].aim),2);
    }
    #[test]
    fn game_apc_passengers_use_separate_shots_and_their_own_weapon_families(){
        for kind in ["grunt","tough","pyro","laser"]{
            let mut g=field();g.spawn("apc",0,320.,320.,1,Some(super::super::Crew{kind:kind.into(),hp:100.}));
            g.units[0].cargo=vec![super::super::Crew{kind:kind.into(),hp:100.};5];g.units[0].hold_position=true;
            g.spawn("heavy",1,320.,160.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;g.units[1].native_reload7b=127;
            g.step(0.059);let events=g.events.iter().filter(|e|e["kind"]=="shot"&&e["weapon"]=="passenger").collect::<Vec<_>>();
            assert_eq!(events.len(),6,"{kind}");assert!(events.iter().all(|e|e["type"]==kind&&e["carrier"]==g.units[0].id));
            assert!(!g.events.iter().any(|e|e["kind"]=="shot"&&e["team"]==0&&e["weapon"]=="primary"));
            if matches!(kind,"tough"|"pyro"){assert_eq!(g.projectiles.len(),6);}else{assert!(!g.bullets.is_empty());assert!(g.bullets.iter().all(|b|b.laser==(kind=="laser")));}
        }
    }
    #[test]
    fn apc_passenger_fire_retains_the_carrier_rate_and_damaged_aim(){
        let mut g=field();g.spawn("apc",0,320.,320.,1,Some(super::super::Crew{kind:"pyro".into(),hp:80.}));
        g.units[0].hold_position=true;let state=g.units[0].original_combat.as_mut().unwrap();state.aim_skill=13;state.fire_rate=60;
        assert_eq!(apc_shooter(&g.units[0],"pyro").original_combat.unwrap().aim_skill,13);
        g.spawn("heavy",1,320.,160.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;g.units[1].native_reload7b=127;
        g.step(0.059);assert_eq!(g.units[0].native_reload7b,60);assert_eq!(g.units[0].cooldown,60.*0.059);
        assert_eq!(apc_reload(98,false),3);assert_eq!(apc_reload(80,false),30);assert_eq!(apc_reload(80,true),20);
    }
    #[test]
    fn a_manual_apc_burst_uses_the_anchor_and_all_twenty_one_live_roster_slots(){
        let mut g=field();g.spawn("apc",0,1000.,1200.,1,Some(super::super::Crew{kind:"tough".into(),hp:100.}));
        g.units[0].cargo=vec![super::super::Crew{kind:"tough".into(),hp:100.};20];g.units[0].original_combat.as_mut().unwrap().fire_rate=70;
        g.original_rng=original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:false};
        assert!(g.fire_native_apc_point(0,(800,900),200));assert_eq!(g.projectiles.len(),21);
        assert!(g.projectiles.iter().all(|p|p.x==1000.&&p.y==1200.));assert_eq!(g.units[0].native_reload7b,30);assert_eq!(g.units[0].cooldown,30.*0.059);
        assert_eq!(g.events.iter().filter(|e|e["weapon"]=="passenger").count(),21);
    }
    #[test]
    fn one_carrier_weapon_class_controls_every_passenger_port(){
        let mut g=field();g.spawn("apc",0,320.,320.,1,Some(super::super::Crew{kind:"laser".into(),hp:60.}));
        g.units[0].cargo=vec![super::super::Crew{kind:"grunt".into(),hp:30.},super::super::Crew{kind:"pyro".into(),hp:80.}];
        g.units[0].hold_position=true;g.spawn("heavy",1,320.,160.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;g.units[1].native_reload7b=127;
        g.step(0.059);let shots=g.events.iter().filter(|e|e["weapon"]=="passenger").collect::<Vec<_>>();
        assert_eq!(shots.len(),3);assert!(shots.iter().all(|e|e["type"]=="laser"));assert!(g.bullets.iter().all(|b|b.laser));assert!(g.projectiles.is_empty());
    }
}
