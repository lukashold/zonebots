//! Movement facts verified against the local GOG executable. Its SDL loop uses
//! a 1000/17 ms threshold with integer SDL ticks (59 ms); 00435ad8 advances all
//! shared pattern cursors before unit updates. 004525a0 and 00451da0..004520a0
//! use integer, major-axis stepping, not normalized vector interpolation.
use super::{Game, original_combat, original_terrain};
use serde::{Serialize,Deserialize};

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
const WALK_COUNTER:[i8;13]=[10,8,6,5,5,4,3,2,2,2,1,1,1];
#[derive(Clone,Serialize,Deserialize)]
pub struct Walk{pub frame:i16,timer:i8}
impl Walk{
    fn new(index:usize)->Self{Self{frame:0,timer:WALK_COUNTER[index]}}
    ///525a0 tests only the LOW frame byte. Unusual high-byte values survive
    ///until the walking handler's next increment masks the word to0..3.
    pub(crate) fn replacement(index:usize,frame:i16,timer:i8)->Self{Self{frame:if frame as u8&252!=0{0}else{frame},timer:if timer<1||WALK_COUNTER[index]<=timer{WALK_COUNTER[index]}else{timer}}}
    fn advance(&mut self,index:usize){self.timer=self.timer.wrapping_sub(1);if self.timer<0{self.timer=WALK_COUNTER[index];self.frame=self.frame.wrapping_add(1)&3;}}
}

#[derive(Clone,Default)]
pub(crate) struct Segment {
    goal:(i32,i32), position:(i32,i32), major:i32, minor:i32, remaining:i32,
    error:i32, sx:i32, sy:i32, x_major:bool, heading:f64,
}
impl Segment {
    pub(crate) fn correct_contact_position(&mut self,p:[i16;2]){self.position=(i32::from(p[0]),i32::from(p[1]));}
    pub(crate) fn axis81(&self)->u8{if self.x_major{if self.sx>0{0}else{4}}else if self.sx>0{2}else{6}}
    pub(crate) fn new(x:f64,y:f64,goal:(f64,f64))->Self {
        let position=(coordinate(x),coordinate(y));let goal=(coordinate(goal.0),coordinate(goal.1));
        let dx=goal.0-position.0;let dy=goal.1-position.1;
        let x_major=dx.abs()>=dy.abs();let major=dx.abs().max(dy.abs());
        Self{goal,position,major,minor:dx.abs().min(dy.abs()),remaining:major,error:major/2,sx:dx.signum(),sy:dy.signum(),x_major,heading:super::native_math::angle(super::native_math::direction(dx,dy))}
    }
    pub(crate) fn heading(&self)->f64 {self.heading}
    pub(crate) fn remaining(&self)->i16 {self.remaining as i16}
    ///4490b0's direct525a0 attack approach shortens only48, preserving4a's
    ///original dominant-axis length and line error for subsequent stepping.
    pub(crate) fn shorten_for_attack(&mut self){let left=(self.remaining as i16)>>4;self.remaining=i32::from(if left==0{1}else{left});}
    #[cfg(test)]pub(crate) fn set_source_fixture(&mut self,f:[i16;4]){self.remaining=i32::from(f[0]);self.major=i32::from(f[1]);self.error=i32::from(f[2]);self.minor=i32::from(f[3]);}
    #[cfg(test)]pub(crate) fn source_line_fields(&self)->[i16;4]{[self.remaining as i16,self.major as i16,self.error as i16,self.minor as i16]}
    #[cfg(test)]pub(crate) fn source_vertical_sign(&self)->u8{if self.sy<0{255}else{1}}
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

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum LocalResult{Moving,Arrived,Blocked}
#[derive(Clone,Copy,Debug,Default)]pub(crate) struct WalkStep{pub moved:bool,pub ended:bool,pub contact:Option<super::original_collision::Contact>}

impl Game {
    ///41b3bd stores the initial bucket before the first4f1e0 contextupdate.
    pub(crate) fn initialize_native_walk_fields(&mut self,i:usize){let Some(u)=self.units.get_mut(i)else{return;};if self.map.original&&u.native_slot.is_some(){u.native_directiona2=-1;u.native_directiona3=0;u.native_speed8b=-1;u.native_bucket38=(((coordinate(u.y)>>3)as u8)&240).wrapping_add((coordinate(u.x)>>7)as u8);}}
    ///4f572 copies the selected global movement pattern's current step to62.
    ///Call at the existing4f1e0 constructor/5a080 position-context boundary.
    pub(crate) fn refresh_native_pattern_step(&mut self,i:usize,tick:u64){
        self.refresh_native_pattern_context(i,tick,false);
    }
    ///5a080's4f1e0 stage: cell/69:4, Crane contact, then51d40/pattern62.
    ///The generic constructor refresh deliberately has no contact callback.
    pub(crate) fn refresh_native_main_pattern_step(&mut self,i:usize,tick:u64){
        self.refresh_native_pattern_context(i,tick,true);
    }
    fn refresh_native_pattern_context(&mut self,i:usize,tick:u64,main_contact:bool){
        let Some(u)=self.units.get(i)else{return;};if !self.map.original||u.native_slot.is_none(){return;}let x=if u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside){10000}else{coordinate(u.x)};let y=coordinate(u.y);let slot=u.native_slot.unwrap();let cell=((y as u32).wrapping_mul(8).wrapping_add(64)&0x7ff80).wrapping_add((x.wrapping_add(8)>>4)as u32);self.units[i].native_cell10=if x!=10000&&cell<0x4001{cell as i32}else{0};if x==10000{return;}if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags69|=4;}
        if main_contact&&cell<0x4001{self.refresh_native_crane_main_contact(i);}
        let u=&self.units[i];
        let control=u.original_combat.as_ref().map(|s|s.movement_control).unwrap_or_else(||original_combat::CombatState::new(&u.kind).movement_control);
        let code=self.native_terrain(coordinate(u.x)+8,coordinate(u.y)+8);
        let motion_index=(index(control,code)as i32+i32::from(u.motion_bonus)+i32::from(u.native_group_motion_bonus)).clamp(0,12)as usize;
        if self.units[i].native_speed8b!=motion_index as i8{self.units[i].native_pattern20=Some(crate::original_retired_unit_preamble::PATTERNS[motion_index]);}self.units[i].native_speed8b=motion_index as i8;let actual=self.units[i].native_pattern20.and_then(|p|crate::original_retired_unit_preamble::PATTERNS.iter().position(|&token|token==p)).unwrap_or(motion_index);let pattern=STEPS[actual];self.units[i].native_field62=pattern[tick as usize%pattern.len()];
    }
    ///Directional movement leaf only.452200/48060 own their counter update
    ///and subsequent acquisition, so neither is repeated here. Contact and
    ///line-end handling remain visible events for the native path-poolowner.
    pub(crate) fn advance_native_walk_position(&mut self,i:usize)->Option<WalkStep>{
        let u=self.units.get(i)?;let old=(coordinate(u.x)as i16,coordinate(u.y)as i16);
        let segment=u.original_motion.clone().filter(|s|s.position==(i32::from(old.0),i32::from(old.1))).or_else(||u.path.front().copied().map(|goal|Segment::new(u.x,u.y,goal)));
        let Some(mut segment)=segment else{return(u.native_remaining48==0).then_some(WalkStep{ended:u.native_field62!=0,..Default::default()});};
        self.units[i].native_axis81=segment.axis81();
        let step=self.units[i].native_field62;if step==0{self.units[i].original_motion=Some(segment);return Some(WalkStep::default());}
        let mut next=segment.clone();next.advance(step);let attempted=(next.position.0 as i16,next.position.1 as i16);
        let vehicle=self.units[i].native_class7c==1;
        let (hit,contact)=if self.map.original&&self.units[i].native_slot.is_some(){match self.native_wrapped_collision(i,[attempted.0,attempted.1]){Ok(v)=>v,Err(e)=>{self.native_route_boundary=Some(e);return None;}}}else{
            let u=&self.units[i];let mask=if super::stats(&u.kind).cls==0{super::original_collision::MaskKind::Foot}else if u.kind=="crane"{super::original_collision::MaskKind::Crane}else{super::original_collision::MaskKind::Vehicle};let team=u.team as u8&3;let octant=super::original_combat::facing(u.angle);let mut c=super::original_collision::Contact::new(old,attempted);let hit=c.wrapped((attempted.0.wrapping_sub(old.0)as i8,attempted.1.wrapping_sub(old.1)as i8),vehicle,|cell|{if !(0..16384).contains(&i32::from(cell)){return -1;}let n=cell as usize;let(x,y)=(n&127,n>>7);let code=if x<self.map.width&&y<self.map.height{self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(127)}else{self.map.native_terrain_plane.get(n).copied().unwrap_or(127)};super::original_collision::world_permission(code,mask,team,octant,self.map.world==1)});(hit,c)
        };
        if hit{segment.error=next.error;segment.correct_contact_position([contact.position.0,contact.position.1]);let u=&mut self.units[i];u.x=f64::from(contact.position.0)*WORLD_SCALE;u.y=f64::from(contact.position.1)*WORLD_SCALE;u.original_motion=Some(segment);self.native_attack_walk_marked=true;u.native_pose7e|=128;return Some(WalkStep{contact:Some(contact),..Default::default()});}
        let ended=next.remaining==0;let u=&mut self.units[i];u.native_counter66=0;if vehicle{u.native_counter67=0;}u.x=f64::from(attempted.0)*WORLD_SCALE;u.y=f64::from(attempted.1)*WORLD_SCALE;u.native_remaining48=next.remaining();u.original_motion=Some(next);u.moving=old!=attempted;if u.moving{u.native_idle=None;}
        Some(WalkStep{moved:old!=attempted,ended,contact:None})
    }
    pub(crate) fn prepare_native_walking_line(&mut self,i:usize){let Some(u)=self.units.get(i)else{return;};if u.original_motion.is_some(){return;}let Some(goal)=u.path.front().copied()else{return;};let segment=Segment::new(u.x,u.y,goal);if segment.remaining()!=0&&!self.prepare_native_line_occupancy(i,[coordinate(goal.0)as i16,coordinate(goal.1)as i16]){return;}let u=&self.units[i];let axis=segment.axis81();let heading=original_combat::facing(segment.heading());let flags=u.native_flags64;let slot=u.native_slot;if segment.remaining()!=0{self.units[i].native_directiona2=-1;self.units[i].native_directiona3=0;}self.units[i].native_axis81=axis;self.units[i].native_remaining48=segment.remaining();self.units[i].angle=segment.heading();self.units[i].original_motion=Some(segment);if let Some(slot)=slot{self.native_group_rosters.record_segment_direction(slot,heading,flags);}}
    ///The infantry branch of525a0. Call only for a nonzero accepted local
    ///line, before its caller optionally overrides phase2 with phase7.
    pub(crate) fn prepare_native_walk(&mut self,i:usize){
        let u=&self.units[i];if super::stats(&u.kind).cls!=0{return;}
        let control=u.original_combat.as_ref().map(|s|s.movement_control).unwrap_or_else(||original_combat::CombatState::new(&u.kind).movement_control);
        let code=self.native_terrain(coordinate(u.x)+8,coordinate(u.y)+8);
        let motion_index=(index(control,code)as i32+i32::from(u.motion_bonus)+i32::from(u.native_group_motion_bonus)).clamp(0,12)as usize;
        //525a0 reads the already-prepared8b byte. A caller's new8c bonus
        //does not select a different pattern until the next4f1e0 boundary.
        let motion_index=if u.native_slot.is_some(){u.native_speed8b.max(0).min(12)as usize}else{motion_index};
        let walk=if u.native_phase35==2{Walk{frame:u.native_frame32,timer:u.native_timer3c}}else{Walk::replacement(motion_index,u.native_frame32,u.native_timer3c)};
        let u=&mut self.units[i];u.native_frame32=walk.frame;u.native_timer3c=walk.timer;u.native_walk=Some(walk);u.native_phase35=2;
    }
    ///525a0 can replace the current line without touching either path list.
    ///462fd0 uses that operation for an incoming projectile. This callback
    ///deliberately neither consumes nor replaces the primary route's front.
    pub(crate) fn move_native_local(&mut self,i:usize,segment:&mut Segment,first_tick:u64,ticks:u64,aim_follows:bool)->LocalResult{
        self.units[i].native_remaining48=segment.remaining();
        for tick in first_tick..first_tick+ticks{
            let u=&self.units[i];
            let control=u.original_combat.as_ref().map(|s|s.movement_control).unwrap_or_else(||original_combat::CombatState::new(&u.kind).movement_control);
            let code=self.native_terrain(coordinate(u.x)+8,coordinate(u.y)+8);
            let motion_index=(index(control,code)as i32+i32::from(u.motion_bonus)+i32::from(u.native_group_motion_bonus)).clamp(0,12)as usize;
            let pattern=STEPS[motion_index];let step=pattern[tick as usize%pattern.len()];
            if super::stats(&u.kind).cls==0{let mut walk=u.native_walk.clone().unwrap_or(Walk{frame:u.native_frame32,timer:u.native_timer3c});walk.advance(motion_index);self.units[i].native_frame32=walk.frame;self.units[i].native_timer3c=walk.timer;self.units[i].native_walk=Some(walk);}
            if step==0{continue;}
            let old=segment.position;let mut next=segment.clone();next.advance(step);
            let point=(f64::from(next.position.0)*WORLD_SCALE,f64::from(next.position.1)*WORLD_SCALE);
            if !self.movement_segment_clear_unit(&self.units[i],point){
                self.native_attack_walk_marked=true; //518e0 physical-contact event
                //518e0 advances the robot counter before51170.52200 then
                //performs its own normal counter update in the same callback.
                if super::stats(&self.units[i].kind).cls==0{let fields=self.units[i].native_walk.as_mut().map(|walk|{walk.advance(motion_index);(walk.frame,walk.timer)});if let Some((frame,timer))=fields{self.units[i].native_frame32=frame;self.units[i].native_timer3c=timer;}}
                return LocalResult::Blocked;
            }
            *segment=next;let u=&mut self.units[i];u.native_remaining48=segment.remaining();u.x=point.0;u.y=point.1;
            if segment.position!=old{u.angle=segment.heading;u.native_idle=None;u.moving=true;if aim_follows{u.aim=u.angle;}}
            let slot=u.native_slot;let position=segment.position;
            if let Some(m)=slot.and_then(|s|self.native_group_rosters.members.get_mut(s)).and_then(Option::as_mut){m.flags69|=4;}
            if let Some(slot)=slot{let code=self.native_terrain(position.0+8,position.1+8);self.native_group_rosters.record_terrain_formation(slot,code);}
            self.native_grenade_contact(i);
            if super::stats(&self.units[i].kind).cls>0{self.advance_native_hardware_movement(i);}
            //51170 clears64:40 at the end of the line. The caller may load
            //the old route again, but never spends excess pixels this tick.
            if segment.remaining==0{self.native_attack_walk_marked=true;return LocalResult::Arrived;}
        }
        LocalResult::Moving
    }
    ///Managed hardware uses the same51da0..521a0 leaves and51170 link
    ///retirement. The full5c5c0 targeting/turret body remains a separate owner.
    fn move_native_owned_hardware(&mut self,i:usize){
        let Some(slot)=self.units[i].native_slot else{return;};if self.native_group_rosters.members[slot].as_ref().is_none_or(|m|m.flags31&64!=0||m.flags69&8!=0){return;}
        let w=self.units[i].native_hardware_slot;let record=w.map(|w|self.native_hardware_pool[w].order).unwrap_or(self.native_hardware_dummy);
        if record.field17&2!=0{if self.resume_native_reset_path(i){self.check_native_reset_path(i);}self.units[i].native_phase35=9;let mut record=record;record.field17=(record.field17&!2)|1;if let Some(w)=w{self.native_hardware_pool[w].set_order(record);}else{self.native_hardware_dummy=record;}return;}
        self.prepare_native_walking_line(i);self.native_path_contact.goal_check_changed=false;let Some(result)=self.advance_native_walk_position(i)else{return;};if let Some(contact)=result.contact{match self.native_route_contact(i,contact){Ok(_)=>{},Err(e)=>{self.native_route_boundary=Some(e);}}}if result.ended{self.finish_native_walk_line(i);}self.restore_native_contact_facing(i,self.native_path_contact.goal_check_changed);
        self.units[i].native_bucket38=(((coordinate(self.units[i].y)>>3)as u8)&240).wrapping_add((coordinate(self.units[i].x)>>7)as u8);if result.moved{self.advance_native_hardware_movement(i);}
    }
    pub(crate) fn move_original(&mut self,i:usize,first_tick:u64,ticks:u64,aim_follows:bool) {
        if self.owns_native_route(i)&&self.units[i].native_class7c==1{self.move_native_owned_hardware(i);return;}
        //43e190 leaves a local contour pending. Resume the original destination
        //on the next callback, after the last local waypoint has been consumed.
        if self.units[i].path.is_empty(){if let Some(goal)=self.units[i].native_route_pending.get(){
            let unit=self.units[i].clone();self.units[i].path=self.route_unit(&unit,goal.0,goal.1);
        }}
        self.units[i].moving=!self.units[i].path.is_empty();
        if self.units[i].moving&&super::stats(&self.units[i].kind).cls==0&&self.units[i].native_phase35!=7&&self.units[i].native_phase35!=2{self.prepare_native_walk(i);}
        if !self.units[i].path.is_empty(){self.native_move_order_started(i);}
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
            let motion_index=(index(control,code)as i32+i32::from(u.motion_bonus)+i32::from(u.native_group_motion_bonus)).clamp(0,12)as usize;
            let pattern=STEPS[motion_index];let step=pattern[tick as usize%pattern.len()];
            if super::stats(&u.kind).cls==0{
                let mut walk=u.native_walk.clone().unwrap_or(Walk{frame:u.native_frame32,timer:u.native_timer3c});walk.advance(motion_index);self.units[i].native_frame32=walk.frame;self.units[i].native_timer3c=walk.timer;self.units[i].native_walk=Some(walk);
            }
            let u=&self.units[i];
            if step==0{continue;}
            let existing=u.original_motion.clone().filter(|s|s.goal==(coordinate(goal.0),coordinate(goal.1))
                &&s.position==(coordinate(u.x),coordinate(u.y)));
            let new_segment=existing.is_none();let mut segment=existing.unwrap_or_else(||Segment::new(u.x,u.y,goal));
            if new_segment{if let Some(slot)=u.native_slot{self.native_group_rosters.record_segment_direction(slot,original_combat::facing(segment.heading),u.native_flags64);}}
            self.units[i].native_remaining48=segment.remaining();let u=&self.units[i];
            let old=segment.position;segment.advance(step);
            let (x,y)=(f64::from(segment.position.0)*WORLD_SCALE,f64::from(segment.position.1)*WORLD_SCALE);
            if !self.movement_segment_clear_unit(u,(x,y)){self.native_attack_walk_marked=true;self.units[i].original_motion=None;break;}
            let u=&mut self.units[i];u.x=x;u.y=y;u.native_remaining48=segment.remaining();
            if segment.position!=old {
                u.angle=segment.heading;
                u.native_idle=None;

                u.moving=true;if aim_follows{u.aim=u.angle;}
            }
            if segment.remaining==0 {
                self.native_attack_walk_marked=true; //51170, before replacing the path head
                // Native movement truncates this tick's step at the waypoint.
                // It starts the next segment without spending the excess.
                u.path.pop_front();u.original_motion=None;
                if u.path.is_empty()&&u.native_secondary_route.get(){u.native_secondary_route.set(false);if let Some(slot)=u.native_slot{self.native_group_rosters.record_secondary_path(slot,false);}}
                if super::stats(&u.kind).cls==0&&u.path.is_empty(){u.native_phase35=if u.native_route_pending.get().is_some(){2}else{0};}
            }else{u.original_motion=Some(segment);}
            let slot=u.native_slot;let position=(coordinate(u.x),coordinate(u.y));
            if let Some(m)=slot.and_then(|s|self.native_group_rosters.members.get_mut(s)).and_then(Option::as_mut){m.flags69|=4;}
            if let Some(slot)=slot{let code=self.native_terrain(position.0+8,position.1+8);self.native_group_rosters.record_terrain_formation(slot,code);}
            self.native_grenade_contact(i);
            if super::stats(&self.units[i].kind).cls>0{self.advance_native_hardware_movement(i);}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_624_walking_counter_steps_match_the_executed_native_handler(){
        let mut hash=2166136261u32;
        for speed in 0..13{let mut walk=Walk::new(speed);for tick in 1..=48{walk.advance(speed);for n in [speed as i32,tick,i32::from(walk.frame),i32::from(walk.timer)]{hash=(hash^n as u32).wrapping_mul(16777619);}}}
        assert_eq!(hash,4264211241,"ZNativeAnimationOracle");
    }
    #[test]
    fn all_832_replacement_frame_and_counter_seeds_match_native_525a0(){
        let mut hash=2166136261u32;let mut scenario=0;
        for index in 0..13{for frame in[0i16,1,2,3,4,255,256,-256]{for timer in[-128i8,-1,0,1,2,7,10,127]{let walk=Walk::replacement(index,frame,timer);for n in[scenario,i32::from(walk.frame),i32::from(walk.timer),2]{hash=(hash^n as u32).wrapping_mul(16777619);}scenario+=1;}}}
        assert_eq!(scenario,832);assert_eq!(hash,4122382637,"ZNativeWalkReplacementOracle");
    }
    #[test]
    fn replacement_lines_and_primary_path_return_match_1152_executed_native_records(){
        let mut hash=2166136261u32;let mut rows=0;let mut scenario=0;
        let mut record=|values:&[i32]|{for &n in values{hash=(hash^n as u32).wrapping_mul(16777619);}rows+=1;};
        for class in [0,1]{for path in 0..2{for blocked in 0..2{for speed in 1..=4{for delta in [(9,3),(3,9),(-9,3),(-3,9),(9,-3),(3,-9),(-9,-3),(-3,-9)]{
            let mut segment=Segment::new(1000.,1000.,(1000.+f64::from(delta.0)*2.,1000.+f64::from(delta.1)*2.));
            let mut state=if class==1{9}else{2};let mut flags=64;let mut hardware=class;
            record(&[scenario,state,flags,path,segment.remaining,i32::from(super::super::original_combat::facing(segment.heading)),hardware]);
            for tick in 1..=16{
                let collision=blocked!=0&&tick==2;
                if !collision{segment.advance(speed);}
                if collision||segment.remaining==0{
                    flags=0;
                    if class==0{state=if path==0{0}else{2};}
                    else if path==0{hardware=0;}
                    if path!=0{segment=Segment::new(f64::from(segment.position.0)*2.,f64::from(segment.position.1)*2.,(1504.,1288.));}
                }
                record(&[scenario,tick,segment.position.0,segment.position.1,state,flags,path,segment.remaining,hardware]);
                if flags==0{break;}
            }
            scenario+=1;
        }}}}}
        assert_eq!(rows,1152);assert_eq!(hash,2851233581,"ZNativeDodgeReturnOracle:256 end/block/route fixtures");
    }
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

///Known globalpattern step; the native pointerdomain is checked by itscaller.
pub(crate) fn known_pattern_step(index:usize,tick:u64)->Option<u8>{let p=STEPS.get(index)?;Some(p[tick as usize%p.len()])}

#[cfg(test)]mod pattern_pointer_tests{
 use super::*;
 #[test]fn source_constructor_speed_ff_initializes_real_pointer_and_unchanged_speed_retains_it(){let mut g=Game::new(0,1,1);g.map.original=true;g.units.clear();g.native_group_rosters.clear();g.spawn("grunt",0,640.,640.,1,None);assert!(g.units[0].native_pattern20.is_some());if let Some(c)=&mut g.units[0].original_combat{c.movement_control=0;}g.initialize_native_walk_fields(0);assert_eq!(g.units[0].native_speed8b,-1,"41b34d writesFF before first4f1e0");g.units[0].native_pattern20=None;g.refresh_native_pattern_step(0,0);assert_eq!(g.units[0].native_pattern20,Some(crate::original_retired_unit_preamble::PATTERNS[0]));assert_eq!(g.units[0].native_field62,1);g.units[0].native_pattern20=Some(crate::original_retired_unit_preamble::PATTERNS[1]);g.refresh_native_pattern_step(0,4);assert_eq!(g.units[0].native_pattern20,Some(crate::original_retired_unit_preamble::PATTERNS[1]),"451d40 does not rewrite the pointer when speed stays0");assert_eq!(g.units[0].native_field62,1,"the actual savedpattern1 cursor4%4 is used, not recomputedpattern0 cursor4%5");}
}
