//! Terrain-aware browser navigation. Native permission masks live separately
//! in original_terrain; A* and live segment checks are engine adaptations.
use super::{original_terrain::{self, Profile}, Game, TILE};
use std::{cmp::Reverse, collections::{BinaryHeap, VecDeque}};

impl Game {
    fn cell(&self, x:i32,y:i32)->Option<usize>{
        if x<1||y<1||x>=self.map.width as i32-1||y>=self.map.height as i32-1{return None;}
        Some(y as usize*self.map.width+x as usize)
    }
    fn terrain_open(&self,x:i32,y:i32,profile:Profile)->bool {
        let Some(j)=self.cell(x,y) else{return false;};
        let Some(&tile)=self.map.tiles.get(j) else{return false;};
        if tile==2||tile==6{return false;}
        // A destroyed bridge must not inherit the original intact bridge code.
        if self.map.bridge_base.get(j).is_some_and(|&id|id>=0)&&tile==3{return false;}
        if self.map.original {
            if let Some(&code)=self.map.terrain_codes.get(j){
                return original_terrain::passable(code&127,self.map.world,profile);
            }
        }
        tile!=3||(profile==Profile::Infantry&&self.map.world!=1)
    }
    fn passable_profile(&self,x:i32,y:i32,profile:Profile)->bool {
        if !self.terrain_open(x,y,profile){return false;}
        let(px,py)=((x as f64+0.5)*TILE,(y as f64+0.5)*TILE);
        !self.buildings.iter().any(|b|b.hp>0.&&px>b.x&&px<b.x+b.w&&py>b.y&&py<b.y+b.h
            &&!b.gate.map(|(gx,gy)|(px-gx).abs()<24.&&(if b.orientation=="north"{py<=gy+24.}else{py>=gy-24.})).unwrap_or(false))
    }
    #[cfg(test)]
    pub(crate) fn passable(&self,x:i32,y:i32,infantry:bool)->bool{
        self.passable_profile(x,y,if infantry{Profile::Infantry}else{Profile::Tracked})
    }
    #[cfg(test)]
    pub(crate) fn passable_for(&self,x:i32,y:i32,kind:&str)->bool{
        self.passable_profile(x,y,original_terrain::profile(kind))
    }
    fn factor_profile(&self,x:i32,y:i32,profile:Profile,control:u8)->f64{
        let Some(j)=self.cell(x,y) else{return 1.;};
        if self.map.original {
            if let Some(&code)=self.map.terrain_codes.get(j){return original_terrain::speed_factor_for(code,control);}
        }
        let road=self.map.tiles.get(j)==Some(&1)||self.map.tiles.get(j)==Some(&5);
        if road&&profile!=Profile::Infantry{1.3}else{1.}
    }
    #[cfg(test)]
    pub(crate) fn terrain_factor_for(&self,x:i32,y:i32,kind:&str)->f64{
        self.factor_profile(x,y,original_terrain::profile(kind),super::original_combat::CombatState::new(kind).movement_control)
    }
    fn routing_cost(&self,x:i32,y:i32,profile:Profile,control:u8)->f64{
        if self.map.original {
            if let Some(code)=self.cell(x,y).and_then(|j|self.map.terrain_codes.get(j)){
                return f64::from(original_terrain::route_cost(*code,profile));
            }
        }
        5./self.factor_profile(x,y,profile,control)
    }
    #[cfg(test)]
    pub(crate) fn route(&self,x:f64,y:f64,dx:f64,dy:f64,infantry:bool)->VecDeque<(f64,f64)>{
        self.route_profile(x,y,dx,dy,if infantry{Profile::Infantry}else{Profile::Tracked},40,None,None)
    }
    #[cfg(test)]
    pub(crate) fn route_for(&self,x:f64,y:f64,dx:f64,dy:f64,kind:&str)->VecDeque<(f64,f64)>{
        self.route_profile(x,y,dx,dy,original_terrain::profile(kind),super::original_combat::CombatState::new(kind).movement_control,None,None)
    }
    pub(crate) fn route_unit(&self,unit:&super::Unit,dx:f64,dy:f64)->VecDeque<(f64,f64)>{
        let control=unit.original_combat.as_ref().map(|s|s.movement_control)
            .unwrap_or_else(||super::original_combat::CombatState::new(&unit.kind).movement_control);
        self.route_profile(unit.x,unit.y,dx,dy,original_terrain::profile(&unit.kind),control,None,None)
    }
    pub(crate) fn route_to_attack(&self,unit:&super::Unit,tx:f64,ty:f64,range:f64)->VecDeque<(f64,f64)>{
        let control=unit.original_combat.as_ref().map(|s|s.movement_control)
            .unwrap_or_else(||super::original_combat::CombatState::new(&unit.kind).movement_control);
        self.route_profile(unit.x,unit.y,tx,ty,original_terrain::profile(&unit.kind),control,Some(range.max(0.)),None)
    }
    pub(crate) fn terrain_factor_unit(&self,unit:&super::Unit)->f64{
        let control=unit.original_combat.as_ref().map(|s|s.movement_control)
            .unwrap_or_else(||super::original_combat::CombatState::new(&unit.kind).movement_control);
        self.factor_profile((unit.x/TILE).floor()as i32,(unit.y/TILE).floor()as i32,original_terrain::profile(&unit.kind),control)
    }
    pub(crate) fn route_profile(&self,x:f64,y:f64,dx:f64,dy:f64,profile:Profile,control:u8,firing_range:Option<f64>,wall_target:Option<usize>)->VecDeque<(f64,f64)>{
        let empty=VecDeque::new();
        if ![x,y,dx,dy].iter().all(|n|n.is_finite())||x<0.||y<0.{return empty;}
        let(w,h)=(self.map.width,self.map.height);
        let(sx,sy)=((x/TILE)as usize,(y/TILE)as usize);
        if sx>=w||sy>=h{return empty;}
        let(gx,gy)=((dx/TILE).floor()as i32,(dy/TILE).floor()as i32);
        if gx< -6||gy< -6||gx>w as i32+6||gy>h as i32+6{return empty;}
        let exact=firing_range.is_none()&&self.passable_profile(gx,gy,profile);
        let end=if exact{gy as usize*w+gx as usize}else{usize::MAX};
        let start=sy*w+sx;
        if start==end{
            // Boarding can require approaching a hardware anchor within the
            // same tile: an unloaded robot may still be outside crew range.
            return if (x-dx).hypot(y-dy)>1.&&self.segment_profile(x,y,dx,dy,profile){
                VecDeque::from([(dx,dy)])
            }else{empty};
        }
        let mut distance=vec![u32::MAX;w*h];let mut prev=vec![usize::MAX;w*h];
        let minimum_cost=if self.map.original {
            (0..=66).filter(|&code|original_terrain::passable(code,self.map.world,profile))
                .map(|code|f64::from(original_terrain::route_cost(code,profile))).fold(5./1.3,f64::min)
        }else{5./1.3};
        let heuristic=|j:usize|->u32{
            if let Some(range)=firing_range{
                let(px,py)=((j%w)as f64*TILE+16.,(j/w)as f64*TILE+16.);
                let radius=wall_target.and_then(|id|self.wall(id)).map(|wall|wall.w.hypot(wall.h)/2.).unwrap_or(TILE*std::f64::consts::SQRT_2);
                return (((px-dx).hypot(py-dy)-range-radius).max(0.)/TILE*1000.*minimum_cost).floor()as u32;
            }
            if !exact{return 0;}
            let ax=((j%w)as i32-gx).unsigned_abs();let ay=((j/w)as i32-gy).unsigned_abs();
            ((ax.min(ay)*1414+(ax.max(ay)-ax.min(ay))*1000)as f64*minimum_cost).floor()as u32
        };
        let mut open=BinaryHeap::new();distance[start]=0;prev[start]=start;
        open.push(Reverse((heuristic(start),0,start)));
        let mut reached=None;let mut fallback:Option<(i64,u32,usize)>=None;
        while let Some(Reverse((_,cost,j)))=open.pop(){
            if cost!=distance[j]{continue;}
            if j==end{reached=Some(j);break;}
            let(cx,cy)=((j%w)as i32,(j/w)as i32);
            if let Some(range)=firing_range{
                let(px,py)=(cx as f64*TILE+16.,cy as f64*TILE+16.);
                let can_fire=if let Some(wall)=wall_target.and_then(|id|self.wall(id)){let point=wall.point_from(px,py);(px-point.0).hypot(py-point.1)<=range&&self.wall_los(px,py,point.0,point.1,wall.id)}else{(px-dx).hypot(py-dy)<=range&&self.los(px,py,dx,dy)};
                if self.passable_profile(cx,cy,profile)&&can_fire{
                    reached=Some(j);break;
                }
            }
            if firing_range.is_none()&&!exact&&(cx-gx).abs()<=6&&(cy-gy).abs()<=6&&self.passable_profile(cx,cy,profile){
                let rank=(i64::from(cx-gx).pow(2)+i64::from(cy-gy).pow(2),cost,j);
                if fallback.is_none_or(|old|rank<old){fallback=Some(rank);}
            }
            for(xx,yy)in[(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1)]{
                let(nx,ny)=(cx+xx,cy+yy);
                if !self.passable_profile(nx,ny,profile)||(xx!=0&&yy!=0&&(!self.passable_profile(cx+xx,cy,profile)||!self.passable_profile(cx,cy+yy,profile))){continue;}
                let k=ny as usize*w+nx as usize;
                let length=if xx!=0&&yy!=0{1414.}else{1000.};
                // Native terrain prices, adapted to this eight-neighbor grid.
                // Consecutive high-price discounting of the native planner is
                // not reproduced by this stateless A* adapter.
                let edge=(length*(self.routing_cost(cx,cy,profile,control)+self.routing_cost(nx,ny,profile,control))/2.).round()as u32;
                let next=cost.saturating_add(edge.max(1));
                if next<distance[k]{distance[k]=next;prev[k]=j;open.push(Reverse((next.saturating_add(heuristic(k)),next,k)));}
            }
        }
        let Some(mut j)=reached.or_else(||fallback.map(|v|v.2))else{return empty;};
        if j==start&&firing_range.is_some(){
            let point=((j%w)as f64*TILE+16.,(j/w)as f64*TILE+16.);
            return if (point.0-x).hypot(point.1-y)>1.&&self.segment_profile(x,y,point.0,point.1,profile){VecDeque::from([point])}else{empty};
        }
        let mut path=Vec::new();
        while j!=start{path.push(((j%w)as f64*TILE+16.,(j/w)as f64*TILE+16.));j=prev[j];}
        path.reverse();path.into()
    }
    #[cfg(test)]
    pub(crate) fn movement_segment_clear(&self,x:f64,y:f64,dx:f64,dy:f64,infantry:bool)->bool{
        self.segment_profile(x,y,dx,dy,if infantry{Profile::Infantry}else{Profile::Tracked})
    }
    pub(crate) fn movement_segment_clear_for(&self,x:f64,y:f64,dx:f64,dy:f64,kind:&str)->bool{
        self.segment_profile(x,y,dx,dy,original_terrain::profile(kind))
    }
    fn segment_profile(&self,x:f64,y:f64,dx:f64,dy:f64,profile:Profile)->bool{
        if ![x,y,dx,dy].iter().all(|v|v.is_finite()){return false;}
        let start=((x/TILE).floor()as i32,(y/TILE).floor()as i32);let mut old=start;
        let samples=((dx-x).hypot(dy-y)/4.).ceil().max(1.)as usize;
        for n in 1..=samples{
            let t=n as f64/samples as f64;let p=(((x+(dx-x)*t)/TILE).floor()as i32,((y+(dy-y)*t)/TILE).floor()as i32);
            // Original spawn/boarding anchors can begin inside a blocked tile;
            // allow leaving that tile, never entering another blocked tile.
            if p!=start&&!self.passable_profile(p.0,p.1,profile){return false;}
            if p.0!=old.0&&p.1!=old.1&&(!self.passable_profile(p.0,old.1,profile)||!self.passable_profile(old.0,p.1,profile)){return false;}
            old=p;
        }
        true
    }
}
