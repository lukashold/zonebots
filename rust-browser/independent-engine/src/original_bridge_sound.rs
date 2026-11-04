//!41f870: the bridge's own viewport admission and shared explosion cycle.
//!Unlike420200 it does not inspect/set the callback gate or quiet flag.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Plan{pub sample:i16,pub next_cycle:u8}
pub fn plan(point:[i16;2],camera:[i16;2],size:[i16;2],cycle:u8)->Option<Plan>{
    crate::original_fragment_blast::sound(point,camera,size,false,false,cycle)
        .map(|sample|Plan{sample,next_cycle:cycle.wrapping_add(1)&7})
}
#[cfg(test)]mod tests{
 use super::*;use crate::original_audio::{State,Gate};
 #[test]fn all2048_native_bridge_sound_and_real_descriptor_calls_match(){let native=std::env::var("NATIVE_BRIDGE_SOUND_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|l|l.split(',').map(|n|n.parse::<i64>().unwrap()).collect::<Vec<_>>()).collect::<Vec<_>>());let mut hash=2166136261u32;
    for s in 0..2048usize{let bank=s%5;let cycle=(s/5%8)as u8;let delta=[-49i16,-48,0,672,673,512,32767,-32768];let camera:[i16;2]=[[0,500,32760,-32760][s/40%4],[0,500,-32760,32760][s/160%4]];let point=[camera[0].wrapping_add(delta[s/8%8]),camera[1].wrapping_add(delta[s/64%8])];let size=[[640,0,32767,-32768][s/512%4],480];let clock=if s/80%2==0{0}else{120};let id=[1i16,2,3,4,5,1,2,3][usize::from(cycle)];let mut state=State::default();state.last[bank].fill(0);state.effect_counter=(s*17)as i16;
        let event=plan(point,camera,size,cycle);let played=event.is_some_and(|p|state.admit(bank,p.sample,clock,Gate::Enabled,s/10%2!=0).is_some());let after=event.map_or(cycle,|p|p.next_cycle);let gate=[0,1,255][s%3];
        //681588's high word is retained by the native low-word writer. It is
        //not an invented second audio counter in the Game state.
        let counter=0x12340000u32|u32::from(state.effect_counter as u16);let row=[s as i64,bank as i64,i64::from(point[0]),i64::from(point[1]),i64::from(camera[0]),i64::from(camera[1]),i64::from(size[0]),i64::from(size[1]),i64::from(cycle),i64::from(id),i64::from(played),i64::from(after),gate,i64::from(counter),i64::from(state.last[bank].iter().find(|&&n|n!=0).copied().unwrap_or(0)),0,0];if let Some(records)=&native{assert_eq!(row.as_slice(),records[s].as_slice(),"bridge sound{s}");}for n in row{hash=(hash^n as u32).wrapping_mul(16777619);}
    }assert_eq!(hash,1134249563,"real41f870/46e660; SDL returns-1/3 are the explicit playback boundary; no terrain/pool/RNG writes");
 }
}
