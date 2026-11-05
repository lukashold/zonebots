//! Game bindings for the fort queue. There is one scene counter, shared
//! with unit feedback and rendering, and one team marker shared with cleanup.
use super::{Game,fort_admission,fort_cleanup_game,effects::fort::Burst,json};
use crate::original_fort_scene::{Action,Admission,AdmissionAction,AdmissionWorld,Context,DefeatedFort,Notice,Return,World,camera::Camera,orders};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum Boundary {
    Fort(u8),
    Admission(fort_admission::Boundary),
    Cleanup(fort_cleanup_game::Boundary),
    Sweep(orders::game::Boundary),
    Pan(crate::original_fort_scene::game::PanBoundary),
    Camera(i16),
}

#[derive(Default)]
pub(super) struct Runtime {
    pub(super) enabled:bool,
    owner:u8,
    payload:u32,
    selected:u16,
    local_menu_flag:u8,
    local_defeat:u8,
    redraw:u8,
    score_time:Option<f64>,
    camera:Option<Camera>,
    last_camera:Option<[i16;2]>,
    pub(super) boundary:Option<Boundary>,
    door_boundary:Option<orders::game::Boundary>,
}

struct Scene<'a>(&'a mut Game);

impl Scene<'_> {
    fn registers(&self)->&Runtime{&self.0.native_world.as_ref().unwrap().fort_runtime}
    fn registers_mut(&mut self)->&mut Runtime{&mut self.0.native_world.as_mut().unwrap().fort_runtime}
}

impl World for Scene<'_> {
    type Error=Boundary;

    fn context(&self)->Context{Context{counter:self.0.native_feedback.scene_gate,payload:self.registers().payload,owner:self.registers().owner}}
    fn set_counter(&mut self,counter:i32){self.0.native_feedback.scene_gate=counter;}
    fn set_owner(&mut self,owner:u8){self.registers_mut().owner=owner;}
    fn set_payload(&mut self,payload:u32){self.registers_mut().payload=payload;}
    fn mixer_loaded(&self)->bool{true}
    fn local_team(&self)->u8{self.0.native_feedback.local_team}
    fn notice_pending(&self,kind:Notice)->bool{
        let radio=&self.0.native_feedback.fort_radio;
        match kind{Notice::Win=>radio.win.flag!=0,Notice::Loss=>radio.loss.flag!=0}
    }
    fn fort_base(&mut self,owner:u8)->Result<[i16;2],Boundary>{
        let bases=self.0.native_fort_bases().ok_or(Boundary::Fort(owner))?;
        match owner{1=>Ok(bases[1]),2=>Ok(bases[0]),_=>Err(Boundary::Fort(owner))}
    }
    fn execute(&mut self,action:Action)->Result<bool,Boundary>{
        match action {
            Action::HaltMixer=>self.0.events.push(json!({"kind":"nativeFortHaltMixer"})),
            Action::Notice{kind,local_team}=>self.0.native_feedback.fort_radio.arm(kind,local_team),
            Action::SelectFort(index)=>self.registers_mut().selected=index,
            Action::RemoveTeam(team)=>self.0.remove_native_fort_team_actors(team).map_err(Boundary::Cleanup)?,
            Action::PanToFort{base}=>{
                let request=self.0.prepare_native_fort_pan(base,0).map_err(Boundary::Pan)?;
                let current=self.0.native_view.map_or([0,0],|(p,_)|p);
                self.registers_mut().camera=Some(Camera{current,request,delta_eighth:[0,0],remaining:0,mode:0,target:current});
                self.0.events.push(json!({"kind":"nativeFortDeselect"}));
            },
            Action::Disperse{base,reverse}=>{
                let selected=self.registers().selected;
                self.0.issue_native_fort_local_sweep(base,selected,reverse).map_err(Boundary::Sweep)?;
            },
            Action::FirstBurst{base}=>{
                let owner=self.registers().owner;
                self.registers_mut().redraw=1;
                self.0.run_native_fort_burst_effects(Burst::First,base,owner);
            },
            Action::SecondBurst{base}=>{
                let owner=self.registers().owner;
                self.0.run_native_fort_burst_effects(Burst::Second,base,owner);
            },
            Action::RetireFortTeam=>{
                let owner=self.registers().owner;
                let time=self.0.time;
                self.registers_mut().score_time.get_or_insert(time);
                return self.0.retire_native_fort_team(owner).ok_or(Boundary::Fort(owner))?.map_err(|_|Boundary::Fort(owner));
            },
            Action::MarkSceneDirty=>self.0.native_world.as_mut().unwrap().fort_end.scene_dirty=1,
        }
        Ok(false)
    }
}

impl AdmissionWorld for Scene<'_> {
    fn team_activity(&self,team:u8)->u8{self.0.native_world.as_ref().unwrap().fort_team_activity[usize::from(team)]}

    fn apply_admission(&mut self,action:AdmissionAction)->Result<(),Boundary>{
        match action {
            // The browser has already sent a clicked product to the engine;
            // its open panel has no deferred original-style hover commit.
            AdmissionAction::CancelProductionSelection=>{
                self.0.events.push(json!({"kind":"nativeFortCloseProduction"}));
            },
            AdmissionAction::MarkTeamActivity{team,value}=>self.0.native_world.as_mut().unwrap().fort_team_activity[usize::from(team)]=value,
            AdmissionAction::WriteTeamMarker{team,value}=>{
                let marker=&mut self.0.native_world.as_mut().unwrap().fort_cleanup.marker;
                let mut bytes=marker.to_le_bytes();bytes[usize::from(team)]=value;*marker=u32::from_le_bytes(bytes);
            },
            AdmissionAction::ClearLocalDefeatFlag=>self.registers_mut().local_menu_flag=0,
            AdmissionAction::ArmLocalDefeat=>self.registers_mut().local_defeat=1,
            AdmissionAction::ApplyWorldWrites(fort)=>{
                let base=self.fort_base(fort.team()+1)?;
                self.0.apply_native_fort_admission_world(fort.team(),base).map_err(Boundary::Admission)?;
            },
        }
        Ok(())
    }
}

impl Game {
    pub(crate) fn native_fort_sequence_enabled(&self)->bool{
        !self.multiplayer&&self.native_world.as_ref().is_some_and(|w|w.fort_runtime.enabled)
    }

    pub(crate) fn admit_native_fort_defeat(&mut self,side:DefeatedFort,cause:i8){
        if !self.native_fort_sequence_enabled()||self.native_feedback.scene_gate!=0{return;}
        let mut queue=std::mem::take(&mut self.native_world.as_mut().unwrap().fort_scene_queue);
        let admitted=queue.admit_defeat(&mut Scene(self),side,cause);
        let world=self.native_world.as_mut().unwrap();
        world.fort_scene_queue=queue;
        if let Err(e)=admitted{world.fort_runtime.boundary=Some(e);}
        if matches!(admitted,Ok(Admission::Queued(_))){self.map.revision=self.map.revision.wrapping_add(1);}
    }

    pub(crate) fn native_fort_door_contact(&mut self,index:usize,cell:i16){
        use crate::original_fort_scene::door::{self,Door};
        if !self.native_fort_sequence_enabled(){return;}
        let Some(door)=Door::from_terrain(self.world_code(cell))else{return;};
        let unit=&self.units[index];let Some(slot)=unit.native_slot else{return;};
        let Some(member)=self.native_group_rosters.members[slot].as_mut().filter(|u|u.flags31&4!=0)else{return;};
        let flags=member.flags31;member.flags69&=!4;
        let point=[(unit.x/2.).floor() as i16,(unit.y/2.).floor() as i16];
        let activity=self.native_fort_team_activity().unwrap();
        let action=door::contact(door,flags,unit.native_class7c as i8,point,activity,false);
        self.units[index].native_word90=0;
        match action{
            door::Action::None=>{},
            door::Action::Defeat{fort,cause}=>self.admit_native_fort_defeat(fort,cause),
            door::Action::ForcedMove{destination}=>{
                if let Err(e)=self.issue_native_fort_door_local_move(orders::Move{slot:slot as u8,target:destination}){
                    self.native_world.as_mut().unwrap().fort_runtime.door_boundary=Some(e);
                }
            },
        }
    }

    pub(super) fn advance_native_fort_scene_queue(&mut self)->Option<u8>{
        let world=self.native_world.as_mut()?;
        if world.fort_runtime.boundary.is_some(){return None;}
        let mut queue=std::mem::take(&mut world.fort_scene_queue);
        let result=queue.advance(&mut Scene(self));
        let world=self.native_world.as_mut().unwrap();
        world.fort_scene_queue=queue;
        match result {
            Ok(Return::Continue)=>Some(0),
            Ok(Return::Terminal)=>{
                let owner=world.fort_runtime.owner;
                if matches!(owner,1|2){self.winner=Some(i32::from(2-owner));}
                Some(1)
            },
            Err(e)=>{world.fort_runtime.boundary=Some(e);None},
        }
    }

    pub(crate) fn advance_native_fort_camera(&mut self){
        let Some(world)=self.native_world.as_mut()else{return;};
        world.fort_runtime.last_camera=None;
        let Some(camera)=world.fort_runtime.camera.as_mut()else{return;};
        if let Err(e)=camera.tick(){world.fort_runtime.boundary=Some(Boundary::Camera(e.0));return;}
        world.fort_runtime.last_camera=Some(camera.current);
        let point=camera.current;
        if camera.remaining==0{world.fort_runtime.camera=None;}
        if let Some((_,size))=self.native_view {
            self.native_view=Some((point,size));
            self.set_native_world_effect_view(point,[0,0,size[0],size[1]]);
        }
    }

    pub(crate) fn native_fort_scene_snapshot(&self)->Option<serde_json::Value>{
        let world=self.native_world.as_ref()?;
        self.native_fort_sequence_enabled().then(||json!({
            "active":self.native_feedback.scene_gate!=0,
            "frame":(0..4).map(|i|world.fort_scene_queue.counter(i)).find(|&n|n>0),
            "camera":world.fort_runtime.last_camera,
            "panning":world.fort_runtime.camera.is_some(),
            "owner":world.fort_runtime.owner,
            "dirty":world.fort_end.scene_dirty!=0,
            "scoreTime":world.fort_runtime.score_time,
            "boundary":world.fort_runtime.boundary.map(|e|format!("{e:?}")),
        }))
    }
}

#[cfg(test)]
#[path="original_fort_runtime_game_tests.rs"]
mod tests;
