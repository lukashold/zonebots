//! Public synthetic regressions discovered by command/gameplay audit.
use super::*;
fn field(original:bool,kind:&str)->(Game,usize){
 let mut g=Game::new(0,1,671);let(w,h)=(64,48);g.map.width=w;g.map.height=h;g.map.world=0;g.map.original=original;g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.territory=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();
 g.units.clear();g.buildings.clear();g.sectors.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.effects.clear();g.events.clear();g.ai_teams.clear();g.ai_clock=10000.;g.winner=None;g.next_id=1;
 g.spawn(kind,0,320.,320.,1,None);let id=g.units[0].id;g.spawn("gatling",1,1900.,1400.,2,None);
 for u in &mut g.units{u.hold_position=true;u.cooldown=10000.;}
 (g,id)
}
fn unit(g:&Game,id:usize)->&Unit{g.units.iter().find(|u|u.id==id).unwrap()}
fn unit_mut(g:&mut Game,id:usize)->&mut Unit{let i=at(g,id);&mut g.units[i]}
fn at(g:&Game,id:usize)->usize{g.units.iter().position(|u|u.id==id).unwrap()}
fn building(g:&mut Game,id:usize,team:i32,hp:f64)->usize{
 g.buildings.push(Building{id,x:480.,y:288.,w:64.,h:64.,kind:"vehicle".into(),team,sector:0,stars:1,hp,max_hp:100.,product:None,progress:0.,ready_guns:vec![],gate:None,orientation:String::new(),remaining_work:None,completion_ticks:0,raw_base:None});g.buildings.len()-1
}
fn barrier(g:&mut Game,x:usize){for y in 1..g.map.height-1{g.map.tiles[y*g.map.width+x]=2;g.map.terrain_codes[y*g.map.width+x]=19;}}
#[test]
fn unload_clears_old_orders_and_recrewing_does_not_resume_them(){
 for original in [false,true]{let(mut g,id)=field(original,"medium");g.command(&json!({"ids":[id],"kind":"move","x":1000,"y":600}));assert!(!unit(&g,id).path.is_empty());g.command(&json!({"ids":[id],"kind":"unload"}));let u=unit(&g,id);assert_eq!(u.team,-1);assert!(u.path.is_empty());assert_eq!(u.dest,None);assert_eq!(u.target,None);let position=(u.x,u.y);let robot=g.units.last().unwrap().id;let ri=at(&g,robot);g.units[ri].x=position.0;g.units[ri].y=position.1;g.command(&json!({"ids":[robot],"kind":"enter","target":id}));g.step(0.);assert_eq!(unit(&g,id).team,0);g.step(0.1);assert_eq!((unit(&g,id).x,unit(&g,id).y),position);}
}
#[test]
fn lost_boarding_targets_release_the_order_instead_of_chasing_forever(){
 for change in ["destroyed","enemy","full","removed"]{let(mut g,id)=field(true,"grunt");g.spawn("apc",0,800.,320.,3,None);let target=g.units.last().unwrap().id;g.command(&json!({"ids":[id],"kind":"board","target":target}));g.step(0.);assert!(!unit(&g,id).path.is_empty());let ti=at(&g,target);match change{"destroyed"=>g.units[ti].hp=-1.,"enemy"=>g.units[ti].team=1,"full"=>g.units[ti].cargo=vec![Crew{kind:"grunt".into(),hp:30.};6],_=>g.units.retain(|u|u.id!=target)};g.step(0.);assert_eq!(unit(&g,id).order,"guard","{change}");assert_eq!(unit(&g,id).target,None);assert!(unit(&g,id).path.is_empty());}
}
#[test]
fn boarding_a_moving_apc_replans_before_the_old_route_finishes(){
 let(mut g,id)=field(true,"grunt");g.spawn("apc",0,900.,320.,3,None);let carrier=g.units.last().unwrap().id;g.command(&json!({"ids":[carrier],"kind":"move","x":900,"y":1000}));g.command(&json!({"ids":[id],"kind":"board","target":carrier}));g.step(0.05);let initial=unit(&g,id).dest.unwrap();for _ in 0..20{g.step(0.1);}let u=unit(&g,id);assert_eq!(u.order,"board");assert!(!u.path.is_empty());assert!(u.dest.unwrap().1>initial.1+32.);assert!((u.dest.unwrap().1-unit(&g,carrier).y).abs()<40.);
}
#[test]
fn cranes_repair_stable_building_ids_and_stop_exactly_at_completion(){
 for original in [false,true]{let(mut g,id)=field(original,"crane");let bi=building(&mut g,17,0,95.);unit_mut(&mut g,id).x=512.;unit_mut(&mut g,id).y=380.;g.command(&json!({"ids":[id],"kind":"repair","target":17}));for _ in 0..4{g.step(0.1);}assert_eq!(g.buildings[bi].hp,100.);assert_eq!(unit(&g,id).order,"guard");assert!(unit(&g,id).path.is_empty());assert_eq!(unit(&g,id).target,None);assert_eq!(g.events.iter().filter(|e|e["kind"]=="repair").count(),1);}
}
#[test]
fn crane_reconstruction_allows_owned_rubble_but_not_foreign_targets(){
 let(mut g,id)=field(true,"crane");let bi=building(&mut g,17,0,0.);let ui=at(&g,id);g.units[ui].x=512.;g.units[ui].y=380.;g.command(&json!({"ids":[id],"kind":"repair","target":17}));g.step(0.1);assert!(g.buildings[bi].hp>0.);g.buildings[bi].team=1;g.step(0.1);assert_eq!(unit(&g,id).order,"guard");let hp=g.buildings[bi].hp;g.command(&json!({"ids":[id],"kind":"repair","target":17}));g.step(0.1);assert_eq!(g.buildings[bi].hp,hp);
}
#[test]
fn invalid_or_inapplicable_commands_preserve_an_existing_valid_order(){
 let(mut g,id)=field(true,"grunt");building(&mut g,17,0,20.);g.command(&json!({"ids":[id],"kind":"move","x":900,"y":320}));let old=unit(&g,id).path.clone();for kind in ["repair","repairBridge","unknownOrder"]{g.command(&json!({"ids":[id],"kind":kind,"target":17,"x":400,"y":400}));assert_eq!(unit(&g,id).order,"move");assert_eq!(unit(&g,id).path,old);}
}
#[test]
fn stationary_guns_ignore_move_and_do_not_acquire_ghost_routes(){
 for kind in ["gatling","cannon","missile","howitzer"]{let(mut g,id)=field(true,kind);g.command(&json!({"ids":[id],"kind":"move","x":900,"y":320}));assert_eq!(unit(&g,id).order,"guard");assert!(unit(&g,id).path.is_empty());assert_eq!(unit(&g,id).dest,None);}
}
#[test]
fn structure_attacks_cannot_shoot_through_a_sealed_cliff(){
 for original in [false,true]{let(mut g,id)=field(original,"heavy");let bi=building(&mut g,17,1,100.);barrier(&mut g,13);unit_mut(&mut g,id).cooldown=0.;g.command(&json!({"ids":[id],"kind":"attackBuilding","target":17}));for _ in 0..40{g.step(0.1);}assert_eq!(g.buildings[bi].hp,100.);assert!(!g.events.iter().any(|e|e["kind"]=="buildingLost"));}
}
#[test]
fn reachable_structure_attack_uses_id_and_stops_to_fire_at_the_surface(){
 for original in [false,true]{let(mut g,id)=field(original,"heavy");let bi=building(&mut g,17,1,100.);unit_mut(&mut g,id).cooldown=0.;g.command(&json!({"ids":[id],"kind":"attackBuilding","target":17,"x":512,"y":376}));let before=(unit(&g,id).x,unit(&g,id).y);g.step(0.1);assert_eq!(g.buildings[bi].hp,0.);assert_eq!((unit(&g,id).x,unit(&g,id).y),before);g.step(0.1);assert_eq!(unit(&g,id).order,"guard");}
}
#[test]
fn mounted_guns_do_not_treat_their_own_fort_roof_as_an_external_cliff(){
 let(mut g,id)=field(true,"cannon");let bi=building(&mut g,8,0,100.);g.buildings[bi].kind="fort".into();g.buildings[bi].x=288.;g.buildings[bi].y=288.;g.buildings[bi].w=128.;g.buildings[bi].h=128.;let ui=at(&g,id);g.units[ui].fort_id=Some(8);g.units[ui].cooldown=0.;for y in 9..13{for x in 9..13{g.map.tiles[y*g.map.width+x]=2;}}g.spawn("apc",1,480.,320.,4,None);let target=g.units.last().unwrap().id;g.units.last_mut().unwrap().cooldown=10000.;let hp=unit(&g,target).hp;g.step(0.1);assert!(unit(&g,target).hp<hp);g.map.tiles[10*g.map.width+14]=2;g.units[ui].cooldown=0.;let hp=unit(&g,target).hp;g.step(0.1);assert_eq!(unit(&g,target).hp,hp,"an external cliff must still stop roof fire");
}
#[test]
fn bridge_attacks_require_sight_and_completed_repairs_clear_the_target(){
 let(mut g,id)=field(true,"heavy");let j=10*g.map.width+15;g.map.tiles[j]=5;g.map.terrain_codes[j]=23;g.map.bridge_base[j]=0;g.map.bridge_hp=vec![1.];barrier(&mut g,13);unit_mut(&mut g,id).cooldown=0.;g.command(&json!({"ids":[id],"kind":"attackBridge","x":496,"y":336}));for _ in 0..10{g.step(0.1);}assert_eq!(g.map.bridge_hp[0],1.);
 let(mut g,id)=field(true,"crane");let j=10*g.map.width+12;g.map.tiles[j]=3;g.map.terrain_codes[j]=23;g.map.bridge_base[j]=0;g.map.bridge_hp=vec![399.];g.command(&json!({"ids":[id],"kind":"repairBridge","x":400,"y":336}));g.step(0.1);assert_eq!(g.map.bridge_hp[0],400.);assert_eq!(unit(&g,id).order,"guard");assert_eq!(unit(&g,id).dest,None);assert_eq!(unit(&g,id).target,None);assert!(unit(&g,id).path.is_empty());
}

#[test]
fn exposed_fort_roof_guns_remain_valid_targets_without_allowing_external_cliffs(){
 let(mut g,id)=field(true,"medium");let bi=building(&mut g,8,1,100.);g.buildings[bi].kind="fort".into();g.buildings[bi].w=128.;g.buildings[bi].h=128.;g.spawn("cannon",1,512.,320.,4,None);let target=g.units.last().unwrap().id;g.units.last_mut().unwrap().fort_id=Some(8);g.units.last_mut().unwrap().cooldown=10000.;for y in 9..13{for x in 15..19{g.map.tiles[y*g.map.width+x]=2;}}unit_mut(&mut g,id).cooldown=0.;g.command(&json!({"ids":[id],"kind":"attack","target":target}));g.step(0.1);assert!(g.effects.iter().any(|e|e.kind=="shot"&&e.x2==512.&&e.y2==320.),"roof unit must not be hidden behind its own fort's terrain");
 g.map.tiles[10*g.map.width+14]=2;unit_mut(&mut g,id).cooldown=0.;g.effects.clear();g.step(0.1);assert!(!g.effects.iter().any(|e|e.kind=="shot"&&e.x2==512.&&e.y2==320.),"an unrelated cliff still blocks the shot");
}

#[test]
fn malformed_and_oversized_wall_ids_never_wrap_to_a_real_target(){
 let(mut g,id)=field(true,"heavy");g.import_walls(&json!({"walls":[{"id":0,"x":640,"y":320,"w":32,"h":32,"hp":30,"maxHp":30}]})).unwrap();g.command(&json!({"ids":[id],"kind":"move","x":900,"y":600}));let old=unit(&g,id).path.clone();
 for bad in [json!(-1),json!("0"),json!(0.5),json!(4294967296_u64),json!(u64::MAX)] {g.command(&json!({"ids":[id],"kind":"attackWall","target":bad}));assert_eq!(unit(&g,id).order,"move");assert_eq!(unit(&g,id).target,None);assert_eq!(unit(&g,id).path,old);}
 g.command(&json!({"ids":[id],"kind":"attackWall","target":0}));assert_eq!(unit(&g,id).target,Some(0));assert_eq!(unit(&g,id).order,"attackWall");
}
