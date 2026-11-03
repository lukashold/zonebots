//!Separate500x6 track/exhaust cosmetic pool5a11a6.4616f0 consumes the CRT
//!stream independently of deterministic gameplay RNG. NegativeY is encoded.
use super::{Record,State as Smoke};
#[derive(Clone,Copy)]pub struct Input{pub position:[i16;2],pub family:u8,pub direction:u8,pub frame:i16,pub flags31:u8,pub flags71:u8,pub hp:i16,pub maximum:i16,pub multiplayer:bool}
pub trait World{fn random(&mut self)->u16;fn terrain(&self,cell:i16)->u8;}
pub struct State{pub records:Vec<Record>,pub draws:Vec<Draw>}
#[derive(Clone,Copy)]pub struct Draw{pub slot:usize,pub position:[i16;2],pub sprite:i16,pub queued:bool}
impl Default for State{fn default()->Self{Self{records:vec![Record::default();500],draws:vec![]}}}
fn cell(p:[i16;2])->i16{((p[1]as u16&0xfff0).wrapping_mul(8)).wrapping_add((p[0]>>4)as u16)as i16}
fn random(input:Input,w:&mut impl World)->u16{if input.multiplayer{0}else{w.random()}}
fn free(pool:&[Record])->Option<usize>{pool.iter().position(|r|r.position[0]==-1)}
fn keep(r:&mut Record,w:&impl World){let code=w.terrain(cell(r.position.map(|n|n.wrapping_add(4))))&127;if !matches!(code,0|8|22|35|36|45|53|54|55){r.position[0]=-1;}}
fn table_point(input:Input,t:&[i16;40],side:usize)->[i16;2]{let i=usize::from(input.direction&7)*5;[input.position[0].wrapping_add(t[i+1+side*2]),input.position[1].wrapping_add(t[i+2+side*2])]}
fn pair(pool:&mut[Record],input:Input,t:&[i16;40],wheel:bool,w:&mut impl World){
 let kind=t[usize::from(input.direction&7)*5]as u8|if wheel{64}else{0};let Some(slot)=free(pool)else{return;};pool[slot].kind=kind;
 if random(input,w)&3!=0{pool[slot].position=table_point(input,t,0);pool[slot].frame=50;keep(&mut pool[slot],w);}
 let Some(slot)=free(pool)else{return;};pool[slot].frame=50;pool[slot].kind=kind;if random(input,w)&3!=0{pool[slot].position=table_point(input,t,1);keep(&mut pool[slot],w);}
}
fn exhaust(pool:&mut[Record],input:Input,t:&[i16;40],w:&mut impl World){
 let Some(slot)=free(pool)else{return;};let kind=t[usize::from(input.direction&7)*5]as u8;pool[slot].kind=kind;
 if random(input,w)&1!=0{pool[slot].position=table_point(input,t,0);pool[slot].frame=(random(input,w)&128)as u8;keep(&mut pool[slot],w);pool[slot].position[1]=pool[slot].position[1].wrapping_neg();}
 if random(input,w)&1!=0{let Some(slot)=free(pool)else{return;};pool[slot].kind=kind;pool[slot].position=table_point(input,t,1);pool[slot].frame=(random(input,w)&128)as u8;keep(&mut pool[slot],w);pool[slot].position[1]=pool[slot].position[1].wrapping_neg();}
}
fn muzzle(smoke:&mut[Record;20],input:Input,table:&[i8;32],side:usize,directional:bool,w:&impl World){
 let base=cell(input.position);if[0,1,128,129].iter().any(|&d|matches!(w.terrain(base.wrapping_add(d))&127,20|21|30|60|61)){return;}
 let Some(slot)=free(smoke)else{return;};let offset=usize::from(input.direction&7)*4+side*2;smoke[slot]=Record{position:[input.position[0].wrapping_add(i16::from(table[offset])),input.position[1].wrapping_add(i16::from(table[offset+1]))],frame:0,kind:input.direction|if directional{128}else{0}};
}
fn family_index(f:u8)->usize{match f{24=>0,25=>1,30=>2,32=>3,34=>4,36=>5,38=>6,_=>7}}
fn damage(pool:&mut[Record],smoke:&mut[Record;20],input:Input,severity:u8,w:&mut impl World){
 let t=&MUZZLES[family_index(input.family)];let offset=match input.family{24|32|36=>[10,10],30=>[7,12],34|38=>[8,8],_=>[12,12]};
 if severity>=2&&random(input,w)&if severity==3{15}else{7}==0{if let Some(slot)=free(pool){let n=random(input,w)as u8;pool[slot]=Record{position:[input.position[0].wrapping_add(offset[0]),input.position[1].wrapping_add(offset[1])],frame:124,kind:((n&2|1).wrapping_sub(1))|128};}}
 if severity==3&&random(input,w)&3==0{let side=if random(input,w)&1!=0{0}else{1};if side==0||input.family!=34{muzzle(smoke,input,t,side,true,w);}}
 if random(input,w)&3==0{let side=if random(input,w)&1!=0{0}else{1};if side==0||input.family!=34{muzzle(smoke,input,t,side,false,w);}}
}
pub fn moved(pool:&mut State,smoke:&mut[Record;20],input:Input,w:&mut impl World)->i16{
 if input.flags31&64!=0||input.flags71&1!=0{return input.frame;}let index=family_index(input.family);
 if random(input,w)&1!=0{exhaust(&mut pool.records,input,&EXHAUST[index],w);}
 let forward=matches!(input.family,24|32|36)||!matches!(input.direction as i8,0|1|2|7);let frame;
 if matches!(input.family,34|38){frame=input.frame.wrapping_add(1)&3;}
 else if forward{frame=input.frame.wrapping_add(1);if frame!=3{return frame;}}
 else{frame=input.frame.wrapping_sub(1);if frame>=0{return frame;}}
 let frame=if matches!(input.family,34|38){frame}else if forward{0}else{2};
 if input.family==34&&frame==0{muzzle(smoke,input,&MUZZLES[4],0,false,w);}
 pair(&mut pool.records,input,&TRACKS[index],matches!(input.family,34|38),w);
 let hp=i32::from(input.hp);let max=i32::from(input.maximum);let severity=if hp<max/5{3}else if hp<max*2/5{2}else if hp<max*4/5{1}else{0};if severity!=0{damage(&mut pool.records,smoke,input,severity,w);}frame
}
impl State{
 pub fn clear_for_mission(&mut self){for r in &mut self.records{r.position[0]=-1;}self.draws.clear();}
 pub fn bytes(&self)->Vec<u8>{let mut out=Vec::with_capacity(3000);for r in &self.records{out.extend(r.position[0].to_le_bytes());out.extend(r.position[1].to_le_bytes());out.extend([r.frame,r.kind]);}out}
 pub fn advance(&mut self,camera:[i16;2],size:[i16;2],native_world:u8,pulse:bool){
  self.draws.clear();let p=PROFILES[usize::from(native_world.min(4))];
  for(slot,r)in self.records.iter_mut().enumerate(){if r.position[0]==-1{continue;}let x=r.position[0].wrapping_sub(camera[0]);let negative=r.position[1]<0;let y=if negative{r.position[1].wrapping_add(camera[1]).wrapping_neg()}else{r.position[1].wrapping_sub(camera[1])};let visible=x>-8&&x<size[0]&&y>-8&&y<size[1];
   let sprite=if negative{let (base,frame)=if r.frame&128!=0&&p[3]!=-1{(p[3],i16::from(r.frame&127))}else{(p[2],i16::from(r.frame as i8))};(base!=-1).then_some(base.wrapping_add(frame))}
   else if r.kind&128!=0{Some(1836+i16::from(r.kind&127)+if (r.frame as i8)<31{2}else if(r.frame as i8)<62{1}else{0})}
   else{let base=if r.kind&64!=0{p[1]}else{p[0]};let frame=(r.frame as i8>>2).min(2);(base!=-1).then_some(base.wrapping_add(i16::from(if r.kind&64!=0{r.kind&31}else{r.kind}as i8)).wrapping_sub(i16::from(frame)).wrapping_add(2))};
   if visible{if let Some(sprite)=sprite{self.draws.push(Draw{slot,position:[x.wrapping_add(camera[0]),y.wrapping_add(camera[1])],sprite,queued:negative});}}
   if !pulse{continue;}if negative{let d=NEGATIVE_MOVEMENT[usize::from(r.kind&7)];r.position[0]=r.position[0].wrapping_add(d[0]);r.position[1]=r.position[1].wrapping_sub(d[1]);let next=(r.frame&127).wrapping_add(1);if next==if p[2]==1330{6}else{5}{r.position[0]=-1;}else{r.frame=(r.frame&128).wrapping_add(next);}}
   else{r.frame=r.frame.wrapping_sub(1);if if r.kind&128!=0{r.frame==0}else{(r.frame as i8)<0}{r.position[0]=-1;}}
  }
 }
}
const NEGATIVE_MOVEMENT:[[i16;2];8]=[[0,0],[-1,1],[-1,0],[-1,-1],[0,-1],[1,-1],[1,-1],[1,0]];
const PROFILES:[[i16;4];5]=[[1496,1546,1816,1821],[1520,-1,1300,1305],[-1,-1,-1,-1],[1532,-1,1320,1325],[1508,-1,1330,1330]];
#[cfg(test)]mod tests{
 use super::*;fn hash(b:&[u8])->u32{b.iter().fold(2166136261u32,|h,&b|(h^u32::from(b)).wrapping_mul(16777619))}
 struct Env{state:u32,calls:u32,cell:i16,code:u8}impl World for Env{fn random(&mut self)->u16{self.state=self.state.wrapping_mul(214013).wrapping_add(2531011);self.calls+=1;((self.state>>16)&32767)as u16}fn terrain(&self,cell:i16)->u8{if cell==self.cell{self.code}else{0}}}
 #[test]fn all_4096_native_movement_calls_match_both_pools_frame_and_crt_consumption(){let mut all=2166136261u32;
  for s in 0..4096usize{let mut smoke=Smoke::default();let mut tracks=State::default();let free=[0,1,499,500][s/256%4];let sf=[0,19,20,1][s/64%4];for(i,r)in smoke.records.iter_mut().enumerate(){let b=(0..6).map(|j|(s+(i*6+j)*17)as u8).collect::<Vec<_>>();*r=Record{position:[if i<sf{100+i as i16}else{-1},i16::from_le_bytes([b[2],b[3]])],frame:b[4],kind:b[5]};}for(i,r)in tracks.records.iter_mut().enumerate(){let b=(0..6).map(|j|(s+(i*6+j)*19)as u8).collect::<Vec<_>>();*r=Record{position:[if i<free{100+i as i16}else{-1},i16::from_le_bytes([b[2],b[3]])],frame:b[4],kind:b[5]};}
   let mut w=Env{state:s as u32*7919^0x7731a5c9,calls:0,cell:2580+[0,1,128,129][s/8%4],code:[0,7,20,35,60,22,21,61][s/32%8]};let input=Input{position:[320,320],family:[24,25,30,32,34,36,38,8][s%8],direction:(s/128%8)as u8,frame:(s/8%4)as i16,flags31:if s/1024%4==1{64}else{4},flags71:if s/1024%4==2{1}else{0},hp:[0,19,20,39,40,59,60,79,80,100][s/16%10],maximum:100,multiplayer:s/2048!=0};let frame=moved(&mut tracks,&mut smoke.records,input,&mut w);for v in[s as u32,frame as u16 as u32,w.calls,w.state,hash(&smoke.bytes()),hash(&tracks.bytes())]{all=(all^v).wrapping_mul(16777619);}
  }assert_eq!(all,2635178596);
 }
 #[test]fn all_2048_native_track_callbacks_match_records_and_both_draw_paths(){let mut all=2166136261u32;for s in 0..128usize{let mut state=State::default();for i in 0..8{state.records[i]=Record{position:[[0,100,319,-9,500][s/8%5]+i as i16,if i<3{-100-i as i16}else{100+i as i16}],frame:(s*17+i*13)as u8,kind:[0,1,7,64,71,128,130,0][i]};}for tick in 0..16usize{let camera=[(s%3*12)as i16,(s%5*8)as i16];state.advance(camera,[320,200],(s%5)as u8,tick&1==0);let mut queue=Vec::new();let mut direct=Vec::new();for d in &state.draws{let dst=if d.queued{&mut queue}else{&mut direct};for n in[d.position[0].wrapping_sub(camera[0]),d.position[1].wrapping_sub(camera[1]),d.sprite,255]{dst.extend(n.to_le_bytes());}}for n in[(s*16+tick)as u32,hash(&state.bytes()),queue.len()as u32,hash(&queue),direct.len()as u32,hash(&direct)]{all=(all^n).wrapping_mul(16777619);}}}assert_eq!(all,320926999);}
}
//Numeric source anchor tables follow; each row describes eight directions.
const EXHAUST:[[i16;40];8]=[
 [0,11,12,251,12,1,-6,5,3,15,2,-9,-1,-9,10,3,-1,-12,-8,1,4,11,-4,-5,-4,5,20,0,2,-5,6,18,-1,18,10,7,20,5,9,15],
 [0,-4,11,15,11,1,-9,4,4,12,2,-9,1,-9,13,3,-1,-10,-11,1,4,-4,-7,14,-7,5,3,-2,17,2,6,20,14,20,3,7,18,5,8,14],
 [0,-4,11,15,11,1,-9,4,4,12,2,-9,1,-9,13,3,-1,-10,-11,1,4,-4,-7,14,-7,5,3,-2,17,2,6,20,14,20,3,7,18,5,8,14],
 [0,-3,15,15,15,1,-7,9,3,18,2,-8,-2,-8,11,3,2,-1,-9,4,4,-3,-3,15,-3,5,5,-5,20,3,6,20,-2,20,11,7,10,16,20,9],
 [0,-4,12,12,12,1,-6,7,2,16,2,-5,2,-9,14,3,3,7,-7,4,4,11,-7,-3,-7,5,21,3,10,-1,6,19,13,19,3,7,20,8,8,18],
 [0,-3,15,15,15,1,-7,9,3,18,2,-8,-2,-8,11,3,2,-1,-9,4,4,-3,-3,15,-3,5,5,-5,20,16,6,20,-2,20,11,7,10,16,20,9],
 [0,-3,15,15,15,1,-7,9,3,18,2,-8,-2,-8,11,3,2,-1,-9,4,4,-3,-3,15,-3,5,5,-5,20,16,6,20,-2,20,11,7,10,16,20,9],
 [0,-3,17,16,17,1,-10,7,0,17,2,-10,5,-10,15,3,3,-6,-8,3,4,-4,-7,15,-7,5,8,-12,18,-3,6,21,2,21,12,7,9,19,19,10]
];
const TRACKS:[[i16;40];8]=[
 [0,-4,13,14,13,9,-3,9,7,18,3,-4,1,-4,14,6,6,-3,-5,6,0,-4,7,14,7,9,8,-4,20,6,3,9,-1,14,14,6,5,16,16,7],
 [0,-4,15,14,15,9,-6,8,5,18,3,-4,3,-4,18,6,4,-5,-5,7,0,-6,-1,13,-1,9,5,-3,18,7,3,14,1,14,16,6,2,14,17,11],
 [0,-3,15,13,15,9,-7,7,3,16,3,-4,3,-4,18,6,0,-1,-11,8,0,-6,-1,12,-1,9,4,-6,16,4,3,14,1,14,16,6,3,20,15,11],
 [0,-3,15,13,15,9,-4,10,5,18,3,-4,4,-4,17,6,2,-3,-5,10,0,-3,-1,13,-1,9,6,-4,18,6,3,14,4,14,17,6,5,18,15,11],
 [0,-3,15,13,15,9,-3,11,7,20,3,-3,19,-3,8,6,4,3,-7,12,0,-2,1,12,1,9,12,2,21,9,3,14,8,14,19,6,14,6,14,25],
 [0,-3,15,13,15,9,-4,10,5,18,3,-4,4,-4,17,6,2,-3,-5,10,0,-3,-1,13,-1,9,6,-4,18,6,3,14,4,14,17,6,5,18,15,11],
 [0,-3,15,13,15,9,-4,10,5,18,3,-4,4,-4,17,6,2,-3,-5,10,0,-3,-1,13,-1,9,6,-4,18,6,3,14,4,14,17,6,5,18,15,11],
 [0,-4,21,15,21,9,-9,11,1,19,3,-4,5,-4,15,6,3,-4,-8,5,0,-4,1,15,1,9,8,-5,18,4,3,19,5,19,15,6,9,20,19,13]
];
const MUZZLES:[[i8;32];8]=[
 [-2,17,5,17,-15,14,-11,17,-21,-3,-21,1,-7,-12,-13,-5,-6,-15,2,-15,11,-16,19,-11,24,1,24,-3,19,12,16,15],
 [-1,18,5,18,-16,11,-11,16,-20,-3,-20,2,-16,-13,-10,-19,-1,-17,5,-17,10,-15,17,-11,24,2,24,-3,17,18,21,15],
 [-3,17,2,17,-17,10,-12,14,-24,-1,-24,4,-16,-17,-21,-12,2,-15,-3,-15,6,-12,14,-7,19,4,19,-1,9,15,12,5],
 [-2,21,4,21,-17,19,-14,22,-19,-4,-19,2,-15,-18,-20,-13,-2,-16,4,-16,11,-14,18,-10,23,2,23,-4,20,17,15,22],
 [-4,20,4,19,-18,15,-14,16,-18,0,-18,3,-11,-16,-16,-11,6,-8,4,-10,21,-8,12,-10,24,6,19,3,17,22,14,17],
 [0,19,4,19,-15,16,-14,18,-19,-2,-19,1,-12,-15,-17,-10,-2,-16,4,-16,11,-14,18,-10,23,3,23,-2,20,17,17,20],
 [0,19,4,19,-15,16,-14,18,-19,-2,-19,1,-12,-15,-17,-10,-2,-16,4,-16,11,-14,18,-10,23,3,23,-2,20,17,17,20],
 [-2,19,4,19,-18,15,-14,16,-18,-3,-18,3,-11,-16,-16,-11,-2,-10,4,-10,6,-13,12,-10,19,-3,19,3,17,15,14,17]
];
