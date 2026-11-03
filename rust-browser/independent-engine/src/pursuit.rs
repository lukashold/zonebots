//! Live explicit-target pursuit. Native infantry use52ab0's callback counter;
//! the procedural game retains its elapsed-time replanning adapter.
use super::{original_combat, Game, Unit};

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) target:usize,
    pub(crate) planned_goal:(f64,f64),
    pub(crate) next_plan:f64,
    pub(crate) revision:u32,
    pub(crate) native_counter:Option<i8>,
}

impl Game {
    ///A native UNIT slot survives hardware takeover. Browser entity IDs use a
    ///separate neutral-hardware descriptor, so preserve live attack windups and
    ///replanning counters when that stable native slot acquires the new ID.
    pub(crate) fn remap_native_pursuit_identity(&mut self,previous:usize,current:usize){
        for unit in &mut self.units{
            if let Some(hardware)=unit.native_hardware.as_mut(){hardware.remap_target(previous,current);}
            if unit.order=="attack"&&unit.target==Some(previous){unit.target=Some(current);}
            if let Some(pursuit)=unit.pursuit.as_mut(){if pursuit.target==previous{pursuit.target=current;}}
            if let Some(fire)=unit.native_fire.as_mut(){if fire.target==previous{fire.target=current;}}
        }
    }
    /// Returns true while the chosen enemy can be fired upon from here.
    pub(crate) fn update_pursuit(&mut self,index:usize,snapshot:&[Unit],range:f64,speed:f64)->bool {
        let unit=self.units[index].clone();
        if unit.order=="attackWall" {return false;}
        if unit.order!="attack"||unit.target.is_none(){self.units[index].pursuit=None;return false;}
        let source=if self.map.original{self.units.as_slice()}else{snapshot};
        let target=unit.target.and_then(|id|source.iter().find(|u|u.id==id
            &&original_combat::alive(self.map.original,u.hp)&&!super::workshops::hidden(u)&&u.team>=0&&u.team!=unit.team)).cloned();
        let Some(target)=target.as_ref()else{
            let u=&mut self.units[index];u.order="guard".into();u.target=None;
            u.dest=None;u.path.clear();u.pursuit=None;u.moving=false;return false;
        };
        let goal=(target.x,target.y);
        self.units[index].dest=Some(goal);
        let can_fire=(if self.map.original{self.native_target_in_range(&unit,target,range)}else{(unit.x-target.x).hypot(unit.y-target.y)<=range})
            &&self.unit_target_los(&unit,target);
        if can_fire {
            let u=&mut self.units[index];u.path.clear();
            // Leaving range or losing LOS must immediately create a fresh route.
            u.pursuit=None;return true;
        }
        if speed<=0.{self.units[index].path.clear();self.units[index].pursuit=None;return false;}
        //52ab0/48250 belong to the pursuit handler. Infantry fire callbacks
        //do not spend that counter while their separate windup runs.
        if self.map.original&&super::stats(&unit.kind).cls==0&&unit.native_fire.is_some(){return false;}
        let previous=unit.pursuit.as_ref();
        let new_target=previous.is_none_or(|s|s.target!=target.id);
        let revised=previous.is_some_and(|s|s.revision!=self.map.revision);
        let moved=previous.is_some_and(|s|(s.planned_goal.0-goal.0).hypot(s.planned_goal.1-goal.1)>=8.);
        let native=self.map.original&&super::stats(&unit.kind).cls==0;
        let due=if native{previous.is_none_or(|s|s.native_counter.is_none())||unit.native_counter68<0}else{previous.is_none_or(|s|self.time>=s.next_plan)};
        if native {let next=self.units[index].native_counter68.wrapping_sub(1);if let Some(state)=self.units[index].pursuit.as_mut(){if state.native_counter.is_some(){self.units[index].native_counter68=next;self.units[index].pursuit.as_mut().unwrap().native_counter=Some(next);}}}
        if new_target||revised||(due&&(native||moved||unit.path.is_empty())) {
            let path=if native&&self.map.native_road_graph.is_some(){self.route_unit(&unit,(target.x/2.).floor()*2.-16.,(target.y/2.).floor()*2.-16.)}else{self.route_to_unit_attack(&unit,target,range)};
            let delay=if path.is_empty(){0.75}else{0.25};
            let native_counter=native.then(||super::original_pursuit::refresh_counter(false,Some(target.native_class7c),true,
                unit.original_combat.as_ref().map_or(60,|s|s.aim_skill)as i8,
                [(unit.x/2.).floor()as i16,(unit.y/2.).floor()as i16],[(target.x/2.).floor()as i16,(target.y/2.).floor()as i16]));
            let u=&mut self.units[index];u.path=path;if let Some(counter)=native_counter{u.native_counter68=counter;}
            u.original_motion=None;
            u.pursuit=Some(State{target:target.id,planned_goal:goal,next_plan:self.time+delay,revision:self.map.revision,native_counter});
        }
        false
    }
}
