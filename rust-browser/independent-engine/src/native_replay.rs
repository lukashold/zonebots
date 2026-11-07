//! Read-only observations for tick-by-tick replay diagnostics.
//! This is an explicit subset of native state, not a savegame or a full
//! original-process image. Matching checkpoints alone cannot prove 1:1.
use crate::{Game, Value, json};

fn hash(bytes: impl IntoIterator<Item=u8>) -> u32 {
    bytes.into_iter().fold(2166136261, |h, b| (h ^ u32::from(b)).wrapping_mul(16777619))
}

pub(crate) fn checkpoint(game: &Game) -> Value {
    if !game.map.original { return json!({"error":"Replay checkpoints require an original mission"}); }
    let paths=game.native_routes.borrow();
    let slots=(0..240).map(|slot| {
        let member=game.native_group_rosters.members[slot].as_ref();
        let unit=game.units.iter().rfind(|u|u.native_slot==Some(slot));
        let active=member.is_some_and(|m|m.flags31&4!=0);
        // Inactive residual UNIT bytes are not all modeled. Do not invent
        // a zero-filled native memory record for a retired slot.
        if !active{return Value::Null;}
        let Some(u)=unit else{return json!({"missingActiveSlot":slot});};
        let links=paths.pool.links[slot];
        json!({
            "position2c":[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16],
            "flags31":member.map(|m|m.flags31),"group":member.map(|m|m.group),
            "role":member.map(|m|m.role),"phase35":u.native_phase35,
            "frame32":u.native_frame32,"timer3c":u.native_timer3c,
            "order5c":u.native_order5c,"remaining48":u.native_remaining48,
            "line48":u.original_motion.as_ref().map(|m|m.source_line_fields()),
            "hardware17":u.native_hardware_slot.and_then(|n|game.native_hardware_pool.get(n)).map(|h|h.order.field17),
            "flags64":u.native_flags64,"flags9c":u.native_flags9c,
            "control06":u.native_control06,"class7c":u.native_class7c,
            "primary":links.primary,"secondary":links.secondary
        })
    }).collect::<Vec<_>>();
    let pool_hash=hash(paths.pool.nodes.iter().flat_map(|n|n.point[0].to_le_bytes()
        .into_iter().chain(n.point[1].to_le_bytes()).chain(n.next.to_le_bytes())));
    let scratch_hash=hash(paths.road_scratch.frame.iter().chain(paths.road_scratch.candidate_tail.iter()).copied()
        .chain(paths.road_scratch.pairs.iter().flatten().flat_map(|n|n.to_le_bytes())));
    json!({
        "schema":"z-native-checkpoint-v1",
        "tick":game.motion_tick,"rng":game.original_rng.words,
        "cosmeticRng":game.native_cosmetic_rng,
        "clocks":{"ui":game.native_feedback.ui_clock,"frame16":game.native_frame16,
            "object":game.native_object_frame,"flag":game.native_flag_frame,
            "wait":game.native_wait_clock,"callback":game.native_callback_mode},
        "units":slots,"routePoolHash":pool_hash,"routeScratchHash":scratch_hash,
        "routeMaskHash":hash(paths.mask.iter().copied()),
        "boundary":game.native_route_boundary.map(|b|format!("{b:?}")),
        "winner":game.winner
    })
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]
    #[ignore="requires the private original campaign"]
    fn mission_one_recorded_march_keeps_complete_native_callbacks() {
        let path=std::env::var_os("Z_PRIVATE_CAMPAIGN").expect("private campaign path");
        let campaign:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let script:Value=serde_json::from_str(include_str!("../../tools/replays/mission-01-march.json")).unwrap();
        let initial=crate::request(json!({"op":"init","mission":0,"difficulty":1,"seed":7163,
            "aiTeams":[1],"originalMission":campaign["missions"][0],"nativeCalendar":{"month":1,"day":1}}));
        assert!(initial["error"].is_null(),"{initial}");
        let actions=script["actions"].as_array().unwrap();let mut cursor=0;let mut last_tick=0;
        for tick in 1..=script["ticks"].as_u64().unwrap(){
            while cursor<actions.len()&&actions[cursor]["tick"].as_u64()==Some(tick){
                crate::request(actions[cursor]["request"].clone());cursor+=1;
            }
            crate::GAME.with(|cell|{let mut state=cell.borrow_mut();let g=state.as_mut().unwrap();g.events.clear();g.step(0.059);});
            let done=crate::GAME.with(|cell|{
                let state=cell.borrow();let g=state.as_ref().unwrap();
                assert_eq!(g.native_route_boundary,None,"first failure at callback {tick}");
                assert_eq!(g.motion_tick,tick);
                g.winner.is_some()
            });
            last_tick=tick;if done{break;}
        }
        eprintln!("Recorded mission 1 completed {last_tick} callbacks with {cursor} input actions");
    }
    #[test] fn observation_is_read_only_even_with_mutating_request_metadata() {
        let mut game=Game::new(0,1,51);
        game.map.original=true;
        game.events.push(json!({"kind":"keep-this-event"}));
        crate::GAME.with(|s|*s.borrow_mut()=Some(game));
        let before=crate::request(json!({"op":"snapshot"}));
        let first=crate::request(json!({"op":"replayCheckpoint"}));
        let second=crate::request(json!({"op":"replayCheckpoint","nativeSelected":123,
            "nativeAudio":{"soundEnabled":false},"nativeView":{"camera":[500,500],"size":[1,1]}}));
        assert_eq!(first,second);
        assert_eq!(before,crate::request(json!({"op":"snapshot"})));
        assert_eq!(first["units"].as_array().unwrap().len(),240);
        assert_eq!(first["rng"].as_array().unwrap().len(),5);
    }
    #[test] fn hidden_rng_and_candidate_tail_changes_are_observable() {
        let mut game=Game::new(0,1,51);game.map.original=true;
        let before=checkpoint(&game);
        game.original_rng.words[4]^=1;
        game.native_routes.get_mut().road_scratch.candidate_tail[0]=19;
        let after=checkpoint(&game);
        assert_ne!(before["rng"],after["rng"]);
        assert_ne!(before["routeScratchHash"],after["routeScratchHash"]);
        assert_eq!(before["units"],after["units"]);
    }
}
