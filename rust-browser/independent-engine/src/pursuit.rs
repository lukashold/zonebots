//! Live explicit-target pursuit. These replanning intervals are browser-engine
//! choices, not a claim that the original Windows AI has been reproduced.
use super::{original_combat, Game, Unit};

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) target:usize,
    pub(crate) planned_goal:(f64,f64),
    pub(crate) next_plan:f64,
    pub(crate) revision:u32,
}

impl Game {
    /// Returns true while the chosen enemy can be fired upon from here.
    pub(crate) fn update_pursuit(&mut self,index:usize,snapshot:&[Unit],range:f64,speed:f64)->bool {
        let unit=self.units[index].clone();
        if unit.order=="attackWall" {return false;}
        if unit.order!="attack"||unit.target.is_none(){self.units[index].pursuit=None;return false;}
        let target=unit.target.and_then(|id|snapshot.iter().find(|u|u.id==id
            &&original_combat::alive(self.map.original,u.hp)&&u.team>=0&&u.team!=unit.team));
        let Some(target)=target else{
            let u=&mut self.units[index];u.order="guard".into();u.target=None;
            u.dest=None;u.path.clear();u.pursuit=None;u.moving=false;return false;
        };
        let goal=(target.x,target.y);
        self.units[index].dest=Some(goal);
        let can_fire=(unit.x-target.x).hypot(unit.y-target.y)<=range
            &&self.los(unit.x,unit.y,target.x,target.y);
        if can_fire {
            let u=&mut self.units[index];u.path.clear();
            // Leaving range or losing LOS must immediately create a fresh route.
            u.pursuit=None;return true;
        }
        if speed<=0.{self.units[index].path.clear();self.units[index].pursuit=None;return false;}
        let previous=unit.pursuit.as_ref();
        let new_target=previous.is_none_or(|s|s.target!=target.id);
        let revised=previous.is_some_and(|s|s.revision!=self.map.revision);
        let moved=previous.is_some_and(|s|(s.planned_goal.0-goal.0).hypot(s.planned_goal.1-goal.1)>=8.);
        let due=previous.is_none_or(|s|self.time>=s.next_plan);
        if new_target||revised||(due&&(moved||unit.path.is_empty())) {
            let path=self.route_to_attack(&unit,goal.0,goal.1,range);
            let delay=if path.is_empty(){0.75}else{0.25};
            let u=&mut self.units[index];u.path=path;
            u.pursuit=Some(State{target:target.id,planned_goal:goal,next_plan:self.time+delay,revision:self.map.revision});
        }
        false
    }
}
