//!420ea0 constructs only the first two native fort pools. Unlike41f8f0,
//!each template supplies its own timers and all thresholds are multiplied.
use crate::original_building_fragments::Pool;
use crate::original_display::Record;
use crate::Value;
#[derive(Clone)]pub(super) struct State{pub pools:[Pool;4],pub templates:Option<Vec<[i16;6]>>,pub draws:Vec<(Record,usize,usize)>}
impl Default for State{fn default()->Self{Self{pools:std::array::from_fn(|_|Pool::default()),templates:None,draws:Vec::new()}}}
impl State{
 pub fn import(old:Option<&Self>,v:&Value)->Self{let mut out=old.cloned().unwrap_or_default();for p in &mut out.pools{p.clear();}out.draws.clear();out.templates=v["nativeFortFragments"]["templates"].as_array().filter(|r|r.len()<=50).and_then(|rows|rows.iter().map(|r|{let a=r.as_array().filter(|r|r.len()==6)?;let mut t=[0;6];for(i,n)in a.iter().enumerate(){t[i]=i16::try_from(n.as_i64()?).ok()?;}Some(t)}).collect());out}
}
pub(super) fn construct(pool:&mut Pool,position:[i16;2],templates:&[[i16;6]],current:i16){for t in templates{if t[0]==-1{break;}let Some(r)=pool.records.iter_mut().find(|r|r.word(0)==-1)else{continue;};r.set(0,position[0].wrapping_add(t[0]));r.set(2,position[1].wrapping_add(t[1]).wrapping_sub(32));r.set(8,t[2].wrapping_abs());r.set(10,t[3].wrapping_mul(current/9));r.bytes[12]=0;r.bytes[13]=t[4]as u8;r.bytes[14]=u8::from(t[2]<0);r.bytes[15]=t[5]as u8;r.bytes[16]=t[5]as u8;}}
#[cfg(test)]#[path="original_fort_fragments_tests.rs"]mod tests;
