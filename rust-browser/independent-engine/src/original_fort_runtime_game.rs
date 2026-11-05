//! Game bindings for the fort queue. There is one scene counter, shared
//! with unit feedback and rendering, and one team marker shared with cleanup.
use super::{Game,fort_admission,fort_cleanup_game,effects::fort::Burst,json};
use crate::original_fort_scene::{Action,Admission,AdmissionAction,AdmissionWorld,Context,DefeatedFort,Notice,Return,World,camera::Camera,end::ScoreClock,orders};
use orders::queue::{self,Dispatch,DrainWorld,PeerOrders};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum Boundary {
    Fort(u8),
    Admission(fort_admission::Boundary),
    Cleanup(fort_cleanup_game::Boundary),
    Sweep(orders::game::Boundary),
    Pan(crate::original_fort_scene::game::PanBoundary),
    Camera(i16),
    PeerCursor(queue::CursorBoundary),
    PeerOrder(i16),
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
    score_clock:ScoreClock,
    tally:crate::original_fort_score::Tally,
    applied_palette:Option<u8>,
    drawn_palette:Option<u8>,
    camera:Option<Camera>,
    last_camera:Option<[i16;2]>,
    pub(super) boundary:Option<Boundary>,
    door_boundary:Option<orders::game::Boundary>,
    pub(super) peer_orders:Option<PeerOrders>,
    peer_clock:i32,
    peer_pan:Option<[i16;2]>,
}

struct Scene<'a>(&'a mut Game);

struct PeerDispatch<'a>{game:&'a mut Game,clock:i32,bypass:u8}

impl DrainWorld for PeerDispatch<'_>{
    type Boundary=Boundary;
    fn clock(&self)->i32{self.clock}
    fn set_local_router(&mut self,value:u8){self.bypass=value;}
    fn local_order(&mut self,order:Dispatch)->Result<(),Boundary>{
        let Ok(slot)=u8::try_from(order.slot)else{return Err(Boundary::PeerOrder(order.slot));};
        if slot>=240||order.forced!=1||order.opcode!=1||order.argument!=3||self.bypass!=1{return Err(Boundary::PeerOrder(order.slot));}
        // Address the current physical slot, even if its original actor was
        // replaced during the delay. An empty slot has no live Web recipient.
        if !self.game.units.iter().any(|u|u.native_slot==Some(usize::from(slot))){return Ok(());}
        self.game.accept_native_fort_forced_move(orders::Move{slot,target:order.target}).map_err(Boundary::Sweep)
    }
    fn release_dispatch_context(&mut self){}
}

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
        if self.0.multiplayer{return false;}
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
            Action::Notice{kind,local_team}=>{
                if self.0.multiplayer{let owner=self.registers().owner;self.0.arm_native_fort_result_radio(owner);}
                else{self.0.native_feedback.fort_radio.arm(kind,local_team);}
            },
            Action::SelectFort(index)=>self.registers_mut().selected=index,
            Action::RemoveTeam(team)=>self.0.remove_native_fort_team_actors(team).map_err(Boundary::Cleanup)?,
            Action::PanToFort{base}=>{
                let request=self.0.prepare_native_fort_pan(base,0).map_err(Boundary::Pan)?;
                if self.0.multiplayer{self.registers_mut().peer_pan=Some(request);}
                else{
                    let current=self.0.native_view.map_or([0,0],|(p,_)|p);
                    self.registers_mut().camera=Some(Camera{current,request,delta_eighth:[0,0],remaining:0,mode:0,target:current});
                }
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
        self.native_world.as_ref().is_some_and(|w|w.fort_runtime.enabled&&(!self.multiplayer||w.fort_runtime.peer_orders.is_some()))
    }

    pub(crate) fn record_native_fort_group_loss(&mut self,team:u8,pointer:u32){
        if !self.native_fort_sequence_enabled(){return;}
        self.native_world.as_mut().unwrap().fort_runtime.tally.retire(team,pointer);
    }

    pub(crate) fn native_fort_result_snapshot(&self)->Option<serde_json::Value>{
        let winner=self.winner?;
        if !self.native_fort_sequence_enabled(){return None;}
        let runtime=&self.native_world.as_ref()?.fort_runtime;
        let tally=&runtime.tally;
        let points=(!self.multiplayer&&winner==0&&!tally.unknown_value).then(||{
            crate::original_fort_score::score(self.mission+1,runtime.score_clock.seconds,
                tally.opponent_losses(0),tally.by_team[0],0).map(|result|result.points)
        }).flatten();
        Some(json!({"losses":tally.by_team.map(|loss|loss.count),"points":points}))
    }

    pub(crate) fn begin_native_fort_main(&mut self){
        if !self.native_fort_sequence_enabled(){return;}
        let world=self.native_world.as_mut().unwrap();
        world.fort_runtime.score_clock.ui_callback(world.fort_end.scene_dirty);
        if self.multiplayer {
            world.fort_runtime.peer_clock=world.fort_runtime.peer_clock.wrapping_add(1);
            let clock=world.fort_runtime.peer_clock;
            let mut queues=world.fort_runtime.peer_orders.take().unwrap();
            let result=queues.host.drain_due(&mut PeerDispatch{game:self,clock,bypass:0},true);
            let runtime=&mut self.native_world.as_mut().unwrap().fort_runtime;
            runtime.peer_orders=Some(queues);
            if let Err(error)=result{runtime.boundary.get_or_insert(error);}
        }
        let world=self.native_world.as_mut().unwrap();
        world.fort_end.begin_main();
        world.fort_runtime.drawn_palette=world.fort_runtime.applied_palette;
    }

    pub(crate) fn stage_native_fort_peer_move(&mut self,command:orders::Move)->bool{
        if !self.multiplayer{return false;}
        let Some(world)=self.native_world.as_mut()else{return false;};
        let runtime=&mut world.fort_runtime;
        let Some(orders)=runtime.peer_orders.as_mut()else{return false;};
        let input=|ring_selector|queue::Input{clock:runtime.peer_clock,selection_flag:0,auxiliary:0,selection_context:[0;12],ring_selector};
        if let Err(e)=orders.stage(command,input(0),input(1)){runtime.boundary.get_or_insert(Boundary::PeerCursor(e));}
        true
    }

    pub(crate) fn finish_native_fort_peer_frame(&mut self){
        if !self.multiplayer{return;}
        let Some(world)=self.native_world.as_mut()else{return;};
        let runtime=&mut world.fort_runtime;
        let Some(orders)=runtime.peer_orders.as_mut()else{return;};
        if let Err(e)=orders.finish_callback(0){runtime.boundary.get_or_insert(Boundary::PeerCursor(e));}
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
        world.fort_end.advance_palette();
        let mut queue=std::mem::take(&mut world.fort_scene_queue);
        let result=queue.advance(&mut Scene(self));
        let world=self.native_world.as_mut().unwrap();
        world.fort_scene_queue=queue;
        match result {
            Ok(Return::Continue)=>{
                if let Some(row)=world.fort_end.palette_copy_row(){
                    world.fort_runtime.applied_palette=Some(row);
                }
                Some(0)
            },
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
        let frame=(0..4).map(|i|world.fort_scene_queue.counter(i)).find(|&n|n>0);
        self.native_fort_sequence_enabled().then(||json!({
            "active":self.native_feedback.scene_gate!=0,
            "frame":frame,
            "camera":world.fort_runtime.last_camera,
            "panRequest":world.fort_runtime.peer_pan,
            "panning":world.fort_runtime.camera.is_some()||world.fort_runtime.peer_pan.is_some()&&frame.is_some_and(|n|(2..18).contains(&n)),
            "owner":world.fort_runtime.owner,
            "dirty":world.fort_end.scene_dirty!=0,
            "scoreTime":world.fort_runtime.score_clock.seconds,
            "paletteRow":world.fort_runtime.drawn_palette,
            "boundary":world.fort_runtime.boundary.map(|e|format!("{e:?}")),
        }))
    }
}

#[cfg(test)]
#[path="original_fort_runtime_game_tests.rs"]
mod tests;
