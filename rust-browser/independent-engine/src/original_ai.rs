//! Native strategic decision clock00417b30/00419fb0. The strategy that consumes
//! this clock is still the browser adapter; the original sector task chain is
//! not replaced by a claim of complete AI parity.
use super::original_combat::OriginalRng;

pub struct Clock {pub control:i16,remaining:i32,pub mode:bool}
impl Clock {
 pub fn new(control:i16)->Self {Self{control,remaining:(127-i32::from(control))*2,mode:true}}
 pub fn advance(&mut self,tick:u64)->bool {
  if tick&1!=0{return false;}
  self.remaining-=1;
  if self.remaining>=0{return false;}
  self.mode=!self.mode;self.remaining=(127-i32::from(self.control))*2;true
 }
}
///0041a8b0 perturbs a strategy score without consuming the gameplay RNG state.
/// The signed arithmetic shift and the control>127 branch matter here.
pub fn perturb(score:i32,control:i16,rng:&OriginalRng)->i32 {
 if control==127{return score;}
 let mut copy=rng.clone();let span=127-i32::from(control);
 let random=(copy.next_u32()&65535)as i32;
 let amount=if span<0{random%(1-span)+span}else{random%(span+1)};
 let mut delta=(i64::from(amount)*i64::from(score)>>7)as i32;
 if copy.next_u32()&1!=0{delta=-delta;}score.wrapping_add(delta)
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn decisions_use_alternate_callbacks_and_strictly_negative_countdown() {
  for control in [0,55,100,126,127,140] {
   let mut clock=Clock::new(control);let interval=(127-i32::from(control)).max(0)*4+2;
   let due=(1..1000).find(|t|clock.advance(*t));assert_eq!(due,Some(interval as u64));assert!(!clock.mode);
   let due=(interval as u64+1..2000).find(|t|clock.advance(*t));assert_eq!(due,Some(2*interval as u64));assert!(clock.mode);
  }
 }
 #[test]
 fn scores_match_fifteen_calls_to_the_original_x86_function() {
  let rng=OriginalRng::seeded(371);
  for (control,input,expected) in [(0, -128, -46),(0, 0, 0),(0, 1000, 360),(55, -128, -123),(55, 0, 0),(55, 1000, 961),(100, -128, -122),(100, 0, 0),(100, 1000, 954),(127, -128, -128),(127, 0, 0),(127, 1000, 1000),(140, -128, -135),(140, 0, 0),(140, 1000, 1055)] {assert_eq!(perturb(input,control,&rng),expected);}
 }

 #[test]
 fn strategy_noise_never_changes_the_gameplay_generator() {
  let rng=OriginalRng::seeded(371);let before=rng.words;
  let score=perturb(1000,55,&rng);assert_eq!(score,961);assert_eq!(rng.words,before);
  assert_eq!(perturb(1000,127,&rng),1000);
  assert_eq!(perturb(1000,55,&rng),score);
 }
}
