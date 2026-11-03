//! Browser mouse-event bridge to the source UI classifier and ordered group
//! broadcast. The accepted Web order backend remains responsible for routes.
use super::{Game,Value,json,original_combat,original_attack_click_game::Input};
impl Game{
 pub(crate) fn try_native_human_attack_command(&mut self,v:&Value,ids:&[usize],team:i32)->bool{
  let kind=v["kind"].as_str().unwrap_or("move");
  if !self.map.original||self.map.native_road_graph.is_none()||self.multiplayer||v["nativeEvent"]==false||!matches!(kind,"attack"|"attackBuilding"|"attackWall"|"attackBridge")||v.get("nativeClick").is_none(){return false;}
  let Some(point)=v["nativeClick"].as_array().filter(|p|p.len()==2).and_then(|p|Some([i16::try_from(p[0].as_i64()?).ok()?,i16::try_from(p[1].as_i64()?).ok()?]))else{return true;};
  if point[0]<0||point[1]<0||usize::from(point[0]as u16)>=self.map.width*16||usize::from(point[1]as u16)>=self.map.height*16{return true;}
  let eligible=ids.iter().filter_map(|id|self.units.iter().position(|u|u.id==*id&&u.team==team&&original_combat::alive(true,u.hp)&&!super::workshops::hidden(u))).collect::<Vec<_>>();
  //Ctrl-single group detachment still belongs to the older selection adapter.
  if eligible.iter().any(|&i|self.units[i].native_slot.and_then(|s|self.native_group_rosters.members[s].as_ref()).is_some_and(|m|m.role>0&&!self.native_group_rosters.pool.get(usize::from(m.group)).and_then(|g|g.member(0)).is_some_and(|s|eligible.iter().any(|&j|self.units[j].native_slot==Some(usize::from(s)))))){return false;}
  let Some(primary)=v["nativePrimary"].as_u64().and_then(|id|eligible.iter().copied().find(|&i|self.units[i].id==id as usize)).or_else(||eligible.first().copied()).and_then(|i|{
   let slot=self.units[i].native_slot?;let m=self.native_group_rosters.members[slot].as_ref()?;
   if m.role<=0{Some(i)}else{let slot=self.native_group_rosters.pool.get(usize::from(m.group))?.member(0)?;eligible.iter().copied().find(|&i|self.units[i].native_slot==Some(usize::from(slot)))}
  })else{return true;};
  let input=Input{hardware:matches!(self.units[primary].native_class7c,1|2),point,fallback_point:point,inside_view:true,cursor_icon:0xadf,override_ground:None,repair_records:None};
  //No workshop timer or absent saved-class/link byte is invented. These
  //typed unavailable-context cases retain the existing interaction backend.
  if self.classify_native_human_attack(primary,&input).is_err(){return false;}
  let before=self.native_selection.clone();let mut groups=vec![];
  for &i in &eligible{if let Some(g)=self.units[i].native_slot.and_then(|s|self.native_group_rosters.members[s].as_ref()).map(|m|m.group).filter(|&g|g!=255){if !groups.contains(&g){groups.push(g);}}}
  self.select_native_groups(&groups);
  let batch=match self.prepare_native_human_attack_batch(primary,&input){Ok(b)=>b,Err(_)=>{self.native_selection=before;return false;}};
  for order in batch.orders{
   let index=order.index;let actor=&self.units[index];if actor.team!=team||!ids.contains(&actor.id)||!original_combat::alive(true,actor.hp){continue;}
   let member_ids=ids.iter().copied().filter(|id|self.units.iter().any(|u|u.id==*id&&u.team==team&&u.squad==actor.squad)).collect::<Vec<_>>();let event=order.event;
   let mut command=json!({"team":team,"ids":member_ids,"nativeEvent":false});
   match event.opcode{
    1=>{command["kind"]=json!("move");command["x"]=json!(f64::from(event.waypoint[0].wrapping_sub(8))*2.);command["y"]=json!(f64::from(event.waypoint[1].wrapping_sub(8))*2.);},
    4=>{command["kind"]=json!("stop");},
    2|5=>{
     if let Some(target)=self.native_order_target_index(index,event){command["kind"]=json!("attack");command["target"]=json!(self.units[target].id);}
     else{let p=if event.opcode==5{[event.waypoint[1],event.metadata]}else{event.waypoint};
      if (event.opcode==5&&event.waypoint[0]!=3456)||(event.opcode==2&&matches!(event.waypoint[0],-4321|-1234)){continue;}
      command["kind"]=json!(kind);command["x"]=json!(f64::from(p[0])*2.);command["y"]=json!(f64::from(p[1])*2.);
      if kind!="attack"{command["target"]=v["target"].clone();}
      if kind=="attackBuilding"{command["impactX"]=command["x"].clone();command["impactY"]=command["y"].clone();}
     }
    },
    _=>continue,
   }
   self.command(&command);self.record_native_order_event(index,event);
  }
  if batch.decision.reset_selected_actor{
   if let Some(slot)=self.native_selected_slot{if let Some(m)=self.native_group_rosters.members.get(slot).and_then(Option::as_ref).filter(|m|m.role<=0){if let Some(g)=self.native_group_rosters.pool.get_mut(usize::from(m.group)){g.roster[9]&=!2;}}}
   self.native_selected_slot=None;
  }
  true
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn field()->Game{
  let mut g=Game::new(0,1,743);let template=g.buildings.iter().find(|b|b.kind=="fort").unwrap().clone();
  g.map.original=true;g.map.width=64;g.map.height=64;g.map.tiles=vec![0;4096];g.map.terrain_codes=vec![1;4096];g.map.native_terrain_plane=vec![1;16384];
  g.map.native_road_graph=Some(super::super::original_route::RoadGraph{regions:vec![super::super::original_route::RoadRegion{bounds:[0;4],waypoint:0,next:vec![255;96]};96]});
  g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.native_group_rosters.clear();
  for(i,k)in["grunt","jeep","tough"].into_iter().enumerate(){g.spawn(k,0,100.+i as f64*40.,100.,7+i,None);}
  g.spawn("grunt",1,600.,600.,19,None);
  for(i,base)in[[800.,32.],[800.,800.]].into_iter().enumerate(){let mut b=template.clone();b.id=500+i;b.team=1-i as i32;b.raw_base=Some((base[0],base[1]));b.x=base[0]*2.;b.y=base[1]*2.;g.buildings.push(b);}
  g.maintain_native_group_rosters();g.import_native_world(&Value::Null);g
 }
 #[test]fn mouse_attack_broadcast_preserves_mixed_move_and_attack_decisions(){
  let mut g=field();let ids=g.units[..3].iter().map(|u|u.id).collect::<Vec<_>>();
  g.command(&json!({"kind":"attack","ids":ids,"nativePrimary":ids[2],"x":800.,"y":1000.,"nativeClick":[400,500]}));
  assert_eq!(g.units[..3].iter().map(|u|u.order.as_str()).collect::<Vec<_>>(),vec!["move","move","attack"]);
  assert_eq!(g.units[..3].iter().map(|u|u.native_order5c).collect::<Vec<_>>(),vec![1,1,5]);
  let slot=g.units[2].native_slot.unwrap();let m=g.native_group_rosters.members[slot].as_ref().unwrap();assert_eq!(m.promotion.target_x,3456);assert_eq!(m.promotion.target_y,400);assert_eq!(m.promotion.target_aux,500);
 }
 #[test]fn mouse_live_target_uses_native_group_reference_and_keeps_web_identity(){
  let mut g=field();let id=g.units[0].id;let target=g.units[3].id;
  g.command(&json!({"kind":"attack","ids":[id],"target":target,"nativeClick":[308,308]}));
  assert_eq!(g.units[0].target,Some(target));assert_eq!(g.units[0].native_order5c,5);
  let slot=g.units[0].native_slot.unwrap();assert!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.target_x<0);
 }
}
