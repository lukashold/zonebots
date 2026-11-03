//! Original multi-group selection rows and destination dispersal.
//! Geometry queries are explicit inputs: this module does not replace the
//! native click classifier with browser order names.
pub const COUNT:usize=240;
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Row{pub primary:u8,pub members:[u8;COUNT]}
impl Default for Row{fn default()->Self{Self{primary:255,members:[255;COUNT]}}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct SourceContext{pub icon_a8:Option<u16>,pub link28:Option<u32>,pub saved_class8e:Option<u8>}
#[derive(Clone,Debug)]pub struct State{pub rows:Vec<Row>,pub insertion:u8,pub used:[u8;COUNT],pub contexts:Vec<SourceContext>}
impl Default for State{fn default()->Self{Self{rows:vec![Row::default();COUNT],insertion:0,used:[0;COUNT],contexts:vec![SourceContext::default();COUNT]}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Found{pub row:usize,pub member:usize}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum AddResult{Added,Rejected,SignedInsertionAddress}
impl State{
 ///43ad60 compares an unsigned group byte to a SIGNED primary byte, then
 ///searches all240 member bytes. A header match and member0 both return0.
 pub fn find(&self,group:u8)->Option<Found>{if group==255{return None;}for(row,r)in self.rows.iter().enumerate().take(COUNT){if r.primary==255{continue;}if i32::from(group)==i32::from(r.primary as i8){return Some(Found{row,member:0});}if let Some(member)=r.members.iter().position(|&x|x==group){return Some(Found{row,member});}}None}
 ///43ac70 removes the first occurrence. Header replacement is the LAST
 ///remaining member, and a one-member row dissolves instead of compacting.
 pub fn remove_from(&mut self,row:usize,group:u8)->u8{let Some(r)=self.rows.get_mut(row)else{return 255;};let Some(i)=r.members.iter().position(|&x|x==group)else{return 255;};r.members[i]=255;if i32::from(r.primary as i8)==i32::from(group){r.primary=255;for &x in &r.members{if x!=255{r.primary=x;}}}let n=r.members.iter().filter(|&&x|x!=255).count();if n<2{let remaining=if n==1{r.primary}else{255};*r=Row::default();remaining}else{255}}
 pub fn remove(&mut self,group:u8)->u8{self.find(group).map_or(255,|f|self.remove_from(f.row,group))}
 ///43ade0 walks the mutating member bytes in place.
 pub fn dissolve_containing(&mut self,group:u8){if let Some(f)=self.find(group){for i in 0..COUNT{let x=self.rows[f.row].members[i];if x!=255{self.remove_from(f.row,x);}}}}
 ///43ab30. `uniform` is the real54070||540b0 result for the queried roster.
 ///The native signed insertion-byte address below the pool is not emulated.
 pub fn add(&mut self,group:u8,class:u8,family:u8,uniform:bool)->AddResult{if class==2||family==36||!uniform{return AddResult::Rejected;}if let Some(f)=self.find(group){self.rows[f.row].primary=group;return AddResult::Added;}if self.insertion!=255{if self.insertion>=128{return AddResult::SignedInsertionAddress;}let r=&mut self.rows[usize::from(self.insertion)];if let Some(i)=r.members.iter().position(|&x|x==255){r.members[i]=group;r.primary=group;return AddResult::Added;}return AddResult::Rejected;}
  for(i,r)in self.rows.iter_mut().enumerate().take(COUNT){self.insertion=i as u8;if r.primary==255{*r=Row::default();r.primary=group;r.members[0]=group;return AddResult::Added;}}AddResult::Rejected}
 ///37ac0 removes all groups from their old rows before rebuilding selection.
 pub fn select(&mut self,groups:&[(u8,u8,u8,bool)]){if groups.is_empty(){return;}for &(g,_,_,_)in groups{self.remove(g);}self.insertion=255;for &(g,c,f,u)in groups{self.add(g,c,f,u);}}
 ///43aaf0 selects a leader as the primary through the lookup's member index.
 pub fn promote(&mut self,group:u8,role:i8){if role== -1{if let Some(f)=self.find(group){self.rows[f.row].primary=self.rows[f.row].members[f.member];}}}
 ///43a9f0's ordered candidate roster IDs. Its reaction handler and recursive
 ///45ef00 remain explicit caller events, rather than a fresh selection scan.
 pub fn broadcast_groups(&mut self,group:u8,role:i8)->Vec<u8>{if role==0{return vec![];}let Some(f)=self.find(group)else{return vec![];};self.used.fill(0);self.rows[f.row].members.iter().copied().filter(|&x|x!=255).collect()}
 ///460430 clears rows. The separate43a9f0 mask and5849fa scratch are not reset.
 pub fn clear_rows_for_mission(&mut self){self.rows.fill(Row::default());self.contexts.fill(SourceContext::default());}
 ///43a600 retries the SAME blocked offset until its1200-iteration limit.
 ///41c290 may adjust the local point before43a520; accepted globalXY still
 ///adds the original offset. Used and negative candidates advance the index.
 pub fn disperse(&mut self,group:u8,class:u8,opcode:i8,click:[i16;2],fort:[i16;2],mut building:impl FnMut(&mut [i16;2])->Option<i8>,mut blocked:impl FnMut([i16;2])->bool)->Dispersal{
  let mut result=Dispersal{point:click,index:None,iterations:0,active:false};if fort[0]>=click[0]&&fort[0]<=click[0].wrapping_add(160)&&fort[1]>=click[1]&&fort[1]<=click[1].wrapping_add(160){return result;}if self.find(group).is_none_or(|f|f.member==0){return result;}result.active=true;let start=if class==1{9}else{0};let mut index=start;
  for _ in 0..1200{result.iterations+=1;let raw=OFFSETS[index];let delta=if class==1{[raw[0].wrapping_mul(2),raw[1].wrapping_mul(2)]}else{raw};let candidate=[click[0].wrapping_add(delta[0]),click[1].wrapping_add(delta[1])];if self.used[index]!=0||candidate.iter().any(|&x|x<0){index+=1;if index==COUNT{self.used.fill(0);index=start;}continue;}let mut local=candidate;if !(opcode==1&&building(&mut local)==Some(0))&&!blocked(local){self.used[index]=1;result.point=candidate;result.index=Some(index);return result;}}
  result
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Dispersal{pub point:[i16;2],pub index:Option<usize>,pub iterations:usize,pub active:bool}
///43a520 bounds, APC point query and437900's family-specific footprint dispatch.
///A true result means blocked. APC query uses forced metadata extent mode.
pub fn destination_blocked(class:u8,family:u8,point:[i16;2],bounds:[i16;2],mut apc_at:impl FnMut([i16;2])->bool,mut terrain_blocked:impl FnMut([i16;2],bool)->bool)->bool{
 let in_bounds=|p:[i16;2]|p[0]>=8&&p[1]>=8&&p[0]<=bounds[0]&&p[1]<=bounds[1];if !in_bounds(point){return true;}let mut p=[point[0].wrapping_sub(8),point[1].wrapping_sub(8)];if class==1{if !in_bounds(p){return true;}p=[p[0].wrapping_sub(8),p[1].wrapping_sub(8)];}else if apc_at(point){return true;}terrain_blocked(p,matches!(family,6|25|30|24|32|34|36|38))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ClickQuery{Flag,Apc,Building,Snap}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum MoveSource{Flag,Apc,Building,Snap,Terrain}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct MoveClick{pub point:[i16;2],pub source:MoveSource}
///4416b0's move-click destination classifier. A successful Flag/APC lookup
///replaces the point, BUILD/Snap mutate it, and the terrain check is only used
///when both of the latter fail. The source APC lookup receives(clickY,clickY).
pub fn move_click_destination(class:u8,actor:[i16;2],click:[i16;2],flag_mode:bool,mut query:impl FnMut(ClickQuery,&mut [i16;2])->bool,mut terrain_blocked:impl FnMut([i16;2])->bool)->Option<MoveClick>{
 let mut p=click;if class!=1{if flag_mode&&query(ClickQuery::Flag,&mut p){return Some(MoveClick{point:p,source:MoveSource::Flag});}p=[click[1],click[1]];if query(ClickQuery::Apc,&mut p){let offset=if actor==p{16}else{8};return Some(MoveClick{point:p.map(|x|x.wrapping_add(offset)),source:MoveSource::Apc});}}
 p=click;if query(ClickQuery::Building,&mut p){return Some(MoveClick{point:p,source:MoveSource::Building});}if query(ClickQuery::Snap,&mut p){return Some(MoveClick{point:p,source:MoveSource::Snap});}if terrain_blocked(p){None}else{Some(MoveClick{point:p,source:MoveSource::Terrain})}
}
///4387b0, ordered as the fixed d54,d58,d5c,d60 base globals.
pub fn fort_snap(code:u8,forts:[[i16;2];4])->Option<[i16;2]>{let i=match code&127{20=>1,21=>0,60=>2,61=>3,_=>return None};Some([forts[i][0].wrapping_add(80),forts[i][1].wrapping_add(64)])}
///41cbd0's first raw flag record. The returned position has no center offset.
pub fn flag_click(point:[i16;2],flags:&[[i16;2]])->Option<[i16;2]>{flags.iter().take(64).copied().find(|flag|flag[0]!= -1&&(0..2).all(|a|point[a].wrapping_sub(8)<=flag[a].wrapping_add(if a==0{4}else{8})&&flag[a].wrapping_add(if a==0{4}else{8})<=point[a].wrapping_add(8)))}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Command{pub opcode:i8,pub args:[i16;3]}
#[derive(Clone,Copy,Debug)]pub struct ReactionInput{pub command:Command,pub source_family:u8,pub source_icon:u16,pub family:u8,pub class:u8,pub saved_class:u8,pub grenade_total_low:u8,pub ground_code:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Reaction{pub outer:Command,pub recursive:Option<Command>,pub handled:bool,pub icon:Option<u16>,pub copy_context:bool}
const CLASSIFICATION:[u8;20]=[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5,4,3,2,0];
const FAMILIES:[u8;39]=[0,0,0,0,0,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,8,0,0,0,0,6,0,13,0,11,0,10,0,12];
const REACTIONS:[[u8;16];6]=[
 [0,0,0,0,0,0,0,0,0,0,3,3,3,3,0,0], //494c64
 [3,3,3,3,3,3,3,3,3,3,3,3,6,3,0,0], //494c74
 [5,5,5,5,5,5,0,0,0,0,4,0,4,0,0,0], //494c84
 [0,3,3,3,3,3,0,0,0,0,3,3,3,3,0,0], //494c94
 [0,0,0,0,0,0,2,2,2,2,1,2,1,2,0,0], //4951f4
 [3,3,3,3,3,0,3,3,3,3,3,3,3,3,0,0], //495204
];
///43a830/43a7a0 and the three recursive reaction wrappers. The two ground
///queries return4093e0 bridge-hit and41c290 BUILD-hit and may adjust the raw
///ground point. Leader lookup is by native group slot, not browser entity ID.
pub fn broadcast_reaction(input:ReactionInput,mut ground_query:impl FnMut(&mut [i16;2])->(bool,bool),mut leader:impl FnMut(i16)->Option<u8>)->Option<Reaction>{
 let mut r=Reaction{outer:input.command,recursive:None,handled:false,icon:None,copy_context:false};let normal_icon=||Some(if matches!(input.source_icon,0xacb|0xac3|65535){0xabb}else{input.source_icon});if r.outer.opcode==1{r.icon=normal_icon();return Some(r);}
 let mut kind=if input.family==4{*CLASSIFICATION.get(usize::from(input.class))?}else{*FAMILIES.get(usize::from(input.family))?};let table=match r.outer.opcode{2=>2,5=>{if r.outer.args[0]!=3456{4}else if input.source_family==38{1}else if input.class==15&&matches!(input.ground_code&127,59|51){5}else{let mut hit=input.family==4&&input.grenade_total_low!=0;if input.class==15&&(input.family!=4||input.grenade_total_low==0){let mut p=[r.outer.args[1],r.outer.args[2]];let(a,b)=ground_query(&mut p);r.outer.args[1]=p[0];r.outer.args[2]=p[1];hit=a||b;}
   if kind==13{kind=*CLASSIFICATION.get(usize::from(input.saved_class))?;hit=false;if kind==5{let mut p=[r.outer.args[1],r.outer.args[2]];let(a,b)=ground_query(&mut p);r.outer.args[1]=p[0];r.outer.args[2]=p[1];hit=a||b;}}if hit{0}else{3}}},_=>return Some(r)};
 let action=*REACTIONS[table].get(usize::from(kind))?;match action{
  1|2=>{r.recursive=Some(Command{opcode:2,args:[-1234,(-1i16).wrapping_sub(r.outer.args[0]),r.outer.args[2]]});r.handled=true;r.icon=Some(0xadf);},
  3=>{r.recursive=Some(Command{opcode:1,args:[r.outer.args[1],r.outer.args[2],2]});r.handled=true;r.icon=Some(0xabb);},
  5=>{r.handled=true;if let Some(slot)=leader(r.outer.args[1]).filter(|&x|x!=255){r.recursive=Some(Command{opcode:5,args:[i16::from(slot),r.outer.args[1],0]});r.icon=Some(0xadf);}},
  6=>r.copy_context=true,
  _=>r.icon=normal_icon(),
 }Some(r)
}
//494cc0, including the original repeated entries and table-tail values.
include!("original_selection_offsets.rs");
#[cfg(test)]mod tests{
 use super::*;
 fn mix(h:&mut u32,n:i64){*h=(*h^(n as u32)).wrapping_mul(16777619);}
 fn checksum(s:&State)->u32{let mut h=2166136261;for r in &s.rows{mix(&mut h,i64::from(r.primary));for &x in &r.members{mix(&mut h,i64::from(x));}}h}
 #[test]fn selection_rows_match_256_native_mutation_runs(){let mut h=2166136261;for s in 0..256usize{let mut a=State::default();for row in 0..4{a.rows[row].primary=((s+row*67)%240)as u8;for j in 0..5{a.rows[row].members[j*7]=((s+row*31+j*13)%240)as u8;}}let found=a.find(((s*17)%256)as u8);let remaining=a.remove_from(s%4,((s+13)%240)as u8);let removed=checksum(&a);a.promote(((s+31)%240)as u8,-1);let promoted=checksum(&a);a.insertion=if s%3==0{255}else{5};let added=a.add(((s*23)%240)as u8,if s%9==0{2}else{0},if s%11==0{36}else{4},true)==AddResult::Added;for n in [s as i64,found.is_some()as i64,found.map_or(-1,|f|f.row as i64),found.map_or(-1,|f|f.member as i64),i64::from(remaining),i64::from(removed),i64::from(promoted),added as i64,i64::from(a.insertion),i64::from(checksum(&a))]{mix(&mut h,n);}}assert_eq!(h,2677714466,"real43ad60/ac70/ab30/aaf0, no stubs");}
 #[test]fn dispersal_matches_128_native_runs_with_explicit_spatial_queries(){let mut h=2166136261;for s in 0..128usize{let mut a=State::default();a.rows[0].primary=3;a.rows[0].members[..3].copy_from_slice(&[3,7,9]);for i in 0..COUNT{a.used[i]=u8::from(s%17==0||i<s%25);}let click=[if s%13==0{-20}else{200+(s%5)as i16},300+(s%7)as i16];let fort=if s%11==0{[click[0]+100,click[1]+100]}else{[1000,1000]};let(mut b,mut p)=(0,0);let r=a.disperse(if s%7==0{3}else{7},u8::from(s%3==0),if s%2==0{1}else{5},click,fort,|point|{b+=1;let hit=s%19==0||b<=s%3;if !hit{point[0]=point[0].wrapping_add((s%5)as i16);point[1]=point[1].wrapping_sub((s%4)as i16);}hit.then_some(0)},|_|{p+=1;s%23==0||p<=s%4});let mut mask=2166136261;for &x in &a.used{mix(&mut mask,i64::from(x));}for n in [s as i64,i64::from(r.point[0]),i64::from(r.point[1]),b as i64,p as i64,i64::from(mask)]{mix(&mut h,n);}}assert_eq!(h,3145486217,"real43a600/43ad60, only41c290/43a520 query boundaries");}
 #[test]fn blocked_offsets_retry_without_silently_trying_a_different_slot(){let mut s=State::default();s.select(&[(3,0,4,true),(7,0,4,true),(9,0,4,true)]);let mut probes=vec![];let r=s.disperse(7,0,1,[200,300],[1000,1000],|_|None,|p|{probes.push(p);true});assert_eq!(r.iterations,1200);assert_eq!(probes,vec![[200,316];1200]);assert!(r.index.is_none());assert_eq!(s.used,[0;240]);}
 #[test]fn header_sign_and_last_remaining_primary_are_preserved(){let mut s=State::default();s.rows[0].primary=200;assert!(s.find(200).is_none());s.rows[0].members[8]=200;assert_eq!(s.find(200),Some(Found{row:0,member:8}));s.rows[0].primary=3;s.rows[0].members[0]=3;s.rows[0].members[2]=9;s.remove_from(0,3);assert_eq!(s.rows[0].primary,200);assert_eq!(s.remove_from(0,9),200);assert_eq!(s.rows[0],Row::default());}
 #[test]fn source_bounds_and_family_footprint_dispatch_use_different_anchors(){let mut point=None;assert!(!destination_blocked(1,25,[100,100],[1000,1000],|_|panic!("class1 skips APC query"),|p,wide|{point=Some((p,wide));false}));assert_eq!(point,Some(([84,84],true)));assert!(destination_blocked(0,4,[100,100],[1000,1000],|p|p==[100,100],|_,_|panic!("APC already blocks")));assert!(destination_blocked(1,25,[8,8],[1000,1000],|_|false,|_,_|false));}
 #[test]fn move_click_classifier_matches_256_real_native_branch_sequences(){let mut hash=2166136261;for s in 0..256usize{let mut queries=0;let mut terrain=0;let mut apc=[-999,-999];let r=move_click_destination(if s%4==0{1}else{0},if s%3==0{[500,600]}else{[50,60]},[200+s as i16,300+s as i16],s&1!=0,|kind,p|{let tag=match kind{ClickQuery::Flag=>1,ClickQuery::Apc=>2,ClickQuery::Building=>3,ClickQuery::Snap=>4};queries=queries*10+tag;match kind{ClickQuery::Flag=>{if s&2!=0{*p=[700,800];true}else{false}},ClickQuery::Apc=>{apc=*p;if s&4!=0{*p=[500,600];true}else{false}},ClickQuery::Building=>{p[0]+=3;p[1]-=5;s&8!=0},ClickQuery::Snap=>{p[0]-=7;p[1]+=11;s&16!=0}}},|_|{terrain+=1;s&32!=0});let point=r.map_or([-999,-999],|r|r.point);for n in [s as i64,r.is_some()as i64,i64::from(point[0]),i64::from(point[1]),queries,i64::from(terrain),i64::from(apc[0]),i64::from(apc[1])]{mix(&mut hash,n);}}assert_eq!(hash,821751069,"real4416b0; query outcomes/45ef00/UI feedback are recorded boundaries");}
 #[test]fn broadcast_reactions_match_1024_native_dispatch_runs(){let mut hash=2166136261;for s in 0..1024usize{let family=[4,6,24,25,30,32,34,36,38][s%9];let class=[0,15,16,17,18][s/9%5];let saved_class=[0,15,16,17,18][s/45%5];let command=Command{opcode:[1,2,5,4][s/225%4],args:[if s%2==0{3456}else{-1-(s%100)as i16},(s%240)as i16,300+(s%30)as i16]};let input=ReactionInput{command,source_family:if s%7==0{38}else{4},source_icon:[0xacb,0xac3,65535,111][s%4],family,class,saved_class,grenade_total_low:(s*13)as u8,ground_code:[0,51,59][s/7%3]};let mut query=0;let r=broadcast_reaction(input,|p|{query+=1;p[0]+=3;p[1]-=5;(s&2!=0,s&4!=0)},|g|Some(if g==0{0}else if s%11==0{255}else{((g as usize*7)%240)as u8})).unwrap();let e=r.recursive.unwrap_or(Command{opcode:0,args:[0;3]});let values=[s as i64,r.handled as i64,query,r.recursive.is_some()as i64,i64::from(e.opcode),i64::from(e.args[0]),i64::from(e.args[1]),i64::from(e.args[2]),r.icon.map_or(-1,i64::from),i64::from(r.outer.args[0]),i64::from(r.outer.args[1]),i64::from(r.outer.args[2]),if r.copy_context{0x12345678}else{0x98765432}];for n in values{mix(&mut hash,n);}}assert_eq!(hash,1712449762,"real43a830/43a7a0/43a360/43a3f0/43a480/11b90; geometry and recursive45ef00 explicit");}
}
