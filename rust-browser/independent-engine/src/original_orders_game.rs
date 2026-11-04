//! Verified45f070 post-validation order writes and538f0 start transition.
//! Classifier/dispersal and spatial target validation remain caller inputs;
//! a browser order name is never used to recover a native opcode after thefact.
use super::{Game,original_group_lifecycle as group};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Event{pub opcode:i8,pub dispatched_opcode:i8,pub waypoint:[i16;2],pub metadata:i16}
impl Event{
 ///An already classified45ef00 input. Coordinates are native waypoints,
 ///rather than the top-left anchor used by the browser's movement renderer.
 pub fn classified(opcode:i8,waypoint:[i16;2],metadata:i16)->Self{Self{opcode,dispatched_opcode:opcode,waypoint,metadata}}
}
///15ee0. Infantry identifies an individual by its low-byte native unit slot,
///or a group by the wrapped negative group byte. Its Y argument is deliberately
///the previous62dc3c global: this source branch does not write that global.
pub fn target_event(actor_class:u8,actor_family:u8,target_slot:u8,target_role:i8,target_group:u8,metadata:i16,previous_y:i16)->Option<Event>{
 if !matches!(actor_class,1|2){let x=if target_role==0{target_slot}else{target_group.wrapping_add(1).wrapping_neg()};return Some(Event::classified(5,[i16::from(x),previous_y],metadata));}
 if actor_family==36{return None;}
 Some(Event::classified(2,if target_role==0{[-4321,i16::from(target_slot)]}else{[-1234,i16::from(target_group)]},metadata))
}
///150a0 encodes a BUILD/ground target in opcode5's coordinate sentinel form.
///The impact Y is stored in metadata, even for a hardware actor.
pub fn building_target_event(impact:[i16;2])->Event{Event::classified(5,[3456,impact[0]],impact[1])}
///48470 reference decoding. The byte result can address255 in malformed native
///state; a caller without that raw memory must report the reference unavailable.
pub fn target_reference(class:u8,field50:i32,field54:i16,mut leader:impl FnMut(usize)->Option<u8>)->Option<u8>{
 if class!=1{if field50<0{let magnitude=field50.wrapping_neg()as u8;return leader(usize::from(magnitude).checked_sub(1)?);}return(field50!=3456).then_some(field50 as u8);}
 if field50== -1234&&!matches!(field54,-1|255){return usize::try_from(field54).ok().and_then(&mut leader);}if field50== -4321{return u8::try_from(field54).ok();}None
}
pub fn valid_live_target(actor_flags:u8,target_flags:u8,target_protection:u8)->bool{target_flags&4!=0&&(actor_flags^target_flags)&3!=0&&target_protection==0}
///53290 successful live-target branch, whose spatial/path calls remain caller
///inputs. This is where an accepted positive5 changes to−5;48250 is later work.
pub fn accept_live_target(member:&mut group::Member,flags9c:&mut u8,flags71:&mut u8,current:Option<&mut group::GroupSlot>){
 *flags9c&=!32;member.flags9c=*flags9c;member.flags31&=!8;member.contact_order=0;if let Some(row)=current{row.roster[9]&=!8;row.roster[61]=0;}*flags71&=!32;if member.promotion.target_aux!=0{*flags71|=32;}member.promotion.order=(member.promotion.order as i8).wrapping_neg()as u8;member.promotion.motion=1;
}
///441940's validated infantry target branch versus441bf0's hardware branch.
///The UI dispatcher chooses the branch; this does not guess it from an order
///name. Unlike15ee0, UI infantry keeps the negative group index signed16-bit.
pub fn ui_target_event(hardware:bool,target_slot:u8,target_role:i8,target_group:u8,previous_y:i16)->Event{
 if hardware{Event::classified(2,if target_role==0{[-4321,i16::from(target_slot)]}else{[-1234,i16::from(target_group)]},0)}
 else{Event::classified(5,[if target_role==0{i16::from(target_slot)}else{-1-i16::from(target_group)},previous_y],0)}
}
///52810's control writes before its525a0 boundary. Both path heads are raw
///waypoints. ReturningNone can still clear64:80/71:40 when both paths are empty.
pub fn start_path(flags31:u8,flags9c:u8,flags64:&mut u8,flags71:&mut u8,contact8c:&mut u8,phase61:&mut u8,primary:Option<[i16;2]>,secondary:Option<[i16;2]>)->Option<[i16;2]>{
 if *flags71&1!=0{return None;}*flags64&=!128;*flags71&=!64;
 let head=if flags31&32!=0{*flags64|=128;secondary}else{primary}?;
 if flags31&8==0&&flags9c&16==0{*contact8c=0;}*phase61=0;
 Some([head[0].wrapping_sub(8),head[1].wrapping_sub(8)])
}
///437790/437830. The callback reads unit+18's permission at the terrain code
///from the raw128-cell-wide plane. Its input is the native signed cell index.
pub fn footprint_clear(point:[i16;2],wide:bool,mut allowed:impl FnMut(i16)->bool)->bool{
 let r=if wide{15}else{7};let left=point[0].wrapping_sub(r)>>4;let right=point[0].wrapping_add(r)>>4;
 let top=((i32::from(point[1])*8-i32::from(r)*8+7)&!127)as i16;let bottom=((i32::from(point[1])*8+i32::from(r)*8)&!127)as i16;
 [left.wrapping_add(top),right.wrapping_add(top),left.wrapping_add(bottom),right.wrapping_add(bottom)].into_iter().all(&mut allowed)
}
///19340 tries eight source offsets, consuming exactly one caller-supplied
///60910 result. A reserved point is tested only after the footprint succeeds.
pub fn approach_point(origin:[i16;2],family:u8,random:u32,mut allowed:impl FnMut(i16)->bool,mut reserved:impl FnMut([i16;2])->bool)->Option<[i16;2]>{
 const OFFSETS:[[i16;2];8]=[[-48,-48],[0,-48],[48,-48],[48,0],[48,48],[0,48],[-48,48],[-48,0]];
 for n in 0..8{let o=OFFSETS[((random as usize&7)+n)&7];let point=[origin[0].wrapping_add(o[0]),origin[1].wrapping_add(o[1])];if footprint_clear(point,matches!(family,6|25|30|24|32|34|36|38),&mut allowed)&&!reserved(point){return Some(point);}}None
}
///58620 reads the CURRENT global632194 roster, which need not be the actor's
///own group. It copies arguments without creating an event or changing flags.
pub fn sync_group_order(member:&mut group::Member,current:&group::GroupSlot)->bool{
 let command=current.roster[58];if command==0||member.role!= -1||member.promotion.order!=12&&member.flags69&128!=0{return false;}
 member.promotion.order=command;member.promotion.target_x=i32::from(current.word(44));member.promotion.target_y=current.word(46);member.promotion.target_aux=current.word(48);true
}
#[derive(Clone,Copy,Debug)]pub struct GoalReservation{pub active:bool,pub leader_exists:bool,pub role:i8,pub class:u8,pub flags31:u8,pub phase:u8,pub command:i8,pub goal:[i16;2],pub valid_sector:bool}
///19240. Only leader roles participate; hardware class1 uses group command1
///and31:10 clear, while other classes use walking phase2.
pub fn reserved_goal(point:[i16;2],r:GoalReservation)->bool{
 r.active&&r.leader_exists&&r.role== -1&&(if r.class==1{r.flags31&16==0&&r.command==1}else{r.phase==2})&&r.valid_sector&&r.goal[0]&!15==point[0]&!15&&r.goal[1]&!15==point[1]&!15
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct HardwareOrderRecord{
 ///Raw hardware+0e/+10, interpreted as sentinels/IDs or coordinates by handler.
 pub target:[i16;2],pub selected_target12:u8,pub counter13:i8,pub counter14:i8,pub field17:u8,
}
impl HardwareOrderRecord{
 pub(crate)fn overlay(self,b:&mut[u8;44]){b[14..16].copy_from_slice(&self.target[0].to_le_bytes());b[16..18].copy_from_slice(&self.target[1].to_le_bytes());b[18]=self.selected_target12;b[19]=self.counter13 as u8;b[20]=self.counter14 as u8;b[23]=self.field17;}
 pub(crate)fn from_bytes(b:&[u8;44])->Self{Self{target:[i16::from_le_bytes([b[14],b[15]]),i16::from_le_bytes([b[16],b[17]])],selected_target12:b[18],counter13:b[19]as i8,counter14:b[20]as i8,field17:b[23]}}

 ///41ba80/41bad0/41bb20/41bb70/57ff0/58050/58090 reset12 and17;
 ///582c0 resets only12. All target/counter bytes remain from the allocated
 ///44-byte pool slot. `Default` represents cold BSS, not a reused-slot reset.
 pub fn after_constructor(mut previous:Self,vehicle_constructor:bool)->Self{previous.selected_target12=255;if vehicle_constructor{previous.field17=0;}previous}
}
#[derive(Clone,Copy,Debug,Default)]pub struct HardwarePoolSlot{pub live_marker:i16,pub order:HardwareOrderRecord,pub attributes:super::original_entry::HardwareAttributes}
impl HardwarePoolSlot{
 ///41ba00's exact projection of the selected weapon table. It preserves
 ///animation/target bytes not written by that initializer.
 pub fn initialize_profile(&mut self,hardware:super::native_ai_scoring::Hardware){let b=&mut self.attributes.bytes;b[5]=hardware.range;b[13]=hardware.range;b[7]=hardware.movement;b[12]=hardware.movement;b[6]=hardware.damage as u8;b[8..10].copy_from_slice(&hardware.hp.to_le_bytes());b[10..12].copy_from_slice(&hardware.hp.to_le_bytes());b[32]=hardware.fire_rate;b[33]=hardware.projectile_speed;b[35]=hardware.secondary_damage as u8;b[36]=hardware.secondary_rate;b[37]=hardware.secondary_speed;b[38]=hardware.secondary_range;}
 pub fn set_order(&mut self,order:HardwareOrderRecord){self.order=order;self.synchronize_order_bytes();}
 pub fn synchronize_order_bytes(&mut self){let b=&mut self.attributes.bytes;b[..2].copy_from_slice(&self.live_marker.to_le_bytes());b[14..16].copy_from_slice(&self.order.target[0].to_le_bytes());b[16..18].copy_from_slice(&self.order.target[1].to_le_bytes());b[18]=self.order.selected_target12;b[19]=self.order.counter13 as u8;b[20]=self.order.counter14 as u8;b[23]=self.order.field17;}
}
///581f0 scans240 entries at59e6c2, stride44, testing only the first signed
///word forFFFF. Allocation writes that word0 and preserves all order bytes.
pub fn allocate_hardware_slot(pool:&mut[HardwarePoolSlot])->Option<usize>{let slot=pool.iter().take(240).position(|s|s.live_marker== -1)?;pool[slot].live_marker=0;pool[slot].attributes.bytes[..2].fill(0);Some(slot)}
///460430 mission reset touches only each slot's first word; all target and
///counter bytes survive.58660 deactivation frees only the valid live pointer.
pub fn clear_hardware_pool_for_mission(pool:&mut[HardwarePoolSlot]){for slot in pool.iter_mut().take(240){slot.live_marker= -1;slot.attributes.bytes[..2].fill(255);}}
pub fn release_hardware_slot(pool:&mut[HardwarePoolSlot],slot:usize){if let Some(slot)=pool.get_mut(slot).filter(|_|slot<240){slot.live_marker= -1;slot.attributes.bytes[..2].fill(255);}}
///54160 before59d80/54100. `restore_group` is the raw4a651e global, while
///`uniform` is exactly54070||540b0 on current632194. Cleanup must execute
///before the caller clears64:8 and dispatches the returned opcode.
pub fn command_prefix(member:&mut group::Member,current:Option<&mut group::GroupSlot>,flags9c:&mut u8,protection:&mut u8,restore_group:bool,uniform:bool)->Option<i8>{
 let mut current=current;if member.promotion.order as i8<=0{if !restore_group{return None;}if let Some(row)=current.as_deref(){sync_group_order(member,row);}if member.promotion.order as i8<=0{return None;}}
 member.flagsb7&=!1;if *flags9c&32!=0{if *flags9c&4!=0{return None;}if *flags9c&16!=0{*protection=68;*flags9c=(*flags9c&!16)|4;if member.role<0{if let Some(row)=current.as_deref_mut(){row.roster[9]|=32;}}}}
 member.flags9c=*flags9c;if !uniform{return None;}Some((member.promotion.order as i8).wrapping_abs())
}
///52d90. The hardware+12 write also targets the native dummy record when
///UNIT+8 is null/-1; its caller owns that shared dummy record separately.
pub fn stop_order(member:&mut group::Member,flags9c:&mut u8,record:Option<&mut HardwareOrderRecord>){*flags9c&=!32;member.flags9c=*flags9c;if let Some(record)=record{record.selected_target12=255;}member.flags31&=!16;member.promotion.order=0;member.promotion.timer3d=0;member.promotion.motion=2;}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct HardwareStart{pub clear_primary:bool,pub clear_field68:bool,pub phase:Option<u8>,pub clear_timer_word:bool,pub result:u8}
///52b70. `movement7` is unsigned hardware+7; `aim6` is signed UNIT+6. A
///missing real record deliberately leaves opcode2 unchanged and returns4.
pub fn hardware_order(member:&mut group::Member,flags9c:&mut u8,flags64:&mut u8,flags71:&mut u8,record:Option<&mut HardwareOrderRecord>,class:u8,family:u8,movement7:u8,aim6:i8,current:Option<&mut group::GroupSlot>)->HardwareStart{
 *flags9c&=!32;member.flags9c=*flags9c;let Some(record)=record else{return HardwareStart{clear_primary:false,clear_field68:false,phase:None,clear_timer_word:false,result:4};};member.promotion.order=0;if let Some(row)=current{row.roster[61]=0;}record.target=[member.promotion.target_x as i16,member.promotion.target_y];*flags71&=!32;if member.promotion.target_aux!=0{*flags71|=32;}member.flags31|=16;member.promotion.motion=3;
 if class!=1{let counter=127u8.wrapping_sub(movement7)as i8;record.counter13=counter;record.counter14=counter;return HardwareStart{clear_primary:false,clear_field68:false,phase:None,clear_timer_word:true,result:4};}
 let counter=((127i32-i32::from(aim6))>>1)as i8;record.counter13=counter;record.counter14=counter;let phase=match family{6=>9,24=>53,30=>55,32=>56,34=>58,36=>59,38=>60,_=>54};*flags64|=8;member.promotion.phase=phase;HardwareStart{clear_primary:true,clear_field68:true,phase:Some(phase),clear_timer_word:false,result:phase}
}
///45f494..45f58c after43aaf0/43a9f0 classification.43a600 may adjust the
///point or dispatched opcode; the original opcode is still written into5c.
pub fn write_event(member:&mut group::Member,roster:Option<&mut group::GroupSlot>,flags9c:&mut u8,counter6e:&mut u8,event:Event){
 *flags9c|=32;member.flags9c=*flags9c;member.promotion.order=event.opcode as u8;
 if matches!(event.opcode,8|9){return;}
 if !matches!(event.opcode,11|12)&&member.role!=0{if let Some(roster)=roster{roster.roster[44..46].copy_from_slice(&event.waypoint[0].to_le_bytes());roster.roster[46..48].copy_from_slice(&event.waypoint[1].to_le_bytes());roster.roster[48..50].copy_from_slice(&event.metadata.to_le_bytes());roster.roster[58]=event.dispatched_opcode as u8;}}
 if event.opcode==5{member.flags69&=0x7f;}
 member.promotion.target_x=i32::from(event.waypoint[0]);member.promotion.target_y=event.waypoint[1];member.promotion.target_aux=event.metadata;member.flags69&=!8;
 if event.dispatched_opcode!=12{member.flags69&=!32;}member.flags69&=!16;*counter6e=0;
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn ai_target_arguments_match_512_native_constructor_records(){
  let mut hash=2166136261u32;for s in 0..512{let cls=[0,1,2,3,4,5,6,15][s%8];let event=target_event(cls,if s%5==0{36}else{4},(s*13%240)as u8,[-1,0,1][s/8%3],(s*19%240)as u8,-300+s as i16*11,-32700+s as i16*13);
   let values=if let Some(e)=event{[s as i32,1,i32::from(e.opcode),i32::from(e.waypoint[0]),i32::from(e.waypoint[1]),i32::from(e.metadata)]}else{[s as i32,0,77,-1900+s as i32,i32::from(-32700+s as i16*13),3900+s as i32]};for n in values{hash=(hash^n as u32).wrapping_mul(16777619);}
  }assert_eq!(hash,3351344855,"ZNativeOrderConstructorOracle executes15ee0, stops before45ef00");
 }
 #[test]fn ui_target_arguments_match_512_native_constructor_records(){let mut hash=2166136261u32;for s in 0..512{let e=ui_target_event(s&1!=0,(s*13%240)as u8,[-1,0,1][s/2%3],(s*19%240)as u8,-32000+s as i16*17);for n in[s as i32,i32::from(e.opcode),i32::from(e.waypoint[0]),i32::from(e.waypoint[1]),i32::from(e.metadata)]{hash=(hash^n as u32).wrapping_mul(16777619);}}assert_eq!(hash,1685632851,"actual441940 target branch/441bf0, UI feedback isolated");}
 #[test]fn path_start_flags_and_anchor_match_512_executed_native_records(){let mut hash=2166136261u32;for s in 0..512{let(mut f64,mut f71,mut c8c,mut p61)=((s*3)as u8,s as u8,(s*11)as u8,(s*13)as u8);let target=start_path((s*5)as u8,(s*7)as u8,&mut f64,&mut f71,&mut c8c,&mut p61,(s%3!=0).then_some([-300+s as i16*17,400-s as i16*23]),(s%5!=0).then_some([200+s as i16*7,-250-s as i16*13]));for n in[s as i32,i32::from(target.is_some()),i32::from(target.unwrap_or([0,0])[0]),i32::from(target.unwrap_or([0,0])[1]),i32::from(f64),i32::from(f71),i32::from(c8c),i32::from(p61)]{hash=(hash^n as u32).wrapping_mul(16777619);}}assert_eq!(hash,3806350524,"actual52810;525a0 line initialization isolated");}
 #[test]fn approach_selection_and_footprint_match_512_native_records(){let mut hash=2166136261u32;for s in 0..512{let mut checks=0;let target=approach_point([512+(s%16)as i16,768+(s/16%16)as i16],[4,6,25,30,24,32,34,36,38,12][s%10],s as u32*31,|cell|{let x=cell as usize&127;let y=cell as usize>>7;(x*3+y*7+s)%17>=s%5},|point|{checks+=1;(i32::from(point[0]>>4)+i32::from(point[1]>>4)+s as i32)%5==0});for n in[s as i32,i32::from(target.is_none()),i32::from(target.unwrap_or([-123,456])[0]),i32::from(target.unwrap_or([-123,456])[1]),1,checks]{hash=(hash^n as u32).wrapping_mul(16777619);}}assert_eq!(hash,1373291029,"19340 plus native437790/437830; RNG and19240 predicate are explicit inputs");}
 #[test]fn group_order_sync_matches_512_complete_native_executions(){let mut hash=2166136261u32;for s in 0..512{let mut roster=group::GroupSlot::empty();roster.roster[58]=(s%15)as u8;roster.roster[44..46].copy_from_slice(&(-32700+s as i16*23).to_le_bytes());roster.roster[46..48].copy_from_slice(&(32700-s as i16*17).to_le_bytes());roster.roster[48..50].copy_from_slice(&(-12345+s as i16*31).to_le_bytes());let mut member=group::Member{slot:0,group:0,role:[-1,0,1][s%3],flags31:5,flags69:(s*7)as u8,flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:group::PromotionState{order:(s%14)as u8,target_x:0x1234abcd+s as i32,target_y:-600-s as i16,target_aux:800+s as i16,..Default::default()}};sync_group_order(&mut member,&roster);for n in[s as i32,member.promotion.order as i8 as i32,member.promotion.target_x,i32::from(member.promotion.target_y),i32::from(member.promotion.target_aux),i32::from(member.flags69)]{hash=(hash^n as u32).wrapping_mul(16777619);}}assert_eq!(hash,712869616,"actual58620 without stubs");}
 #[test]fn aligned_goal_reservation_matches_512_complete_native_executions(){let mut hash=2166136261u32;for s in 0..512{let point=[800+(s%16)as i16,600+(s/16%16)as i16];let found=(0..3).any(|i|reserved_goal(point,GoalReservation{active:(s>>i)&1!=0,leader_exists:s&(8<<i)==0,role:[-1,0,1][(s+i)%3],class:((s/3+i)%4)as u8,flags31:(s*7+i*5)as u8,phase:((s/11+i)%4)as u8,command:((s/13+i)%3)as i8,goal:[point[0]+[-32,-1,0,15,16][(s+i)%5],point[1]+[-16,0,1,16][(s/5+i)%4]],valid_sector:s&128==0}));for n in[s as u32,u32::from(found)]{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,4174562818,"complete19240 plus17400 with no stubs");}
 #[test]fn stop_and_hardware_order_writes_match_1024_native_executions(){let mut hash=2166136261u32;for mode in 0..2{for s in 0..512{let mut row=group::GroupSlot::empty();row.roster[61]=(s*31)as u8;let mut member=group::Member{slot:0,group:0,role:-1,flags31:(s*5)as u8,flags69:0,flags8a:0,flags9c:(s*7)as u8,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:group::PromotionState{order:if mode==0{4}else{2},phase:(7+s%50)as u8,timer3d:(s*19)as u8,motion:(s*13)as u8,target_x:0x1234abcd+s as i32,target_y:-600-s as i16,target_aux:(s&1)as i16,..Default::default()}};let(mut f9c,mut f64,mut f71,mut t3c,mut field68)=((s*7)as u8,(s*3)as u8,s as u8,(s*11)as u8,(s*37)as u8);let mut record=HardwareOrderRecord{target:[500+s as i16,600-s as i16],selected_target12:(s*17)as u8,counter13:(s*19)as i8,counter14:(s*23)as i8,field17:(s*29)as u8};let mut cleared=0;
  if mode==0{stop_order(&mut member,&mut f9c,Some(&mut record));}else{let result=hardware_order(&mut member,&mut f9c,&mut f64,&mut f71,if s%5==0{None}else{Some(&mut record)},[0,1,2,4,6][s%5],[5,6,24,25,30,32,34,36,38][s%9],(s*13)as u8,(s*11)as i8,Some(&mut row));t3c=0;if result.clear_timer_word{member.promotion.timer3d=0;}if result.clear_field68{field68=0;}cleared=i32::from(result.clear_primary);}
  for n in[mode,s as i32,member.promotion.order as i8 as i32,i32::from(f9c),i32::from(member.flags31),i32::from(f64),i32::from(f71),i32::from(member.promotion.phase),i32::from(t3c),i32::from(member.promotion.timer3d),i32::from(member.promotion.motion),i32::from(field68),i32::from(row.roster[61]),i32::from(record.target[0]),i32::from(record.target[1]),i32::from(record.selected_target12),i32::from(record.counter13),i32::from(record.counter14),i32::from(record.field17),cleared]{hash=(hash^n as u32).wrapping_mul(16777619);}
 }}assert_eq!(hash,138190065,"actual52d90/52b70;52500 path destruction isolated and counted");}
 #[test]fn command_dispatch_prefix_matches_512_native_executions(){let mut hash=2166136261u32;for s in 0..512{let mut row=group::GroupSlot::empty();row.roster[9]=(s*23)as u8;row.roster[58]=(s%15)as u8;row.roster[44..46].copy_from_slice(&(-32700+s as i16*23).to_le_bytes());row.roster[46..48].copy_from_slice(&(32700-s as i16*17).to_le_bytes());row.roster[48..50].copy_from_slice(&(-12345+s as i16*31).to_le_bytes());let mut member=group::Member{slot:0,group:0,role:[-1,0,1][s%3],flags31:5,flags69:(s*7)as u8,flags8a:0,flags9c:(s*11)as u8,flagsb7:(s*13)as u8,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:group::PromotionState{order:(s as i32%31-15)as u8,target_x:0x1234abcd+s as i32,target_y:-600-s as i16,target_aux:800+s as i16,..Default::default()}};let(mut flags,mut aa,mut flags64)=((s*11)as u8,(s*17)as u8,(s*19)as u8);let bits=(0..3).map(|i|((s*(i+3)+i*17)as u8)&32!=0).collect::<Vec<_>>();let uniform=bits.iter().all(|&b|b==bits[0]);let result=command_prefix(&mut member,Some(&mut row),&mut flags,&mut aa,s&1!=0,uniform);let cleanup=if result.is_some(){flags64&=!8;2}else{0};let handler=result.filter(|p|matches!(p,1|2|3|4|5|12|13)).unwrap_or(0);if handler==12&&s&2==0{member.flags69|=32;}
  for n in[s as i32,member.promotion.order as i8 as i32,i32::from(flags),i32::from(member.flagsb7),i32::from(aa),i32::from(flags64),i32::from(member.flags69),member.promotion.target_x,i32::from(member.promotion.target_y),i32::from(member.promotion.target_aux),i32::from(row.roster[9]),cleanup,i32::from(handler)]{hash=(hash^n as u32).wrapping_mul(16777619);}
 }assert_eq!(hash,4036233918,"real54160/54070/540b0; cleanup and final handlers are explicit recorded boundaries");}
 #[test]fn hardware_spawn_reuses_the_shared_pool_without_erasing_stale_order_fields(){let mut g=Game::new(0,1,9001);g.map.original=true;g.units.clear();g.native_group_rosters.clear();clear_hardware_pool_for_mission(&mut g.native_hardware_pool);g.spawn("cannon",0,200.,200.,1,None);g.maintain_native_group_rosters();let slot=g.units[0].native_hardware_slot.unwrap();g.native_hardware_pool[slot].order=HardwareOrderRecord{target:[-300,700],selected_target12:3,counter13:-20,counter14:90,field17:77};assert!(g.release_native_hardware_record(0));g.units[0].hp=-1.;g.spawn("medium",0,400.,200.,2,None);assert_eq!(g.units[1].native_hardware_slot,Some(slot));assert_eq!(g.native_hardware_pool[slot].order,HardwareOrderRecord{target:[-300,700],selected_target12:255,counter13:-20,counter14:90,field17:0});g.spawn("heavy",-1,600.,200.,3,None);assert_eq!(g.units[2].native_hardware_slot,None);}
 #[test]fn hardware_allocator_preserves_owned_fields_in_all240_slots_in_256_native_cases(){let mut hash=2166136261u32;for s in 0..256{let mut pool=(0..240).map(|i|{let b=|j|(i*7+j*13+s)as u8;HardwarePoolSlot{live_marker:if i==s{-1}else{0},order:HardwareOrderRecord{target:[i16::from_le_bytes([b(14),b(15)]),i16::from_le_bytes([b(16),b(17)])],selected_target12:b(18),counter13:b(19)as i8,counter14:b(20)as i8,field17:b(23)},..Default::default()}}).collect::<Vec<_>>();let chosen=allocate_hardware_slot(&mut pool).map_or(-1,|i|i as i32);let mut h=2166136261u32;for slot in pool{for n in[i32::from(slot.live_marker),i32::from(slot.order.target[0]),i32::from(slot.order.target[1]),i32::from(slot.order.selected_target12),i32::from(slot.order.counter13),i32::from(slot.order.counter14),i32::from(slot.order.field17)]{h=(h^n as u32).wrapping_mul(16777619);}}for n in[s as u32,chosen as u32,h]{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,3587461077,"complete581f0 with no stubs, all240 slot markers and order-owned bytes");}
 #[test]fn hardware_profile_projection_matches_all19_original_records_and304_stale_slots(){let mut hash=2166136261u32;let kinds=["cannon","medium","grunt","tough","sniper","heavy","psycho","light","missile","launcher","gatling","jeep","truck","apc","howitzer","firetruck","crane","pyro","laser"];for s in 0..304{let mut slot=HardwarePoolSlot::default();for(i,b)in slot.attributes.bytes.iter_mut().enumerate(){*b=(s*7+i*13)as u8;}slot.initialize_profile(super::super::native_ai_scoring::default_input(kinds[s%19]).unwrap().hardware);let mut h=2166136261u32;for b in slot.attributes.bytes{h=(h^u32::from(b)).wrapping_mul(16777619);}for n in[s as u32,h]{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,1071639909,"complete41ba00 executes real robots.dat weapon records");}
 #[test]fn live_infantry_acceptance_matches256_native53290_records(){let mut hash=2166136261u32;for s in 0..256{let mut row=group::GroupSlot::empty();row.roster[9]=(s*13)as u8;row.roster[61]=(s*17)as u8;let mut member=group::Member{slot:0,group:0,role:-1,flags31:4|if s&4!=0{8}else{0},flags69:0,flags8a:0,flags9c:(s*11)as u8,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:3,position:[0,0],reserved_object:Some(0),promotion:group::PromotionState{order:5,target_x:1,target_aux:(s&1)as i16,..Default::default()}};let(mut flags9c,mut flags71)=((s*11)as u8,(s*7)as u8);accept_live_target(&mut member,&mut flags9c,&mut flags71,Some(&mut row));let counter=super::super::original_pursuit::refresh_counter(false,Some((s%4)as u8),true,[-128,-1,0,40,60,120,127][s%7],[50+(s%10)as i16*11,60+(s%7)as i16*13],[300+s as i16*31,400+s as i16*17]);let object13=if s&4!=0{(s*13)as u8&!1}else{(s*13)as u8};for n in[s as i32,member.promotion.order as i8 as i32,i32::from(member.flags31),i32::from(flags9c),i32::from(flags71),i32::from(member.promotion.motion),i32::from(counter),i32::from(row.roster[9]),i32::from(row.roster[61]),i32::from(object13),1,i32::from(s&1==0),8]{hash=(hash^n as u32).wrapping_mul(16777619);}}assert_eq!(hash,1910397716,"actual53290/48470/52ab0; range and paths explicit oracle boundaries");}
 #[test]fn target_acceptance_uses_raw_group_reference_and_waits_for_real_protection(){let mut g=Game::new(0,1,8091);g.map.original=true;g.units.clear();g.native_group_rosters.clear();g.spawn("grunt",0,200.,200.,10,None);g.spawn("grunt",1,600.,500.,20,None);g.maintain_native_group_rosters();g.native_group_rosters.set_current_roster(0,10);let e=g.native_ui_target_order_event(1,false).unwrap();assert!(e.waypoint[0]<0);assert!(g.record_native_order_event(0,e));g.units[0].target=None;g.units[1].native_protectionaa=1;assert_eq!(g.accept_native_infantry_target_order(0),None);assert_eq!(g.units[0].native_order5c,5);g.units[1].native_protectionaa=0;assert_eq!(g.accept_native_infantry_target_order(0),Some(g.units[1].id));assert_eq!(g.units[0].native_order5c,-5);let slot=g.units[0].native_slot.unwrap();assert_eq!(g.native_group_rosters.members[slot].as_ref().unwrap().promotion.motion,1);assert_eq!(g.accept_native_infantry_target_order(0),None,"an already accepted−5 must not replay the positive command prefix");}
 #[test]fn accepted_order_unit_and_roster_writes_match_512_executed_native_records(){
  let mut hash=2166136261u32;for s in 0..512{let opcode=(s%13+1)as i8;let mut roster=group::GroupSlot::empty();for(i,b)in roster.roster.iter_mut().enumerate(){*b=(s*3+i)as u8;}let mut member=group::Member{slot:0,group:0,role:[-1,0,1][(s/13)%3],flags31:5,flags69:(s*7)as u8,flags8a:0,flags9c:0,flagsb7:0,primary_path:false,secondary_path:false,cursor:0,contact_order:0,position:[0,0],reserved_object:None,promotion:group::PromotionState{target_x:0x1234abcd+s as i32,target_y:-900-s as i16,target_aux:190+s as i16,..Default::default()}};let mut flags=(s*13)as u8;let mut counter=s as u8;
   let point=[-200+s as i16*3+if opcode==1&&s&2!=0{5}else{0},300-s as i16*7-if opcode==1&&s&2!=0{7}else{0}];let event=Event{opcode,dispatched_opcode:if opcode==1&&s&1!=0{2}else{opcode},waypoint:point,metadata:-3+s as i16*11};write_event(&mut member,Some(&mut roster),&mut flags,&mut counter,event);
   for n in[s as u32,member.promotion.order as i8 as u32,u32::from(flags),u32::from(member.flags69),u32::from(counter),member.promotion.target_x as u32,member.promotion.target_y as u32,member.promotion.target_aux as u32,roster.word(44)as u32,roster.word(46)as u32,roster.word(48)as u32,roster.roster[58]as i8 as u32]{hash=(hash^n).wrapping_mul(16777619);}
  }assert_eq!(hash,2683690996,"ZNativeOrderEventOracle real45f070, classifier/dispersal andB3 spatial lookup are explicit boundaries");
 }
 #[test]fn new_orders_preserve_independent_protection_and_inactive_units_do_not_tick_it(){
  let mut g=Game::new(0,1,711);g.map.original=true;g.units.clear();g.native_group_rosters.clear();g.spawn("grunt",0,300.,300.,71,None);g.maintain_native_group_rosters();g.units[0].native_protectionaa=3;g.units[0].native_flags9c=0x34;
  assert!(g.record_native_order_event(0,Event{opcode:1,dispatched_opcode:1,waypoint:[300,400],metadata:0}));assert_eq!(g.units[0].native_protectionaa,3);g.advance_native_order_preamble(0);assert_eq!(g.units[0].native_protectionaa,2);assert_eq!(g.units[0].native_flags9c,32);
  g.native_group_rosters.set_current_roster(0,71);g.native_move_order_started(0);assert_eq!(g.units[0].native_order5c,-1);assert_eq!(g.units[0].native_flags9c,0);assert_eq!(g.units[0].native_protectionaa,2);g.units[0].hp=-1.;g.advance_native_order_preamble(0);assert_eq!(g.units[0].native_protectionaa,2);
 }
}
impl Game{
 ///Read-only48470 identity decoding for an already classified event. This
 ///does not test affinity/protection or acknowledge a positive pendingorder.
 pub(crate) fn native_order_target_index(&self,_index:usize,event:Event)->Option<usize>{
  let class=match event.dispatched_opcode{5=>0,2=>1,_=>return None};let slot=target_reference(class,i32::from(event.waypoint[0]),event.waypoint[1],|g|self.native_group_rosters.pool.get(g)?.member(0))?;self.units.iter().rposition(|u|u.native_slot==Some(usize::from(slot)))
 }
 pub(crate) fn native_ground_attack_point(&self,index:usize)->Option<[i16;2]>{let u=self.units.get(index)?;let m=self.native_group_rosters.members.get(u.native_slot?)?.as_ref()?;(m.promotion.target_x==3456).then_some([m.promotion.target_y,m.promotion.target_aux])}
 ///Allocate the real shared581f0/initial-constructor hardware slot. Repeated
 ///calls retain identity; a reused slot carries its prior target/counter bytes.
 pub(crate) fn allocate_native_hardware_record(&mut self,index:usize,vehicle_constructor:bool)->Option<usize>{
  if let Some(slot)=self.units.get(index)?.native_hardware_slot{return Some(slot);}let profile=super::native_ai_scoring::default_input(&self.units[index].kind)?;let slot=self.allocate_native_takeover_record(index)?;self.native_hardware_pool[slot].initialize_profile(profile.hardware);if let Some(state)=self.units[index].original_combat.as_mut(){state.fire_rate=profile.hardware.fire_rate;state.projectile_speed=Some(profile.hardware.projectile_speed);}self.units[index].native_reload7b=0;self.units[index].native_elevation76=0;self.units[index].native_range4=Some(profile.hardware.range.wrapping_mul(2)as i8);let old=self.native_hardware_pool[slot].order;self.native_hardware_pool[slot].set_order(HardwareOrderRecord::after_constructor(old,vehicle_constructor));Some(slot)
 }
 ///581f0 allocation alone.50be0/582c0's subsequent initialization belongs to
 ///the entry dispatcher; none of their writes are folded into this allocator.
 pub(crate) fn allocate_native_takeover_record(&mut self,index:usize)->Option<usize>{let unit=self.units.get(index)?;if !self.map.original||unit.team<0||unit.native_slot.is_none(){return None;}if let Some(slot)=unit.native_hardware_slot{return Some(slot);}let slot=allocate_hardware_slot(&mut self.native_hardware_pool)?;self.units[index].native_hardware_slot=Some(slot);Some(slot)}
 ///58660: only an active native unit with a real hardware pointer frees its
 ///pool marker. The source retains UNIT+8, so this does not erase its slot ID.
 pub(crate) fn release_native_hardware_record(&mut self,index:usize)->bool{
  let Some(unit)=self.units.get(index)else{return false;};let Some(unit_slot)=unit.native_slot else{return false;};if self.native_group_rosters.members.get(unit_slot).and_then(Option::as_ref).is_none_or(|m|m.flags31&4==0){return false;}let Some(slot)=unit.native_hardware_slot else{return false;};if slot>=240||slot>=self.native_hardware_pool.len(){return false;}release_hardware_slot(&mut self.native_hardware_pool,slot);true
 }
 ///54160's accepted-command prefix, before a specific native handler.
 ///Call once per unit callback; a FactoryExit caller already running its
 ///same prefix must not call this in addition.4a651e remains an explicitinput.
 pub(crate) fn prepare_native_order_dispatch(&mut self,index:usize,restore_group:bool)->Option<i8>{
  if !self.map.original||self.units.get(index).is_none_or(|u|u.team<0||!super::original_combat::alive(true,u.hp)){return None;}let slot=self.units[index].native_slot?;let uniform=self.original_global_transport_uniform();let context=self.native_group_rosters.current_roster;
  let mut member=self.native_group_rosters.members.get_mut(slot)?.take()?;member.promotion.order=self.units[index].native_order5c as u8;let u=&mut self.units[index];let result=command_prefix(&mut member,context.and_then(|s|self.native_group_rosters.pool.get_mut(s)),&mut u.native_flags9c,&mut u.native_protectionaa,restore_group,uniform);u.native_order5c=member.promotion.order as i8;self.native_group_rosters.members[slot]=Some(member);let result=result?;
  self.cleanup_native_order_transports(index);
  //54100 only cancels ground-target fire by the actor's own followers.
  let followers=self.native_group_rosters.members[slot].as_ref().filter(|m|m.role== -1).and_then(|m|self.native_group_rosters.pool.get(usize::from(m.group))).map(|g|g.roster[20..40].to_vec()).unwrap_or_default();
  for slot in followers.into_iter().filter(|&s|s!=255){let Some(i)=self.units.iter().position(|u|u.native_slot==Some(usize::from(slot)))else{continue;};let ground=self.native_group_rosters.members[usize::from(slot)].as_ref().is_some_and(|m|m.promotion.target_x==3456);if matches!(self.units[i].native_order5c,5|-5)&&ground{self.units[i].native_order5c=0;if let Some(m)=self.native_group_rosters.members[usize::from(slot)].as_mut(){m.promotion.order=0;}self.reset_native_idle(i);}}
  self.units[index].native_flags64&=!8;if result==12&&super::original_combat::animation_family(&self.units[index].kind)!=4{if let Some(m)=self.native_group_rosters.members[slot].as_mut(){m.flags69|=32;}}Some(result)
 }
 ///52d90 preserves inline movement and finite heads. The accepted browser
 ///caption is separate;54160 invokes these native writes in the callback.
 ///Pass the shared dummy record when the hardware pointer is null/-1.
 pub(crate) fn complete_native_stop_order(&mut self,index:usize,record:Option<&mut HardwareOrderRecord>)->bool{
  let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return false;};let Some(member)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut)else{return false;};stop_order(member,&mut self.units[index].native_flags9c,record);self.units[index].native_order5c=0;true
 }
 ///52b70 after54160. The source hardware pool record is supplied by its owner;
 ///no missing record is manufactured from the browser's unit kind.
 pub(crate) fn begin_native_hardware_order(&mut self,index:usize,record:Option<&mut HardwareOrderRecord>,movement7:u8,aim6:i8)->Option<HardwareStart>{
  let slot=self.units.get(index)?.native_slot?;let current=self.native_group_rosters.current_roster;let member=self.native_group_rosters.members.get_mut(slot)?.as_mut()?;member.promotion.order=self.units[index].native_order5c as u8;let u=&mut self.units[index];let result=hardware_order(member,&mut u.native_flags9c,&mut u.native_flags64,&mut u.native_initiative_scan.flags71,record,u.native_class7c,super::original_combat::animation_family(&u.kind),movement7,aim6,current.and_then(|s|self.native_group_rosters.pool.get_mut(s)));u.native_order5c=member.promotion.order as i8;u.native_timer3c=0;if result.clear_timer_word{member.promotion.timer3d=0;}if let Some(phase)=result.phase{u.native_phase35=phase;u.native_idle=None;u.native_walk=None;u.native_fire=None;u.native_cover=None;}if result.clear_primary{u.path.clear();u.original_motion=None;u.native_secondary_route.set(false);self.native_group_rosters.record_secondary_path(slot,false);}Some(result)
 }
 pub(crate) fn sync_native_group_order(&mut self,index:usize)->bool{
  if !self.map.original{return false;}let Some(slot)=self.units.get(index).and_then(|u|u.native_slot)else{return false;};let Some(current)=self.native_group_rosters.current_roster.and_then(|r|self.native_group_rosters.pool.get(r)).cloned()else{return false;};let Some(member)=self.native_group_rosters.members.get_mut(slot).and_then(Option::as_mut)else{return false;};member.promotion.order=self.units[index].native_order5c as u8;let changed=sync_group_order(member,&current);if changed{self.units[index].native_order5c=member.promotion.order as i8;}changed
 }
 ///5a080's independentAA preamble. Call once per actual native unitcallback,
 ///before54160/handlers. A new command never clears this counter.
 pub(crate) fn advance_native_order_preamble(&mut self,index:usize){
  if !self.map.original||self.units.get(index).is_none_or(|u|u.native_protectionaa==0||u.team<0||u.native_slot.is_none()||!super::original_combat::alive(true,u.hp)){return;}self.native_protected_order_seen=true;let u=&mut self.units[index];u.native_protectionaa-=1;u.native_flags9c&=!0x14;let slot=u.native_slot;
  if let Some(exit)=u.native_factory_exit.as_mut(){exit.protection=u.native_protectionaa;exit.flags9c=u.native_flags9c;}
  if let Some(member)=slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){member.flags9c=self.units[index].native_flags9c;let group=usize::from(member.group);if let Some(row)=self.native_group_rosters.pool.get_mut(group){row.roster[9]&=!32;}}
 }
 ///Call only for an accepted command, with native waypoint coordinates.
 ///Role0 updates the unit; group members additionally write theirown roster.
 pub(crate) fn record_native_order_event(&mut self,index:usize,event:Event)->bool{
  if !self.map.original{return false;}let Some(unit)=self.units.get(index)else{return false;};let Some(slot)=unit.native_slot.filter(|&s|s<240)else{return false;};
  if self.native_group_rosters.members[slot].is_none(){self.maintain_native_group_rosters();}
  let Some(mut member)=self.native_group_rosters.members[slot].take()else{return false;};let roster=self.native_group_rosters.pool.get_mut(usize::from(member.group));let u=&mut self.units[index];
  self.native_order_arguments=[event.waypoint[0],event.waypoint[1],event.metadata];
  write_event(&mut member,roster,&mut u.native_flags9c,&mut u.native_counter6e,event);u.native_order5c=member.promotion.order as i8;self.native_group_rosters.members[slot]=Some(member);if let Some(marker)=self.native_order_object_marker(event){self.units[index].native_fieldb3=marker;}true
 }
 ///Source15ee0 constructor using stable native identities, not browser IDs.
 pub(crate) fn native_target_order_event(&self,index:usize,target:usize,metadata:i16)->Option<Event>{
  let actor=self.units.get(index)?;let target=self.units.get(target)?;let slot=target.native_slot?;let member=self.native_group_rosters.members.get(slot)?.as_ref()?;
  target_event(actor.native_class7c,super::original_combat::animation_family(&actor.kind),slot as u8,member.role,member.group,metadata,self.native_order_arguments[1])
 }
 pub(crate) fn native_ui_target_order_event(&self,target:usize,hardware:bool)->Option<Event>{let target=self.units.get(target)?;let slot=target.native_slot?;let member=self.native_group_rosters.members.get(slot)?.as_ref()?;Some(ui_target_event(hardware,slot as u8,member.role,member.group,self.native_order_arguments[1]))}
 ///53290's successful ordinary infantry live-target subset. Group/unit target
 ///identity is resolved from raw50/54, not the browser target ID. Ground3456,
 ///class1/2, invalid/protected targets, and the near-origin random-jitter path
 ///remain outside this acceptance helper and must not be falsely acknowledged.
 pub(crate) fn accept_native_infantry_target_order(&mut self,index:usize)->Option<usize>{
  let unit=self.units.get(index)?.clone();if !self.map.original||unit.native_order5c!=5||matches!(unit.native_class7c,1|2){return None;}let slot=unit.native_slot?;let member=self.native_group_rosters.members.get(slot)?.as_ref()?;let context=self.native_group_rosters.current_roster?;let reference=target_reference(unit.native_class7c,member.promotion.target_x,member.promotion.target_y,|s|self.native_group_rosters.pool.get(s)?.member(0))?;
  let target=self.units.iter().rev().find(|u|u.native_slot==Some(usize::from(reference)))?.clone();let target_member=self.native_group_rosters.members.get(usize::from(reference))?.as_ref()?;if !valid_live_target(member.flags31,target_member.flags31,target.native_protectionaa){return None;}
  let xy=[(target.x/2.).floor()as i16,(target.y/2.).floor()as i16];if i32::from(xy[0]).wrapping_mul(i32::from(xy[0])).wrapping_add(i32::from(xy[1]).wrapping_mul(i32::from(xy[1])))<1024{return None;}
  self.release_original_claim(index);let member=self.native_group_rosters.members.get_mut(slot)?.as_mut()?;let u=&mut self.units[index];accept_live_target(member,&mut u.native_flags9c,&mut u.native_initiative_scan.flags71,self.native_group_rosters.pool.get_mut(context));u.native_order5c=member.promotion.order as i8;
  //52500 precedes the accepted53290 path/52ab0 branch. The existing pursuit
  //planner constructs its first route, so no flattened path pointer is forged.
  u.path.clear();u.original_motion=None;u.native_route_pending.set(None);u.native_secondary_route.set(false);u.pursuit=None;self.native_group_rosters.record_secondary_path(slot,false);Some(target.id)
 }
 ///538f0 after the movement command has actually obtained its path. Onlythe
 ///leader clears9c20/restores the saved formation; every role negates5c.
 pub(crate) fn native_move_order_started(&mut self,index:usize){
  if !self.map.original||self.units[index].native_order5c!=1{return;}let slot=self.units[index].native_slot;let role=slot.and_then(|s|self.native_group_rosters.role(s));
  if role==Some(-1){self.units[index].native_flags9c&=!32;if let Some(group)=self.native_group_rosters.current_roster.filter(|&s|s<240){let row=&mut self.native_group_rosters.pool[group].roster;row[8]=row[60];}}
  self.units[index].native_order5c=-1;if let Some(m)=slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.promotion.order=255;m.flags9c=self.units[index].native_flags9c;m.flags69&=0x7f;}
  //538f0 after a selected52810 head, hardware-only movement state.
  if self.units[index].native_class7c==1{let family=super::original_combat::animation_family(&self.units[index].kind);self.units[index].native_phase35=super::original_move_order::hardware_phase(family);self.units[index].native_flags64&=!8;if let Some(m)=slot.and_then(|s|self.native_group_rosters.members[s].as_mut()){m.flags31&=!16;}if self.units[index].native_initiative_scan.flags71&1==0{let w=self.units[index].native_hardware_slot;let mut record=w.map(|w|self.native_hardware_pool[w].order).unwrap_or(self.native_hardware_dummy);record.field17=(record.field17&!2)|1;if let Some(w)=w{self.native_hardware_pool[w].set_order(record);}else{self.native_hardware_dummy=record;}}}
 }
 ///53290 entry accepts non-class2 actors and clears9c20 before target work.
 ///The actual5→−5 transition belongs to53290's accepted live-target branch.
 pub(crate) fn native_infantry_target_order_entered(&mut self,index:usize){if self.map.original&&self.units[index].native_class7c!=2{self.units[index].native_flags9c&=!32;}}
}
