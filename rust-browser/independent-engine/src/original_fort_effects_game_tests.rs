use super::*;
use super::super::{effects::{bridge, Particle}, State};
use crate::original_combat::OriginalRng;
use std::io::{BufRead, BufReader};

#[test]
#[ignore = "requires private original wreck contact trace"]
fn original_wreck_contacts_match_the_entire_game_effect_pool() {
    let path=std::env::var_os("NATIVE_FORT_WRECK_PARTICLE_TRACE").expect("private trace path");
    let rows=BufReader::new(std::fs::File::open(path).unwrap());
    let mut game=Game::new(0,1,0x420890);
    game.map.original=true;
    let mut totals=[0;4];
    for (id,line) in rows.lines().enumerate(){
        let v:serde_json::Value=serde_json::from_str(&line.unwrap()).unwrap();let n=|key:&str|v[key].as_u64().unwrap();
        let cap=n("capacity") as usize;let mode=n("mode") as usize;
        assert_eq!(n("id"),id as u64);totals[0]+=1;
        let warm=|slot:usize|{
            let mut raw=std::array::from_fn(|j|(id*17+slot*13+j*19) as u8);
            let marker=if slot==0&&cap>1{0i16}else{-1000i16};raw[..2].copy_from_slice(&marker.to_le_bytes());raw
        };
        let mut state=State::empty();state.effects=(0..cap).map(|i|Some(Particle::from_bytes(warm(i)))).collect();
        state.effect_origin=[13,57];state.effect_owner=61;state.effect_fade=41;state.effect_flags=31;state.effect_sprite=515;state.effect_maximum=19;
        state.effect_tint=31;state.effect_suppression=n("suppressed") as u8;
        state.bridge_fx=bridge::Raw{byte32:69,payload36:0x12345678,payload40:0xabcdef01,last_record:Some(0xdeadbeef),..Default::default()};
        game.native_world=Some(state);game.multiplayer=n("mp")!=0;
        game.original_rng=OriginalRng{words:std::array::from_fn(|i|v["rngBefore"][i].as_u64().unwrap() as u32),multiplayer:game.multiplayer};
        let point=std::array::from_fn(|i|v["point"][i].as_i64().unwrap() as i16);
        let made=if mode==1{0}else{game.spawn_native_fort_retirement_contact(point,n("unit64") as u8,mode==0).unwrap()};
        assert_eq!(made as u64*if mode==0{4}else{3},n("calls"),"calls {id}");
        totals[1+usize::from(mode!=0)]+=made;totals[3]+=n("calls") as usize;
        let state=game.native_world.as_ref().unwrap();
        let pool=v["pool"].as_str().unwrap();assert_eq!(pool.len(),44000);
        for slot in 0..500{
            let expected:[u8;44]=std::array::from_fn(|j|u8::from_str_radix(&pool[2*(slot*44+j)..2*(slot*44+j)+2],16).unwrap());
            let actual=if slot<cap{state.effects[slot].as_ref().unwrap().bytes()}else{warm(slot)};
            assert_eq!(actual,expected,"pool {id}/{slot}");
        }
        assert_eq!(game.original_rng.words,std::array::from_fn(|i|v["rngAfter"][i].as_u64().unwrap() as u32),"rng {id}");
        let projection=[i64::from(state.effect_origin[0]),i64::from(state.effect_origin[1]),i64::from(state.effect_owner),
            i64::from(state.effect_fade),i64::from(state.effect_flags),i64::from(state.effect_sprite),i64::from(state.effect_maximum),
            i64::from(state.bridge_fx.constructor.unwrap_or(0)),i64::from(state.bridge_fx.last_record.unwrap()),i64::from(state.effect_suppression)];
        assert_eq!(projection,std::array::from_fn(|i|v["globals"][i].as_i64().unwrap()),"globals {id}");
    }
    assert_eq!(totals,[128,12,1072,3264]);
}

#[test]
#[ignore = "requires private original 41ddb0 particle trace"]
fn original_fort_release_poof_matches_the_shared_game_pool_and_all_rng_words() {
    let path=std::env::var_os("NATIVE_FORT_RELEASE_POOF_TRACE").expect("private trace path");
    let rows=BufReader::new(std::fs::File::open(path).unwrap());
    let mut game=Game::new(0,1,0x41ddb0);
    game.map.original=true;
    let mut counts=[0;2];
    for (id,row) in rows.lines().enumerate() {
        let v:serde_json::Value=serde_json::from_str(&row.unwrap()).unwrap();
        let n=|key:&str|v[key].as_u64().unwrap();
        assert_eq!(n("id"),id as u64);
        counts[0]+=1;
        let cap=n("capacity") as usize;
        let mut original=[0;5];
        for (i,w) in original.iter_mut().enumerate(){*w=v["rngBefore"][i].as_u64().unwrap() as u32;}
        game.multiplayer=n("mp")!=0;
        game.original_rng=OriginalRng{words:original,multiplayer:game.multiplayer};
        let warm=|slot:usize| {
            let mut raw=std::array::from_fn(|j|(id*17+slot*13+j*19) as u8);
            let marker=if slot==0&&cap>1 {0i16}else{-1000i16};
            raw[..2].copy_from_slice(&marker.to_le_bytes());raw
        };
        let mut state=State::empty();
        state.effects=(0..cap).map(|i|Some(Particle::from_bytes(warm(i)))).collect();
        state.effect_tint=31;
        state.effect_suppression=n("suppressed") as u8;
        state.bridge_fx=bridge::Raw{byte32:69,payload36:0x12345678,payload40:0xabcdef01,last_record:Some(0xdeadbeef),..Default::default()};
        game.native_world=Some(state);
        let point=std::array::from_fn(|i|v["point"][i].as_i64().unwrap() as i16);
        let made=game.spawn_native_fort_release_poof(point,n("unit64") as u8).unwrap();
        assert_eq!(made as u64,n("calls")/6);
        counts[1]+=made;
        let state=game.native_world.as_ref().unwrap();
        let original_pool=v["pool"].as_str().unwrap();assert_eq!(original_pool.len(),44000);
        for slot in 0..500 {
            let expected: [u8;44]=std::array::from_fn(|j|u8::from_str_radix(&original_pool[2*(slot*44+j)..2*(slot*44+j)+2],16).unwrap());
            let actual=if slot<cap{state.effects[slot].as_ref().unwrap().bytes()}else{warm(slot)};
            assert_eq!(actual,expected,"pool {id}/{slot}");
        }
        assert_eq!(game.original_rng.words,std::array::from_fn(|i|v["rngAfter"][i].as_u64().unwrap() as u32));
        let projection=[i64::from(state.effect_origin[0]),i64::from(state.effect_origin[1]),i64::from(state.effect_owner),0,
            i64::from(state.effect_fade),i64::from(state.effect_flags),i64::from(state.effect_sprite),i64::from(state.effect_maximum),
            i64::from(state.bridge_fx.constructor.unwrap()),i64::from(state.bridge_fx.last_record.unwrap()),0x5e30c8+7*192];
        assert_eq!(projection,std::array::from_fn(|i|v["globals"][i].as_i64().unwrap()),"globals {id}");
    }
    assert_eq!(counts,[96,36]);
}

#[test]
#[ignore = "private executed GOG fort burst trace; supply NATIVE_FORT_PARTICLE_TRACE"]
fn original_fort_bursts_match_all_384_typed_pool_and_rng_states() {
    let path = std::env::var_os("NATIVE_FORT_PARTICLE_TRACE").expect("NATIVE_FORT_PARTICLE_TRACE");
    let rows = BufReader::new(std::fs::File::open(path).unwrap());
    let mut game = Game::new(0, 1, 0x422a60);
    game.map.original = true;
    game.native_world = Some(State::empty());
    let mut count = 0;
    let mut allocations = 0;
    for row in rows.lines() {
        let v: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        let n = |field: &str| v[field].as_u64().unwrap();
        assert_eq!(n("scenario"), count);
        let cap = n("cap") as usize;
        let burst = if n("kind") == 0 { Burst::First } else { Burst::Second };
        game.multiplayer = n("mp") != 0;
        game.events.clear();
        game.native_feedback.ui_clock = 500;
        game.native_feedback.sound_enabled = n("master") as u8;
        game.native_audio = Default::default();
        game.native_audio.effect_counter = 3;
        game.native_listener_audio.guest_audio = Default::default();
        game.native_listener_audio.guest_audio.effect_counter = 3;
        let camera = [n("far") as i16 * 2000, 0];
        let other = [2000 - camera[0], 0];
        game.native_view = Some((camera, [1024, 768]));
        if game.multiplayer {
            game.set_native_listener_settings(&crate::json!({
                "0":{"soundEnabled":n("master")!=0,"view":{"camera":camera,"size":[1024,768]},"hardwareContext":"1:0"},
                "1":{"soundEnabled":n("master")!=0,"view":{"camera":other,"size":[1024,768]},"hardwareContext":"1:0"}
            }));
        }
        game.original_rng = OriginalRng {
            words: std::array::from_fn(|i| v["rngBefore"][i].as_u64().unwrap() as u32),
            multiplayer: game.multiplayer,
        };
        let state = game.native_world.as_mut().unwrap();
        for (slot, p) in state.effects.iter_mut().enumerate() {
            let mut raw = std::array::from_fn(|j| (count as usize * 17 + slot * 13 + j * 19) as u8);
            raw[..2].copy_from_slice(&(-1000i16).to_le_bytes());
            *p = Some(Particle::from_bytes(raw));
        }
        state.effect_tint = 31;
        state.effect_suppression = n("suppressed") as u8;
        state.bridge_fx = bridge::Raw {
            byte32:69, payload36:0x12345678, payload40:0xabcdef01,
            last_record:Some(0xdeadbeef), ..Default::default()
        };
        let made = if cap == 500 {
            game.run_native_fort_burst_effects(burst, [640, 512], n("owner") as u8).unwrap()
        } else {
            game.emit_native_spatial_audio(crate::original_audio::spatial::Chance::FortBurst {point:[640,512]});
            let state = game.native_world.as_mut().unwrap();
            let mut g = globals(state);
            let made = fort::spawn(&mut World {state, rng: &mut game.original_rng, limit:cap}, &mut g, burst, [640,512], n("owner") as u8);
            commit(state, g);
            made
        };
        assert_eq!(made, n("rngEntries") as usize / 4);
        let state = game.native_world.as_ref().unwrap();
        let pool = v["pool"].as_str().unwrap();
        assert_eq!(pool.len(), 44000);
        for (slot, part) in state.effects.iter().enumerate() {
            let expected: [u8;44] = std::array::from_fn(|j| {
                let at = 2 * (slot * 44 + j);
                u8::from_str_radix(&pool[at..at+2], 16).unwrap()
            });
            assert_eq!(part.as_ref().unwrap().bytes(), expected, "scenario {count}, effect slot {slot}");
        }
        assert_eq!(game.original_rng.words, std::array::from_fn(|i| v["rng"][i].as_u64().unwrap() as u32));
        assert_eq!(state.effect_origin, std::array::from_fn(|i| v["origin"][i].as_i64().unwrap() as i16));
        assert_eq!(state.effect_sprite, n("sprite") as i16);
        assert_eq!(state.effect_maximum, n("maximum") as i8);
        assert_eq!(state.effect_flags, n("flags") as u8);
        assert_eq!(state.effect_owner, n("effectOwner") as u8);
        assert_eq!(state.effect_fade, n("fade") as u8);
        assert_eq!(state.bridge_fx.constructor, Some(n("ctor") as u32));
        assert_eq!(state.bridge_fx.last_record, Some(n("last") as u32));
        assert_eq!(state.bridge_fx.payload36, n("payload36") as u32);
        assert_eq!(state.bridge_fx.payload40, n("payload40") as u32);
        let samples = game.events.iter().filter(|e|e["kind"]=="nativeSample").collect::<Vec<_>>();
        assert!(samples.iter().all(|e|e["sampleId"]==1));
        assert_eq!(samples.iter().filter(|e|e["recipientTeam"].as_u64().unwrap_or(0)==0).count() as u64, n("mixes"));
        assert_eq!(game.native_audio.opportunity_gate, n("audioGate") as u8);
        assert_eq!(game.native_audio.effect_counter, n("counter") as u16 as i16);
        if game.multiplayer {
            assert_eq!(samples.iter().filter(|e|e["recipientTeam"]==1).count() as u64, n("far")*n("master"));
            assert_eq!(game.native_listener_audio.guest_audio.opportunity_gate, n("far") as u8);
        }
        count += 1;
        allocations += made;
    }
    assert_eq!((count, allocations), (384, 1120));
}
