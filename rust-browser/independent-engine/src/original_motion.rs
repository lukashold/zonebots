//! Movement facts verified against the local GOG executable. Its SDL loop uses
//! a 1000/17 ms threshold with integer SDL ticks (59 ms); 00435ad8 advances all
//! shared pattern cursors before unit updates. 004525a0 and 00451da0..004520a0
//! use integer, major-axis stepping, not normalized vector interpolation.
use super::{Game, original_combat, original_terrain};

pub const TICK_SECONDS:f64=0.059;
const WORLD_SCALE:f64=2.;
const STEPS:[&[u8];13]=[
    &[1,0,0,0,0], &[1,0,0,0], &[1,0,0], &[1,0,0,1,0], &[1,0],
    &[1,1,0], &[1,1,1,1,0], &[1], &[2,1,1], &[2,1], &[2], &[3,2,2], &[3,2],
];

pub fn index(control:u8,code:u8)->usize {
    (i32::from(control)/10+original_terrain::movement_modifier(code)/10).clamp(0,12)as usize
}
pub fn speed(control:u8)->f64 {
    let pattern=STEPS[index(control,0)];
    pattern.iter().map(|&v|f64::from(v)).sum::<f64>()/pattern.len()as f64*WORLD_SCALE/TICK_SECONDS
}
fn coordinate(v:f64)->i32 {(v/WORLD_SCALE).floor()as i32}

#[derive(Clone,Default)]
pub(crate) struct Segment {
    goal:(i32,i32), position:(i32,i32), major:i32, minor:i32, remaining:i32,
    error:i32, sx:i32, sy:i32, x_major:bool, heading:f64,
}
impl Segment {
    fn new(x:f64,y:f64,goal:(f64,f64))->Self {
        let position=(coordinate(x),coordinate(y));let goal=(coordinate(goal.0),coordinate(goal.1));
        let dx=goal.0-position.0;let dy=goal.1-position.1;
        let x_major=dx.abs()>=dy.abs();let major=dx.abs().max(dy.abs());
        Self{goal,position,major,minor:dx.abs().min(dy.abs()),remaining:major,error:major/2,sx:dx.signum(),sy:dy.signum(),x_major,heading:f64::from(dy).atan2(f64::from(dx))}
    }
    fn advance(&mut self,step:u8) {
        for _ in 0..i32::from(step).min(self.remaining) {
            self.error-=self.minor;let diagonal=self.error<0;
            if diagonal{self.error+=self.major;}
            if self.x_major {self.position.0+=self.sx;if diagonal{self.position.1+=self.sy;}}
            else {self.position.1+=self.sy;if diagonal{self.position.0+=self.sx;}}
            self.remaining-=1;
        }
    }
}

impl Game {
    pub(crate) fn move_original(&mut self,i:usize,first_tick:u64,ticks:u64,aim_follows:bool) {
        self.units[i].moving=!self.units[i].path.is_empty();
        for tick in first_tick..first_tick+ticks {
            let Some(goal)=self.units[i].path.front().copied()else{self.units[i].original_motion=None;break;};
            let u=&self.units[i];
            let control=u.original_combat.as_ref().map(|s|s.movement_control)
                .unwrap_or_else(||original_combat::CombatState::new(&u.kind).movement_control);
            // Native terrain lookup samples the anchor plus eight native pixels.
            let col=((u.x+16.)/32.).floor()as usize;let row=((u.y+16.)/32.).floor()as usize;
            let code=if col<self.map.width&&row<self.map.height {
                self.map.terrain_codes.get(row*self.map.width+col).copied().unwrap_or(0)
            }else{0};
            let pattern=STEPS[index(control,code)];let step=pattern[tick as usize%pattern.len()];
            if step==0{continue;}
            let mut segment=u.original_motion.clone().filter(|s|s.goal==(coordinate(goal.0),coordinate(goal.1))
                &&s.position==(coordinate(u.x),coordinate(u.y)))
                .unwrap_or_else(||Segment::new(u.x,u.y,goal));
            let old=segment.position;segment.advance(step);
            let (x,y)=(f64::from(segment.position.0)*WORLD_SCALE,f64::from(segment.position.1)*WORLD_SCALE);
            // Keep the existing live collision adapter; a rounded native step
            // must never sneak across forbidden water or building boundaries.
            if !self.movement_segment_clear_for(u.x,u.y,x,y,&u.kind){self.units[i].original_motion=None;break;}
            let u=&mut self.units[i];u.x=x;u.y=y;
            if segment.position!=old {
                u.angle=segment.heading;
                u.moving=true;if aim_follows{u.aim=u.angle;}
            }
            if segment.remaining==0 {
                // Native movement truncates this tick's step at the waypoint.
                // It starts the next segment without spending the excess.
                u.path.pop_front();u.original_motion=None;
            }else{u.original_motion=Some(segment);}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_segments_match_an_independent_major_axis_reference_in_every_octant() {
        for dx in -13_i32..=13 {for dy in -13_i32..=13 {
            if dx==0&&dy==0{continue;}
            let mut s=Segment::new(200.,200.,(200.+f64::from(dx)*2.,200.+f64::from(dy)*2.));
            let major=dx.abs().max(dy.abs());
            for n in 1..=major {
                s.advance(1);
                // Nearest integer with exact-half ties kept on the major axis.
                let minor=(2*n*dx.abs().min(dy.abs())+major-1)/(2*major);
                let expected=if dx.abs()>=dy.abs(){(100+n*dx.signum(),100+minor*dy.signum())}
                    else{(100+minor*dx.signum(),100+n*dy.signum())};
                assert_eq!(s.position,expected,"delta {dx},{dy}, step {n}");
            }
            assert_eq!(s.remaining,0);
        }}
    }
}
