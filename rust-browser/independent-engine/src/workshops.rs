//! Explicit vehicle-workshop orders. Native entry/work/healing rules are
//! documented separately; routing and queue presentation are browser adapters.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub building_id: usize,
    pub phase: String,
    /// Remaining/total native repair work, not wall-clock seconds.
    pub remaining: f64,
    pub total: f64,
    pub seconds: Option<u32>,
    pub progress: f64,
    #[serde(skip)] pub(crate) tick_clock: f64,
    #[serde(skip)] pub(crate) heal_ticks: u32,
    #[serde(skip)] pub(crate) admitting: bool,
}

pub(crate) fn capable(unit: &Unit, original: bool) -> bool {
    unit.team >= 0 && original_combat::alive(original, unit.hp)
        && stats(&unit.kind).cls == 1 && stats(&unit.kind).speed > 0.
        && unit.crew.is_some() && unit.hp < unit.max_hp
}

pub(crate) fn hidden(unit: &Unit) -> bool {
    unit.workshop.as_ref().is_some_and(|s| s.phase == "repairing")
}

fn bay(b: &Building, original: bool) -> (f64, f64) {
    if original { (b.x+32.,b.y+96.) } else { (b.x+b.w/2.,b.y+b.h-16.) }
}
fn stage(b: &Building, original: bool) -> (f64, f64) {
    (bay(b,original).0,b.y+b.h+16.)
}
fn exit(b: &Building, original: bool) -> (f64, f64) {
    let p=bay(b,original);
    if original { (p.0+16.,p.1+132.) } else { (p.0,b.y+b.h+64.) }
}
fn work_for(hp: f64, max_hp: f64) -> f64 {
    // 0044ebf0: ((max(1, trunc(missing/5))+1)*884)/gameFactor; factor12.
    ((f64::max(1.,((max_hp-hp).max(0.)/5.).trunc())+1.)*884./12.).trunc()
}

impl Game {
    fn workshop_rate(&self,b: &Building,team:i32)->i32 {
        let owned=self.sectors.iter().filter(|s|s.owner==team&&s.counts_for_production).count();
        let total=self.sectors.iter().filter(|s|s.counts_for_production).count();
        original_stats::rate(b.hp,b.max_hp,owned,total)
    }
    fn workshop_path(&self,u:&Unit,b:&Building)->Option<VecDeque<(f64,f64)>> {
        let p=stage(b,self.map.original);
        if !self.navigation_point_clear(&u.kind,p.0,p.1){return None;}
        if (u.x-p.0).hypot(u.y-p.1)<1e-6 {return Some(VecDeque::new());}
        let path=self.route_unit(u,p.0,p.1);
        path.back().filter(|q|(q.0-p.0).hypot(q.1-p.1)<0.5).map(|_|path.clone())
    }
    pub(crate) fn begin_workshop(&mut self,index:usize,target:usize)->bool {
        let u=self.units[index].clone();
        let Some(b)=self.buildings.iter().find(|b|b.id==target&&b.kind=="repair"&&b.team==u.team&&b.hp>0.).cloned()else{return false;};
        if !capable(&u,self.map.original){return false;}
        if u.workshop.as_ref().is_some_and(|s|s.building_id==target){return true;}
        // Cancel an old hidden job before planning the new approach.
        self.cancel_workshop(index);
        let Some(path)=self.workshop_path(&self.units[index],&b)else{return false;};
        let total=work_for(u.hp,u.max_hp);
        let unit=&mut self.units[index];unit.order="repairVehicle".into();unit.target=Some(target);
        unit.dest=Some(stage(&b,self.map.original));unit.path=path;unit.initiative=None;unit.pursuit=None;unit.hold_position=false;
        unit.workshop=Some(State{building_id:target,phase:"approach".into(),remaining:total,total,seconds:None,progress:0.,tick_clock:0.,heal_ticks:0,admitting:false});
        true
    }
    fn release_workshop_position(&mut self,index:usize,building:usize) {
        let Some(b)=self.buildings.iter().find(|b|b.id==building)else{return;};
        let mut candidates=vec![stage(b,self.map.original),exit(b,self.map.original)];
        for n in 0..=8 {let a=n as f64/8.;candidates.extend([(b.x-16.,b.y+a*b.h),(b.x+b.w+16.,b.y+a*b.h),(b.x+a*b.w,b.y+b.h+16.)]);}
        if let Some(p)=candidates.into_iter().find(|p|self.navigation_point_clear(&self.units[index].kind,p.0,p.1)) {
            self.units[index].x=p.0;self.units[index].y=p.1;
        }
    }
    pub(crate) fn cancel_workshop(&mut self,index:usize) {
        let Some(s)=self.units[index].workshop.take()else{return;};
        if s.phase=="repairing"||s.phase=="exiting"||s.admitting {self.release_workshop_position(index,s.building_id);}
        let u=&mut self.units[index];u.path.clear();u.dest=None;u.target=None;
        u.order="guard".into();u.moving=false;u.hold_position=self.map.original;
    }
    fn workshop_portal_clear(&self,u:&Unit,b:&Building,goal:(f64,f64))->bool {
        let n=((u.x-goal.0).hypot(u.y-goal.1)/2.).ceil().max(1.)as usize;
        for i in 1..=n {
            let t=i as f64/n as f64;let(x,y)=(u.x+(goal.0-u.x)*t,u.y+(goal.1-u.y)*t);
            let inside=if self.map.original {x>=b.x+16.&&x<=b.x+96.&&y>=b.y+64.&&y<=b.y+b.h}else{x>=b.x+b.w/2.-24.&&x<=b.x+b.w/2.+24.&&y>=b.y&&y<=b.y+b.h};
            if inside {
                let(cx,cy)=((x/TILE).floor()as usize,(y/TILE).floor()as usize);
                if cx>=self.map.width||cy>=self.map.height{return false;}
                let j=cy*self.map.width+cx;
                if self.map.tiles[j]==6{return false;}
                if self.map.original {
                    let code=self.map.terrain_codes.get(j).copied().unwrap_or(127)&127;
                    // The native owned workshop opens its code26 doorway over
                    // the closed19/44 footprint. Permit only this job's portal.
                    if ![0,19,26,44,66].contains(&code){return false;}
                } else if matches!(self.map.tiles[j],2|3){return false;}
                if self.buildings.iter().any(|other|other.id!=b.id&&other.hp>0.&&x>other.x&&x<other.x+other.w&&y>other.y&&y<other.y+other.h){return false;}
            }else if !self.navigation_point_clear(&u.kind,x,y){return false;}
        }
        true
    }
    fn workshop_motion(&mut self,index:usize,b:&Building,goal:(f64,f64),dt:f64)->bool {
        let u=self.units[index].clone();
        if !self.workshop_portal_clear(&u,b,goal){return false;}
        let distance=(goal.0-u.x).hypot(goal.1-u.y);
        if distance<0.5 {self.units[index].x=goal.0;self.units[index].y=goal.1;return true;}
        let factor=u.original_combat.as_ref().map(|s|s.speed_scale(&u.kind)).unwrap_or(1.);
        let moved=(stats(&u.kind).speed*factor*dt).min(distance);
        let current=&mut self.units[index];current.x+=(goal.0-u.x)/distance*moved;current.y+=(goal.1-u.y)/distance*moved;
        current.angle=(goal.1-u.y).atan2(goal.0-u.x);current.aim=current.angle;current.moving=moved>0.;
        moved>=distance-1e-9
    }
    /// Returns true when this update is consumed by the workshop state machine.
    pub(crate) fn update_workshop(&mut self,index:usize,dt:f64)->bool {
        let Some(mut state)=self.units[index].workshop.clone()else{return false;};
        let u=self.units[index].clone();
        let b=self.buildings.iter().find(|b|b.id==state.building_id&&b.kind=="repair"&&b.team==u.team&&b.hp>0.).cloned();
        if u.order!="repairVehicle"||u.team<0||!original_combat::alive(self.map.original,u.hp)||stats(&u.kind).cls!=1||u.crew.is_none()||b.is_none(){
            self.cancel_workshop(index);return false;
        }
        let b=b.unwrap();
        self.units[index].moving=false;
        if state.phase=="exiting" {
            if self.workshop_motion(index,&b,exit(&b,self.map.original),dt) {
                let unit=&mut self.units[index];unit.workshop=None;unit.order="guard".into();unit.target=None;unit.dest=None;unit.path.clear();unit.moving=false;unit.hold_position=self.map.original;
                self.events.push(json!({"kind":"vehicleRepaired","team":u.team,"type":u.kind,"unit":u.id,"factory":b.id}));
            }
            return true;
        }
        if state.phase=="repairing" {
            let rate=self.workshop_rate(&b,u.team).max(0)as f64;
            state.tick_clock+=dt*17.;let ticks=state.tick_clock.floor()as u32;state.tick_clock-=ticks as f64;
            for _ in 0..ticks {
                state.remaining=(state.remaining-rate).max(0.);
                state.heal_ticks+=1;
                if state.heal_ticks>=17 {
                    state.heal_ticks=0;
                    let eta=if rate>0.{(state.remaining/(rate*17.)).trunc()as u32}else{0};
                    if eta>0 {let missing=(u.max_hp-self.units[index].hp).max(0.);let addition=(missing/eta as f64).trunc()+1.;self.units[index].hp=(self.units[index].hp+addition.max(1.)).min(u.max_hp);}
                }
                if state.remaining<=0. {
                    self.units[index].hp=u.max_hp;state.phase="exiting".into();state.progress=1.;state.seconds=Some(0);break;
                }
            }
            if state.phase=="repairing" {state.seconds=(rate>0.).then(||(state.remaining/(rate*17.)).trunc()as u32);state.progress=if state.total>0.{(1.-state.remaining/state.total).clamp(0.,1.)}else{0.};}
            self.units[index].workshop=Some(state);return true;
        }
        if state.admitting {
            if self.workshop_motion(index,&b,bay(&b,self.map.original),dt) {
                if let Some(combat)=&mut self.units[index].original_combat {combat.workshop_entered(&u.kind);}
                state.phase="repairing".into();state.admitting=false;state.remaining=work_for(self.units[index].hp,u.max_hp);state.total=state.remaining;state.tick_clock=0.;state.heal_ticks=0;
                let rate=self.workshop_rate(&b,u.team).max(0)as f64;state.seconds=(rate>0.).then(||(state.remaining/(rate*17.)).trunc()as u32);
                self.units[index].path.clear();self.units[index].dest=None;self.units[index].moving=false;
            }
            self.units[index].workshop=Some(state);return true;
        }
        let front=stage(&b,self.map.original);
        if (u.x-front.0).hypot(u.y-front.1)>1e-6 {
            if self.units[index].path.is_empty() {
                if let Some(path)=self.workshop_path(&u,&b){self.units[index].path=path;self.units[index].dest=Some(front);}
                else{self.cancel_workshop(index);return false;}
            }
            state.phase="approach".into();self.units[index].workshop=Some(state);return false;
        }
        // Normalize the final subpixel remainder at the exact portal column;
        // otherwise floating-point drift can select the closed neighboring tile.
        self.units[index].x=front.0;self.units[index].y=front.1;
        let u=self.units[index].clone();
        let occupied=self.units.iter().any(|other|other.id!=u.id&&other.workshop.as_ref().is_some_and(|s|s.building_id==b.id&&(s.phase=="repairing"||s.phase=="exiting"||s.admitting)));
        if occupied {state.phase="waiting".into();self.units[index].path.clear();self.units[index].dest=None;self.units[index].workshop=Some(state);return true;}
        if !self.workshop_portal_clear(&u,&b,bay(&b,self.map.original)){self.cancel_workshop(index);return false;}
        state.phase="approach".into();state.admitting=true;self.units[index].path.clear();self.units[index].dest=None;self.units[index].workshop=Some(state);true
    }
}
