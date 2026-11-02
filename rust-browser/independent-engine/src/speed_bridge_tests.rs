//! Actual movement and bridge lifecycle on synthetic, asset-free maps.
use super::*;

fn field(original:bool,difficulty:usize)->Game {
    let mut g=Game::new(0,difficulty,991);let(w,h)=(100,48);
    g.map.width=w;g.map.height=h;g.map.world=0;g.map.original=original;
    g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.territory=vec![0;w*h];
    g.map.original_tiles=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();
    g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();
    g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;g.next_id=1;g
}
fn run(g:&mut Game,n:usize){for _ in 0..n{g.step(0.1);}}
fn near(a:f64,b:f64){assert!((a-b).abs()<1e-6,"{a} != {b}");}
fn distance(kind:&str,team:i32,difficulty:usize,code:u8,original:bool)->f64 {
    let mut g=field(original,difficulty);g.map.terrain_codes.fill(code);
    g.spawn(kind,team,320.,320.,1,None);let id=g.units[0].id;
    g.spawn("gatling",1-team,2900.,1400.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;
    g.command(&json!({"kind":"move","ids":[id],"team":team,"x":2500.,"y":320.}));
    run(&mut g,10);g.units.iter().find(|u|u.id==id).unwrap().x-320.
}

#[test]
fn all_original_robot_classes_use_native_pattern_ratios_and_a_common_slower_pace() {
    for (kind,expected) in [("grunt",32.),("psycho",32.),("tough",25.6),("sniper",128./3.),("pyro",32.),("laser",128./3.)] {
        for team in [0,1] {for difficulty in 0..4 {near(distance(kind,team,difficulty,0,true),expected);}}
    }
    near(distance("sniper",0,1,0,true)/distance("grunt",0,1,0,true),4./3.);
}
#[test]
fn original_vehicle_paces_are_shared_by_both_factions_and_fixed_guns_stay_fixed() {
    for (kind,expected) in [("jeep",51.2),("medium",32.),("light",128./3.),("heavy",25.6),("launcher",64./3.),("apc",128./3.),("crane",128./3.),("truck",51.2),("firetruck",128./3.),("gatling",0.),("cannon",0.)] {
        for team in [0,1] {near(distance(kind,team,3,0,true),expected);}
    }
}
#[test]
fn roads_water_and_damage_use_the_same_quantized_speed_patterns() {
    near(distance("grunt",0,1,7,true),51.2);near(distance("sniper",1,1,7,true),64.);
    near(distance("grunt",0,1,9,true),25.6);near(distance("sniper",1,1,9,true),32.);
    let mut g=field(true,1);g.spawn("jeep",1,320.,320.,1,None);let id=g.units[0].id;
    g.units[0].original_combat.as_mut().unwrap().movement_control=45;
    g.spawn("gatling",0,2900.,1400.,2,None);g.units[1].hold_position=true;
    g.command(&json!({"kind":"move","ids":[id],"team":1,"x":2500.,"y":320.}));run(&mut g,10);
    near(g.units[0].x-320.,32.);
}
#[test]
fn strategic_ai_cannot_add_movement_speed_at_any_difficulty() {
    for difficulty in 0..4 {
        let mut g=field(true,difficulty);g.ai_teams=vec![1];g.ai_clock=0.;
        g.sectors.push(Sector{id:0,x:2500.,y:700.,owner:0,home:true,counts_for_production:true});
        g.spawn("grunt",0,320.,320.,1,None);let own=g.units[0].id;
        g.spawn("grunt",1,320.,700.,2,None);let enemy=g.units[1].id;
        g.command(&json!({"kind":"move","ids":[own],"team":0,"x":2500.,"y":320.}));run(&mut g,10);
        near(g.units.iter().find(|u|u.id==own).unwrap().x-320.,32.);
        near(g.units.iter().find(|u|u.id==enemy).unwrap().x-320.,32.);
    }
}
#[test]
fn procedural_movement_keeps_its_previous_pace() {
    near(distance("grunt",0,1,0,false),48.);near(distance("sniper",1,1,0,false),52.8);
    near(distance("jeep",1,3,0,false),72.);
}
fn river(g:&mut Game) {
    for y in 0..g.map.height {for x in 12..15 {let j=y*g.map.width+x;g.map.tiles[j]=3;g.map.terrain_codes[j]=40;}}
    g.map.bridge_hp=vec![0.];g.map.bridge_max_hp=vec![600.];
    for x in 12..15 {let j=12*g.map.width+x;g.map.bridge_base[j]=0;g.map.terrain_codes[j]=23;}
}
#[test]
fn crane_rebuilds_a_destroyed_bridge_to_its_own_capacity_before_vehicles_can_cross() {
    for team in [0,1] {
        let mut g=field(true,1);river(&mut g);
        g.spawn("jeep",team,272.,400.,1,None);let car=g.units[0].id;
        g.spawn("crane",team,336.,400.,2,None);let crane=g.units[1].id;
        g.spawn("gatling",1-team,2900.,1400.,3,None);g.units[2].hold_position=true;
        g.command(&json!({"kind":"move","ids":[car],"team":team,"x":656.,"y":400.}));run(&mut g,40);
        assert!(g.units[0].x<384.);assert_eq!(g.map.tiles[12*g.map.width+13],3);
        g.command(&json!({"kind":"repairBridge","ids":[crane],"team":team,"x":432.,"y":400.}));run(&mut g,80);
        near(g.map.bridge_hp[0],520.);assert_eq!(g.map.tiles[12*g.map.width+13],3,"HP above400 still needs the full600HP reconstruction.");
        run(&mut g,20);near(g.map.bridge_hp[0],600.);assert_eq!(g.map.tiles[12*g.map.width+13],5);
        assert_eq!(g.map.revision,1);assert_eq!(g.units[1].order,"guard");assert!(g.units[1].dest.is_none());
        g.command(&json!({"kind":"move","ids":[car],"team":team,"x":656.,"y":400.}));run(&mut g,180);
        near(g.units.iter().find(|u|u.id==car).unwrap().x,656.);near(g.map.bridge_hp[0],600.);assert_eq!(g.map.revision,1);
    }
}
#[test]
fn importing_a_zero_hp_bridge_overrides_stale_intact_compatibility_tiles() {
    let mut g=field(true,1);river(&mut g);let j=12*g.map.width+13;g.map.tiles[j]=5;
    let mission=json!({"map":g.map,"sectors":[],"buildings":[],"crates":[],"originalObjects":[],"spawns":[]});
    g.apply_original(&mission).unwrap();assert_eq!(g.map.tiles[j],3);near(g.bridge_capacity(0),600.);
    let mut invalid=mission.clone();invalid["map"]["bridgeMaxHp"]=json!([0]);assert!(g.apply_original(&invalid).is_err());
}
