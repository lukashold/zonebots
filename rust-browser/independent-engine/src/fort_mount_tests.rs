use super::*;

fn original_game(spawns: Value) -> Game {
    let mut game = Game::new(0, 1, 337);
    game.map.original = true;
    // A compact synthetic input exercises import metadata without embedding an
    // original campaign's map or placements in the public test source.
    let mut fort = game.buildings.iter().find(|b| b.kind == "fort" && b.team == 0).unwrap().clone();
    fort.id = 0;
    let mut factory = game.buildings.iter().find(|b| b.kind == "robot").unwrap().clone();
    factory.id = 1;
    let payload = json!({"map":game.map,"sectors":game.sectors,
        "buildings":[fort,factory],"crates":[],"spawns":spawns});
    game.apply_original(&payload).unwrap();
    game.ai_teams.clear();
    game
}

#[test]
fn original_fort_socket_import_accepts_each_gun_class() {
    let spawns = json!([
        {"type":"gatling","x":200,"y":200,"team":0,"fortId":0},
        {"type":"cannon","x":240,"y":200,"team":0,"fortId":0},
        {"type":"howitzer","x":280,"y":200,"team":0,"fortId":0},
        {"type":"missile","x":320,"y":200,"team":0,"fortId":0}
    ]);
    let game = original_game(spawns);
    assert!(game.units.iter().all(|u| u.fort_id == Some(0)));
    let exported = serde_json::to_value(&game).unwrap();
    assert_eq!(exported["units"][0]["fortId"], 0);
}

#[test]
fn ground_and_new_guns_do_not_attach_by_owner_or_proximity() {
    let mut game = original_game(json!([
        {"type":"gatling","x":200,"y":200,"team":-1},
        {"type":"cannon","x":200,"y":200,"team":0},
        {"type":"grunt","x":200,"y":200,"team":0,"fortId":0},
        {"type":"jeep","x":200,"y":200,"team":0,"fortId":0},
        {"type":"howitzer","x":200,"y":200,"team":0,"fortId":1},
        {"type":"missile","x":200,"y":200,"team":0,"fortId":999}
    ]));
    assert!(game.units.iter().all(|u| u.fort_id.is_none()));
    game.spawn_group("cannon", 0, 200., 200.);
    assert!(game.units.last().unwrap().fort_id.is_none());
}

#[test]
fn fort_attachment_survives_unload_and_recrewing() {
    let mut game = original_game(json!([
        {"type":"gatling","x":200,"y":200,"team":0,"fortId":0},
        {"type":"grunt","x":1400,"y":1000,"team":1}
    ]));
    let gun = game.units[0].id;
    game.command(&json!({"kind":"unload","ids":[gun]}));
    assert_eq!(game.units[0].fort_id, Some(0));
    assert_eq!(game.units[0].team, -1);
    let driver = game.units.last_mut().unwrap();
    assert!(driver.fort_id.is_none());
    driver.x = 200.;driver.y = 200.;let driver_id = driver.id;
    game.command(&json!({"kind":"enter","ids":[driver_id],"target":gun}));
    game.step(0.1);
    let hardware = game.units.iter().find(|u| u.id == gun).unwrap();
    assert_eq!(hardware.team, 0);
    assert!(hardware.crew.is_some());
    assert_eq!(hardware.fort_id, Some(0));
}
