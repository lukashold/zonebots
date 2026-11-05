use crate::{Game,Crew,original_path_contact::game::Boundary,
    original_apc_automatic::Boundary as Missing};
use crate::original_world_contact_game::crane_state::State;
fn field(kind:&str)->Game {
    let mut g=Game::new(0,1,5306);g.map.original=true;g.units.clear();
    g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();
    g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();
    g.native_group_rosters.clear();g.native_upstream_reactions.clear_for_mission();
    g.native_cranes.clear_for_mission();g.native_world=None;g.native_ambient=None;
    g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];
    g.spawn("apc",0,1000.,1200.,1,Some(Crew{kind:kind.into(),hp:100.}));
    for _ in 0..4{g.spawn("grunt",0,1000.,1200.,1,None);}
    g.maintain_native_group_rosters();g
}
fn phase(g:&Game,slot:usize)->u8{g.native_cranes.phase_b6(slot,g.native_upstream_reactions.generation(slot).unwrap()).unwrap()}
fn seed(g:&mut Game,slot:usize,n:u8){g.native_cranes.seed_phase_b6(slot,g.native_upstream_reactions.generation(slot).unwrap(),n);}
fn roster(g:&mut Game,members:&[u8],count:i8)->usize{
    let slot=g.units[0].native_slot.unwrap();let group=usize::from(g.native_group_rosters.members[slot].as_ref().unwrap().group);
    let r=&mut g.native_group_rosters.pool[group].roster;r[19..40].fill(255);
    r[19..19+members.len()].copy_from_slice(members);r[18]=count as u8;group
}
#[test]fn constructor_phases_cover_all240_slots_without_world_or_renderer_defaults(){
    let mut s=State::default();
    for slot in 0..240{s.constructed(slot,7,1000+slot,false);assert_eq!(s.phase_b6(slot,7),Some(slot as u8&15));assert_eq!(s.phase_b6(slot,6),None);}
    assert_eq!(s.phase_b6(240,7),None);
    s.view_for_legacy_crane(239,8,9);assert_eq!(s.phase_b6(239,8),None,"a warm legacy work view is not a constructor");
    assert_eq!(s.advance_phase_b6(239,8),None);s.constructed(239,9,9,false);assert_eq!(s.phase_b6(239,9),Some(15));
}
#[test]fn phase_survives_rebinding_work_dto_and_initializer_but_reuse_resets_it(){
    let mut s=State::default();s.constructed(17,1,30,false);assert_eq!(s.advance_phase_b6(17,1),Some(2));
    assert!(s.rebind(17,1,30,40));s.initializer57ff0(17,1,40);
    let c=s.view_for_legacy_crane(17,1,40);s.store(17,1,c);s.constructed(17,1,40,true);
    assert_eq!(s.phase_b6(17,1),Some(2));s.constructed(17,2,50,false);
    assert_eq!(s.phase_b6(17,1),None);assert_eq!(s.phase_b6(17,2),Some(1));
    s.clear_for_mission();assert_eq!(s.phase_b6(17,2),None);
}
#[test]fn manual_reads_physical_duplicates_holes_and_inactive_members_without_hp_filter(){
    for mp in [false,true]{let mut g=field("tough");g.multiplayer=mp;g.original_rng.multiplayer=mp;
        roster(&mut g,&[3,255,1,3,0],5);g.units[3].hp=-1.;g.native_group_rosters.members[3].as_mut().unwrap().flags31&=!4;
        let mut rng=g.original_rng.clone();for _ in 0..8{rng.next_u32();}
        assert!(g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap());
        assert_eq!([phase(&g,0),phase(&g,1),phase(&g,2),phase(&g,3)],[1,2,2,5]);
        assert_eq!(g.projectiles.len(),4);assert_eq!(g.original_rng.words,rng.words);
    }
}
#[test]fn manual_uses_signed_count_even_when_later_raw_roster_bytes_are_present(){
    for count in [0,-1,-128]{let mut g=field("tough");roster(&mut g,&[3,1,0],count);let rng=g.original_rng.words;
        assert!(!g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap());
        assert_eq!(phase(&g,3),3);assert_eq!(g.original_rng.words,rng);assert_eq!(g.units[0].native_reload7b,20);
    }
    let mut g=field("tough");roster(&mut g,&[3,1,0],1);g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap();
    assert_eq!([phase(&g,3),phase(&g,1),phase(&g,0)],[4,1,0]);
}
#[test]fn automatic_and_manual_share_the_physical_pyro_gate_and_write_before_skip(){
    let mut g=field("pyro");roster(&mut g,&[3,3,255,1],4);seed(&mut g,3,2);seed(&mut g,1,15);
    g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap();assert_eq!(phase(&g,3),4);assert_eq!(phase(&g,1),0);
    assert_eq!(g.projectiles.len(),2,"duplicate3 advances to4 and skips, slot1 wraps to0");
    g.units[0].native_reload7b=0;g.units[0].cooldown=0.;g.spawn("heavy",1,1000.,1040.,99,None);
    g.update_native_apc_auto_tail_legacy(0).unwrap();assert_eq!(phase(&g,3),6);assert_eq!(phase(&g,1),1);
    g.units[0].native_reload7b=0;g.units[0].cooldown=0.;g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap();
    assert_eq!(phase(&g,3),8);assert_eq!(phase(&g,1),2);
}
#[test]fn invalid_member_keeps_preceding_phase_and_rng_without_replaying_a_legacy_loop(){
    let mut g=field("tough");roster(&mut g,&[3,240,1],3);let mut rng=g.original_rng.clone();rng.next_u32();rng.next_u32();
    assert_eq!(g.fire_native_apc_point(0,(800,900),usize::MAX),Err(Boundary::ApcPhase(Missing::Member(240))));
    assert_eq!(phase(&g,3),4);assert_eq!(phase(&g,1),1);assert_eq!(g.projectiles.len(),1);
    assert_eq!(g.original_rng.words,rng.words);assert_eq!(g.units[0].native_reload7b,0,"the final writer was not reached");
}
#[test]fn missing_constructor_phase_cannot_be_fabricated_from_cargo_or_native_slot(){
    let mut g=field("tough");roster(&mut g,&[3],1);g.native_cranes.slots[3]=None;
    let rng=g.original_rng.words;assert_eq!(g.fire_native_apc_point(0,(800,900),usize::MAX),Err(Boundary::ApcPhase(Missing::Missing("physical B6 receipt"))));
    assert_eq!(g.original_rng.words,rng);assert!(g.projectiles.is_empty());assert!(g.native_cranes.slots[3].is_none());
}
#[test]fn renderer_cargo_is_not_an_extra_physical_passenger_or_phase_owner(){
    let mut g=field("tough");roster(&mut g,&[0],1);g.units[0].cargo=vec![Crew{kind:"tough".into(),hp:100.};20];
    assert!(g.fire_native_apc_point(0,(800,900),usize::MAX).unwrap());assert_eq!(g.projectiles.len(),1);assert_eq!(phase(&g,0),1);
}
#[test]fn automatic_empty_scan_and_reload_do_not_touch_any_passenger_phase(){
    let mut g=field("tough");roster(&mut g,&[3,1,0],3);g.update_native_apc_auto_tail_legacy(0).unwrap();
    assert_eq!([phase(&g,3),phase(&g,1),phase(&g,0)],[3,1,0]);
    g.spawn("heavy",1,1000.,1040.,99,None);g.units[0].native_reload7b=1;
    g.update_native_apc_auto_tail_legacy(0).unwrap();assert_eq!(phase(&g,3),3);
}
#[test]fn automatic_invalid_member_preserves_reload_and_preceding_physical_write(){
    let mut g=field("tough");roster(&mut g,&[3,241,1],3);
    g.spawn("heavy",1,1000.,1040.,99,None);
    assert_eq!(g.update_native_apc_auto_tail_legacy(0),Err(Boundary::ApcPhase(Missing::Member(241))));
    assert_eq!(phase(&g,3),4);assert_eq!(phase(&g,1),1);
    assert_ne!(g.units[0].native_reload7b,0,"automatic reload precedes the roster scan");
    assert_eq!(g.projectiles.len(),1);
}
#[test]
#[ignore="requires the separately owned original447da0 phase recording"]
fn physical_phase_and_rng_replay384_original_manual_controls(){
    let path=std::env::var_os("NATIVE_APC_PHASE_MANUAL_TRACE").expect("owned Source recording path");
    let data=std::fs::read_to_string(path).unwrap();
    let rows=data.lines().map(|s|serde_json::from_str::<crate::Value>(s).unwrap()).collect::<Vec<_>>();
    assert_eq!(rows.len(),768);let mut hash=2166136261u32;
    fn bytes(s:&str)->Vec<u8>{assert_eq!(s.len()%2,0);(0..s.len()).step_by(2).map(|n|u8::from_str_radix(&s[n..n+2],16).unwrap()).collect()}
    for (scenario,pair) in rows.chunks_exact(2).enumerate(){
        let(a,b)=(&pair[0],&pair[1]);assert_eq!(a["scenario"],scenario);assert_eq!(a["stage"],"before");assert_eq!(b["stage"],"after");
        let kind=if a["class"]==3{"tough"}else{"pyro"};let mut g=field(kind);
        for _ in 5..240{g.spawn("grunt",0,1000.,1200.,1,None);}
        assert_eq!(g.units.len(),240);g.maintain_native_group_rosters();
        g.units[17].kind="apc".into();g.units[17].crew=Some(Crew{kind:kind.into(),hp:100.});
        let hw=bytes(a["hardware"].as_str().unwrap());g.units[17].original_combat.as_mut().unwrap().fire_rate=hw[32];
        g.native_group_rosters.members[17].as_mut().unwrap().group=23;
        g.native_group_rosters.pool[23].roster.copy_from_slice(&bytes(a["group"].as_str().unwrap()));
        let before=bytes(a["phases"].as_str().unwrap());for slot in 0..240{seed(&mut g,slot,before[slot]);}
        g.multiplayer=a["mp"]==1;g.original_rng.multiplayer=g.multiplayer;
        g.original_rng.words=std::array::from_fn(|i|a["rng"][i].as_u64().unwrap()as u32);
        // This replay compares B6/reload/RNG only. A full EXISTING Game pool
        // gives normal no-allocation returns; Source's separate speed0 leaves
        // also return without allocation. No emitter or whole Game claim.
        let u=g.units[17].clone();for _ in 0..50{assert!(g.emit_native_projectile(&u,(100,100),(1200,1200),crate::original_projectiles::Kind::Rocket,15,1600,crate::original_combat::Attack::from_unit(&u,usize::MAX,18.,0.)));}
        assert!(!g.fire_native_apc_point(17,(800,900),usize::MAX).unwrap());
        let after=(0..240).map(|s|phase(&g,s)).collect::<Vec<_>>();assert_eq!(after,bytes(b["phases"].as_str().unwrap()),"scenario{scenario}");
        assert_eq!(g.units[17].native_reload7b,30);assert_eq!(g.projectiles.len(),50);
        assert_eq!(g.original_rng.words,std::array::from_fn(|i|b["rng"][i].as_u64().unwrap()as u32));
        for n in std::iter::once(scenario as u32).chain(after.iter().map(|&v|u32::from(v))).chain(std::iter::once(u32::from(g.units[17].native_reload7b as u8))).chain(g.original_rng.words){for byte in n.to_le_bytes(){hash=(hash^u32::from(byte)).wrapping_mul(16777619);}}
    }
    assert_eq!(hash,1723361704);println!("384 physical manual B6/reload/RNG projections: FNV{hash}");
}
