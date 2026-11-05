use crate::{Game,original_combat::OriginalRng};
fn field()->Game{
    let mut g=Game::new(0,1,5508);g.map.original=true;g.units.clear();
    g.buildings.clear();g.sectors.clear();g.objects.clear();g.crates.clear();
    g.walls.clear();g.native_group_rosters.clear();g.native_world=None;
    g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];
    g.native_selection=Default::default();g.native_upstream_reactions.clear_for_mission();
    g.native_cranes.clear_for_mission();g
}
fn spawn(g:&mut Game,kind:&str,slot:usize,full:bool,squad:usize)->usize{
    for _ in 0..slot{g.spawn("grunt",0,640.,640.,1000,None);}
    if full{for w in &mut g.native_hardware_pool{w.live_marker=0;}}
    let index=g.units.len();g.spawn(kind,0,800.,1000.,squad,None);
    assert_eq!(g.units[index].native_slot,Some(slot));index
}
#[test]fn starting_gun_saved_classes_follow_successful_setup_without_granting_failed_pools(){
    for(kind,class)in[("cannon",2),("missile",4),("gatling",5),("howitzer",6)]{
        for slot in [0,17,239]{for full in [false,true]{for mp in [false,true]{
            let mut g=field();g.multiplayer=mp;g.original_rng=OriginalRng{words:[1,2,3,4,5],multiplayer:mp};
            let i=spawn(&mut g,kind,slot,full,17);assert_eq!(g.units[i].native_hardware_slot.is_some(),!full);
            assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(if full{0}else{class}),"{kind}/slot{slot}/full{full}/mp{mp}");
            assert_eq!(g.original_rng.words,[1,2,3,4,5]);
        }}}
    }
}
#[test]fn starting_gun_context_does_not_overwrite_later_class_writes_and_fresh_reuse_clears_it(){
    let mut g=field();spawn(&mut g,"gatling",0,false,1);
    assert_eq!(g.native_selection.contexts[0].saved_class8e,Some(5));
    g.record_native_selection_saved_class(0,15);g.maintain_native_group_rosters();g.import_native_world(&crate::Value::Null);
    assert_eq!(g.native_selection.contexts[0].saved_class8e,Some(15),"membership/import is not a constructor replay");
    g.units[0].hp=-1.;g.units[0].native_slot=None;g.spawn("grunt",0,800.,1000.,2,None);
    assert_eq!(g.units.last().unwrap().native_slot,Some(0));assert_eq!(g.native_selection.contexts[0].saved_class8e,Some(0));
}
#[test]
#[ignore="requires separately owned direct starting-gun Source controls"]
fn starting_gun_saved_class_matches96_original_capacity_controls(){
    let path=std::env::var_os("NATIVE_STARTGUN_CAPACITY_TRACE").expect("owned Source constructor recording path");
    let rows=std::fs::read_to_string(path).unwrap().lines().map(|l|serde_json::from_str::<crate::Value>(l).unwrap()).collect::<Vec<_>>();
    assert_eq!(rows.len(),192);let mut count=0;
    for pair in rows.chunks_exact(2){let(a,b)=(&pair[0],&pair[1]);assert_eq!(a["stage"],"before");assert_eq!(b["stage"],"after");
        let kind=match a["groupId"].as_u64().unwrap(){0=>"cannon",8=>"missile",10=>"gatling",14=>"howitzer",_=>panic!("unexpected controlled gun group")};
        let mut g=field();g.multiplayer=a["mp"]==1;g.original_rng=OriginalRng{words:std::array::from_fn(|i|a["rng"][i].as_u64().unwrap()as u32),multiplayer:g.multiplayer};
        let slot=a["physical"].as_u64().unwrap()as usize;let full=a["full"]==1;
        let i=spawn(&mut g,kind,slot,full,a["logical"].as_u64().unwrap()as usize+1);
        let raw=b["unit"].as_str().unwrap();let saved=u8::from_str_radix(&raw[0x8e*2..0x8e*2+2],16).unwrap();
        assert_eq!(g.native_selection.contexts[slot].saved_class8e,Some(saved),"scenario{}",a["scenario"]);
        assert_eq!(g.units[i].native_hardware_slot.is_some(),!full);
        assert_eq!(g.original_rng.words,std::array::from_fn(|i|b["rng"][i].as_u64().unwrap()as u32));
        // Deliberately only8e/allocation/RNG. Failed-W Source7c/35/39 and
        // the entire startup/global/readset are separate remaining work.
        count+=1;
    }
    assert_eq!(count,96);
}
