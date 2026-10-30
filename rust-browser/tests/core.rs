use serde_json::json;
use zonebots_rust::{model::*, path};
fn fixture() -> Game {
    let b = |id, team, sector, x, kind: &str| json!({"id":id,"kind":kind,"team":team,"sector":sector,"tx":x,"ty":1,"w":2,"h":2,"door":{"tx":x,"ty":4},"cx":x as f64*32.+32.,"cy":64.,"hp":1000,"maxhp":1000,"tech":4});
    let defs = json!({"grunt":{"cls":"robot","tech":1,"time":2,"n":2,"hp":40,"sp":64,"range":110,"dmg":8,"rl":0.3,"w":"bullet","r":7},"jeep":{"cls":"vehicle","tech":1,"time":3,"n":1,"hp":120,"sp":100,"range":120,"dmg":6,"rl":0.5,"w":"bullet","r":12},"apc":{"cls":"vehicle","tech":2,"time":3,"n":1,"hp":300,"sp":80,"range":100,"dmg":3,"rl":0.4,"w":"bullet","r":14,"cargo":6},"crane":{"cls":"vehicle","tech":1,"time":3,"n":1,"hp":200,"sp":60,"r":14,"crane":true}});
    Game::init(&json!({"map":{"W":20,"H":20,"tiles":vec![0;400],"block":vec![0;400],"rockHp":vec![0;400],"bridgeAt":vec![-1;400],"buildings":[b(0,0,0,1,"fort"),b(1,1,1,16,"fort"),b(2,-1,2,8,"robot"),b(3,-1,2,10,"vehicle")],"sectors":[{"id":0,"owner":0},{"id":1,"owner":1},{"id":2,"owner":-1,"flag":{"x":320,"y":320}}],"spawns":[{"t":"grunt","team":0,"tx":2,"ty":5},{"t":"grunt","team":1,"tx":17,"ty":15}]},"defs":defs,"unitOrder":["grunt","jeep","apc","crane"],"lv":{"seed":42,"aiRate":1},"aiTeams":[]})).unwrap()
}
#[test]
fn original_factory_requirements_override_generic_tech() {
    let mut g = fixture();
    g.defs.get_mut("grunt").unwrap().build_levels =
        [("robot".into(), 3), ("fort".into(), 2), ("vehicle".into(), 0)].into();
    g.map.buildings[2].team = 0;
    g.map.buildings[2].tech = 2;
    g.map.buildings[2].prod = None;
    g.sync_controls(&json!({"buildings":[{"id":2,"prod":"grunt"}]}));
    assert!(g.map.buildings[2].prod.is_none());
    g.map.buildings[2].tech = 3;
    g.sync_controls(&json!({"buildings":[{"id":2,"prod":"grunt"}]}));
    assert_eq!(g.map.buildings[2].prod.as_deref(), Some("grunt"));
    g.map.buildings[3].team = 0;
    g.map.buildings[3].prod = None;
    g.sync_controls(&json!({"buildings":[{"id":3,"prod":"grunt"}]}));
    assert!(g.map.buildings[3].prod.is_none());
}
#[test]
fn capture_transfers_all_sector_buildings() {
    let mut g = fixture();
    g.units[0].x = 320.;
    g.units[0].y = 320.;
    g.step(0.04);
    assert_eq!(g.map.sectors[2].owner, 0);
    assert_eq!(g.map.buildings[2].team, 0);
    assert_eq!(g.map.buildings[3].team, 0);
}
#[test]
fn deterministic_simulation() {
    let mut a = fixture();
    let mut b = fixture();
    for _ in 0..100 {
        a.step(0.04);
        b.step(0.04);
    }
    assert_eq!(a.snapshot(), b.snapshot());
}
#[test]
fn wall_is_not_crossed() {
    let mut g = fixture();
    for y in 0..20 {
        g.map.block[y * 20 + 10] = 1;
    }
    assert!(path::find(
        &g.map,
        Point { x: 80., y: 100. },
        Point { x: 500., y: 100. }
    )
    .is_none());
    g.map.block[10 * 20 + 10] = 0;
    assert!(path::find(
        &g.map,
        Point { x: 80., y: 100. },
        Point { x: 500., y: 100. }
    )
    .is_some());
}
#[test]
fn robot_crews_neutral_vehicle() {
    let mut g = fixture();
    let u = g.units[0].clone();
    let v = g.spawn("jeep", -1, u.x + 10., u.y, -1);
    g.command(&json!({"kind":"enter","ids":[u.id],"target":v}));
    g.step(0.04);
    assert_eq!(g.units.iter().find(|x| x.id == v).unwrap().team, 0);
    assert!(g.units.iter().all(|x| x.id != u.id));
}
#[test]
fn passenger_board_and_unload_preserves_identity() {
    let mut g = fixture();
    let u = g.units[0].clone();
    let a = g.spawn("apc", 0, u.x + 10., u.y, -1);
    g.command(&json!({"kind":"board","ids":[u.id],"target":a}));
    g.step(0.04);
    assert_eq!(g.units.iter().find(|x| x.id == a).unwrap().cargo.len(), 1);
    g.command(&json!({"kind":"unload","ids":[a]}));
    assert!(g.units.iter().any(|x| x.id == u.id && !x.dead));
    assert!(g.units.iter().find(|x| x.id == a).unwrap().cargo.is_empty());
}
#[test]
fn enemy_fort_gate_wins() {
    let mut g = fixture();
    let b = g.map.buildings[1].clone();
    let id = g.units[0].id;
    g.units[0].x = b.door.tx as f64 * 32. + 16.;
    g.units[0].y = b.door.ty as f64 * 32. + 16.;
    g.command(&json!({"kind":"hit","ids":[id],"st":{"kind":"b","id":1,"x":b.cx,"y":b.cy}}));
    g.step(0.04);
    assert_eq!(g.over.as_ref().unwrap().winner, 0);
}
#[test]
fn production_uses_ownership_and_can_cancel() {
    let mut g = fixture();
    g.map.buildings[0].prod = Some("grunt".into());
    g.map.buildings[0].progress = 0.99;
    let before = g.force(0);
    g.step(0.1);
    assert_eq!(g.force(0), before + 2);
    g.sync_controls(&json!({"buildings":[{"id":0,"prod":null},{"id":1,"prod":null}]}));
    assert!(g.map.buildings[0].prod.is_none());
    assert!(g.map.buildings[1].prod.is_some());
}
#[test]
fn crane_rebuilds_destroyed_bridge() {
    let mut g = fixture();
    g.map.bridges.push(Bridge {
        id: 0,
        tiles: vec![210],
        hp: 0.,
        maxhp: 500.,
        dead: true,
        x: 336.,
        y: 336.,
        ..Default::default()
    });
    g.map.tiles[210] = 3;
    g.map.block[210] = 1;
    g.map.bridge_at[210] = 0;
    let id = g.spawn("crane", 0, 310., 336., -1);
    g.command(&json!({"kind":"fix","ids":[id],"st":{"kind":"br","id":0,"x":336,"y":336}}));
    for _ in 0..80 {
        g.step(0.04)
    }
    assert!(!g.map.bridges[0].dead);
    assert_eq!(g.map.tiles[210], 5);
    assert_eq!(g.map.block[210], 0);
}
#[test]
fn invalid_map_rejected() {
    assert!(Game::init(&json!({"map":{"W":2,"H":2},"defs":{}})).is_err());
}
