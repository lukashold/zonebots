//! AI-only adapter for the verified native factory product selectors. Producers
//! call this when their initial product isFF or immediately after completion.
//! Human factory defaults and manual production choices remain caller-owned.
use super::{Game,original_factory as native,original_combat,original_stats};

fn products()->Vec<native::Product>{
    //The local unmodifiedGOG ROBOTS.DAT defines products0..18;19..31 are
    //disabled. These exact levels/costs also feed the128 native oracle cases.
    original_stats::PRODUCTS.iter().enumerate().map(|(id,p)|native::Product{id:id as u8,enabled:true,first_member:true,profile:super::native_ai_scoring::default_input(p.0).unwrap().profile,levels:[p.2 as i8,p.3 as i8,p.4 as i8],duration:p.5}).collect()
}

#[cfg(test)]mod tests{
 use super::*;use super::super::{json,original_strategy};
 fn field()->Game{
  let mut g=Game::new(0,1,802);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);g.units.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.buildings.truncate(1);g.ai_teams=vec![1];
  let b=&mut g.buildings[0];b.id=0;b.kind="robot".into();b.team=1;b.sector=0;b.x=200.;b.y=200.;b.hp=1000.;b.max_hp=1000.;b.product=Some("grunt".into());
  g.sectors=vec![super::super::Sector{id:0,x:144.,y:272.,owner:1,home:true,counts_for_production:true},super::super::Sector{id:1,x:656.,y:272.,owner:0,home:true,counts_for_production:true}];
  let controls=json!({"nativeAi":{"decisionControl":0,"reservePercent":25,"populationWeight":1,"initialSectorTasks":[]},"sectors":[
   {"id":0,"rect":[0,0,10,20],"nativeSlot":0,"rawFlagTile":[4,8],"nativeFlags":0,"nativePopulation":0,"owner":1},
   {"id":1,"rect":[10,0,30,20],"nativeSlot":1,"rawFlagTile":[20,8],"nativeFlags":0,"nativePopulation":0,"owner":0}],
   "buildings":[{"id":0,"kind":"robot","rawType":4,"rawTechnology":1000,"rawAttackOffset":[32,32],"team":1,"sector":0,"x":200,"y":200,"hp":1000}]});
  g.native_strategy=original_strategy::State::from_mission(&controls);g.import_native_world(&super::super::Value::Null);g.native_factory_flags6.clear();g.native_factory_flags6.insert(0,0);
  g.spawn("grunt",1,160.,256.,101,None);g.spawn("grunt",0,700.,256.,201,None);g.update_original_strategy(false);g
 }
 #[test]fn cpu_robot_factory_reads_real_technology_and_sets_native_selection_flag(){
  let mut g=field();assert_eq!(g.native_cpu_factory_product(0,false,0),Some("grunt".into()));assert_eq!(g.native_factory_flags6.get(&0),Some(&2));
  assert_eq!(g.buildings[0].product.as_deref(),Some("grunt"),"selector returns the event's next product; producer commits it");
 }
 #[test]fn human_factories_and_missing_native_metadata_keep_their_current_product(){
  let mut g=field();g.buildings[0].team=0;g.buildings[0].product=Some("psycho".into());assert!(g.native_cpu_factory_product(0,false,0).is_none());assert_eq!(g.native_factory_flags6.get(&0),Some(&0));assert_eq!(g.buildings[0].product.as_deref(),Some("psycho"));
  g.buildings[0].team=1;g.native_factory_flags6.remove(&0);assert!(g.native_cpu_factory_product(0,false,0).is_none());assert_eq!(g.buildings[0].product.as_deref(),Some("psycho"));
 }
}
impl Game{
    ///16c60/16ab0/189e0 selection at a producer's actual initial/completion
    ///event. `pending_sector_guns` is the native5ba0a4/61b650 pending-placement
    ///count in this sector; it must not be guessed from ready stock alone.
    pub(crate) fn native_cpu_factory_product(&mut self,index:usize,deferred:bool,pending_sector_guns:i16)->Option<String>{
        let b=self.buildings.get(index)?.clone();
        if !self.map.original||b.team!=1||!self.ai_teams.contains(&1)||!b.alive(true){return None;}
        let producer=match b.kind.as_str(){"fort"=>native::Producer::Fort,"robot"=>native::Producer::Robot,"vehicle"=>native::Producer::Vehicle,_=>return None};
        let flags6=*self.native_factory_flags6.get(&b.id)?;let technology=self.native_factory_technology(b.id)?;
        let(attack,reserve,priority,human,cpu)=self.native_factory_strategy_snapshot(1,b.sector)?;
        let sector_of=|u:&super::Unit|self.native_strategy.as_ref().and_then(|s|s.sector_at(u.x,u.y));
        let live_guns=self.units.iter().filter(|u|u.native_slot.is_some()&&u.team==1&&original_combat::alive(true,u.hp)&&u.native_class7c==2&&sector_of(u)==Some(b.sector)).count()as i16;
        let planned=self.buildings.iter().filter(|b|b.alive(true)&&b.team==1&&b.product.as_deref().is_some_and(super::gun_placement::is_gun)).count()as i16;
        let ordinary_orders=self.buildings.iter().filter(|v|v.alive(true)&&v.team==1&&v.sector==b.sector&&v.product.as_deref().is_some_and(|p|!super::gun_placement::is_gun(p))).count()as i32;
        //3e900 counts all unoccupied world guns in this sector, regardless of
        //team. Finished stock remains inBUILD52..55 and is a different count.
        let neutral=self.native_ai_world_objects()?.iter().filter(|o|o.position[0]>=0&&o.sector==Some(b.sector)&&matches!(o.definition,Some(5|21|22|23))).count()as i16;
        let sector_guns=live_guns.wrapping_add(pending_sector_guns).wrapping_add(neutral);
        let context=native::CpuContext{producer,flags6,technology,deferred,attack,reserve,sector_priority:priority,planned_cpu_guns:planned,sector_load:i32::from(live_guns).wrapping_add(ordinary_orders),gun_capacity_blocked:sector_guns>3};
        let branch=native::cpu_branch(&context);self.native_factory_flags6.insert(b.id,branch.flags6);
        let id=native::select_cpu_product(&products(),&context,human,cpu)?;
        original_stats::PRODUCTS.get(usize::from(id)).map(|p|p.0.to_string())
    }
}
