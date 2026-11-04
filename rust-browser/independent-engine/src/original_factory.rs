//! Verified producer completion and exit kernels40b2a0/40c180/40c340.
///40b070 common initialization plus the descriptor-selected constructor's
///observable control fields. None denotes a byte that this constructor keeps
///from its disk record, rather than a fabricated zero.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct ConstructorFields{pub flags6:u8,pub style5:Option<u8>,pub field10:Option<u8>,pub field11:u8,pub clock3e:Option<u8>,pub state45:u8,pub remaining24:Option<i32>,pub maximum1c:Option<i16>,pub door3a:i8}
///Type0 repair uses460910; types2 fort and4 robot use the separate CRT rand.
///Terrain stamping40cc90 and damage-fragment setup41f8f0 have no RNG calls.
pub fn constructor_fields(kind:i8,flags6:u8,hp:i16,multiplayer:bool,mut random:impl FnMut()->u32,mut crt:impl FnMut()->u16)->Option<ConstructorFields>{
 let k=usize::try_from(kind).ok()?;let door3a=*[32,54,64,32,54,32,32,32,30].get(k)?;let active=matches!(kind,0|1|2|4|8);let mut f=ConstructorFields{flags6:flags6&0xe0,style5:active.then_some(8),field10:matches!(kind,2|4).then_some(0),field11:0,clock3e:None,state45:if kind==0{5}else{0},remaining24:active.then_some(0),maximum1c:(kind!=2).then_some(hp),door3a};if kind==0{f.field11=((random()as u8)&0x33).wrapping_add(0x88);}if matches!(kind,2|4){f.clock3e=Some(if multiplayer{0}else{crt()as u8});}if kind==8{f.flags6&=!32;}Some(f)
}
#[cfg(test)]mod constructor_tests{
 use super::*;
 #[test]fn all_nine_constructor_fields_and_stream_effects_match_72_native_calls(){let mut hash=2166136261u32;for s in 0..72usize{let k=(s%9)as i8;let multiplayer=s/9%2!=0;let hp=100+s as i16*11;let mut rng=crate::original_combat::OriginalRng{words:std::array::from_fn(|i|(s*(i+1)+i+1)as u32),multiplayer};let mut draws=0;let f=constructor_fields(k,(s*13)as u8,hp,multiplayer,||rng.next_u32(),||{draws+=1;(s as u16*31)+0x1234}).unwrap();let maximum=f.maximum1c.unwrap_or(2000+(s%4)as i16*100+s as i16);let current=if k==2{1000+(s%4)as i16*100+s as i16}else{hp};let values=[s as u32,k as u32,multiplayer as u32,u32::from(f.style5.unwrap_or(171)),u32::from(f.flags6),u32::from(f.field10.unwrap_or(188)),u32::from(f.field11),u32::from(f.clock3e.unwrap_or(165+s as u8)),u32::from(f.state45),f.remaining24.unwrap_or(123456)as u32,maximum as u32,current as u32,draws,rng.words[0],rng.words[1],rng.words[2],rng.words[3],rng.words[4]];for n in values{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,203912829,"real9 descriptor constructors,460910/40b1c0/40cc90/41f8f0; DLLrand/sector417400 explicit inputs");}
}
// All coordinates are original signed16 pixels, before the Browser's x2 scale.
// The optional CPU product policy must not override the user's robot default.

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Producer{Fort,Robot,Vehicle}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ExitGeometry{
    pub spawn:[i16;2],
    ///45ef00 command/primary-path waypoint.52810 subtracts8 on both axes.
    pub destination:[i16;2],
}
impl ExitGeometry{pub fn anchor_destination(self)->[i16;2]{[self.destination[0].wrapping_sub(8),self.destination[1].wrapping_sub(8)]}}

///40b2a0 selects globals478590/47858c/478408 for fort exits. Robot factories
///use their restored defaults.40c340 has a separate fixed vehicle exit.
pub fn exit_geometry(producer:Producer,building:[i16;2],owner:u8,role:usize)->ExitGeometry{
    let(dx,dy,exit)=match producer{
        Producer::Fort if owner==1||owner==3=>(71i16,104i16,112i16),
        Producer::Fort=>(72,4,-48),
        Producer::Robot=>(32,50,48),
        Producer::Vehicle=>(23,32,56),
    };
    let offset=if producer==Producer::Vehicle{0}else{(role as i16).wrapping_mul(4)};
    let spawn=[building[0].wrapping_add(dx),building[1].wrapping_add(dy).wrapping_sub(offset)];
    let destination=[spawn[0].wrapping_add(8),spawn[1].wrapping_add(exit)];
    ExitGeometry{spawn,destination}
}
///40c180/40c340 skip creation during the native suppressed callback or when
///the team's622e18 status is not FF. Gun products enter their separate queue.
pub fn can_create_group(suppressed:bool,team_status:i8,gun_product:bool)->bool{!suppressed&&team_status==-1&&!gun_product}
///10570 includes ready/queued guns in the team limit and reserves at least
///seven free native unit slots, independent of the selected group's size.
///The caller sign-extends CPUPLR+8a8/global5c47b4 before this signed16 compare.
pub fn team_capacity_blocked(queued_guns:i16,team_units:i8,team_limit:i16,free_unit_slots:usize)->bool{
    queued_guns.wrapping_add(i16::from(team_units))>=team_limit||free_unit_slots<7
}
///f580/106c0 compare the low signed16 aggregate against three. The count is
///provided after temporarily excluding the producing building's active order.
pub fn gun_capacity_blocked(product_profile:u8,sector_guns:i16)->bool{GUNS.contains(&product_profile)&&sector_guns>3}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Work{
    ///BUILD+24/28/2c. The credit can carry work from a switched product.
    pub remaining:i32,pub total:i32,pub credit:i32,
    ///BUILD+a and+36. The native producer hold is68 callbacks.
    pub animation:i16,pub hold:i16,
}
///40a8a0: initialize after product selection. `duration` is41a310's table
///value; `divisor` is622dba. Native signed32 multiplication wraps first.
pub fn initialize_work(work:&mut Work,duration:i32,divisor:i16){
    assert!(divisor!=0);work.total=duration.wrapping_mul(221)/i32::from(divisor);
    work.remaining=work.total.wrapping_sub(work.credit);if work.remaining<0{work.remaining=1;}work.animation=0;
}
///40b2a0/40c490 work branch after alive/owner/product gates. The caller performs
///40ad60's hold decrement first; capacity blocking freezes work, not animation.
///A true result requires product reinitialization and the producer's completion
///event; `complete_work` below performs the shared arithmetic writes.
pub fn spend_work(work:&mut Work,rate:i32,capacity_blocked:bool)->bool{
    if work.hold!=0{work.animation=0;return false;}
    if rate==0{return false;}
    work.animation=work.animation.wrapping_sub(1);
    if i32::from(work.animation)<=rate.wrapping_sub(27){work.animation=work.animation.wrapping_neg();}
    if capacity_blocked{return false;}
    work.remaining=work.remaining.wrapping_sub(rate);work.remaining<0
}
pub fn complete_work(work:&mut Work,duration:i32,divisor:i16){initialize_work(work,duration,divisor);work.hold=68;}
///45f070 product-cancel command: save the old product inBUILD+44 and retain
///only its completed work as credit. The caller writes productFF.
pub fn cancel_product(work:&mut Work,current_product:i8)->i8{
    work.credit=work.total.wrapping_sub(work.remaining);work.remaining=0;work.animation=0;current_product
}
///45f070 product-select command. Player switches discard another product's
///credit; single-playerCPU owner1 preserves it.40a8a0 consumes the credit once.
pub fn select_product(work:&mut Work,new_product:i8,cancelled_product:i8,owner:i8,multiplayer:bool,duration:i32,divisor:i16){
    if cancelled_product!=new_product&&(multiplayer||owner!=1){work.credit=0;}
    initialize_work(work,duration,divisor);work.credit=0;
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct FactoryExitState{
    ///Native command/waypoint, before52810's−8 anchor conversion.
    pub destination:[i16;2],
    ///Factory45f070 move command1, auxiliary order4. The original changes
    ///the command sign once52810 successfully starts the path.
    pub order:i8,pub auxiliary:i16,pub flags9c:u8,pub protection:u8,
}
impl FactoryExitState{
    pub fn new(destination:[i16;2])->Self{Self{destination,order:1,auxiliary:4,flags9c:0x30,protection:0}}
    ///5a080 prefix. Return true when it also clears roster9 bit20.
    pub fn preamble(&mut self)->bool{
        if self.protection==0{return false;}
        self.flags9c&=!0x14;self.protection-=1;true
    }
    ///54160 command preamble, before the transport-uniform gate/path builder.
    ///Returns(false,_) when the positive order is currently gated; the second
    ///result tells the caller to set roster9 bit20 for a leader.
    pub fn begin_command(&mut self,leader:bool)->(bool,bool){
        if self.order<=0{return(false,false);}
        if self.flags9c&0x20!=0{
            if self.flags9c&4!=0{return(false,false);}
            if self.flags9c&0x10!=0{self.protection=68;self.flags9c=self.flags9c&!0x10|4;return(true,leader);}
        }
        (true,false)
    }
    ///538f0 after52810 successfully starts its OWN path. A leader restores
    ///roster8 from saved3c and clears pending bit20; followers retain that bit.
    pub fn path_started(&mut self,leader:bool){if leader{self.flags9c&=!0x20;}self.order=self.order.wrapping_neg();}
    ///52960→52810 retains an unfinished individual path before any formation
    ///target is considered. The68-callback protection is not an exit delay.
    pub fn blocks_formation(&self,own_path_present:bool)->bool{self.order>0||own_path_present}
}

#[derive(Clone,Debug)]
pub struct Product{
    pub id:u8,pub enabled:bool,pub first_member:bool,pub profile:u8,
    ///Authored ROBOTS.DAT group+49(vehicle)/4a(robot)/4b(fort).
    pub levels:[i8;3],
    ///41a310 and165d0 use the same static member valuation and sum order.
    pub duration:i32,
}
pub const INFANTRY:&[u8]=&[0,3,15,16,17,18];
pub const VEHICLES:&[u8]=&[1,7,8,9,11];
pub const FORT_PRODUCTS:&[u8]=&[0,3,15,16,17,18,1,7,8,9,11];
pub const GUNS:&[u8]=&[2,4,5,6];
#[derive(Clone,Debug)]
pub struct CpuContext{
    pub producer:Producer,pub flags6:u8,pub technology:i32,
    ///584972: prior completed gun placement was deferred.
    pub deferred:bool,
    pub attack:f32,pub reserve:f32,pub sector_priority:i32,
    ///117f0 counts living CPU building orders whose selected product is a gun.
    pub planned_cpu_guns:i16,
    ///0f6f0 counts live CPU guns PLUS living CPU non-gun production orders in
    ///the same sector. It intentionally is not simply a count of cannons.
    pub sector_load:i32,
    ///106c0: gun capacity after temporarily clearing this building's product.
    pub gun_capacity_blocked:bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CpuBranch{pub gun_request:bool,pub flags6:u8,pub level_column:usize,pub sort_by_duration:bool}
impl CpuBranch{
    pub fn profiles(self,producer:Producer)->&'static[u8]{if self.gun_request{GUNS}else{match producer{Producer::Fort=>FORT_PRODUCTS,Producer::Robot=>INFANTRY,Producer::Vehicle=>VEHICLES}}}
}
///16c60/16ab0/189e0. A robot factory's pressure test is strictly >500; fort
///and vehicle use >=500. No RNG is consumed by this product-selection policy.
pub fn cpu_branch(input:&CpuContext)->CpuBranch{
    let pressure=super::native_ai_scoring::defense_priority(input.sector_priority,input.reserve,input.attack);
    let technology=input.producer==Producer::Vehicle||input.technology>1000;
    let pressure_ok=if input.producer==Producer::Robot{pressure>500}else{pressure>=500};
    let limit=if input.producer==Producer::Fort{4}else{2};
    //189e0 compares AX, unlike the other two wrappers' full32-bit result.
    let load=if input.producer==Producer::Robot{i32::from(input.sector_load as i16)}else{input.sector_load};
    let gun_request=!input.deferred&&input.flags6&2!=0&&technology&&pressure_ok&&input.planned_cpu_guns==0&&load<limit&&!input.gun_capacity_blocked;
    CpuBranch{gun_request,flags6:if gun_request{input.flags6}else{input.flags6|2},level_column:match input.producer{Producer::Vehicle=>0,Producer::Robot=>1,Producer::Fort=>if gun_request{1}else{2}},sort_by_duration:input.producer==Producer::Fort&&!gun_request}
}
///16310's level-major32-record scan. On append it compares only the two last
///durations; this is deliberately not a full insertion sort.
pub fn candidates(products:&[Product],technology:i32,column:usize,profiles:&[u8])->Vec<u8>{
    //Above32767 the original signed16 level cursor wraps and never finishes.
    if column>2||technology<1000||technology>32767{return Vec::new();}
    let mut result=Vec::new();
    for level in 1..=technology/1000{
        for id in 0..32{
            let Some(p)=products.iter().find(|p|usize::from(p.id)==id)else{continue;};
            if !p.enabled||!p.first_member||i32::from(p.levels[column])*1000!=level*1000||!profiles.contains(&p.profile){continue;}
            result.push(p.id);
            if result.len()>1{
                let n=result.len();let previous=products.iter().find(|p|p.id==result[n-2]).unwrap().duration;
                if p.duration<previous{result.swap(n-2,n-1);}
            }
        }
    }
    result
}
///164a0/16970/16a10/16530: native signed division, then a25% integer shift
///bias. Zero strength or an empty list would fault/read stale scratch memory
///in the original; these missing native states are explicit None boundaries.
pub fn select_cpu_product(products:&[Product],input:&CpuContext,human_strength:i32,cpu_strength:i32)->Option<u8>{
    let branch=cpu_branch(input);let mut list=candidates(products,input.technology,branch.level_column,branch.profiles(input.producer));
    if branch.sort_by_duration{list.sort_by_key(|id|products.iter().find(|p|p.id==*id).unwrap().duration);}
    select_ranked(&list,human_strength,cpu_strength)
}
pub fn select_ranked(list:&[u8],human_strength:i32,cpu_strength:i32)->Option<u8>{
    let total=human_strength.wrapping_add(cpu_strength);if total==0||list.is_empty(){return None;}
    let n=list.len()as i32;let index=n.wrapping_mul(cpu_strength).checked_div(total)?;let index=index.wrapping_add(index>>2);
    if index<n{usize::try_from(index).ok().and_then(|i|list.get(i).copied())}else{list.last().copied()}
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(values:impl IntoIterator<Item=u32>)->u32{values.into_iter().fold(2166136261u32,|h,n|(h^n).wrapping_mul(16777619))}
 #[test]fn cancelled_work_credit_and_owner_product_switch_rules_match_256_native_command_records(){
  let mut values=Vec::new();for s in 0..128{
   let owner=s%2;let current=(s%19)as i8;let mut work=Work{remaining:100+s*7,total:1000+s*13,credit:0,animation:17,hold:0};let saved=cancel_product(&mut work,current);
   values.extend([s as u32,u32::MAX,saved as u32,work.remaining as u32,work.total as u32,work.credit as u32,work.animation as u32]);
   let next=if s&4!=0{current}else{((s+3)%19)as i8};select_product(&mut work,next,saved,owner as i8,s&2!=0,crate::original_stats::PRODUCTS[next as usize].5,12);
   values.extend([s as u32,next as u32,saved as u32,work.remaining as u32,work.total as u32,work.credit as u32,work.animation as u32]);
  }
  assert_eq!(hash(values),1311412742,"ZNativeFactoryCreditOracle actual45f070 cancel/select with40a8a0/41a310 costs");
 }
 #[test]fn unit_reserve_and_gun_capacity_match_256_native_pool_and_wrapper_executions(){
  let mut values=Vec::new();for scenario in 0..256{
   let queued=(scenario/4)%5;let pending=if scenario&128!=0{1}else{0};let units=[-128,0,118,120][(scenario/20)%4];let limit=[0,120,127,255][(scenario/80)%4];
   let blocked=team_capacity_blocked((queued+pending)as i16,units,limit,scenario%12);let profile=crate::native_ai_scoring::default_input(crate::original_stats::PRODUCTS[scenario%19].0).unwrap().profile;
   let guns=gun_capacity_blocked(profile,(scenario%7)as i16-1);values.extend([scenario as u32,u32::from(blocked),u32::from(guns),99]);
  }
  assert_eq!(hash(values),2514787596,"ZNativeFactoryCapacityOracle actual10510/10570 and f580 with explicit10600 count input");
 }
 #[test]fn signed_cp_limit_and_queued_stock_match_512_executed_whole_capacity_wrappers(){
  let mut values=Vec::new();for s in 0..512usize{
   let limit=[50,120,127,-128,-1,0,1][s/11%7];let units=[48,49,50,51,119,120,127,-128,-1,0,1][s%11];let queued=s/77%5;let pending=s/385%2;let free=[220,7,6][s/33%3];let blocked=team_capacity_blocked((queued+pending)as i16,units,limit,free);
   values.extend([s as u32,limit as u32,units as u32,queued as u32,pending as u32,free as u32,u32::from(blocked)]);
  }assert_eq!(hash(values),1338239080,"actual40c080→10570/10510 and f580; Grunt profile0 needs no intercepted gun-count leaf");
 }
 #[test]fn exit_protection_order_and_layout_preambles_match_128_native_sequences(){
  let mut values=Vec::new();for scenario in 0..128{
   let leader=scenario&1==0;let mut exit=FactoryExitState::new([0,0]);exit.flags9c=(scenario*13)as u8;exit.protection=[0,1,68,255][(scenario>>1)&3];exit.order=[-1,0,1,1][(scenario>>3)&3];let mut group_flags=0xa5;let mut layout=255;let mut bonus=0;
   if exit.preamble(){group_flags&=!0x20;}
   let(accepted,set_group)=exit.begin_command(leader);if set_group{group_flags|=0x20;}
   if accepted{if leader{layout=2;if exit.flags9c&0x20!=0{bonus=if scenario%3==0{30}else{120};}}exit.path_started(leader);}
   values.extend([scenario as u32,u32::from(accepted),u32::from(exit.flags9c),u32::from(exit.protection),exit.order as u32,group_flags,layout,bonus]);
  }
  assert_eq!(hash(values),3760967707,"ZNativeFactoryPrefixOracle actual5a080/54160/538f0 prefixes;52810 path acceptance is an explicit boundary");
 }
 #[test]fn work_initialization_blocking_animation_and_strict_completion_match_native_producer_blocks(){
  let mut values=Vec::new();for scenario in 0..256{
   let duration=crate::original_stats::PRODUCTS[scenario%19].5;let divisor=[1,12,17,221][(scenario/19)%4];let credit=match scenario%3{0=>-1234,1=>500,_=>100000};
   let mut work=Work{remaining:0,total:0,credit,animation:0,hold:0};initialize_work(&mut work,duration,divisor);values.extend([scenario as u32,work.remaining as u32,work.total as u32,work.animation as u32]);
   let rate=(scenario%36)as i32;work.remaining=[-1,0,1,rate,rate-1,rate+1,50000][scenario%7];work.animation=[0,-27,17,-32768,32767][scenario%5];
   let completed=spend_work(&mut work,rate,scenario&1!=0);if completed{complete_work(&mut work,duration,divisor);}
   values.extend([scenario as u32,u32::from(completed),work.remaining as u32,work.total as u32,work.animation as u32,work.hold as u32]);
  }
  assert_eq!(hash(values),1429707117,"ZNativeFactoryWorkOracle,256 full40a8a0 and bounded40b2a0 executions");
 }
 #[test]fn all_2048_original_main_build_callbacks_draw_old_work_then_spend_and_complete_before_b180(){
  let mut values=Vec::new();for s in 0..2048i32{let kind=if s%2==0{2}else{4};let hold=[0,1,2,68][(s/2%4)as usize];let hp=[0,1,20,100][(s/8%4)as usize];let owned=s/32%9;let owner=[0,1,4][(s/288%3)as usize];let blocked=s/864%2;let phase=s%4;let rate=hp*(1+owned*100/8/4)/100;let remaining=[-1,0,1,rate,rate+1,1000][(s*7%6)as usize];let animation=[0,17,-27][(s%3)as usize];let mut work=Work{remaining,total:4000+s,credit:0,animation:animation as i16,hold:crate::original_building_display::completion_at_draw(hold as i16)};let(mut clocks,mut clock_work,mut clock_hold,mut seconds,mut spawns)=(0,0,0,0,0);
   if hp!=0&&owner<4{if work.hold!=0{work.animation=0;if phase&2!=0{clocks=1;clock_work=remaining;clock_hold=i32::from(work.hold);}}else{clocks=1;clock_work=remaining;seconds=if rate==0{-1}else{remaining/(rate*17)};if spend_work(&mut work,rate,blocked!=0){complete_work(&mut work,crate::original_stats::PRODUCTS[0].5,12);spawns=1;}}}
   values.extend([s,kind,owner,hp,owned,blocked,hold,remaining,animation,phase,rate,clocks,clock_work,clock_hold,seconds,spawns,work.remaining,work.total,i32::from(work.hold),i32::from(work.animation),0].map(|v|v as u32));
  }assert_eq!(hash(values),3477325678,"actual40ad60+40a790+40b2a0/40c490+40a6d0+40ab90+40a8a0; capacity/spawn/CPU/raster are explicit boundaries;40b180 executes and preserves work");
 }
 #[test]fn exits_match_80_commands_from_32_real_native_factory_creations_without_allocator_stubs(){
  let mut values=Vec::new();for scenario in 0..32{
   let owner=scenario%4;let producer=(scenario/4)%4;let kind=if producer==0{Producer::Fort}else if producer==3{Producer::Vehicle}else{Producer::Robot};
   let count=if producer==3{1}else{3};
   for role in 0..count{let e=exit_geometry(kind,[100+scenario*3,200+scenario*5],owner as u8,role);values.extend([scenario as u32,producer as u32,owner as u32,role as u32,if role==0{u32::MAX}else{role as u32},e.spawn[0]as u32,e.spawn[1]as u32,e.destination[0]as u32,e.destination[1]as u32]);}
  }
  assert_eq!(hash(values),3091907773,"ZNativeFactoryOracle actual group/unit initialization chain");
 }
 #[test]fn cpu_defensive_product_branches_match_384_native_wrapper_executions(){
  let mut values=Vec::new();for scenario in 0..384{
   let producer=[Producer::Fort,Producer::Robot,Producer::Vehicle][scenario%3];
   let c=CpuContext{producer,flags6:if scenario&8!=0{2}else{0},technology:if scenario&16!=0{2000}else{1000},deferred:scenario&4!=0,attack:if scenario&128!=0{100.}else{1.},reserve:if scenario&128!=0{100.}else{0.},sector_priority:[499,500,501,1000][(scenario>>5)&3],planned_cpu_guns:if scenario&256!=0{1}else{0},sector_load:((scenario/3)%5)as i32,gun_capacity_blocked:(scenario/15)%2!=0};
   let b=cpu_branch(&c);values.extend([scenario as u32,u32::from(b.gun_request),b.level_column as u32,u32::from(b.flags6)]);
  }
  assert_eq!(hash(values),1233909170,"ZNativeFactoryChoicesOracle wrapper count/capacity boundaries are explicit inputs");
 }
 #[test]fn static_candidate_scans_sorting_and_strength_ratio_match_128_native_selection_executions(){
  let products=crate::original_stats::PRODUCTS.iter().enumerate().map(|(id,p)|Product{id:id as u8,enabled:true,first_member:true,profile:crate::native_ai_scoring::default_input(p.0).unwrap().profile,levels:[p.2 as i8,p.3 as i8,p.4 as i8],duration:p.5}).collect::<Vec<_>>();
  let mut values=Vec::new();for scenario in 0..128{
   let kind=scenario%4;let column=match kind{0=>2,1=>1,2=>0,_=>(scenario/4)%3};let profiles=match kind{0=>FORT_PRODUCTS,1=>INFANTRY,2=>VEHICLES,_=>GUNS};
   let mut list=candidates(&products,(1000+(scenario/12)%5*1000)as i32,column,profiles);if kind==0{list.sort_by_key(|id|products.iter().find(|p|p.id==*id).unwrap().duration);}
   let selected=select_ranked(&list,2500-scenario as i32*10,1+scenario as i32*17).unwrap();values.extend([scenario as u32,u32::from(selected),list.len()as u32,hash(list.iter().map(|&id|u32::from(id)))]);
  }
  assert_eq!(hash(values),396661753,"ZNativeFactoryChoicesOracle real16310/41a310/165d0/ratio;1a9b0 snapshot refresh isolated");
 }
}
