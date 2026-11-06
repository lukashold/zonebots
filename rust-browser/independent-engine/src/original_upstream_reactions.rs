//! The two source caller prefixes preceding blast damage and idle acquisition.
//! Current native slot identity is authoritative. Removed slots are available
//! only after the real removal hook captured this narrow, source-proven header.
#[path="original_upstream_reactions_game.rs"]pub(crate) mod game;
#[cfg(test)]#[path="original_upstream_reactions_fixture.rs"]mod fixture;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Source{
 pub slot:u8,pub position:[i16;2],pub cell10:i32,pub family:u8,pub class:u8,
 pub flags31:u8,pub role:i8,pub group:u8,pub flags69:u8,pub protectionaa:u8,pub phase35:u8,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{
 MissingSource(u8),MissingRecipient(usize),MissingMember(usize),MissingGroup(u8),
 RetiredGroup(u8),RetiredBroadcast(u8),RetiredIdleTarget(u8),
 MissingIdleTarget(u8),RecipientChanged(usize),
}
use crate::original_reactions::DodgeUnit;

#[derive(Clone,Copy,Debug)]struct Retired{
 generation:u64,source:Source,group_supported:bool,flags64:u8,aim02:i8,
}
pub(crate) struct State{mission_unit_reset:bool,generation:[u64;240],retired:[Option<Retired>;240]}
impl Default for State{fn default()->Self{Self{mission_unit_reset:false,generation:[0;240],retired:[None;240]}}}
impl State{
 pub fn clear_for_mission(&mut self){*self=Self{mission_unit_reset:true,..Self::default()};}
 ///60430 REP STOSD proves only this activity bit until the first constructor.
 pub fn unconstructed_inactive(&self,slot:usize)->bool{self.mission_unit_reset&&self.generation.get(slot)==Some(&0)}
 pub fn constructed(&mut self,slot:usize){if slot<240{self.generation[slot]=self.generation[slot].wrapping_add(1);self.retired[slot]=None;}}
 pub fn generation(&self,slot:usize)->Option<u64>{self.generation.get(slot).copied()}
 pub fn removed(&mut self,slot:usize,generation:u64,source:Source,flags64:u8,aim02:i8){
  if self.generation.get(slot)==Some(&generation){
   self.retired[slot]=Some(Retired{generation,source,group_supported:source.group==255,flags64,aim02});
  }
 }
 fn retired_record(&self,slot:u8)->Option<&Retired>{
  let slot=usize::from(slot);
  self.retired.get(slot)?.as_ref().filter(|record|record.generation==self.generation[slot])
 }
 pub fn source(&self,slot:u8)->Result<Source,Boundary>{
  let r=self.retired_record(slot).ok_or(Boundary::MissingSource(slot))?;
  if r.group_supported{Ok(r.source)}else{Err(Boundary::RetiredGroup(slot))}
 }
 pub fn update_retired(&mut self,slot:usize,generation:u64,source:Source,flags64:u8){
  if self.generation.get(slot)!=Some(&generation){return;}
  if let Some(record)=self.retired[slot].as_mut(){
   record.source=source;
   record.flags64=flags64;
  }
 }
 /// 462fd0 can read an inactive UNIT still listed in a prepared bucket.
 /// Its predicate only reads the UNIT header and never dereferences GROUP.
 pub fn retired_dodge(&self,slot:u8)->Option<DodgeUnit>{
  let r=self.retired_record(slot)?;
  Some(DodgeUnit{position:r.source.position,flags31:r.source.flags31,flags64:r.flags64,
   flags69:r.source.flags69,class7c:r.source.class,state35:r.source.phase35,aim02:r.aim02})
 }
 pub fn write_retired_dodge(&mut self,slot:u8,phase:u8){
  if let Some(r)=self.retired.get_mut(usize::from(slot)).and_then(Option::as_mut){
   r.flags64|=64;
   r.source.phase35=phase;
  }
 }
}
///4208f0's class-independent admission precedes49bc0's own class gate.
pub fn blast_admission(recipient_flags31:u8,owner:u8,shooter:u8)->(bool,bool){
 let candidate=recipient_flags31&4!=0&&recipient_flags31&3!=owner;
 (candidate,candidate&&owner!=255&&shooter!=255)
}
pub trait BlastWorld{type Error;fn reaction(&mut self,slot:u8)->Result<(),Self::Error>;fn report(&mut self,slot:u8)->Result<(),Self::Error>;fn broadcast(&mut self,slot:u8)->Result<(),Self::Error>;}
pub fn blast_prefix<W:BlastWorld>(flags31:u8,owner:u8,shooter:u8,world:&mut W)->Result<bool,W::Error>{let(candidate,react)=blast_admission(flags31,owner,shooter);if react{world.reaction(shooter)?;world.report(shooter)?;world.broadcast(shooter)?;}Ok(candidate)}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn construction_invalidates_old_removed_slot_even_when_old_web_entity_exists(){let mut s=State::default();let raw=Source{slot:2,position:[500,600],cell10:0,family:255,class:0,flags31:1,role:0,group:255,flags69:0,protectionaa:0,phase35:0};s.constructed(2);let g=s.generation(2).unwrap();s.removed(2,g,raw,16,60);assert_eq!(s.source(2),Ok(raw));assert_eq!(s.retired_dodge(2).unwrap().flags64,16);s.constructed(2);assert_eq!(s.source(2),Err(Boundary::MissingSource(2)));assert!(s.retired_dodge(2).is_none());s.removed(2,g,raw,16,60);assert_eq!(s.source(2),Err(Boundary::MissingSource(2)));}
 #[test]fn ownerless_and_missing_shooter_damage_without_reading_source_and_classes_do_not_gate_prefix(){struct W(Vec<u8>);impl BlastWorld for W{type Error=();fn reaction(&mut self,_:u8)->Result<(),()>{self.0.push(1);Ok(())}fn report(&mut self,_:u8)->Result<(),()>{self.0.push(2);Ok(())}fn broadcast(&mut self,_:u8)->Result<(),()>{self.0.push(3);Ok(())}}let mut w=W(vec![]);assert_eq!(blast_prefix(4,255,255,&mut w),Ok(true));assert!(w.0.is_empty());assert_eq!(blast_prefix(4,1,255,&mut w),Ok(true));assert!(w.0.is_empty());assert_eq!(blast_prefix(4,1,0,&mut w),Ok(true));assert_eq!(w.0,[1,2,3]);assert_eq!(blast_prefix(5,1,0,&mut w),Ok(false));}
}
