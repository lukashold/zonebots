use super::*;

fn initialize() -> Value {
    request(json!({"op":"init","mission":0,"difficulty":0,"seed":529,"multiplayer":true,"aiTeams":[0,1]}))
}

#[test]
fn multiplayer_initialization_is_fair_and_never_runs_strategic_ai() {
    let state = initialize();
    assert_eq!(state["multiplayer"], true);
    assert_eq!(state["difficulty"], 1);
    GAME.with(|cell| {
        let state = cell.borrow(); let game = state.as_ref().unwrap();
        assert!(game.ai_teams.is_empty(), "An explicit aiTeams cannot turn human opponents back into AI");
        for kind in TYPES {
            let count = |team| game.units.iter().filter(|u| u.team == team && u.kind == kind).count();
            assert_eq!(count(0), count(1), "Easy must not add an extra human squad: {kind}");
        }
    });
}

#[test]
fn both_players_can_command_only_their_own_units() {
    let initial = initialize();
    let units = initial["units"].as_array().unwrap();
    let red = units.iter().find(|u| u["team"] == 0 && u["type"] == "grunt").unwrap()["id"].clone();
    let blue = units.iter().find(|u| u["team"] == 1 && u["type"] == "grunt").unwrap()["id"].clone();
    let after = request(json!({"op":"command","team":1,"ids":[red,blue],"kind":"move","x":900,"y":500}));
    let result = |id: &Value| after["units"].as_array().unwrap().iter().find(|u| u["id"] == *id).unwrap();
    assert_eq!(result(&red)["order"], "guard");
    assert_eq!(result(&blue)["order"], "move");
    let after = request(json!({"op":"command","team":0,"ids":[red,blue],"kind":"stop"}));
    let result = |id: &Value| after["units"].as_array().unwrap().iter().find(|u| u["id"] == *id).unwrap();
    assert_eq!(result(&blue)["order"], "move");
    assert_eq!(result(&red)["order"], "guard");
}

#[test]
fn invalid_team_values_do_not_wrap_into_a_human_or_neutral_team() {
    initialize();
    for team in [json!(-1), json!(2), json!(4294967296u64), json!("1"), json!(0.5)] {
        for op in ["command", "produce", "previewGun", "placeGun"] {
            let result = request(json!({"op":op,"team":team,"id":0,"ids":[1],"kind":"stop","x":320,"y":320}));
            assert_eq!(result["error"], "Invalid player team", "{op}: {team}");
        }
    }
}

#[test]
fn blue_player_production_cannot_change_the_red_factory() {
    initialize();
    let (red, blue) = GAME.with(|cell| {
        let mut state = cell.borrow_mut(); let game = state.as_mut().unwrap();
        for b in &mut game.buildings { b.stars = 5; }
        (game.buildings.iter().find(|b| b.team == 0 && b.kind == "fort").unwrap().id,
         game.buildings.iter().find(|b| b.team == 1 && b.kind == "fort").unwrap().id)
    });
    request(json!({"op":"produce","team":1,"id":red,"product":"psycho"}));
    let state = request(json!({"op":"produce","team":1,"id":blue,"product":"psycho"}));
    let b = |id| state["buildings"].as_array().unwrap().iter().find(|b| b["id"] == id).unwrap();
    assert_eq!(b(red)["product"], "grunt");
    assert_eq!(b(blue)["product"], "psycho");
    let state = request(json!({"op":"produce","team":0,"id":blue,"product":"grunt"}));
    assert_eq!(state["buildings"].as_array().unwrap().iter().find(|b| b["id"] == blue).unwrap()["product"], "psycho");
}

#[test]
fn blue_gun_preview_and_placement_validate_the_owning_factory() {
    initialize();
    GAME.with(|cell| {
        let mut state = cell.borrow_mut(); let game = state.as_mut().unwrap();
        game.map.tiles.fill(0); game.map.territory.fill(0); game.map.original = false;
        game.sectors.truncate(1); game.sectors[0].owner = 1; game.sectors[0].x = 1600.; game.sectors[0].y = 1200.;
        game.buildings.truncate(1);
        let b = &mut game.buildings[0]; b.id = 0; b.team = 1; b.sector = 0; b.x = 64.; b.y = 64.; b.w = 64.; b.h = 64.; b.ready_guns = vec!["gatling".into()];
        game.units.clear(); game.crates.clear(); game.objects.clear();
    });
    let wrong = request(json!({"op":"previewGun","team":0,"id":0,"x":320,"y":320}));
    assert_eq!(wrong["valid"], false);
    let right = request(json!({"op":"previewGun","team":1,"id":0,"x":320,"y":320}));
    assert_eq!(right["valid"], true);
    let wrong = request(json!({"op":"placeGun","team":0,"id":0,"x":320,"y":320}));
    assert_eq!(wrong["placementResult"]["valid"], false);
    assert!(wrong["units"].as_array().unwrap().is_empty());
    let right = request(json!({"op":"placeGun","team":1,"id":0,"x":320,"y":320}));
    assert_eq!(right["placementResult"]["valid"], true);
    assert_eq!(right["units"][0]["team"], 1);
    assert_eq!(right["units"][0]["type"], "gatling");
    assert!(right["buildings"][0]["readyGuns"].as_array().unwrap().is_empty());
}

#[test]
fn compact_snapshots_omit_static_arrays_until_a_map_revision_changes() {
    let initial = request(json!({"op":"init","seed":529,"compact":true,"multiplayer":true}));
    for field in ["map", "objects", "catalog"] { assert!(initial.get(field).is_some(), "Initialization must include {field}"); }
    let known = json!({"op":"snapshot","compact":true,"knownMapSeed":initial["mapSeed"],"knownMapRevision":initial["mapRevision"]});
    let compact = request(known.clone());
    for field in ["map", "objects", "catalog"] { assert!(compact.get(field).is_none(), "Known snapshot must omit {field}"); }
    for field in ["units", "buildings", "effects", "events", "sectors", "crates", "productionSeconds", "buildMenus"] { assert!(compact.get(field).is_some(), "Dynamic state needs {field}"); }
    GAME.with(|cell| { let mut state = cell.borrow_mut(); let game = state.as_mut().unwrap(); game.map.revision += 1; game.map.tiles[0] = 3; });
    let revised = request(known.clone());
    assert_eq!(revised["map"]["revision"], 1); assert_eq!(revised["map"]["tiles"][0], 3);
    assert!(revised.get("catalog").is_none());
    let mut force = known; force["includeStatic"] = json!(true);
    let forced = request(force);
    for field in ["map", "objects", "catalog"] { assert!(forced.get(field).is_some()); }
    let full = request(json!({"op":"snapshot"}));
    for field in ["map", "objects", "catalog"] { assert!(full.get(field).is_some(), "Legacy full state needs {field}"); }
}

#[test]
fn snapshot_read_does_not_advance_the_game_or_consume_events() {
    initialize();
    GAME.with(|cell| cell.borrow_mut().as_mut().unwrap().events.push(json!({"kind":"capture","team":1,"sector":4})));
    let before = request(json!({"op":"snapshot"}));
    let after = request(json!({"op":"snapshot"}));
    assert_eq!(before, after);
    assert_eq!(after["time"], 0.);
    assert_eq!(after["events"][0]["kind"], "capture");
}

#[test]
fn compact_serialization_removes_the_large_immutable_map_cost() {
    let mut game = Game::new(0, 1, 529);
    let map = &mut game.map; map.width = 128; map.height = 128; map.original = true;
    map.tiles = vec![0; 16384]; map.territory = vec![0; 16384]; map.bridge_base = vec![-1; 16384];
    map.original_tiles = vec![12345; 16384]; map.terrain_codes = vec![0; 16384];
    let full = game_snapshot(&game, &json!({"op":"snapshot"}));
    let compact = game_snapshot(&game, &json!({"op":"snapshot","compact":true,"knownMapSeed":529,"knownMapRevision":0}));
    let full_bytes = serde_json::to_vec(&full).unwrap().len(); let compact_bytes = serde_json::to_vec(&compact).unwrap().len();
    assert!(full_bytes > 200_000); assert!(compact_bytes * 3 < full_bytes, "full={full_bytes}, compact={compact_bytes}");
    assert_eq!(full["units"], compact["units"]); assert_eq!(full["sectors"], compact["sectors"]);
}

#[test]
fn original_multiplayer_damage_is_equal_for_both_human_teams() {
    let game = Game::new(0, 1, 529);
    let prototype = game.units.iter().find(|u| u.kind == "grunt" && u.team == 0).unwrap().clone();
    for difficulty in [0, 1, 2] {
        let mut red = prototype.clone(); let mut blue = prototype.clone(); blue.team = 1;
        let attack = original_combat::Attack { target: 0, damage: 10, aim: 255, facing: 0, bullet: false };
        original_combat::apply_attack_with_mode(&mut red, attack, difficulty, true, || 0);
        original_combat::apply_attack_with_mode(&mut blue, attack, difficulty, true, || 0);
        assert_eq!(red.hp, blue.hp); assert_eq!(red.hp, prototype.hp - 10.);
    }
}
