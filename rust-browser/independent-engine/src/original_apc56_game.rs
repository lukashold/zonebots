//!Game binding of the bounded447d50 wrapper. Its existing automatic tail is
//!called once after Common and is not represented as an exact447c20 rewrite.
use crate::{Game,original_hardware_common::LegacyCall,original_path_contact::game::Boundary};
struct Env<'a>{game:&'a mut Game,index:usize}
#[cfg(test)]thread_local!{static TRACE:std::cell::RefCell<Vec<(usize,&'static str)>>=const{std::cell::RefCell::new(Vec::new())};}
#[cfg(test)]pub(crate)fn take_trace()->Vec<(usize,&'static str)>{TRACE.with(|v|std::mem::take(&mut*v.borrow_mut()))}
impl Game{
 pub(crate)fn dispatch_native_apc56(&mut self,index:usize)->bool{let Some(u)=self.units.get(index)else{return false;};if !self.native_route_mode||!self.map.original||u.native_slot.is_none()||u.native_class7c!=1||u.native_phase35!=56||u.kind!="apc"{return false;}if u.native_flags64&64!=0&&u.native_dodge.is_some(){if let Some(slot)=u.native_slot{self.native_common_legacy.push((slot as u8,LegacyCall::Dodge44f730));}self.units[index].native_common_callback=None;return false;}if let Err(e)=super::run(&mut Env{game:self,index}){self.native_route_boundary=Some(e);}true}
}
impl super::World for Env<'_>{type Error=Boundary;
 fn phase(&mut self,value:u8){self.game.units[self.index].native_phase35=value;}
 fn scope(&mut self,value:u8){self.game.native_hardware_common.apc_scope=value;}
 fn common(&mut self)->Result<(),Boundary>{#[cfg(test)]TRACE.with(|v|v.borrow_mut().push((self.index,"Common")));self.game.native_hardware_common_body(self.index)}
 fn words(&self)->[i16;2]{let u=&self.game.units[self.index];[u.native_target42,u.native_target46]}
 fn restore_words(&mut self,value:[i16;2]){let u=&mut self.game.units[self.index];u.native_target42=value[0];u.native_target46=value[1];}
 fn legacy_automatic(&mut self)->Result<(),Boundary>{#[cfg(test)]TRACE.with(|v|v.borrow_mut().push((self.index,"LegacyAutomatic")));if let Some(slot)=self.game.units[self.index].native_slot{self.game.native_common_legacy.push((slot as u8,LegacyCall::ApcAuto447c20));}self.game.update_native_apc_auto_tail_legacy(self.index)}
}
