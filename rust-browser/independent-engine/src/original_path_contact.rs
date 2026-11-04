//! Native51330 obstacle attacks and51700 vehicle contact continuation.
//! Coordinates, scheduler identity and failed footprint cells are explicit
//! caller inputs; route goals and command dispatch retain their own owners.
#[path="original_path_contact_game.rs"]pub mod game;
#[path="original_path_goal.rs"]pub mod goal;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Actor{pub slot:u8,pub position:[i16;2],pub family:u8,pub class:u8,pub saved_class:u8,pub role:i8,pub flags31:u8,pub flags71:u8,pub grenades:u8,pub facing:u8,pub counter67:i8,pub directiona2:i8,pub step62:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Context{pub blocked:[i16;2],pub proposed:[i16;2],pub force:bool,pub callback_idle:bool,pub current_actor:bool,pub dummy_actor:bool}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Vehicle{pub slot:u8,pub position:[i16;2],pub owner:u8}
#[derive(Clone,Debug,Default)]pub struct State{pub vehicles:Vec<Vehicle>,pub count:i8,pub donated:bool,pub goal_check_changed:bool,pub collision:Option<usize>,pub command_point:Option<[i16;2]>}
impl State{
 ///45a9f0 runs before the UNIT callbacks;51620 refreshes positions later,
 ///without refreshing membership. The signed byte count can wrap negative.
 pub fn prepare(&mut self,actors:impl IntoIterator<Item=Actor>){self.vehicles=actors.into_iter().filter(|a|a.flags31&4!=0&&a.class==1&&a.flags71&2==0).map(|a|Vehicle{slot:a.slot,position:a.position,owner:a.flags31&3}).collect();self.count=self.vehicles.len()as i8;}
}
// Exact numeric128-byte indexed predicate at49a728, including bytes following
// the logical terrain constants, because native indexing accepts all7bits.
const ATTACK_TERRAIN:[u8;128]=[0,1,1,1,1,1,1,0,0,0,0,1,1,1,1,1,0,1,1,0,0,0,0,0,1,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,2,0,255,255,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,255,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,255,255,255,255,0,0,0,0,255,255,0,0,255,255,0,255];
pub fn attacks_terrain(code:u8)->bool{code&127!=17&&ATTACK_TERRAIN[usize::from(code&127)]!=0}
pub trait AttackWorld{
 type Error;
 fn donor(&mut self)->Result<Option<u8>,Self::Error>;
 fn donate(&mut self,slot:u8);
 fn terrain(&mut self,cell:i16)->Result<u8,Self::Error>;
 fn goal(&mut self)->Result<[i16;2],Self::Error>;
 fn random(&mut self)->u32;
 fn attack(&mut self,point:[i16;2]);
}
pub fn auto_attack<W:AttackWorld>(actor:&mut Actor,state:&mut State,context:Context,world:&mut W)->Result<bool,W::Error>{
 state.donated=false;
 if !context.force&&actor.role>=0{return Ok(false);}
 if matches!(actor.class,0|15|16){if actor.role>=0{return Ok(false);}if actor.grenades==0{let Some(donor)=world.donor()?else{return Ok(false);};state.donated=true;world.donate(donor);actor.grenades=actor.grenades.wrapping_add(1);}}
 if matches!(actor.family,34|36|38)||actor.family==32&&actor.saved_class!=3{return Ok(false);}
 let cell=if context.blocked[0]!= -1{context.blocked[0]}else if context.blocked[1]!= -1{context.blocked[1]}else{return Ok(false);};
 if !attacks_terrain(world.terrain(cell)?){return Ok(false);}
 let anchor=[((cell as u16&127)*16)as i16,((cell as u16>>3)&0xfff0)as i16];
 if actor.flags71&1==0&&actor.flags31&32==0&&(context.callback_idle||!context.current_actor)&&actor.class!=3&&!context.force&&!matches!(actor.family,6|30|25|24|32)&&!matches!(actor.class,0|15|16){let goal=world.goal()?;let dx=i32::from(anchor[0])-i32::from(goal[0]);let dy=i32::from(anchor[1])-i32::from(goal[1]);if dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))>4096{return Ok(false);}}
 let x=anchor[0].wrapping_add((world.random()as u16&15)as i16).wrapping_sub(8);let y=anchor[1].wrapping_add((world.random()as u16&15)as i16).wrapping_sub(8);let point=[x,y];state.command_point=Some(point);world.attack(point);Ok(true)
}
///51530 checks linear neighbors, deliberately allowing row wrap.
pub fn touches_66(cell:i16,mut terrain:impl FnMut(i16)->u8)->bool{if terrain(cell)&127==66{return true;}if cell>=1{if terrain(cell-1)&127==66{return true;}if cell>=127{if terrain(cell-127)&127==66{return true;}if cell>=128{if terrain(cell-128)&127==66{return true;}if cell>=129&&terrain(cell-129)&127==66{return true;}}}}cell<16383&&terrain(cell.wrapping_add(1))&127==66}
pub fn facing_toward(position:[i16;2],proposed:[i16;2],previous:u8)->u8{match(proposed[0].cmp(&position[0]),proposed[1].cmp(&position[1])){(std::cmp::Ordering::Equal,std::cmp::Ordering::Equal)=>previous,(std::cmp::Ordering::Equal,std::cmp::Ordering::Less)=>0,(std::cmp::Ordering::Equal,std::cmp::Ordering::Greater)=>4,(std::cmp::Ordering::Greater,std::cmp::Ordering::Equal)=>2,(std::cmp::Ordering::Greater,std::cmp::Ordering::Less)=>1,(std::cmp::Ordering::Greater,std::cmp::Ordering::Greater)=>3,(std::cmp::Ordering::Less,std::cmp::Ordering::Equal)=>6,(std::cmp::Ordering::Less,std::cmp::Ordering::Less)=>7,(std::cmp::Ordering::Less,std::cmp::Ordering::Greater)=>5}}
pub trait VehicleWorld{fn terrain(&mut self,cell:i16)->u8;fn vehicle(&mut self,slot:u8)->([i16;2],u8);}
pub fn vehicle_goal(actor:&mut Actor,state:&mut State,context:&mut Context,world:&mut impl VehicleWorld)->bool{
 for cell in context.blocked{if cell>=0&&touches_66(cell,|c|world.terrain(c)){return true;}}
 if actor.counter67!=0{actor.counter67=actor.counter67.wrapping_sub(actor.step62 as i8);if actor.counter67<=0{return true;}}else{if actor.directiona2== -1{actor.directiona2=actor.facing as i8;}actor.counter67=if actor.family==34{16}else{8};}
 for i in 0..2{if context.blocked[i]>=0{context.proposed[i]=actor.position[i];}}
 if !context.dummy_actor&&state.count>0{
  for record in state.vehicles.iter_mut().take(state.count as usize){let(p,owner)=world.vehicle(record.slot);record.position=p;record.owner=owner;}
  for(i,record)in state.vehicles.iter().take(state.count as usize).enumerate(){if record.slot==actor.slot||record.owner&3==actor.flags31&3{continue;}let near=(0..2).all(|d|((i32::from(record.position[d])-i32::from(context.proposed[d])).abs()as i16)<32);if near{state.collision=Some(i);context.proposed=actor.position;return true;}}
 }
 actor.facing=facing_toward(actor.position,context.proposed,actor.facing);state.goal_check_changed=true;false
}
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,n:i64){*h=(*h^n as u32).wrapping_mul(16777619);}
 fn first(s:usize)->i16{[520,127,128,16383,-1,1000][s%6]}fn second(s:usize)->i16{if s%11==0{-1}else{640+(s%128)as i16}}
 struct Attack{scenario:usize,inventory:[u8;21],rng:u32,goals:u32,command:Option<[i16;2]>}
 impl AttackWorld for Attack{type Error=();fn donor(&mut self)->Result<Option<u8>,()>{Ok((0..21).find(|&i|(i==0||(self.scenario+i)%7!=0)&&self.inventory[i]!=0).map(|i|i as u8))}fn donate(&mut self,s:u8){self.inventory[usize::from(s)]=self.inventory[usize::from(s)].wrapping_sub(1);}fn terrain(&mut self,_:i16)->Result<u8,()>{Ok((self.scenario/5%128)as u8|(self.scenario%2*128)as u8)}fn goal(&mut self)->Result<[i16;2],()>{self.goals+=1;let s=self.scenario;let c=if first(s)!= -1{first(s)}else{second(s)};Ok([(((c as u16&127)*16)as i16).wrapping_add([0,64,65,-64,-65,32767][s/11%6]),((c as u16>>3)&0xfff0)as i16])}fn random(&mut self)->u32{let r=(self.scenario as u32*7919)^(self.rng*2027)^0x7731a5c9;self.rng+=1;r}fn attack(&mut self,p:[i16;2]){self.command=Some(p);}}
 #[test]fn obstacle_contact_matches8192_original_calls_and_direct_opcode12_writes(){let mut hash=2166136261u32;for s in 0..8192usize{let mut a=Actor{slot:0,role:[-1,0,1][s/72%3],class:[0,1,2,3,15,16,17,18][s%8],family:[4,6,30,25,24,32,34,36,38][s/8%9],saved_class:[0,3,15][s/13%3],flags71:(s/432%2)as u8,flags31:4+(s/864%2*32)as u8,grenades:if s%3==0{0}else{2},..Default::default()};let mut w=Attack{scenario:s,inventory:std::array::from_fn(|i|if i==0{a.grenades}else if s%17!=0&&(s+i)%5==0{3}else{0}),rng:0,goals:0,command:None};let mut state=State::default();let c=Context{blocked:[first(s),second(s)],proposed:[0;2],force:s/216%2!=0,callback_idle:s/29%2==0,current_actor:s/31%2==0,dummy_actor:false};let success=auto_attack(&mut a,&mut state,c,&mut w).unwrap();w.inventory[0]=a.grenades;let mut inv=2166136261;for n in w.inventory{mix(&mut inv,i64::from(n));}let mut flags69=(s*7)as u8;let mut flags9c=(s*11)as u8;let mut counter=(s*13)as u8;let mut opcode=77;let mut point=[0x1234abcd,-1900];let mut metadata=2900;if let Some(p)=w.command{opcode=12;point=[i32::from(p[0]),i32::from(p[1])];metadata=(s as i32*19-30000)as i16 as i32;flags69&=!24;flags9c|=32;counter=0;}for n in[s as i64,success as i64,i64::from(w.rng),i64::from(w.goals),state.donated as i64,i64::from(inv),opcode,i64::from(point[0]),i64::from(point[1]),i64::from(metadata),i64::from(flags69),i64::from(flags9c),i64::from(counter)]{mix(&mut hash,n);}}
 assert_eq!(hash,1327429390,"51330 with native449dc0 and45ef00→45f070; only RNG and43bdd0 isolated");}
 struct Vehicles{terrain:Vec<u8>,actual:Vec<([i16;2],u8)>}impl VehicleWorld for Vehicles{fn terrain(&mut self,c:i16)->u8{self.terrain.get(c as u16 as usize).copied().unwrap_or(0)}fn vehicle(&mut self,s:u8)->([i16;2],u8){self.actual[usize::from(s)]}}
 #[test]fn vehicle_goal_matches4096_complete_original_calls_including_signed_byte_and_distance_wrap(){let mut hash=2166136261;for s in 0..4096usize{let blocked0=[-1,0,1,126,127,128,129,16383][s%8];let blocked1=if s%3==0{-1}else{640+(s%128)as i16};let marker=[-999,0,-1,-127,-128,-129,1][s/8%7];let mut terrain=vec![0;16384];if marker!= -999&&(0..16384).contains(&(blocked0+marker)){terrain[(blocked0+marker)as usize]=66|(s%2*128)as u8;}let p=[(s as i32*353-32000)as i16,(s as i32*229+30000)as i16];let target=[p[0].wrapping_add([0,31,32,33,32767][s/17%5]),p[1].wrapping_add([0,-31,-32,-33,-32768][s/19%5])];let count=[0,1,4,127,128,240][s/31%6];let mut a=Actor{slot:0,position:p,flags31:(s%4)as u8,facing:(s*7)as u8,family:if s%5==0{34}else{6},counter67:[0,1,8,127,-128,-1][s/23%6],step62:[0,1,2,127,128,255][s/29%6],directiona2:if s%3==0{-1}else{(s*11)as i8},..Default::default()};let mut w=Vehicles{terrain,actual:(0..count).map(|i|if i==0{(p,(s%4)as u8)}else{([target[0].wrapping_add(if i%2==0{31}else{32}),target[1].wrapping_add(if i%3==0{-31}else{0})],((s+i)%4)as u8)}).collect()};let mut state=State{count:count as i8,vehicles:(0..count).map(|i|Vehicle{slot:i as u8,position:[-777,-888],owner:0}).collect(),..Default::default()};let mut c=Context{blocked:[blocked0,blocked1],proposed:target,force:false,callback_idle:false,current_actor:false,dummy_actor:s%37==0};let stop=vehicle_goal(&mut a,&mut state,&mut c,&mut w);let mut cache=2166136261;for v in state.vehicles{for n in v.position{mix(&mut cache,i64::from(n));}}for n in[s as i64,stop as i64,i64::from(a.counter67),i64::from(a.directiona2),i64::from(a.facing),i64::from(c.proposed[0]),i64::from(c.proposed[1]),state.goal_check_changed as i64,state.collision.map_or(-1,|n|n as i64),i64::from(cache)]{mix(&mut hash,n);}}
 assert_eq!(hash,2090520412,"full51700/51530/51620/515b0, no stubs");}
}
