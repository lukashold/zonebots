//! Validated interaction lifecycles and structure firing positions.
use super::*;
impl Game {
    pub(crate) fn finish_order(&mut self,index:usize) {
        let u=&mut self.units[index];u.order="guard".into();u.target=None;u.dest=None;
        u.path.clear();u.pursuit=None;u.initiative=None;u.march_resume=None;u.moving=false;u.hold_position=self.map.original;
    }
    pub(crate) fn bridge_at_point(&self,x:f64,y:f64)->Option<usize> {
        if !x.is_finite()||!y.is_finite()||x<0.||y<0.||x>=self.map.width as f64*TILE||y>=self.map.height as f64*TILE{return None;}
        let j=(y/TILE).floor()as usize*self.map.width+(x/TILE).floor()as usize;
        self.map.bridge_base.get(j).copied().filter(|id|*id>=0).map(|id|id as usize).filter(|id|*id<self.map.bridge_hp.len())
    }
    pub(crate) fn building_point(b:&Building,x:f64,y:f64)->(f64,f64) {(x.clamp(b.x,b.x+b.w),y.clamp(b.y,b.y+b.h))}
    pub(crate) fn structure_los(&self,x:f64,y:f64,tx:f64,ty:f64,target:Option<usize>,mount:Option<usize>)->bool {
        if ![x,y,tx,ty].iter().all(|v|v.is_finite()){return false;}
        let steps=((tx-x).hypot(ty-y)/4.).ceil().max(1.)as usize;
        for n in 1..=steps {
            let t=n as f64/steps as f64;let(px,py)=(x+(tx-x)*t,y+(ty-y)*t);
            if px<0.||py<0.||px>=self.map.width as f64*TILE||py>=self.map.height as f64*TILE{return false;}
            let ignored=self.buildings.iter().any(|b|(Some(b.id)==target||Some(b.id)==mount&&b.kind=="fort")&&px>=b.x&&px<=b.x+b.w&&py>=b.y&&py<=b.y+b.h);
            if ignored{continue;}
            let j=(py/TILE).floor()as usize*self.map.width+(px/TILE).floor()as usize;
            if self.map.tiles.get(j).is_some_and(|t|matches!(t,2|6)){return false;}
        }
        true
    }
    pub(crate) fn unit_los(&self,u:&Unit,x:f64,y:f64)->bool {self.structure_los(u.x,u.y,x,y,None,u.fort_id)}
    pub(crate) fn unit_target_los(&self,u:&Unit,target:&Unit)->bool {self.structure_los(u.x,u.y,target.x,target.y,target.fort_id,u.fort_id)}
    pub(crate) fn building_can_fire(&self,u:&Unit,b:&Building,range:f64)->bool {
        let p=Self::building_point(b,u.x,u.y);
        (u.x-p.0).hypot(u.y-p.1)<=range&&self.structure_los(u.x,u.y,p.0,p.1,Some(b.id),u.fort_id)
    }
    pub(crate) fn update_building_attack(&mut self,index:usize,range:f64,speed:f64)->bool {
        let u=self.units[index].clone();if u.order!="attackBuilding"{return false;}
        let b=u.target.and_then(|id|self.buildings.iter().find(|b|b.id==id&&b.hp>0.&&b.team!=u.team)).cloned();
        let Some(b)=b else{self.finish_order(index);return false;};
        if self.building_can_fire(&u,&b,range){self.units[index].path.clear();return true;}
        if speed<=0.{self.units[index].path.clear();return false;}
        if self.units[index].path.is_empty(){self.units[index].path=self.route_to_building_attack(&u,&b,range);}
        false
    }
}
