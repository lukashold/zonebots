//!52ab0's native live-target replanning counter. This counts callbacks,
//!not elapsed browser seconds.48470 resolves the target before this kernel.
pub fn refresh_counter(ground:bool,target_class:Option<u8>,active:bool,aim:i8,position:[i16;2],target:[i16;2])->i8{
 if ground{return 127;}
 if active{if let Some(class)=target_class{
  if class==2{return 127;}
  let dx=i32::from(target[0])-i32::from(position[0]);let dy=i32::from(target[1])-i32::from(position[1]);
  let distance=dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
  if distance>=50000{return(distance/5000+10).min(127)as i8;}
 }}
 //The signed byte comparison happens AFTER conversion back toCL.
 (((127-i32::from(aim))>>1)as i8).min(15)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn live_target_counter_matches_the_executed_native_matrix(){
  let mut hash=2166136261u32;let mut count=0;
  for ground in 0..2{for valid in 0..2{for actor in [0,1]{for class in [0,1,2,4]{for aim in [-128i8,-1,0,40,60,120,127]{for d in [[0i16,0],[200,100],[201,100],[500,0],[1000,0],[-32768,32767]]{
   let timer=refresh_counter(ground!=0,(valid!=0).then_some(class),true,aim,[0,0],d);
   for n in [ground,valid,actor,i32::from(class),i32::from(aim),i32::from(d[0]),i32::from(d[1]),i32::from(timer)]{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}}
  assert_eq!(count,1344);assert_eq!(hash,3916989405,"ZNativePursuitTimerOracle:1344 complete native calls");
 }
}
