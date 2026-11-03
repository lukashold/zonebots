//!404af0 and407120..407850 dispatch into the one native100-record pool.
//!Coordinates are the original callsite arguments, even after the source
//!record's X has been set to−1. Allocation may immediately reuse that slot.
use super::original_ambient as ambient;
#[derive(Clone,Copy,Debug)]pub enum Action{Spawn(ambient::Spawn),Chunk{kind:u8,position:[i16;2],source35:u8}}
pub trait World{fn random(&mut self)->u32;fn chunk(&mut self,kind:u8,position:[i16;2],source35:u8);}
fn spawn(position:[i16;2],template:u8,arg4:u8,arg5:u8,arg6:i8,arg7:i16,arg8:i16,heading:u8,delay:i16,arg11:u8)->Action{Action::Spawn(ambient::Spawn{position,template,arg4,arg5,arg6,arg7,arg8,heading,delay,arg11})}
fn shifted(p:[i16;2],x:i16,y:i16)->[i16;2]{[p[0].wrapping_add(x),p[1].wrapping_add(y)]}
const PART_OFFSET:[[i16;2];4]=[[1,-5],[3,0],[2,4],[-4,0]];
const SMOKE_OFFSET:[[i16;2];4]=[[1,4],[-5,0],[2,-7],[4,0]];
pub fn plan(kind:i16,position:[i16;2],heading:u8,source35:u8,mut random:impl FnMut()->u32)->Vec<Action>{
 let p=position;let mut out=Vec::new();match kind{
  1|2=>out.push(Action::Chunk{kind:kind as u8,position:p,source35}),
  3=>{out.push(spawn(p,5,255,3,127,22,0,heading,0,0));out.push(spawn(shifted(p,0,-5),12,16,2,127,23,-1,0,0,5));},
  4|16=>{let o=SMOKE_OFFSET[usize::from(heading&3)];out.push(spawn(shifted(p,-o[0],-o[1]-16),if kind==4{31}else{26},0,2,-1,-1,-1,0,0,16));},
  5|17=>{let template=if kind==5{31}else{26};out.push(spawn(shifted(p,0,-16),template,0,2,-1,-1,-1,0,0,16));out.push(spawn(p,if kind==5{44}else{36},0,2,-1,-1,-1,0,0,0));let aligned=p.map(|v|v&!15);out.push(spawn(shifted(aligned,0,-16),template,0,2,-1,-1,-1,0,50,16));let d=usize::from(heading>>6);let o=PART_OFFSET[d];out.push(spawn(shifted(aligned,o[0],o[1]),if kind==5{27}else{21}+d as u8,0,2,4,if kind==5{4}else{16},-1,0,50,0));},
  6|7|10|11|18|19=>{let increment=if random()&1!=0{32i16}else{-32};let turn=if kind>=18{heading.wrapping_sub(increment as u8)}else{heading.wrapping_add(increment as u8)};let(template,event,smoke)=match kind{6=>(43,5,31),7=>(43,6,31),10=>(46,9,50),11=>(46,10,50),18=>(35,17,26),_=>(35,18,26)};out.push(spawn(p,template,0,20,127,event,-1,turn,0,0));out.push(spawn(shifted(p,0,-16),smoke,0,2,-1,-1,-1,0,0,16));},
  8|12|20=>{let turn=((random()&7)<<5)as u8;let(template,event)=match kind{8=>(43,7),12=>(46,11),_=>(35,19)};out.push(spawn(p,template,0,20,127,event,-1,turn,0,0));},
  9=>out.push(spawn(shifted(p,0,-16),51,0,2,-1,-1,-1,0,0,16)),
  13=>out.push(spawn(p,48,0,2,-1,-1,-1,heading&64,0,0)),
  14|15=>{let o=VAPOR_OFFSET[usize::from(heading>>6)];out.push(spawn(shifted(p,-o[0],-o[1]),if kind==14{45}else{47},if kind==14{0}else{4},2,127,13,0,heading,0,0));},
  21=>{out.push(spawn(shifted(p,1,0),7,0,2,-1,-1,-1,0,0,0));out.push(spawn(shifted(p,1,-48),0,0,2,-1,-1,-1,0,0,48));},
  22|23=>out.push(spawn([p[0],p[1].wrapping_neg()],if kind==22{14}else{13},0,2,-1,0,-1,0,0,0)),
  _=>{},
 }out
}
const VAPOR_OFFSET:[[i16;2];4]=[[2,4],[0,2],[2,0],[4,2]];
///405c60 allocates independently for every request; failure of an early
///request does not stop later requests or undo the emitter's one RNG draw.
pub fn emit(pool:&mut ambient::State,source_slot:usize,position:[i16;2],heading:u8,world:&mut impl World){let Some(source)=pool.records.get(source_slot)else{return;};let kind=source.word(18);let source35=source.byte(35);for action in plan(kind,position,heading,source35,||world.random()){match action{Action::Spawn(s)=>{pool.spawn(s);},Action::Chunk{kind,position,source35}=>world.chunk(kind,position,source35)}}}
#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,values:&[u32]){for &v in values{*h=(*h^v).wrapping_mul(16777619);}}
 #[test]fn emitter_dispatch_matches_all_6656_executed_native_sequences(){
  let points=[[346,420],[-17,-33],[0,0],[32767,-32768]];let mut total=2166136261u32;
  for kind in 0..26i16{for seed in 0..256u32{let mut calls=0;let actions=plan(if kind==25{-1}else{kind},points[seed as usize/64],seed as u8,seed.wrapping_mul(13)as u8,||{let r=seed.wrapping_mul(7919)^calls*2027^0x7731a5c9;calls+=1;r});let mut eh=2166136261u32;
   for a in &actions{match *a{Action::Chunk{kind,position,source35}=>hash(&mut eh,&[u32::from(kind),position[0]as u32,position[1]as u32,u32::from(source35)]),Action::Spawn(s)=>hash(&mut eh,&[0,s.position[0]as u32,s.position[1]as u32,u32::from(s.template),if s.arg4==255{u32::MAX}else{u32::from(s.arg4)},u32::from(s.arg5),i32::from(s.arg6)as u32,i32::from(s.arg7)as u32,i32::from(s.arg8)as u32,u32::from(s.heading),i32::from(s.delay)as u32,u32::from(s.arg11)])}}
   hash(&mut total,&[kind as u32*256+seed,calls,actions.len()as u32,eh]);
  }}assert_eq!(total,2843692805,"ZNativeAmbientEmitterOracle:26dispatch states×256 headings/positions");
 }
 struct TestWorld{calls:u32,chunks:usize}
 impl World for TestWorld{fn random(&mut self)->u32{self.calls+=1;0}fn chunk(&mut self,_:u8,_:[i16;2],_:u8){self.chunks+=1;}}
 #[test]fn a_retired_source_slot_can_be_reused_without_losing_cached_emitter_arguments(){let mut pool=ambient::State::default();pool.records[0].set_word(18,5);pool.records[0].set_word(0,-1);let mut world=TestWorld{calls:0,chunks:0};emit(&mut pool,0,[346,420],128,&mut world);assert_eq!(pool.records.iter().filter(|r|r.active()).count(),4);assert_eq!(pool.records[0].position(),[346,404]);assert_eq!(pool.records[3].position(),[338,420]);assert_eq!(world.calls,0);assert_eq!(world.chunks,0);}
}
