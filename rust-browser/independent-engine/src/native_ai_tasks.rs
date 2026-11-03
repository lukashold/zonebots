//! Native strategic task kernels, reconstructed from the owned GOG executable.
//!
//! Covers scripted16b70 tasks, reserve16e10/16d70 selection/release, attack
//! collection/allocation/reinforcement, defense, capture arrival and the ordered
//! per-group15000 phases. Post15000 includes building attack150f0, stale pickup
//! release152c0, retries15450, personal rejoin158f0, defense15950, pickup support
//!15c00 and idle frontier15790. Even-callback fort13b50 and hardware197d0 passes
//! are separate APIs. Numerical priorities and raw native member gates are
//! supplied explicitly. Successful4c0b0 group splitting, native movement/path
//! dispatch, and4e910's formation solver remain external systems.

#[derive(Clone,Debug)]
pub struct Sector {
    pub id:usize,
    pub native_slot:usize,
    pub valid:bool,
    pub rect:[i16;4],
    pub owner:i8,
    pub flag:[u8;2],
    pub flags:u8,
    pub human_strength:i16,
    pub priority:i32,
    /// Result of00413ca0 when enabled, otherwise max(priority,0).
    pub rank:i32,
}
#[derive(Clone,Debug)]
pub struct Leader {
    pub native_slot:usize,
    pub team:u8,
    pub runtime_class:u8,
    pub family:u8,
    pub passenger_class:u8,
    pub position:[i16;2],
    /// Native unit+9c has neither bit0x04 nor0x10; unit+aa is zero.
    pub accepts_orders:bool,
    /// Legacy field name: native00411b90 sums roster grenade bytes and this
    /// boolean is the LOW8 of that signed16 sum !=0 (native AL test).
    /// It is not a path/terrain predicate.
    pub laser_can_cross:bool,
}
#[derive(Clone,Debug)]
pub struct Group {
    pub id:usize,
    pub active:bool,
    pub flags:u8,
    pub task:i8,
    pub current_sector:Option<usize>,
    pub via_sector:Option<usize>,
    pub assigned_sector:Option<usize>,
    pub value:i32,
    pub leader:Option<Leader>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Order {pub group:usize,pub sector:usize,pub metadata:i16}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct DirectOrder{pub group:usize,pub position:[i16;2],pub metadata:i16,pub append:bool,pub dispatch:DirectDispatch}
///14bd0 and14c60 replace a leader path without issuing a new45ef00 order.
///Their distinction matters because14c60 also checks raw71bit1 before52810.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum DirectDispatch{Order,AssaultStart,AssaultMobilize}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Movement{Sector(Order),Direct(DirectOrder)}
#[derive(Clone,Debug,Default)]
pub struct Changes{pub sectors:Vec<Order>,pub direct:Vec<DirectOrder>,pub sequence:Vec<Movement>}
///56640/565a0 reads the cached nine UNIT buckets in source order, then tests
///current signed16-bit centers. No active/team/phase/hidden filter is applied.
///Indices outside the256-slot plane are an explicit unavailable-memory bound.
pub fn rectangle_units(buckets:&[Vec<usize>;256],rect:[i16;4],mut unit:impl FnMut(usize)->Option<([i16;2],u8)>)->Vec<usize>{
 let base=(rect[1]>>3&!15).wrapping_add(rect[0]>>7);let mut result=Vec::new();for delta in[-17i16,-16,-15,-1,0,1,15,16,17]{let bucket=base.wrapping_add(delta);let Some(slots)=usize::try_from(bucket).ok().and_then(|i|buckets.get(i))else{continue;};for &slot in slots.iter().take(32){let Some((p,class))=unit(slot)else{continue;};let offset=if class==1{16}else{8};let center=[p[0].wrapping_add(offset),p[1].wrapping_add(offset)];if center[0]>=rect[0]&&center[0]<=rect[2]&&center[1]>=rect[1]&&center[1]<=rect[3]{result.push(slot);}}}result
}
/// External native roster/member predicates used by00414bd0/00414c60. The caller
/// must derive these from unit/group state; assuming them is an explicit adapter.
#[derive(Clone,Copy,Debug)]
pub struct AssaultControl{
    pub group:usize,
    /// All members have the same8a&0x20 bit, and !(leader+9c&4).
    pub can_capture:bool,
    /// Native member flag0x20 gate and !(leader+9c&4).
    pub can_mobilize:bool,
}
#[derive(Clone,Copy,Debug)]
pub struct CaptureMember{
    pub native_slot:usize,
    /// Unit+5e: leader=-1, follower>0, individual=0.
    pub role:i8,
    /// Unit+8a&4: personal object/capture route is already running.
    pub personal_capture:bool,
    /// Unit+31&0x40.
    pub special_path:bool,
    ///00454070||004540b0: roster8a&0x20 flags are uniform.
    pub can_start_path:bool,
    /// Unit+9c&4.
    pub direct_blocked:bool,
}
#[derive(Clone,Debug)]
pub struct CaptureRuntime{
    pub group:usize,
    /// Original roster order, including empty slots, up to21 entries.
    pub roster:Vec<Option<CaptureMember>>,
    /// Leader+31&0x20.
    pub leader_overlay:bool,
    /// Legacy name: native00413670 checks entry to622d58, the enemy fort.
    pub entering_cpu_fort:bool,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PersonalCapture{
    pub group:usize,pub member:usize,pub position:[i16;2],pub personal:bool,
    /// Native004d02ba==0: link the new waypoint to the existing path.
    pub preserve_path:bool,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct LocalGoal{pub group:usize,pub position:[i16;2]}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Arrival {
    /// Native14a80 next invokes hardware14830 and defense14720 immediately.
    Captured{group:usize,sector:usize},
    /// Native14a10 leader command, distinct from19430 sector navigation.
    Capture{group:usize,member:usize,sector:usize},
}
/// Additional native roster fields used after15000. Keep unavailable values
/// explicit instead of substituting movement distance or browser selection state.
#[derive(Clone,Debug)]
pub struct GroupRuntime {
    pub group:usize,
    pub member_count:i8,
    pub combat_ttl:u8,
    pub native_order:i8,
    pub pending:bool,
    pub leader_role:i8,
    pub leader_flags31:u8,
}
#[derive(Clone,Debug)]
pub struct BuildingRecord {
    pub id:usize,
    pub position:[i16;2],
    pub kind:i8,
    pub owner:i8,
    pub sector:Option<usize>,
    /// Runtime BUILD+0x1e, loaded from BUILD disk+0x1c.
    pub hit_points:i16,
    /// Signed descriptor00498c78[kind*76]+0x32/+0x33.
    pub attack_offset:[i8;2],
}
#[derive(Clone,Debug)]
pub struct WorldObject {
    pub id:usize,
    pub position:[i16;2],
    /// None means native OBJECT+4 is null; Some is its metadata table index.
    pub definition:Option<i16>,
    pub flags13:u8,
    pub flags26:u8,
    pub sector:Option<usize>,
}
#[derive(Clone,Debug)]
pub struct RuntimeUnit {
    pub native_slot:usize,
    pub position:[i16;2],
    pub flags31:u8,
    pub flags8a:u8,
    pub sector:Option<usize>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct BuildingAttack {pub group:usize,pub member:usize,pub building:usize,pub position:[i16;2]}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct AvailableHardware {
    pub building:Option<usize>,pub object:Option<usize>,pub position:[i16;2],
    ///17900 result bits:1 living non-CPU BUILD,2 matching world OBJECT. Both may
    /// be set; a world object overrides the target pointer/position.
    pub flags:u8,pub target_flags26:u8,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct HardwareCapture {
    pub group:usize,
    ///4c0b0 attempts to detach this last follower into a new group. The actual
    /// allocator result is external; a null4c0b0 return uses fallback_member.
    pub split_member:Option<usize>,
    pub fallback_member:usize,
    pub object:usize,
    pub position:[i16;2],
}

fn group_runtime(runtime:&[GroupRuntime],id:usize)->Option<&GroupRuntime>{runtime.iter().find(|r|r.group==id)}
fn hardware_definition(definition:Option<i16>)->bool{matches!(definition,Some(5|8|21|22|23|24|25|29|31|33|35|37))}
fn compatible_frontier(leader:&Leader,s:&Sector)->bool{
    !(s.flags&0x20!=0&&leader.runtime_class==1
        ||s.flags&0x40!=0&&matches!(leader.runtime_class,0|15|16)&&!leader.laser_can_cross)
}

///00417900's first living non-CPU BUILD followed by first matching OBJECT scan. No
/// distance ranking is used. The object switch intentionally includes Dead
/// Heavy25 and Fire Truck37, while excluding Heavy26.
pub fn available_hardware(target:usize,buildings:&[BuildingRecord],objects:&[WorldObject])->Option<AvailableHardware>{
    let mut result=buildings.iter().find(|b|b.kind!=-1&&b.hit_points!=0&&b.owner!=1&&b.sector==Some(target)).map(|b|
        AvailableHardware{building:Some(b.id),object:None,position:b.position,flags:1,target_flags26:0});
    if let Some(o)=objects.iter().find(|o|o.position[0]>0&&hardware_definition(o.definition)&&o.sector==Some(target)){
        let flags=result.as_ref().map_or(2,|r|r.flags|2);
        result=Some(AvailableHardware{building:None,object:Some(o.id),position:o.position,flags,target_flags26:o.flags26});
    }
    result
}

///00417a50, called during17b30 initialization, not every strategic callback.
/// Preserve authored0x20/0x80 and the existing0x40 until170a0 recomputes only
/// that bit. Caller also resets native sector+0x15 toFF; it is not represented
/// by Sector here. Fort slots are17400 of each of the four raw fort bases.
pub fn initialize_sector_flags(sectors:&mut[Sector],fort_sectors:[Option<usize>;4],buildings:&[BuildingRecord],objects:&[WorldObject]){
    for s in sectors{
        s.flags&=0xe0;
        for(index,&fort)in fort_sectors.iter().enumerate(){if fort==Some(s.id){s.flags|=1<<index;}}
        if available_hardware(s.id,buildings,objects).is_some(){s.flags|=16;}
    }
}

///0044ea30 invokes this after a flag owner change and before162a0/1d280 and
///161b0. Any controlled task1 group targeting this via sector restarts task0.
pub fn reset_captured_task_phases(groups:&mut[Group],target:usize){
    for g in groups.iter_mut().filter(|g|controlled(g)&&g.task==1&&g.via_sector==Some(target)){g.task=0;}
}

///004161b0, after capture updates flag objects and production. The level value
/// is raw native622c7a (622cbc in the alternate callback), not a UI label.
pub fn clear_hardware_capture_restrictions(objects:&mut[WorldObject],target:usize,native_level:i16)->Vec<usize>{
    let mut changed=Vec::new();if native_level==0x12{return changed;}
    for o in objects.iter_mut().filter(|o|o.position[0]>0&&hardware_definition(o.definition)&&o.sector==Some(target)){
        if o.flags26&1!=0{changed.push(o.id);}o.flags26&=!1;
    }
    changed
}

///00414830 up to its native group-allocation boundary. Clears obsolete task5
/// hardware leaders first, preserving the native early return when another
/// infantry group is already fetching hardware in the same sector. A caller
/// must execute the4c0b0 split and native movement before committing its result.
pub fn hardware_capture_for_group(groups:&mut[Group],index:usize,sectors:&[Sector],runtime:&[GroupRuntime],captures:&[CaptureRuntime],buildings:&[BuildingRecord],objects:&[WorldObject])->Option<HardwareCapture>{
    let g=groups.get(index)?;let current=sector(sectors,g.current_sector)?.id;
    if !g.active||g.task==5||g.leader.as_ref()?.runtime_class==2{return None;}
    loop{
        let Some(other)=groups.iter().position(|other|controlled(other)&&other.task==5&&other.via_sector==Some(current))else{break;};
        let leader=groups[other].leader.as_ref().filter(|l|l.team==1)?;
        if !matches!(leader.runtime_class,1|2){return None;}
        let other=&mut groups[other];other.task=-1;other.via_sector=None;other.assigned_sector=None;
    }
    let g=&groups[index];let l=g.leader.as_ref()?;let rt=group_runtime(runtime,g.id)?;
    if !matches!(g.task,-1|4)&&rt.member_count<=1||g.flags&4!=0||g.flags&8==0{return None;}
    let s=sector(sectors,Some(current))?;if s.flags&16==0||s.owner!=1{return None;}
    let target=available_hardware(current,buildings,objects)?;
    if target.flags&1!=0&&(l.runtime_class==1||target.flags&2==0)||l.runtime_class==1{return None;}
    if target.target_flags26&1!=0&&l.runtime_class!=3&&!l.laser_can_cross{return None;}
    let split_member=if rt.member_count>1{captures.iter().find(|r|r.group==g.id).and_then(|r|r.roster.iter().take(21).skip(1).rev().flatten().next()).map(|m|m.native_slot)}else{None};
    Some(HardwareCapture{group:g.id,split_member,fallback_member:l.native_slot,object:target.object?,position:target.position})
}

///00416070 and14830's successful post-allocation writes for the selected group.
/// The selected member's present sector is supplied after the actual split; it
/// may differ from the original leader's sector at a formation boundary.
pub fn commit_hardware_capture(group:&mut Group,current_member_sector:Option<usize>,member_role:i8,member_flags31:u8)->bool{
    if member_role==0{return false;}
    group.active=false;if member_flags31&3!=1||member_flags31&4==0{return false;}
    group.active=true;group.flags|=0x1c;group.task=5;group.via_sector=current_member_sector;true
}

///004152c0. Native task5 can also originate from15a30's supporting orders;
/// it ends as soon as no qualifying OBJECT remains in its via sector. Only via
/// and task are cleared: assigned pointer and flags deliberately survive.
pub fn retire_missing_hardware(groups:&mut[Group],objects:&[WorldObject])->Vec<usize>{
    let mut retired=Vec::new();
    for g in groups.iter_mut().filter(|g|g.active&&g.task==5){
        if !objects.iter().any(|o|o.position[0]>0&&hardware_definition(o.definition)&&o.sector==g.via_sector){
            g.via_sector=None;g.task=-1;retired.push(g.id);
        }
    }
    retired
}

///004158f0. Return the units for immediate native4e910 formation rejoin; its
/// path/formation solver remains the caller's responsibility. This native pass
/// uses raw31bit4, not a separate browser-active or ownership classification.
pub fn finish_personal_routes(units:&mut[RuntimeUnit],sectors:&[Sector])->Vec<usize>{
    let mut rejoin=Vec::new();
    for u in units{
        if u.flags8a&4!=0&&u.flags31&4!=0&&sector(sectors,u.sector).is_some_and(|s|s.owner==1){
            u.flags8a&=!4;rejoin.push(u.native_slot);
        }
    }
    rejoin
}

///004150f0/004150a0. Attack the first valid BUILD in an occupied target sector
/// while a non-CPU unit is in the inclusive128-pixel flag box. Caller supplies
/// the native56640 query results (its linked spatial index has its own filters).
pub fn attack_target_buildings(groups:&mut[Group],sectors:&[Sector],buildings:&[BuildingRecord],query_results:&[(usize,Vec<RuntimeUnit>)])->Vec<BuildingAttack>{
    let mut attacks=Vec::new();
    for g in groups.iter_mut().filter(|g|controlled(g)&&matches!(g.task,1|3)&&g.current_sector==g.assigned_sector){
        let Some(s)=sector(sectors,g.current_sector).filter(|s|s.flags&3==0&&s.owner==0)else{continue;};
        let Some(l)=g.leader.as_ref().filter(|l|l.team==1)else{continue;};
        let capable=if matches!(l.runtime_class,0|16){l.laser_can_cross}else{matches!(l.runtime_class,3|15)||matches!(l.family,6|24|25|26|27|30)};
        if !capable{continue;}
        let x=(i16::from(s.flag[0]as i8).wrapping_sub(4)).wrapping_mul(16);let y=(i16::from(s.flag[1]as i8).wrapping_sub(4)).wrapping_mul(16);
        let Some((_,units))=query_results.iter().find(|(id,_)|*id==g.id)else{continue;};
        if !units.iter().any(|u|u.flags31&3!=1&&u.position[0]>=x&&i32::from(u.position[0])<=i32::from(x)+128&&u.position[1]>=y&&i32::from(u.position[1])<=i32::from(y)+128){continue;}
        let Some(b)=buildings.iter().find(|b|b.position[0]>=0&&b.kind>=0&&b.sector==Some(s.id))else{continue;};
        attacks.push(BuildingAttack{group:g.id,member:l.native_slot,building:b.id,position:[b.position[0].wrapping_add(i16::from(b.attack_offset[0])),b.position[1].wrapping_add(i16::from(b.attack_offset[1]))]});g.task=6;
    }
    attacks
}

///00415510's unique frontier list, in native owned-sector/neighbor slot order.
/// A non-owned fort may be a frontier target; only the owned origin excludes
/// flags1/2. RNG is consumed once iff more than one candidate exists.
pub fn frontier_candidates(sectors:&[Sector],leader:&Leader)->Vec<usize>{
    let mut targets=Vec::new();
    for owned in sectors.iter().filter(|s|s.valid&&s.owner==1&&s.flags&3==0&&compatible_frontier(leader,s)){
        for next in sectors.iter().filter(|s|s.valid&&s.owner!=1&&adjacent(owned,s)&&compatible_frontier(leader,s)){
            if !targets.contains(&next.id){targets.push(next.id);}
        }
    }
    targets
}

///00415790/00415510. Task10 is an idle interior force's frontier move after two
/// counted callbacks. Pending jobs reset the counter; nonzero native orders
/// preserve it. A missing/wrong-team leader returns from the ENTIRE native pass.
pub fn assign_idle_frontiers(groups:&mut[Group],sectors:&[Sector],runtime:&[GroupRuntime],counters:&mut[i8],mut rng:impl FnMut()->u32)->Vec<DirectOrder>{
    let mut orders=Vec::new();
    for(g,count)in groups.iter_mut().zip(counters){
        if !controlled(g){continue;}
        if g.task!=-1{*count=0;continue;}
        let Some(current)=sector(sectors,g.current_sector)else{continue;};
        if sectors.iter().any(|s|adjacent(current,s)&&s.owner!=1){*count=0;continue;}
        let Some(l)=g.leader.as_ref().filter(|l|l.team==1)else{break;};
        let Some(rt)=group_runtime(runtime,g.id)else{continue;};
        if rt.native_order!=0{continue;}
        if rt.pending{*count=0;continue;}
        *count=count.wrapping_add(1);if *count<2{continue;}
        let candidates=frontier_candidates(sectors,l);if candidates.is_empty(){continue;}
        let slot=if candidates.len()==1{0}else{(rng()as u16 as usize)%candidates.len()};
        let target=sector(sectors,Some(candidates[slot])).unwrap();
        let position=[(i16::from(target.flag[0]as i8)<<4).wrapping_add(8),(i16::from(target.flag[1]as i8)<<4).wrapping_add(8)];
        orders.push(DirectOrder{group:g.id,position,metadata:0,append:false,dispatch:DirectDispatch::Order});g.task=10;g.assigned_sector=Some(target.id);*count=0;
    }
    orders
}

///00415c00/00415a30. Idle infantry supports a pickup already targeted by task5
/// in that sector. The pass stops at its first eligible leader whose15a30 fails,
/// but continues after successful commands. RNG consumption is per considered
/// object unless sector+29bit1 is set. Raw member role/31 flags gate issuance.
pub fn support_hardware_orders(groups:&mut[Group],sectors:&[Sector],runtime:&[GroupRuntime],objects:&[WorldObject],sector_flags29:&[(usize,u8)],mut rng:impl FnMut()->u32)->Vec<DirectOrder>{
    let mut orders=Vec::new();
    for index in 0..groups.len(){
        let g=&groups[index];if !controlled(g)||g.task!=-1{continue;}
        let Some(l)=g.leader.as_ref().filter(|l|l.team==1&&matches!(l.runtime_class,0|3|15|16))else{continue;};
        if !l.accepts_orders{break;}
        let mut chosen=None;let mut distance=i32::MAX;
        for o in objects.iter().filter(|o|o.position[0]>=0&&o.flags13&1==0&&o.flags26&1==0&&hardware_definition(o.definition)){
            let Some(s)=sector(sectors,o.sector)else{continue;};
            if !groups.iter().any(|other|controlled(other)&&other.task==5&&other.via_sector==Some(s.id)){continue;}
            if sector_flags29.iter().find(|(id,_)|*id==s.id).is_none_or(|(_,f)|f&1==0)&&rng()&1==0{continue;}
            let dx=i32::from(l.position[0])-i32::from(o.position[0]);let dy=i32::from(l.position[1])-i32::from(o.position[1]);let next=dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
            if next<distance{distance=next;chosen=Some(o);}
        }
        let Some(o)=chosen else{break;};let Some(rt)=group_runtime(runtime,g.id)else{break;};
        if rt.leader_role==0{break;}
        let g=&mut groups[index];g.active=false;
        if rt.leader_flags31&3!=1||rt.leader_flags31&4==0{break;}
        g.active=true;g.task=5;g.via_sector=o.sector;
        orders.push(DirectOrder{group:g.id,position:o.position,metadata:0,append:false,dispatch:DirectDispatch::Order});
    }
    orders
}

///00413670. Goal/path coordinates are native signed16-bit values. The second
/// accepted path node is the linked waypoint120 pixels north of the fort gate.
pub fn entering_cpu_fort(native_order:i8,group_goal:Option<[i16;2]>,path_head:Option<[i16;2]>,enemy_fort:[i16;2])->bool{
    let x=enemy_fort[0].wrapping_add(80);let y=enemy_fort[1].wrapping_add(72);
    native_order==1&&group_goal==Some([x,y])||path_head.is_some_and(|p|p[0]==x&&(p[1]==y||p[1]==y.wrapping_sub(120)))
}

fn controlled(group:&Group)->bool{group.active&&group.flags&0x18==0x18}
fn sector<'a>(sectors:&'a[Sector],id:Option<usize>)->Option<&'a Sector>{id.and_then(|id|sectors.iter().find(|s|s.id==id&&s.valid))}

///00416b70 consumes active CPUPLR spawn records in native roster order. Their
/// optional sector numbers are1-based, with0 leaving the group unassigned.
pub fn initial_tasks(groups:&mut[Group],sector_numbers:&[u8],sectors:&[Sector])->Vec<Order>{
    let mut orders=Vec::new();let mut entries=sector_numbers.iter();
    for g in groups{
        g.active=g.leader.as_ref().is_some_and(|l|l.team==1);
        if !g.active{continue;}
        g.task=-1;
        let Some(&number)=entries.next()else{continue;};
        if number==0{continue;}
        let Some(s)=sectors.iter().find(|s|s.native_slot==usize::from(number-1)&&s.valid)else{continue;};let id=s.id;
        g.flags|=0x18;g.task=0;g.via_sector=Some(id);g.assigned_sector=Some(id);
        orders.push(Order{group:g.id,sector:id,metadata:0});
    }
    orders
}

///004162a0 updates the current sector separately from the pending task target.
/// Runtime class2 fixed guns are removed from the mobile task candidate set.
pub fn refresh_group(g:&mut Group,current_sector:Option<usize>){
    if !g.active{return;}
    g.flags&=!8;
    if let Some(l)=&g.leader{if l.team==1{
        if l.runtime_class!=2{g.flags|=8;}
        if current_sector.is_some(){g.current_sector=current_sector;}
    }}
}

///00416e10's final flag-selection loop. Values include the native-1 exclusion
/// marker. Return groups that invoke00416d70 when their prior selection expires;
/// the caller owns that release's task/route side effects and fallback sector.
pub fn select_budget_groups(groups:&mut[Group],values:&[i32],budget:f32)->Vec<usize>{
    let mut released=Vec::new();let mut running=0i32;let threshold=budget as i32;
    for (g,&value)in groups.iter_mut().zip(values){
        if !g.active{continue;}
        let old=g.flags;g.flags&=!16;let next=running.wrapping_add(value);
        if next>threshold{if old&16!=0{released.push(g.id);}running=next;}
        else if value!=-1&&old&8!=0{g.flags|=16;}
        else{running=next;}
    }
    released
}

///00416d70. The fallback is native004a84c4, the bit1/622d54 own-fort sector, not
/// an inferred nearest owned flag. A task1 group in the bit2/622d58 enemy fort keeps
/// flag0x10; other released attack forces restart their shared target phases.
pub fn release_budget_groups(groups:&mut[Group],released:&[usize],sectors:&[Sector],fallback:Option<usize>)->Changes{
    let mut changes=Changes::default();
    for &id in released{
        let Some(index)=groups.iter().position(|g|g.id==id&&g.active)else{continue;};
        if groups[index].task==1&&sector(sectors,groups[index].current_sector).is_some_and(|s|s.flags&2!=0){groups[index].flags|=16;continue;}
        if matches!(groups[index].task,0|1|2|3){if let Some(target)=sector(sectors,groups[index].assigned_sector){restart_attack_phases(groups,target.id);}}
        let g=&groups[index];let Some(l)=g.leader.as_ref().filter(|l|l.team==1&&l.runtime_class!=2)else{continue;};
        let goal=sector(sectors,fallback).filter(|s|!(s.flags&0x20!=0&&l.runtime_class==1||s.flags&0x40!=0&&matches!(l.runtime_class,0|15|16)&&!l.laser_can_cross));
        if let Some(s)=goal{let order=Order{group:g.id,sector:s.id,metadata:0};changes.sequence.push(Movement::Sector(order.clone()));changes.sectors.push(order);}
        else{let order=DirectOrder{group:g.id,position:l.position,metadata:0,append:false,dispatch:DirectDispatch::Order};changes.sequence.push(Movement::Direct(order.clone()));changes.direct.push(order);}
    }
    changes
}

///00414a80's captured-target transition. Native hardware pickup follow-up
/// (00414830/00414720) is a separate caller responsibility, identified by IDs.
/// Keep both sector pointers: the native transition only changes the task byte.
pub fn retire_captured_targets(groups:&mut[Group],sectors:&[Sector])->Vec<usize>{
    let mut completed=Vec::new();
    for g in groups{
        if controlled(g)&&matches!(g.task,0|1|2|3|6)&&sector(sectors,g.assigned_sector).is_some_and(|s|s.owner==1){
            g.task=-1;completed.push(g.id);
        }
    }
    completed
}

///00414a80 for one slot in15000's original interleaved group loop. Combat TTLs
/// are native roster+0x10, not a distance or path-idle estimate. The first task1
/// group in roster order determines whether another arrived group joins capture.
pub fn capture_arrival_for_group(groups:&mut[Group],index:usize,sectors:&[Sector],combat_ttls:&[(usize,u8)])->Option<Arrival>{
    let g=groups.get(index)?;
    if !controlled(g)||!matches!(g.task,0|1|2|3|6){return None;}
    let target=sector(sectors,g.assigned_sector)?;
    let group=g.id;let target_id=target.id;
    if target.owner==1{groups[index].task=-1;return Some(Arrival::Captured{group,sector:target_id});}
    if g.current_sector!=Some(target_id){return None;}
    let capturing=groups.iter().find(|other|controlled(other)&&other.task==1&&other.via_sector==Some(target_id))?;
    if combat_ttls.iter().find(|(id,_)|*id==capturing.id)?.1!=0{return None;}
    let member=g.leader.as_ref().filter(|l|l.team==1)?.native_slot;
    if matches!(g.task,1|3){return None;}
    let g=&mut groups[index];g.task=1;g.via_sector=Some(target_id);
    Some(Arrival::Capture{group,member,sector:target_id})
}

///00415450. `can_retry` is the native conjunction groupOrder==0,
/// !0041a0d0, valid enemy leader, and current path pointer==0. The counters are
/// held separately because they are strategyGroup+0x14, not the task byte.
pub fn retry_stalled_tasks(groups:&mut[Group],counters:&mut[i8],can_retry:&[bool])->Vec<usize>{
    let mut released=Vec::new();
    for((g,count),&can_retry)in groups.iter_mut().zip(counters).zip(can_retry){
        if !controlled(g){continue;}
        if !matches!(g.task,-1|2)&&can_retry{
            *count=count.wrapping_add(1);
            if *count<3{continue;}
            g.task=-1;g.via_sector=None;g.assigned_sector=None;released.push(g.id);
        }
        *count=0;
    }
    released
}

fn requirement(requirements:&[(usize,i32)],id:usize)->i32{requirements.iter().find(|r|r.0==id).map(|r|r.1).unwrap_or(0)}

///00414570. Caller supplies00415950's exact float-derived defense priorities.
pub fn cancel_unneeded_defenses(groups:&mut[Group],sectors:&[Sector],requirements:&[(usize,i32)]){
    for g in groups.iter_mut().filter(|g|controlled(g)&&g.task==4){
        if !sector(sectors,g.via_sector).is_some_and(|s|s.owner==1&&requirement(requirements,s.id)>=500){g.via_sector=None;g.task=-1;}
    }
}

///00414610 counts every active owned fixed gun, even when it stands in another
/// sector. Other groups contribute only to their assigned defense sector.
pub fn defense_strength(groups:&[Group],target:usize)->i32{
    groups.iter().filter(|g|g.active&&g.leader.as_ref().is_some_and(|l|l.team==1&&
        (l.runtime_class==2||controlled(g)&&g.task==4&&g.via_sector==Some(target))))
        .fold(0i32,|n,g|n.wrapping_add(g.value))
}
fn defend(g:&mut Group,target:usize)->Order{
    g.task=4;g.via_sector=Some(target);g.assigned_sector=None;Order{group:g.id,sector:target,metadata:0}
}

///00415950. Only the first qualifying sector is processed per strategic cycle.
/// Although the prefix strength test may inspect multiple groups, the native
/// routine assigns only the first candidate to task4 (00415a1c/004146c0).
pub fn assign_defense(groups:&mut[Group],sectors:&[Sector],requirements:&[(usize,i32)],neighbor_only:bool)->Vec<Order>{
    for target in sectors.iter().filter(|s|s.valid&&s.owner==1&&s.flags&3==0){
        let requested=requirement(requirements,target.id);let existing=defense_strength(groups,target.id);
        if requested<500||existing>requested{continue;}
        let candidates=candidate_groups(groups,target,sectors,neighbor_only);
        if prefix(groups,&candidates,requested.wrapping_sub(existing)).is_some(){
            return vec![defend(&mut groups[candidates[0]],target.id)];
        }
        return Vec::new();
    }
    Vec::new()
}

///00414720, called after a sector becomes owned. Native selection always takes
/// the first idle resident, then continues through roster order until a later
/// group's value strictly exceeds the remaining requirement.
pub fn defend_captured_sector(groups:&mut[Group],target:&Sector,requested:i32)->Vec<Order>{
    if target.priority<500||requested==0||defense_strength(groups,target.id)!=0{return Vec::new();}
    let mut orders=Vec::new();let mut remaining=requested;
    for g in groups.iter_mut().filter(|g|controlled(g)&&g.current_sector==Some(target.id)&&g.task==-1){
        let last=!orders.is_empty()&&remaining.wrapping_sub(g.value)<0;
        remaining=remaining.wrapping_sub(g.value);orders.push(defend(g,target.id));if last{break;}
    }
    orders
}

///00413900 uses a signed16-bit accumulator, despite the32-bit group values.
pub fn committed_strength(groups:&[Group],target:usize)->i32{
    i32::from(groups.iter().filter(|g|controlled(g)&&!matches!(g.task,-1|4|5)&&g.assigned_sector==Some(target))
        .fold(0i16,|sum,g|sum.wrapping_add(g.value as i16)))
}
fn restart_attack_phases(groups:&mut[Group],target:usize){
    //00416d30 accepts either flag0x08 OR0x10; it does not require both.
    for g in groups.iter_mut().filter(|g|g.active&&g.flags&0x18!=0&&!matches!(g.task,-1|4|5)&&g.assigned_sector==Some(target)){g.task=0;}
}
fn release_target(groups:&mut[Group],target:usize,sectors:&[Sector])->Changes{
    let mut changes=Changes::default();
    for g in groups.iter_mut().filter(|g|controlled(g)&&g.assigned_sector==Some(target)){
        if let Some(l)=&g.leader{
            let compatible=sector(sectors,g.current_sector).filter(|s|
                !(s.flags&0x20!=0&&l.runtime_class==1||s.flags&0x40!=0&&matches!(l.runtime_class,0|15|16)&&!l.laser_can_cross));
            if let Some(s)=compatible{let order=Order{group:g.id,sector:s.id,metadata:0};changes.sequence.push(Movement::Sector(order.clone()));changes.sectors.push(order);}
            else{let order=DirectOrder{group:g.id,position:[l.position[0].wrapping_add(8),l.position[1].wrapping_add(8)],metadata:0,append:false,dispatch:DirectDispatch::Order};changes.sequence.push(Movement::Direct(order.clone()));changes.direct.push(order);}
        }
        g.task=-1;g.via_sector=None;g.assigned_sector=None;
    }
    changes
}

///00414500/00415c90: strengthen existing task0 assignments before allocating
/// further targets. If no feasible reinforcement can be selected, release that
/// entire target force to its current sectors. No alternative target is invented.
pub fn reinforce_attacks(groups:&mut[Group],sectors:&[Sector],neighbor_only:bool)->Changes{
    let mut changes=Changes::default();
    for index in 0..groups.len(){
        if !controlled(&groups[index])||groups[index].task!=0{continue;}
        let Some(target)=sector(sectors,groups[index].assigned_sector)else{continue;};
        if required(target)<=committed_strength(groups,target.id){continue;}
        restart_attack_phases(groups,target.id);
        if !candidate_groups(groups,target,sectors,neighbor_only).is_empty(){
            if let Some(orders)=allocate_target(groups,target,sectors,neighbor_only){changes.sequence.extend(orders.iter().cloned().map(Movement::Sector));changes.sectors.extend(orders);continue;}
        }
        let released=release_target(groups,target.id,sectors);changes.sequence.extend(released.sequence);changes.sectors.extend(released.sectors);changes.direct.extend(released.direct);
    }
    changes
}

fn capture_position(target:&Sector,enemy_fort:[i16;2])->[i16;2]{
    if target.flags&2!=0{[enemy_fort[0].wrapping_add(80),enemy_fort[1].wrapping_add(72)]}
    else{[(i16::from(target.flag[0]as i8)<<4).wrapping_add(4),(i16::from(target.flag[1]as i8)<<4).wrapping_add(4)]}
}
fn control(controls:&[AssaultControl],id:usize)->AssaultControl{
    controls.iter().find(|c|c.group==id).copied().unwrap_or(AssaultControl{group:id,can_capture:false,can_mobilize:false})
}
fn other_task(groups:&[Group],excluded:usize,target:usize,tasks:&[i8])->bool{
    groups.iter().enumerate().any(|(i,g)|i!=excluded&&controlled(g)&&g.assigned_sector==Some(target)&&tasks.contains(&g.task))
}
fn ready_staging_force(groups:&[Group],target:&Sector)->bool{
    let strength=groups.iter().filter(|g|controlled(g)&&g.task==2&&g.assigned_sector==Some(target.id)).fold(0i32,|n,g|n.wrapping_add(g.value));
    if target.human_strength==0{strength>0}else{strength>=required(target)}
}
fn mobilize(groups:&mut[Group],target:&Sector,controls:&[AssaultControl],enemy_fort:[i16;2],changes:&mut Changes){
    //00414c60 always takes all task0 groups before processing task2 groups.
    for phase in [0,2]{for g in groups.iter_mut().filter(|g|controlled(g)&&g.task==phase&&g.assigned_sector==Some(target.id)){
        g.task=3;
        if control(controls,g.id).can_mobilize{let order=DirectOrder{group:g.id,position:capture_position(target,enemy_fort),metadata:0,append:false,dispatch:DirectDispatch::AssaultMobilize};changes.sequence.push(Movement::Direct(order.clone()));changes.direct.push(order);}
    }}
}

///00414e50/00414b60: stage supporting groups in adjacent sectors, then mobilize
/// task0/task2 forces once the staged strength meets the full requirement. This
/// is distinct from142d0's initial half-strength allocation. `controls` covers
/// native roster/member checks; individual capture-member selection13740 remains
/// outside this kernel.
pub fn advance_assaults(groups:&mut[Group],sectors:&[Sector],controls:&[AssaultControl],enemy_fort:[i16;2])->Changes{
    let count=groups.len();advance_assault_indices(groups,sectors,controls,enemy_fort,0..count)
}

///00414e50 for one slot, preserving15000's interleaving with capture/pickup.
pub fn advance_assault_for_group(groups:&mut[Group],index:usize,sectors:&[Sector],controls:&[AssaultControl],enemy_fort:[i16;2])->Changes{
    advance_assault_indices(groups,sectors,controls,enemy_fort,std::iter::once(index))
}
fn advance_assault_indices(groups:&mut[Group],sectors:&[Sector],controls:&[AssaultControl],enemy_fort:[i16;2],indices:impl IntoIterator<Item=usize>)->Changes{
    let mut changes=Changes::default();
    for index in indices{
        if index>=groups.len(){continue;}
        if !controlled(&groups[index]){continue;}
        let Some(target)=sector(sectors,groups[index].assigned_sector)else{continue;};
        if groups[index].task==0{
            if ready_staging_force(groups,target){mobilize(groups,target,controls,enemy_fort,&mut changes);continue;}
            let neighboring=sector(sectors,groups[index].current_sector).is_some_and(|current|adjacent(current,target));
            if !neighboring{continue;}
            if !other_task(groups,index,target.id,&[0,1]){
                groups[index].task=3;
                if control(controls,groups[index].id).can_capture{let order=DirectOrder{group:groups[index].id,position:capture_position(target,enemy_fort),metadata:0,append:false,dispatch:DirectDispatch::AssaultStart};changes.sequence.push(Movement::Direct(order.clone()));changes.direct.push(order);}
            }else{
                groups[index].task=2;groups[index].via_sector=groups[index].current_sector;
                if let Some(current)=groups[index].current_sector{let order=Order{group:groups[index].id,sector:current,metadata:0};changes.sequence.push(Movement::Sector(order.clone()));changes.sectors.push(order);}
            }
        }else if groups[index].task==2&&!other_task(groups,index,target.id,&[0]){
            let before=changes.sectors.len();fort_support(groups,target,sectors,&mut changes.sectors);
            changes.sequence.extend(changes.sectors[before..].iter().cloned().map(Movement::Sector));
            let order=Order{group:groups[index].id,sector:target.id,metadata:0};changes.sequence.push(Movement::Sector(order.clone()));changes.sectors.push(order);groups[index].task=3;
        }
    }
    changes
}

///004139e0/00413740: a nearby flag is normally assigned to the last surviving
/// follower, preserving the group's assault task. The leader sets task1 only
/// when it performs the capture itself. Raw member/path flags stay explicit;
/// callers without those states must identify their reconstruction as an adapter.
pub fn request_personal_captures(groups:&mut[Group],sectors:&[Sector],runtime:&[CaptureRuntime],enemy_fort:[i16;2])->Vec<PersonalCapture>{
    let count=groups.len();personal_capture_indices(groups,sectors,runtime,enemy_fort,0..count)
}

///004139e0 for one slot, preserving15000's interleaving with arrival/assault.
pub fn personal_capture_for_group(groups:&mut[Group],index:usize,sectors:&[Sector],runtime:&[CaptureRuntime],enemy_fort:[i16;2])->Vec<PersonalCapture>{
    personal_capture_indices(groups,sectors,runtime,enemy_fort,std::iter::once(index))
}
fn personal_capture_indices(groups:&mut[Group],sectors:&[Sector],runtime:&[CaptureRuntime],enemy_fort:[i16;2],indices:impl IntoIterator<Item=usize>)->Vec<PersonalCapture>{
    let mut requests=Vec::new();
    for index in indices{
        if index>=groups.len(){continue;}
        let g=&groups[index];if !controlled(g){continue;}
        let Some(current)=sector(sectors,g.current_sector)else{continue;};
        let Some(rt)=runtime.iter().find(|r|r.group==g.id)else{continue;};
        let Some(leader)=g.leader.as_ref().filter(|l|l.team==1)else{continue;};
        if current.flags&2!=0{
            if rt.entering_cpu_fort{continue;}
        }else{
            if matches!(g.task,1|-1|4|5){continue;}
            if g.task==3&&g.assigned_sector.is_some_and(|id|committed_strength(groups,id)!=g.value){continue;}
            if current.owner==1{continue;}
            if groups.iter().enumerate().any(|(i,other)|i!=index&&other.active&&other.current_sector==Some(current.id)&&other.task==1){continue;}
            if rt.roster.iter().flatten().any(|m|m.personal_capture){continue;}
            if current.flags&0x20!=0&&leader.runtime_class==1{continue;}
            if current.flags&0x40!=0&&(leader.family==32&&leader.passenger_class!=3||leader.family==34||leader.family==4&&!leader.laser_can_cross){continue;}
            let dx=(i16::from(current.flag[0]as i8)<<4).wrapping_sub(leader.position[0]);
            let dy=(i16::from(current.flag[1]as i8)<<4).wrapping_sub(leader.position[1]);
            if i32::from(dx).wrapping_mul(i32::from(dx)).wrapping_add(i32::from(dy).wrapping_mul(i32::from(dy)))>0x63ff{continue;}
        }
        if rt.leader_overlay{continue;}
        let chosen=if current.flags&2==0{rt.roster.iter().take(21).skip(1).rev().flatten().next().copied().or_else(||rt.roster.first().copied().flatten())}
            else{rt.roster.first().copied().flatten()};
        let Some(member)=chosen else{continue;};
        if !member.can_start_path||member.direct_blocked{continue;}
        let personal=!member.special_path&&member.role!=-1;
        if !member.special_path{
            let position=capture_position(current,enemy_fort);
            requests.push(PersonalCapture{group:g.id,member:member.native_slot,position,personal,preserve_path:false});
            if current.flags&2!=0{requests.push(PersonalCapture{group:g.id,member:member.native_slot,position:[position[0],position[1].wrapping_sub(120)],personal,preserve_path:true});}
        }
        if !personal&&member.role!=0{let g=&mut groups[index];g.task=1;g.via_sector=Some(current.id);}
    }
    requests
}

///00413b50's every-even-callback fort capture pass. Ordinary sector captures
/// are handled by the strategic15000 pass, not by this faster helper.
pub fn tactical_fort_captures(groups:&mut[Group],sectors:&[Sector],runtime:&[CaptureRuntime],enemy_fort:[i16;2])->Vec<PersonalCapture>{
    let allowed=runtime.iter().filter(|rt|groups.iter().find(|g|g.id==rt.group&&controlled(g))
        .is_some_and(|g|sector(sectors,g.current_sector).is_some_and(|s|s.flags&2!=0)&&!rt.entering_cpu_fort)).cloned().collect::<Vec<_>>();
    request_personal_captures(groups,sectors,&allowed,enemy_fort)
}

///004197d0, called every even simulation callback. This changes the hardware's
/// local movement segment without issuing a new group command or task. The
/// signed counter is strategyGroup+0x13, separate from15450's stall counter.
pub fn tactical_vehicle_goals(groups:&[Group],sectors:&[Sector],counters:&mut[i8])->Vec<LocalGoal>{
    let mut goals=Vec::new();
    for(g,counter)in groups.iter().zip(counters){
        if !g.active{continue;}
        let old=*counter;*counter=old.wrapping_sub(1);if *counter>=0{continue;}*counter=old;
        let Some(s)=sector(sectors,g.current_sector).filter(|s|s.owner!=1&&s.flags&3==0)else{continue;};
        let Some(l)=g.leader.as_ref().filter(|l|l.team==1&&l.family!=4&&l.runtime_class!=2)else{continue;};
        let position=[i16::from(s.flag[0]as i8)<<4,i16::from(s.flag[1]as i8)<<4];
        let dx=position[0].wrapping_sub(l.position[0]);let dy=position[1].wrapping_sub(l.position[1]);
        if i32::from(dx).wrapping_mul(i32::from(dx)).wrapping_add(i32::from(dy).wrapping_mul(i32::from(dy)))<0x4000{goals.push(LocalGoal{group:g.id,position});*counter=50;}
    }
    goals
}

///00418a50: inclusive rectangle boundaries; corner contact counts as adjacency.
pub fn adjacent(a:&Sector,b:&Sector)->bool{
    if !a.valid||!b.valid||a.id==b.id{return false;}
    let[x1,y1,x2,y2]=a.rect;let[bx1,by1,bx2,by2]=b.rect;
    if x1==bx1||x2==bx2{return y1==by2||y2==by1;}
    if x2==bx1||x1==bx2{return !(y2<by1||by2<y1);}
    y1!=by1&&y2!=by2&&(y2==by1||y1==by2)&&bx1<=x2&&x1<=bx2
}

///00413c00/00413da0. Tied scores preserve the original64-slot sector order.
pub fn target_sectors(sectors:&[Sector],groups:&[Group])->Vec<usize>{
    let mut candidates=sectors.iter().filter(|s|s.valid&&s.owner!=1&&!groups.iter().any(|g|
        controlled(g)&&matches!(g.task,0|1|2)&&g.assigned_sector==Some(s.id))).collect::<Vec<_>>();
    candidates.sort_by(|a,b|b.rank.cmp(&a.rank));candidates.into_iter().map(|s|s.id).collect()
}

fn eligible(g:&Group,s:&Sector,sectors:&[Sector],neighbor_only:bool)->bool{
    if !controlled(g)||g.task!=-1{return false;}
    if s.flags&0x60!=0{
        let Some(l)=g.leader.as_ref().filter(|l|l.team==1)else{return false;};
        if s.flags&0x20!=0&&l.runtime_class==1{return false;}
        if s.flags&0x40!=0&&(l.family==32&&l.passenger_class!=3||l.family==34
            ||l.runtime_class==16&&!l.laser_can_cross){return false;}
    }
    !neighbor_only||sector(sectors,g.current_sector).is_some_and(|current|adjacent(current,s))
}
fn distance(g:&Group,s:&Sector)->i32{
    g.leader.as_ref().filter(|l|l.team==1).map(|l|{
        let dx=i32::from(s.flag[0])*16-i32::from(l.position[0]);
        let dy=i32::from(s.flag[1])*16-i32::from(l.position[1]);
        dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))
    }).unwrap_or(i32::MAX)
}
pub fn candidate_groups(groups:&[Group],target:&Sector,sectors:&[Sector],neighbor_only:bool)->Vec<usize>{
    let mut result=groups.iter().enumerate().filter(|(_,g)|eligible(g,target,sectors,neighbor_only)).map(|(i,_)|i).collect::<Vec<_>>();
    result.sort_by_key(|&i|distance(&groups[i],target));result
}
fn prefix(groups:&[Group],candidates:&[usize],required:i32)->Option<usize>{
    let mut total=0i32;
    for(n,&i)in candidates.iter().enumerate(){total=total.wrapping_add(groups[i].value);if total>=required>>1{return Some(n+1);}}
    None
}
fn required(target:&Sector)->i32{
    if target.human_strength==0{1}else{target.priority.wrapping_add(i32::from(target.human_strength)*2)}
}
fn fort_support(groups:&mut[Group],target:&Sector,sectors:&[Sector],orders:&mut Vec<Order>){
    if target.flags&2==0{return;}
    // The native outer loop is sector-slot order, followed by group-slot order.
    for neighbor in sectors.iter().filter(|s|adjacent(target,s)){
        for g in groups.iter_mut().filter(|g|controlled(g)&&g.current_sector==Some(neighbor.id)){
            if g.task!=3||g.assigned_sector!=Some(target.id){g.task=3;g.via_sector=Some(target.id);g.assigned_sector=Some(target.id);orders.push(Order{group:g.id,sector:target.id,metadata:0});}
        }
    }
}

///004142d0. Allocate the closest prefix supplying at least half the required
/// force. If insufficient, one defending group may be taken from a sector of
/// equal or lower priority. Ties select the last defense slot, as native<= does.
pub fn allocate_target(groups:&mut[Group],target:&Sector,sectors:&[Sector],neighbor_only:bool)->Option<Vec<Order>>{
    let mut candidates=candidate_groups(groups,target,sectors,neighbor_only);
    let need=required(target);
    let count=prefix(groups,&candidates,need).or_else(||{
        let mut defender=None;let mut lowest=i32::MAX;
        for(i,g)in groups.iter().enumerate().filter(|(_,g)|controlled(g)&&g.task==4){
            if let Some(from)=sector(sectors,g.via_sector){if from.priority<=lowest{lowest=from.priority;defender=Some(i);}}
        }
        if lowest>target.priority{return None;}
        candidates.push(defender?);prefix(groups,&candidates,need)
    })?;
    let mut orders=Vec::new();
    for &i in candidates.iter().take(count){
        let g=&mut groups[i];if !g.leader.as_ref().is_some_and(|l|l.accepts_orders){continue;}
        g.task=0;g.via_sector=Some(target.id);g.assigned_sector=Some(target.id);
        orders.push(Order{group:g.id,sector:target.id,metadata:0});
    }
    fort_support(groups,target,sectors,&mut orders);Some(orders)
}

///004143a0 consumes the candidate-sector list made before the first allocation.
pub fn allocate_attacks(groups:&mut[Group],sectors:&[Sector],neighbor_only:bool)->Vec<Order>{
    let mut orders=Vec::new();
    for id in target_sectors(sectors,groups){let target=sector(sectors,Some(id)).unwrap();
        // Native143a0 does not attempt the defensive fallback without any free
        // candidates:00413e10 returns1 and skips004142d0 entirely.
        if candidate_groups(groups,target,sectors,neighbor_only).is_empty(){continue;}
        if let Some(next)=allocate_target(groups,target,sectors,neighbor_only){orders.extend(next);}
    }
    orders
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn rectangle_query_matches256_complete_native_cached_bucket_executions(){let mut hash=2166136261u32;for s in 0..256{let mut buckets:[Vec<usize>;256]=std::array::from_fn(|_|Vec::new());let(x,y)=(512+(s%16)as i16*3,512+(s/16)as i16*3);let mut positions=Vec::new();for i in 0..240{let px=x-80+(i*37%256)as i16;let py=y-80+(i*29%256)as i16;let b=(py>>3&!15)+(px>>7);if (0..256).contains(&b)&&buckets[b as usize].len()<32{buckets[b as usize].push(i);}positions.push(([px+(((s+i)%5)as i16-2)*40,py+(((s*3+i)%5)as i16-2)*40],if i%3==0{1}else{4}));}let ids=rectangle_units(&buckets,[x,y,x+128,y+128],|i|positions.get(i).copied());for n in std::iter::once(s as u32).chain(std::iter::once(ids.len()as u32)).chain(ids.into_iter().map(|i|i as u32)){hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,1733806252,"complete56640/565a0, cached versus live positions, no source stubs");}
    fn sector_fixture(id:usize,priority:i32)->Sector{Sector{id,native_slot:id,valid:true,rect:[id as i16*8,0,id as i16*8+8,8],owner:0,flag:[4+id as u8*8,4],flags:0,human_strength:0,priority,rank:priority.max(0)}}
    fn group_fixture(id:usize,value:i32)->Group{Group{id,active:true,flags:24,task:-1,current_sector:Some(1),via_sector:None,assigned_sector:None,value,leader:Some(Leader{native_slot:id,team:1,runtime_class:0,family:4,passenger_class:0,position:[32+id as i16*16,64],accepts_orders:true,laser_can_cross:false})}}
    #[test]fn all_256_executed_native_allocation_scenarios_match_orders_and_group_states(){
        let mut order_hash=2166136261u32;let mut group_hash=2166136261u32;
        for scenario in 0..256{
            let mut sectors=(0..4).map(|id|{let mut s=sector_fixture(id,(scenario*37+id as i32*71)%400);s.owner=if id==1{1}else{0};s.human_strength=if scenario&4!=0{20+id as i16*10}else{0};s.flags=(if id==2{(scenario as u8&3)*32}else{0})|if id==0&&scenario&128!=0{2}else{0};s}).collect::<Vec<_>>();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,5+(scenario*11+id as i32*23)%150);let l=g.leader.as_mut().unwrap();l.runtime_class=if id<3{1}else if id==3{16}else{0};l.family=if id==0{32}else if id==1{34}else{4};l.passenger_class=if scenario&8!=0{3}else{0};l.laser_can_cross=scenario&8!=0;l.accepts_orders=!(scenario&16!=0&&id==2);g}).collect::<Vec<_>>();
            if scenario&32!=0{groups[7].task=4;groups[7].via_sector=Some(1);sectors[1].priority=1;sectors[1].rank=1;}
            let orders=allocate_attacks(&mut groups,&sectors,scenario&64!=0);
            for(n,o)in orders.iter().enumerate(){for v in [scenario,n as i32,o.group as i32,o.sector as i32]{order_hash=(order_hash^v as u32).wrapping_mul(16777619);}}
            for g in &groups{for v in [scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{group_hash=(group_hash^v as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(order_hash,1057381467,"ZNativeTaskOracle order trace");assert_eq!(group_hash,3413372530,"ZNativeTaskOracle group trace");
    }
    #[test]fn reserve_flags_match_512_native_group_results_and_104_release_notifications(){
        const BUDGETS:[i32;64]=[582,371,192,59,448,247,146,0,484,433,232,53,280,319,119,0,616,514,273,63,482,271,92,0,348,297,246,73,484,343,132,0,480,419,219,49,516,414,173,0,382,331,192,59,348,247,146,0,684,443,232,53,380,319,119,0,416,514,273,63,382,271,92,0];
        let mut flags_hash=2166136261u32;let mut release_hash=2166136261u32;
        for scenario in 0..64{
            let values=(0..8).map(|id|if id==3{-1}else{1+(scenario as i32*17+id as i32*23)%170}).collect::<Vec<_>>();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,values[id]);g.flags=(if id%3!=0{8}else{0})|if(scenario+id)%2!=0{16}else{0};g}).collect::<Vec<_>>();
            let released=select_budget_groups(&mut groups,&values,BUDGETS[scenario]as f32);
            for id in released{for n in [scenario as i32,id as i32]{release_hash=(release_hash^n as u32).wrapping_mul(16777619);}}
            for(id,g)in groups.iter().enumerate(){for n in [scenario as i32,id as i32,values[id],BUDGETS[scenario],i32::from(g.flags)]{flags_hash=(flags_hash^n as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(release_hash,1813327405);assert_eq!(flags_hash,2433125447);
    }
    #[test]fn all_48_native_defense_scenarios_match_66_orders_and_240_group_states(){
        let mut order_hash=2166136261u32;let mut state_hash=2166136261u32;
        for phase in 0..3{for scenario in 0..16{
            let mut sectors=vec![sector_fixture(0,300+scenario*180),sector_fixture(1,1500)];
            for s in &mut sectors{s.owner=if phase==0&&scenario&8!=0&&s.id==0{0}else{1};s.flags=if s.id==0&&scenario&8!=0{2}else{0};}
            let reserve=(scenario%4)as f32*100.;let attack=300f32;
            let requirements=sectors.iter().map(|s|(s.id,((100f32-reserve*100f32/(attack+reserve))*s.priority as f32/100f32)as i32)).collect::<Vec<_>>();
            let mut groups=(0..5).map(|id|{let mut g=group_fixture(id,if id==4{50}else{200+id as i32*100});g.current_sector=Some(if phase==2{0}else{1});g.task=if phase==0{4}else if phase==1&&id==3&&scenario&4!=0{4}else{-1};g.via_sector=if phase==0{Some(id%2)}else if phase==1&&id==3&&scenario&4!=0{Some(1)}else{None};g.active=!(phase==2&&id==4);g.leader.as_mut().unwrap().runtime_class=if id==4{2}else{0};g}).collect::<Vec<_>>();
            let orders=match phase{0=>{cancel_unneeded_defenses(&mut groups,&sectors,&requirements);Vec::new()},1=>assign_defense(&mut groups,&sectors,&requirements,false),_=>defend_captured_sector(&mut groups,&sectors[0],requirements[0].1)};
            for(n,o)in orders.iter().enumerate(){for v in [phase,scenario,n as i32,o.group as i32,o.sector as i32]{order_hash=(order_hash^v as u32).wrapping_mul(16777619);}}
            for g in &groups{for v in [phase,scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
        }}
        assert_eq!(order_hash,1150849568);assert_eq!(state_hash,3519029853);
    }
    #[test]fn reinforcement_matches_83_native_orders_and_256_group_results(){
        let mut order_hash=2166136261u32;let mut state_hash=2166136261u32;
        for scenario in 0..32{
            let mut sectors=vec![sector_fixture(0,300+scenario*170),sector_fixture(1,100)];sectors[1].owner=1;
            for s in &mut sectors{s.human_strength=if scenario&4!=0{20}else{0};}
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,30+(scenario*31+id as i32*67)%400);let assigned=id<if scenario&8!=0{7}else{3};g.task=if assigned{if scenario&2!=0&&id==1{3}else{0}}else{-1};g.via_sector=assigned.then_some(0);g.assigned_sector=assigned.then_some(0);g.leader.as_mut().unwrap().accepts_orders=!(scenario&16!=0&&id==6);g}).collect::<Vec<_>>();
            let changes=reinforce_attacks(&mut groups,&sectors,false);assert!(changes.direct.is_empty());
            for(n,o)in changes.sectors.iter().enumerate(){for v in [scenario,n as i32,o.group as i32,o.sector as i32]{order_hash=(order_hash^v as u32).wrapping_mul(16777619);}}
            for g in &groups{for v in [scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(order_hash,761832293);assert_eq!(state_hash,3087858288);
    }
    #[test]fn assault_staging_matches_128_orders_116_capture_goals_and_512_native_group_states(){
        let mut order_hash=2166136261u32;let mut direct_hash=2166136261u32;let mut state_hash=2166136261u32;let mut sequence_hash=2166136261u32;
        for scenario in 0..64{
            let mut sectors=vec![sector_fixture(0,300+scenario*70),sector_fixture(1,100),sector_fixture(2,100)];sectors[1].owner=1;sectors[0].flags=if scenario&32!=0{2}else{0};
            for s in &mut sectors{s.human_strength=if scenario&4!=0{20}else{0};}
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,30+(scenario*31+id as i32*67)%400);g.current_sector=Some(if id==2{0}else{1});g.task=((scenario+id as i32)%4)as i8;g.via_sector=Some(0);g.assigned_sector=Some(if id==7{2}else{0});g}).collect::<Vec<_>>();
            let controls=groups.iter().map(|g|AssaultControl{group:g.id,can_capture:!(scenario&16!=0&&g.id==6),can_mobilize:!(scenario&16!=0&&g.id==6)}).collect::<Vec<_>>();
            let changes=advance_assaults(&mut groups,&sectors,&controls,[1000,1000]);
            for movement in &changes.sequence{let fields=match movement{Movement::Sector(o)=>[scenario,0,o.group as i32,o.sector as i32,0],Movement::Direct(o)=>[scenario,1,o.group as i32,i32::from(o.position[0]),i32::from(o.position[1])]};for v in fields{sequence_hash=(sequence_hash^v as u32).wrapping_mul(16777619);}}
            for(n,o)in changes.sectors.iter().enumerate(){for v in [scenario,n as i32,o.group as i32,o.sector as i32]{order_hash=(order_hash^v as u32).wrapping_mul(16777619);}}
            for(n,o)in changes.direct.iter().enumerate(){for v in [scenario,n as i32,o.group as i32,i32::from(o.position[0]),i32::from(o.position[1])]{direct_hash=(direct_hash^v as u32).wrapping_mul(16777619);}}
            for g in &groups{for v in [scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(order_hash,1284740341);assert_eq!(direct_hash,2515456245);assert_eq!(state_hash,3927578437);
        assert_eq!(sequence_hash,3017353993);
    }
    #[test]fn personal_capture_matches_218_native_member_routes_and_512_group_states(){
        let mut route_hash=2166136261u32;let mut state_hash=2166136261u32;
        for scenario in 0..64{
            let mut target=sector_fixture(0,100);target.flags=if scenario&32!=0{2}else{(scenario as u8&3)*32};
            let sectors=vec![target];let mut runtime=Vec::new();
            let mut groups=(0..8).map(|id|{
                let mut g=group_fixture(id,200);g.current_sector=Some(0);g.via_sector=Some(0);g.assigned_sector=Some(0);g.task=if scenario&2!=0&&id==3{3}else if scenario&16!=0&&id==7{1}else{0};
                let l=g.leader.as_mut().unwrap();l.native_slot=id*3;l.position=[64,64];l.runtime_class=if id<2{1}else{0};l.family=if id==0{32}else if id==1{34}else{4};l.passenger_class=if scenario&8!=0{3}else{0};l.laser_can_cross=scenario&8!=0;
                let count=1+(scenario as usize+id)%3;
                let roster=(0..21).map(|member|(member<count).then_some(CaptureMember{native_slot:id*3+member,role:if member==0{-1}else{member as i8},personal_capture:scenario&8!=0&&id==3&&member==count-1,special_path:scenario&1!=0&&id==2&&member==count-1,can_start_path:true,direct_blocked:scenario&32!=0&&id==6})).collect();
                runtime.push(CaptureRuntime{group:id,roster,leader_overlay:scenario&4!=0&&id==4,entering_cpu_fort:scenario&16!=0});g
            }).collect::<Vec<_>>();
            let routes=request_personal_captures(&mut groups,&sectors,&runtime,[1000,1000]);
            for(n,o)in routes.iter().enumerate(){for v in [scenario,n as i32,o.group as i32,o.member as i32,i32::from(o.position[0]),i32::from(o.position[1])]{route_hash=(route_hash^v as u32).wrapping_mul(16777619);}}
            for g in &groups{for v in [scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(route_hash,1253972812);assert_eq!(state_hash,1464484881);
    }
    #[test]fn budget_release_matches_2048_native_group_results_and_132_fallbacks(){
        let mut state_hash=2166136261u32;let mut order_hash=2166136261u32;
        for scenario in 0..32{
            let sectors=(0..4).map(|id|{let mut s=sector_fixture(id,0);s.flags=if id==0{1}else if id==1{2}else{0};s}).collect::<Vec<_>>();
            let tasks=[-1,0,1,2,3,4,5,6];
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,0);g.active=scenario&16==0||id%2==0;g.task=tasks[(id+scenario as usize)&7];g.flags=8*(1+id as u8%3);g.current_sector=Some(if scenario&1!=0&&id%2==0{1}else{2});g.assigned_sector=if scenario&4!=0&&id%3==0{None}else{Some(if(scenario as usize+id)&2!=0{2}else{3})};if scenario&8!=0&&id%3==0{g.leader=None;}else{g.leader.as_mut().unwrap().team=if scenario&2!=0&&id%3==0{0}else{1};}g}).collect::<Vec<_>>();
            for released in 0..8{
                let changes=release_budget_groups(&mut groups,&[released],&sectors,Some(0));assert!(changes.direct.is_empty());
                for o in changes.sectors{for v in [scenario,released as i32,o.group as i32,i32::from(o.metadata),o.sector as i32]{order_hash=(order_hash^v as u32).wrapping_mul(16777619);}}
                for g in &groups{for v in [scenario,released as i32,g.id as i32,i32::from(g.flags),i32::from(g.task)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
            }
        }
        assert_eq!(state_hash,1463706153);assert_eq!(order_hash,2359590789);
    }
    #[test]fn tactical_hardware_goal_matches_7040_native_counters_and_fifteen_local_goals(){
        let mut state_hash=2166136261u32;let mut goal_hash=2166136261u32;
        for scenario in 0..8{
            let mut target=sector_fixture(0,100);target.owner=if scenario&1!=0{1}else{0};target.flags=if scenario&2!=0{2}else{0};let sectors=vec![target];
            let families=[6,4,5,32,34,36,25,6];let classes=[1,0,2,1,1,1,1,1];let xs=[64,64,64,192,191,-64,64,64];
            let groups=(0..8).map(|id|{let mut g=group_fixture(id,100);g.current_sector=Some(0);g.active=!(id==7&&scenario&4!=0);let l=g.leader.as_mut().unwrap();l.family=families[id];l.runtime_class=classes[id];l.position=[xs[id],64];l.team=if id==6{0}else{1};g}).collect::<Vec<_>>();
            let mut counters=[0,1,2,3,4,5,6,7];
            for tick in 1..=110{
                for goal in tactical_vehicle_goals(&groups,&sectors,&mut counters){for v in [scenario,tick,goal.group as i32,i32::from(goal.position[0]),i32::from(goal.position[1])]{goal_hash=(goal_hash^v as u32).wrapping_mul(16777619);}}
                for(id,&counter)in counters.iter().enumerate(){for v in [scenario,tick,id as i32,i32::from(counter)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
            }
        }
        assert_eq!(goal_hash,2146717015);assert_eq!(state_hash,2740450471);
    }
    #[test]fn tactical_fort_wrapper_and_512_entry_predicates_match_native_execution(){
        let mut wrapper_hash=2166136261u32;let mut predicate_hash=2166136261u32;
        for scenario in 0..32{
            let sectors=(0..4).map(|id|{let mut s=sector_fixture(id,100);s.flags=id as u8;s}).collect::<Vec<_>>();
            let mut runtime=Vec::new();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,100);g.current_sector=if scenario&16!=0&&id==7{None}else{Some((id+scenario as usize)%4)};g.flags=((id+scenario as usize)%4*8)as u8;g.active=scenario&8==0||id%2==0;g.task=0;
                runtime.push(CaptureRuntime{group:id,roster:vec![Some(CaptureMember{native_slot:id,role:-1,personal_capture:false,special_path:false,can_start_path:true,direct_blocked:false})],leader_overlay:false,entering_cpu_fort:(scenario as usize+id)&4!=0});g}).collect::<Vec<_>>();
            let routes=tactical_fort_captures(&mut groups,&sectors,&runtime,[1000,1000]);
            for pair in routes.chunks_exact(2){assert_eq!(pair[0].group,pair[1].group);for v in [scenario,pair[0].group as i32]{wrapper_hash=(wrapper_hash^v as u32).wrapping_mul(16777619);}}
        }
        for scenario in 0..64{
            let fort:[i16;2]=if scenario&32!=0{[32760,-32760]}else{[1000,1000]};
            for id in 0..8{
                let goal=[fort[0].wrapping_add(80).wrapping_add(i16::from(id==1)),fort[1].wrapping_add(72).wrapping_add(i16::from(id==2))];
                let path=if scenario&2!=0&&id==3||scenario&4!=0&&id==4||scenario&8!=0&&id==5||scenario&16!=0&&id==6{None}else{Some([fort[0].wrapping_add(80).wrapping_add(i16::from(id==0)),fort[1].wrapping_add(if id%3==0{72}else if id%3==1{-48}else{73})])};
                let result=entering_cpu_fort(if scenario&1!=0{1}else{0},Some(goal),path,fort);
                for v in [scenario,id,i32::from(result),i32::from(result)]{predicate_hash=(predicate_hash^v as u32).wrapping_mul(16777619);}
            }
        }
        assert_eq!(wrapper_hash,940444892);assert_eq!(predicate_hash,2932714469);
    }
    #[test]fn arrival_transitions_match_18_capture_commands_70_completions_and_512_native_states(){
        let mut capture_hash=2166136261u32;let mut completed_hash=2166136261u32;let mut state_hash=2166136261u32;
        let tasks=[1,0,2,3,6,4,5,-1];
        for scenario in 0..64{
            let sectors=(0..4).map(|id|{let mut s=sector_fixture(id,100);s.owner=if scenario&1!=0&&id%2==0{1}else{0};s}).collect::<Vec<_>>();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,100);let target=id%2;g.current_sector=Some(if scenario&2!=0&&id==2{3}else{target});g.via_sector=Some(target);g.assigned_sector=if scenario&4!=0&&id==4{None}else{Some(target)};g.task=tasks[(id+scenario as usize/8)%8];g.active=scenario&8==0||id%2==0;g.leader.as_mut().unwrap().team=if scenario&32!=0&&id==2{0}else{1};g}).collect::<Vec<_>>();
            let combat=(0..8).map(|id|(id,if scenario&16!=0&&id%3==0{1}else{0})).collect::<Vec<_>>();
            for index in 0..8{if let Some(arrival)=capture_arrival_for_group(&mut groups,index,&sectors,&combat){match arrival{
                Arrival::Capture{group,member,sector}=>for v in [scenario,group as i32,member as i32,sector as i32]{capture_hash=(capture_hash^v as u32).wrapping_mul(16777619);},
                Arrival::Captured{group,sector}=>for v in [scenario,group as i32,sector as i32]{completed_hash=(completed_hash^v as u32).wrapping_mul(16777619);},
            }}}
            for g in &groups{for v in [scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1)]{state_hash=(state_hash^v as u32).wrapping_mul(16777619);}}
        }
        assert_eq!(capture_hash,1396119489);assert_eq!(completed_hash,3078227661);assert_eq!(state_hash,1406455933);
    }
    #[test]fn five_post_capture_phases_match_320_executed_native_worlds_and_rng_consumption(){
        fn add(h:&mut u32,values:impl IntoIterator<Item=i32>){for n in values{*h=(*h^n as u32).wrapping_mul(16777619);}}
        let mut states=[2166136261u32;5];let mut rng_hash=[2166136261u32;5];let mut movements=[2166136261u32;5];let mut rejoin_hash=2166136261u32;let mut unit_hash=2166136261u32;
        let types=[5,8,25,26,37,10,21,35];
        for phase in 0..5{for scenario in 0..64{
            let mut sectors=(0..if phase==4{7}else{4}).map(|id|sector_fixture(id,100)).collect::<Vec<_>>();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,100);g.current_sector=Some(0);g.via_sector=Some(0);g.assigned_sector=Some(0);g}).collect::<Vec<_>>();
            let mut runtime=(0..8).map(|id|GroupRuntime{group:id,member_count:1,combat_ttl:0,native_order:0,pending:false,leader_role:-1,leader_flags31:5}).collect::<Vec<_>>();
            let mut counters=[0i8;8];let mut rng=123u32+scenario as u32;let mut calls=0;
            let mut random=||{rng=rng.wrapping_mul(1664525).wrapping_add(1013904223);calls+=1;rng};
            if phase==0{
                let objects=(0..8).map(|id|WorldObject{id,position:[if scenario%4==0&&id%3==0{0}else{(id%4)as i16*128+32},64],definition:if scenario&8!=0&&id%2==0{None}else{Some(types[(scenario as usize+id)%8])},flags13:0,flags26:0,sector:Some(id%4)}).collect::<Vec<_>>();
                for g in &mut groups{g.via_sector=Some(g.id%4);g.assigned_sector=Some(3);g.task=if scenario&16!=0&&g.id%2==1{4}else{5};g.active=scenario&32==0||g.id%3!=0;}
                retire_missing_hardware(&mut groups,&objects);
            }else if phase==1{
                for s in &mut sectors{s.owner=if(scenario as usize+s.id)%3==0{1}else{0};}
                let mut units=(0..8).map(|id|RuntimeUnit{native_slot:id,position:[(id%4)as i16*128+32,64],flags31:((scenario as usize+id)%4)as u8|if(scenario as usize+id)%3!=0{4}else{0},flags8a:((scenario as usize*7+id*3)&31)as u8,sector:Some(id%4)}).collect::<Vec<_>>();
                for id in finish_personal_routes(&mut units,&sectors){add(&mut rejoin_hash,[phase as i32,scenario,id as i32]);}
                for u in units{add(&mut unit_hash,[phase as i32,scenario,u.native_slot as i32,i32::from(u.flags8a)]);}
            }else if phase==2{
                for s in &mut sectors{s.owner=if scenario&2!=0&&s.id==2{1}else{0};s.flags=if scenario&4!=0&&s.id==3{2}else{0};}
                let buildings=(0..4).map(|id|BuildingRecord{id,position:[id as i16*128+16,32],kind:if scenario&8!=0&&id==1{-1}else{id as i8},owner:0,sector:Some(id),hit_points:0,attack_offset:[-7-id as i8,23+id as i8]}).collect::<Vec<_>>();
                let classes=[0,16,15,3,1,1,0,1];let families=[4,4,4,4,6,34,4,24];
                for g in &mut groups{g.current_sector=Some(g.id%4);g.assigned_sector=Some(g.id%4);g.task=if g.id%2==0{1}else{3};let l=g.leader.as_mut().unwrap();l.runtime_class=classes[g.id];l.family=families[g.id];l.laser_can_cross=scenario&1!=0;}
                let query=(0..8).map(|id|(id,vec![RuntimeUnit{native_slot:16,position:[(id%4)as i16*128+if scenario&16!=0{129}else{64},64],flags31:if scenario&32!=0{1}else{0},flags8a:0,sector:None}])).collect::<Vec<_>>();
                for(n,o)in attack_target_buildings(&mut groups,&sectors,&buildings,&query).iter().enumerate(){add(&mut movements[phase],[phase as i32,scenario,n as i32,o.member as i32,i32::from(o.position[0]),i32::from(o.position[1])]);}
            }else if phase==3{
                for g in &mut groups{let id=g.id;g.via_sector=Some(id%4);g.assigned_sector=Some(3);g.task=if id>=5{5}else{-1};let l=g.leader.as_mut().unwrap();l.runtime_class=if id==4{1}else if id%4==0{0}else if id%4==1{16}else if id%4==2{3}else{15};l.accepts_orders=!(scenario&2!=0&&id==2);runtime[id].leader_role=if scenario&4!=0&&id==1{0}else{-1};}
                let objects=(0..8).map(|id|WorldObject{id,position:[(id%4)as i16*128+32,64],definition:Some(types[(scenario as usize+id)%8]),flags13:if scenario&8!=0&&id%3==0{1}else{0},flags26:if scenario&16!=0&&id%3==1{1}else{0},sector:Some(id%4)}).collect::<Vec<_>>();
                let flags=(0..4).map(|id|(id,if scenario&1!=0&&id%2!=0{1}else{0})).collect::<Vec<_>>();
                for(n,o)in support_hardware_orders(&mut groups,&sectors,&runtime,&objects,&flags,&mut random).iter().enumerate(){add(&mut movements[phase],[phase as i32,scenario,n as i32,o.group as i32,i32::from(o.position[0]),i32::from(o.position[1])]);}
            }else{
                for s in &mut sectors{s.owner=if matches!(s.id,4|6){0}else{1};s.flags=if s.id==4&&scenario&8!=0{32}else if s.id==6&&scenario&16!=0{64}else{0};}
                let classes=[0,1,16,15,3,0,1,2];
                for g in &mut groups{let id=g.id;g.current_sector=Some(id%2);g.via_sector=Some(3);g.assigned_sector=Some(2);counters[id]=(id%3)as i8;let l=g.leader.as_mut().unwrap();l.runtime_class=classes[id];l.laser_can_cross=scenario&4!=0;if scenario&32!=0&&id==1{g.leader=None;}runtime[id].native_order=if scenario&1!=0&&id==3{1}else{0};runtime[id].pending=scenario&2!=0&&id==2;}
                for(n,o)in assign_idle_frontiers(&mut groups,&sectors,&runtime,&mut counters,&mut random).iter().enumerate(){add(&mut movements[phase],[phase as i32,scenario,n as i32,o.group as i32,i32::from(o.position[0]),i32::from(o.position[1])]);}
            }
            add(&mut rng_hash[phase],[phase as i32,scenario,calls]);
            for g in &groups{add(&mut states[phase],[phase as i32,scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1),i32::from(g.active),i32::from(counters[g.id])]);}
        }}
        assert_eq!(states,[2430221997,983961925,1658572917,1460337025,512945725]);
        assert_eq!(rng_hash,[2878163269,1303697221,2020748357,1408294581,3952221121]);
        assert_eq!(movements[2],589159801);assert_eq!(movements[3],3173532441);assert_eq!(movements[4],201660421);
        assert_eq!(rejoin_hash,108988365);assert_eq!(unit_hash,2239513421);
    }
    #[test]fn hardware_capture_fallback_matches_64_native_worlds_16_orders_and_512_states(){
        fn add(h:&mut u32,values:impl IntoIterator<Item=i32>){for n in values{*h=(*h^n as u32).wrapping_mul(16777619);}}
        let mut state_hash=2166136261u32;let mut order_hash=2166136261u32;let mut split_hash=2166136261u32;
        for scenario in 0..64{
            let sectors=(0..4).map(|id|{let mut s=sector_fixture(id,100);s.owner=1;s.flags=if scenario&1==0{16}else{0};s}).collect::<Vec<_>>();
            let buildings=vec![BuildingRecord{id:0,position:[16,32],kind:0,owner:0,sector:Some(0),hit_points:if scenario&2==0{100}else{0},attack_offset:[0,0]}];
            let defs=[5,26,8];let objects=(0..3).map(|id|WorldObject{id,position:[if scenario&8!=0&&id==0{0}else{32+id as i16*16},64],definition:Some(defs[id]),flags13:0,flags26:if id==0&&scenario&4!=0{1}else{0},sector:Some(0)}).collect::<Vec<_>>();
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,100);g.current_sector=Some(0);g.via_sector=Some(0);g.assigned_sector=Some(3);g.flags=if id==6{28}else{24};g.task=if matches!(id,1|2){5}else if id==3{4}else if matches!(id,4|5){3}else{-1};g.active=!(id==3&&scenario%5==0);let l=g.leader.as_mut().unwrap();l.runtime_class=if id==0&&scenario&16!=0{1}else if id==1{if scenario&32!=0{1}else{0}}else if matches!(id,2|7){2}else if id==3{3}else{0};l.laser_can_cross=scenario%3!=0;l.position=[16+id as i16*8,64];g}).collect::<Vec<_>>();
            let runtime=(0..8).map(|id|GroupRuntime{group:id,member_count:if id==4{2}else{1},combat_ttl:0,native_order:0,pending:false,leader_role:-1,leader_flags31:5}).collect::<Vec<_>>();
            let captures=(0..8).map(|id|CaptureRuntime{group:id,roster:vec![Some(CaptureMember{native_slot:id,role:-1,personal_capture:false,special_path:false,can_start_path:true,direct_blocked:false}),(id==4).then_some(CaptureMember{native_slot:8,role:1,personal_capture:false,special_path:false,can_start_path:true,direct_blocked:false})],leader_overlay:false,entering_cpu_fort:false}).collect::<Vec<_>>();
            let mut split=0;let mut n=0;
            for index in 0..8{if !controlled(&groups[index]){continue;}
                if let Some(order)=hardware_capture_for_group(&mut groups,index,&sectors,&runtime,&captures,&buildings,&objects){
                    split+=1;add(&mut order_hash,[scenario,n,order.fallback_member as i32,i32::from(order.position[0]),i32::from(order.position[1])]);n+=1;
                    // Native fixture's4c0b0 returns0: use its exact leader fallback.
                    assert!(commit_hardware_capture(&mut groups[index],Some(0),-1,5));
                }
            }
            add(&mut split_hash,[scenario,split]);
            for g in &groups{add(&mut state_hash,[scenario,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1),i32::from(g.flags),i32::from(g.active)]);}
        }
        assert_eq!(state_hash,2296789731);assert_eq!(order_hash,1249712528);assert_eq!(split_hash,4273370949);
    }
    #[test]fn initial_flags_capture_release_and_4096_retry_states_match_native_lifecycle(){
        fn add(h:&mut u32,values:impl IntoIterator<Item=i32>){for n in values{*h=(*h^n as u32).wrapping_mul(16777619);}}
        let mut initial_hash=2166136261u32;let mut object_hash=2166136261u32;let mut retry_hash=2166136261u32;
        for scenario in 0..64{
            let mut sectors=(0..4).map(|id|{let mut s=sector_fixture(id,100);s.flags=((scenario as usize*13+id*37)&255)as u8;s}).collect::<Vec<_>>();
            let buildings=vec![BuildingRecord{id:0,position:[160,64],kind:0,owner:0,sector:Some(1),hit_points:if scenario&1!=0{0}else{100},attack_offset:[0,0]}];
            let defs=[5,26,25,37];let mut objects=(0..4).map(|id|WorldObject{id,position:[if scenario&2!=0&&id==0{0}else{id as i16*128+32},64],definition:if scenario&8!=0&&id==2{None}else{Some(defs[id])},flags13:0,flags26:7+id as u8,sector:Some(id)}).collect::<Vec<_>>();
            initialize_sector_flags(&mut sectors,[Some(0),Some(1),(scenario&16!=0).then_some(2),(scenario&32!=0).then_some(3)],&buildings,&objects);
            for s in sectors{add(&mut initial_hash,[scenario,s.id as i32,i32::from(s.flags),255]);}
            clear_hardware_capture_restrictions(&mut objects,scenario as usize%4,if scenario&4!=0{18}else{17});
            for o in objects{add(&mut object_hash,[scenario,o.id as i32,i32::from(o.flags26)]);}
        }
        let tasks=[-1,0,1,2,3,4,5,6];
        for scenario in 0..64{
            let mut groups=(0..8).map(|id|{let mut g=group_fixture(id,100);g.via_sector=Some(1);g.assigned_sector=Some(2);g.flags=if scenario&2!=0&&id==1{8}else if scenario&4!=0&&id==2{16}else{24};g.task=tasks[(scenario as usize+id)%8];g.active=scenario&1==0||id!=6;g}).collect::<Vec<_>>();
            let can_retry=(0..8).map(|id|id!=5&&!(scenario&32!=0&&id==6)&&!(scenario&8!=0&&id==3)&&(id!=4||scenario&16!=0)).collect::<Vec<_>>();
            let mut counters=[0,1,2,3,127,-1,-128,0];
            for tick in 1..=8{
                retry_stalled_tasks(&mut groups,&mut counters,&can_retry);
                for g in &groups{add(&mut retry_hash,[scenario,tick,g.id as i32,i32::from(g.task),g.via_sector.map(|n|n as i32).unwrap_or(-1),g.assigned_sector.map(|n|n as i32).unwrap_or(-1),i32::from(counters[g.id])]);}
            }
        }
        assert_eq!(initial_hash,3558998917);assert_eq!(object_hash,109755141);assert_eq!(retry_hash,392089797);
    }
    #[test]fn candidates_use_scripted_tasks_stable_scores_and_sector_capabilities(){
        let mut sectors=vec![sector_fixture(0,100),sector_fixture(1,100),sector_fixture(2,200)];sectors[1].owner=1;
        let mut groups=vec![group_fixture(0,20),group_fixture(1,30)];
        assert_eq!(target_sectors(&sectors,&groups),vec![2,0]);
        let orders=initial_tasks(&mut groups,&[1,0],&sectors);assert_eq!(orders,vec![Order{group:0,sector:0,metadata:0}]);
        assert_eq!(target_sectors(&sectors,&groups),vec![2]);
        sectors[2].flags=0x20;groups[1].leader.as_mut().unwrap().runtime_class=1;
        assert!(candidate_groups(&groups,&sectors[2],&sectors,false).is_empty());
        groups[1].leader.as_mut().unwrap().runtime_class=0;assert_eq!(candidate_groups(&groups,&sectors[2],&sectors,false),vec![1]);
    }
    #[test]fn allocation_uses_half_force_threshold_and_keeps_blocked_group_out_of_orders(){
        let mut sectors=vec![sector_fixture(0,100),sector_fixture(1,10)];sectors[0].human_strength=20;sectors[1].owner=1;
        let mut groups=vec![group_fixture(0,20),group_fixture(1,30),group_fixture(2,40)];
        groups[1].leader.as_mut().unwrap().accepts_orders=false;
        let orders=allocate_target(&mut groups,&sectors[0],&sectors,false).unwrap();
        assert_eq!(orders.iter().map(|o|o.group).collect::<Vec<_>>(),vec![2]);
        assert_eq!(groups[0].task,-1);assert_eq!(groups[1].task,-1);
    }
    #[test]fn fort_support_retasks_neighbor_groups_and_defense_ties_use_later_slot(){
        let mut sectors=vec![sector_fixture(0,300),sector_fixture(1,10),sector_fixture(2,10)];sectors[0].human_strength=100;sectors[1].owner=1;sectors[2].owner=1;
        let mut groups=vec![group_fixture(0,20),group_fixture(1,400),group_fixture(2,400)];
        for i in 1..3{groups[i].task=4;groups[i].via_sector=Some(i);}
        let orders=allocate_target(&mut groups,&sectors[0],&sectors,false).unwrap();assert_eq!(orders.iter().map(|o|o.group).collect::<Vec<_>>(),vec![0,2]);
        assert_eq!(groups[1].task,4);
        sectors[0].flags=2;groups[0].task=-1;groups[0].value=1000;groups[1].current_sector=Some(1);
        let orders=allocate_target(&mut groups,&sectors[0],&sectors,false).unwrap();assert!(orders.iter().any(|o|o.group==1));assert_eq!(groups[1].task,3);
    }
}
