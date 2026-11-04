//! Game ownership adapter for the verified240 native roster slots. Browser
//! squad/entity IDs remain separate from native group/member slot numbers.
//! Web-owned paths are transferred as owned vectors; no raw pool pointer is
//! fabricated for the native promotion oracle or its sharing checks.
use super::{Game,Unit,stats,original_combat,native_math,native_ai_tasks as task,original_group_lifecycle as life};
use std::collections::{BTreeMap,BTreeSet,VecDeque};

#[derive(Clone)]
struct OrderSnapshot{order:String,target:Option<usize>,dest:Option<(f64,f64)>,path:VecDeque<(f64,f64)>,march_resume:Option<(f64,f64)>}
impl OrderSnapshot{fn of(u:&Unit)->Self{Self{order:u.order.clone(),target:u.target,dest:u.dest,path:u.path.clone(),march_resume:u.march_resume}}}

pub struct State{
    pub pool:Box<[life::GroupSlot;240]>,
    pub members:Box<[Option<life::Member>;240]>,
    bindings:BTreeMap<(i32,usize),usize>,
    entities:Box<[Option<usize>;240]>,
    snapshots:Box<[Option<OrderSnapshot>;240]>,
    before_removal:Box<[Option<OrderSnapshot>;240]>,
    primary_orders:Box<[Option<OrderSnapshot>;240]>,
    contact_targets:Box<[Option<u8>;240]>,
    contact_claims:Box<[i8;240]>,
    claim_controls:Box<[life::ClaimControl;240]>,
    produced_formations:BTreeMap<(i32,usize),u8>,
    constructed_descriptors:BTreeMap<(i32,usize),u8>,
    direct_placements:BTreeSet<(i32,usize)>,
    last_tick:Option<u64>,
    last_cooldown_tick:Option<u64>,
    pub current_roster:Option<usize>,
    ///The current Game has Web paths and lacks the full native unit35 phase
    ///record. Exact raw promotion remains available through the pure API.
    pub raw_paths_available:bool,
}
impl Default for State{
    fn default()->Self{Self{pool:boxed_slots(life::GroupSlot::empty),members:boxed_slots(||None),bindings:BTreeMap::new(),entities:boxed_slots(||None),snapshots:boxed_slots(||None),before_removal:boxed_slots(||None),primary_orders:boxed_slots(||None),contact_targets:boxed_slots(||None),contact_claims:boxed_slots(||-1),claim_controls:boxed_slots(life::ClaimControl::default),produced_formations:BTreeMap::new(),constructed_descriptors:BTreeMap::new(),direct_placements:BTreeSet::new(),last_tick:None,last_cooldown_tick:None,current_roster:None,raw_paths_available:false}}
}
//Build through Vec so debug/Wasm compilation never constructs a large inline
//temporary array before allocating its Box.
fn boxed_slots<T>(make:impl FnMut()->T)->Box<[T;240]>{std::iter::repeat_with(make).take(240).collect::<Vec<_>>().into_boxed_slice().try_into().ok().unwrap()}
impl State{
    pub fn clear(&mut self){*self=Self::default();}
    ///460430 resets the72-byte roster through4b530, but its24-byte strategy
    ///loop only writes pointer+0=FFFFFFFF and flags+16=0. Even the inactive
    ///rows retain their old value/task/assignment fields until later writers.
    pub fn clear_for_mission_preserving_strategy_values(&mut self){
        let strategy=self.pool.iter().map(|g|g.strategy).collect::<Vec<_>>();
        *self=Self::default();for(group,old)in self.pool.iter_mut().zip(strategy){group.strategy=old;group.strategy[..4].fill(255);group.strategy[16]=0;}
    }
    fn primary_snapshot(&self,slot:usize,u:&Unit)->OrderSnapshot{
        //578b0's temporary contact path does not replace native50/54/5c or
        //the original group move. The Web UI names that temporary path enter.
        if u.initiative.is_some()&&u.march_resume.is_some(){if let Some(order)=self.primary_orders[slot].clone(){return order;}}
        OrderSnapshot::of(u)
    }
    fn remember_primary(&mut self,slot:usize,u:&Unit){if u.order=="move"&&u.initiative.is_none(){self.primary_orders[slot]=Some(OrderSnapshot::of(u));}}
    pub fn group_slot(&self,team:i32,squad:usize)->Option<usize>{self.bindings.get(&(team,squad)).copied()}
    ///A source identity-preserving UNIT conversion can change only its Web
    ///render entity. Detachment/rejoin belongs to its explicit native event.
    pub fn remap_entity(&mut self,slot:usize,old:usize,new:usize)->bool{let Some(entity)=self.entities.get_mut(slot)else{return false;};if *entity!=Some(old){return false;}*entity=Some(new);true}
    pub fn role(&self,unit_slot:usize)->Option<i8>{self.members.get(unit_slot)?.as_ref().map(|m|m.role)}
    pub fn formation_layout(&self,team:i32,squad:usize)->Option<u8>{Some(self.pool[self.group_slot(team,squad)?].roster[8])}
    pub fn leader_slot(&self,team:i32,squad:usize)->Option<usize>{self.pool[self.group_slot(team,squad)?].member(0).map(usize::from)}
    pub fn roster(&self,team:i32,squad:usize)->Vec<Option<usize>>{self.group_slot(team,squad).map(|slot|(0..21).map(|role|self.pool[slot].member(role).map(usize::from)).collect()).unwrap_or_default()}
    pub fn squad_for_slot(&self,slot:usize)->Option<(i32,usize)>{self.bindings.iter().find(|(_,v)|**v==slot).map(|(key,_)|*key)}
    pub fn set_current_roster(&mut self,team:i32,squad:usize){self.current_roster=self.group_slot(team,squad);}
    pub fn observe_unit_context(&mut self,unit_slot:usize){if let Some(m)=self.members.get(unit_slot).and_then(Option::as_ref).filter(|m|m.role!=0){self.current_roster=Some(usize::from(m.group));}}
    pub fn record_contact_target(&mut self,unit_slot:usize,object_slot:u8){if unit_slot<240{self.contact_targets[unit_slot]=Some(object_slot);}}
    pub fn record_contact_claim(&mut self,unit_slot:usize,object_slot:u8){if unit_slot<240{self.contact_targets[unit_slot]=Some(object_slot);self.contact_claims[unit_slot]=object_slot as i8;}}
    pub fn contact_claim(&self,unit_slot:usize)->i8{self.contact_claims.get(unit_slot).copied().unwrap_or(-1)}
    pub fn clear_contact_claim(&mut self,unit_slot:usize){if unit_slot<240{self.contact_claims[unit_slot]=-1;}}
    pub fn contact_target(&self,unit_slot:usize)->Option<u8>{self.contact_targets.get(unit_slot).copied().flatten()}
    ///Raw controls are source constructor zeroes until a runtime writer records
    ///them. AA uses the59ms callback index; the protection gate is not guessed
    ///from a browser order name or target distance.
    pub fn record_claim_control(&mut self,unit_slot:usize,flags64:u8,protection:u8,tick:u64){if unit_slot<240{self.claim_controls[unit_slot]=life::ClaimControl{flags64,protection,observed_tick:tick};}}
    pub fn claim_control(&self,unit_slot:usize)->life::ClaimControl{self.claim_controls.get(unit_slot).copied().unwrap_or_default()}
    pub fn record_personal_capture(&mut self,unit_slot:usize,active:bool){if let Some(m)=self.members.get_mut(unit_slot).and_then(Option::as_mut){m.flags8a=(m.flags8a&!4)|if active{4}else{0};}}
    ///Raw transport event writers (456c00/56c40/59720/456f20). The APC
    ///leader retains20 after occupation; Exiting64 also retains it untilrelease.
    pub fn record_transport_inside(&mut self,unit_slot:usize,inside:bool){self.record_transport_flags(unit_slot,if inside{32}else{0},if inside{0}else{32});}
    pub fn record_transport_flags(&mut self,unit_slot:usize,set:u8,clear:u8){if let Some(m)=self.members.get_mut(unit_slot).and_then(Option::as_mut){m.flags8a=(m.flags8a&!clear)|set;}}
    ///31:20 selects the native secondary path. Workshop occupancy is not this
    ///bit; route writers and52500/52810 own the actual transitions.
    pub fn record_secondary_path(&mut self,unit_slot:usize,active:bool){if let Some(m)=self.members.get_mut(unit_slot).and_then(Option::as_mut){m.secondary_path=active;m.flags31=(m.flags31&!32)|if active{32}else{0};}}
    pub fn record_physical_contact(&mut self,unit_slot:usize,active:bool){if let Some(m)=self.members.get_mut(unit_slot).and_then(Option::as_mut){m.flags31=(m.flags31&!64)|if active{64}else{0};}}
    ///45f070 stores command/goal/aux when the actor has a nonzero native role.
    pub fn record_group_order(&mut self,team:i32,squad:usize,command:i8,goal:[i16;2],aux:i16){
        if let Some(slot)=self.group_slot(team,squad){let r=&mut self.pool[slot].roster;r[58]=command as u8;r[44..46].copy_from_slice(&goal[0].to_le_bytes());r[46..48].copy_from_slice(&goal[1].to_le_bytes());r[48..50].copy_from_slice(&aux.to_le_bytes());}
    }
    fn bind(&mut self,team:i32,squad:usize,slot:usize){self.bindings.retain(|_,v|*v!=slot);self.bindings.insert((team,squad),slot);}
    pub fn set_raw_promotion(&mut self,slot:usize,promotion:life::PromotionState){if let Some(member)=self.members.get_mut(slot).and_then(Option::as_mut){member.promotion=promotion;}}
    ///52200 writer AFTER its movement/contact gates; actual raw flags71/64
    ///are caller inputs. Current global roster context is intentionally used.
    pub fn record_walking_formation(&mut self,unit_slot:usize,flags71:u8,flags64:u8,terrain:u8){
        if self.role(unit_slot)!=Some(-1){return;}
        if let Some(slot)=self.current_roster.filter(|&s|s<240){life::walking_formation(&mut self.pool[slot],flags71,flags64,terrain);}
    }
    ///4f1e0 after its valid-position gate, with its exact terrain byte.
    pub fn record_terrain_formation(&mut self,unit_slot:usize,terrain:u8){
        let Some(role)=self.role(unit_slot)else{return;};
        if let Some(slot)=self.current_roster.filter(|&s|s<240){life::terrain_formation(&mut self.pool[slot],role,terrain);}
    }
    ///3e190 outcome writer uses the member's own roster, unlike52200/4f1e0.
    pub fn record_secondary_formation(&mut self,unit_slot:usize,created:bool){
        if let Some(member)=self.members.get(unit_slot).and_then(Option::as_ref){let group=usize::from(member.group);if group<240{life::secondary_formation(&mut self.pool[group],member.role,created);}}
    }
    ///525a0 once a new segment is accepted. The native role/flags64 gates are
    ///retained; the global current roster can differ from the member's own one.
    pub fn record_segment_direction(&mut self,unit_slot:usize,direction:u8,flags64:u8){
        if self.role(unit_slot).is_none_or(|role|role>=0)||flags64&0x40!=0{return;}
        let Some(group)=self.current_roster.filter(|&s|s<240)else{return;};
        let mut members=self.members.iter().flatten().cloned().collect::<Vec<_>>();life::set_segment_direction(&mut self.pool[group],&mut members,direction);
        for member in members{let slot=usize::from(member.slot);self.members[slot]=Some(member);}
        if self.pool[group].roster[0x3f]==0{self.pool[group].roster[9]|=1;}
    }
    ///40c180/340 creates under584963=1: temporaryFF, authored saved layout.
    ///This may be called before or after maintain registers the new group.
    pub fn record_produced_group(&mut self,team:i32,squad:usize,saved_layout:u8){
        if let Some(slot)=self.group_slot(team,squad){self.pool[slot].roster[8]=255;self.pool[slot].roster[0x3c]=saved_layout;}
        else{self.produced_formations.insert((team,squad),saved_layout);}
    }
    ///A proven1b5d0 event can precede the browser's deferred registration.
    ///Apply its48 descriptor once on that new binding, never on later reform.
    pub fn record_constructed_descriptor(&mut self,team:i32,squad:usize,descriptor:u8){
        if let Some(slot)=self.group_slot(team,squad){self.pool[slot].roster[65]=descriptor;}
        else{self.constructed_descriptors.insert((team,squad),descriptor);}
    }
    ///17610 runs under4a69f3=1, bypassing1b5d0 formation initialization.
    pub fn record_direct_placement_group(&mut self,team:i32,squad:usize){self.direct_placements.insert((team,squad));}
    pub fn store_task_groups(&mut self,team:i32,groups:&[task::Group],sectors:&[task::Sector]){
        let pointer=|id:Option<usize>|id.and_then(|id|sectors.iter().find(|s|s.id==id)).map(|s|0x5c3f0cu32+s.native_slot as u32*30).unwrap_or(u32::MAX);
        for g in groups{let Some(slot)=self.group_slot(team,g.id)else{continue;};let p=&mut self.pool[slot].strategy;
            for(offset,id)in[(0,g.current_sector),(4,g.via_sector),(12,g.assigned_sector)]{p[offset..offset+4].copy_from_slice(&pointer(id).to_le_bytes());}
            p[8..12].copy_from_slice(&g.value.to_le_bytes());p[16]=g.flags;p[17]=g.task as u8;p[21]=u8::from(g.active);
        }
    }
}

fn native_position(u:&Unit)->[i16;2]{[(u.x/2.).floor()as i16,(u.y/2.).floor()as i16]}
fn sector_for(position:[i16;2],sectors:&[task::Sector])->Option<usize>{let x=position[0]>>4;let y=position[1]>>4;for(dx,dy)in[(0,0),(1,1)]{if let Some(s)=sectors.iter().find(|s|s.valid&&x+dx>=s.rect[0]&&x+dx<s.rect[2]&&y+dy>=s.rect[1]&&y+dy<s.rect[3]){return Some(s.id);}}None}
fn sector_pointer(position:[i16;2],sectors:&[task::Sector])->Option<u32>{let id=sector_for(position,sectors)?;sectors.iter().find(|s|s.id==id).map(|s|0x5c3f0cu32+s.native_slot as u32*30)}
fn decode_sector(bytes:&[u8],sectors:&[task::Sector])->Option<usize>{let pointer=u32::from_le_bytes(bytes.try_into().ok()?);sectors.iter().find(|s|0x5c3f0cu32+s.native_slot as u32*30==pointer).map(|s|s.id)}
fn member_from_unit(u:&Unit,old:Option<&life::Member>)->life::Member{
    let mut m=old.cloned().unwrap_or(life::Member{slot:u.native_slot.unwrap_or(0)as u8,group:255,role:0,flags31:0,flags69:if native_position(u)[0]!=10000{4}else{0},flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:native_position(u),reserved_object:None,promotion:life::PromotionState::default()});
    m.position=native_position(u);m.flags31=(m.flags31&!0x27)|(u.team as u8&3)|4|if u.native_secondary_route.get(){32}else{0};
    //Personal capture8a:4 is written by its strategic source event. Automatic
    //57860 flag contact uses the same UI marker without writing this bit.
    //8a10/18/20 are event state, including occupied APCleader andExiting64.
    m.flags9c=u.native_flags9c;if let Some(e)=&u.native_factory_exit{m.flags9c|=e.flags9c;}
    m.primary_path=!u.path.is_empty()&&m.flags31&32==0;m.secondary_path=!u.path.is_empty()&&m.flags31&32!=0;m.promotion.phase=u.native_phase35;m.promotion.order=u.native_order5c as u8;m
}

impl State{
    fn remove_member(&mut self,slot:usize,g:&mut Game){
        let Some(mut old)=self.members[slot].take()else{return;};let source=usize::from(old.group);
        if source>=240{return;}
        let role=if old.role==-1{0}else if old.role>0{old.role as usize}else{return;};
        if role>20||self.pool[source].member(role)!=Some(slot as u8){return;}
        if old.flags31&8!=0{if let Some(object)=old.reserved_object.and_then(|slot|u8::try_from(slot).ok()){g.release_native_object(object);}}
        for group in self.pool.iter_mut(){if group.roster[59]==slot as u8{group.roster[59]=255;}}
        let source_group=&mut self.pool[source];source_group.roster[19+role]=255;source_group.roster[18]=source_group.roster[18].wrapping_sub(1);
        old.role=0;old.group=255;old.flags31&=!8;old.flags69&=!0x80;
        if source_group.roster[18]==0{
            source_group.cooldown=200;source_group.roster[9]&=!1;
            source_group.roster[40..42].copy_from_slice(&old.position[0].to_le_bytes());source_group.roster[42..44].copy_from_slice(&old.position[1].to_le_bytes());
        }else if role>0{source_group.roster[9]|=0x40;}
        else{
            source_group.roster[9]&=!1;
            let inherited=self.before_removal[slot].take().or_else(||self.snapshots[slot].clone()).or_else(||self.entities[slot].and_then(|id|g.units.iter().find(|u|u.id==id)).map(OrderSnapshot::of));
            let mut members=self.members.iter().flatten().cloned().collect::<Vec<_>>();members.push(old);
            //Only membership/field transfer is consumed here. Web VecDeque
            //ownership is explicit; it is never encoded as a fake pool pointer.
            let mut raw_paths=[0u32;240];
            let promoted=life::promote_first_follower(source_group,&mut members,slot as u8,&mut raw_paths,||g.original_rng.next_u32());
            for member in members{if usize::from(member.slot)!=slot{let id=usize::from(member.slot);self.members[id]=Some(member);}}
            if let Some(p)=promoted{
                if let Some(index)=g.units.iter().position(|u|u.native_slot==Some(usize::from(p.leader))&&original_combat::alive(true,u.hp)){
                    let unit=&mut g.units[index];unit.formation=None;
                    if let Some(member)=self.members[usize::from(p.leader)].as_ref(){unit.native_phase35=member.promotion.phase;}
                    if let Some(order)=inherited{unit.order=order.order;unit.target=order.target;unit.dest=order.dest;unit.path=order.path;unit.march_resume=order.march_resume;unit.original_motion=None;unit.moving=!unit.path.is_empty();}
                }
            }
        }
        self.entities[slot]=None;self.snapshots[slot]=None;self.before_removal[slot]=None;self.primary_orders[slot]=None;
    }
    fn refresh_formations(&self,g:&mut Game){
        for(group_slot,group)in self.pool.iter().enumerate(){
            let Some(leader_slot)=group.member(0)else{continue;};
            let Some(leader)=g.units.iter().find(|u|u.native_slot==Some(usize::from(leader_slot))&&original_combat::alive(true,u.hp)).cloned()else{continue;};
            let Some((team,squad))=self.squad_for_slot(group_slot)else{continue;};
            for role in 1..21{let Some(member_slot)=group.member(role)else{continue;};
                let Some(index)=g.units.iter().position(|u|u.native_slot==Some(usize::from(member_slot))&&u.team==team&&u.squad==squad&&original_combat::alive(true,u.hp))else{continue;};
                if stats(&g.units[index].kind).cls!=0{g.units[index].formation=None;continue;}
                let blocked={let u=&g.units[index];u.explicit_stop||u.order!="move"||u.hold_position||u.initiative.is_some()||u.workshop.is_some()||u.native_contact.is_some()||u.native_transport.is_some()||u.transport_carrier.is_some()||u.native_factory_exit.as_ref().is_some_and(|e|e.blocks_formation(!u.path.is_empty()))};
                if blocked{
                    //Membership changes still update the existing ROLE and
                    //leader reference, without creating/replanning an order.
                    if let Some(f)=g.units[index].formation.as_mut(){f.slot=role;f.leader=leader.id;}
                    continue;
                }
                let current=g.units[index].formation.clone();
                let layout=group.roster[8];
                if current.as_ref().is_some_and(|f|f.leader==leader.id&&f.slot==role&&f.layout==layout){continue;}
                let goal=current.as_ref().map(|f|f.command_goal).or(leader.dest).unwrap_or((leader.x,leader.y));
                let direction=current.as_ref().map(|f|f.direction).unwrap_or_else(||original_combat::facing(leader.angle));
                g.units[index].formation=Some(super::original_formation::Follow::new(leader.id,role,layout,goal,direction,g.map.revision));
            }
        }
    }
    pub fn set_group_angle(&mut self,team:i32,squad:usize,angle:i16){if let Some(slot)=self.group_slot(team,squad){self.pool[slot].roster[10..12].copy_from_slice(&angle.to_le_bytes());}}
    fn advance_cooldowns(&mut self,tick:u64){if self.last_cooldown_tick!=Some(tick){life::advance_cooldowns(&mut self.pool);self.last_cooldown_tick=Some(tick);}}
    fn advance_counters(&mut self,g:&mut Game){
        if self.last_tick==Some(g.motion_tick){return;}
        //Unit counters are the current Game mirror of roster10/3d. Read their
        //pre-update values; original_groups updates that mirror once separately.
        for(slot,group)in self.pool.iter_mut().enumerate(){
            if let Some((team,squad))=self.bindings.iter().find_map(|(key,&value)|(value==slot).then_some(*key)){
                let units=g.units.iter().filter(|u|u.team==team&&u.squad==squad);
                let(ttl,bonus)=units.fold((0u8,0u8),|(a,b),u|(a.max(u.group_activity_ticks),b.max(u.group_bonus_ticks)));
                group.roster[16]=ttl;group.roster[0x3d]=bonus;
            }
        }
        let mut members=self.members.iter().flatten().cloned().collect::<Vec<_>>();life::advance_group_counters(&mut self.pool,&mut members);
        for member in members{let slot=usize::from(member.slot);self.members[slot]=Some(member);}
        self.last_tick=Some(g.motion_tick);self.refresh_formations(g);
    }
}

impl Game{
    ///50cd0/50db0:4bba0→4b2f0→4b140 for the SAME native UNIT slot.
    ///This is distinct from4c0b0's last-follower split and from APC conversion,
    ///which retains its group. The caller owns descriptor/entry motion writes.
    pub(crate) fn reform_native_hardware_group(&mut self,index:usize)->Option<usize>{
        let unit=self.units.get(index)?.clone();let slot=unit.native_slot.filter(|&s|s<240)?;let mut state=std::mem::take(&mut self.native_group_rosters);
        let result=(||{let mut member=member_from_unit(&unit,state.members[slot].as_ref());state.snapshots[slot]=Some(state.primary_snapshot(slot,&unit));state.remove_member(slot,self);member.role=0;member.group=255;member.flags31&=!8;member.contact_order=0;member.flags69&=!128;
            let sector=self.native_strategy.as_ref().and_then(|s|s.sector_pointer(unit.x,unit.y));let target=life::allocate_single(&mut state.pool,&mut member,0,sector,false)?;let squad=self.units.iter().map(|u|u.squad).max().unwrap_or(0)+1;self.units[index].squad=squad;self.units[index].formation=None;state.bind(unit.team,squad,target);state.entities[slot]=Some(unit.id);state.snapshots[slot]=Some(OrderSnapshot::of(&self.units[index]));state.members[slot]=Some(member);state.current_roster=Some(target);Some(target)
        })();state.refresh_formations(self);self.native_group_rosters=state;result
    }
    ///Preserve a caller-owned primary move before a compatibility detour. The
    ///native578b0 adapters already record this at the actual claim event.
    pub(crate) fn remember_native_primary_order(&mut self,index:usize){if let Some(unit)=self.units.get(index).cloned(){if let Some(slot)=unit.native_slot.filter(|&s|s<240){self.native_group_rosters.remember_primary(slot,&unit);}}}
    ///4b8b0 precedes5a240's unit loop, which precedes19fb0.
    pub(crate) fn advance_original_group_cooldowns(&mut self){
        if !self.map.original{return;}
        self.native_group_rosters.advance_cooldowns(self.motion_tick);
    }
    ///Called before19fb0. It preserves already assigned native roles, removes
    ///dead members, and registers actual new members in the first free slot.
    pub(crate) fn maintain_native_group_rosters(&mut self){
        if !self.map.original{return;}
        let mut state=std::mem::take(&mut self.native_group_rosters);
        state.advance_cooldowns(self.motion_tick);
        let mut live=BTreeMap::new();
        for(index,u)in self.units.iter().enumerate().filter(|(_,u)|(0..=1).contains(&u.team)&&original_combat::alive(true,u.hp)){
            if let Some(slot)=u.native_slot.filter(|&slot|slot<240){live.entry(slot).or_insert(index);}
        }
        let missing=(0..240).filter(|&slot|state.members[slot].is_some()&&!live.contains_key(&slot)).collect::<Vec<_>>();
        for slot in missing{state.remove_member(slot,self);state.contact_claims[slot]=-1;state.contact_targets[slot]=None;}
        for(slot,index)in live{
            let unit=self.units[index].clone();
            if let Some(object)=unit.transport_carrier.or_else(||matches!(unit.order.as_str(),"enter"|"board").then_some(unit.target).flatten()).and_then(|id|self.native_object_slot(id)){state.record_contact_target(slot,object);}
            let sector=self.native_strategy.as_ref().and_then(|s|s.sector_pointer(unit.x,unit.y));
            let previous=state.members[slot].clone();
            let changed_entity=state.entities[slot].is_some_and(|id|id!=unit.id);
            let same_group=previous.as_ref().and_then(|m|state.squad_for_slot(usize::from(m.group))).is_some_and(|key|key==(unit.team,unit.squad));
            if previous.is_some()&&(changed_entity&&unit.kind!="apc"||!same_group){
                //APC changes the Web entity backing the SAME native member.
                //Other occupied hardware executes native detach/reallocate.
                if changed_entity&&unit.kind=="apc"{let group=usize::from(previous.as_ref().unwrap().group);state.bind(unit.team,unit.squad,group);}
                else{state.remove_member(slot,self);}
            }
            let old=state.members[slot].as_ref();let mut member=member_from_unit(&unit,old);member.reserved_object=state.contact_target(slot).map(usize::from);
            if old.is_none(){
                let target=state.group_slot(unit.team,unit.squad).filter(|&i|state.pool[i].roster[18]!=0);
                let allocated=if let Some(target)=target{life::append_member(&mut state.pool[target],&mut member,target,0).then_some(target)}else{life::allocate_single(&mut state.pool,&mut member,0,sector,false)};
                let Some(target)=allocated else{continue;};state.bind(unit.team,unit.squad,target);
                //Authored and produced first members run1b5d0. Rebinding an
                //occupied non-APC vehicle only runs4b2f0 and preserves stale
                //mask fields.4c0b0's split adapter likewise never calls this.
                if previous.is_none()&&state.pool[target].roster[18]==1&&!state.direct_placements.remove(&(unit.team,unit.squad)){
                    let produced=state.produced_formations.remove(&(unit.team,unit.squad));
                    state.pool[target].roster[8]=if produced.is_some(){255}else{unit.formation_layout};state.pool[target].roster[0x3c]=produced.unwrap_or(unit.formation_layout);
                }
                if let Some(descriptor)=state.constructed_descriptors.remove(&(unit.team,unit.squad)){state.pool[target].roster[65]=descriptor;}
            }
            state.entities[slot]=Some(unit.id);state.snapshots[slot]=Some(state.primary_snapshot(slot,&unit));state.members[slot]=Some(member);
        }
        //4b280 follows production and sees its newly allocated rosters, unlike
        //4b8b0's independent cooldown pass at the beginning of the callback.
        state.advance_counters(self);state.refresh_formations(self);self.native_group_rosters=state;
    }

    ///Death handlers can invoke this BEFORE finish_order clears the leader's
    ///Web route. maintain_native_group_rosters remains the reconciliation pass.
    pub(crate) fn remove_original_group_member(&mut self,index:usize){
        let Some(slot)=self.units.get(index).and_then(|u|u.native_slot).filter(|&slot|slot<240)else{return;};
        let mut state=std::mem::take(&mut self.native_group_rosters);
        state.snapshots[slot]=Some(state.primary_snapshot(slot,&self.units[index]));state.remove_member(slot,self);state.refresh_formations(self);self.native_group_rosters=state;
    }
    ///Capture before a crew-kill/damage handler clears the original order.
    ///Removal/promotion consumes this snapshot before later empty-path mirrors.
    pub(crate) fn remember_original_group_order(&mut self,index:usize,previous:&Unit){
        let slot=previous.native_slot.or_else(||self.units.get(index).and_then(|u|u.native_slot));
        if let Some(slot)=slot.filter(|&slot|slot<240){self.native_group_rosters.before_removal[slot]=Some(self.native_group_rosters.primary_snapshot(slot,previous));}
    }
    ///The initial578b0 reservation event, before50890/B5 portal claims. Existing
    ///reservation/secondary-path/transport/protection gates remain native.
    pub(crate) fn claim_native_hardware_order(&mut self,index:usize,target_id:usize)->Option<life::HardwareOrder>{
        let unit=self.units.get(index)?.clone();let slot=unit.native_slot.filter(|&s|s<240)?;
        if self.native_group_rosters.members[slot].is_none(){self.maintain_native_group_rosters();}
        let object=self.native_object_slot(target_id)?;let world=self.native_ai_world_objects()?.into_iter().find(|o|o.id==usize::from(object))?;
        let kind=u16::try_from(world.definition?).ok()?;let(sprite,_,_)=super::original_world_contact::object_metadata(kind)?;
        let mut member=member_from_unit(&unit,self.native_group_rosters.members[slot].as_ref());let group=usize::from(member.group);if group>=240{return None;}
        let apc_allowed=self.native_group_apc_target(index,object)==Some(object);
        let order=life::claim_hardware(&mut member,unit.native_flags64,unit.native_protectionaa,object,sprite as i16,world.position,world.flags13,apc_allowed)?;
        if !self.reserve_native_object(object){return None;}
        self.native_group_rosters.remember_primary(slot,&unit);
        self.native_group_rosters.record_contact_target(slot,object);self.native_group_rosters.members[slot]=Some(member);self.units[index].motion_bonus=order.motion_bonus as i8;
        Some(order)
    }
    ///57790's EDX is the candidate OBJECT slot, not an all-object search. A
    ///successful41cbd0 flag fallback is represented bySome(255).
    pub(crate) fn native_group_apc_target(&self,index:usize,candidate:u8)->Option<u8>{
        let unit=self.units.get(index)?;let slot=unit.native_slot?;let member=self.native_group_rosters.members.get(slot)?.as_ref()?;if member.role!=-1{return None;}
        let group=self.native_group_rosters.current_roster.and_then(|s|self.native_group_rosters.pool.get(s))?;if group.roster[58]!=1{return None;}let goal=[group.word(44),group.word(46)];
        if let Some(o)=self.native_world_scan_objects().into_iter().find(|o|o.slot==candidate){
            let kind=o.definition;let extent=o.extent20;
                    //499738's hardware/pickup admission table; all ordinary
                    //hardware is enabled, includingAPC type31.
                    let enabled=matches!(kind,5|7|8|9|10|21|22|23|24|26|29|31|33|35|37|58);
                    //41c8b0 deliberately uses metadata+20 for BOTH axes;
                    //the runtime override is significant for this square test.
                    if enabled&&o.flags13&0x50!=0&&(0..2).all(|a|o.position[a]<=goal[a]&&i32::from(goal[a])-i32::from(extent)<=i32::from(o.position[a])){return Some(candidate);}
        }
        //41d280 populates5a1d5e from signedCPUPLR flag bytes<<4, excluding the
        //two fort sectors;41cbd0 tests(flagX+4,flagY+8) within goal±8.
        self.sectors.iter().filter(|s|s.counts_for_production&&!s.home).any(|s|{
            let flag=[i16::from(((s.x/32.)-0.5)as i8).wrapping_mul(16).wrapping_add(4),i16::from(((s.y/32.)-0.5)as i8).wrapping_mul(16).wrapping_add(8)];
            (0..2).all(|a|goal[a].wrapping_sub(8)<=flag[a]&&flag[a]<=goal[a].wrapping_add(8))
        }).then_some(255)
    }
    ///57860 flag claim; OBJECT+1f is an explicit runtime input from the world
    ///flag-state adapter. The output is a direct525a0 anchor, unlike hardware.
    pub(crate) fn claim_native_flag_order(&mut self,index:usize,object:u8,flag_delay1f:u8)->Option<life::FlagOrder>{
        let unit=self.units.get(index)?.clone();let slot=unit.native_slot.filter(|&s|s<240)?;if self.native_group_rosters.members[slot].is_none(){self.maintain_native_group_rosters();}
        let world=self.native_ai_world_objects()?.into_iter().find(|o|o.id==usize::from(object)&&o.definition==Some(28))?;
        let mut member=member_from_unit(&unit,self.native_group_rosters.members[slot].as_ref());let order=life::claim_flag(&mut member,unit.native_flags64,unit.native_protectionaa,object,world.position,world.flags13,flag_delay1f)?;
        if !self.reserve_native_object(object){return None;}self.native_group_rosters.remember_primary(slot,&unit);self.native_group_rosters.record_contact_target(slot,object);self.native_group_rosters.members[slot]=Some(member);self.units[index].motion_bonus=order.motion_bonus as i8;Some(order)
    }
    ///4bba0/53730 clear31:8 and contact5d, releasing OBJECT36 without clearing
    ///the saved36 target or persistentB5 successful/start-contact claim.
    pub(crate) fn release_original_claim(&mut self,index:usize)->bool{
        let Some(slot)=self.units.get(index).and_then(|u|u.native_slot).filter(|&s|s<240)else{return false;};
        let Some(member)=self.native_group_rosters.members[slot].as_mut()else{return false;};if member.flags31&8==0{return false;}
        member.flags31&=!8;member.contact_order=0;let object=member.reserved_object.and_then(|o|u8::try_from(o).ok());if let Some(object)=object{self.release_native_object(object);}true
    }

    ///Return the exact new roster/task group and selected native unit slot;
    ///the15000 caller issues its hardware order and commits task5 afterwards.
    pub(crate) fn split_original_hardware_member(&mut self,team:i32,request:&task::HardwareCapture,groups:&mut Vec<task::Group>,sectors:&[task::Sector])->Option<(task::Group,usize)>{
        let mut state=std::mem::take(&mut self.native_group_rosters);
        state.store_task_groups(team,groups,sectors);
        let result=(||{
            let source=state.group_slot(team,request.group)?;
            let selected=if state.pool[source].roster[18]as i8>1{(1..21).rev().find_map(|r|state.pool[source].member(r)).map(usize::from)}else{state.pool[source].member(0).map(usize::from)}?;
            let point=state.members[selected].as_ref()?.position;let pointer=sector_pointer(point,sectors);
            let mut members=state.members.iter().flatten().cloned().collect::<Vec<_>>();
            let split=life::split_last_follower(&mut state.pool,&mut members,source,pointer,false);
            if let life::Split::Detached{release_reservation:Some(object),..}=&split{if let Ok(object)=u8::try_from(*object){self.release_native_object(object);}}
            let(target,member,detached)=match split{
                life::Split::NoFollower=>(source,state.pool[source].member(0).map(usize::from)?,false),
                life::Split::Detached{new_group:Some(target),member,..}=>(target,usize::from(member),true),
                life::Split::Detached{new_group:None,..}=>{for m in members{let slot=usize::from(m.slot);state.members[slot]=Some(m);}return None;},
                life::Split::InvalidRoster=>return None,
            };
            for m in members{let slot=usize::from(m.slot);state.members[slot]=Some(m);}
            let index=self.units.iter().position(|u|u.native_slot==Some(member)&&u.team==team&&original_combat::alive(true,u.hp))?;
            let squad=if detached{self.units.iter().map(|u|u.squad).max().unwrap_or(0)+1}else{request.group};
            if detached{
                let u=&mut self.units[index];u.squad=squad;u.formation=None;u.path.clear();u.original_motion=None;u.moving=false;
                state.bind(team,squad,target);state.snapshots[member]=Some(OrderSnapshot::of(u));
            }
            state.current_roster=Some(target);state.refresh_formations(self);
            let u=&self.units[index];let p=&state.pool[target].strategy;
            let grenade_members=state.pool[target].roster[19..40].iter().filter_map(|&slot|(slot!=255).then_some(slot)).filter_map(|slot|self.units.iter().find(|v|v.native_slot==Some(usize::from(slot))).map(|v|v.grenades as i8)).collect::<Vec<_>>();
            let leader=task::Leader{native_slot:member,team:1,runtime_class:u.native_class7c,family:original_combat::animation_family(&u.kind),passenger_class:u.crew.as_ref().map(|c|original_combat::runtime_class(&c.kind)).unwrap_or(0),position:native_position(u),accepts_orders:u.native_flags9c&0x14==0&&u.native_protectionaa==0,laser_can_cross:super::native_ai_scoring::group_can_demolish(&grenade_members)};
            Some((task::Group{id:squad,active:p[21]!=0,flags:p[16],task:p[17]as i8,current_sector:decode_sector(&p[..4],sectors).or_else(||sector_for(native_position(u),sectors)),via_sector:decode_sector(&p[4..8],sectors),assigned_sector:decode_sector(&p[12..16],sectors),value:i32::from_le_bytes(p[8..12].try_into().unwrap()),leader:Some(leader)},member))
        })();
        self.native_group_rosters=state;result
    }

    ///158f0's immediate4e910 follow-up. Normal followers get their preserved
    ///native ROLE offset; they do not resume an obsolete independent march.
    pub(crate) fn rejoin_original_member(&mut self,unit_slot:usize){
        if unit_slot>=240{return;}
        let Some(index)=self.units.iter().position(|u|u.native_slot==Some(unit_slot)&&original_combat::alive(true,u.hp))else{return;};
        //158f0 clears8a&4 before4e910's independent transport/contact gate.
        if matches!(self.units[index].initiative,Some(super::original_initiative::Initiative::Flag(_))){self.units[index].initiative=None;self.units[index].march_resume=None;}
        let mut state=std::mem::take(&mut self.native_group_rosters);
        let result=(||{
            let mut member=member_from_unit(&self.units[index],state.members[unit_slot].as_ref());member.reserved_object=state.contact_target(unit_slot).map(usize::from);member.flags8a&=!4;
            state.members[unit_slot]=Some(member.clone());
            let own=usize::from(member.group);if own>=240{return None;}
            let context=state.current_roster?;if context>=240{return None;}
            let bits=state.pool[context].roster[19..40].iter().filter(|&&slot|slot!=255).filter_map(|&slot|state.members[usize::from(slot)].as_ref()).map(|m|m.flags8a&0x20!=0).collect::<Vec<_>>();
            let uniform=bits.iter().all(|&v|v)||bits.iter().all(|&v|!v);
            let leader_slot=state.pool[own].member(0)?;
            let leader=self.units.iter().find(|u|u.native_slot==Some(usize::from(leader_slot))&&original_combat::alive(true,u.hp))?.clone();
            let role=member.role;let layout=state.pool[own].roster[8];
            let offset=super::original_formation::raw_offset(role.max(1)as usize,layout);
            let rejoin=life::rejoin(&mut member,&mut state.pool[own],native_position(&leader),uniform,false,offset,native_math::sine);
            state.current_roster=Some(own);state.members[unit_slot]=Some(member);
            Some((rejoin,leader,role,layout))
        })();
        self.native_group_rosters=state;
        let Some((rejoin,leader,role,layout))=result else{return;};
        if rejoin.cleared_path{let u=&mut self.units[index];u.path.clear();u.original_motion=None;u.initiative=None;u.march_resume=None;u.moving=false;}
        if rejoin.special_arrival{
            self.native_group_special_arrival(index,unit_slot);return;
        }
        if let Some(target)=rejoin.target{
            //4e910 always emits a59c60 primary waypoint;52810 subtracts8.
            let goal=(f64::from(target[0].wrapping_sub(8))*2.,f64::from(target[1].wrapping_sub(8))*2.);let path=self.route_unit(&self.units[index],goal.0,goal.1);
            let current=self.units[index].formation.clone();let command_goal=current.as_ref().map(|f|f.command_goal).or(leader.dest).unwrap_or(goal);let direction=current.as_ref().map(|f|f.direction).unwrap_or_else(||original_combat::facing(leader.angle));
            let u=&mut self.units[index];u.path=path;u.dest=Some(goal);u.order="move".into();u.target=None;u.explicit_stop=false;u.hold_position=false;
            if role>0{u.formation=Some(super::original_formation::Follow::new(leader.id,role as usize,layout,command_goal,direction,self.map.revision));}
        }
    }
    fn native_group_special_arrival(&mut self,index:usize,unit_slot:usize){
        //50b30 uses OBJECT36 even after occupation removed the neutral render
        //binding. B5 persists independently; failed claims cancel the roster.
        if let Some(object)=self.native_group_rosters.contact_target(unit_slot){
            let group=self.native_group_rosters.members[unit_slot].as_ref().map(|m|m.group).unwrap_or(255);
            let claims=self.native_group_rosters.members.iter().enumerate().filter_map(|(slot,m)|m.as_ref().map(|m|life::ContactClaim{flags31:m.flags31,group:m.group,object:self.native_group_rosters.contact_claims[slot]})).collect::<Vec<_>>();
            let x=self.native_ai_world_objects().and_then(|objects|objects.iter().find(|o|o.id==usize::from(object)).map(|o|o.position[0])).unwrap_or(-1);
            if !life::special_contact_allowed(&claims,group,object,x){self.cancel_native_transport(index,true);return;}
            self.native_group_rosters.record_contact_claim(unit_slot,object);
        }
        self.update_native_transport(index);
    }
    ///4d570 idle preamble. An existing primary path retains its individual
    ///destination; unlike4e910 this function never clears it to force a rejoin.
    ///The global forced-formation byte4d029b has no Game writer yet (false).
    pub(crate) fn follow_native_idle_group(&mut self,index:usize){
        self.follow_native_idle_group_with_force(index,false);
    }
    pub(crate) fn follow_native_idle_group_forced(&mut self,index:usize){
        self.follow_native_idle_group_with_force(index,true);
    }
    fn follow_native_idle_group_with_force(&mut self,index:usize,forced:bool){
        let Some(unit)=self.units.get(index).cloned()else{return;};let Some(slot)=unit.native_slot.filter(|&s|s<240)else{return;};
        let Some(mut member)=self.native_group_rosters.members[slot].clone()else{return;};member=member_from_unit(&unit,Some(&member));
        let Some(context)=self.native_group_rosters.current_roster.filter(|&s|s<240)else{return;};
        let Some(leader_slot)=self.native_group_rosters.pool[context].member(0)else{return;};
        let Some(leader)=self.units.iter().find(|u|u.native_slot==Some(usize::from(leader_slot)))else{return;};
        let layout=self.native_group_rosters.pool[context].roster[8];let offset=super::original_formation::raw_offset(member.role.max(1)as usize,layout);
        let result=life::idle_follow(&mut member,&mut self.native_group_rosters.pool[context],native_position(leader),!unit.path.is_empty()||unit.native_route_pending.get().is_some(),forced,offset,native_math::sine);
        self.native_group_rosters.members[slot]=Some(member);
        if let Some(bonus)=result.motion_bonus{self.units[index].motion_bonus=bonus as i8;}
        if result.special_arrival{self.native_group_special_arrival(index,slot);return;}
        let Some(route)=result.route else{return;};let(target,direct)=match route{life::FollowRoute::Primary(p)=>([p[0].wrapping_sub(8),p[1].wrapping_sub(8)],false),life::FollowRoute::Direct(p)=>(p,true)};
        let goal=(f64::from(target[0])*2.,f64::from(target[1])*2.);let already=native_position(&unit)==target;
        let path=if direct{if already{VecDeque::new()}else{VecDeque::from([goal])}}else{self.route_unit(&unit,goal.0,goal.1)};
        if direct&&!already{if !self.prepare_native_line_occupancy(index,target){return;}self.prepare_native_walk(index);}
        let u=&mut self.units[index];u.path=path;u.dest=Some(goal);u.original_motion=None;u.moving=!u.path.is_empty();
        if direct{
            if !already{u.native_remaining48=super::original_motion::Segment::new(u.x,u.y,goal).remaining();}
            u.native_phase35=7;if let Some(member)=self.native_group_rosters.members[slot].as_mut(){life::finish_idle_follow(member,already,u.native_remaining48);}
        }
    }
    ///5a080's leader prefix and its4b590 expiry callback. This belongs in each
    ///native unit callback, after current-roster selection and before handlers.
    pub(crate) fn advance_original_group_leader(&mut self,index:usize){
        if self.units.get(index).is_none_or(|u|u.team<0||!original_combat::alive(true,u.hp)){return;}
        let Some(slot)=self.units.get(index).and_then(|u|u.native_slot).filter(|&s|s<240)else{return;};
        if self.native_group_rosters.role(slot).is_none_or(|role|role>=0){return;}
        self.units[index].native_counter6e=life::advance_leader_counter(self.units[index].native_counter6e);
        let Some(group)=self.native_group_rosters.current_roster.filter(|&s|s<240)else{return;};
        if life::begin_leader_preamble(&mut self.native_group_rosters.pool[group]){
            //4b590 changes only these flags for the duration of its follower
            //loop. Its decision tree is the same verified4d570 route preamble.
            let flags=self.native_group_rosters.pool[group].roster[9];self.native_group_rosters.pool[group].roster[9]=(flags&!0x10)|1;
            for role in 1..21{if let Some(member)=self.native_group_rosters.pool[group].member(role){if let Some(index)=self.units.iter().position(|u|u.native_slot==Some(usize::from(member))){self.follow_native_idle_group(index);}}}
            self.native_group_rosters.pool[group].roster[9]=flags;
        }
        if let Some(current)=self.native_group_rosters.current_roster.filter(|&s|s<240){life::finish_leader_preamble(&mut self.native_group_rosters.pool[current],self.units[index].native_phase35);}
    }
    ///4e9b0, before4ea30's territory lookup/ownership gates. Every live same-
    ///team native slot in this sector loses8a:4, in slot order. Whether rejoin
    ///is called depends on the triggering actor's class7c, not each follower.
    pub(crate) fn native_group_flag_cleanup(&mut self,index:usize){
        let Some(actor)=self.units.get(index)else{return;};let team=actor.team;let class=original_combat::runtime_class(&actor.kind);
        let Some(sector)=self.native_strategy.as_ref().and_then(|s|s.sector_at(actor.x,actor.y))else{return;};
        for slot in 0..240{
            let Some(other)=self.units.iter().find(|u|u.native_slot==Some(slot))else{continue;};
            let target=self.native_strategy.as_ref().and_then(|s|s.sector_at(other.x,other.y));
            let Some(member)=self.native_group_rosters.members[slot].as_mut()else{continue;};
            if !life::clear_personal_capture_in_sector(member,team as u8,Some(sector),target){continue;}
            if class!=2{self.rejoin_original_member(slot);}
        }
    }
}

#[cfg(test)]mod tests{
    use super::*;
    fn field()->Game{
        let mut g=Game::new(0,1,771);g.map.original=true;g.map.tiles.fill(0);g.map.terrain_codes=vec![0;g.map.tiles.len()];g.map.bridge_base.fill(-1);
        g.units.clear();g.buildings.clear();g.crates.clear();g.objects.clear();g.walls.clear();g.effects.clear();g.ai_teams.clear();g.native_strategy=None;g.native_group_rosters=State::default();
        for i in 0..3{g.spawn("grunt",1,200.+i as f64*32.,200.,101,None);}
        for u in &mut g.units{u.order="move".into();}
        g.maintain_native_group_rosters();g
    }
    #[test]fn mission_reset_matches_native_roster_clear_and_preserves_every_unwritten_strategy_byte(){
        let hash=|values:Vec<u32>|values.into_iter().fold(2166136261u32,|h,n|(h^n).wrapping_mul(16777619));let mut values=Vec::new();
        for scenario in 0..16{let mut state=State::default();for(slot,g)in state.pool.iter_mut().enumerate(){for(j,b)in g.roster.iter_mut().enumerate(){*b=(scenario*11+(slot*72+j)*7)as u8;}for(j,b)in g.strategy.iter_mut().enumerate(){*b=(scenario*19+(slot*24+j)*3)as u8;}g.cooldown=117;}
            state.clear_for_mission_preserving_strategy_values();let roster=hash(state.pool.iter().flat_map(|g|g.roster.iter().map(|&b|u32::from(b))).collect());let strategy=hash(state.pool.iter().flat_map(|g|g.strategy.iter().map(|&b|u32::from(b))).collect());values.extend([scenario as u32,roster,strategy]);assert!(state.pool.iter().all(|g|g.cooldown==0));
        }
        assert_eq!(hash(values),1049528069,"ZNativeGroupMissionResetOracle real4b530 +460430 strategy loop over16 seeded whole pools");
    }
    #[test]fn idle_group_follow_keeps_an_unfinished_primary_route_and_uses_phase7_at_the_direct_target(){
        let mut g=field();let slot=g.units[1].native_slot.unwrap();let group=g.native_group_rosters.group_slot(1,101).unwrap();g.native_group_rosters.current_roster=Some(group);g.native_group_rosters.pool[group].roster[9]|=1;g.native_group_rosters.pool[group].roster[0x3f]=0;
        let original=VecDeque::from([(400.,300.),(900.,700.)]);g.units[1].path=original.clone();g.units[1].dest=Some((900.,700.));g.units[1].motion_bonus=1;g.units[1].native_phase35=3;g.follow_native_idle_group(1);assert_eq!(g.units[1].path,original);assert_eq!(g.units[1].dest,Some((900.,700.)));assert_eq!(g.units[1].motion_bonus,1);assert_eq!(g.units[1].native_phase35,3);
        g.units[1].path.clear();g.native_group_rosters.set_group_angle(1,101,0);let delta=super::super::original_formation::offset_layout(1,0,0);let direct=(g.units[0].x+delta.0,g.units[0].y+delta.1);g.units[1].x=direct.0;g.units[1].y=direct.1;g.units[1].native_remaining48=5;
        g.follow_native_idle_group(1);assert_eq!(g.units[1].dest,Some(direct));assert!(g.units[1].path.is_empty());assert_eq!(g.units[1].native_phase35,7);assert_ne!(g.native_group_rosters.members[slot].as_ref().unwrap().flags31&0x80,0);
    }
    #[test]fn leader_creation_counter_releases_follower_routes_on_the_fourth_callback_and_restores_group_flags(){
        let mut g=field();let group=g.native_group_rosters.group_slot(1,101).unwrap();g.native_group_rosters.current_roster=Some(group);g.native_group_rosters.pool[group].roster[9]=0x56;g.native_group_rosters.pool[group].roster[0x40]=7;g.native_group_rosters.set_group_angle(1,101,0);g.units[1].x=208.;
        for expected in [3,2,1]{g.advance_original_group_leader(0);assert_eq!(g.native_group_rosters.pool[group].roster[0x3f],expected);assert!(g.units[1].path.is_empty());}
        g.advance_original_group_leader(0);assert_eq!(g.native_group_rosters.pool[group].roster[0x3f],0);assert_eq!(g.native_group_rosters.pool[group].roster[9],0x56);assert_eq!(g.native_group_rosters.pool[group].roster[0x40],3);assert!(!g.units[1].path.is_empty());assert_eq!(g.units[1].native_phase35,7);
    }
    #[test]fn automatic_flag_ui_marker_does_not_create_or_restore_the_personal_capture_bit(){
        let mut g=field();let slot=g.units[1].native_slot.unwrap();g.units[1].initiative=Some(super::super::original_initiative::Initiative::Flag(0));g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().flags8a&4,0);
        g.native_group_rosters.record_personal_capture(slot,true);g.maintain_native_group_rosters();assert_ne!(g.native_group_rosters.members[slot].as_ref().unwrap().flags8a&4,0);g.native_group_rosters.record_personal_capture(slot,false);g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().flags8a&4,0);
    }
    #[test]fn stop_and_contact_refresh_never_recreate_or_replace_a_march_order(){
        let mut g=field();g.units[1].formation=None;g.units[1].explicit_stop=true;g.units[1].order="guard".into();g.maintain_native_group_rosters();assert!(g.units[1].formation.is_none());
        let goal=g.units[2].formation.as_ref().unwrap().command_goal;g.units[2].order="enter".into();g.units[2].formation.as_mut().unwrap().slot=19;g.maintain_native_group_rosters();let follow=g.units[2].formation.as_ref().unwrap();assert_eq!(follow.slot,2);assert_eq!(follow.command_goal,goal);assert_eq!(g.units[2].order,"enter");
    }
    #[test]fn crew_death_promotion_uses_the_snapshot_from_before_damage_cleared_the_path(){
        let mut g=field();g.units[0].dest=Some((600.,500.));g.units[0].path=VecDeque::from([(400.,300.),(600.,500.)]);let previous=g.units[0].clone();g.remember_original_group_order(0,&previous);
        g.units[0].path.clear();g.units[0].dest=None;g.units[0].order="guard".into();g.units[0].hp=-1.;g.remove_original_group_member(0);
        assert_eq!(g.units[1].order,"move");assert_eq!(g.units[1].dest,previous.dest);assert_eq!(g.units[1].path,previous.path);
    }
    fn empty_hardware(g:&mut Game,kind:&str,object_type:u8,equipment:u8)->usize{
        g.spawn(kind,-1,400.,400.,999,None);let id=g.units.last().unwrap().id;
        g.objects=vec![crate::json!({"type":object_type,"equipmentId":equipment,"nativeSlot":17,"x":400,"y":400})];g.import_native_world(&crate::Value::Null);assert_eq!(g.native_object_slot(id),Some(17));id
    }
    #[test]fn initial_hardware_claim_uses_native_pool_slots_and_release_preserves_the_saved_target(){
        let mut g=field();let id=empty_hardware(&mut g,"medium",8,1);let slot=g.units[0].native_slot.unwrap();g.units[0].order="enter".into();g.units[0].target=Some(id);g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().flags31&8,0,"a browser order name is not a native reservation event");
        let order=g.claim_native_hardware_order(0,id).unwrap();assert_eq!(order.object,17);assert_eq!(order.destination,[208,196]);assert_eq!(g.units[0].motion_bonus,3);assert!(g.claim_native_hardware_order(1,id).is_none());
        assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().reserved_object,Some(17));assert!(g.release_original_claim(0));assert_eq!(g.native_group_rosters.contact_target(slot),Some(17));assert!(g.claim_native_hardware_order(1,id).is_some());
    }
    #[test]fn promotion_during_local_boarding_retains_the_original_march_not_the_temporary_enter_order(){
        let mut g=field();let id=empty_hardware(&mut g,"medium",8,1);g.units[0].dest=Some((1400.,320.));g.units[0].path=VecDeque::from([(1400.,320.)]);let primary=g.units[0].path.clone();g.claim_native_hardware_order(0,id).unwrap();
        g.units[0].initiative=Some(super::super::original_initiative::Initiative::Vehicle(id));g.units[0].march_resume=Some((1400.,320.));g.units[0].order="enter".into();g.units[0].target=Some(id);g.units[0].dest=Some((400.,400.));g.units[0].path=VecDeque::from([(400.,400.)]);let previous=g.units[0].clone();g.remember_original_group_order(0,&previous);g.units[0].path.clear();g.remove_original_group_member(0);
        assert_eq!(g.units[1].order,"move");assert_eq!(g.units[1].dest,Some((1400.,320.)));assert_eq!(g.units[1].path,primary);assert_eq!(g.units[1].target,None);
    }
    #[test]fn apc_claim_requires_the_native_group_move_target_and_preserves_578b0_busy_controls(){
        let mut g=field();let id=empty_hardware(&mut g,"apc",31,10);let slot=g.units[0].native_slot.unwrap();assert!(g.claim_native_hardware_order(0,id).is_none());
        g.native_group_rosters.set_current_roster(1,101);
        g.native_group_rosters.record_group_order(1,101,1,[208,208],0);g.units[0].native_flags64=8;g.native_group_rosters.record_claim_control(slot,8,0,g.motion_tick);assert!(g.claim_native_hardware_order(0,id).is_none());
        g.units[0].native_flags64=0;g.units[0].native_protectionaa=1;assert!(g.claim_native_hardware_order(0,id).is_none());g.advance_native_order_preamble(0);
        assert!(g.claim_native_hardware_order(1,id).is_none(),"57790 only admits the leader");let order=g.claim_native_hardware_order(0,id).unwrap();assert_eq!(order.destination,[212,200]);
    }
    #[test]fn apc_goal_lookup_uses_current_roster_and_runtime_object_extent(){let mut g=field();let _id=empty_hardware(&mut g,"apc",31,10);g.objects[0]["templateExtent"]=crate::json!(1);g.import_native_world(&crate::Value::Null);g.native_group_rosters.record_group_order(1,101,1,[208,208],0);g.native_group_rosters.set_current_roster(1,101);assert_eq!(g.native_group_apc_target(0,17),None,"41c8b0 reads runtime metadata+20, not the static default");g.objects[0]["templateExtent"]=crate::json!(16);g.import_native_world(&crate::Value::Null);assert_eq!(g.native_group_apc_target(0,17),Some(17));g.native_group_rosters.current_roster=None;assert_eq!(g.native_group_apc_target(0,17),None,"57790 consumes actual global632194");}
    #[test]fn hardware_takeover_reforms_the_same_native_member_before_new_group_dispatch(){let mut g=field();let original=g.native_group_rosters.group_slot(1,101).unwrap();let slot=g.units[0].native_slot;g.units[0].path=VecDeque::from([(700.,600.)]);g.units[0].dest=Some((700.,600.));g.maintain_native_group_rosters();g.native_group_rosters.pool[original+1].roster[8]=3;g.native_group_rosters.pool[original+1].roster[60]=4;let new=g.reform_native_hardware_group(0).unwrap();assert_eq!(new,original+1);assert_eq!(g.units[0].native_slot,slot);assert_eq!(g.native_group_rosters.role(slot.unwrap()),Some(-1));assert_eq!(g.native_group_rosters.current_roster,Some(new));assert_eq!(g.native_group_rosters.pool[new].roster[8],3);assert_eq!(g.native_group_rosters.pool[new].roster[60],4);assert_eq!(g.native_group_rosters.pool[original].roster[18],2);assert_eq!(g.native_group_rosters.role(g.units[1].native_slot.unwrap()),Some(-1));assert_eq!(g.units[1].dest,Some((700.,600.)));}
    #[test]fn dead_followers_leave_sparse_roles_and_new_members_fill_the_first_hole(){
        let mut g=field();let group=g.native_group_rosters.group_slot(1,101).unwrap();assert_eq!(group,120);
        let slots=g.units.iter().map(|u|u.native_slot.unwrap()).collect::<Vec<_>>();
        assert_eq!(g.native_group_rosters.role(slots[2]),Some(2));
        g.units[1].hp=-1.;g.remove_original_group_member(1);
        assert_eq!(g.native_group_rosters.pool[group].member(1),None);assert_eq!(g.native_group_rosters.role(slots[2]),Some(2));
        g.spawn("grunt",1,232.,200.,101,None);g.maintain_native_group_rosters();
        let new=g.units.last().unwrap().native_slot.unwrap();assert_eq!(g.native_group_rosters.role(new),Some(1));assert_eq!(g.native_group_rosters.role(slots[2]),Some(2));
    }
    #[test]fn marked_follower_holes_compact_once_when_combat_ttl_expires(){
        let mut g=field();let group=g.native_group_rosters.group_slot(1,101).unwrap();let slot=g.units[2].native_slot.unwrap();
        for u in &mut g.units{u.group_activity_ticks=2;}
        g.units[1].hp=-1.;g.remove_original_group_member(1);assert_eq!(g.native_group_rosters.role(slot),Some(2));
        g.motion_tick+=1;g.advance_original_group_cooldowns();g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.role(slot),Some(2));
        assert_eq!(g.units[2].group_activity_ticks,2);g.update_native_groups();assert_eq!(g.units[2].group_activity_ticks,1);
        g.motion_tick+=1;g.advance_original_group_cooldowns();g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.role(slot),Some(1));assert_eq!(g.native_group_rosters.pool[group].member(2),None);
        assert_eq!(g.units[2].formation.as_ref().unwrap().slot,1);assert_eq!(g.units[2].group_activity_ticks,1);
        g.advance_original_group_cooldowns();assert_eq!(g.native_group_rosters.pool[group].roster[16],0);
    }
    #[test]fn authored_formation_is_initialized_and_temporary_column_is_preserved_during_maintenance(){
        let mut g=field();g.native_group_rosters=State::default();for u in &mut g.units{u.formation_layout=1;}
        g.maintain_native_group_rosters();let group=g.native_group_rosters.group_slot(1,101).unwrap();
        assert_eq!((g.native_group_rosters.pool[group].roster[8],g.native_group_rosters.pool[group].roster[60]),(1,1));
        g.native_group_rosters.set_current_roster(1,101);let leader=g.units[0].native_slot.unwrap();g.native_group_rosters.record_walking_formation(leader,0,0,23);
        g.maintain_native_group_rosters();assert_eq!(g.units[1].formation.as_ref().unwrap().layout,255);assert_eq!(g.native_group_rosters.pool[group].roster[60],1);
    }
    #[test]fn produced_group_registration_preserves_temporary_and_saved_masks_separately(){
        let mut g=field();g.native_group_rosters=State::default();g.native_group_rosters.record_produced_group(1,101,1);g.maintain_native_group_rosters();
        let group=g.native_group_rosters.group_slot(1,101).unwrap();assert_eq!((g.native_group_rosters.pool[group].roster[8],g.native_group_rosters.pool[group].roster[60]),(255,1));
        g.native_group_rosters.record_produced_group(1,101,2);assert_eq!((g.native_group_rosters.pool[group].roster[8],g.native_group_rosters.pool[group].roster[60]),(255,2));
    }
    #[test]fn hardware_pickup_detaches_last_native_follower_without_changing_entity_or_unit_slot(){
        let mut g=field();let slot=g.units[2].native_slot.unwrap();let id=g.units[2].id;
        let sectors=vec![task::Sector{id:0,native_slot:0,valid:true,rect:[0,0,32,32],owner:1,flag:[4,4],flags:16,human_strength:0,priority:0,rank:0}];
        let leader=task::Leader{native_slot:g.units[0].native_slot.unwrap(),team:1,runtime_class:0,family:4,passenger_class:0,position:native_position(&g.units[0]),accepts_orders:true,laser_can_cross:false};
        let mut groups=vec![task::Group{id:101,active:true,flags:24,task:-1,current_sector:Some(0),via_sector:None,assigned_sector:None,value:100,leader:Some(leader)}];
        let request=task::HardwareCapture{group:101,split_member:Some(slot),fallback_member:g.units[0].native_slot.unwrap(),object:999,position:[80,80]};
        let(new,selected)=g.split_original_hardware_member(1,&request,&mut groups,&sectors).unwrap();assert_eq!(selected,slot);assert_ne!(new.id,101);
        assert_eq!(g.units[2].id,id);assert_eq!(g.units[2].native_slot,Some(slot));assert_eq!(g.units[2].squad,new.id);assert!(g.units[2].formation.is_none());
        let source=g.native_group_rosters.group_slot(1,101).unwrap();let target=g.native_group_rosters.group_slot(1,new.id).unwrap();
        assert_eq!(target,121);assert_eq!(g.native_group_rosters.pool[source].member(2),None);assert_eq!(g.native_group_rosters.pool[target].member(0),Some(slot as u8));assert_eq!(g.native_group_rosters.role(slot),Some(-1));
    }
    #[test]fn flag_capture_rejoins_the_native_formation_instead_of_resuming_an_old_individual_goal(){
        let mut g=field();for u in &mut g.units{u.angle=native_math::angle(0);}let state=std::mem::take(&mut g.native_group_rosters);state.refresh_formations(&mut g);g.native_group_rosters=state;
        let slot=g.units[1].native_slot.unwrap();g.units[1].initiative=Some(super::super::original_initiative::Initiative::Flag(0));g.units[1].march_resume=Some((900.,900.));
        g.native_group_rosters.set_current_roster(1,101);g.native_group_rosters.set_group_angle(1,101,0);
        let delta=super::super::original_formation::offset_layout(1,0,0);let expected=(g.units[0].x+delta.0-16.,g.units[0].y+delta.1-16.);
        g.rejoin_original_member(slot);assert_eq!(g.units[1].dest,Some(expected));assert!(g.units[1].initiative.is_none());assert!(g.units[1].march_resume.is_none());assert_eq!(g.units[1].formation.as_ref().unwrap().slot,1);
    }
    #[test]fn apc_conversion_preserves_native_roster_roles_while_rebinding_the_web_entity(){
        let mut g=field();let slot=g.units[0].native_slot.unwrap();let driver=g.units[0].id;let group=g.native_group_rosters.group_slot(1,101).unwrap();
        g.spawn("apc",-1,400.,400.,0,None);let carrier=g.units.len()-1;let id=g.units[carrier].id;
        g.units[carrier].team=1;g.units[carrier].squad=101;g.units[carrier].native_slot=Some(slot);g.units[carrier].transport_actor=Some(driver);g.units[0].native_slot=None;g.units[0].transport_carrier=Some(id);
        g.maintain_native_group_rosters();assert_eq!(g.native_group_rosters.group_slot(1,101),Some(group));assert_eq!(g.native_group_rosters.pool[group].roster[18],3);assert_eq!(g.native_group_rosters.leader_slot(1,101),Some(slot));assert_eq!(g.native_group_rosters.role(g.units[2].native_slot.unwrap()),Some(2));
    }
    #[test]fn blocked_rejoin_still_clears_the_completed_flag_mark_before_transport_gating(){
        let mut g=field();let slot=g.units[1].native_slot.unwrap();let hidden=g.units[2].native_slot.unwrap();g.native_group_rosters.set_current_roster(1,101);g.native_group_rosters.members[hidden].as_mut().unwrap().flags8a|=32;
        g.units[1].initiative=Some(super::super::original_initiative::Initiative::Flag(0));g.units[1].march_resume=Some((900.,900.));g.units[1].path=VecDeque::from([(300.,300.)]);
        g.rejoin_original_member(slot);assert!(g.units[1].initiative.is_none());assert!(g.units[1].march_resume.is_none());assert_eq!(g.units[1].path,VecDeque::from([(300.,300.)]));assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().flags8a&4,0);
    }
}
