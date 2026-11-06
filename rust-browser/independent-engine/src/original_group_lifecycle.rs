//! Native4c0b0 follower detachment and4e910 formation rejoin.
//! Roster bytes retain the original72-byte layout so allocation preserves stale
//! fields exactly; only writes proven in4b7b0/4b2f0 are performed here.

pub const GROUP_SLOTS:usize=240;
pub const MEMBERS:usize=21;
const RINFO_BASE:u32=0x619ae8;
const RINFO_BYTES:u32=82;
/// Source record token, used only as a label in the native roster bytes.
pub fn authored_rinfo(gid:usize)->Option<[u8;4]>{(gid<19).then(||(RINFO_BASE+RINFO_BYTES*gid as u32).to_le_bytes())}
#[derive(Clone,Copy,Debug)]
pub struct SlotUsage{pub count:u8,pub leader:u8,pub cooldown:u8}

///4b2f0 preferred scan followed by its unrestricted fallback. The original
/// single-player teams are0/1; higher team values index outside240 roster slots.
pub fn free_group_slot(slots:&[SlotUsage;GROUP_SLOTS],team:u8)->Option<usize>{
    let team=team&3;if team>1{return None;}
    let free=|s:&SlotUsage|s.count==0&&s.leader==255;
    let(start,end)=if team==0{(0,GROUP_SLOTS)}else{(120,GROUP_SLOTS)};
    (start..end).find(|&i|free(&slots[i])&&slots[i].cooldown==0)
        .or_else(||(0..GROUP_SLOTS).find(|&i|free(&slots[i])))
}

#[derive(Clone,Debug)]
pub struct GroupSlot{
    /// Native0060ec00 +slot*72.
    pub roster:[u8;72],
    /// Native005c47b8 +slot*24.
    pub strategy:[u8;24],
    /// Native00586880 +slot. Allocation does not clear this byte.
    pub cooldown:u8,
}
impl GroupSlot{
    pub fn empty()->Self{let mut result=Self{roster:[0;72],strategy:[0;24],cooldown:0};result.roster[19..40].fill(255);result.strategy[..4].fill(255);result}
    pub fn usage(&self)->SlotUsage{SlotUsage{count:self.roster[18],leader:self.roster[19],cooldown:self.cooldown}}
    pub fn member(&self,role:usize)->Option<u8>{self.roster.get(19+role).copied().filter(|&id|id!=255)}
    fn rinfo(&self,offset:usize)->Option<[u8;4]>{
        let token:[u8;4]=self.roster[offset..offset+4].try_into().ok()?;let relative=u32::from_le_bytes(token).checked_sub(RINFO_BASE)?;
        (relative<32*RINFO_BYTES&&relative%RINFO_BYTES==0).then_some(token)
    }
    pub fn active_rinfo(&self)->Option<[u8;4]>{self.rinfo(0)}
    pub fn cached_rinfo(&self)->Option<[u8;4]>{self.rinfo(4)}
    pub fn set_active_rinfo(&mut self,token:[u8;4]){self.roster[..4].copy_from_slice(&token);}
    pub fn set_cached_rinfo(&mut self,token:[u8;4]){self.roster[4..8].copy_from_slice(&token);}
    pub(crate) fn restore_cached_rinfo(&mut self){
        if self.roster[4..8]!=[0;4]{
            self.roster.copy_within(4..8,0);
            self.roster[4..8].fill(0);
        }
    }
    pub fn word(&self,offset:usize)->i16{i16::from_le_bytes([self.roster[offset],self.roster[offset+1]])}
    fn set_word(&mut self,offset:usize,value:i16){self.roster[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
}

///4b8b0 runs in the main simulation callback immediately before19fb0. Count
/// nonzero slots before decrementing; a byte1 contributes once and becomes0.
pub fn advance_cooldowns(groups:&mut[GroupSlot;GROUP_SLOTS])->usize{
    let mut occupied=0;for g in groups{if g.cooldown!=0{occupied+=1;g.cooldown-=1;}}occupied
}
///4b220 preserves the leader and stably fills follower holes. Native updates
///the existing signed role by the displacement, rather than recomputing it.
pub fn compact_followers(group:&mut GroupSlot,members:&mut[Member]){
    let mut next=1;
    for role in 1..MEMBERS{
        let Some(id)=group.member(role)else{continue;};
        if role!=next{
            group.roster[19+next]=id;group.roster[19+role]=255;
            if let Some(member)=members.iter_mut().find(|m|m.slot==id){member.role=member.role.wrapping_add((next as i8).wrapping_sub(role as i8));}
        }
        next+=1;
    }
}
///4b280 in5a240, before the240 native unit handlers. Bit40 requests follower
///compaction once combat TTL reaches zero. The busy count excludes bit40 groups.
pub fn advance_group_counters(groups:&mut[GroupSlot;GROUP_SLOTS],members:&mut[Member])->u16{
    let mut busy=0u16;
    for group in groups{
        for offset in [0x42,0x3d]{if group.roster[offset]!=0{group.roster[offset]-=1;}}
        let compact=group.roster[9]&0x40!=0;
        if group.roster[16]!=0{
            if !compact{busy=busy.wrapping_add(1);}
            group.roster[16]-=1;
            if group.roster[16]!=0{continue;}
        }else if !compact{group.roster[0x3d]=0;group.roster[0x40]=0;continue;}
        compact_followers(group,members);group.roster[9]&=!0x40;
        group.roster[0x3d]=0;group.roster[0x40]=0;
    }
    busy
}
///5a080 leader prefix:4b590 must run immediately on the3f transition tozero,
///before finish_leader_preamble examines the possibly changed current roster.
pub fn advance_leader_counter(counter6e:u8)->u8{counter6e.saturating_add(1)}
pub fn begin_leader_preamble(group:&mut GroupSlot)->bool{if group.roster[0x3f]==0{return false;}group.roster[0x3f]-=1;group.roster[0x3f]==0}
pub fn finish_leader_preamble(group:&mut GroupSlot,phase:u8){
    if group.roster[0x40]as i8>0{group.roster[0x40]-=1;}
    if group.roster[0x3b]!=255&&group.roster[0x10]==0&&!matches!(phase,8|5|19){group.roster[0x3b]=255;}
}
///1b5d0 after successful group creation: authored ROBOTS.DAT record+1 is
///saved in+3c; alternate callback mode changes only the current mask to FF.
pub fn initialize_formation(group:&mut GroupSlot,layout:u8,descriptor48:u8,alternate:bool){
    group.roster[8]=if alternate{255}else{layout};group.roster[0x3c]=layout;group.roster[0x41]=descriptor48;
}
///52200's leader-only writer, AFTER506b0 accepted the walking update and50890.
///The caller must retain those gates. Bit71:4 preserves the temporary mask.
pub fn walking_formation(group:&mut GroupSlot,flags71:u8,flags64:u8,terrain:u8){
    if flags71&4==0{group.roster[8]=group.roster[0x3c];}
    if flags64&0x80!=0||terrain&0x7f==23{group.roster[8]=255;}
}
///4f1e0's terrain writer after a valid (x!=10000,in-range) position update.
pub fn terrain_formation(group:&mut GroupSlot,role:i8,terrain:u8){if role!=0&&terrain&0x7f==23{group.roster[8]=255;}}
///3e190's successful/failed secondary route branches, after route work itself.
pub fn secondary_formation(group:&mut GroupSlot,role:i8,created:bool){if role==-1{group.roster[8]=if created{255}else{group.roster[0x3c]};}}
///59c00 after59af0 produced the movement direction. Changing the group angle
///invalidates every registered member's bit80, including non-live members.
pub fn set_segment_direction(group:&mut GroupSlot,members:&mut[Member],direction:u8)->bool{
    let angle=(i16::from(direction).wrapping_mul(-64))&511;
    if group.word(10)==angle{return false;}
    group.set_word(10,angle);
    for &id in &group.roster[19..40]{if id!=255{if let Some(member)=members.iter_mut().find(|m|m.slot==id){member.flags31&=0x7f;}}}
    true
}
#[derive(Clone,Debug)]
pub struct Member{
    pub slot:u8,pub group:u8,pub role:i8,
    pub flags31:u8,pub flags69:u8,pub flags8a:u8,pub flags9c:u8,pub flagsb7:u8,
    pub primary_path:bool,pub secondary_path:bool,
    pub cursor:u16,pub contact_order:u8,
    pub position:[i16;2],
    pub reserved_object:Option<usize>,
    pub promotion:PromotionState,
}
#[derive(Clone,Copy,Debug,Default)]
pub struct ClaimControl{pub flags64:u8,pub protection:u8,pub observed_tick:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct HardwareOrder{
    pub object:u8,
    ///59c60 waypoint;52810 subtracts8 to obtain the actual525a0 actor anchor.
    pub destination:[i16;2],pub contact:u8,pub motion_bonus:u8,
}
impl HardwareOrder{pub fn anchor_destination(self)->[i16;2]{[self.destination[0].wrapping_sub(8),self.destination[1].wrapping_sub(8)]}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct FlagOrder{pub object:u8,pub destination:[i16;2],pub motion_bonus:u8}
///578b0 outer gates plus57860. Unlike59c60 hardware paths this destination
///goes directly to525a0, so it already is the actor anchor (no−8 conversion).
pub fn claim_flag(member:&mut Member,flags64:u8,protection:u8,object:u8,position:[i16;2],object_flags:u8,flag_delay1f:u8)->Option<FlagOrder>{
    if object==255||protection!=0||member.flags31&0x68!=0||flags64&8!=0||object_flags&1!=0||flag_delay1f!=0{return None;}
    member.flags31|=8;member.reserved_object=Some(usize::from(object));member.contact_order=5;Some(FlagOrder{object,destination:position,motion_bonus:3})
}
///578b0's recognized hardware branches. The caller separately proves APC57790
///and owns the OBJECT13 reservation write; no order-name inference is used.
pub fn claim_hardware(member:&mut Member,flags64:u8,protection:u8,object:u8,sprite:i16,position:[i16;2],object_flags:u8,apc_allowed:bool)->Option<HardwareOrder>{
    if object==255||protection!=0||member.flags31&0x68!=0||flags64&8!=0||object_flags&1!=0{return None;}
    let(contact,offset)=match sprite{
        352=>(1,[8i16,-4i16]),1641|1657|5765=>(1,[12,12]),
        5165|5168|5169|5172|5214|5215|5834=>(3,[8,-4]),
        5173 if apc_allowed=>(3,[12,0]),_=>return None,
    };
    member.flags31|=8;member.reserved_object=Some(usize::from(object));member.contact_order=contact;
    Some(HardwareOrder{object,destination:[position[0].wrapping_add(offset[0]),position[1].wrapping_add(offset[1])],contact,motion_bonus:3})
}
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct PromotionState{
    ///Unit+50 dword; promotion copies/sign-extends only its low16 bits.
    pub target_x:i32,
    pub target_y:i16,pub target_aux:i16,
    ///Unit+5c and+61.
    pub order:u8,pub motion:u8,
    pub phase:u8,pub phase_word:i16,
    ///Raw phase7 prefix fields+39/+3c/+3d/+40/+41/+7d.
    pub frame39:u8,pub variant3c:u8,pub timer3d:u8,pub frame40:u8,pub timer41:u8,pub delay7d:u8,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Promotion{pub leader:u8,pub previous_role:usize,pub primary_pointer:u32}

///4b990 after4bba0 has removed/decremented the old leader. It takes the first
///occupied ROLE, preserving all other holes. Raw native path pointers are an
///explicit boundary: Web path vectors must not be passed as invented addresses.
pub fn promote_first_follower(group:&mut GroupSlot,units:&mut[Member],old_leader:u8,native_paths:&mut[u32;240],mut rng:impl FnMut()->u32)->Option<Promotion>{
    let old_index=units.iter().position(|u|u.slot==old_leader)?;
    let Some(role)=(1..21).find(|&role|group.member(role).is_some())else{
        let old=&mut units[old_index];group.set_word(40,old.position[0]);group.set_word(42,old.position[1]);clear_path(old);native_paths[usize::from(old_leader)]=0;return None;
    };
    let id=group.member(role).unwrap();let index=units.iter().position(|u|u.slot==id)?;
    group.roster[19+role]=255;group.roster[19]=id;
    let inherited=units[old_index].promotion.clone();let path=native_paths[usize::from(old_leader)];
    native_paths[usize::from(old_leader)]=0;units[old_index].primary_path=false;
    clear_path(&mut units[index]);native_paths[usize::from(id)]=0;
    let in_pool=((path.wrapping_sub(0x5eee28)as i32)>>3)as u32<=0x12c0;
    let shared=units.iter().any(|u|u.slot!=id&&u.flags31&4!=0&&native_paths[usize::from(u.slot)]==path);
    let transferred=if path==0||path==u32::MAX||in_pool&&!shared{path}else{0};
    native_paths[usize::from(id)]=transferred;
    let promoted=&mut units[index];promoted.role=-1;promoted.primary_path=transferred!=0;
    let p=&mut promoted.promotion;p.target_x=i32::from(inherited.target_x as i16);p.target_y=inherited.target_y;p.target_aux=inherited.target_aux;p.order=inherited.order;p.motion=inherited.motion;
    if p.phase==7{p.phase=1;p.phase_word=1;p.timer3d=8;p.variant3c=rng()as u8&3;p.timer41=0;p.frame40=p.frame39;p.delay7d=((rng()as u16)%53)as u8+12;}
    Some(Promotion{leader:id,previous_role:role,primary_pointer:transferred})
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Split{
    NoFollower,
    InvalidRoster,
    Detached{
        member:u8,source:usize,new_group:Option<usize>,
        ///4bba0 releases this object's reservation bit without clearing+36.
        release_reservation:Option<usize>,
    },
}

fn clear_path(member:&mut Member){member.primary_path=false;member.secondary_path=false;member.flags31&=!0x20;}
///4b140 (also inlined in4b2f0): assigned pointer and other counters survive.
pub fn initialize_cpu_strategy(slot:&mut GroupSlot,member:&Member,current_sector:Option<u32>,alternate_callback:bool){
    if alternate_callback||member.role==0{return;}
    slot.strategy[21]=0;
    if member.flags31&3==1&&member.flags31&4!=0{
        slot.strategy[21]=1;
        if let Some(pointer)=current_sector.filter(|&p|p!=0){slot.strategy[..4].copy_from_slice(&pointer.to_le_bytes());}
        slot.strategy[16]=8;slot.strategy[17]=255;slot.strategy[4..8].fill(0);
    }
}

///4b7b0, including first-empty follower placement with roster holes preserved.
/// `descriptor` is global63238c for a new roster;4c0b0 subsequently replaces it
/// with the source descriptor under4d029a. On a full roster native still writes
/// member.group before failing; member.role remains unchanged.
pub fn append_member(group:&mut GroupSlot,member:&mut Member,slot:usize,descriptor:u32)->bool{
    if slot>=GROUP_SLOTS{return false;}
    member.group=slot as u8;
    if group.roster[18]==0{
        member.role=-1;group.roster[19]=member.slot;group.roster[18]=1;
        group.set_word(10,0);group.roster[17]=0;group.roster[61]=0;group.roster[58]=0;group.roster[59]=255;
        group.roster[12..16].fill(255);group.roster[4..8].fill(0);group.set_word(66,0);
        member.flagsb7&=0xe0;member.cursor=0xabb;group.roster[..4].copy_from_slice(&descriptor.to_le_bytes());
        return true;
    }
    if group.roster[18]==21{return false;}
    if let Some(role)=(0..21).find(|&role|group.roster[19+role]==255){
        group.roster[19+role]=member.slot;group.roster[18]=group.roster[18].wrapping_add(1);member.role=role as i8;return true;
    }
    false
}

///Explicit registration for mission/newly-produced leaders and followers.
/// This combines4b7b0 with the separate4b140 strategy refresh, without sorting
/// surviving roles or compacting holes. No free-slot policy is substituted.
pub fn initialize_member(group:&mut GroupSlot,member:&mut Member,slot:usize,descriptor:u32,current_sector:Option<u32>,alternate_callback:bool)->bool{
    if !append_member(group,member,slot,descriptor){return false;}
    initialize_cpu_strategy(group,member,current_sector,alternate_callback);true
}

///4b2f0's free-slot selection and new-roster simulation writes. The descriptor
/// is explicit because the native function also has a separate UI-name path.
pub fn allocate_single(groups:&mut[GroupSlot;GROUP_SLOTS],member:&mut Member,descriptor:u32,current_sector:Option<u32>,alternate_callback:bool)->Option<usize>{
    let usage=std::array::from_fn(|i|groups[i].usage());let slot=free_group_slot(&usage,member.flags31&3)?;
    if !initialize_member(&mut groups[slot],member,slot,descriptor,current_sector,alternate_callback){return None;}
    groups[slot].roster[63]=4;Some(slot)
}

///4c0b0 executes exactly one last-follower split. `sector_pointer` is17400 of
/// that member's present position; it affects only the new CPU strategy record.
/// A saturated allocator leaves the native malformed role0/old-group result
/// explicit as new_group=None; it must not be called a successful allocation.
pub fn split_last_follower(groups:&mut[GroupSlot;GROUP_SLOTS],units:&mut[Member],source:usize,sector_pointer:Option<u32>,alternate_callback:bool)->Split{
    let Some(group)=groups.get(source)else{return Split::InvalidRoster;};
    if group.roster[18]as i8<=1{return Split::NoFollower;}
    let Some(id)=(1..MEMBERS).rev().find_map(|role|group.member(role))else{return Split::NoFollower;};
    let Some(index)=units.iter().position(|u|u.slot==id)else{return Split::InvalidRoster;};
    let role=units[index].role;
    if !(1..=20).contains(&role)||usize::from(units[index].group)!=source{return Split::InvalidRoster;}
    let member=&mut units[index];let name=groups[source].roster[..4].to_vec();
    clear_path(member);
    //4b1b0 clears every roster's remembered opponent equal to this native unit.
    crate::original_group_reward::clear_target_reports(groups.iter_mut().map(|group| &mut group.roster), id);
    let release_reservation=if member.flags31&8!=0{member.flags31&=!8;member.contact_order=0;member.reserved_object}else{None};
    member.flags69&=!0x80;
    groups[source].roster[19+role as usize]=255;
    groups[source].roster[18]=groups[source].roster[18].wrapping_sub(1);
    groups[source].roster[9]|=0x40;
    member.role=0;member.group=source as u8; //saved group is restored by4c0b0.
    let usages=std::array::from_fn(|i|groups[i].usage());
    let Some(target)=free_group_slot(&usages,member.flags31&3)else{
        return Split::Detached{member:id,source,new_group:None,release_reservation};
    };
    let g=&mut groups[target];let descriptor=u32::from_le_bytes(name.try_into().unwrap());
    let added=append_member(g,member,target,descriptor);debug_assert!(added);g.roster[63]=4;
    //4d029a is set around4b2f0: retain the source descriptor instead of naming.
    initialize_cpu_strategy(g,member,sector_pointer,alternate_callback);
    Split::Detached{member:id,source,new_group:Some(target),release_reservation}
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Rejoin{
    pub cleared_path:bool,
    pub target:Option<[i16;2]>,
    ///At a special group+32/+38 destination, native50b30 decides whether to
    ///cancel all roster special-contact routes (59d80/59e60). No route is issued.
    pub special_arrival:bool,
}
#[derive(Clone,Copy,Debug)]
pub struct ContactClaim{pub flags31:u8,pub group:u8,pub object:i8}

///50b30's actual foreign-claim/deleted-object failure predicate. The claimant's
/// unit+B5 is sign-extended, whereas the arriving member's unit+36 is unsigned;
/// preserve that asymmetry for original object slots128..199.
pub fn special_contact_allowed(claims:&[ContactClaim],group:u8,object:u8,object_x:i16)->bool{
    object_x!=-1&&!claims.iter().any(|c|c.flags31&4!=0&&i16::from(c.object)==i16::from(object)&&c.group!=group)
}

///4e910 with real4d430 geometry. `transport_uniform` is54070||540b0 for the
/// ORIGINAL global632194 roster, before4e910 selects member.group. `offset` is
/// the signed formation-mask pair for member.roleâˆ’1, and sine is the native
/// SINCOS table (native_math::sine is independently verified against all640).
pub fn rejoin(member:&mut Member,group:&mut GroupSlot,leader_position:[i16;2],transport_uniform:bool,forced_formation:bool,offset:[i8;2],sine:impl Fn(i32)->i32)->Rejoin{
    let mut result=Rejoin{cleared_path:false,target:None,special_arrival:false};
    if !transport_uniform||member.flags9c&4!=0{return result;}
    clear_path(member);result.cleared_path=true;
    let flags=group.roster[9];group.roster[9]|=1;
    let target=formation_target(member,group,leader_position,forced_formation,offset,sine);
    group.roster[9]=flags;result.target=target.target;result.special_arrival=target.special_arrival;result
}
///4d430 without its caller's temporary enable bit or path clearing. It uses
///the caller's current global roster, which need not be member.group.
pub fn formation_target(member:&Member,group:&mut GroupSlot,leader_position:[i16;2],forced_formation:bool,offset:[i8;2],sine:impl Fn(i32)->i32)->Rejoin{
    let mut result=Rejoin{cleared_path:false,target:None,special_arrival:false};
    if member.role<=0||member.flags31&8!=0{return result;}
    if !forced_formation&&member.flags8a&0x10!=0{
        let target=[group.word(50),group.word(56)];
        if member.position==target{result.special_arrival=true;}else{result.target=Some(target);}
        return result;
    }
    if !forced_formation&&group.roster[9]&1==0{return result;}
    let mut angle=group.word(10);if angle>512{angle=0;group.set_word(10,0);}
    let s=sine(i32::from(angle));let c=sine(i32::from(angle)+128);
    let x=i32::from(offset[0]);let y=i32::from(offset[1]);
    result.target=Some([
        leader_position[0].wrapping_add(((y*s>>10)+(x*c>>10))as i16),
        leader_position[1].wrapping_add(((y*c>>10)-(s*x>>10))as i16),
    ]);
    result
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum FollowRoute{Primary([i16;2]),Direct([i16;2])}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct IdleFollow{pub resume_primary:bool,pub motion_bonus:Option<u8>,pub route:Option<FollowRoute>,pub special_arrival:bool}
///4d570 through route emission. 52960 and525a0 are explicit path-engine
///boundaries: unfinished primary paths resume before formation lookup, while
///finish_idle_follow receives525a0's real return and post-call remaining48.
pub fn idle_follow(member:&mut Member,group:&mut GroupSlot,leader_position:[i16;2],primary_unfinished:bool,forced:bool,offset:[i8;2],sine:impl Fn(i32)->i32)->IdleFollow{
    let mut result=IdleFollow{resume_primary:false,motion_bonus:None,route:None,special_arrival:false};
    if member.flags31&0x40!=0{return result;}
    if primary_unfinished{result.resume_primary=true;return result;}
    if group.roster[0x3f]!=0{return result;}
    result.motion_bonus=Some(if group.roster[9]&1!=0||member.flags9c&0x10!=0{4}else{0});
    let target=formation_target(member,group,leader_position,forced,offset,sine);result.special_arrival=target.special_arrival;
    if let Some(target)=target.target{
        let dx=i32::from(target[0])-i32::from(member.position[0]);let dy=i32::from(target[1])-i32::from(member.position[1]);
        if forced||dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))>1024{result.route=Some(FollowRoute::Primary(target));member.flags31&=0x7f;result.motion_bonus=Some(4);}
        else{result.route=Some(FollowRoute::Direct(target));}
    }
    result
}
pub fn finish_idle_follow(member:&mut Member,already_at_goal:bool,remaining48:i16){member.promotion.phase=7;if already_at_goal&&remaining48<6{member.flags31|=0x80;}}
///One slot of4e9b0's sequential240-slot loop. Sector lookup and any4e910
///follow-up are performed between slots by the caller, preserving side effects.
pub fn clear_personal_capture_in_sector(member:&mut Member,team:u8,actor_sector:Option<usize>,member_sector:Option<usize>)->bool{
    if actor_sector.is_none()||member.flags31&4==0||member.flags31&3!=team||member.flags8a&4==0||actor_sector!=member_sector{return false;}
    member.flags8a&=!4;true
}
#[cfg(test)]mod tests{
 use super::*;
 fn hash(values:impl IntoIterator<Item=u32>)->u32{values.into_iter().fold(2166136261u32,|h,n|(h^n).wrapping_mul(16777619))}
 #[test]fn leader_preamble_counters_and_phase_selector_match_256_native_executions(){
  let mut values=Vec::new();for s in 0..256{let mut group=GroupSlot::empty();let role=[-1,0,1][(s>>1)%3];let mut counter=[0,1,254,255][(s>>3)&3];let phase=[0,5,8,19,7][(s>>5)%5];group.roster[0x3f]=[0,1,4,255][(s>>2)&3];group.roster[0x40]=[0,1,127,128,255][(s>>4)%5];group.roster[0x3b]=if s&1!=0{255}else{17};group.roster[0x10]=u8::from(s&128!=0);group.roster[9]=(s*7)as u8;
   if role<0{counter=advance_leader_counter(counter);let _empty_follower_expiry=begin_leader_preamble(&mut group);finish_leader_preamble(&mut group,phase);}
   values.extend([s as u32,u32::from(counter),u32::from(group.roster[0x3f]),u32::from(group.roster[0x40]),u32::from(group.roster[0x3b]),u32::from(group.roster[9])]);
  }
  assert_eq!(hash(values),28633721,"ZNativeLeaderGroupPrefixOracle actual5a080 prefix including real empty4b590 callback");
 }
 #[test]fn sector_personal_capture_cleanup_and_actor_class_rejoin_match_32_native_whole_pools(){
  let mut values=Vec::new();for scenario in 0..32{
   let sector=|x:i16|if(0..160).contains(&x){Some(0)}else if(160..480).contains(&x){Some(1)}else{None};
   let origin=sector(if scenario&8!=0{-32}else if scenario&4!=0{400}else{80});let mut flags=Vec::new();let mut joins=Vec::new();
   for id in 0..240{let mut member=Member{slot:id as u8,group:0,role:0,flags31:if id==239{4|(scenario%4)as u8}else{((id+scenario)%4)as u8|if(id+scenario)%5!=0{4}else{0}},flags69:0,flags8a:0xa0|if(id+scenario)%2==0{4}else{0},flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:PromotionState::default()};
    let x=if id==239{if scenario&8!=0{-32}else if scenario&4!=0{400}else{80}}else if(id+scenario)%3==0{400}else{80};
    if clear_personal_capture_in_sector(&mut member,(scenario%4)as u8,origin,sector(x))&&scenario%3!=2{joins.push(id as u32);}flags.push(u32::from(member.flags8a));
   }
   values.extend([scenario as u32,hash(flags),joins.len()as u32,hash(joins)]);
  }
  assert_eq!(hash(values),2652041649,"ZNativeGroupFlagCleanupOracle real4e9b0 +17400,4e910 follow-up recorded as boundary");
 }
 #[test]fn idle_follow_primary_precedence_geometry_and_phase_match_1024_native_executions(){
  let mut values=Vec::new();for s in 0..1024{
   let mut group=GroupSlot::empty();group.roster[9]=0x46|if s&8!=0{1}else{0};group.roster[0x3f]=u8::from(s&4!=0);group.set_word(50,500);group.set_word(56,800);group.set_word(10,if s%5==0{513}else{(s%513)as i16});
   let role=[-1,0,1,2][(s>>4)&3];let mut member=Member{slot:1,group:0,role,flags31:5|128|if s&1!=0{64}else{0}|if s&64!=0{8}else{0},flags69:0,flags8a:if s&256!=0{16}else{0},flags9c:if s&512!=0{16}else{0},flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[match s%3{0=>500,1=>480,_=>400},800],reserved_object:None,promotion:PromotionState{phase:3,..Default::default()}};
   let offset=if role==2{[-9,11]}else{[3,7]};let result=idle_follow(&mut member,&mut group,[500,800],s&2!=0,s&128!=0,offset,|i|(i*73)%2048-1024);
   let (route,target)=match result.route{None=>(0,[0,0]),Some(FollowRoute::Primary(p))=>(1,p),Some(FollowRoute::Direct(p))=>{finish_idle_follow(&mut member,s%2!=0,(s%9)as i16);(2,p)}};
   values.extend([s as u32,route,target[0]as u32,target[1]as u32,u32::from(result.resume_primary||route==1),u32::from(result.special_arrival),u32::from(member.flags31),u32::from(result.motion_bonus.unwrap_or(99)),u32::from(member.promotion.phase),group.word(10)as u32]);
  }
  assert_eq!(hash(values),2139542033,"ZNativeIdleFollowOracle real4d570/4d430 with explicit52960/525a0/path emission boundaries");
 }
 #[test]fn hardware_claim_flags_targets_and_apc_group_predicate_match_512_native_executions(){
  let mut values=Vec::new();let sprites=[352,5165,1641,1657,5765,5169,5168,5172,5173,5214,5834,5215,5317];
  for scenario in 0..512{
   let flags31=5|if scenario&8!=0{8}else{0}|if scenario&16!=0{32}else{0}|if scenario&32!=0{64}else{0};let object_flags=0x70|if scenario&1!=0{1}else{0};
   let mut member=Member{slot:0,group:0,role:if scenario&2!=0{1}else{-1},flags31,flags69:0,flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:PromotionState::default()};
   let apc_allowed=scenario&2==0&&scenario&4==0&&scenario&256==0;
   let result=claim_hardware(&mut member,if scenario&128!=0{8}else{0},if scenario&64!=0{1}else{0},17,sprites[scenario%13],[300,400],object_flags,apc_allowed);let target=result.map_or([0,0],|o|o.destination);
   values.extend([scenario as u32,u32::from(result.is_some()),target[0]as u32,target[1]as u32,u32::from(member.flags31),u32::from(object_flags|u8::from(result.is_some())),u32::from(member.contact_order),member.reserved_object.unwrap_or(0)as u32,result.map_or(0,|o|u32::from(o.motion_bonus))]);
  }
  assert_eq!(hash(values),233107659,"ZNativeHardwareClaimOracle real578b0/57790/41c8b0; final path acceptance isolated");
 }
 #[test]fn accepted_segment_direction_matches_128_complete_native_group_invalidation_executions(){
  let mut values=Vec::new();
  for scenario in 0..128{
   let mut group=GroupSlot::empty();for(j,b)in group.roster.iter_mut().enumerate(){*b=(scenario*7+j)as u8;}for role in 0..21{group.roster[19+role]=if(role+scenario)%5==0{255}else{role as u8};}group.set_word(10,(scenario%8*64)as i16);
   let mut members=(0..21).map(|id|Member{slot:id as u8,group:0,role:0,flags31:(scenario*11+id*7)as u8,flags69:0,flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:PromotionState::default()}).collect::<Vec<_>>();
   let dx=(scenario*13%27)as i32-13;let dy=(scenario*7%27)as i32-13;
   set_segment_direction(&mut group,&mut members,crate::native_math::direction(dx,dy));
   values.extend([scenario as u32,hash(group.roster.iter().map(|&n|u32::from(n))),hash(members.iter().map(|m|u32::from(m.flags31)))]);
  }
  assert_eq!(hash(values),3346958326,"ZNativeGroupDirectionOracle,real59c00→59af0 without stubs");
 }
 #[test]fn temporary_column_uses_the_signed_native_preceding_pointer_and_all_twenty_offsets(){
  let mut values=Vec::new();
  for scenario in 0..20{
   let role=scenario+1;let mut group=GroupSlot::empty();group.roster[8]=255;group.roster[9]=0x46;group.set_word(10,scenario*19);
   let mut member=Member{slot:1,group:1,role:role as i8,flags31:5|32,flags69:0,flags8a:0,flags9c:0,flagsb7:0,primary_path:true,secondary_path:true,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:PromotionState::default()};
   let result=rejoin(&mut member,&mut group,[500,800],true,false,[0,(role*10)as i8],|i|(i*73)%2048-1024);let target=result.target.unwrap();
   values.extend([scenario as u32,1,target[0]as u32,target[1]as u32,u32::from(member.primary_path),u32::from(member.secondary_path),u32::from(member.flags31),group.word(10)as u32,u32::from(result.special_arrival),u32::from(group.roster[9])]);
  }
  assert_eq!(hash(values),459641413,"ZNativeColumnRejoinOracle,actual4e910→4d430 with nativeFF mask");
 }
 #[test]fn authored_walking_terrain_and_secondary_formation_writers_match_native_instruction_blocks(){
  let mut values=Vec::new();
  for scenario in 0..256{
   let mut group=GroupSlot::empty();for(j,b)in group.roster.iter_mut().enumerate(){*b=(scenario*7+j)as u8;}
   let layout=(scenario%5)as u8;let flags71=if scenario&2!=0{4}else{0};let flags64=if scenario&4!=0{128}else{0};let terrain=[0,23,151,35][(scenario>>3)&3];let role=[-1,0,1,20][(scenario>>5)&3];
   initialize_formation(&mut group,layout,(scenario+17)as u8,scenario&1!=0);values.extend([scenario as u32,group.roster[8]as u32,group.roster[60]as u32,group.roster[65]as u32]);
   walking_formation(&mut group,flags71,flags64,terrain);values.push(group.roster[8]as u32);
   terrain_formation(&mut group,role,terrain);values.push(group.roster[8]as u32);
   secondary_formation(&mut group,role,true);values.push(group.roster[8]as u32);
   secondary_formation(&mut group,role,false);values.push(group.roster[8]as u32);
  }
  assert_eq!(hash(values),2779638353,"ZNativeFormationWritersOracle,256 exact bounded instruction sequences");
 }
 #[test]fn conditional_compaction_and_counters_match_128_native_complete_pool_executions(){
  let mut values=Vec::new();
  for scenario in 0..32{
   let mut members=(0..240).map(|id|Member{slot:id as u8,group:0,role:0,flags31:0,flags69:0,flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:PromotionState::default()}).collect::<Vec<_>>();
   let mut groups=std::array::from_fn(|g|{
    let mut group=GroupSlot::empty();for(j,b)in group.roster.iter_mut().enumerate(){*b=(scenario*7+g*3+j)as u8;}group.roster[19..40].fill(255);
    group.roster[9]=if(g+scenario)%2==0{64}else{5};group.roster[16]=((g+scenario)%4)as u8;group.roster[18]=0;
    if g<8{for role in 0..21{if role==0||(role+scenario+g)%5>=2{let id=g*21+role;group.roster[19+role]=id as u8;group.roster[18]+=1;members[id].role=if role==0{-1}else{role as i8+if scenario%3==0{3}else{0}};}}}group
   });
   for tick in 0..4{
    let busy=advance_group_counters(&mut groups,&mut members);
    values.extend([scenario as u32,tick,busy.into(),hash(groups.iter().flat_map(|g|g.roster.iter().map(|&b|u32::from(b)))),hash(members.iter().map(|m|m.role as u32))]);
   }
  }
  assert_eq!(hash(values),3243896274,"ZNativeGroupCountersOracle,4 native passes in32 complete pool fixtures");
 }
#[test]fn promotion_membership_fields_raw_paths_and_phase7_match_64_native_executions(){
  const EXPECTED:[[u32;4];64]=[[3,2552257433,432344315,0],[3,155995337,3815343154,0],[1,2793195104,1872294564,0],[4294967295,81719335,1457858659,0],[3,1550946193,1295106483,0],[3,3407945633,238168364,0],[1,3328829592,3155759937,0],[4294967295,1105655823,3479150087,0],[3,1153397561,3052670595,2],[3,676107433,2200184963,2],[1,1608989216,318774984,2],[4294967295,127723687,3065433971,0],[3,3558267425,4212331386,2],[3,3037128049,2651337439,2],[1,4115051592,3022362359,2],[4294967295,2056551967,828204919,0],[3,2105682265,14107099,0],[3,1094968137,2194306834,0],[1,2128129120,3149932132,0],[4294967295,2088736775,3189486163,0],[3,2294693617,2271307603,0],[3,561433345,2247555916,0],[1,2815826968,74339585,0],[4294967295,3631230735,1305299383,0],[3,2305821113,1274870225,2],[3,411526889,3871792281,2],[1,73752544,2943022598,2],[4294967295,3573483719,2521526307,0],[3,2821796257,1034955956,2],[3,3260910705,4161749846,2],[1,1040723432,3508211361,2],[4294967295,4158262847,572216679,0],[3,1876997977,650132539,0],[3,1514262665,1075039858,0],[1,619128992,3047686884,0],[4294967295,2348615271,2949280323,0],[3,777669649,212308211,0],[3,4090858145,503943532,0],[1,1736490776,1948960321,0],[4294967295,3409361039,3193433895,0],[3,2172796409,2602968327,2],[3,3427830057,2438950751,2],[1,2482851808,2112238117,2],[4294967295,3597172455,2256580435,0],[3,781479265,3981033738,2],[3,4287029041,2599918927,2],[1,3941558344,2959543523,2],[4294967295,1077095583,3879568023,0],[3,3617579289,601560411,0],[3,1867231497,3749625362,0],[1,2775110816,2924623780,0],[4294967295,3620960775,4194976947,0],[3,2625148849,2123906003,0],[3,1134347969,1397833676,0],[1,951604632,1833380737,0],[4294967295,658782863,2775400535,0],[3,3278016249,2086125729,2],[3,2140700265,3343566837,2],[1,1144175776,2374680195,2],[4294967295,2377869383,301933187,0],[3,4051248353,2366899992,2],[3,3399054001,1909301961,2],[1,1123972520,78463869,2],[4294967295,4015318527,3708442503,0]];
  for scenario in 0..64{
   let mut group=GroupSlot::empty();for(j,b)in group.roster.iter_mut().enumerate(){*b=(scenario*7+j)as u8;}group.roster[19..40].fill(255);
   let mode=scenario%4;group.roster[18]=if mode==3{0}else if mode==2{2}else{3};
   if mode!=3{group.roster[19+if mode==0{1}else if mode==1{2}else{5}]=if mode==2{1}else{3};if mode!=2{group.roster[24]=1;}group.roster[39]=2;}
   let base=0x5eee28u32;let table=[0,u32::MAX,base+800,base+0x12c0*8,base+0x12c1*8,base-8,base+960,base+1];let path=table[scenario%8];
   let mut paths=[0u32;240];
   let mut members=(0..5).map(|i|{
    let pointer=if i==0{path}else if i==4&&scenario%8==6{path}else{base+1600+i as u32*8};paths[i]=pointer;
    Member{slot:i as u8,group:if i==0{255}else{120},role:if i==0{0}else if i==1{5}else if i==2{20}else if mode==0{1}else{2},flags31:if i==0{0}else{5|32},flags69:0,flags8a:0,flags9c:0,flagsb7:0,primary_path:pointer!=0,secondary_path:true,cursor:0,contact_order:0,position:[32+i as i16*2,64],reserved_object:None,
     promotion:PromotionState{target_x:if i==0{0x12348000+scenario as i32*333}else{100000+i as i32},target_y:if i==0{-300+scenario as i16*5}else{100+i as i16},target_aux:if i==0{1000+scenario as i16*7}else{200+i as i16},order:if i==0{17+scenario as u8}else{33},motion:if i==0{99-scenario as u8}else{44},phase:if i!=0&&scenario&8!=0{7}else{1},phase_word:-7,frame39:10+i as u8,variant3c:21,timer3d:22,frame40:23,timer41:24,delay7d:25}}
   }).collect::<Vec<_>>();
   let mut rng=123u32+scenario as u32;let mut calls=0;
   let result=promote_first_follower(&mut group,&mut members,0,&mut paths,||{calls+=1;rng=rng.wrapping_mul(1664525).wrapping_add(1013904223);rng});
   let rh=hash(group.roster.iter().map(|&n|u32::from(n)));
   let mh=hash(members.iter().flat_map(|m|{let p=&m.promotion;[u32::from(m.slot),u32::from(m.group),m.role as u32,u32::from(m.flags31),u32::from(m.primary_path),u32::from(m.secondary_path),paths[usize::from(m.slot)],p.target_x as u32,p.target_y as u32,p.target_aux as u32,u32::from(p.order),u32::from(p.motion),u32::from(p.phase),p.phase_word as u32,u32::from(p.frame39),u32::from(p.variant3c),u32::from(p.timer3d),u32::from(p.frame40),u32::from(p.timer41),u32::from(p.delay7d)]}));
   assert_eq!([result.map_or(u32::MAX,|p|u32::from(p.leader)),rh,mh,calls],EXPECTED[scenario],"scenario {scenario}");
  }
 }
 #[test]fn rejoin_gates_and_geometry_match_256_real_native_executions(){
  let mut values=Vec::new();let roles=[-1,0,1,20,3,2,4,1];
  for scenario in 0..256{
   let mut group=GroupSlot::empty();group.roster[9]=0x46;group.set_word(10,if scenario%5==0{513}else{(scenario*71)%513});group.set_word(50,100);group.set_word(56,200);
   let mut member=Member{slot:1,group:1,role:roles[scenario as usize%8],flags31:5|32|if scenario&32!=0{8}else{0},flags69:0,flags8a:if scenario&8!=0{16}else{0},flags9c:if scenario&16!=0{4}else{0},flagsb7:0,primary_path:true,secondary_path:true,cursor:0,contact_order:0,position:if scenario&128!=0{[100,200]}else{[0,0]},reserved_object:Some(0),promotion:PromotionState::default()};
   let role=member.role;let offset=if role>0{let i=i32::from(role)-1;[((i*13)%255-127)as i8,((i*29)%255-127)as i8]}else{[0,0]};
   let leader=if scenario&64!=0{[32760,-32760]}else{[500,800]};
   let result=rejoin(&mut member,&mut group,leader,matches!(scenario&3,0|3),scenario&4!=0,offset,|i|(i*73)%2048-1024);
   let target=result.target.unwrap_or([0,0]);
   values.extend([scenario as u32,u32::from(result.target.is_some()),target[0]as u32,target[1]as u32,u32::from(member.primary_path),u32::from(member.secondary_path),u32::from(member.flags31),group.word(10)as u32,u32::from(result.special_arrival),u32::from(group.roster[9])]);
  }
  assert_eq!(hash(values),1763671398,"ZNativeRejoinOracle:22 emitted routes and2 special arrivals");
 }
 #[test]fn real_native_follower_allocator_matches_all_64_complete_roster_strategy_and_member_traces(){
  const EXPECTED:[[u32;8];64]=[[1,121,4294967295,287274020,701636567,2187437701,1174114169,255],[2,122,4294967295,694154483,3212303903,3333386984,2798978515,255],[3,0,4294967295,2527243014,3310125104,1679396491,3259354632,255],[4,125,4294967295,3745005643,803458988,37954232,4001036192,255],[5,120,0,345451319,880290789,2187437701,1837441074,254],[4294967295,4294967295,0,2413093053,1634276037,2187437701,2594282043,255],[4294967295,4294967295,0,1127962932,1763588389,2187437701,3370894453,255],[20,121,4294967295,327514763,864874324,2187437701,1340237177,254],[9,0,4294967295,3999741447,2958092146,2187437701,3697006885,255],[10,0,4294967295,2291087105,3780097181,3333386984,2432973042,255],[11,0,4294967295,2268152886,819611316,1679396491,3008778275,255],[12,125,4294967295,724596267,2761431928,37954232,1952339771,255],[13,120,0,506739143,203785285,2187437701,287979161,254],[4294967295,4294967295,0,3423201149,1799041093,2187437701,3255685572,255],[4294967295,4294967295,0,554061596,960478533,2187437701,1873096474,255],[20,0,4294967295,3129215858,3604108923,2187437701,1630194001,254],[17,121,4294967295,733138788,2197802533,2187437701,2654932489,255],[18,122,4294967295,1979449523,2224509541,3333386984,72359107,255],[19,0,4294967295,1706838022,2051669925,1679396491,1140243000,255],[20,125,4294967295,331351051,3221799461,37954232,128519472,255],[1,120,0,1024603147,667231333,2187437701,1564001062,254],[4294967295,4294967295,0,2079491517,2024355077,2187437701,2594282043,255],[4294967295,4294967295,0,1646711712,758141605,2187437701,3370894453,255],[20,121,4294967295,4087979163,1063959141,2187437701,1340237177,254],[5,0,4294967295,2751053911,415488613,2187437701,2903128321,255],[6,0,4294967295,4103670013,205997125,3333386984,1417631342,255],[7,0,4294967295,997669030,3140793253,1679396491,2902762199,255],[8,125,4294967295,436142651,1455700485,37954232,577948735,255],[9,120,0,377256627,117514053,2187437701,2011654917,254],[4294967295,4294967295,0,2917406589,3235439685,2187437701,3255685572,255],[4294967295,4294967295,0,688284008,1962822789,2187437701,1873096474,255],[20,0,4294967295,4056984851,4120011301,2187437701,1630194001,254],[13,121,4294967295,2124520452,46183607,2187437701,753306045,255],[14,122,4294967295,3519773675,4053145855,3333386984,1283874887,255],[15,0,4294967295,3012535318,2176356112,1679396491,1600900948,255],[16,125,4294967295,1019099451,2434558988,37954232,2141786068,255],[17,120,0,2445513467,2218412645,2187437701,299347830,254],[4294967295,4294967295,0,3790635389,3220510085,2187437701,2620121627,255],[4294967295,4294967295,0,3317460144,2549412005,2187437701,1517968213,255],[20,121,4294967295,233338463,158305588,2187437701,1608870009,254],[1,0,4294967295,3699836847,4253030034,2187437701,3658224861,255],[2,0,4294967295,489819233,361295101,3333386984,4195281914,255],[3,0,4294967295,3984371742,1834033812,1679396491,3373120411,255],[4,125,4294967295,3165197539,1996499288,37954232,3396250451,255],[5,120,0,949150007,2174353029,2187437701,2931741377,254],[4294967295,4294967295,0,1781585405,2284884101,2187437701,484055140,255],[4294967295,4294967295,0,3446988724,730198021,2187437701,2119123962,255],[20,0,4294967295,1513784546,2402648091,2187437701,877566481,254],[9,121,4294967295,1792914300,2103973797,2187437701,385961761,255],[10,122,4294967295,2498468059,1368184933,3333386984,656487035,255],[11,0,4294967295,278227390,258490277,1679396491,1747590320,255],[12,125,4294967295,1775721571,2639142949,37954232,4164396392,255],[13,120,0,3040638807,335443301,2187437701,3610307690,254],[4294967295,4294967295,0,2842166141,1116906437,2187437701,2620121627,255],[4294967295,4294967295,0,10673596,842287013,2187437701,1517968213,255],[20,121,4294967295,2250294863,2469941733,2187437701,1608870009,254],[17,0,4294967295,3527431855,1681528549,2187437701,2196669741,255],[18,0,4294967295,530643601,732025861,3333386984,222236778,255],[19,0,4294967295,3957728190,3077935397,1679396491,1918511723,255],[20,125,4294967295,582634371,2082347845,37954232,3187749379,255],[1,120,0,2905874075,318063621,2187437701,3850642269,254],[4294967295,4294967295,0,784542461,1714071045,2187437701,484055140,255],[4294967295,4294967295,0,2289526016,2665867589,2187437701,2119123962,255],[20,0,4294967295,2701610994,2813910181,2187437701,877566481,254]];
  for scenario in 0..64{
   let mut groups:[GroupSlot;240]=std::array::from_fn(|i|{
    let mut g=GroupSlot::empty();for(j,b)in g.roster.iter_mut().enumerate(){*b=(scenario*7+i*3+j)as u8;}for(j,b)in g.strategy.iter_mut().enumerate(){*b=(scenario*11+i*5+j)as u8;}g.roster[18]=1;g.roster[19..40].fill(255);g.roster[19]=200;g
   });
   let make_free=|g:&mut GroupSlot,cooldown|{g.roster[18]=0;g.roster[19]=255;g.cooldown=cooldown;};
   let mode=scenario%8;let mut count=2+scenario%20;
   if matches!(mode,0|1|7){make_free(&mut groups[0],0);make_free(&mut groups[121],if mode==1{7}else{0});make_free(&mut groups[122],0);}
   else if mode==2{make_free(&mut groups[0],7);make_free(&mut groups[125],7);}else if mode==3{make_free(&mut groups[125],7);}
   if mode==5{count=1;}if mode==7{count=2;}
   groups[120].roster[18]=count as u8;
   for role in 0..21{groups[120].roster[19+role]=if role==0{0}else if mode==6{255}else if mode==7{if role==20{20}else{255}}else if role<count{role as u8}else{255};}
   let mut members=(0..21).map(|i|Member{slot:i as u8,group:120,role:if i==0{-1}else{i as i8},flags31:(if scenario&8!=0{0}else{1})|4|if scenario&32!=0{32}else{0}|if scenario&4!=0{8}else{0},flags69:255,flags8a:0,flags9c:0,flagsb7:159,primary_path:scenario&1!=0,secondary_path:scenario&2!=0,cursor:0x1234,contact_order:7,position:[32+i as i16*2,64],reserved_object:(scenario&4!=0).then_some(0),promotion:PromotionState::default()}).collect::<Vec<_>>();
   let outcome=split_last_follower(&mut groups,&mut members,120,Some(0x5c3f0c),scenario&16!=0);
   let(member,group,role,reservation)=match outcome{Split::NoFollower=>(u32::MAX,u32::MAX,0,255),Split::InvalidRoster=>panic!("invalid fixture"),Split::Detached{member,release_reservation,..}=>{let m=&members[member as usize];(u32::from(member),u32::from(m.group),m.role as u32,if release_reservation.is_some(){254}else{255})}};
   let rh=hash(groups.iter().flat_map(|g|g.roster.iter().map(|&n|u32::from(n))));let sh=hash(groups.iter().flat_map(|g|g.strategy.iter().map(|&n|u32::from(n))));let ch=hash(groups.iter().map(|g|u32::from(g.cooldown)));
   let mh=hash(members.iter().flat_map(|m|[u32::from(m.slot),u32::from(m.group),m.role as u32,u32::from(m.flags31),u32::from(m.flags69),u32::from(m.flagsb7),u32::from(m.primary_path),u32::from(m.secondary_path),u32::from(m.cursor),u32::from(m.contact_order),m.position[0]as u32,m.position[1]as u32]));
   assert_eq!([member,group,role,rh,sh,ch,mh,reservation],EXPECTED[scenario],"scenario {scenario}");
  }
 }
}
