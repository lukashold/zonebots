//!4490b0's ordered transaction. Native functions are void: these outcomes
//!describe the reached branch and never manufacture a native success flag.
use crate::original_attack_order::{State,Route,route_kind};
#[path="original_attack_approach_game.rs"]mod game;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Outcome{Skipped,GoalPoolFull,Completed(Route)}
pub trait World{
 type Error;
 fn refresh_counter(&mut self,s:&mut State)->Result<(),Self::Error>;
 fn clear_paths(&mut self,s:&mut State)->Result<(),Self::Error>;
 fn allocate_goal(&mut self,point:[i16;2])->Option<u32>;
 fn build_goal(&mut self,s:&mut State,point:[i16;2],link:u32)->Result<(),Self::Error>;
 fn start_head(&mut self,s:&mut State)->Result<(),Self::Error>;
 fn direct_short_line(&mut self,s:&mut State,point:[i16;2])->Result<(),Self::Error>;
 fn hardware_start(&mut self,s:&mut State);
}
///The direct switch deliberately omits family38; queued includes it.
pub fn phase(family:u8,kind:Route)->u8{match family{24=>53,25=>54,30=>55,32=>56,34=>58,36=>59,38 if kind==Route::Queued=>60,_=>9}}
pub fn approach<W:World>(s:&mut State,point:[i16;2],world:&mut W)->Result<Outcome,W::Error>{
 let kind=route_kind(s);if kind==Route::None{return Ok(Outcome::Skipped);}
 world.refresh_counter(s)?;
 if kind==Route::Queued{
  world.clear_paths(s)?;
  let Some(link)=world.allocate_goal(point)else{return Ok(Outcome::GoalPoolFull);};
  world.build_goal(s,point,link)?;
  //490b0 ignores52810's inverse-start return after successful allocation.
  world.start_head(s)?;
 }else{world.direct_short_line(s,point)?;}
 s.flags71|=128;s.motion61=1;
 if s.class==1{s.phase=phase(s.family,kind);world.hardware_start(s);}
 Ok(Outcome::Completed(kind))
}
