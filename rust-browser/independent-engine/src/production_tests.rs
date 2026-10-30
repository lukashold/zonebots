use super::*;

// Synthetic layouts keep the original campaign's geometry and resources out of
// public tests. Both teams have a distant robot so a production tick cannot win.
fn original_factory(kind: &str, team: i32, stars: usize, product: Option<&str>) -> Game {
    let mut game = Game::new(0, 1, 177);
    let mut map = game.map.clone();
    map.original = true;
    map.world = 0;
    map.tiles.fill(0);
    map.territory.fill(1);
    let sectors = vec![
        Sector { id: 0, x: 320., y: 320., owner: team, home: false, counts_for_production: true },
        Sector { id: 1, x: 150., y: 900., owner: 0, home: true, counts_for_production: true },
        Sector { id: 2, x: 1500., y: 1000., owner: 1, home: true, counts_for_production: true },
    ];
    let mut factory = game.buildings[0].clone();
    factory.id = 0;
    factory.kind = kind.into();
    factory.team = team;
    factory.sector = 0;
    factory.stars = stars;
    factory.x = 300.;
    factory.y = 200.;
    factory.w = 96.;
    factory.h = 80.;
    factory.hp = 1000.;
    factory.max_hp = 1000.;
    factory.product = product.map(str::to_string);
    factory.progress = 0.;
    factory.remaining_work = None;
    factory.gate = None;
    game.apply_original(&json!({
        "map": map, "sectors": sectors, "buildings": [factory], "crates": [],
        "spawns": [
            {"type":"grunt","x":150,"y":900,"team":0},
            {"type":"grunt","x":1500,"y":1000,"team":1}
        ]
    })).unwrap();
    game.ai_teams.clear();
    for unit in &mut game.units { unit.hold_position = true; }
    game
}

fn capture_with_player(game: &mut Game) {
    let robot = game.units.iter_mut().find(|u| u.team == 0).unwrap();
    robot.x = game.sectors[0].x;
    robot.y = game.sectors[0].y;
    game.step(0.1);
    assert_eq!(game.sectors[0].owner, 0);
    assert_eq!(game.buildings[0].team, 0);
}

#[test]
fn owned_robot_factories_and_forts_start_with_robots_at_every_technology_level() {
    for kind in ["robot", "fort"] {
        for stars in 1..=5 {
            for team in [0, 1] {
                let game = original_factory(kind, team, stars, None);
                assert_eq!(game.buildings[0].product.as_deref(), Some("grunt"), "{kind}, {stars} stars, team {team}");
                assert!(game.remaining_seconds(&game.buildings[0]).unwrap() > 0);
            }
        }
    }
}

#[test]
fn capturing_an_idle_robot_factory_starts_production_in_the_same_tick() {
    let mut game = original_factory("robot", -1, 1, None);
    assert!(game.buildings[0].product.is_none());
    capture_with_player(&mut game);
    let factory = &game.buildings[0];
    assert_eq!(factory.product.as_deref(), Some("grunt"));
    assert!(factory.progress > 0.);
    assert!(factory.remaining_work.unwrap() < original_stats::workload("grunt") as f64);
    assert!(game.remaining_seconds(factory).unwrap() > 0);
    assert!(game.events.iter().any(|e| e["kind"] == "capture"));
}

#[test]
fn capturing_active_enemy_production_keeps_the_robot_type_and_existing_work() {
    let mut game = original_factory("robot", 1, 5, Some("psycho"));
    game.buildings[0].progress = 0.72;
    game.buildings[0].remaining_work = Some(500.);
    capture_with_player(&mut game);
    let factory = &game.buildings[0];
    assert_eq!(factory.product.as_deref(), Some("psycho"));
    assert!(factory.remaining_work.unwrap() < 500.);
    assert!(factory.remaining_work.unwrap() > 400., "Capture must not reset or finish existing work");
    assert!(factory.progress >= 0.72);
}

#[test]
fn robot_production_repeats_after_each_completed_group() {
    for kind in ["grunt", "psycho"] {
        let mut game = original_factory("robot", 0, 5, Some(kind));
        for _ in 0..2 {
            let before = game.units.iter().filter(|u| u.team == 0 && u.kind == kind).count();
            game.buildings[0].remaining_work = Some(1.);
            game.step(0.1);
            assert_eq!(game.units.iter().filter(|u| u.team == 0 && u.kind == kind).count(), before + stats(kind).members);
            assert_eq!(game.buildings[0].product.as_deref(), Some(kind));
            assert_eq!(game.buildings[0].remaining_work, Some(original_stats::workload(kind) as f64));
            assert!(game.events.iter().any(|e| e["kind"] == "ready" && e["type"] == kind));
            game.step(0.1);
            assert!(game.buildings[0].progress > 0., "Next group must start without another command");
        }
    }
}

#[test]
fn clearing_production_returns_to_robots_without_restarting_an_existing_default() {
    let mut game = original_factory("robot", 0, 5, Some("psycho"));
    game.buildings[0].progress = 0.4;
    game.buildings[0].remaining_work = Some(1234.);
    game.produce(0, None);
    assert_eq!(game.buildings[0].product.as_deref(), Some("grunt"));
    assert_eq!(game.buildings[0].progress, 0.);
    assert_eq!(game.buildings[0].remaining_work, None);
    game.buildings[0].progress = 0.55;
    game.buildings[0].remaining_work = Some(4321.);
    game.produce(0, None);
    assert_eq!(game.buildings[0].product.as_deref(), Some("grunt"));
    assert_eq!(game.buildings[0].progress, 0.55);
    assert_eq!(game.buildings[0].remaining_work, Some(4321.));
    game.produce(0, Some("truck"));
    assert_eq!(game.buildings[0].remaining_work, Some(4321.), "Invalid products must remain rejected");
}

#[test]
fn neutral_destroyed_radar_and_repair_buildings_do_not_start_production() {
    let mut neutral = original_factory("robot", -1, 5, None);
    neutral.step(0.1);
    assert!(neutral.buildings[0].product.is_none());
    assert!(neutral.buildings[0].remaining_work.is_none());
    for kind in ["radar", "repair"] {
        let mut game = original_factory(kind, 0, 0, None);
        game.step(0.1);
        assert!(game.buildings[0].product.is_none(), "{kind}");
        assert!(game.buildings[0].remaining_work.is_none(), "{kind}");
    }
    let mut destroyed = original_factory("robot", -1, 5, None);
    destroyed.buildings[0].team = 0;
    destroyed.sectors[0].owner = 0;
    destroyed.buildings[0].hp = 0.;
    destroyed.step(0.1);
    assert!(destroyed.buildings[0].product.is_none());
    assert!(destroyed.buildings[0].remaining_work.is_none());
}

#[test]
fn owned_vehicle_factories_keep_their_allowed_hardware_production() {
    for stars in 1..=5 {
        let mut game = original_factory("vehicle", 0, stars, None);
        let product = game.buildings[0].product.clone().unwrap();
        assert!(stats(&product).cls > 0);
        assert!(game.products(&game.buildings[0]).contains(&product.as_str()));
        game.step(0.1);
        assert_eq!(game.buildings[0].product.as_deref(), Some(product.as_str()));
        assert!(game.buildings[0].progress > 0.);
    }
}

#[test]
fn severely_damaged_factory_keeps_its_active_order_but_respects_zero_rate() {
    let mut game = original_factory("robot", 0, 5, None);
    game.buildings[0].hp = 1.;
    game.step(0.1);
    assert_eq!(game.buildings[0].product.as_deref(), Some("grunt"));
    assert_eq!(game.buildings[0].remaining_work, Some(original_stats::workload("grunt") as f64));
    assert_eq!(game.buildings[0].progress, 0.);
    assert_eq!(game.remaining_seconds(&game.buildings[0]), None);
}
