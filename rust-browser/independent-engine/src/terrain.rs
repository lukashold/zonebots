//! Terrain-aware browser navigation. Native permission masks live separately
//! in original_terrain. Native candidate/direct prices and arbitrary-angle
//! waypoint reduction are retained; the grid search and travel-time guard are
//! browser adaptations rather than the complete original movement scheduler.
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
        let exact=firing_range.is_none()&&self.point_profile(dx,dy,profile);
        let end=if exact{gy as usize*w+gx as usize}else{usize::MAX};
        let start=sy*w+sx;
        if start==end{
            // Boarding can require approaching a hardware anchor within the
            // same tile: an unloaded robot may still be outside crew range.
            return if (x-dx).hypot(y-dy)>1.&&self.segment_profile(x,y,dx,dy,profile){
                VecDeque::from([(dx,dy)])
            }else{empty};
        }
        // A cell is revisited by up to eight neighbors and their corner
        // guards. Cache its terrain/building predicate for this one search;
        // live map/building changes naturally invalidate it on the next route.
        let mut permitted=vec![2_u8;w*h];
        let mut passable=|cx:i32,cy:i32|->bool{
            let Some(j)=self.cell(cx,cy)else{return false;};
            if permitted[j]==2{permitted[j]=u8::from(self.passable_profile(cx,cy,profile));}
            permitted[j]==1
        };
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
                if passable(cx,cy)&&can_fire{
                    reached=Some(j);break;
                }
            }
            if firing_range.is_none()&&!exact&&(cx-gx).abs()<=6&&(cy-gy).abs()<=6&&passable(cx,cy){
                let rank=(i64::from(cx-gx).pow(2)+i64::from(cy-gy).pow(2),cost,j);
                if fallback.is_none_or(|old|rank<old){fallback=Some(rank);}
            }
            for(xx,yy)in[(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1)]{
                let(nx,ny)=(cx+xx,cy+yy);
                let goal=exact&&nx==gx&&ny==gy;
                if (!goal&&!passable(nx,ny))||(xx!=0&&yy!=0&&(!passable(cx+xx,cy)||!passable(cx,cy+yy))){continue;}
                let from=if j==start{(x,y)}else{(cx as f64*TILE+16.,cy as f64*TILE+16.)};
                let to=if goal{(dx,dy)}else{(nx as f64*TILE+16.,ny as f64*TILE+16.)};
                if (j==start&&!self.segment_profile(from.0,from.1,to.0,to.1,profile))
                    ||(j!=start&&!self.building_segment_clear(from.0,from.1,to.0,to.1)){continue;}
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
        path.reverse();
        if exact {
            // A click is a pixel position, not merely the containing tile.
            let previous=if path.len()>1 {path[path.len()-2]}else{(x,y)};
            if self.segment_profile(previous.0,previous.1,dx,dy,profile) {
                if let Some(last)=path.last_mut(){*last=(dx,dy);}
            }else if let Some(&last)=path.last() {
                if self.segment_profile(last.0,last.1,dx,dy,profile) {path.push((dx,dy));}
                else {
                    // A non-grid-aligned building can occupy part of the goal
                    // cell. Try a legal in-cell elbow rather than cutting it.
                    for elbow in [(last.0,dy),(dx,last.1)] {
                        if self.segment_profile(last.0,last.1,elbow.0,elbow.1,profile)
                            && self.segment_profile(elbow.0,elbow.1,dx,dy,profile) {
                            path.push(elbow);path.push((dx,dy));break;
                        }
                    }
                }
            }
        }
        self.reduce_route((x,y),path,profile,control)
    }
    #[cfg(test)]
    pub(crate) fn movement_segment_clear(&self,x:f64,y:f64,dx:f64,dy:f64,infantry:bool)->bool{
        self.segment_profile(x,y,dx,dy,if infantry{Profile::Infantry}else{Profile::Tracked})
    }
    pub(crate) fn movement_segment_clear_for(&self,x:f64,y:f64,dx:f64,dy:f64,kind:&str)->bool{
        self.segment_profile(x,y,dx,dy,original_terrain::profile(kind))
    }
    /// Cells touched by a continuous segment, with its exact length in each.
    /// Simultaneous boundary crossings emit both side cells at zero length:
    /// unlike fixed-distance sampling, even a subpixel water corner is tested.
    fn segment_cells(&self,x:f64,y:f64,dx:f64,dy:f64)->Option<Vec<(i32,i32,f64)>>{
        if ![x,y,dx,dy].iter().all(|v|v.is_finite()){return None;}
        let(vx,vy)=(dx-x,dy-y);let length=vx.hypot(vy);
        let(mut cx,mut cy)=((x/TILE).floor()as i32,(y/TILE).floor()as i32);
        let ex=(dx/TILE).floor()as i32;let ey=(dy/TILE).floor()as i32;
        let sx=if vx>0.{1}else if vx<0.{-1}else{0};let sy=if vy>0.{1}else if vy<0.{-1}else{0};
        let step_x=if sx==0{f64::INFINITY}else{TILE/vx.abs()};
        let step_y=if sy==0{f64::INFINITY}else{TILE/vy.abs()};
        let mut tx=if sx>0{((cx+1)as f64*TILE-x)/vx}else if sx<0{(cx as f64*TILE-x)/vx}else{f64::INFINITY};
        let mut ty=if sy>0{((cy+1)as f64*TILE-y)/vy}else if sy<0{(cy as f64*TILE-y)/vy}else{f64::INFINITY};
        let mut t=0.;let mut cells=Vec::new();
        // Valid maps are at most128x128; reject absurd external coordinates.
        if cx.abs()>512||cy.abs()>512||ex.abs()>512||ey.abs()>512{return None;}
        for _ in 0..4096 {
            let next=tx.min(ty).min(1.).max(t);
            cells.push((cx,cy,(next-t)*length));
            if next>=1.-1e-12 {
                if (tx-ty).abs()<1e-12&&tx<=1.+1e-12 {
                    cells.push((cx+sx,cy,0.));cells.push((cx,cy+sy,0.));cells.push((cx+sx,cy+sy,0.));
                }
                if (cx,cy)!=(ex,ey){cells.push((ex,ey,0.));}return Some(cells);
            }
            let corner=(tx-ty).abs()<1e-12;
            if corner {
                cells.push((cx+sx,cy,0.));cells.push((cx,cy+sy,0.));
                cx+=sx;cy+=sy;tx+=step_x;ty+=step_y;
            }else if tx<ty {cx+=sx;tx+=step_x;}else{cy+=sy;ty+=step_y;}
            t=next;
        }
        None
    }
    fn point_profile(&self,x:f64,y:f64,profile:Profile)->bool{
        self.terrain_open((x/TILE).floor()as i32,(y/TILE).floor()as i32,profile)
            &&!self.buildings.iter().any(|b|b.hp>0.&&x>b.x&&x<b.x+b.w&&y>b.y&&y<b.y+b.h
                &&!b.gate.map(|(gx,gy)|(x-gx).abs()<24.&&(if b.orientation=="north"{y<=gy+24.}else{y>=gy-24.})).unwrap_or(false))
    }
    fn building_segment_clear(&self,x:f64,y:f64,dx:f64,dy:f64)->bool{
        // Open rectangle clipping protects actual building edges, rather than
        // only the centers of their raster cells. A gate subtracts its existing
        // entry corridor. Starting inside a solid permits departure only.
        fn hits(x:f64,y:f64,dx:f64,dy:f64,l:f64,t:f64,r:f64,b:f64)->bool{
            if l>=r||t>=b{return false;}
            let inside=x>l&&x<r&&y>t&&y<b;
            let(mut enter,mut leave)=(0_f64,1_f64);
            for (p,v,min,max) in [(x,dx-x,l,r),(y,dy-y,t,b)] {
                if v.abs()<1e-12 {if p<=min||p>=max{return false;}}
                else {let a=(min-p)/v;let z=(max-p)/v;enter=enter.max(a.min(z));leave=leave.min(a.max(z));}
            }
            leave-enter>1e-10&&!inside
        }
        let(min_x,min_y,max_x,max_y)=(x.min(dx),y.min(dy),x.max(dx),y.max(dy));
        for b in self.buildings.iter().filter(|b|b.hp>0.) {
            let(l,t,r,bottom)=(b.x,b.y,b.x+b.w,b.y+b.h);
            // Most search edges are nowhere near a building. Reject disjoint
            // bounds before clipping and avoid allocating rectangles per edge.
            if max_x<=l||min_x>=r||max_y<=t||min_y>=bottom{continue;}
            if let Some((gx,gy))=b.gate {
                let(left,right)=((gx-24.).clamp(l,r),(gx+24.).clamp(l,r));
                if hits(x,y,dx,dy,l,t,left,bottom)||hits(x,y,dx,dy,right,t,r,bottom){return false;}
                let cap=if b.orientation=="north"{(left,(gy+24.).clamp(t,bottom),right,bottom)}else{(left,t,right,(gy-24.).clamp(t,bottom))};
                if hits(x,y,dx,dy,cap.0,cap.1,cap.2,cap.3){return false;}
            }else if hits(x,y,dx,dy,l,t,r,bottom){return false;}
        }
        true
    }
    fn segment_profile(&self,x:f64,y:f64,dx:f64,dy:f64,profile:Profile)->bool{
        let Some(cells)=self.segment_cells(x,y,dx,dy)else{return false;};
        let start=((x/TILE).floor()as i32,(y/TILE).floor()as i32);
        cells.into_iter().all(|(cx,cy,_)|(cx,cy)==start||self.terrain_open(cx,cy,profile))
            &&self.building_segment_clear(x,y,dx,dy)
    }
    fn segment_time(&self,a:(f64,f64),b:(f64,f64),profile:Profile,control:u8)->f64{
        self.segment_cells(a.0,a.1,b.0,b.1).map(|cells|cells.into_iter()
            .map(|(x,y,length)|length/self.factor_profile(x,y,profile,control).max(0.001)).sum()).unwrap_or(f64::INFINITY)
    }
    /// 0045cf90/0045cdc0 compare dominant-axis raster cells, not Euclidean
    /// distance. The final cell is excluded, and high-price runs reset per leg.
    pub(crate) fn native_leg_price(&self,a:(f64,f64),b:(f64,f64),profile:Profile)->u64{
        let(mut x,mut y)=((a.0/TILE).floor()as i32,(a.1/TILE).floor()as i32);
        let(mut ex,mut ey)=((b.0/TILE).floor()as i32,(b.1/TILE).floor()as i32);
        if x>=ex{std::mem::swap(&mut x,&mut ex);std::mem::swap(&mut y,&mut ey);}
        let ax=(ex-x).abs();let ay=(ey-y).abs();let sy=if ey>y{1}else{-1};
        let steps=ax.max(ay);let mut error=steps/2;let mut high=false;let mut price=0_u64;
        for _ in 0..steps {
            let code=self.cell(x,y).and_then(|j|self.map.terrain_codes.get(j)).copied().unwrap_or(127);
            let raw=original_terrain::route_cost(code,profile);
            price+=u64::from(if raw>=10&&high{raw/2}else{raw});high=raw>=10;
            if ax>ay {x+=1;error-=ay;if error<0{y+=sy;error+=ax;}}
            else {y+=sy;error-=ax;if error<0{x+=1;error+=ay;}}
        }
        price
    }
    fn reduce_route(&self,start:(f64,f64),path:Vec<(f64,f64)>,profile:Profile,control:u8)->VecDeque<(f64,f64)>{
        if path.is_empty(){return VecDeque::new();}
        // First remove redundant collinear grid centers without changing the
        // route geometry. This keeps later shortcut probes bounded on mazes.
        let mut points=vec![start];
        for point in path {
            while points.len()>=2 {
                let a=points[points.len()-2];let b=points[points.len()-1];
                let(u,v)=((b.0-a.0,b.1-a.1),(point.0-b.0,point.1-b.1));
                if (u.0*v.1-u.1*v.0).abs()>1e-8||u.0*v.0+u.1*v.1<0.{break;}
                points.pop();
            }
            if points.last()!=Some(&point){points.push(point);}
        }
        let mut prefix=vec![0.;points.len()];
        for i in 1..points.len(){prefix[i]=prefix[i-1]+self.segment_time(points[i-1],points[i],profile,control);}
        let mut retained=Vec::new();let mut i=0;
        while i+1<points.len(){
            let mut next=i+1;
            for j in ((i+2)..points.len().min(i+98)).rev(){
                if self.segment_profile(points[i].0,points[i].1,points[j].0,points[j].1,profile)
                    &&self.segment_time(points[i],points[j],profile,control)<=prefix[j]-prefix[i]+1e-7{next=j;break;}
            }
            retained.push(points[next]);i=next;
        }
        let goal=*retained.last().unwrap();
        if self.segment_profile(start.0,start.1,goal.0,goal.1,profile){
            let mut previous=start;let mut candidate_time=0.;let mut candidate_price=0;
            for &point in &retained {candidate_time+=self.segment_time(previous,point,profile,control);candidate_price+=self.native_leg_price(previous,point,profile);previous=point;}
            let direct_time=self.segment_time(start,goal,profile,control);
            // Native ba60 prefers the direct candidate on equal raster prices.
            // A separate physical-time guard prevents our A* candidate from
            // choosing a road detour slower than an actually legal shortcut.
            if (self.map.original&&self.native_leg_price(start,goal,profile)<=candidate_price)
                ||direct_time<=candidate_time+1e-7{return VecDeque::from([goal]);}
        }
        retained.into()
    }
}
