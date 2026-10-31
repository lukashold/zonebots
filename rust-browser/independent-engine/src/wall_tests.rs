//! Synthetic fixtures exercise wall behavior without publishing original maps.
use super::*;

fn field() -> Game {
    let mut g = Game::new(0, 1, 1729);
    g.map.original = true; g.map.world = 0;
    g.map.tiles.fill(0); g.map.terrain_codes = vec![0; g.map.tiles.len()];
    g.map.original_tiles = vec![17; g.map.tiles.len()];
    g.map.territory.fill(0); g.map.bridge_base.fill(-1); g.map.bridge_hp.clear();
    g.units.clear(); g.buildings.clear(); g.objects.clear(); g.crates.clear(); g.effects.clear();
    g.sectors = vec![Sector { id: 0, x: -1000., y: -1000., owner: -1, home: true, counts_for_production: false }];
    g.ai_teams.clear(); g.ai_clock = 10_000.; g.winner = None;
    g.spawn("grunt", 1, 1680., 1200., 80, None);
    let enemy = g.units.last_mut().unwrap(); enemy.cooldown = 10_000.; enemy.hold_position = true;
    g
}

fn wall(id: usize, x: f64, y: f64, hp: f64) -> Value {
    json!({"id":id,"x":x,"y":y,"w":32.,"h":32.,"hp":hp,"maxHp":hp})
}
fn shooter(g: &mut Game, kind: &str, x: f64, y: f64) -> usize {
    g.spawn(kind, 0, x, y, 81, None);
    let u = g.units.last_mut().unwrap(); u.hold_position = true; u.cooldown = 0.; u.id
}
fn unit(g: &Game, id: usize) -> &Unit { g.units.iter().find(|u| u.id == id).unwrap() }

#[test]
fn imported_walls_block_terrain_los_and_preserve_exact_ground_on_collapse() {
    let mut g = field(); g.import_walls(&json!({"walls":[wall(7,640.,320.,80.)]})).unwrap();
    assert!(!g.passable_for(20,10,"light")); assert!(!g.passable_for(20,10,"grunt"));
    assert!(!g.los(600.,336.,700.,336.)); assert!(g.wall_los(600.,336.,656.,336.,7));
    assert!(!g.damage_wall(7,20.,0)); assert_eq!(g.walls[0].hp,60.); assert_eq!(g.map.revision,0);
    assert!(g.damage_wall(7,60.,0)); assert_eq!(g.map.revision,1);
    assert!(g.passable_for(20,10,"light")); assert!(g.los(600.,336.,700.,336.));
    let cell=10*g.map.width+20; assert_eq!(g.map.original_tiles[cell],17); assert_eq!(g.map.terrain_codes[cell],0);
    assert_eq!(g.events.iter().filter(|e|e["kind"]=="wallDestroyed").count(),1);
    assert!(!g.damage_wall(7,60.,0)); assert_eq!(g.map.revision,1,"A dead fragment cannot collapse twice");
}

#[test]
fn native_zero_hp_is_intact_until_first_positive_contact() {
    let mut g = field(); let mut input=wall(7,640.,320.,0.); input["nativeHp"]=json!(0);
    g.import_walls(&json!({"walls":[input]})).unwrap();
    assert!(g.walls[0].alive()); assert!(!g.passable_for(20,10,"light"));
    assert!(g.damage_wall(7,1.,0)); assert!(g.passable_for(20,10,"light"));
}

#[test]
fn wall_attack_fires_at_selected_wall_and_stops_on_collapse() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,640.,320.,10.)]})).unwrap();
    let id=shooter(&mut g,"light",480.,336.); g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));
    g.step(0.05); assert!(!g.walls[0].alive()); assert_eq!(unit(&g,id).order,"guard");
    assert!(unit(&g,id).path.is_empty()); assert!(g.effects.iter().any(|fx|fx.kind=="shot"&&fx.x2==640.));
}

#[test]
fn distant_tank_reaches_a_valid_firing_position_without_entering_wall() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,1152.,320.,10.)]})).unwrap();
    let id=shooter(&mut g,"light",96.,336.); g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));
    assert!(!unit(&g,id).path.is_empty());
    for _ in 0..1000 { g.step(0.05); if !g.walls[0].alive(){break;} }
    assert!(!g.walls[0].alive()); assert!(unit(&g,id).x<1152.,"Stop outside the wall, not inside its footprint");
}

#[test]
fn an_intervening_cliff_or_other_wall_cannot_be_shot_through() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,640.,320.,10.),wall(8,576.,320.,10.)]})).unwrap();
    assert!(!g.wall_los(480.,336.,640.,336.,7));
    let j=10*g.map.width+17; g.map.tiles[j]=2;
    assert!(!g.wall_los(480.,336.,576.,336.,8));
    assert!(!g.damage_wall(999,1000.,0)); assert_eq!(g.map.tiles[j],2,"Permanent cliffs have no destructible wall identity");
}

#[test]
fn destruction_does_not_clear_overlapping_live_wall_cells() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,640.,320.,10.),wall(8,640.,320.,10.)]})).unwrap();
    assert!(g.damage_wall(7,10.,0)); assert!(!g.passable_for(20,10,"light"));
    assert!(g.damage_wall(8,10.,0)); assert!(g.passable_for(20,10,"light"));
}

#[test]
fn manual_stop_and_foreign_team_commands_cancel_or_cannot_replace_wall_order() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,1152.,320.,10.)]})).unwrap();
    let id=shooter(&mut g,"light",96.,336.); g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));
    g.command(&json!({"team":1,"ids":[id],"kind":"attackWall","target":7})); assert_eq!(unit(&g,id).target,Some(7));
    g.command(&json!({"ids":[id],"kind":"stop"})); g.step(0.05); assert_eq!(unit(&g,id).order,"guard"); assert!(g.walls[0].alive());
}

#[test]
fn compact_snapshots_keep_dynamic_wall_health_and_map_revision() {
    let mut g=field(); g.import_walls(&json!({"walls":[wall(7,640.,320.,80.)]})).unwrap();
    let request=json!({"op":"tick","compact":true,"knownMapSeed":g.map.seed,"knownMapRevision":g.map.revision});
    let before=game_snapshot(&g,&request); assert!(before.get("map").is_none()); assert_eq!(before["walls"][0]["hp"],80.);
    g.damage_wall(7,20.,0); let hurt=game_snapshot(&g,&request); assert!(hurt.get("map").is_none()); assert_eq!(hurt["walls"][0]["hp"],60.);
    g.damage_wall(7,60.,0); let dead=game_snapshot(&g,&request); assert!(dead.get("map").is_some()); assert_eq!(dead["walls"][0]["hp"],0.);
}

#[test]
fn optional_owned_campaign_wall_composition_matches_the_extractor_for_all_twenty_maps() {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/original/campaign.json");
    let Ok(data)=std::fs::read(path) else{return;};
    let source:Value=serde_json::from_slice(&data).unwrap();
    if source["missions"][0].get("walls").is_none(){return;}
    for mission in source["missions"].as_array().unwrap() {
        let mut g=Game::new(0,1,17);g.apply_original(mission).unwrap();
        let expected:Map=serde_json::from_value(mission["map"].clone()).unwrap();
        assert_eq!(g.map.original_tiles,expected.original_tiles,"Mission{}: visual forms",mission["number"]);
        assert_eq!(g.map.terrain_codes,expected.terrain_codes,"Mission{}: native collision codes",mission["number"]);
        assert_eq!(g.map.tiles,expected.tiles,"Mission{}: compatibility classes",mission["number"]);
        assert_eq!(g.walls.len(),mission["walls"].as_array().unwrap().len());
    }
}

fn native_lookup() -> Value {
    json!({"version":1,"headTiles":(100..116).collect::<Vec<_>>(),
        "interiorTiles":(200..216).collect::<Vec<_>>(),"frontRemap":{"115":300},
        "frontStep":20,"rubbleTiles":(400..408).collect::<Vec<_>>(),"rubbleCode":22})
}
fn fragment(id:usize,x:f64,y:f64,record:usize,bit:usize)->Value {
    let mut result=wall(id,x,y,5.);result["h"]=json!(96.);result["height"]=json!(2);
    result["wallRecord"]=json!(record);result["wallBit"]=json!(bit);result["variant"]=json!(3);result
}

#[test]
fn native_neighbor_forms_front_remaps_and_rubble_update_after_one_piece_breaks() {
    let mut g=field();g.map.wall_tile_lookup=native_lookup();
    g.import_walls(&json!({"walls":[fragment(7,640.,320.,0,0),fragment(8,672.,320.,0,1)]})).unwrap();
    let cell=10*g.map.width+20;
    assert_eq!(g.map.original_tiles[cell],113,"East neighbor removes exposed-E flag");
    assert_eq!(g.map.original_tiles[cell+1],107,"West neighbor removes exposed-W flag");
    g.damage_wall(8,5.,0);
    assert_eq!(g.map.original_tiles[cell],115,"Surviving neighbor gains its closed end cap");
    assert_eq!(g.map.original_tiles[cell+g.map.width],320,"Front remap retains the 256 high bit");
    assert_eq!(g.map.original_tiles[cell+2*g.map.width],340);
    for dy in 0..3 {
        let j=cell+1+dy*g.map.width;
        assert_eq!(g.map.original_tiles[j],400+dy as u16);
        assert_eq!(g.map.terrain_codes[j],22);assert_eq!(g.map.tiles[j],0);
        assert!(g.passable_for(21,10+dy as i32,"light"));
    }
}

#[test]
fn native_records_stay_separate_and_live_fronts_overlay_destroyed_fragments() {
    let mut g=field();g.map.wall_tile_lookup=native_lookup();
    g.import_walls(&json!({"walls":[fragment(7,640.,320.,0,0),fragment(8,672.,320.,1,0),fragment(9,640.,352.,0,32)]})).unwrap();
    assert_eq!(g.walls[1].neighbor_mask,15,"Adjacent different records never join");
    let cell=10*g.map.width+20;g.damage_wall(9,5.,0);
    assert_eq!(g.map.tiles[cell+g.map.width],6);assert_eq!(g.map.tiles[cell+2*g.map.width],6);
    assert_eq!(g.map.tiles[cell+3*g.map.width],0,"The dead lowest front opens without deleting the upper wall");
    assert_eq!(g.map.terrain_codes[cell+3*g.map.width],22);
}

#[test]
fn an_overlapping_southern_head_blocks_fire_at_the_upper_wall_but_can_itself_be_hit() {
    let mut g=field();g.map.wall_tile_lookup=native_lookup();
    g.import_walls(&json!({"walls":[fragment(7,640.,320.,0,0),fragment(9,640.,352.,0,32)]})).unwrap();
    assert!(!g.wall_los(656.,464.,656.,416.,7),"Lower wall owns the upper fragment's visible front cells");
    assert!(g.wall_los(600.,368.,640.,368.,9),"The lower head is visible from its flank, despite the upper fragment behind it");
    assert!(!g.wall_los(600.,368.,640.,368.,7),"An explicit upper target cannot ignore the lower visible head");
    let id=shooter(&mut g,"cannon",656.,464.);
    g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));g.step(0.05);
    assert_eq!(g.walls[0].hp,5.);assert_eq!(g.walls[1].hp,5.);assert!(!g.events.iter().any(|e|e["kind"]=="shot"));
    g.command(&json!({"ids":[id],"kind":"attackWall","target":9}));g.step(0.05);
    assert_eq!(g.walls[0].hp,5.);assert_eq!(g.walls[1].hp,0.);
}

#[test]
fn machine_gun_hardware_and_empty_infantry_cannot_damage_stone() {
    for kind in ["grunt","psycho","sniper","laser","pyro","jeep","gatling","apc","crane"] {
        let mut g=field();g.import_walls(&json!({"walls":[wall(7,640.,320.,5.)]})).unwrap();
        let id=shooter(&mut g,kind,480.,336.);g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));
        g.step(0.05);assert_eq!(g.walls[0].hp,5.,"{kind} without explosives");
        assert!(!g.events.iter().any(|event|event["kind"]=="shot"));
    }
}

#[test]
fn all_five_native_terrain_grenade_robot_classes_spend_one_only_on_contact() {
    for kind in ["grunt","psycho","sniper","laser","pyro"] {
        let mut g=field();g.import_walls(&json!({"walls":[wall(7,512.,320.,80.)]})).unwrap();
        let id=shooter(&mut g,kind,480.,336.);g.units.last_mut().unwrap().grenades=1;
        g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));g.step(0.05);
        assert_eq!(g.walls[0].hp,30.,"{kind}: verified50-damage grenade contact");
        assert_eq!(unit(&g,id).grenades,0);assert!(g.events.iter().any(|e|e["weapon"]=="grenade"));
        for _ in 0..50 {g.step(0.05);}
        assert_eq!(g.walls[0].hp,30.,"No ammunition means no additional wall hits");
        assert_eq!(unit(&g,id).order,"guard");
    }
}

#[test]
fn explosive_families_and_both_human_teams_break_stone_with_primary_contact() {
    for kind in ["tough","light","medium","heavy","launcher","cannon","missile","howitzer"] {
        for team in [0,1] {
            let mut g=field();g.multiplayer=true;g.import_walls(&json!({"walls":[wall(7,512.,320.,5.)]})).unwrap();
            g.spawn("grunt",0,1680.,1056.,82,None);g.units.last_mut().unwrap().cooldown=10_000.;
            g.units.last_mut().unwrap().hold_position=true;
            let id=shooter(&mut g,kind,480.,336.);g.units.last_mut().unwrap().team=team;
            g.command(&json!({"team":team,"ids":[id],"kind":"attackWall","target":7}));g.step(0.05);
            assert!(!g.walls[0].alive(),"{kind}/team{team}");
            assert!(g.events.iter().any(|e|e["kind"]=="shot"&&e["weapon"]=="primary"));
            assert_eq!(unit(&g,id).grenades,0);
        }
    }
}

#[test]
fn an_unreachable_cliff_blocks_actual_wall_fire_instead_of_becoming_destructible() {
    let mut g=field();g.import_walls(&json!({"walls":[wall(7,640.,320.,5.)]})).unwrap();
    for y in 0..g.map.height {g.map.tiles[y*g.map.width+17]=2;}
    let id=shooter(&mut g,"light",480.,336.);g.command(&json!({"ids":[id],"kind":"attackWall","target":7}));
    for _ in 0..20 {g.step(0.05);assert!(!g.events.iter().any(|e|e["kind"]=="shot"));}
    assert_eq!(g.walls[0].hp,5.);assert_eq!(unit(&g,id).x,480.);
    assert!(g.map.tiles.iter().enumerate().filter(|(i,_)|i%g.map.width==17).all(|(_,tile)|*tile==2));
}
