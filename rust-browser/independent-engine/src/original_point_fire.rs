//! Native point-fire handlers20/21/73/74. Point orders are distinct from
//! live UNIT-target fire states; they retain native timer signs and endings.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Shot{Bullet,Flame,Laser,Rocket,Grenade}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{Wait,Fire(Shot),Idle,SyncAndIdle}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct State{pub phase:u8,pub class:u8,pub frame:i16,pub timer:i8,pub cooldown:u8,pub rate:u8,pub target46:i16}
impl State{
 ///44af30/44b020/44b080/44b0e0. Cooldown preamble is external; only a shot
 ///writes100âˆ’rate, including when the downstream projectile pool is full.
 pub fn advance(&mut self)->Action{
  self.timer=self.timer.wrapping_sub(1);
  if self.phase==66{
   if self.timer!=0{return Action::Wait;}self.frame=self.frame.wrapping_add(1);self.timer=3;
   if self.frame==3{self.cooldown=200u8.wrapping_sub(self.rate.wrapping_mul(2));self.target46=-1;return Action::Fire(Shot::Grenade);}
   return if self.frame>2{Action::Idle}else{Action::Wait};
  }
  if self.phase==20{
   if self.timer!=0{return Action::Wait;}self.frame=self.frame.wrapping_add(1);self.timer=3;
   if self.frame>4{return Action::Idle;}if self.frame!=4{return Action::Wait;}
   if self.cooldown!=0{self.frame=3;return Action::Wait;}
   self.cooldown=100u8.wrapping_sub(self.rate);return Action::Fire(match self.class{15=>Shot::Flame,16=>Shot::Laser,_=>Shot::Bullet});
  }
  if self.timer>=0{return Action::Wait;}self.frame=self.frame.wrapping_add(1);
  if self.frame==2{self.timer=2;return Action::Wait;}if self.frame>2{return Action::SyncAndIdle;}
  self.timer=0;if self.cooldown!=0{self.frame=self.frame.wrapping_sub(1);return Action::Wait;}
  self.cooldown=100u8.wrapping_sub(self.rate);
  Action::Fire(match self.phase{73=>{self.target46=-1;Shot::Flame},74=>{self.target46=-1;Shot::Laser},_=>Shot::Rocket})
 }
}
///52a40 plus the two additional23/46 exclusions of the point initializers.
pub fn protected(phase:u8)->bool{matches!(phase,23|25|26|27|28|29|30|31|34|38|41|46)}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct FlameTarget{pub position:[i16;2],pub class:u8}
#[derive(Clone,Copy,Debug)]pub struct FlameInput{pub position:[i16;2],pub heading:u8,pub point:[i16;2],pub target46:i16,pub aim:u8,pub flags69:u8,pub counter97:u8,pub speed:i8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct FlamePlan{pub origin:[i32;2],pub target:[i32;2],pub heading:u8,pub speed:i8,pub target46:i16}
///44ad40â†’56840 for a non-BUILD point order (50!=3456). The old body
///heading chooses the muzzle BEFORE448c20 can change39/42/44. A nonnegative
///46 resolves its LOW BYTE as a raw UNIT slot; no live/team/phase gate applies.
///56840 restores the full46 word after448c20 temporarily writesFFFF.
pub fn point_flame(input:FlameInput,mut lookup:impl FnMut(u8)->Option<FlameTarget>,mut random:impl FnMut()->u32)->FlamePlan{
 let muzzle=crate::native_weapon_data::FLAME_MUZZLE[usize::from(input.heading&7)];let origin=std::array::from_fn(|axis|i32::from(input.position[axis].wrapping_add(muzzle[axis]as i16).wrapping_sub(8)as u16));let mut point=input.point;let mut heading=input.heading;
 if input.target46>=0{if let Some(target)=lookup(input.target46 as u8){point=target.position;if target.class==2{point[0]=point[0].wrapping_add(8);}let effective=if input.flags69&128!=0{input.aim}else{input.aim.wrapping_add(input.aim>>2).min(127)};let mut amplitude=((128-i32::from(effective))>>2)as i16;if input.counter97!=0{amplitude=amplitude.wrapping_mul(2);}
  let dx=i32::from(input.position[0])-i32::from(point[0]);let dy=i32::from(input.position[1])-i32::from(point[1]);let squared=dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));let scaled=squared.wrapping_add((squared>>31)&63)>>6;let distance=(f64::from(scaled as f32).sqrt()as f32)as i32;
  if distance>6{let amplitude=if amplitude==0{1}else{amplitude};let divisor=65535/i32::from(amplitude);let divisor=if divisor==0{1}else{divisor};let radius=(random()&65535)as i32/divisor;let angle=(random()&511)as i32;point[0]=point[0].wrapping_add(((crate::native_math::sine(angle)*radius)>>10)as i16);point[1]=point[1].wrapping_add(((crate::native_math::sine(angle+128)*radius)>>10)as i16);}
  heading=crate::native_math::direction(i32::from(point[0])-i32::from(input.position[0]),i32::from(point[1])-i32::from(input.position[1]));
 }}
 FlamePlan{origin,target:[i32::from(point[0]as u16),i32::from(point[1]as u16)+12],heading,speed:input.speed,target46:input.target46}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum GroundQuery{LiveBridge,DestroyedBridge,LiveBuilding,Ruin,Rejected{reduced_range:bool},Destructible}
///495740's26 positions. The random probe precedes this fixed traversal;
///nonzero fallback offsets temporarily decrement74 (Pyro/Tough) or W5.
pub const GROUND_OFFSETS:[[i16;2];26]=[[-16,-16],[-8,-16],[0,-16],[8,-16],[16,-16],[24,-16],[-16,-8],[-8,-8],[0,-8],[8,-8],[16,-8],[24,-8],[-16,0],[-8,0],[0,0],[8,0],[16,0],[24,0],[-16,8],[-8,8],[0,8],[8,8],[16,8],[24,8],[-16,16],[-8,16]];
///448750's query order. Live bridges/BUILDs accept the original ground
///point; destroyed bridges/ruins reject it. Returned BUILD shape adjustments
///are deliberately not used by this caller. No browser rectangle is queried.
pub fn resolve_ground_point(point:[i16;2],flags69:u8,mut query:impl FnMut(GroundQuery,[i16;2])->bool,mut random:impl FnMut()->u32)->Option<[i16;2]>{
 if flags69&32!=0||query(GroundQuery::LiveBridge,point){return Some(point);}if query(GroundQuery::DestroyedBridge,point){return None;}if query(GroundQuery::LiveBuilding,point){return Some(point);}if query(GroundQuery::Ruin,point){return None;}
 if !query(GroundQuery::Rejected{reduced_range:false},point){return Some(point);}let offset=GROUND_OFFSETS[((random()&65535)%26)as usize];let candidate=std::array::from_fn(|a|point[a].wrapping_add(offset[a]));if !query(GroundQuery::Rejected{reduced_range:false},candidate){return Some(candidate);}
 for offset in GROUND_OFFSETS{let candidate=std::array::from_fn(|a|point[a].wrapping_add(offset[a]));if !query(GroundQuery::Rejected{reduced_range:offset!=[0,0]},candidate){return Some(candidate);}}
 query(GroundQuery::Destructible,point).then_some(point)
}
#[derive(Clone,Copy,Debug)]pub struct GroundObject{pub position:[i16;2],pub definition:u16,pub flags18:u8}
///4485a0 with4d028f=8: five samples against an inclusive16x16 fire square.
///Only the seven exact fire templates71..77 qualify; runtimeflags13 do not.
pub fn fire_at_point(objects:&[GroundObject],point:[i16;2])->bool{
 objects.iter().take(200).any(|o|o.position[0]!=-1&&(71..=77).contains(&o.definition)&&[[0,0],[8,0],[0,8],[8,8],[4,4]].into_iter().any(|d|(0..2).all(|a|{let p=point[a].wrapping_add(d[a]);o.position[a]<=p&&p<=o.position[a].wrapping_add(16)})))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct BuildFlamePlan{pub shot:FlamePlan,pub amplitude:i16}
///56840's50==3456 branch. Its origin is raw54/56; selected targetY gets
///no+12. Unlike448c20, aim promotion uses signed SAR and44ab00 compares
///the original ground point with the selected point, not the shooter's XY.
pub fn build_flame(input:FlameInput,mut query:impl FnMut(GroundQuery,[i16;2])->bool,mut random:impl FnMut()->u32)->Option<BuildFlamePlan>{
 let mut target=resolve_ground_point(input.point,input.flags69,&mut query,&mut random)?;let effective=if input.flags69&128!=0{i32::from(input.aim as i8)}else{i32::from((input.aim as i8).wrapping_add((input.aim as i8)>>2)as u8).min(127)};let amplitude=((128-effective)>>1)as i16;
 scatter_ground_point(input.point,&mut target,amplitude,&mut random);
 Some(BuildFlamePlan{shot:FlamePlan{origin:input.point.map(|n|i32::from(n as u16)),target:target.map(|n|i32::from(n as u16)),heading:input.heading,speed:input.speed,target46:input.target46},amplitude})
}
///44ab00's signed16 radius and native float distance threshold.
pub fn scatter_ground_point(origin:[i16;2],target:&mut[i16;2],amplitude:i16,mut random:impl FnMut()->u32){
 let dx=i32::from(origin[0])-i32::from(target[0]);let dy=i32::from(origin[1])-i32::from(target[1]);let squared=dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));let scaled=squared.wrapping_add((squared>>31)&63)>>6;if(f64::from(scaled as f32).sqrt()as f32)as i32<=6{return;}let divisor=65535/i32::from(if amplitude==0{1}else{amplitude});let radius=(random()&65535)as i32/if divisor==0{1}else{divisor};let angle=(random()&511)as i32;target[0]=target[0].wrapping_add(((crate::native_math::sine(angle)*radius)>>10)as i16);target[1]=target[1].wrapping_add(((crate::native_math::sine(angle+128)*radius)>>10)as i16);
}
#[derive(Clone,Copy,Debug)]pub struct Member{pub slot:u8,pub live:bool,pub phase:u8,pub frame:i16,pub timer:i8,pub heading:u8,pub point:[i16;2],pub grenades:u8,pub group_grenades:bool,pub touched:bool}
///53b80/53ac0/52dd0/52e90/52fe0. A grenade candidate stops the remaining
///roster; a Tough order examines only the first nonFF slot, even ifprotected.
///411b90's group grenade sum and59af0's heading are explicit inputs.
pub fn begin(class:u8,role:i8,flags9c:&mut u8,point:[i16;2],members:&mut[Member],mut heading:impl FnMut(u8)->u8)->bool{
 if role==0{return false;}let override_grenades=if matches!(class,3|15|16){false}else{let b=*flags9c&8!=0;*flags9c&=!8;b};
 for m in members.iter_mut().filter(|m|m.slot!=255){
  if class==3{if !protected(m.phase){m.touched=true;m.point=point;m.heading=heading(m.slot);if m.phase!=21{m.phase=21;m.frame=0;m.timer=1;}}break;}
  if protected(m.phase)||!matches!(class,15|16)&&!m.live{continue;}m.touched=true;m.point=point;let previous=m.heading;m.heading=heading(m.slot);
  if !override_grenades&&m.group_grenades{if m.grenades!=0{m.phase=66;m.frame=0;m.timer=3;break;}if !matches!(class,15|16){m.heading=previous;}continue;}
  let phase=match class{15=>73,16=>74,_=>20};if m.phase!=phase{m.phase=phase;m.frame=if phase==20{3}else{0};m.timer=1;}
 }true
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn all_512_native_group_point_initializers_match_phase_guards_roster_order_and_grenade_selection(){
  let mut hash=2166136261u32;let mut count=0;
  for class in[0,3,15,16]{for role in[-1,0]{for flags in[0,8]{for scenario in 0..32{
   let mut members=(0..6).map(|j|Member{slot:if(j+scenario)%7==0{255}else{j as u8},live:(j+scenario)%7!=1,phase:[1,2,20,21,73,74,23,25,46,66][(j+scenario)%10],frame:j as i16-2,timer:7,heading:2,point:[11,22],grenades:if(j+scenario)%5==0{1}else{0},group_grenades:false,touched:false}).collect::<Vec<_>>();let grenades=members.iter().filter(|m|m.slot!=255).map(|m|m.grenades).fold(0u8,u8::wrapping_add)!=0;for m in &mut members{m.group_grenades=grenades;}let mut flags9c=flags;let started=begin(class,role,&mut flags9c,[300,400],&mut members,|_|5);
   for n in[i32::from(class),i32::from(role),i32::from(flags),scenario as i32,if started{0}else{12},i32::from(flags9c)]{hash=(hash^n as u32).wrapping_mul(16777619);}
   for m in members{for n in[i32::from(m.phase),i32::from(m.frame),i32::from(m.timer),i32::from(m.heading),i32::from(m.point[0]),i32::from(m.point[1])]{hash=(hash^n as u32).wrapping_mul(16777619);}}count+=1;
  }}}}
  assert_eq!(count,512);assert_eq!(hash,2643238168,"real53b80/52a40/411b90,59af0 heading isolated at5");
 }
 #[test]fn all_7560_executed_native_point_fire_handlers_match_counters_actions_and_target_reset(){
  let mut hash=2166136261u32;let mut count=0;
  for phase in[20,21,73,74,66]{for class in[0,15,16,3]{for frame in[-1,0,1,2,3,4,5]{for timer in[-128,-1,0,1,2,3]{for cooldown in[0,1,100]{for rate in[0,80,98]{
   let mut s=State{phase,class,frame,timer,cooldown,rate,target46:123};let action=s.advance();let actions=match action{Action::Wait=>0,Action::Fire(Shot::Bullet)=>1,Action::Fire(Shot::Flame)=>2,Action::Fire(Shot::Laser)=>3,Action::Fire(Shot::Rocket)=>4,Action::Fire(Shot::Grenade)=>7,Action::Idle=>5,Action::SyncAndIdle=>65};
   for n in[i32::from(phase),i32::from(class),i32::from(frame),i32::from(timer),i32::from(cooldown),i32::from(rate),i32::from(s.frame),i32::from(s.timer),i32::from(s.cooldown),i32::from(s.target46),actions]{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}}
  assert_eq!(count,7560);assert_eq!(hash,122693836,"ZNativePointFireOracle actual five native phase handlers");
 }
 #[test]fn all_2560_native_point_flame_chains_match_retained_raw_target_muzzle_heading_scatter_and_rng(){
  let(mut hash,mut count)=(2166136261u32,0);
  for phase in[20,73]{for raw in[-32768,-123,-1,0,1,0x5500,0x5501,0x5502,239,0x55ef]{for d in 0..8u8{for aim in[0,60,120,255]{for counter in 0..2{for mp in 0..2{
   let mut state=State{phase,class:15,frame:if phase==20{3}else{0},timer:if phase==20{1}else{0},cooldown:0,rate:98,target46:raw};assert_eq!(state.advance(),Action::Fire(Shot::Flame));let mut rng=crate::original_combat::OriginalRng{words:[1,2,3,4,5],multiplayer:mp!=0};
   let plan=point_flame(FlameInput{position:[500,600],heading:d,point:[300+i16::from(d)*10,400+i16::from(counter)*3],target46:state.target46,aim,flags69:if d%2==0{0}else{128},counter97:counter,speed:13},|slot|Some(match slot{0=>FlameTarget{position:[500,600],class:15},1=>FlameTarget{position:[500+if d%2==0{40}else{400},570+i16::from(d)*7],class:0},2=>FlameTarget{position:[680,510],class:2},239=>FlameTarget{position:[1300,500],class:1},_=>Default::default()}),||rng.next_u32());
   let mut values=vec![i64::from(phase),i64::from(raw),i64::from(d),i64::from(aim),i64::from(counter),mp,i64::from(state.frame),i64::from(state.timer),i64::from(state.cooldown),i64::from(plan.target46),i64::from(plan.heading),i64::from(plan.target[0]as i16),i64::from((plan.target[1]-12)as i16)];values.extend(plan.origin.map(i64::from));values.extend(plan.target.map(i64::from));values.push(i64::from(plan.speed));values.extend(rng.words.map(i64::from));for n in values{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;
  }}}}}}
  assert_eq!((count,hash),(2560,3605930065),"real44af30/44b080â†’44ad40â†’56840â†’448c20; libm sqrt and projectile allocation are explicit boundaries");
 }
 #[test]fn all_896_native_build_flame_resolutions_match_bridges_builds_ruins_fire_and_emit_guards(){
  use crate::original_world_contact as w;
  let(mut hash,mut count)=(2166136261u32,0);
  for mode in 0..14{for flags in[0,32,128,160]{for aim in[0,60,120,255]{for mp in 0..2{for speed in[0,13]{let(x,y)=(500+mode%4,600+mode/4);let mut map=vec![0u8;16384];if[5,7,11].contains(&mode){map.fill(8);}if mode==6{map[(y>>4)as usize*128+(x>>4)as usize+1]=8;}let mut bridges=vec![];if[1,2,10,13].contains(&mode){bridges.push(w::Bridge{position:[x-8,y-8],size:[32,32],hp:0,maximum:0,flags15:if[2,13].contains(&mode){2}else{0}});if mode==10{bridges.push(w::Bridge{flags15:2,..bridges[0]});}}let mut buildings=vec![];if[3,4,11,12].contains(&mode){buildings.push(w::Building{position:[x-8,y-8],kind:4,hp:if[4,12].contains(&mode){0}else{100}});}let mut objects=vec![];if[7,11].contains(&mode){let definition=71+mode as u16%7;objects.push(GroundObject{position:[x-4,y-4],definition,flags18:w::object_metadata(definition).unwrap().1});}if mode==8{objects.push(GroundObject{position:[x-8,y-8],definition:0,flags18:17});}let mut rng=crate::original_combat::OriginalRng{words:[mode as u32+1,2,3,4,5],multiplayer:mp!=0};let plan=build_flame(FlameInput{position:[300,400],heading:(mode%8)as u8,point:[x,y],target46:123,aim,flags69:flags,counter97:0,speed},|q,p|match q{GroundQuery::LiveBridge=>bridges.iter().any(|b|w::bridge_contains(*b,p)),GroundQuery::DestroyedBridge=>bridges.iter().any(|b|b.flags15&2!=0&&w::bridge_contains(w::Bridge{flags15:0,..*b},p)),GroundQuery::LiveBuilding=>w::first_building(&buildings,p,true,false).is_some(),GroundQuery::Ruin=>w::first_building(&buildings,p,true,true).is_some(),GroundQuery::Rejected{..}=>fire_at_point(&objects,p)||crate::original_attack_click::terrain_admission(map[((p[1]>>4)as usize)*128+(p[0]>>4)as usize],15,4,0,(mode%2)as u8)==0,GroundQuery::Destructible=>objects.iter().any(|o|o.position[0]>=0&&o.flags18&16!=0&&(0..2).all(|a|{let extent=((i16::from(o.flags18 as i8)+1)*8)&24;o.position[a]<=p[a]&&i32::from(p[a])<i32::from(o.position[a])+i32::from(extent)}))},||rng.next_u32());let emitted=plan.is_some_and(|p|p.shot.origin!=p.shot.target&&p.shot.speed!=0);let mut values=vec![i64::from(mode),i64::from(flags),i64::from(aim),mp,i64::from(speed),i64::from(plan.is_some()),i64::from(emitted),plan.map_or(-999,|p|i64::from(p.amplitude))];if let Some(p)=plan{values.extend(p.shot.origin.map(i64::from));values.extend(p.shot.target.map(i64::from));values.push(i64::from(p.shot.speed));}else{values.extend([-1;5]);}values.extend([15,19,123,i64::from(mode%8)]);values.extend(rng.words.map(i64::from));for n in values{hash=(hash^n as u32).wrapping_mul(16777619);}count+=1;}}}}}
  assert_eq!((count,hash),(896,2015998565),"real56840→448750 including35BUILD/12Bridge/200OBJECT queries and464a20 guards; sqrt and downstream projectile allocation isolated");
 }

}
