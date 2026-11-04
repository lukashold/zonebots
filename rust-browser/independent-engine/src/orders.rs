//! Validated interaction lifecycles and structure firing positions.
use super::*;
impl Game {
    ///Mirror live writes to the allocated native W record. The saved crew
    ///and range bytes keep their own constructor/transport owners.
    pub(crate) fn sync_native_hardware_combat(&mut self,index:usize){let Some(u)=self.units.get(index)else{return;};if !self.map.original{return;}let Some(w)=u.native_hardware_slot.and_then(|s|self.native_hardware_pool.get_mut(s)).filter(|w|w.live_marker>=0)else{return;};w.attributes.set_word(8,u.hp as i16);w.attributes.set_word(10,u.max_hp as i16);if let Some(c)=&u.original_combat{w.attributes.bytes[7]=c.movement_control;w.attributes.bytes[32]=c.fire_rate;if let Some(speed)=c.projectile_speed{w.attributes.bytes[33]=speed;}}}

    ///54160 is visited once before the native phase handler. Factory exit
    ///owns its own visit while its source protection sequence is active.
    pub(crate) fn dispatch_recorded_native_order(&mut self,index:usize){
        let Some(op)=self.prepare_native_order_dispatch(index,false)else{return;};
        let slot=self.units[index].native_hardware_slot.filter(|&s|s<self.native_hardware_pool.len());
        match op{
            1=>{self.dispatch_native_move_order(index);},
            12=>{if let Err(boundary)=self.dispatch_native_point_order(index){self.native_route_boundary=Some(boundary);}},
            4=>{let mut record=slot.map(|s|self.native_hardware_pool[s].order).unwrap_or(self.native_hardware_dummy);self.complete_native_stop_order(index,Some(&mut record));if let Some(s)=slot{self.native_hardware_pool[s].set_order(record);}else{self.native_hardware_dummy=record;}},
            2=>{let mut record=slot.map(|s|self.native_hardware_pool[s].order);let movement=slot.map_or(0,|s|self.native_hardware_pool[s].attributes.byte(7));let aim=self.units[index].native_control06;self.begin_native_hardware_order(index,record.as_mut(),movement,aim);if let(Some(s),Some(record))=(slot,record){self.native_hardware_pool[s].set_order(record);}},
            5=>{if let Some(result)=self.dispatch_native_attack_order(index){
                use super::original_attack_order::AttackOrderResult as Result;
                match result{
                    Result::Live(target)=>{self.units[index].order="attack".into();self.units[index].target=Some(target);},
                    Result::Ground(_)=>{if self.units[index].order=="attack"{self.units[index].target=None;}},
                    Result::Move=>{self.units[index].target=None;},
                    Result::Cancelled=>{let u=&mut self.units[index];u.order="guard".into();u.target=None;u.dest=None;u.path.clear();u.original_motion=None;u.moving=false;},
                    Result::Ignored=>{},
                }
            }else{self.native_infantry_target_order_entered(index);if let Some(target)=self.accept_native_infantry_target_order(index){self.units[index].target=Some(target);}}},
            _=>{},
        }
    }
    pub(crate) fn bridge_capacity(&self,id:usize)->f64 {
        self.map.bridge_max_hp.get(id).copied().filter(|hp|hp.is_finite()&&*hp>0.).unwrap_or(400.)
    }
    pub(crate) fn finish_order(&mut self,index:usize) {
        if self.map.original{self.release_original_claim(index);}
        let u=&mut self.units[index];u.order="guard".into();u.target=None;u.dest=None;
        u.path.clear();u.original_motion=None;u.native_walk=None;u.native_fire=None;u.pursuit=None;u.initiative=None;u.motion_bonus=0;u.march_resume=None;u.moving=false;u.hold_position=self.map.original;u.native_route_pending.set(None);if self.map.original&&stats(&u.kind).cls==0{u.native_phase35=0;}
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
        let b=u.target.and_then(|id|self.buildings.iter().find(|b|b.id==id&&b.alive(self.map.original)&&b.team!=u.team)).cloned();
        let Some(b)=b else{self.finish_order(index);return false;};
        if self.building_can_fire(&u,&b,range){self.units[index].path.clear();return true;}
        if speed<=0.{self.units[index].path.clear();return false;}
        if self.units[index].path.is_empty(){self.units[index].path=self.route_to_building_attack(&u,&b,range);}
        false
    }
}

#[cfg(test)]mod native_event_integration_tests{
 use super::*;
 fn field()->Game{let mut g=Game::new(0,1,31);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.ai_teams.clear();g.winner=None;g.next_id=1;g.spawn("grunt",0,320.,320.,71,None);g.spawn("gatling",1,1800.,1200.,72,None);g.maintain_native_group_rosters();g}
 #[test]fn accepted_move_and_stop_use_source_opcodes_and_preserve_protection(){let mut g=field();let id=g.units[0].id;g.units[0].native_protectionaa=3;g.command(&json!({"team":0,"ids":[id],"kind":"move","x":640.,"y":320.}));assert_eq!(g.units[0].native_order5c,1);assert_eq!(g.units[0].native_flags9c&32,32);assert_eq!(g.native_order_arguments,[328,168,0]);assert_eq!(g.units[0].native_protectionaa,3);g.step(0.059);assert_eq!(g.units[0].native_order5c,-1);assert_eq!(g.units[0].native_protectionaa,2);g.command(&json!({"team":0,"ids":[id],"kind":"stop"}));assert_eq!(g.units[0].native_order5c,4);assert_eq!(g.native_order_arguments,[328,168,0]);g.step(0.059);assert_eq!(g.units[0].native_order5c,0);assert_eq!(g.units[0].native_flags9c&32,0);assert_eq!(g.native_hardware_dummy.selected_target12,255);assert_eq!(g.units[0].native_protectionaa,1);}
 #[test]fn rejected_and_explicitly_adapter_owned_orders_do_not_write_native_events(){let mut g=field();let id=g.units[0].id;let before=g.native_order_arguments;g.command(&json!({"team":0,"ids":[id],"kind":"attack","target":999999}));assert_eq!(g.native_order_arguments,before);assert_eq!(g.units[0].native_order5c,0);g.command(&json!({"team":0,"ids":[id],"kind":"move","x":640.,"y":320.,"nativeEvent":false}));assert_eq!(g.native_order_arguments,before);assert_eq!(g.units[0].native_order5c,0);assert!(!g.units[0].path.is_empty());}
}
