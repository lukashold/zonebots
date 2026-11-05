//! Source-bound attack feedback, phrase admission, and stop-pose control.
#[path="original_feedback_game.rs"]mod game;
pub const UNDER_ATTACK:[u16;5]=[0x419,0x41b,0x41c,0x41e,0x41f];
pub const CANNOT_REACH:u16=0x725;
pub const RETIREMENT_CREDIT:[u16;7]=[0x728,0x729,0x72a,0x72b,0x72c,0x72d,0x72e];
pub const VOICE_SAMPLES:[i16;22]=[152,154,155,157,158,164,167,168,169,170,171,172,173,208,209,210,211,212,213,214,215,216];
#[derive(Clone)]pub struct State{
 pub scene_gate:i32,pub local_team:u8,pub portraits_enabled:u8,pub portrait_mode:u8,pub interruptible:u8,
 pub current_message:i32,pub next_mode:i32,pub next_message:i32,pub speaker:Option<usize>,pub display_speaker:Option<usize>,
 pub attack_dirty:bool,pub attack_membership:[u8;240],pub attacked_groups:Vec<(u8,u8,i32)>,
 pub sound_enabled:u8,pub ui_clock:i16,pub phrase_counter:i16,pub phrase_frame:i16,pub phrase_length:i16,pub phrase_index:i16,pub double_portrait_advance:bool,pub start_clock:i32,pub last_voice_ticks:[i16;VOICE_SAMPLES.len()],pub voice_hold:i32,pub fort_radio:crate::original_fort_scene::radio::Radio,
}
impl Default for State{fn default()->Self{Self{scene_gate:0,local_team:0,portraits_enabled:1,portrait_mode:0,interruptible:0,current_message:0,next_mode:0,next_message:0,speaker:None,display_speaker:None,attack_dirty:false,attack_membership:[0;240],attacked_groups:vec![],sound_enabled:0,ui_clock:0,phrase_counter:0,phrase_frame:0,phrase_length:0,phrase_index:-1,double_portrait_advance:false,start_clock:0,last_voice_ticks:[0;VOICE_SAMPLES.len()],voice_hold:0,fort_radio:Default::default()}}}
#[derive(Clone,Copy)]pub struct Voice{pub clip:&'static str,pub phrase:u8,pub length:i16,pub maximum:u8}
pub fn voice(message:u16)->Option<(usize,Voice)>{let(index,clip,phrase,length,maximum)=match message{0x419=>(0,"ROB25",22,67,66),0x41b=>(1,"ROB27",24,70,94),0x41c=>(2,"ROB28",25,69,71),0x41e=>(3,"ROB30",27,73,68),0x41f=>(4,"ROB31",26,67,60),0x725=>(5,"ROB37",30,65,59),
0x728=>(6,"ROB40",50,48,47),0x729=>(7,"ROB41",52,52,49),0x72a=>(8,"ROB42",53,53,53),
0x72b=>(9,"ROB43",54,46,45),0x72c=>(10,"ROB44",55,76,77),0x72d=>(11,"ROB45",56,44,45),
0x72e=>(12,"ROB46",57,68,68),
0xa3d=>(13,"ROB61",1,48,90),0xa3e=>(14,"ROB62",1,48,73),0xa3f=>(15,"ROB63",1,48,90),
0xa40=>(16,"ROB64",1,48,63),0xa41=>(17,"ROB65",1,48,69),0xa42=>(18,"ROB66",1,48,78),
0xb43=>(19,"ROB67",1,48,54),0xb44=>(20,"ROB68",1,48,60),0xb45=>(21,"ROB69",1,48,66),_=>return None};Some((index,Voice{clip,phrase,length,maximum}))}
impl State{
 ///33ca0 keeps the UI clock and auxiliary globals across mission loads.
 ///46a650(2,0) preserves the old phrase pointer and its counter.
 pub fn initialize_for_mission(&mut self){self.scene_gate=0;self.fort_radio.win.flag=0;self.fort_radio.loss.flag=0;self.speaker=None;if self.portraits_enabled!=0{if self.portrait_mode==0{self.next_mode=2;self.next_message=0;}else{self.interruptible=0;}self.current_message=0;}self.portrait_mode=2;self.start_clock=i32::from(self.ui_clock);self.attack_dirty=true;}
 pub fn select_voice(&mut self,v:Voice){self.start_clock=i32::from(self.ui_clock);self.phrase_counter=0;self.phrase_frame=0;self.phrase_index=i16::from(v.phrase);self.phrase_length=v.length;self.voice_hold=i32::from(v.maximum);}
 ///46ad70 reads the old phrase-frame index before replacing it. The two
 ///passes in46ade0 are explicit; they are not inferred from browser speed.
 pub fn advance_phrase(&mut self){if self.portrait_mode==0{return;}if matches!(self.portrait_mode,3|4)&&self.phrase_index!=-1&&self.phrase_frame>=self.phrase_length{self.phrase_index=-1;self.portrait_mode=3;}self.phrase_frame=((i32::from(self.phrase_counter)*72)/17)as i16;self.phrase_counter=self.phrase_counter.wrapping_add(1);}
 pub fn sound_admitted(&mut self,index:usize)->bool{if self.portraits_enabled!=1||self.sound_enabled==0{return false;}let delta=self.ui_clock.wrapping_sub(self.last_voice_ticks[index]);let absolute=delta.wrapping_abs();if absolute<1{return false;}self.last_voice_ticks[index]=self.ui_clock;true}
}
#[derive(Clone,Copy,Debug)]pub struct AlertUnit{pub slot:u8,pub group:u8,pub flags31:u8,pub flags69:u8,pub position:[i16;2]}
///32c30: first native slot per attacked group, stable signed-distance order.
pub fn attacked_groups(units:&[AlertUnit],local_team:u8,origin:[i16;2],membership:&mut[u8;240])->Result<Vec<(u8,u8,i32)>,u8>{
 membership.fill(255);let mut list=vec![];for slot in 0..240u16{let Some(u)=units.iter().find(|u|u.slot==slot as u8)else{continue;};if u.flags31&3!=local_team||u.flags31&4==0||u.flags69&128==0||u.group==255{continue;}let Some(seen)=membership.get_mut(usize::from(u.group))else{return Err(u.group);};if *seen==0{continue;}*seen=0;let x=i32::from(u.position[0])-i32::from(origin[0]);let y=i32::from(u.position[1])-i32::from(origin[1]);let distance=x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y));list.push((u.group,u.slot,distance));}list.sort_by_key(|r|r.2);Ok(list)
}
///41f4d0 returns true outside a signed16, asymmetrically padded viewport.
pub fn outside_view(point:[i16;2],camera:[i16;2],size:[i16;2])->bool{
 let x=point[0].wrapping_sub(camera[0]);let y=point[1].wrapping_sub(camera[1]);
 x< -48||i32::from(x)>i32::from(size[0])+32||y< -48||i32::from(y)>i32::from(size[1])+32
}
pub fn group_gate_4326f0(group:u8,membership:&[u8;240])->bool{group==255||membership.get(usize::from(group)).is_none_or(|&v|v==0)}
pub fn under_attack_allowed(scene_gate:i32,group:u8,cooldown:u8,local_team:u8,flags31:u8)->bool{scene_gate==0&&group!=255&&cooldown==0&&local_team==flags31&3}
pub fn path_feedback_allowed(building:bool,bridge:bool,scene_gate:i32,current_message:i32,local_team:u8,flags31:u8)->bool{!building&&!bridge&&scene_gate==0&&current_message!=i32::from(CANNOT_REACH)&&local_team==flags31&3}
///421700 admission changes neither the current speaker nor its cooldown;
///46e890 stores those before calling it, even if this gate rejects playback.
pub fn admit(state:&mut State,kind:i16,message:i16)->bool{
 if state.scene_gate!=0{return false;}
 if state.portraits_enabled!=0{
  if state.portrait_mode!=0{
   if kind==4&&state.interruptible==0&&state.portrait_mode==4&&matches!(state.current_message&0xff00,0x400|0x500){return false;}
   state.interruptible=0;state.current_message=i32::from(message);
   if kind==4{state.next_mode=0;state.next_message=i32::from(message);}
  }else{state.next_mode=i32::from(kind);state.next_message=i32::from(message);state.current_message=i32::from(message);}
 }
 state.portrait_mode=if kind>=5{0}else{kind as u8};true
}
#[derive(Clone,Copy,Debug)]pub struct PoseUnit{pub slot:u8,pub class:u8,pub phase:u8}
pub fn stop_pose(actor:u8,role:i8,followers:&[u8],units:&[PoseUnit],terrain:u8)->Option<u8>{
 if matches!(terrain&127,9|38|39|42){return None;}
 let target=if role== -1{followers.iter().take(20).find_map(|&slot|units.iter().find(|u|u.slot==slot&&matches!(u.phase,0|1))).map_or(actor,|u|u.slot)}else{actor};
 units.iter().find(|u|u.slot==target&&matches!(u.class,0|3|15|16|17|18)).map(|u|u.slot)
}
///459200 treats88 as next phase and89 as the signed desired heading.
pub fn turn_transition(facing:u8,packed:i16)->(u8,Option<u8>){let target=(packed>>8)as i8;if target==-1||facing as i8==target{(facing,Some(packed as u8))}else if (i32::from(target)-i32::from(facing as i8))&4==0{(facing.wrapping_add(1)&7,None)}else{(facing.wrapping_sub(1)&7,None)}}
///457ae0's frame/cycle writes. True requests44cac0 and, for followers,
///the existing forced4d570 call. Neither byte80 nor the high7e byte changes.
pub fn tick_stop_pose(frame:&mut i16,cycles:&mut i16,callback_mode:u8,prevented:bool)->bool{
 if callback_mode!=0{return false;}if !prevented{*frame=frame.wrapping_add(1)&3;if *frame!=0{return false;}let low=(*cycles as u8)&7;if low!=0{*cycles=(*cycles&!255)|i16::from((*cycles as u8&248)|(low-1));*frame=0;return false;}}true
}
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for v in values{*h=(*h^v).wrapping_mul(16777619);}}
 #[test]fn native_2048_under_attack_callbacks_match_gate_cooldown_crt_calls_and_exact_message(){let mut h=2166136261;for s in 0..2048u32{let group=[0,3,255][s as usize%3];let cooldown=[0,1,127,128,255][(s/3%5)as usize];let allowed=under_attack_allowed(i32::from(s/7%3==0),group,cooldown,(s/11%4)as u8,(s/5%8)as u8);let draws=u32::from(allowed&&s/13%2==0);let msg=if allowed{i32::from(UNDER_ATTACK[if draws!=0{((s*31+0x1234)&32767)as usize%5}else{0}])}else{-1};mix(&mut h,[s,draws,u32::from(allowed),if allowed{4}else{u32::MAX},msg as u32,u32::from(if allowed{68}else{cooldown}),u32::from(allowed)]);}assert_eq!(h,457701326);}
 #[test]fn native_2048_phrase_admissions_match_busy_modes_current_message_and_refresh_sound_calls(){let mut h=2166136261;for s in 0..2048u32{let kind=[0,1,2,3,4,5][s as usize%6];let msg=[0x419,0x41b,0x41c,0x41e,0x41f,0x725][(s/6%6)as usize];let mut state=State{scene_gate:i32::from(s/7%3==0),portraits_enabled:(s/11%3)as u8,portrait_mode:[0,1,3,4][(s/13%4)as usize],interruptible:(s/17%2)as u8,current_message:[0,0x419,0x519,0x725][(s/19%4)as usize],next_mode:17,next_message:19,sound_enabled:(s/23%2)as u8,ui_clock:500,phrase_counter:9,phrase_frame:11,..Default::default()};let accepted=admit(&mut state,kind,msg);let refresh=accepted&&state.portraits_enabled!=0;let sounds=accepted&&kind==4&&state.portraits_enabled==1&&state.sound_enabled!=0;if accepted&&kind==4{state.select_voice(voice(msg as u16).unwrap().1);}mix(&mut h,[s,u32::from(state.portrait_mode),u32::from(state.interruptible),state.current_message as u32,state.next_mode as u32,state.next_message as u32,u32::from(state.phrase_counter as u16),u32::from(state.phrase_frame as u16),u32::from(refresh),u32::from(sounds)]);}assert_eq!(h,720522465);}
 #[test]fn native_1024_failed_path_feedback_gates_match_queries_scene_current_message_and_owner(){let mut h=2166136261;for s in 0..1024u32{let allowed=path_feedback_allowed(s&1!=0,s/2%2!=0,(s/7%2)as i32,if s/11%2==0{0x725}else{0x419},(s/13%4)as u8,(s/17%8)as u8);mix(&mut h,[s,u32::from(allowed),u32::from(allowed)]);}assert_eq!(h,111522623);}
 fn put(bytes:&mut[u8],offset:usize,value:u16){bytes[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
 #[test]fn native_1024_stop_pose_initializers_match_first_idle_follower_class_gate_and_all_unit_bytes(){let mut h=2166136261;for s in 0..1024usize{let mut records=vec![vec![0;192];21];let mut units=vec![];let mut followers=vec![];for(i,r)in records.iter_mut().enumerate(){r[124]=[0,1,2,3,15,16,17,18][(s/7+i)%8];r[53]=[0,1,2,7,33,18][(s+i*5)%6];put(r,50,0x1234);put(r,136,0x7faf);put(r,126,0x12e5);r[128]=239;units.push(PoseUnit{slot:i as u8,class:r[124],phase:r[53]});if i>0{followers.push(if(i+s)%4==0{255}else{i as u8});}}let role=[-1,0,1][s%3];records[0][94]=role as u8;records[0][95]=3;put(&mut records[0],44,512);put(&mut records[0],46,640);if let Some(slot)=stop_pose(0,role,&followers,&units,[0,9,38,39,42,128,255][s/3%7]){let r=&mut records[usize::from(slot)];put(r,136,0x421);r[53]=42;put(r,50,0);put(r,126,7);r[128]=3;}let mut bh=2166136261;mix(&mut bh,records.into_iter().flatten().map(u32::from));mix(&mut h,[s as u32,bh]);}assert_eq!(h,147161094);}
 #[test]fn native_1024_turn_transitions_match_signed_packed_target_and_octant_direction(){let mut h=2166136261;for s in 0..1024u32{let(facing,phase)=turn_transition([0,1,4,7,127,128,255][s as usize%7],(s*257+0x421)as i16);mix(&mut h,[s,u32::from(phase.unwrap_or(42)),u32::from(facing)]);}assert_eq!(h,1962447603);}
 #[test]fn native_1024_stop_pose_callbacks_match_alternate_gate_low_cycles_and_forced_follow_call(){let mut h=2166136261;for s in 0..1024u32{let mut frame=(s*31)as i16;let mut cycles=(s*19)as i16;let reset=tick_stop_pose(&mut frame,&mut cycles,(s%2)as u8,s/3%2!=0);let follow=reset&&[-1,0,1][(s/7%3)as usize]!= -1;mix(&mut h,[s,if reset{1}else{33},u32::from(frame as u16),u32::from(cycles as u16),3,u32::from(reset),u32::from(follow),u32::from(follow)]);}assert_eq!(h,2712267879);}
 #[test]fn native_4096_viewport_queries_match_wrapped_signed_coordinates_and_asymmetric_padding(){let mut h=2166136261;for s in 0..4096i32{let outside=outside_view([(s*353-32000)as i16,(s*229+30000)as i16],[(s*521-31000)as i16,(s*907+32000)as i16],[[320,640,-1,32767][s as usize%4],[200,480,-48,32767][(s/4%4)as usize]]);mix(&mut h,[s as u32,u32::from(outside)]);}assert_eq!(h,658326758);}
 #[test]fn native_384_portrait_wrapper_callbacks_match_single_double_pass_and_busy_release(){let mut h=2166136261;let mut scenario=0;for message in UNDER_ATTACK.into_iter().chain([CANNOT_REACH]){for twice in[false,true]{let mut s=State{portrait_mode:4,..Default::default()};s.select_voice(voice(message).unwrap().1);for _ in 0..32{s.advance_phrase();if twice&&s.portrait_mode!=0{s.advance_phrase();}mix(&mut h,[scenario,u32::from(s.portrait_mode),u32::from(s.phrase_index as u16),u32::from(s.phrase_counter as u16),u32::from(s.phrase_frame as u16),1]);scenario+=1;}}}assert_eq!(h,863143744);}
 #[test]fn native_512_voice_descriptor_gates_match_signed_clock_wrap_and_same_tick_suppression(){let mut h=2166136261;for s in 0..512i32{let now=(s*997)as i16;let last=if s%7==0{now}else if s%11==0{now.wrapping_sub(i16::MIN)}else{(s*991)as i16};let mut f=State{ui_clock:now,sound_enabled:1,..Default::default()};f.last_voice_ticks[0]=last;let played=f.sound_admitted(0);mix(&mut h,[s as u32,u32::from(!played),u32::from(f.last_voice_ticks[0]as u16),u32::from(played)]);}assert_eq!(h,2463348233);}
 #[test]fn native_512_attacked_group_lists_match_distinct_groups_signed_sort_and_membership(){let mut h=2166136261;for s in 0..512i32{let units=(0..240i32).map(|i|AlertUnit{slot:i as u8,group:if i%13==0{255}else{((i*19+s)%73)as u8},flags31:(4+i/3%4)as u8|if i%17==0{16}else{0},flags69:if(i+s)%5<2{128}else{0},position:[(s*17-32000+i*101)as i16,(s*23+30000-i*199)as i16]}).collect::<Vec<_>>();let mut membership=[0;240];let list=attacked_groups(&units,(s%4)as u8,[(s*31)as i16,(s*47)as i16],&mut membership).unwrap();let mut lh=2166136261;for &(_,slot,distance)in &list{mix(&mut lh,[u32::from(slot),distance as u32]);}let mut mh=2166136261;mix(&mut mh,membership.map(u32::from));mix(&mut h,[s as u32,list.len()as u32,lh,mh,0]);}assert_eq!(h,4167780462);}
}
