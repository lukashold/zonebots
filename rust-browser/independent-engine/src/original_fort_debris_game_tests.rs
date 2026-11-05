use super::*;
use crate::original_audio::spatial::Chance;

fn bytes(s:&str)->Vec<u8>{
    s.as_bytes().chunks_exact(2).map(|n|u8::from_str_radix(std::str::from_utf8(n).unwrap(),16).unwrap()).collect()
}

#[test]
#[ignore="requires the original debris-expiry trace"]
fn original_debris_expiry_matches_warm_animation_and_particle_records(){
    let path=std::env::var_os("NATIVE_FORT_DEBRIS_TRACE").unwrap();
    let trace=std::fs::read_to_string(path).unwrap();
    assert_eq!(trace.lines().count(),144);
    for line in trace.lines(){
        let row:Value=serde_json::from_str(line).unwrap();
        let n=|key:&str|row[key].as_i64().unwrap();
        let b=|key:&str|bytes(row[key].as_str().unwrap());
        let case=n("case");let seed=(case%48) as u32;
        let mode=n("mode");let point=[n("x") as i16,n("y") as i16];
        let mut g=Game::new(0,1,123);
        g.map.original=true;g.map.width=64;g.map.height=64;
        g.map.world=[0,2,4,1,3][n("world") as usize];
        g.map.wall_tile_lookup=json!({"nativeWorld":n("world")});
        g.map.native_terrain_plane=vec![n("code") as u8;16384];g.map.terrain_codes=vec![n("code") as u8;4096];
        g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.sectors.clear();
        g.import_native_world(&Value::Null);
        let view=([n("camera") as i16,0],[640,600]);g.native_view=Some(view);
        g.native_feedback.sound_enabled=n("soundOn") as u8;g.native_feedback.ui_clock=500;
        g.native_audio.effect_counter=3;
        g.multiplayer=mode!=4&&n("mp")!=0;
        g.original_rng=crate::original_combat::OriginalRng{
            words:std::array::from_fn(|i|0x173510a7u32.wrapping_add(seed*7919).wrapping_add((i as u32).wrapping_mul(0x9e3779b9))),
            multiplayer:g.multiplayer
        };
        let world=g.native_world.as_mut().unwrap();
        world.effects=b("lateBefore").chunks_exact(44).map(|r|Some(effects::Particle::from_bytes(r.try_into().unwrap()))).collect();
        world.effect_suppression=n("disabled") as u8;world.effect_tint=69;world.last_projection=[141,99];
        assert_eq!(world.effects[1].as_ref().unwrap().bytes().as_slice(),b("before"),"case {case}");
        for (i,r) in g.native_ambient_animations.records.iter_mut().enumerate(){r.set_word(0,100+i as i16);}
        let slot=usize::try_from(n("free")).ok();
        if let Some(slot)=slot{g.native_ambient_animations.records[slot].bytes=b("ambientBefore").try_into().unwrap();}

        if mode==4{
            g.finish_native_fort_debris_expiry(1,point);
            assert_eq!(g.blasts.len(),n("blast") as usize,"ownerless blast case {case}");
            if n("blast")!=0{
                assert_eq!((g.blasts[0].team,g.blasts[0].strength),(255,0),"case {case}");
                assert_eq!(row["retiredAtBlast"],true);assert_eq!(row["ownerlessAtBlast"],true);
                assert!(!g.native_blast_quiet());
                assert_ne!(g.native_world.as_ref().unwrap().effects[1].as_ref().unwrap().expiry,4);
            }
            let got=g.events.iter().filter(|e|e["kind"]=="nativeSample").map(|e|e["sampleId"].as_i64().unwrap()).collect::<Vec<_>>();
            assert_eq!(got,if n("sound")<0{vec![]}else{vec![n("sound")]},"splash audio case {case}");
            assert_eq!(g.native_audio.effect_counter,n("counter") as i16,"case {case}");
            if let Some(slot)=slot{assert_eq!(g.native_ambient_animations.records[slot].bytes.as_slice(),b("ambientAfter"),"ambient slot {slot}, case {case}");}
        }else{
            g.finish_native_water_expiry(1);
            assert_eq!(g.native_world.as_ref().unwrap().effect_tint,n("tint") as u8,"spray tint case {case}");
        }
        if n("blast")==0{
            let got=g.native_world.as_ref().unwrap().effects.iter().flatten().flat_map(effects::Particle::bytes).collect::<Vec<_>>();
            assert_eq!(got,b("lateAfter"),"late records case {case}");
        }
        let plan=g.native_audio.opportunity_plan(Chance::FortDebrisWater4{point},view.0,view.1).unwrap();
        if n("sound")==9{assert_eq!(plan.request.unwrap().sample,9);}
    }
}
