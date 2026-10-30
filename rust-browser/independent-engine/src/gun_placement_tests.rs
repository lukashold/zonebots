use super::*;

// A synthetic clear map exercises placement without publishing original assets
// or campaign geometry. Two distant armies prevent production-only victory.
fn fixture(team: i32) -> Game {
    let mut game = Game::new(0, 1, 511);
    let mut map = game.map.clone();
    map.width = 20;
    map.height = 20;
    map.original = true;
    map.tiles = vec![0; 400];
    map.original_tiles = vec![0; 400];
    map.terrain_codes = vec![0; 400];
    map.territory = (0..400).map(|i| if i % 20 >= 16 { 1 } else { 0 }).collect();
    map.bridge_base = vec![-1; 400];
    map.bridge_hp.clear();
    let mut factory = game.buildings[0].clone();
    factory.id = 0;
    factory.kind = "robot".into();
    factory.team = team;
    factory.sector = 0;
    factory.stars = 5;
    factory.x = 96.; factory.y = 64.; factory.w = 64.; factory.h = 64.;
    factory.hp = 1000.; factory.max_hp = 1000.;
    factory.product = Some("gatling".into());
    factory.progress = 0.; factory.remaining_work = None;
    factory.ready_guns.clear(); factory.raw_base = None;
    game.apply_original(&json!({
        "map":map,
        "sectors":[
            {"id":0,"x":80,"y":480,"owner":team,"home":false,"countsForProduction":true},
            {"id":1,"x":600,"y":600,"owner":1,"home":true,"countsForProduction":true}
        ],
        "buildings":[factory],"crates":[],"originalObjects":[],
        "spawns":[
            {"type":"grunt","team":0,"x":32,"y":560,"squad":1},
            {"type":"grunt","team":1,"x":600,"y":600,"squad":2}
        ]
    })).unwrap();
    game.ai_teams.clear();
    for unit in &mut game.units { unit.hold_position = true; }
    game
}

fn ready(team: i32, kind: &str) -> Game {
    let mut game = fixture(team);
    game.buildings[0].ready_guns.push(kind.into());
    game
}

fn gun_count(game: &Game) -> usize {
    game.units.iter().filter(|u| stats(&u.kind).cls == 2).count()
}

#[test]
fn completed_guns_wait_in_four_slots_instead_of_appearing_at_the_factory_exit() {
    let mut game = fixture(0);
    let initial_units = game.units.len();
    for n in 1..=4 {
        game.events.clear();
        game.buildings[0].remaining_work = Some(1.);
        game.step(0.1);
        assert_eq!(game.buildings[0].ready_guns.len(), n);
        assert_eq!(game.units.len(), initial_units);
        assert_eq!(gun_count(&game), 0);
        assert_eq!(game.events.iter().filter(|e| e["kind"] == "ready" && e["type"] == "gatling").count(), 1);
    }
    game.buildings[0].remaining_work = Some(999.);
    game.buildings[0].progress = 0.5;
    game.events.clear();
    game.step(0.1);
    assert_eq!(game.buildings[0].ready_guns.len(), 4);
    assert_eq!(game.buildings[0].remaining_work, Some(999.));
    assert_eq!(game.buildings[0].progress, 0.5);
    assert!(!game.events.iter().any(|e| e["kind"] == "ready"));

    // A full gun rack does not block a newly selected robot order or discard it.
    game.produce(0, Some("grunt"));
    game.buildings[0].remaining_work = Some(1.);
    game.step(0.1);
    assert_eq!(game.buildings[0].ready_guns.len(), 4);
    assert_eq!(game.units.len(), initial_units + stats("grunt").members);
}

#[test]
fn ground_preview_is_read_only_snaps_to_grid_and_consumes_exactly_one_selected_slot() {
    let mut game = ready(0, "gatling");
    game.buildings[0].ready_guns.push("howitzer".into());
    game.events.push(json!({"kind":"ready","team":0,"type":"gatling"}));
    let before = serde_json::to_value(&game).unwrap();
    let next_id = game.next_id;
    let preview = game.gun_placement_preview(0, 1, 333., 335., 0);
    assert!(preview.valid, "{}", preview.reason);
    assert_eq!(preview.kind.as_deref(), Some("howitzer"));
    assert_eq!((preview.x, preview.y, preview.width, preview.height), (320., 320., 64, 64));
    assert_eq!(preview.fort_id, None);
    assert_eq!(serde_json::to_value(&game).unwrap(), before);
    assert_eq!(game.next_id, next_id);

    let placed = game.place_gun_for_team(0, 1, 333., 335., 0);
    assert!(placed.valid, "{}", placed.reason);
    assert_eq!(game.buildings[0].ready_guns, vec!["gatling"]);
    let gun = game.units.last().unwrap();
    assert_eq!(gun.kind, "howitzer");
    assert_eq!((gun.x, gun.y, gun.team), (320., 320., 0));
    assert!(gun.crew.is_some());
    assert_eq!(gun.fort_id, None);
    assert_eq!(gun_count(&game), 1);
    assert!(game.events.iter().any(|e| e["kind"] == "placed" && e["type"] == "howitzer"));

    let before = serde_json::to_value(&game).unwrap();
    assert!(!game.place_gun_for_team(0, 0, 333., 335., 0).valid, "Occupied ground must be revalidated at commit time");
    assert_eq!(serde_json::to_value(&game).unwrap(), before);
}

#[test]
fn invalid_placement_preserves_stock_for_bounds_terrain_territory_and_obstacles() {
    let mut cases = Vec::new();
    let mut game = ready(0, "gatling");
    game.map.terrain_codes[10 * 20 + 10] = 7;
    cases.push(game);
    let mut game = ready(0, "gatling");
    game.map.territory[11 * 20 + 11] = 1;
    cases.push(game);
    let mut game = ready(0, "gatling");
    let mut building = game.buildings[0].clone();
    building.id = 1; building.x = 320.; building.y = 320.; building.ready_guns.clear();
    game.buildings.push(building);
    cases.push(game);
    let mut game = ready(0, "gatling");
    game.spawn("grunt", 0, 340., 340., 3, None);
    cases.push(game);
    let mut game = ready(0, "gatling");
    game.crates.push(Crate { x:340., y:340., taken:false });
    cases.push(game);
    let mut game = ready(0, "gatling");
    game.objects.push(json!({"x":340,"y":340,"type":3,"templateFlags":0}));
    cases.push(game);
    let mut game = ready(0, "gatling");
    game.sectors[0].x = 340.; game.sectors[0].y = 340.;
    cases.push(game);
    for mut game in cases {
        let before = serde_json::to_value(&game).unwrap();
        let result = game.place_gun_for_team(0, 0, 320., 320., 0);
        assert!(!result.valid);
        assert!(!result.reason.is_empty());
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
    }
    let mut game = ready(0, "gatling");
    for (x, y) in [(-1., 320.), (608., 320.), (f64::NAN, 320.), (320., f64::INFINITY)] {
        let result = game.place_gun_for_team(0, 0, x, y, 0);
        assert!(!result.valid);
        assert!(result.x.is_finite() && result.y.is_finite());
        assert_eq!(game.buildings[0].ready_guns.len(), 1);
    }
    assert!(!game.place_gun_for_team(0, usize::MAX, 320., 320., 0).valid);
}

#[test]
fn placement_frees_a_queue_slot_and_the_factory_keeps_producing_guns() {
    let mut game = ready(0, "gatling");
    game.buildings[0].remaining_work = Some(1.);
    assert!(game.place_gun_for_team(0, 0, 320., 320., 0).valid);
    assert!(game.buildings[0].ready_guns.is_empty());
    game.step(0.1);
    assert_eq!(game.buildings[0].ready_guns, vec!["gatling"]);
    assert_eq!(gun_count(&game), 1, "The next completed gun stays in stock until another placement");
    assert_eq!(game.buildings[0].product.as_deref(), Some("gatling"));
    assert!(game.remaining_seconds(&game.buildings[0]).unwrap() > 0);
}

#[test]
fn captured_gun_stock_follows_the_factory_and_is_not_discarded_by_product_changes() {
    let mut game = ready(1, "cannon");
    assert!(!game.gun_placement_preview(0, 0, 320., 320., 0).valid);
    let robot = game.units.iter_mut().find(|u| u.team == 0).unwrap();
    robot.x = game.sectors[0].x; robot.y = game.sectors[0].y;
    game.step(0.1);
    assert_eq!(game.buildings[0].team, 0);
    assert_eq!(game.buildings[0].ready_guns, vec!["cannon"]);
    game.produce(0, Some("grunt"));
    assert_eq!(game.buildings[0].ready_guns, vec!["cannon"]);
    assert!(game.place_gun_for_team(0, 0, 320., 320., 0).valid);
    assert_eq!(game.units.last().unwrap().team, 0);
}

fn with_fort(kind: &str) -> Game {
    let mut game = ready(0, kind);
    let mut fort = game.buildings[0].clone();
    fort.id = 1; fort.kind = "fort".into(); fort.ready_guns.clear();
    fort.x = 192.; fort.y = 128.; fort.w = 288.; fort.h = 384.;
    fort.raw_base = Some((96., 64.));
    game.buildings.push(fort);
    game
}

#[test]
fn original_fort_placement_uses_all_six_roof_sockets_and_rejects_occupied_mounts() {
    let native = [(17.,170.),(113.,1.),(113.,170.),(17.,1.),(113.,49.),(17.,49.)];
    for kind in ["cannon", "gatling", "howitzer", "missile"] {
        let game = with_fort(kind);
        for (x, y) in native {
            let (click_x, click_y) = ((96.+x)*2., (64.+y)*2.);
            let result = game.gun_placement_preview(0, 0, click_x, click_y, 0);
            assert!(result.valid, "{kind} at {x},{y}: {}", result.reason);
            let (dx, dy) = if kind == "cannon" { (0.,0.) } else { (-4.,-16.) };
            assert_eq!((result.x, result.y), (click_x+dx, click_y+dy));
            assert_eq!(result.fort_id, Some(1));
        }
    }
    let mut game = with_fort("cannon");
    let result = game.place_gun_for_team(0, 0, 226., 130., 0);
    assert!(result.valid);
    assert_eq!(game.units.last().unwrap().fort_id, Some(1));
    game.buildings[0].ready_guns.push("gatling".into());
    game.units.last_mut().unwrap().team = -1;
    game.units.last_mut().unwrap().crew = None;
    let occupied = game.gun_placement_preview(0, 0, 226., 130., 0);
    assert!(!occupied.valid, "Even an unmanned gun occupies its roof socket");
    assert_eq!(game.buildings[0].ready_guns.len(), 1);
}

#[test]
fn ai_deploys_its_ready_guns_but_does_not_place_the_players_stock() {
    let mut enemy = ready(1, "gatling");
    enemy.place_ai_guns(1);
    assert!(enemy.buildings[0].ready_guns.is_empty());
    assert_eq!(gun_count(&enemy), 1);
    assert_eq!(enemy.units.last().unwrap().team, 1);
    let mut player = ready(0, "gatling");
    player.place_ai_guns(1);
    assert_eq!(player.buildings[0].ready_guns.len(), 1);
    assert_eq!(gun_count(&player), 0);
}

#[test]
fn top_roof_clicks_work_above_the_imported_forts_physical_building_box() {
    for kind in ["cannon", "gatling", "howitzer", "missile"] {
        let mut game = with_fort(kind);
        // Real imports use physicalY=(rawBaseY+16)*2, while the upper gun
        // control anchors are only rawBaseY+1 native pixels high.
        game.buildings[1].y = (64.+16.)*2.;
        for native_x in [17., 113.] {
            let (x, y) = ((96.+native_x)*2., (64.+1.)*2.);
            assert!(y < game.buildings[1].y);
            let p = game.gun_placement_preview(0, 0, x, y, 0);
            assert!(p.valid, "{kind}: {}", p.reason);
            assert_eq!(p.fort_id, Some(1));
            let (dx, dy) = if kind == "cannon" { (0., 0.) } else { (-4., -16.) };
            assert_eq!((p.x, p.y), (x+dx, y+dy));
        }
    }
}

#[test]
fn public_preview_and_place_requests_ignore_a_forged_team_and_preview_preserves_events() {
    let mut game = ready(1, "cannon");
    game.events.push(json!({"kind":"ready","type":"cannon","team":1}));
    GAME.with(|cell| *cell.borrow_mut() = Some(game));
    let before = GAME.with(|cell| serde_json::to_value(cell.borrow().as_ref().unwrap()).unwrap());
    let result = request(json!({"op":"previewGun","id":0,"slot":0,"x":320,"y":320,"team":1}));
    assert_eq!(result["valid"], false);
    for key in ["units", "buildings", "catalog", "events", "time"] { assert!(result.get(key).is_none()); }
    let after = GAME.with(|cell| serde_json::to_value(cell.borrow().as_ref().unwrap()).unwrap());
    assert_eq!(before, after, "Read-only polling must not erase a production event");
    let placed = request(json!({"op":"placeGun","id":0,"slot":0,"x":320,"y":320,"team":1}));
    assert_eq!(placed["placementResult"]["valid"], false);
    assert!(placed.get("error").is_none());
    assert_eq!(placed["buildings"][0]["readyGuns"], json!(["cannon"]));
    assert_eq!(placed["units"].as_array().unwrap().len(), 2);
}
