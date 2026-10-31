//! Synthetic fixtures for the original object's tank coordinate conversions.
use super::*;
use std::f64::consts::PI;

fn field(original:bool)->Game {
    let mut g=Game::new(0,1,671);let(w,h)=(64,48);
    g.map.width=w;g.map.height=h;g.map.world=0;g.map.original=original;
    g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.territory=vec![0;w*h];
    g.map.original_tiles=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();
    g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();
    g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;
    g.winner=None;g.next_id=1;g
}
fn at(g:&Game,id:usize)->usize {g.units.iter().position(|u|u.id==id).unwrap()}

#[test]
fn tank_boarding_and_unloading_use_distinct_native_origins_and_preserve_identity() {
    // Values independently recovered from the native object and family tables,
    // including Heavy's additional initializer offset; every placement is made up.
    for (kind,board,exit) in [("medium",(16.,12.),(-16.,-12.)),("heavy",(12.,8.),(-12.,-8.)),("light",(22.,14.),(-22.,-14.))] {
        for original in [false,true] {for team in [0,1] {
            let mut g=field(original);g.spawn(kind,-1,640.,640.,1,None);let tank=g.units[0].id;
            let hull=stats(kind).hp/5.;g.units[0].hp=hull;
            g.units[0].angle=0.42;g.units[0].aim=-0.71;
            let max_hp=g.units[0].max_hp;
            g.spawn("gatling",1-team,1900.,1400.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;
            g.spawn("sniper",team,640.,650.,3,None);let mut robot=g.units[2].id;g.units[2].hp=17.;
            let mut origin=(640.,640.);
            for _ in 0..2 {
                let ri=at(&g,robot);g.units[ri].x=origin.0;g.units[ri].y=origin.1+10.;
                g.command(&json!({"team":team,"ids":[robot],"kind":"enter","target":tank}));g.step(0.);
                let ti=at(&g,tank);let u=&g.units[ti];
                let occupied=if original {(origin.0+board.0,origin.1+board.1)}else{origin};
                assert_eq!((u.x,u.y),occupied,"{kind} original={original} team={team} boarding");
                assert_eq!(u.team,team);assert_eq!(u.kind,kind);assert_eq!(u.id,tank);
                assert_eq!((u.angle,u.aim),if original{(PI,PI)}else{(0.42,-0.71)});
                assert_eq!((u.hp,u.max_hp),(hull,max_hp));
                assert_eq!(u.crew.as_ref().unwrap().kind,"sniper");assert_eq!(u.crew.as_ref().unwrap().hp,17.);
                assert!(!g.units.iter().any(|u|u.id==robot),"boarding consumes the incoming robot");
                g.command(&json!({"team":team,"ids":[tank],"kind":"unload"}));
                origin=if original {(occupied.0+exit.0,occupied.1+exit.1)}else{occupied};
                let u=&g.units[at(&g,tank)];assert_eq!((u.x,u.y),origin);
                assert_eq!(u.team,-1);assert!(u.crew.is_none());assert_eq!(u.kind,kind);
                assert_eq!((u.hp,u.max_hp),(hull,max_hp));assert!(u.path.is_empty());assert!(u.target.is_none());
                robot=g.units.last().unwrap().id;let driver=g.units.last().unwrap();
                assert_eq!(driver.kind,"sniper");assert_eq!(driver.team,team);assert_eq!(driver.hp,17.);
            }
            assert_eq!(origin,(640.,640.),"repeated crew changes must not move the parked hull");
        }}
    }
}

#[test]
fn owned_mission_tank_spawns_already_use_the_occupied_origin() {
    for kind in ["light","medium","heavy"] {for team in [0,1] {
        let mut g=field(true);g.spawn(kind,team,640.,640.,1,None);
        let id=g.units[0].id;assert_eq!((g.units[0].x,g.units[0].y),(640.,640.));
        assert!(g.units[0].crew.is_some());
        g.spawn("gatling",1-team,1900.,1400.,2,None);g.units[1].hold_position=true;g.units[1].cooldown=10000.;
        g.units[0].hold_position=true;g.step(0.);
        assert_eq!((g.units[at(&g,id)].x,g.units[at(&g,id)].y),(640.,640.));
    }}
}

#[test]
fn killing_a_tank_crew_releases_its_parked_object_without_hull_damage_or_a_new_robot() {
    for (kind,exit) in [("medium",(-16.,-12.)),("heavy",(-12.,-8.)),("light",(-22.,-14.))] {
        for original in [false,true] {for team in [0,1] {
            let mut g=field(original);g.rng=1;
            g.spawn(kind,team,640.,640.,1,None);let tank=g.units[0].id;
            let hull=g.units[0].hp;
            g.units[0].crew.as_mut().unwrap().hp=1.;
            // Tank crews are exposed to native bullets only in an open hatch
            // state. This exercises the actual damage adapter and Game caller.
            if original {g.units[0].original_combat.as_mut().unwrap().secondary_state=3;}
            g.units[0].hold_position=true;g.units[0].cooldown=10000.;
            g.spawn("sniper",1-team,700.,640.,2,None);
            g.units[1].hold_position=true;g.units[1].cooldown=0.;
            let sniper=g.units[1].id;
            g.step(0.);
            let u=&g.units[at(&g,tank)];
            assert_eq!(u.team,-1,"{kind} original={original} team={team}: crew must be killed");
            assert!(u.crew.is_none());assert_eq!(u.kind,kind);assert_eq!(u.hp,hull);
            assert_eq!((u.x,u.y),if original{(640.+exit.0,640.+exit.1)}else{(640.,640.)});
            assert!(u.path.is_empty());assert!(u.dest.is_none());assert!(u.target.is_none());
            assert_eq!(g.units.len(),2,"crew death must not spawn a living robot");
            assert!(g.units.iter().any(|u|u.id==sniper&&u.team==1-team));
        }}
    }
}

#[test]
fn simultaneous_crew_hits_convert_each_tank_origin_only_once() {
    for (kind,exit) in [("medium",(-16.,-12.)),("heavy",(-12.,-8.)),("light",(-22.,-14.))] {
        let mut g=field(true);g.rng=1;g.spawn(kind,0,640.,640.,1,None);
        let tank=g.units[0].id;g.units[0].crew.as_mut().unwrap().hp=1.;
        g.units[0].original_combat.as_mut().unwrap().secondary_state=3;
        g.units[0].hold_position=true;g.units[0].cooldown=10000.;
        for i in 0..2 {g.spawn("sniper",1,700.,640.+i as f64*8.,2+i,None);g.units.last_mut().unwrap().hold_position=true;}
        g.step(0.);let u=&g.units[at(&g,tank)];assert_eq!(u.team,-1);assert!(u.crew.is_none());
        assert_eq!((u.x,u.y),(640.+exit.0,640.+exit.1),"{kind}: conversion applies once for the occupied-to-vacant transition");
    }
}
