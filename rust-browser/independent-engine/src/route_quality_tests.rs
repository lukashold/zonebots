//! Route quality and continuous-navigation regressions on synthetic maps.
use super::*;
fn field(kind:&str,start:(f64,f64))->(Game,usize){
 let mut g=Game::new(0,1,20417);let(w,h)=(64,32);g.map.width=w;g.map.height=h;
 g.map.tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.territory=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.original=true;g.map.world=0;
 g.units.clear();g.buildings.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.sectors.clear();g.ai_teams.clear();g.ai_clock=10000.;g.effects.clear();g.events.clear();g.winner=None;
 g.spawn(kind,0,start.0,start.1,1,None);let id=g.units.last().unwrap().id;
 g.spawn("gatling",1,1968.,976.,2,None);
 for u in &mut g.units{u.cooldown=10000.;u.hold_position=true;}
 (g,id)
}
fn get(g:&Game,id:usize)->&Unit{g.units.iter().find(|u|u.id==id).unwrap()}
fn walk(g:&mut Game,id:usize,goals:&[(f64,f64)])->f64{
 let mut ticks=0;
 for &(x,y)in goals{
  g.command(&json!({"ids":[id],"kind":"move","x":x,"y":y}));
  for _ in 0..10000{if get(g,id).path.is_empty(){break;}g.step(0.05);ticks+=1;}
  let u=get(g,id);assert!(u.path.is_empty());assert!((u.x-x).hypot(u.y-y)<1e-6,"actual click endpoint must survive");
 }
 ticks as f64*0.05
}
fn add_building(g:&mut Game,x:f64,y:f64,w:f64,h:f64,gate:Option<(f64,f64)>,orientation:&str){
 g.buildings.push(Building{id:0,x,y,w,h,kind:"vehicle".into(),team:-1,sector:0,stars:1,hp:100.,max_hp:100.,product:None,progress:0.,ready_guns:Vec::new(),gate,orientation:orientation.into(),remaining_work:None,completion_ticks:0,raw_base:None});
}
#[test]
fn exact_oblique_click_keeps_a_straight_open_field_route(){
 let(g,_)=field("medium",(179.,340.));let goal=(771.,549.);let path=g.route_for(179.,340.,goal.0,goal.1,"medium");
 assert_eq!(path.back(),Some(&goal));let mut p=(179.,340.);let mut length=0.;
 for q in path{assert!(g.movement_segment_clear_for(p.0,p.1,q.0,q.1,"medium"));length+=(q.0-p.0).hypot(q.1-p.1);p=q;}
 assert!((length-(goal.0-179.).hypot(goal.1-340.)).abs()<1e-6);
}
#[test]
fn an_expensive_road_detour_cannot_beat_a_legal_manual_shortcut(){
 let(mut automatic,id)=field("jeep",(144.,496.));
 for x in 4..=50{let j=5*automatic.map.width+x;automatic.map.tiles[j]=1;automatic.map.terrain_codes[j]=7;}
 let(mut manual,_)=field("jeep",(144.,496.));manual.map=automatic.map.clone();let one=walk(&mut automatic,id,&[(1616.,496.)]);let two=walk(&mut manual,id,&[(880.,496.),(1616.,496.)]);
 assert!(one<=two+0.05,"automatic {one}s vs manual {two}s");
}
#[test]
fn a_nearby_useful_road_still_improves_arrival_time(){
 let(mut plain,id)=field("medium",(112.,336.));let(mut road,_)=field("medium",(112.,336.));
 for x in 3..=28{let j=9*road.map.width+x;road.map.tiles[j]=1;road.map.terrain_codes[j]=7;}
 let ground=walk(&mut plain,id,&[(912.,336.)]);let paved=walk(&mut road,id,&[(912.,336.)]);
 assert!(paved<ground*0.93,"nearby road {paved}s vs ground {ground}s");
}
#[test]
fn movement_budget_survives_multiple_short_waypoints(){
 let(mut g,id)=field("jeep",(144.,496.));g.command(&json!({"ids":[id],"kind":"move","x":208.,"y":496.}));
 let i=g.units.iter().position(|u|u.id==id).unwrap();g.units[i].path=VecDeque::from([(146.,496.),(148.,496.),(176.,496.),(208.,496.)]);
 g.step(0.1);assert!((get(&g,id).x-(144.+stats("jeep").speed*0.1)).abs()<1e-8,"unused time must continue through both short waypoints");
}
#[test]
fn exact_grid_traversal_rejects_a_corner_and_accepts_a_legal_near_miss(){
 let(mut g,_)=field("jeep",(127.,127.));let j=4*g.map.width+3;g.map.tiles[j]=3;g.map.terrain_codes[j]=40;
 assert!(!g.movement_segment_clear_for(127.,127.,160.,160.,"jeep"),"a true diagonal corner touches blocked water");
 assert!(!g.movement_segment_clear_for(127.,127.,128.,128.,"jeep"),"ending exactly at the corner must not avoid its blocked side cells");
 assert!(g.movement_segment_clear_for(127.,127.,160.,158.,"jeep"),"a legal line can pass the other side of a corner without entering water");
 g.map.tiles[3*g.map.width+4]=3;g.map.terrain_codes[3*g.map.width+4]=40;
 assert!(!g.movement_segment_clear_for(127.999999,127.999998,160.,144.,"jeep"),"even a subpixel visit to a blocked cell must fail");
}
#[test]
fn continuous_building_margin_blocks_shortcuts_but_gates_stay_open(){
 let(mut g,_)=field("grunt",(280.,323.));add_building(&mut g,330.,280.,100.,46.,None,"");
 assert!(!g.movement_segment_clear_for(280.,323.,480.,323.,"grunt"),"actual building margin lies outside blocked tile centers");
 g.buildings.clear();add_building(&mut g,320.,320.,128.,128.,Some((384.,448.)),"south");
 assert!(g.movement_segment_clear_for(384.,424.,384.,500.,"grunt"),"south gate corridor must permit departure");
 assert!(!g.movement_segment_clear_for(340.,500.,340.,400.,"grunt"),"the same building remains solid beside its gate");
 g.buildings[0].orientation="north".into();g.buildings[0].gate=Some((384.,320.));
 assert!(g.movement_segment_clear_for(384.,344.,384.,280.,"grunt"),"north gate corridor must permit departure");
 assert!(!g.movement_segment_clear_for(340.,280.,340.,400.,"grunt"));
}
#[test]
fn a_blocked_spawn_may_leave_its_cell_but_cannot_enter_another_block(){
 let(mut g,_)=field("grunt",(178.,338.));let j=10*g.map.width+5;g.map.tiles[j]=2;g.map.terrain_codes[j]=19;
 assert!(g.movement_segment_clear_for(178.,338.,208.,336.,"grunt"));
 g.map.tiles[10*g.map.width+6]=2;g.map.terrain_codes[10*g.map.width+6]=19;
 assert!(!g.movement_segment_clear_for(178.,338.,240.,336.,"grunt"));
}
#[test]
fn firing_routes_retain_a_legal_firing_endpoint_after_reduction(){
 let(g,id)=field("medium",(144.,496.));let u=get(&g,id);let target=(1500.,800.);let range=240.;
 let path=g.route_to_attack(u,target.0,target.1,range);let last=*path.back().unwrap();
 assert!((last.0-target.0).hypot(last.1-target.1)<=range);assert!(g.los(last.0,last.1,target.0,target.1));
 let mut previous=(u.x,u.y);for p in path{assert!(g.movement_segment_clear_for(previous.0,previous.1,p.0,p.1,&u.kind));previous=p;}
}
#[test]
fn offcenter_start_routes_do_not_retry_an_illegal_first_leg(){
 let(mut g,id)=field("grunt",(414.,331.));add_building(&mut g,330.,280.,100.,46.,None,"");
 // The start is just south of the real building and inside a cell whose center
 // is clear. A direct first diagonal northward would clip its bottom-right edge.
 let goal=(500.,260.);let path=g.route_for(414.,331.,goal.0,goal.1,"grunt");assert!(!path.is_empty());
 let mut p=(414.,331.);for q in &path{assert!(g.movement_segment_clear_for(p.0,p.1,q.0,q.1,"grunt"));p=*q;}
 walk(&mut g,id,&[goal]);
}

#[test]
fn legal_pixel_beside_a_building_remains_reachable_even_when_its_cell_center_is_solid(){
 let(mut g,id)=field("grunt",(280.,340.));add_building(&mut g,330.,280.,100.,100.,None,"");
 assert!(!g.passable_for(10,10,"grunt"));
 let end=(322.,340.);let path=g.route_for(280.,340.,end.0,end.1,"grunt");assert_eq!(path.back(),Some(&end));
 walk(&mut g,id,&[end]);
 let blocked=g.route_for(end.0,end.1,336.,340.,"grunt");assert_ne!(blocked.back(),Some(&(336.,340.)),"a point inside the actual building remains forbidden");
}
#[test]
fn native_raster_price_uses_major_cells_not_euclidean_length_and_resets_each_leg(){
 use super::original_terrain::Profile;
 let(mut g,_)=field("grunt",(176.,176.));
 assert_eq!(g.native_leg_price((176.,176.),(496.,176.),Profile::Infantry),50);
 assert_eq!(g.native_leg_price((176.,176.),(496.,496.),Profile::Infantry),50);
 assert_eq!(g.native_leg_price((496.,176.),(176.,176.),Profile::Infantry),50);
 for x in 5..=10{g.map.terrain_codes[5*g.map.width+x]=42;}
 assert_eq!(g.native_leg_price((176.,176.),(336.,176.),Profile::Infantry),30);
 let split=g.native_leg_price((176.,176.),(272.,176.),Profile::Infantry)+g.native_leg_price((272.,176.),(336.,176.),Profile::Infantry);
 assert_eq!(split,35,"consecutive expensive discount restarts in each segment");
}
