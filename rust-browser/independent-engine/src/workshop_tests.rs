//! Real command/step workshop lifecycle fixtures, with newly authored terrain.
use super::*;
fn field(kind:&str)->(Game,usize) {
 let mut g=Game::new(0,1,2417);let(w,h)=(48,40);g.map.width=w;g.map.height=h;g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.territory=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.original=true;g.map.world=0;
 g.units.clear();g.buildings.clear();g.walls.clear();g.crates.clear();g.objects.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;
 g.sectors=vec![Sector{id:0,x:64.,y:64.,owner:0,home:true,counts_for_production:true},Sector{id:1,x:1400.,y:1200.,owner:1,home:true,counts_for_production:true}];
 g.buildings.push(Building{id:7,x:512.,y:320.,w:160.,h:128.,kind:"repair".into(),team:0,sector:0,stars:0,hp:1500.,max_hp:1500.,product:None,progress:0.,ready_guns:vec![],gate:Some((592.,448.)),orientation:String::new(),remaining_work:None,completion_ticks:0,raw_base:None});
 for row in 0..4 {for col in 0..5 {let j=(10+row)*w+16+col;let code=if (col==1||col==2)&&row>0 {match row{1=>26,2=>44,_=>19}}else{32};g.map.terrain_codes[j]=code;g.map.tiles[j]=if matches!(code,19|32){2}else{0};}}
 g.spawn(kind,0,272.,600.,1,None);let id=g.units.last().unwrap().id;g.units.last_mut().unwrap().hp=stats(kind).hp/2.;
 g.spawn("gatling",1,1400.,1200.,2,None);for u in &mut g.units{u.cooldown=10000.;u.hold_position=true;}
 (g,id)
}
fn get(g:&Game,id:usize)->&Unit{g.units.iter().find(|u|u.id==id).unwrap()}
fn get_mut(g:&mut Game,id:usize)->&mut Unit{g.units.iter_mut().find(|u|u.id==id).unwrap()}
fn start(g:&mut Game,id:usize){g.command(&json!({"kind":"repairVehicle","ids":[id],"target":7}));}
fn reach_repair(g:&mut Game,id:usize){for _ in 0..2000{if get(g,id).workshop.as_ref().is_some_and(|s|s.phase=="repairing"){return;}g.step(0.1);}panic!("vehicle did not enter workshop: {}",serde_json::to_string(get(g,id)).unwrap());}
fn finish(g:&mut Game,id:usize)->usize{let mut completed=0;for _ in 0..4000{g.events.clear();g.step(0.1);completed+=g.events.iter().filter(|e|e["kind"]=="vehicleRepaired").count();if get(g,id).workshop.is_none(){return completed;}}panic!("workshop did not complete");}
#[test]
fn explicit_workshop_order_enters_hidden_repairs_then_exits_once(){
 let(mut g,id)=field("heavy");start(&mut g,id);assert_eq!(get(&g,id).order,"repairVehicle");assert!(!get(&g,id).path.is_empty());reach_repair(&mut g,id);
 let u=get(&g,id);assert!(workshops::hidden(u));assert_eq!((u.x,u.y),(544.,416.));assert_eq!(u.workshop.as_ref().unwrap().total,2283.);assert_eq!(u.workshop.as_ref().unwrap().seconds,Some(10));
 assert_eq!(finish(&mut g,id),1);let u=get(&g,id);assert_eq!(u.hp,u.max_hp);assert_eq!(u.order,"guard");assert_eq!((u.x,u.y),(560.,548.));assert!(u.workshop.is_none());
}
#[test]
fn admission_restores_only_movement_and_persistent_base_aim(){
 let(mut g,id)=field("heavy");{let u=get_mut(&mut g,id);u.crew.as_mut().unwrap().hp=9.;let c=u.original_combat.as_mut().unwrap();c.crew_boarded("tough");c.aim_skill=7;c.movement_control=5;c.fire_rate=11;c.secondary_state=3;}
 start(&mut g,id);reach_repair(&mut g,id);let u=get(&g,id);let c=u.original_combat.as_ref().unwrap();assert_eq!(c.aim_skill,40);assert_eq!(c.movement_control,30);assert_eq!(c.fire_rate,11);assert_eq!(c.secondary_state,3);assert_eq!(u.crew.as_ref().unwrap().hp,9.);
 finish(&mut g,id);let c=get(&g,id).original_combat.as_ref().unwrap();assert_eq!(c.fire_rate,11);assert_eq!(get(&g,id).crew.as_ref().unwrap().hp,9.);
}
#[test]
fn initial_vehicle_base_aim_is_not_guessed_from_placeholder_driver(){
 let(mut g,id)=field("heavy");get_mut(&mut g,id).original_combat.as_mut().unwrap().aim_skill=5;start(&mut g,id);reach_repair(&mut g,id);assert_eq!(get(&g,id).original_combat.as_ref().unwrap().aim_skill,80);
}
#[test]
fn workshop_requires_damaged_owned_mobile_hardware_and_valid_team(){
 for kind in ["grunt","tough","gatling","cannon","howitzer","missile"]{let(mut g,id)=field(kind);start(&mut g,id);assert!(get(&g,id).workshop.is_none(),"{kind}");}
 for reason in ["full","enemy_shop","neutral_shop","destroyed_shop","no_crew","enemy_request"] {
  let(mut g,id)=field("jeep");match reason{"full"=>get_mut(&mut g,id).hp=stats("jeep").hp,"enemy_shop"=>g.buildings[0].team=1,"neutral_shop"=>g.buildings[0].team=-1,"destroyed_shop"=>g.buildings[0].hp=0.,"no_crew"=>get_mut(&mut g,id).crew=None,_=>{}}
  if reason=="enemy_request"{g.command(&json!({"kind":"repairVehicle","ids":[id],"team":1,"target":7}));}else{start(&mut g,id);}
  assert!(get(&g,id).workshop.is_none(),"{reason}");assert_eq!(get(&g,id).order,"guard");
 }
}
#[test]
fn two_damaged_vehicles_queue_without_repairing_together(){
 let(mut g,first)=field("jeep");get_mut(&mut g,first).x=544.;get_mut(&mut g,first).y=464.;g.spawn("light",0,544.,464.,3,None);let second=g.units.last().unwrap().id;get_mut(&mut g,second).hp=50.;g.command(&json!({"kind":"repairVehicle","ids":[first,second],"target":7}));
 let mut queued=false;let mut repaired=0;
 for _ in 0..3000{g.events.clear();g.step(0.1);let active=g.units.iter().filter(|u|u.workshop.as_ref().is_some_and(|s|s.phase=="repairing")).count();assert!(active<=1);queued|=g.units.iter().any(|u|u.workshop.as_ref().is_some_and(|s|s.phase=="waiting"));repaired+=g.events.iter().filter(|e|e["kind"]=="vehicleRepaired").count();if get(&g,first).workshop.is_none()&&get(&g,second).workshop.is_none(){break;}}
 assert!(queued);assert_eq!(repaired,2);assert_eq!(get(&g,first).hp,get(&g,first).max_hp);assert_eq!(get(&g,second).hp,get(&g,second).max_hp);
}
#[test]
fn stopping_or_replacing_a_hidden_order_releases_vehicle_without_free_heal(){
 for replacement in ["stop","move"]{let(mut g,id)=field("medium");start(&mut g,id);reach_repair(&mut g,id);let hp=get(&g,id).hp;g.command(&json!({"kind":replacement,"ids":[id],"x":900.,"y":700.}));let u=get(&g,id);assert!(u.workshop.is_none());assert_eq!(u.hp,hp);assert!(!workshops::hidden(u));assert!(g.navigation_point_clear(&u.kind,u.x,u.y));assert!(!g.events.iter().any(|e|e["kind"]=="vehicleRepaired"));}
}
#[test]
fn workshop_capture_or_destruction_releases_occupant_without_completion(){
 for lost in [false,true]{let(mut g,id)=field("apc");start(&mut g,id);reach_repair(&mut g,id);let hp=get(&g,id).hp;if lost{g.buildings[0].team=1;}else{g.buildings[0].hp=0.;}g.events.clear();g.step(0.1);let u=get(&g,id);assert!(u.workshop.is_none());assert_eq!(u.hp,hp);assert_eq!(u.team,0);assert!(!g.events.iter().any(|e|e["kind"]=="vehicleRepaired"));}
}
#[test]
fn zero_work_rate_pauses_then_resumes_without_free_passive_healing(){
 let(mut g,id)=field("medium");start(&mut g,id);reach_repair(&mut g,id);g.buildings[0].hp=1.;let hp=get(&g,id).hp;let work=get(&g,id).workshop.as_ref().unwrap().remaining;
 for _ in 0..60{g.step(0.1);}assert_eq!(get(&g,id).hp,hp);let state=get(&g,id).workshop.as_ref().unwrap();assert_eq!(state.remaining,work);assert_eq!(state.seconds,None);g.buildings[0].hp=1500.;assert_eq!(finish(&mut g,id),1);
}
#[test]
fn proximity_alone_no_longer_repairs_a_vehicle(){
 let(mut g,id)=field("jeep");get_mut(&mut g,id).x=592.;get_mut(&mut g,id).y=468.;let hp=get(&g,id).hp;for _ in 0..100{g.step(0.1);}assert_eq!(get(&g,id).hp,hp);
}
#[test]
fn closed_workshop_door_is_authorized_only_for_its_repair_job(){
 let(mut g,id)=field("medium");assert!(!g.movement_segment_clear_for(544.,464.,544.,416.,"medium"));start(&mut g,id);reach_repair(&mut g,id);assert!(workshops::hidden(get(&g,id)));
 let(mut g,id)=field("medium");g.map.tiles[13*g.map.width+17]=6;start(&mut g,id);for _ in 0..1500{g.step(0.1);}assert!(!workshops::hidden(get(&g,id)));assert!(get(&g,id).hp<get(&g,id).max_hp);
}
#[test]
fn hidden_vehicle_cannot_be_shot_boarded_or_capture_a_flag(){
 let(mut g,id)=field("apc");start(&mut g,id);reach_repair(&mut g,id);let p=(get(&g,id).x,get(&g,id).y);
 g.spawn("heavy",1,p.0,p.1,5,None);let enemy=g.units.last().unwrap().id;get_mut(&mut g,enemy).cooldown=0.;get_mut(&mut g,enemy).hold_position=true;g.sectors.push(Sector{id:2,x:p.0,y:p.1,owner:-1,home:false,counts_for_production:false});let hp=get(&g,id).hp;
 g.step(0.05);assert_eq!(get(&g,id).hp,hp);assert_ne!(g.sectors[2].owner,0);assert!(!g.events.iter().any(|e|e["kind"]=="shot"&&e["team"]==1));
 // A visible friendly robot may be shot, but must not board the hidden APC.
 g.spawn("grunt",0,p.0+16.,p.1,4,None);let robot=g.units.last().unwrap().id;get_mut(&mut g,robot).hold_position=true;g.command(&json!({"kind":"board","ids":[robot],"target":id}));g.step(0.05);assert!(get(&g,id).cargo.is_empty());assert_eq!(get(&g,id).hp,hp);
}
#[test]
fn a_zero_hull_original_vehicle_survives_and_can_be_repaired(){
 let(mut g,id)=field("jeep");get_mut(&mut g,id).hp=0.;start(&mut g,id);reach_repair(&mut g,id);assert_eq!(finish(&mut g,id),1);assert_eq!(get(&g,id).hp,stats("jeep").hp);
}
