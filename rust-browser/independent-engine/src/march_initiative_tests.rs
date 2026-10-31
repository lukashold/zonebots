//! Synthetic route-interaction regressions; no original level data is embedded.
use super::*;
use super::original_initiative::Initiative;

fn field(original:bool,team:i32)->Game {
    let mut g=Game::new(0,1,337);let(w,h)=(80,48);
    g.map.original=original;g.map.world=0;g.map.width=w;g.map.height=h;
    g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.territory=vec![0;w*h];
    g.map.original_tiles=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();
    g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();
    g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;
    g.winner=None;g.next_id=1;
    g.spawn("grunt",team,320.,320.,1,None);
    g.spawn("gatling",1-team,2300.,1300.,2,None);
    g.units[1].hold_position=true;g.units[1].cooldown=10000.;
    g
}
fn unit(g:&Game,id:usize)->&Unit {g.units.iter().find(|u|u.id==id).unwrap()}
fn flag(g:&mut Game,x:f64,y:f64)->usize {
    let id=g.sectors.len();g.sectors.push(Sector{id,x,y,owner:-1,home:false,counts_for_production:true});id
}
fn march(g:&mut Game,ids:&[usize],team:i32,x:f64,y:f64) {
    g.command(&json!({"kind":"move","team":team,"ids":ids,"x":x,"y":y}));
}
fn run(g:&mut Game,steps:usize){for _ in 0..steps{g.step(0.1);}}

#[test]
fn marching_soldier_detours_to_several_flags_then_reaches_the_ordered_destination() {
    for team in [0,1] {
        let mut g=field(true,team);let robot=g.units[0].id;
        flag(&mut g,480.,360.);flag(&mut g,880.,360.);
        march(&mut g,&[robot],team,1400.,320.);
        assert!(!unit(&g,robot).path.is_empty());g.step(0.1);
        assert_eq!(unit(&g,robot).initiative,Some(Initiative::Flag(0)));
        assert_eq!(unit(&g,robot).march_resume,Some((1400.,320.)));
        run(&mut g,700);
        assert!(g.sectors.iter().all(|s|s.owner==team));
        let u=unit(&g,robot);assert!((u.x-1400.).hypot(u.y-320.)<0.01);
        assert!(u.path.is_empty());assert!(u.initiative.is_none());assert!(u.march_resume.is_none());
    }
}

#[test]
fn marching_driver_transfers_the_destination_to_the_new_mobile_vehicle() {
    for team in [0,1] {
        let mut g=field(true,team);let robot=g.units[0].id;
        g.spawn("jeep",-1,460.,360.,3,None);let hardware=g.units.last().unwrap().id;
        march(&mut g,&[robot],team,1400.,320.);g.step(0.1);
        assert_eq!(unit(&g,robot).initiative,Some(Initiative::Vehicle(hardware)));
        run(&mut g,700);
        assert!(!g.units.iter().any(|u|u.id==robot),"The actual boarding consumes its infantry driver.");
        let v=unit(&g,hardware);assert_eq!(v.kind,"jeep");assert_eq!(v.team,team);assert!(v.crew.is_some());
        assert!((v.x-1400.).hypot(v.y-320.)<0.01,"Newly crewed vehicle must continue the original march.");
        assert!(v.path.is_empty());assert!(v.initiative.is_none());assert!(v.march_resume.is_none());
    }
}

#[test]
fn automatic_stationary_gun_crew_stays_in_place_without_an_impossible_march() {
    let mut g=field(true,0);let robot=g.units[0].id;
    g.spawn("cannon",-1,460.,360.,3,None);let hardware=g.units.last().unwrap().id;
    march(&mut g,&[robot],0,1400.,320.);run(&mut g,250);
    let gun=unit(&g,hardware);assert_eq!(gun.team,0);assert_eq!(gun.kind,"cannon");assert!(gun.crew.is_some());
    assert_eq!((gun.x,gun.y),(460.,360.));assert_eq!(gun.order,"guard");
    assert!(gun.dest.is_none());assert!(gun.path.is_empty());assert!(gun.march_resume.is_none());
}

#[test]
fn marching_squad_reserves_one_driver_and_other_members_keep_their_own_destinations() {
    let mut g=field(true,0);let first=g.units[0].id;
    g.spawn("grunt",0,332.,320.,1,None);let second=g.units.last().unwrap().id;
    g.spawn("grunt",0,344.,320.,1,None);let third=g.units.last().unwrap().id;
    g.spawn("jeep",-1,460.,360.,3,None);let hardware=g.units.last().unwrap().id;
    let ids=[first,second,third];march(&mut g,&ids,0,1400.,320.);g.step(0.1);
    assert_eq!(g.units.iter().filter(|u|u.initiative==Some(Initiative::Vehicle(hardware))).count(),1);
    run(&mut g,700);
    assert_eq!(g.units.iter().filter(|u|u.team==0&&stats(&u.kind).cls==0).count(),2);
    assert_eq!(unit(&g,hardware).team,0);
    for (n,id) in ids.iter().enumerate() {if let Some(u)=g.units.iter().find(|u|u.id==*id) {
        assert!((u.x-(1400.+n as f64*12.)).hypot(u.y-320.)<0.01);
        assert!(u.initiative.is_none());assert!(u.march_resume.is_none());
    }}
}

#[test]
fn losing_a_flag_or_vehicle_claim_resumes_the_saved_march() {
    for hardware_case in [false,true] {
        let mut g=field(true,0);let robot=g.units[0].id;
        let goal=if hardware_case {
            g.spawn("jeep",-1,480.,400.,3,None);Initiative::Vehicle(g.units.last().unwrap().id)
        }else{Initiative::Flag(flag(&mut g,480.,400.))};
        march(&mut g,&[robot],0,1400.,320.);g.step(0.1);assert_eq!(unit(&g,robot).initiative,Some(goal));
        match goal {
            Initiative::Flag(id)=>g.sectors[id].owner=0,
            Initiative::Vehicle(id)=>{let v=g.units.iter_mut().find(|u|u.id==id).unwrap();v.team=0;v.crew=Some(Crew{kind:"grunt".into(),hp:30.});v.hold_position=true;},
        }
        g.step(0.1);let u=unit(&g,robot);assert!(u.initiative.is_none());assert_eq!(u.order,"move");assert_eq!(u.dest,Some((1400.,320.)));
        run(&mut g,700);let u=unit(&g,robot);assert!((u.x-1400.).hypot(u.y-320.)<0.01);
    }
}

#[test]
fn stop_and_new_commands_discard_the_old_detour_continuation() {
    for stop in [false,true] {
        let mut g=field(true,0);let robot=g.units[0].id;flag(&mut g,480.,400.);
        march(&mut g,&[robot],0,1400.,320.);g.step(0.1);assert!(unit(&g,robot).march_resume.is_some());
        if stop {g.command(&json!({"kind":"stop","ids":[robot]}));}
        else {march(&mut g,&[robot],0,100.,320.);}
        let u=unit(&g,robot);assert!(u.initiative.is_none());assert!(u.march_resume.is_none());
        if stop {let point=(u.x,u.y);run(&mut g,100);assert_eq!((unit(&g,robot).x,unit(&g,robot).y),point);assert_eq!(g.sectors[0].owner,-1);}
        else {assert_eq!(u.dest,Some((100.,320.)));}
    }
}

#[test]
fn attack_and_explicit_entry_orders_are_not_hijacked_by_nearer_objects() {
    for explicit_enter in [false,true] {
        let mut g=field(true,0);let robot=g.units[0].id;flag(&mut g,400.,360.);
        g.spawn("jeep",-1,460.,360.,3,None);let near=g.units.last().unwrap().id;
        let target=if explicit_enter {g.spawn("jeep",-1,1000.,320.,4,None);g.units.last().unwrap().id}else{g.units[1].id};
        g.command(&json!({"kind":if explicit_enter{"enter"}else{"attack"},"ids":[robot],"target":target,"x":1000,"y":320}));
        g.step(0.1);let u=unit(&g,robot);
        assert!(u.initiative.is_none());assert!(u.march_resume.is_none());assert_eq!(u.target,Some(target));
        assert_ne!(u.target,Some(near));assert_eq!(g.sectors[0].owner,-1);
    }
}

#[test]
fn marching_vehicles_cannot_supply_a_driver_to_other_hardware() {
    let mut g=field(true,0);g.units.clear();g.spawn("jeep",0,320.,320.,1,None);let first=g.units[0].id;
    g.spawn("gatling",1,2300.,1300.,2,None);g.units[1].hold_position=true;
    g.spawn("jeep",-1,460.,360.,3,None);let empty=g.units.last().unwrap().id;
    march(&mut g,&[first],0,1400.,320.);run(&mut g,400);
    assert_eq!(unit(&g,empty).team,-1);assert!(unit(&g,empty).crew.is_none());
    assert!((unit(&g,first).x-1400.).hypot(unit(&g,first).y-320.)<0.01);
}

#[test]
fn procedural_marches_keep_their_previous_non_detouring_behavior() {
    let mut g=field(false,0);let robot=g.units[0].id;flag(&mut g,480.,380.);
    g.spawn("jeep",-1,460.,360.,3,None);let empty=g.units.last().unwrap().id;
    march(&mut g,&[robot],0,1400.,320.);g.step(0.1);
    assert!(unit(&g,robot).initiative.is_none());assert!(unit(&g,robot).march_resume.is_none());
    run(&mut g,700);assert_eq!(unit(&g,empty).team,-1);assert_eq!(g.sectors[0].owner,-1);
    assert!((unit(&g,robot).x-1400.).hypot(unit(&g,robot).y-320.)<0.01);
}

#[test]
fn active_march_includes_the_native_quantized_scan_boundary_but_idle_scan_does_not() {
    for marching in [false,true] {
        let mut g=field(true,0);let robot=g.units[0].id;flag(&mut g,560.,320.);
        assert_eq!(original_initiative::object_distance(320.,320.,560.,320.),15);
        if marching {march(&mut g,&[robot],0,1400.,320.);assert!(!unit(&g,robot).path.is_empty());}
        g.step(0.);
        assert_eq!(unit(&g,robot).initiative,if marching{Some(Initiative::Flag(0))}else{None});
        assert_eq!(unit(&g,robot).march_resume,if marching{Some((1400.,320.))}else{None});
    }
}

#[test]
fn marching_scan_rejects_dead_or_owned_hardware_and_flags_behind_cliffs() {
    let mut g=field(true,0);let robot=g.units[0].id;flag(&mut g,400.,400.);
    g.map.tiles[12*g.map.width+12]=2;
    for (team,x) in [(0,400.),(1,440.),(-1,480.)] {
        g.spawn("jeep",team,x,440.,4,None);let v=g.units.last_mut().unwrap();
        if team<0 {v.hp=-1.;}else{v.hold_position=true;v.cooldown=10000.;}
    }
    march(&mut g,&[robot],0,1400.,320.);g.step(0.);
    let u=unit(&g,robot);assert!(u.initiative.is_none());assert!(u.march_resume.is_none());
    assert_eq!(u.dest,Some((1400.,320.)));assert!(!u.path.is_empty());assert_eq!(g.sectors[0].owner,-1);
}

#[test]
fn another_driver_taking_the_vehicle_earlier_in_the_same_tick_restores_the_march() {
    let mut g=field(true,0);let explicit_driver=g.units[0].id;
    g.units[0].x=900.;g.units[0].y=900.;g.units[0].hold_position=true;g.units[0].hp=27.;
    g.spawn("grunt",0,320.,320.,3,None);let marcher=g.units.last().unwrap().id;
    g.units.last_mut().unwrap().hp=13.;
    g.spawn("jeep",-1,460.,360.,4,None);let hardware=g.units.last().unwrap().id;
    march(&mut g,&[marcher],0,1400.,320.);g.step(0.1);
    assert_eq!(unit(&g,marcher).initiative,Some(Initiative::Vehicle(hardware)));
    let first=g.units.iter_mut().find(|u|u.id==explicit_driver).unwrap();first.x=460.;first.y=360.;
    g.command(&json!({"kind":"enter","ids":[explicit_driver],"target":hardware}));
    g.step(0.1);
    assert!(!g.units.iter().any(|u|u.id==explicit_driver));
    assert_eq!(unit(&g,hardware).crew.as_ref().unwrap().hp,27.);
    let u=unit(&g,marcher);assert_eq!(u.hp,13.);assert!(u.initiative.is_none());assert!(u.march_resume.is_none());
    assert_eq!(u.order,"move");assert_eq!(u.dest,Some((1400.,320.)));assert!(!u.path.is_empty());
    run(&mut g,700);assert!((unit(&g,marcher).x-1400.).hypot(unit(&g,marcher).y-320.)<0.01);
}

#[test]
fn the_new_vehicle_replans_the_infantry_march_via_a_bridge_instead_of_wading() {
    let mut g=field(true,0);let robot=g.units[0].id;
    for y in 0..g.map.height {let j=y*g.map.width+24;g.map.tiles[j]=3;g.map.terrain_codes[j]=9;}
    let bridge=20*g.map.width+24;g.map.tiles[bridge]=5;g.map.terrain_codes[bridge]=23;
    assert!(g.passable_for(24,10,"grunt"));assert!(!g.passable_for(24,10,"jeep"));
    g.spawn("jeep",-1,460.,360.,3,None);let hardware=g.units.last().unwrap().id;
    march(&mut g,&[robot],0,1400.,320.);
    for _ in 0..200 {g.step(0.1);if unit(&g,hardware).team==0 {break;}}
    let v=unit(&g,hardware);assert_eq!(v.team,0);assert_eq!(v.order,"move");
    assert_eq!(v.dest,Some((1400.,320.)));assert!(!v.path.is_empty());
    let mut previous=(v.x,v.y);
    for &(x,y) in &v.path {assert!(g.movement_segment_clear_for(previous.0,previous.1,x,y,"jeep"));previous=(x,y);}
    let mut max_y=v.y;
    for _ in 0..1000 {
        g.step(0.1);let v=unit(&g,hardware);max_y=max_y.max(v.y);
        let cell=(v.y/TILE).floor()as usize*g.map.width+(v.x/TILE).floor()as usize;
        assert_ne!(g.map.terrain_codes[cell],9,"The new driver must not retain the infantry water route.");
    }
    assert!(max_y>600.,"A vehicle crossing this river must use the distant bridge.");
    let v=unit(&g,hardware);assert!((v.x-1400.).hypot(v.y-320.)<0.01);
}
