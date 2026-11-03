//! Native strategic numerical kernels. These functions consume raw runtime
//! attributes; they do not invent those attributes from browser HP or speed.
//! Numeric coefficients: mult.dat, loaded at004a8980, selected through the
//! local GOG executable's pointer tables0047a0b8/0047a140. That destination is
//! BSS: reading its bytes directly from an EXE file is incorrect.

const TRAIT_WEIGHTS:[[i8;7];19]=[
 [5,4,10,8,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [-1,4,10,0,2,-1,0],
 [5,4,10,8,2,-1,0],
 [-1,4,10,0,2,-1,0],
 [-1,4,10,0,2,-1,0],
 [-1,4,10,0,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [0,5,10,-1,2,-1,0],
 [5,4,10,8,2,-1,0],
 [5,4,10,8,2,-1,0],
 [5,4,10,8,2,-1,0],
 [5,4,10,8,2,-1,0],
];
const WEAPON_WEIGHTS:[[i8;11];19]=[
 [-1,-1,-1,20,1,6,-1,-1,-1,-1,2],
 [6,4,10,8,2,-1,5,1,1,-1,6],
 [6,-1,3,8,14,25,-1,-1,-1,-1,3],
 [-1,-1,-1,22,1,6,-1,-1,-1,-1,2],
 [6,-1,3,8,6,25,-1,-1,-1,-1,7],
 [6,-1,3,8,10,25,-1,-1,-1,-1,3],
 [6,-1,3,8,10,25,-1,-1,-1,-1,5],
 [6,20,2,10,10,25,-1,-1,-1,-1,8],
 [6,20,2,10,10,25,5,1,1,-1,6],
 [6,20,2,10,10,25,5,1,1,-1,5],
 [-1,20,14,0,0,0,0,0,0,0,4],
 [6,20,2,10,10,25,-1,-1,-1,-1,2],
 [-1,15,10,-1,-1,-1,-1,-1,-1,-1,1],
 [-1,15,10,-1,-1,-1,-1,-1,-1,-1,1],
 [-1,20,14,-1,-1,-1,-1,-1,-1,-1,1],
 [-1,-1,-1,20,1,6,-1,-1,-1,-1,2],
 [-1,-1,-1,20,1,6,-1,-1,-1,-1,3],
 [-1,-1,-1,20,1,6,-1,-1,-1,-1,4],
 [-1,-1,-1,25,1,6,-1,-1,-1,-1,4],
];

/// Fields are in the native scoring vector order. For hardware this is:
/// range*8, movement, hull HP, primary damage/rate/speed, secondary range*8,
/// secondary damage/rate/speed. All except HP are unsigned bytes before scaling.
#[derive(Clone,Copy,Debug,Default)]
pub struct Hardware{
 pub range:u8,pub movement:u8,pub hp:i16,
 pub damage:u8,pub fire_rate:u8,pub projectile_speed:u8,
 pub secondary_range:u8,pub secondary_damage:u8,pub secondary_rate:u8,pub secondary_speed:u8,
}
impl Hardware{
 fn vector(self)->[i32;10]{[i32::from(self.range)*8,i32::from(self.movement),i32::from(self.hp),i32::from(self.damage),i32::from(self.fire_rate),i32::from(self.projectile_speed),i32::from(self.secondary_range)*8,i32::from(self.secondary_damage),i32::from(self.secondary_rate),i32::from(self.secondary_speed)]}
}
#[derive(Clone,Copy,Debug,Default)]
pub struct UnitInput{
 pub profile:u8,pub runtime_class:u8,
 /// Signed bytes at native unit+[00,02,04,07,3a,3e,7a].
 pub traits:[i8;7],
 pub grenades:i8,
 /// Infantry unit+65 is signed; rate/speed/range bytes are unsigned.
 pub damage:i8,pub fire_rate:u8,pub projectile_speed:u8,pub range:u8,
 pub hardware:Hardware,
}
/// Initial ordinary unit attributes from robots.dat and0041b210/0041ba00/
///0041c090. A live adapter must update aim, crew HP, hardware HP/rate/movement
///and grenade count from the actual runtime state. Trait[3] remains the native
///robot movement byte; hardware movement belongs in hardware.movement instead.
pub fn default_input(kind:&str)->Option<UnitInput>{Some(match kind{
  "cannon"=>UnitInput{profile:2,runtime_class:2,traits:[0,60,30,40,30,60,0],grenades:0,damage:75,fire_rate:60,projectile_speed:10,range:15,hardware:Hardware{range:15,movement:0,hp:100,damage:75,fire_rate:60,projectile_speed:10,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "medium"=>UnitInput{profile:1,runtime_class:1,traits:[0,80,30,0,30,80,0],grenades:0,damage:80,fire_rate:70,projectile_speed:10,range:15,hardware:Hardware{range:15,movement:40,hp:200,damage:80,fire_rate:70,projectile_speed:10,secondary_range:14,secondary_damage:4,secondary_rate:80,secondary_speed:15}},
  "grunt"=>UnitInput{profile:0,runtime_class:0,traits:[25,60,30,40,30,60,0],grenades:0,damage:5,fire_rate:80,projectile_speed:15,range:15,hardware:Hardware{range:15,movement:0,hp:0,damage:5,fire_rate:80,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "tough"=>UnitInput{profile:3,runtime_class:3,traits:[100,40,30,30,100,40,0],grenades:0,damage:40,fire_rate:80,projectile_speed:10,range:15,hardware:Hardware{range:15,movement:0,hp:0,damage:40,fire_rate:80,projectile_speed:10,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "sniper"=>UnitInput{profile:17,runtime_class:0,traits:[40,120,36,50,50,120,0],grenades:0,damage:10,fire_rate:80,projectile_speed:15,range:18,hardware:Hardware{range:18,movement:3,hp:0,damage:10,fire_rate:80,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "heavy"=>UnitInput{profile:8,runtime_class:1,traits:[0,80,36,0,30,80,0],grenades:0,damage:120,fire_rate:60,projectile_speed:8,range:18,hardware:Hardware{range:18,movement:30,hp:300,damage:120,fire_rate:60,projectile_speed:8,secondary_range:45,secondary_damage:4,secondary_rate:80,secondary_speed:15}},
  "psycho"=>UnitInput{profile:18,runtime_class:0,traits:[30,50,30,40,50,50,0],grenades:0,damage:2,fire_rate:98,projectile_speed:15,range:15,hardware:Hardware{range:15,movement:0,hp:0,damage:2,fire_rate:98,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "launcher"=>UnitInput{profile:7,runtime_class:1,traits:[0,80,40,0,30,80,0],grenades:0,damage:120,fire_rate:20,projectile_speed:4,range:20,hardware:Hardware{range:20,movement:20,hp:300,damage:120,fire_rate:20,projectile_speed:4,secondary_range:96,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "light"=>UnitInput{profile:9,runtime_class:1,traits:[0,80,30,0,30,80,2],grenades:0,damage:50,fire_rate:80,projectile_speed:12,range:15,hardware:Hardware{range:15,movement:50,hp:100,damage:50,fire_rate:80,projectile_speed:12,secondary_range:0,secondary_damage:4,secondary_rate:80,secondary_speed:15}},
  "missile"=>UnitInput{profile:4,runtime_class:2,traits:[0,60,40,40,30,60,2],grenades:0,damage:100,fire_rate:30,projectile_speed:5,range:20,hardware:Hardware{range:20,movement:0,hp:100,damage:100,fire_rate:30,projectile_speed:5,secondary_range:74,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "gatling"=>UnitInput{profile:5,runtime_class:2,traits:[0,50,30,40,30,50,0],grenades:0,damage:6,fire_rate:97,projectile_speed:15,range:15,hardware:Hardware{range:15,movement:0,hp:50,damage:6,fire_rate:97,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "jeep"=>UnitInput{profile:11,runtime_class:1,traits:[0,40,30,0,30,40,0],grenades:0,damage:4,fire_rate:98,projectile_speed:15,range:15,hardware:Hardware{range:15,movement:60,hp:50,damage:4,fire_rate:98,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "truck"=>UnitInput{profile:14,runtime_class:1,traits:[0,60,30,40,30,60,5],grenades:0,damage:0,fire_rate:0,projectile_speed:0,range:0,hardware:Hardware{range:0,movement:60,hp:75,damage:0,fire_rate:0,projectile_speed:0,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "apc"=>UnitInput{profile:10,runtime_class:1,traits:[30,60,30,40,30,60,0],grenades:0,damage:5,fire_rate:80,projectile_speed:15,range:15,hardware:Hardware{range:15,movement:50,hp:200,damage:5,fire_rate:80,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "howitzer"=>UnitInput{profile:6,runtime_class:2,traits:[40,60,50,40,30,60,0],grenades:0,damage:100,fire_rate:10,projectile_speed:5,range:25,hardware:Hardware{range:25,movement:0,hp:75,damage:100,fire_rate:10,projectile_speed:5,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "firetruck"=>UnitInput{profile:13,runtime_class:1,traits:[0,60,30,40,40,60,0],grenades:0,damage:20,fire_rate:98,projectile_speed:8,range:15,hardware:Hardware{range:15,movement:50,hp:200,damage:20,fire_rate:98,projectile_speed:8,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "crane"=>UnitInput{profile:12,runtime_class:1,traits:[0,60,30,40,40,60,0],grenades:0,damage:120,fire_rate:50,projectile_speed:0,range:15,hardware:Hardware{range:15,movement:50,hp:300,damage:120,fire_rate:50,projectile_speed:0,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "pyro"=>UnitInput{profile:15,runtime_class:15,traits:[50,80,30,40,80,80,0],grenades:0,damage:20,fire_rate:98,projectile_speed:13,range:15,hardware:Hardware{range:15,movement:0,hp:0,damage:20,fire_rate:98,projectile_speed:13,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  "laser"=>UnitInput{profile:16,runtime_class:16,traits:[50,40,34,50,60,40,0],grenades:0,damage:15,fire_rate:80,projectile_speed:15,range:17,hardware:Hardware{range:17,movement:0,hp:0,damage:15,fire_rate:80,projectile_speed:15,secondary_range:0,secondary_damage:0,secondary_rate:0,secondary_speed:0}},
  _=>return None,
})}
fn weighted(values:&[i32],weights:&[i8])->(i32,i32){
 let mut sum=0i32;let mut count=0;
 for (&value,&weight) in values.iter().zip(weights){if weight!=-1{sum=sum.wrapping_add(value.wrapping_mul(i32::from(weight)));count+=1;}}
 (sum,count)
}
fn trunc_sse(value:f32)->i32{
 //CVTTSS2SI returns integer indefinite on overflow/NaN, not Rust's saturation.
 if !(-2147483648.0..2147483648.0).contains(&value){i32::MIN}else{value as i32}
}
///0041a560. The weapon stage multiplies BEFORE the float32 division. Changing
/// this to integer division, or moving the multiplier after division, changes it.
pub fn unit_score(u:&UnitInput)->i32{
 let profile=usize::from(u.profile);assert!(profile<19,"Native score profile outside the original table");
 let (trait_sum,trait_count)=weighted(&u.traits.map(i32::from),&TRAIT_WEIGHTS[profile]);
 let mut result=if trait_count==0{0}else{trait_sum/trait_count};
 let values=if matches!(u.runtime_class,0|3|15|16){[i32::from(u.range)*8,0,0,i32::from(u.damage),i32::from(u.fire_rate),i32::from(u.projectile_speed),0,0,0,0]}else{u.hardware.vector()};
 let (weapon_sum,weapon_count)=weighted(&values,&WEAPON_WEIGHTS[profile][..10]);
 if weapon_count!=0{
  let product=weapon_sum.wrapping_mul(i32::from(WEAPON_WEIGHTS[profile][10]));
  result=result.wrapping_add(trunc_sse(product as f32/weapon_count as f32));
 }
 result.wrapping_add(i32::from(u.grenades)*25)
}
///004165d0's static production member valuation. Its vector is deliberately
///different from a live unit: [range_i16,move_u8,hp_i8*25,damage_i8,rate_i8,
///speed_u8,secondary_range_u8*8,secondary_damage_u8,secondary_rate_u8,
///secondary_speed_u8]. Integer division precedes the weapon multiplier.
///If no trait coefficient participates, the entire preceding member score is
///retained before adding this weapon score, matching the native local410.
pub fn static_production_member_score(profile:u8,traits:[i8;7],weapon_vector:[i32;10],previous_member_score:i32)->i32{
 let profile=usize::from(profile);assert!(profile<19,"Native score profile outside the original table");
 let(trait_sum,trait_count)=weighted(&traits.map(i32::from),&TRAIT_WEIGHTS[profile]);
 let mut result=if trait_count==0{previous_member_score}else{trait_sum/trait_count};
 let(sum,count)=weighted(&weapon_vector,&WEAPON_WEIGHTS[profile][..10]);
 if count!=0{result=result.wrapping_add((sum/count).wrapping_mul(i32::from(WEAPON_WEIGHTS[profile][10])));}
 result
}
///0041aae0. For this function hardware.hp is the object's raw HP byte, before
/// the native multiplication by25. It uses integer division rather than the
/// live unit's float32 operation. Values are unperturbed;0041a8b0 is separate.
pub fn object_score(object_type:u8,hardware:Hardware)->i32{
 let profile=match object_type{5=>2,8=>1,21=>4,22=>5,23=>6,24=>7,26=>8,29=>9,31=>10,33=>11,35=>12,_=>return 0};
 let mut values=hardware.vector();values[2]=i32::from(hardware.hp)*25;
 let (sum,count)=weighted(&values,&WEAPON_WEIGHTS[profile][..10]);
 if count==0{0}else{sum.wrapping_mul(i32::from(WEAPON_WEIGHTS[profile][10]))/count}
}
///00416e10, after summing every eligible group score. The original aggression
/// byte is sign extended. Attack allocation can exceed total strength when the
/// requested enemy deficit is greater than the entire army.
pub fn reserve_budget(total:i32,aggression:i8,territory_surplus:f32,territory_deficit:f32)->(f32,f32){
 let strength=total as f32;
 let mut reserve=((100-i32::from(aggression))as f32*strength)/100.-territory_surplus;
 if reserve<0.{reserve=0.;}if strength<reserve{reserve=strength;}
 let mut attack=strength-reserve;let missing=territory_deficit-attack;
 if missing>0.{reserve-=missing;attack+=missing;if reserve<0.{reserve=0.;}}
 (reserve,attack)
}
///00413ca0. Native signed flag tile deltas are shifted before square root.
/// The zero-distance sentinel is deliberately finite and independent of score.
pub fn distance_priority(score:i32,flag:(i8,i8),origin:(i8,i8),distance_bias_enabled:bool)->i32{
 if !distance_bias_enabled{return score;}
 let dx=(i32::from(flag.0)-i32::from(origin.0))>>2;let dy=(i32::from(flag.1)-i32::from(origin.1))>>2;
 let distance=((dx*dx+dy*dy)as f32).sqrt();
 if distance==0.{0x1fffffff}else{trunc_sse(score as f32/distance)}
}
///00415950/00414570. Keep the native float32 operation order: reserve is
///multiplied by100 before dividing by total strength, then subtracted from100.
pub fn defense_priority(priority:i32,reserve:f32,attack:f32)->i32{
 let total=attack+reserve;let denominator=if total==0.{1.}else{total};
 trunc_sse(((100.-(reserve*100.)/denominator)*priority as f32)/100.)
}
///00415d60. Strengths must already be the native signed16 sector accumulators.
/// Population is not an input here. Only valid sectors with flag0x80 contribute;
/// the calculation is disabled without any valid flag0x01 first-fort sector.
#[derive(Clone,Copy,Debug)]
pub struct PressureSector{pub present:bool,pub flags:u8,pub human:i16,pub cpu:i16}
pub fn territory_pressure(sectors:&[PressureSector],reserve_percent:i8)->(f32,f32){
 if !sectors.iter().any(|s|s.present&&s.flags&1!=0){return (0.,0.);}
 let(mut human,mut cpu)=(0i32,0i32);
 for s in sectors.iter().filter(|s|s.present&&s.flags&0x80!=0){human=human.wrapping_add(i32::from(s.human));cpu=cpu.wrapping_add(i32::from(s.cpu));}
 let adjusted=|value:i32|value.wrapping_add((i32::from(reserve_percent)-50).wrapping_mul(value)/100);
 if cpu<=human{(adjusted(human.wrapping_sub(cpu))as f32,0.)}else{(0.,adjusted(human)as f32)}
}
///0041a290's update gate. Byte29 is independent of the flags at byte24.
pub fn should_update_sector_priority(lower_x:i16,alternate_mode:bool,scoring_flags:u8)->bool{
 lower_x>0&&(!alternate_mode||scoring_flags&1!=0)
}
///0041a290/0041a150/0041a230. Object and building values must be individually
///perturbed by0041a8b0 before summation. Inactive/deleted records are omitted;
///destroyed buildings with a valid native type still contribute their value.
pub fn sector_priority(population:i8,population_weight:i8,human_strength:i16,objects:&[i32],buildings:&[i32])->i32{
 let base=(i32::from(population)*i32::from(population_weight)*16).wrapping_sub(i32::from(human_strength));
 objects.iter().chain(buildings).fold(base,|sum,&value|sum.wrapping_add(value))
}
///00411b90 sums signed grenade bytes in the21 native group slots. Callers
///normally test only AL; a sum of256 therefore fails their capability test.
pub fn group_grenade_sum(grenades:&[i8])->u16{
 grenades.iter().take(21).fold(0u16,|sum,&count|sum.wrapping_add(count as i16 as u16))
}
pub fn group_can_demolish(grenades:&[i8])->bool{group_grenade_sum(grenades)as u8!=0}
///0044ea30 disables the initial distance weighting after the first ownership
///change made by a team other than the local player. It stays disabled until
///a new mission's00417b30 initialization.
pub fn distance_bias_after_capture(enabled:bool,capturing_team:u8,local_player:u8,changed:bool)->bool{
 enabled&&(!changed||capturing_team==local_player)
}

//00499888, used by004170a0's four-neighbor infantry flood.
const FLOOD_TERRAIN:[u8;128]=[0,255,255,255,255,255,255,0,0,0,0,255,255,255,255,255,255,255,255,255,0,0,0,0,255,255,0,0,255,255,0,255,255,255,255,0,0,0,0,0,255,0,0,255,0,0,255,255,255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,0,0,0,0,0,0,0,0,5,0,255,255,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,2,1,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0];
///004170a0. Returns whether to set sector flag0x40. The source is the complete
///128x128 native terrain plane, including cells beyond the visible map size.
///sector_at receives native pixel coordinates and must use00417400 lookup.
pub fn enclosed_infantry_area(codes:&[u8],flag:[i8;2],sector_at:impl FnMut(i16,i16)->Option<usize>)->bool{
 let origin=[i16::from(flag[0])*16,i16::from(flag[1])*16];
 enclosed_area(codes,i16::from(flag[1])*128+i16::from(flag[0]),origin,sector_at)
}
///0043eb20, used by the OBJECT constructor's byte26 flag. This uses the same
///flood as004170a0, but retains the exact original XY for the code35 comparison.
pub fn enclosed_object_area(codes:&[u8],origin:[i16;2],sector_at:impl FnMut(i16,i16)->Option<usize>)->bool{
 let start=((origin[1]as u16&0xfff0).wrapping_mul(8)).wrapping_add((origin[0]>>4)as u16)as i16;
 enclosed_area(codes,start,origin,sector_at)
}
fn enclosed_area(codes:&[u8],start:i16,origin:[i16;2],mut sector_at:impl FnMut(i16,i16)->Option<usize>)->bool{
 assert_eq!(codes.len(),128*128);
 let mut stack=vec![start];let mut visited=vec![false;128*128];
 while let Some(index)=stack.pop(){
  if index<=0x100||index>=0x3f00{return false;}
  let at=index as usize;visited[at]=true;
  if codes[at]&127==35&&sector_at((index&127)*16+8,(index>>7)*16+6)!=sector_at(origin[0],origin[1]){return false;}
  //Visited is marked when popped, not when pushed; duplicate pending entries
  //and their contribution to the1000-entry stack cutoff are native behavior.
  for next in [index+1,index-128,index+128,index-1]{let n=next as usize;if !visited[n]&&FLOOD_TERRAIN[usize::from(codes[n]&127)]==0{stack.push(next);}}
  if stack.len()>=1000{return true;}
 }
 true
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,n:i32){*h=(*h^n as u32).wrapping_mul(16777619);}
 fn sample(profile:i32,variant:i32,field:i32)->u8{((profile*13+variant*31+field*17)&255)as u8}
 fn sample_unit(profile:i32,cls:i32,variant:i32)->UnitInput{
  let s=|field|sample(profile,variant,field);
  UnitInput{profile:profile as u8,runtime_class:cls as u8,traits:std::array::from_fn(|i|s(i as i32)as i8),grenades:(variant*43)as i8,
   damage:s(7)as i8,fire_rate:s(8),projectile_speed:s(9),range:s(10),
   hardware:Hardware{range:s(11),damage:s(12),movement:s(13),fire_rate:s(14),projectile_speed:s(15),secondary_damage:s(16),secondary_rate:s(17),secondary_speed:s(18),secondary_range:s(19),hp:(300-variant*117)as i16}}
 }
 #[test]fn live_unit_scores_match_the_executed_x86_function(){
  let mut h=2166136261;
  for profile in 0..19{for cls in [0,1,2,3,15,16]{for variant in 0..6{
   let score=unit_score(&sample_unit(profile,cls,variant));for n in [profile,cls,variant,score]{hash(&mut h,n);}
  }}}
  assert_eq!(h,3609297929);
 }
 #[test]fn parked_object_scores_match_the_executed_x86_function(){
  let mut h=2166136261;
  for typ in [5,8,21,22,23,24,26,29,31,33,35,37,0]{for variant in 0..6{
   let s=|field|sample(typ,variant,field);let w=Hardware{range:s(0),movement:s(1),hp:i16::from(s(2)),damage:s(3),fire_rate:s(4),projectile_speed:s(5),secondary_range:s(6),secondary_damage:s(7),secondary_rate:s(8),secondary_speed:s(9)};
   let score=object_score(typ as u8,w);for n in [typ,variant,score]{hash(&mut h,n);}
  }}assert_eq!(h,752240877);
 }
 #[test]fn reserve_budget_matches_the_executed_x86_function(){
  let mut h=2166136261;
  for aggression in [0,25,50,100,127,255]{for own in [0,1,3571,131071]{for surplus in [0,1000,100000]{for deficit in [0,2500,200000]{
   let(r,a)=reserve_budget(own,aggression as i8,surplus as f32,deficit as f32);for n in [aggression,own,surplus,deficit,r.to_bits()as i32,a.to_bits()as i32]{hash(&mut h,n);}
  }}}}assert_eq!(h,3918427161);
 }
 #[test]fn distance_priority_matches_294_executed_native_cases_including_signed_rounding(){
  let mut h=2166136261;
  for dx in [-65,-17,-1,0,1,17,65]{for dy in [-65,-17,-1,0,1,17,65]{for score in [-131071,0,1,3571,131071,2147483647]{
   let result=distance_priority(score,((40+dx)as i8,(40+dy)as i8),(40,40),true);
   for n in [dx,dy,score,result]{hash(&mut h,n);}
  }}}
  assert_eq!(h,1195070520);
  assert_eq!(distance_priority(12,(0,0),(0,0),false),12);
 }
 #[test]fn nineteen_ordinary_attribute_sets_match_native_live_unit_scores(){
  let kinds=["cannon","medium","grunt","tough","sniper","heavy","psycho","launcher","light","missile","gatling","jeep","truck","apc","howitzer","firetruck","crane","pyro","laser"];
  let mut h=2166136261;
  for(raw,kind)in kinds.iter().enumerate(){let input=default_input(kind).unwrap();for n in [raw as i32,i32::from(input.profile),unit_score(&input)]{hash(&mut h,n);}}
  assert_eq!(h,2778310521);assert!(default_input("invalid").is_none());
 }
 #[test]fn defense_priority_matches_128_executed_native_cases(){
  let mut h=2166136261;
  for priority in [-131071,0,1,499,500,3571,131071,2147483647]{for reserve in [0,1,3571,100000]{for attack in [0,1,2500,200000]{
   let result=defense_priority(priority,reserve as f32,attack as f32);
   for n in [priority,reserve,attack,result]{hash(&mut h,n);}
  }}}
  assert_eq!(h,1525590046);
 }
 #[test]fn territory_pressure_matches_448_executed_native_cases(){
  let mut h=2166136261;
  for reserve in [-128,-1,0,25,50,100,127]{for scenario in 0..64{
   let mut sectors=Vec::new();if scenario&1!=0{sectors.push(PressureSector{present:true,flags:1,human:0,cpu:0});}
   for i in 0..6{sectors.push(PressureSector{present:i<scenario%7,flags:if scenario>>(i%5)&1!=0{128}else{0},human:(scenario*3571+i*5303)as i16,cpu:(scenario*7919+i*157)as i16});}
   let(surplus,deficit)=territory_pressure(&sectors,reserve as i8);
   for n in [reserve,scenario,surplus.to_bits()as i32,deficit.to_bits()as i32]{hash(&mut h,n);}
  }}assert_eq!(h,1928899141);
 }
 #[test]fn priority_base_and_update_gates_match_128_executed_native_cases(){
  let mut h=2166136261;
  for scenario in 0..128{
   let lower=scenario%3-1;let population=(scenario*43)as i8;let weight=(scenario*17)as i8;let human=(scenario*3571)as i16;let before=scenario*941;
   let result=if should_update_sector_priority(lower as i16,scenario>>4&1!=0,(scenario>>3&1)as u8){sector_priority(population,weight,human,&[],&[])}else{before};
   for n in [scenario,result]{hash(&mut h,n);}
  }assert_eq!(h,1788966805);
  assert_eq!(sector_priority(2,10,77,&[123,-10],&[5000,1000]),6356);
 }
 #[test]fn group_grenades_match_256_executed_native_cases(){
  let mut h=2166136261;
  for scenario in 0..256{
   let grenades:Vec<i8>=(0..21).filter(|i|scenario>>(i%8)&1!=0).map(|i|(scenario*37+i*17)as i8).collect();
   for n in [scenario,i32::from(group_grenade_sum(&grenades))]{hash(&mut h,n);}
  }
  assert_eq!(h,1870974933);
  assert!(!group_can_demolish(&[127,127,2]));
  assert!(group_can_demolish(&[1]));
 }
 #[test]fn enclosed_terrain_matches_32_executed_native_cases(){
  let mut h=2166136261;
  for scenario in 0..32{
   let mode=scenario%8;let radius=1+scenario/8*3;let mut codes=vec![1;128*128];
   if mode==3{codes.fill(0);}else if mode!=0{
    let radius=if mode==4{35}else{radius};
    for y in 64-radius..=64+radius{for x in 64-radius..=64+radius{codes[y as usize*128+x as usize]=0;}}
    if mode==2{for y in 0..=64{codes[y*128+64]=0;}}
    if mode==5||mode==6{codes[64*128+65]=35;}
   }
   let flags=if mode==7{2}else if enclosed_infantry_area(&codes,[64,64],|x,_|Some(if mode==5&&x>1024{1}else{0})){64}else{0};
   for n in [scenario,flags]{hash(&mut h,n);}
  }
  assert_eq!(h,2735782341);
 }
 #[test]fn object_flood_with_subtile_origin_matches_32_executed_native_cases(){
  let mut h=2166136261;
  for scenario in 0..32{
   let mode=scenario%8;let radius=1+scenario/8*3;let mut codes=vec![1;128*128];
   if mode==3{codes.fill(0);}else if mode!=0{
    let radius=if mode==4{35}else{radius};
    for y in 64-radius..=64+radius{for x in 64-radius..=64+radius{codes[y as usize*128+x as usize]=0;}}
    if mode==2{for y in 0..=64{codes[y*128+64]=0;}}
    if mode==5||mode==6{codes[64*128+65]=35;}
   }
   let origin=[(1024+scenario%16)as i16,(1024+(scenario*3)%16)as i16];
   let enclosed=enclosed_object_area(&codes,origin,|x,_|Some(if mode==5&&x>1024{1}else{0}));
   for n in [scenario,i32::from(enclosed)]{hash(&mut h,n);}
  }
  assert_eq!(h,2226743941);
 }
 #[test]fn opponent_capture_disables_distance_bias_until_mission_reset(){
  assert!(distance_bias_after_capture(true,0,0,true));
  assert!(distance_bias_after_capture(true,1,0,false));
  assert!(!distance_bias_after_capture(true,1,0,true));
  assert!(!distance_bias_after_capture(false,0,0,true));
 }
}
