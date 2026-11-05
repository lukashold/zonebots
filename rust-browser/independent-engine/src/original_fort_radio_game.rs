use super::{Notice,Speaker,World,message};
use crate::Game;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Boundary { RawUnit(usize), MissingSpeaker(usize) }

struct Listener<'a>(&'a mut Game);

impl World for Listener<'_> {
    type Boundary=Boundary;
    fn scene_gate(&self)->i32 {self.0.native_feedback.scene_gate}
    fn voice_hold(&self)->i32 {self.0.native_feedback.voice_hold}
    fn set_voice_hold(&mut self,value:i32) {self.0.native_feedback.voice_hold=value;}
    fn physical_speaker(&mut self,slot:usize)->Result<Option<Speaker>,Boundary> {
        let member=self.0.native_group_rosters.members.get(slot).ok_or(Boundary::RawUnit(slot))?;
        Ok(member.as_ref().map(|m|Speaker{flags31:m.flags31,role:m.role}))
    }
    fn announce(&mut self,kind:Notice,slot:usize)->Result<(),Boundary> {
        let game=&mut self.0;
        if !game.units.iter().any(|u|u.native_slot==Some(slot)) {return Err(Boundary::MissingSpeaker(slot));}
        let scene=game.native_feedback.scene_gate as i16;
        game.native_feedback.scene_gate=0;
        game.native_feedback.speaker=Some(slot);
        game.native_feedback.portrait_mode=2;
        let random=if game.multiplayer {0}else{game.native_cosmetic_random()};
        game.admit_native_feedback(4,message(kind,random));
        game.native_feedback.scene_gate=i32::from(scene);
        Ok(())
    }
}

impl Game {
    /// The scene sends each participating player the result for their team.
    /// The raw flags in the listener decide whether an earlier cue is pending.
    pub(crate) fn arm_native_fort_result_radio(&mut self, defeated_owner:u8) {
        self.for_native_listeners(|game| {
            let feedback=&mut game.native_feedback;
            let owner=feedback.local_team;
            let notice=if i32::from(owner as i8)+1==i32::from(defeated_owner as i8){Notice::Loss}else{Notice::Win};
            feedback.fort_radio.arm(notice,owner);
        });
    }

    pub(crate) fn advance_native_fort_result_radio(&mut self)->Option<Boundary> {
        let mut failure=None;
        self.for_native_listeners(|game| {
            let mut radio=std::mem::take(&mut game.native_feedback.fort_radio);
            let result=radio.advance(&mut Listener(game));
            game.native_feedback.fort_radio=radio;
            if let Err(boundary)=result { failure.get_or_insert(boundary); }
        });
        failure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fort_radio_sends_separate_win_and_loss_cues_without_changing_the_scene_clock() {
        let mut game=Game::new(0,1,78);
        game.map.original=true;game.multiplayer=true;game.original_rng.multiplayer=true;
        game.units.clear();game.native_group_rosters.clear();game.ai_teams.clear();
        game.spawn("grunt",0,320.,320.,1,None);
        game.spawn("grunt",1,480.,480.,2,None);
        game.maintain_native_group_rosters();
        game.native_feedback.scene_gate=2;
        game.native_feedback.local_team=0;game.native_feedback.ui_clock=30;game.native_feedback.sound_enabled=1;
        let guest=&mut game.native_listener_audio.guest_feedback;
        guest.local_team=1;guest.ui_clock=30;guest.sound_enabled=1;
        let random=game.native_cosmetic_rng;let battle_rng=game.original_rng.words;
        game.events.clear();
        game.arm_native_fort_result_radio(2);
        assert_eq!(game.advance_native_fort_result_radio(),None);
        assert_eq!(game.events.len(),1);
        assert_eq!(game.events[0]["clip"],"ROB61");
        assert_eq!(game.events[0]["recipientTeam"],0);
        assert_eq!(game.events[0]["unitId"],game.units[0].id);
        assert_eq!(game.events[0]["playAudio"],true);
        assert_eq!(game.native_feedback.voice_hold,88);
        assert_eq!(game.advance_native_fort_result_radio(),None);
        assert_eq!(game.events.len(),2);
        assert_eq!(game.events[1]["clip"],"ROB67");
        assert_eq!(game.events[1]["recipientTeam"],1);
        assert_eq!(game.events[1]["unitId"],game.units[1].id);
        assert_eq!(game.events[1]["playAudio"],true);
        assert_eq!(game.native_listener_audio.guest_feedback.voice_hold,54);
        assert_eq!(game.native_feedback.voice_hold,84);
        assert_eq!((game.native_feedback.scene_gate,game.native_cosmetic_rng,game.original_rng.words),(2,random,battle_rng));
    }
}
