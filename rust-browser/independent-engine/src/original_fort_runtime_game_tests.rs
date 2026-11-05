use super::*;
use serde_json::Value;

fn load(mission:&Value)->Game {
    load_mode(mission,false)
}

fn load_mode(mission:&Value,multiplayer:bool)->Game {
    let mut g=Game::new(mission["number"].as_u64().unwrap() as usize-1,1,19);
    g.multiplayer=multiplayer;g.original_rng.multiplayer=multiplayer;
    g.apply_original(mission).unwrap();g.ai_teams.clear();g.events.clear();
    assert!(g.native_fort_sequence_enabled());
    g.native_view=Some(([240,140],[480,320]));
    g.native_feedback.sound_enabled=1;
    if multiplayer{g.set_native_listener_settings(&json!({"0":{"soundEnabled":true},"1":{"soundEnabled":true}}));}
    g
}

fn enter(g:&mut Game,actor:i32,door_code:u8){
    let i=g.units.iter().position(|u|u.team==actor&&u.kind=="grunt").or_else(||g.units.iter().position(|u|u.team==actor&&u.native_slot.is_some())).unwrap();
    let (cell,_)=g.map.terrain_codes.iter().enumerate().find(|&(_,v)|*v==door_code).unwrap();
    let (x,y)=(cell%g.map.width,cell/g.map.width);
    g.units[i].x=(x*32) as f64;g.units[i].y=(y*32) as f64;
    g.units[i].native_word90=450;
    g.refresh_native_main_pattern_step(i,0);
    assert_eq!(g.units[i].native_word90,0);
    assert_eq!(g.native_group_rosters.members[g.units[i].native_slot.unwrap()].as_ref().unwrap().flags69&4,0);
}

#[test]
#[ignore="requires locally approved original campaign data"]
fn original_campaign_fort_door_arms_the_full_battle_scene() {
    let path=std::env::var("NATIVE_FORT_CAMPAIGN").expect("private campaign path");
    let data:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let missions=data["missions"].as_array().unwrap();
    assert_eq!(missions.len(),20);
    for mission in missions {
        for (winner,door,owner) in [(0,21,2),(1,20,1)] {
            let mut g=load(mission);
            enter(&mut g,winner,door);
            let w=g.native_world.as_ref().unwrap();
            assert_eq!(w.fort_runtime.boundary,None,"mission {}",mission["number"]);
            assert_eq!(w.fort_runtime.payload,winner as u32);
            assert_eq!(w.fort_runtime.owner,owner);
            assert_eq!(w.fort_scene_queue.counter(0),1);
            assert_eq!(w.fort_team_activity[usize::from(owner-1)],2);
            assert_eq!(g.winner,None);
        }
    }

    // Run actual Main ticks for both directions in one complete mission.
    for (winner,door,owner) in [(0,21,2),(1,20,1)] {
        let mut g=load(&missions[0]);
        let base=g.native_fort_bases().unwrap()[usize::from(owner==1)];
        enter(&mut g,winner,door);
        assert!(g.events.iter().any(|e|e["kind"]=="nativeFortCloseProduction"));
        let mut panning=0;
        let mut result_radio=0;
        for frame in 2..=170 {
            g.step(0.059);
            let w=g.native_world.as_ref().unwrap();
            assert_eq!(w.fort_runtime.boundary,None,"owner {owner}, frame {frame}");
            let expected=if frame==170{-1}else{frame};
            assert_eq!(w.fort_scene_queue.counter(0),expected,"owner {owner}, frame {frame}");
            assert_eq!(w.fort_end.scene_dirty!=0,matches!(frame,17|170));
            let pulses=frame-1-i32::from(frame>=18);
            assert_eq!(w.fort_runtime.score_clock.seconds,pulses/17);
            assert_eq!(w.fort_runtime.score_clock.subtick,(pulses%17) as i16);
            assert_eq!(w.fort_team_activity[usize::from(owner-1)],if frame<17{2}else{0});
            assert_eq!(g.winner,(frame==170).then_some(winner));
            if w.fort_runtime.last_camera.is_some(){panning+=1;}
            if frame==18{assert_eq!(w.fort_runtime.last_camera,Some([base[0]-96,base[1]-24]));}
            if frame==2 {
                assert!(g.events.iter().any(|e|e["kind"]=="nativeFortHaltMixer"));
                assert!(g.events.iter().any(|e|e["kind"]=="nativeFortDeselect"));
            }
            if frame==15{assert_eq!(w.fort_runtime.redraw,1);}
            if frame>=17{assert_eq!(w.fort_end.defeated_owner,owner);}
            result_radio+=g.events.iter().filter(|e|e["kind"]=="nativeVoice"&&matches!(e["message"].as_u64(),Some(0x0a3d..=0x0a42|0x0b43..=0x0b45))).count();
        }
        assert_eq!(panning,16);
        assert!(result_radio>0,"owner {owner}");
    }

    // A real blast callback uses cause 4 and waits for signed HP below zero.
    let mut g=load(&missions[0]);
    let [x,y]=g.native_fort_bases().unwrap()[0].map(i32::from);
    g.buildings.iter_mut().find(|b|b.kind=="fort"&&b.team==1).unwrap().hp=1.;
    g.native_blast_environment(x+36,y+32,1,0,false);
    assert_eq!(g.native_feedback.scene_gate,0);
    assert_eq!(g.winner,None);
    g.native_blast_environment(x+36,y+32,1,0,false);
    assert_eq!(g.native_feedback.scene_gate,1);
    let w=g.native_world.as_ref().unwrap();
    assert_eq!(w.fort_runtime.payload,4);
    assert_eq!(w.fort_runtime.owner,2);
    assert_eq!(w.fort_scene_queue.counter(0),1);
    assert_eq!(g.winner,None);
}

#[test]
#[ignore="requires locally approved original campaign data"]
fn original_mission_one_infantry_can_march_through_the_enemy_fort_gate() {
    let path=std::env::var("NATIVE_FORT_CAMPAIGN").expect("private campaign path");
    let data:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut g=load(&data["missions"][0]);
    // Isolate traversal from the original fort defenders' weapon hits.
    for i in 0..g.units.len(){if g.units[i].team==1&&g.units[i].native_slot.is_some(){
        let point=[(g.units[i].x/2.) as i16,(g.units[i].y/2.) as i16];
        g.native_direct_removal(i,point,true,false);
    }}
    g.spawn("grunt",1,1600.,880.,199,None);g.maintain_native_group_rosters();
    let mut players=g.units.iter().filter(|u|u.team==0&&u.kind=="grunt").map(|u|u.id).take(2);
    let first=players.next().unwrap();let second=players.next().unwrap();
    for (u,x) in [(first,1008.),(second,1036.)]{let u=g.units.iter_mut().find(|v|v.id==u).unwrap();u.x=x;u.y=600.;}
    let index=g.units.iter().position(|u|u.id==first).unwrap();
    assert_eq!(g.classify_native_human_move(index,[512,248],false).unwrap().waypoint,[512,192]);
    g.command(&json!({"ids":[first,second],"kind":"attackBuilding","target":g.buildings.iter().find(|b|b.kind=="fort"&&b.team==1).unwrap().id,"nativeClick":[512,248],"x":1024,"y":496}));
    let mut trace=Vec::new();
    for n in 0..180 {
        if n<4||n%12==0{let u=g.units.iter().find(|u|u.id==first).unwrap();let m=g.native_group_rosters.members[u.native_slot.unwrap()].as_ref().unwrap();trace.push((n,[u.x,u.y],u.dest,[i32::from(u.native_order5c),i32::from(u.native_phase35),m.promotion.target_x,i32::from(m.promotion.target_y)],u.path.iter().next().copied()));}
        g.step(0.059);
        if g.native_feedback.scene_gate!=0{break;}
    }
    let w=g.native_world.as_ref().unwrap();
    let positions:Vec<_>=g.units.iter().filter(|u|u.id==first||u.id==second).map(|u|(u.x,u.y,u.dest,u.order.clone())).collect();
    assert_eq!(w.fort_runtime.boundary,None);
    assert_eq!(w.fort_runtime.owner,2,"positions {positions:?}, route {:?}, trace {trace:?}",g.native_route_boundary);
}

#[test]
#[ignore="requires locally approved original campaign data"]
fn multiplayer_fort_orders_are_staged_and_consumed_in_actual_main_ticks(){
    let path=std::env::var("NATIVE_FORT_CAMPAIGN").expect("private campaign path");
    let data:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mission=&data["missions"][0];
    for (winner,door,owner) in [(0,21,2),(1,20,1)] {
        let mut g=load_mode(mission,true);
        let slot=g.units.iter().find(|u|u.team==winner&&u.kind=="grunt").unwrap().native_slot.unwrap();
        let base=g.native_fort_bases().unwrap()[usize::from(owner==1)];
        enter(&mut g,winner,door);
        let target=[base[0]+80,base[1]+if owner==2{300}else{-300}];
        let mut cues=[0;2];
        for frame in 2..=170 {
            g.step(0.059);
            let world=g.native_world.as_ref().unwrap();
            let runtime=&world.fort_runtime;
            assert_eq!(runtime.boundary,None,"owner {owner}, frame {frame}");
            assert_eq!(runtime.peer_clock,frame-1);
            assert_eq!(g.winner,(frame==170).then_some(winner));
            if frame==2{
                let snapshot=g.native_fort_scene_snapshot().unwrap();
                assert_eq!(snapshot["panRequest"],json!([base[0]-64,base[1]-32]));
                assert!(snapshot["camera"].is_null());
            }
            if (3..=23).contains(&frame){
                let queue=&runtime.peer_orders.as_ref().unwrap().host;
                let rows:Vec<_>=queue.rings.iter().map(|ring|ring.iter().filter(|r|r[8]==slot as u8&&
                    i16::from_le_bytes([r[12],r[13]])==target[0]&&i16::from_le_bytes([r[14],r[15]])==target[1]&&
                    i32::from_le_bytes(r[..4].try_into().unwrap())==22).count()).collect();
                assert_eq!(rows,if frame<23{[1,1]}else{[0,0]},"queued sweep: owner {owner}, frame {frame}");
                if frame==23 {
                    let member=g.native_group_rosters.members[slot].as_ref().unwrap();
                    assert_eq!(member.promotion.target_aux,3);
                    assert_eq!([member.promotion.target_x as i16,member.promotion.target_y],target);
                }
            }
            for e in &g.events{
                if e["kind"]=="nativeVoice"&&matches!(e["message"].as_u64(),Some(0x0a3d..=0x0a42|0x0b43..=0x0b45)){
                    let listener=e["recipientTeam"].as_u64().unwrap() as usize;
                    cues[listener]+=1;
                    assert_eq!(e["message"],if listener==winner as usize{0x0a3d}else{0x0b43});
                }
            }
        }
        assert!(cues[winner as usize]>0,"original winner cue");
        if owner==1{assert!(cues[0]>0,"local losing side is still physically present");}
    }
}
