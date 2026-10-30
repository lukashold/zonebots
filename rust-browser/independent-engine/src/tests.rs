use super::*;
#[test]
fn movement_snapshot_exports_the_actual_route() {
    let mut g = game();
    let id = g.units.iter().find(|u| u.team == 0 && u.kind == "grunt").unwrap().id;
    g.command(&json!({"ids":[id],"kind":"move","x":400,"y":1100}));
    let unit = g.units.iter().find(|u|u.id==id).unwrap();
    assert!(!unit.path.is_empty());
    let snapshot = serde_json::to_value(unit).unwrap();
    assert_eq!(snapshot["path"].as_array().unwrap().len(),unit.path.len());
    assert_eq!(snapshot["path"][0],json!(unit.path[0]));
}
fn game() -> Game {
    let mut g = Game::new(2, 1, 7291);
    g.ai_teams.clear();
    g
}
#[test]
fn flag_transfers_factory_with_progress() {
    let mut g = game();
    let b = g.buildings.iter().position(|b| b.sector == 0).unwrap();
    g.buildings[b].product = Some("grunt".into());
    g.buildings[b].progress = 0.72;
    let s = g.sectors[0].clone();
    let i = g
        .units
        .iter()
        .position(|u| u.team == 0 && u.kind == "grunt")
        .unwrap();
    g.units[i].x = s.x;
    g.units[i].y = s.y;
    g.step(0.05);
    assert_eq!(g.sectors[0].owner, 0);
    assert_eq!(g.buildings[b].team, 0);
    assert!(g.buildings[b].progress >= 0.72);
}
#[test]
fn damage_slows_factory() {
    let mut a = game();
    let mut b = game();
    let id = a.buildings.iter().position(|b| b.team == 0).unwrap();
    a.buildings[id].progress = 0.;
    b.buildings[id].progress = 0.;
    b.buildings[id].hp *= 0.5;
    a.step(0.1);
    b.step(0.1);
    assert!((a.buildings[id].progress - b.buildings[id].progress * 2.).abs() < 1e-9);
}
#[test]
fn more_territory_speeds_production() {
    let mut a = game();
    let mut b = game();
    let id = a.buildings.iter().position(|b| b.team == 0).unwrap();
    b.sectors[0].owner = 0;
    a.step(0.1);
    b.step(0.1);
    assert!(b.buildings[id].progress > a.buildings[id].progress);
}
#[test]
fn crew_entry_and_exit_conserves_robot() {
    let mut g = game();
    g.spawn("jeep", -1, 640., 640., 0, None);
    let car = g.units.last().unwrap().id;
    let i = g
        .units
        .iter()
        .position(|u| u.team == 0 && u.kind == "grunt")
        .unwrap();
    let robot = g.units[i].id;
    g.units[i].x = 640.;
    g.units[i].y = 650.;
    g.units[i].hp = 27.;
    let before = g
        .units
        .iter()
        .filter(|u| u.team == 0 && stats(&u.kind).cls == 0)
        .count();
    g.command(&json!({"ids":[robot],"kind":"enter","target":car}));
    g.step(0.05);
    assert_eq!(g.units.iter().find(|u| u.id == car).unwrap().team, 0);
    assert_eq!(
        g.units
            .iter()
            .filter(|u| u.team == 0 && stats(&u.kind).cls == 0)
            .count(),
        before - 1
    );
    g.command(&json!({"ids":[car],"kind":"unload"}));
    assert_eq!(
        g.units
            .iter()
            .filter(|u| u.team == 0 && stats(&u.kind).cls == 0)
            .count(),
        before
    );
    assert_eq!(g.units.last().unwrap().hp, 27.);
}
#[test]
fn movement_rules_distinguish_water_and_rock() {
    let mut g = game();
    let j = 15 * g.map.width + 18;
    g.map.tiles[j] = 3;
    assert!(!g.passable(18, 15, false));
    assert!(g.passable(18, 15, true));
    g.map.tiles[j] = 2;
    assert!(!g.passable(18, 15, true));
}
#[test]
fn moving_robot_does_not_return_fire() {
    let mut g = game();
    g.spawn("grunt", 0, 640., 640., 100, None);
    let a = g.units.len() - 1;
    g.spawn("grunt", 1, 700., 640., 101, None);
    let b = g.units.len() - 1;
    g.units[b].cooldown = 100.;
    g.units[a].order = "move".into();
    g.units[a].path.push_back((640., 700.));
    g.step(0.05);
    assert_eq!(g.units[b].hp, stats("grunt").hp);
    assert!(g.units[a].y > 640.);
}
#[test]
fn fort_infiltration_wins() {
    let mut g = game();
    let b = g
        .buildings
        .iter()
        .find(|b| b.kind == "fort" && b.team == 1)
        .unwrap()
        .clone();
    g.spawn("grunt", 0, b.x + b.w / 2., b.y + b.h + 5., 100, None);
    g.step(0.05);
    assert_eq!(g.winner, Some(0));
}
#[test]
fn bridge_destruction_and_crane_repair() {
    let mut g = game();
    let Some((j, id)) = g
        .map
        .bridge_base
        .iter()
        .enumerate()
        .find(|(_, id)| **id >= 0)
        .map(|(j, id)| (j, *id as usize))
    else {
        panic!("own map needs bridge")
    };
    let (x, y) = (
        (j % g.map.width) as f64 * 32. + 16.,
        (j / g.map.width) as f64 * 32. + 16.,
    );
    g.map.bridge_hp[id] = 1.;
    g.spawn("laser", 0, x - 70., y, 100, None);
    let attacker = g.units.last().unwrap().id;
    g.spawn("jeep", 1, x, y, 101, None);
    let victim = g.units.last().unwrap().id;
    g.command(&json!({"ids":[attacker],"kind":"attackBridge","x":x,"y":y}));
    g.step(0.1);
    assert_eq!(g.map.tiles[j], 3);
    assert!(!g.units.iter().any(|u| u.id == victim));
    g.spawn("crane", 0, x - 60., y, 102, None);
    let crane = g.units.last().unwrap().id;
    g.command(&json!({"ids":[crane],"kind":"repairBridge","x":x,"y":y}));
    for _ in 0..100 {
        g.step(0.1);
    }
    assert_eq!(g.map.tiles[j], 5);
}
