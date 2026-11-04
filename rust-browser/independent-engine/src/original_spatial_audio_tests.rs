use super::*;use crate::Value;
fn rows(variable:&str)->Option<Vec<Value>>{let p=std::env::var_os(variable)?;Some(std::fs::read_to_string(p).unwrap().lines().map(|l|serde_json::from_str(l).unwrap()).collect())}
fn n(r:&Value,k:&str)->i64{r[k].as_i64().unwrap()}
fn fold(h:&mut u32,v:i64){*h=(*h^v as u32).wrapping_mul(16777619);}
#[test]fn spatial_audio_matches1536_actual_original_prefixes(){
 let Some(rows)=rows("NATIVE_SPATIAL_AUDIO_PREFIX")else{return;};assert_eq!(rows.len(),1536);let mut signature=2166136261;
 for scenario in 0..128{let mut s=State::default();s.last[0].fill(if scenario/8%2==0{0}else{500});s.effect_counter=3;s.opportunity_gate=[0,1,255][scenario/4%3];s.explosion_cycle=(scenario/8%8)as u8;
  for event in 0..12{let r=&rows[scenario*12+event];assert_eq!((n(r,"scenario"),n(r,"event")),(scenario as i64,event as i64));if event==4||event==8{s.opportunity_gate=0;}assert_eq!((i64::from(s.opportunity_gate),i64::from(s.explosion_cycle)),(n(r,"gateBefore"),n(r,"cycleBefore")));assert_eq!((n(r,"counterBefore")as u16)as i16,s.effect_counter);
   let point=[n(r,"x")as i16,n(r,"y")as i16];let camera=[n(r,"cameraX")as i16,0];let chance=match n(r,"kind"){0=>Chance::Blast420200{point,bounds:[4096,4096],mask4f:n(r,"suppress")as u8,quiet:n(r,"quiet")!=0},1=>Chance::Bridge41f870{point},2=>Chance::Owner4Expiry3{point},3=>Chance::Removal6{point},4=>Chance::Owner4Expiry3{point},5=>Chance::Crate6{point},_=>unreachable!()};
   //Kind4 is the separately recorded fort prefix policy; this does NOT wire
   //the currently unported422a60/e00 fort destructor caller into the game.
   let reached=!(n(r,"kind")==3&&n(r,"deathQuiet")!=0);let master=n(r,"enabled")!=0;let plan=if reached{s.opportunity_plan(chance,camera,[640,480]).unwrap()}else{Plan::default()};let requested=if master{plan.request.map_or(-1,|q|i64::from(q.sample))}else{-1};let result=if reached{s.apply_opportunity(chance,camera,[640,480],0,500,master)}else{None};let counter=(n(r,"counterBefore")as u32&0xffff0000)|u32::from(s.effect_counter as u16);
   assert_eq!((i64::from(s.opportunity_gate),i64::from(s.explosion_cycle),i64::from(counter),requested,i64::from(requested!=-1),i64::from(result.is_some())),(n(r,"gate"),n(r,"cycle"),n(r,"counter"),n(r,"requested"),n(r,"requests"),n(r,"mixes")),"source{scenario}/{event}");assert!(s.opportunity_boundary.is_none());for v in [i64::from(s.opportunity_gate),i64::from(s.explosion_cycle),i64::from(counter),requested,i64::from(result.is_some())]{fold(&mut signature,v);}
  }
 }assert_eq!(signature,3102958722);println!("SPATIAL_SOURCE1536_STATE signature={signature}");
}
#[test]fn spatial_audio_matches128_actual_expiry1_7_first_suppressed_prefixes(){
 let Some(rows)=rows("NATIVE_SUPPRESSED_AUDIO_PREFIX")else{return;};assert_eq!(rows.len(),128);let mut signature=2166136261;
 for r in rows{let mut s=State::default();s.last[0].fill(0);s.effect_counter=3;s.opportunity_gate=n(&r,"gateBefore")as u8;s.explosion_cycle=n(&r,"cycleBefore")as u8;let chance=Chance::Blast420200{point:[n(&r,"x")as i16,n(&r,"y")as i16],bounds:[4096,4096],mask4f:n(&r,"mask4f")as u8,quiet:n(&r,"quiet")!=0};let camera=[n(&r,"camera")as i16,0];let master=n(&r,"enabled")!=0;let plan=s.opportunity_plan(chance,camera,[320,240]).unwrap();let requested=if master{plan.request.map_or(-1,|q|i64::from(q.sample))}else{-1};let d=s.apply_opportunity(chance,camera,[320,240],0,500,master);
  assert_eq!((n(&r,"suppressionBeforeCall"),n(&r,"suppressionAfterPrefix"),n(&r,"strength")),(1,1,if n(&r,"kind")==0{10}else{0}));assert_eq!((i64::from(s.opportunity_gate),i64::from(s.explosion_cycle),0x12340000|i64::from(s.effect_counter as u16),requested,i64::from(d.is_some())),(n(&r,"gate"),n(&r,"cycle"),n(&r,"counter"),n(&r,"requested"),n(&r,"mixes")));for v in [i64::from(s.opportunity_gate),i64::from(s.explosion_cycle),requested,i64::from(d.is_some())]{fold(&mut signature,v);}
 }assert_eq!(signature,1031114429);println!("SPATIAL_SOURCE128_STATE signature={signature}");
}
#[test]fn spatial_audio_asymmetric_coupling_consumes_disabled_chances_without_shared_alias(){
 let(mut host,mut guest)=(State::default(),State::default());let views=[[0,0],[400,400]];let sources=[Chance::Owner4Expiry3{point:[308,288]},Chance::Blast420200{point:[308,288],bounds:[1024,1024],mask4f:0,quiet:false},Chance::Bridge41f870{point:[320,320]},Chance::Crate6{point:[850,850]}];let mut clips=[vec![],vec![]];
 for source in sources{for(i,s)in[&mut host,&mut guest].into_iter().enumerate(){if let Some(d)=s.apply_opportunity(source,views[i],[320,320],0,500,i==0){clips[i].push(d.id);}}}
 assert_eq!(clips,[vec![1],vec![]]);assert_eq!((host.opportunity_gate,guest.opportunity_gate),(1,1));assert_eq!((host.explosion_cycle,guest.explosion_cycle),(1,0));assert_eq!((host.effect_counter,guest.effect_counter),(1,0));assert!(guest.apply_opportunity(Chance::FragmentWater10{point:[450,450]},views[1],[320,320],0,500,true).is_none());assert_eq!(guest.apply_opportunity(Chance::FallingWater7{point:[450,450]},views[1],[320,320],0,500,true).unwrap().id,8);assert_eq!(guest.opportunity_gate,1);
}
#[test]fn spatial_audio_ff_crate_water_and_enabled_metal_have_distinct_writers(){
 let mut s=State::default();s.opportunity_gate=255;assert!(s.apply_opportunity(Chance::Crate6{point:[50,50]},[0,0],[320,240],0,500,false).is_none());assert_eq!(s.opportunity_gate,1);s.opportunity_gate=0;assert_eq!(s.apply_opportunity(Chance::FragmentWater10{point:[50,50]},[0,0],[320,240],0,500,true).unwrap().id,8);assert_eq!(s.opportunity_gate,0);assert_eq!(s.effect_counter,1);assert_eq!(s.apply_opportunity(Chance::FragmentMetal10{point:[50,50]},[0,0],[320,240],0,500,true).unwrap().id,11);assert_eq!(s.effect_counter,2,"Enabled counts once, no extra Direct increment");assert_eq!(s.opportunity_gate,0);assert_eq!(s.explosion_cycle,0);
}
#[test]fn spatial_audio_double_camera_wrap_and_invalid_cycle_are_explicit(){
 let s=State::default();assert_eq!(s.opportunity_plan(Chance::Crate6{point:[-32760,-32760]},[16380,16380],[320,240]).unwrap().gate,Some(1));assert_eq!(s.opportunity_plan(Chance::Removal6{point:[-32760,-32760]},[16380,16380],[320,240]).unwrap().gate,None);
 let mut s=State::default();s.explosion_cycle=255;let before=s.last.clone();assert!(s.apply_opportunity(Chance::Bridge41f870{point:[0,0]},[0,0],[320,240],0,500,true).is_none());assert_eq!(s.opportunity_boundary,Some(Boundary::InvalidCycle(255)));assert_eq!((s.opportunity_gate,s.explosion_cycle,s.effect_counter),(0,255,0));assert_eq!(s.last,before);
}
