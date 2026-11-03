//! Native local contour walkers0043c8d0/0043d130/2b0/430/5b0. The caller
//! supplies the dynamic permission plane and the original target-ray marks.
//! No graph-search heuristic or estimated edge price appears in this walker.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Edge{North,South,East,West}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Progress{Pending,Complete,Failed}
#[derive(Clone,Debug)]
pub struct Contour{
 pub cell:i32,pub wall:i32,pub step:i32,pub steps:u32,pub scratch:i32,
 pub edge:Edge,pub points:Vec<i16>,pub score:u8,pub complete:bool,
}
impl Contour{
 pub fn new(cell:i32,wall:i32,step:i32,steps:u32,edge:Edge)->Self{Self{cell,wall,step,steps,scratch:0,edge,points:Vec::new(),score:0,complete:false}}
 fn append(&mut self)->bool{
  let cell=self.cell as i16;
  if self.points.last()==Some(&cell){self.steps=self.steps.wrapping_sub(256)&65535;return false;}
  if self.points.len()>1&&self.points[self.points.len()-2]==cell{self.points.pop();self.steps=self.steps.wrapping_sub(512)&65535;self.score=(self.steps>>8)as u8;return false;}
  //0043c270 asks the separate world-aware line simplifier before exceeding
  //48 points. This pure walker exposes failure at that boundary.
  if self.points.len()>47{return true;}
  self.points.push(cell);self.score=(self.steps>>8)as u8;self.points.len()>47
 }
 fn finish(&mut self){self.complete=true;}
 pub fn count(&self)->i8{if self.complete{-(self.points.len()as i8)}else{self.points.len()as i8}}
 fn index(&self)->Option<usize>{usize::try_from(self.cell).ok().filter(|&p|p<16384)}
 pub fn foot(&mut self,mask:&mut[u8],mut open:impl FnMut(i32)->bool)->Progress{
  for _ in 0..40{
   self.cell+=self.step;self.wall+=self.step;
   let Some(index)=self.index().filter(|&p|p<mask.len())else{return Progress::Failed;};if self.steps>63999{return Progress::Failed;}
   let old;
   if !open(self.cell){
    self.cell-=self.step;self.wall-=self.step;if self.append(){return Progress::Failed;}
    self.scratch=self.cell-self.wall;self.wall=self.step+self.cell;self.step=self.scratch;continue;
   }else if !open(self.wall){old=mask[index];mask[index]=(self.steps>>8)as u8;self.steps=self.steps.wrapping_add(256);}
   else{
    self.wall-=self.step;if self.append(){return Progress::Failed;}
    let inside=self.cell-self.step;let next=self.wall-inside;self.wall-=next;self.step=next;self.scratch=inside;
    old=mask[index];mask[index]=(self.steps>>8)as u8;self.steps=self.steps.wrapping_add(256);
   }
   if old==255{mask[index]=255;self.append();self.finish();return Progress::Complete;}
  }
  Progress::Pending
 }
 fn blocked_turn(&mut self){
  let less=self.cell<self.wall;
  match self.edge{
   Edge::North=>{self.wall=self.cell-128;self.step=if less{-1}else{1};self.edge=if less{Edge::West}else{Edge::East};},
   Edge::South=>{self.wall=self.cell+256;self.step=if less{-1}else{1};self.edge=if less{Edge::West}else{Edge::East};},
   Edge::East=>{self.wall=self.cell+2;self.step=if less{-128}else{128};self.edge=if less{Edge::North}else{Edge::South};},
   Edge::West=>{self.wall=self.cell-1;self.step=if less{-128}else{128};self.edge=if less{Edge::North}else{Edge::South};},
  }
 }
 fn open_turn(&mut self){
  let less=self.cell<self.wall;
  match self.edge{
   Edge::North=>{self.wall=self.cell+256;self.step=if less{1}else{-1};self.edge=if less{Edge::East}else{Edge::West};},
   Edge::South=>{self.wall=self.cell-128;self.step=if less{1}else{-1};self.edge=if less{Edge::East}else{Edge::West};},
   Edge::East=>{self.wall=self.cell-1;self.step=if less{128}else{-128};self.edge=if less{Edge::South}else{Edge::North};},
   Edge::West=>{self.wall=self.cell+2;self.step=if less{128}else{-128};self.edge=if less{Edge::South}else{Edge::North};},
  }
 }
 pub fn vehicle(&mut self,mask:&mut[u8],mut open:impl FnMut(i32)->bool)->Progress{
  let mut moves=0;let mut budget=75;
  loop{
   budget-=1;if moves>39||budget<0{return Progress::Pending;}
   self.cell+=self.step;self.wall+=self.step;
   let Some(index)=self.index().filter(|&p|p+129<mask.len())else{return Progress::Failed;};if self.steps>63999{return Progress::Failed;}
   let leading=match self.edge{Edge::North=>[self.cell,self.cell+1],Edge::South=>[self.cell+128,self.cell+129],Edge::East=>[self.cell+1,self.cell+129],Edge::West=>[self.cell,self.cell+128]};
   if leading.into_iter().any(|p|!open(p)){
    self.cell-=self.step;self.wall-=self.step;if self.append(){return Progress::Failed;}self.blocked_turn();continue;
   }
   let adjacent=match self.edge{Edge::North|Edge::South=>[self.wall,self.wall+128],Edge::East|Edge::West=>[self.wall,self.wall+1]};
   if adjacent.into_iter().all(&mut open){self.append();self.open_turn();continue;}
   for offset in [0,1,128,129]{let old=mask[index+offset];mask[index+offset]=(self.steps>>8)as u8;if old==255{mask[index]=255;mask[index+128]=255;self.append();self.finish();return Progress::Complete;}}
   self.steps=self.steps.wrapping_add(256);moves+=1;
  }
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn hash(h:&mut u32,ns:&[i32]){for n in ns{*h=(*h^*n as u32).wrapping_mul(16777619);}}
 #[test]fn all_55_native_foot_and_vehicle_contour_callbacks_match_including_visited_masks(){
  let mut h=2166136261;let mut rows=0;
  for vehicle in 0..2{for mode in 0..3{for direction in 0..4{
   let mut map=vec![0u8;16384];let mut mask=vec![65u8;16384];
   for y in 0..128{for x in 0..128{let blocked=(30..=34).contains(&x)&&(20..=40).contains(&y)||mode==1&&(30..=45).contains(&x)&&(40..=41).contains(&y)||mode==2&&(35..=38).contains(&x)&&(15..=25).contains(&y);map[y*128+x]=u8::from(blocked);if if direction<2{(29..=31).contains(&y)&&(35..=70).contains(&x)}else{(31..=33).contains(&x)&&(42..=70).contains(&y)}{mask[y*128+x]=255;}}}
   let cell=if direction<2{if vehicle==1{28}else{29}}else{32}+if direction<2{30}else if vehicle==1{18}else{19}*128;
   let wall=if direction<2{30+30*128}else{32+20*128};let step=[-128,128,1,-1][direction];let edge=[Edge::North,Edge::South,Edge::East,Edge::West][direction];let mut s=Contour::new(cell,wall,step,256,edge);mask[cell as usize]=0;
   for tick in 1..=20{let open=|j:i32|usize::try_from(j).ok().and_then(|i|map.get(i)).is_some_and(|&code|code==0);let progress=if vehicle==1{s.vehicle(&mut mask,open)}else{s.foot(&mut mask,open)};let status=i32::from(progress==Progress::Pending);let edge=match s.edge{Edge::North=>0,Edge::South=>1,Edge::East=>2,Edge::West=>3};let mut mh=2166136261u32;for b in &mask{mh=(mh^u32::from(*b)).wrapping_mul(16777619);}
    hash(&mut h,&[vehicle,mode,direction as i32,tick,status,s.cell,s.wall,s.step,s.steps as i32,edge,i32::from(s.score),i32::from(s.count())]);for p in &s.points{hash(&mut h,&[i32::from(*p)]);}hash(&mut h,&[mh as i32]);rows+=1;if status==0{break;}
   }
  }}}
  assert_eq!(rows,55);assert_eq!(h,2089043093);
 }
}

///004517c0/0043cb20 create a target-ray marking plane. The callback is the
/// raw unit+18 permission test (including native code30's explicit override).
/// side_grace is the raw00586862 mode; the center starts with grace2.
pub fn target_mask(start:(i16,i16),goal:(i16,i16),vehicle:bool,alternate:bool,side_grace:u8,mut open:impl FnMut(i32)->bool)->Vec<u8>{
 let mut mask=vec![65u8;16384];let goal_index=i32::from(goal.0>>4)+i32::from(goal.1>>4)*128;if let Ok(i)=usize::try_from(goal_index){if let Some(v)=mask.get_mut(i){*v=255;}}
 let offset=if vehicle&&alternate{(-7,-8)}else{(8,8)};
 let(mut x,mut y)=((i32::from(start.0)+offset.0)&!15,(i32::from(start.1)+offset.1)&!15);let(gx,gy)=((i32::from(goal.0)+8)&!15,(i32::from(goal.1)+8)&!15);
 let dx=(gx-x).abs();let dy=(gy-y).abs();let sx=if gx<x{-1}else{1};let sy=if gy<y{-1}else{1};let ymajor=dx<dy;let length=dx.max(dy);let mut error=length>>1;let mut remaining=length+16;let mut grace=[2,side_grace,side_grace];
 loop{
  let cell=(x>>4)+(y>>4)*128;if cell<128||cell>=16384{break;}
  let mut mark=|p:i32|{if let Ok(i)=usize::try_from(p){if let Some(v)=mask.get_mut(i){*v=255;}}};
  if open(cell){if grace[0]==0{mark(cell);if vehicle{mark(cell+1);mark(cell+128);mark(cell+129);}}}else{grace[0]=0;}
  for(side,delta)in [if ymajor{1}else{128},if ymajor{-1}else{-128}].into_iter().enumerate(){let p=cell+delta;if open(p){if grace[side+1]==0{mark(p);}}else{grace[side+1]=0;}}
  if remaining<1{break;}
  if ymajor{y+=16*sy;for _ in 0..16{error-=dx;if error<0{error+=dy;x+=sx;}}}else{x+=16*sx;for _ in 0..16{error-=dy;if error<0{error+=dx;y+=sy;}}}
  remaining-=16;for n in &mut grace{if *n>0{*n-=1;}}
 }
 mask
}

#[cfg(test)]mod mask_tests{
 use super::*;
 #[test]fn all_192_native_goal_ray_masks_match_pixel_raster_grace_and_vehicle_offset(){
  let mut h=2166136261u32;let starts=[(464,512),(513,525),(512,304)];let goals=[(648,520),(648,648),(168,520),(512,824)];
  for vehicle in 0..2{for alternate in 0..2{for grace in [0,1,2,127]{for(si,&s)in starts.iter().enumerate(){for(gi,&g)in goals.iter().enumerate(){
   let open=|j:i32|{let(x,y)=(j%128,j/128);!((30..=34).contains(&x)&&(20..=40).contains(&y))};let mask=target_mask(s,g,vehicle!=0,alternate!=0,grace,open);let mut mh=2166136261u32;for b in mask{mh=(mh^u32::from(b)).wrapping_mul(16777619);}for n in [vehicle,alternate,i32::from(grace),si as i32,gi as i32,0,mh as i32]{h=(h^n as u32).wrapping_mul(16777619);}
  }}}}}
  assert_eq!(h,2207505941);
 }
}

/// Native local-search seed functions0043c4c0/5a0/680/770 and vehicle
///0043da20/7f0/8a0/960. Contact values are unit+58/+5a tile indices.
pub fn seed(start_cell:i32,contact_x:Option<i16>,contact_y:Option<i16>,direction:usize,vehicle:bool,previous_steps:u32,mask:&mut[u8],mut open:impl FnMut(i32)->bool)->Option<Contour>{
 let contact=if direction<2{contact_x}else{contact_y}.filter(|&p|p>0)?;let wall=i32::from(contact);let edge=[Edge::North,Edge::South,Edge::East,Edge::West][direction];let step=[-128,128,1,-1][direction];
 let mut s=Contour::new(start_cell,wall,step,previous_steps,edge);
 if vehicle{
  s.cell=if direction<2{(wall&!127)+(start_cell&127)}else{(start_cell&!127)|(wall&127)};
  if direction>=2&&s.cell<wall&&s.cell==wall-128{s.cell-=128;}
  s.steps=256;
 }else{
  fn adjust(s:&mut Contour,wall:i32,stride:i32,side:i32,nudge:bool,open:&mut impl FnMut(i32)->bool)->bool{
   s.wall=wall;let minus=wall-stride;let plus=wall+stride;
   let goal=if s.cell==minus{if !nudge{return true;}minus}else if s.cell==plus{if !nudge{return true;}plus}
    else if open(minus)&&(s.cell==minus-side||s.cell==minus+side){minus}
    else if open(plus)&&(s.cell==plus-side||s.cell==plus+side){plus}else{return false;};
   s.cell=goal;s.steps&=0xffff00ff;s.append();true
  }
  let(stride,side)=if direction<2{(1,128)}else{(128,1)};
  if !adjust(&mut s,wall,stride,side,false,&mut open){s.cell+=side;if !adjust(&mut s,wall,stride,side,true,&mut open){s.cell-=side*2;if !adjust(&mut s,wall,stride,side,true,&mut open){return None;}}}
  s.steps=if direction==0{256}else if direction==1{1}else{(s.steps&0xffff00ff)+256};
 }
 if let Some(value)=usize::try_from(s.cell).ok().and_then(|i|mask.get_mut(i)){*value=0;}
 Some(s)
}

///0043e4a0/0043dad0 schedule N,S,E,W on one shared marking plane and one
/// persistent native step counter. Countdown20 permits19 walker callbacks.
/// Header selection remains0043e190's strict first-lowest completed candidate.
pub fn search(start_cell:i32,contact_x:Option<i16>,contact_y:Option<i16>,vehicle:bool,mask:&mut[u8],mut open:impl FnMut(i32)->bool)->[Option<Contour>;4]{
 search_with_steps(start_cell,contact_x,contact_y,vehicle,0,mask,&mut open).0
}
pub fn search_with_steps(start_cell:i32,contact_x:Option<i16>,contact_y:Option<i16>,vehicle:bool,mut steps:u32,mask:&mut[u8],mut open:impl FnMut(i32)->bool)->([Option<Contour>;4],u32){
 let candidates=std::array::from_fn(|direction|{
  let mut contour=seed(start_cell,contact_x,contact_y,direction,vehicle,steps,mask,&mut open)?;
  for _ in 0..19{let result=if vehicle{contour.vehicle(mask,&mut open)}else{contour.foot(mask,&mut open)};if result!=Progress::Pending{break;}}
  steps=contour.steps;Some(contour)
 });(candidates,steps)
}

///0043c1b0 greedily removes one middle vertex per successful native dummy
/// walk, keeping the same preceding vertex for the next comparison.
pub fn simplify(points:&mut Vec<i16>,mut clear:impl FnMut(Option<i16>,i16)->bool){
 if points.len()<3{return;}let mut previous:Option<usize>=None;let mut count=points.len()-1;
 while count>0{let next=previous.map_or(1,|i|i+2);if next>=points.len(){break;}
  if clear(previous.map(|i|points[i]),points[next]){points.remove(previous.map_or(0,|i|i+1));}else{previous=Some(previous.map_or(0,|i|i+1));}
  count-=1;
 }
}

#[cfg(test)]mod search_tests{
 use super::*;
 #[test]fn all_48_native_local_schedules_match_shared_masks_counter_carry_and_vehicle_alternation(){
  let starts=[(464i16,512i16,Some(4126),None),(560,512,Some(4130),None),(512,304,None,Some(2592)),(512,656,None,Some(5152))];let goals=[(648,520),(648,648),(168,520)];let mut h=2166136261u32;
  for vehicle in 0..2{for previous in 0..2{for(si,&(x,y,cx,cy))in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){
   let alternate=vehicle==1&&previous==0;let offset=if alternate{(-7,-8)}else{(8,8)};let start_cell=i32::from((x+offset.0)>>4)+i32::from((y+offset.1)>>4)*128;
   let open=|j:i32|{let(x,y)=(j&127,j>>7);!((30..=34).contains(&x)&&(20..=40).contains(&y))};let mut mask=target_mask((x,y),goal,vehicle==1,alternate,127,open);let candidates=search(start_cell,cx,cy,vehicle==1,&mut mask,open);
   for n in[vehicle,previous,si as i32,gi as i32,start_cell,if vehicle==1{i32::from(alternate)}else{previous}]{h=(h^n as u32).wrapping_mul(16777619);}
   for candidate in candidates{let(score,count,points)=candidate.map(|c|(c.score,c.count(),c.points)).unwrap_or((0,0,vec![]));for n in[i32::from(score),i32::from(count)]{h=(h^n as u32).wrapping_mul(16777619);}for p in points{h=(h^p as u32).wrapping_mul(16777619);}}
  }}}}
  assert_eq!(h,3601599772);
 }
 #[test]fn all_20_native_route_simplifiers_match_reverse_dummy_walks(){
  let fixtures=[vec![2461i16,2467,4003],vec![5277,5283,4259],vec![2461,2467,5283,5277,4253],vec![4126,4127,4128,4129,4130,4131],vec![2461,2467,4003,4136]];let mut h=2166136261u32;
  for vehicle in 0..2{for mode in 0..2{for(fixture,points)in fixtures.iter().enumerate(){let mut points=points.clone();
   simplify(&mut points,|from,to|{let from=from.map(|p|super::super::segment_anchor(super::super::local_waypoint(p as u16,vehicle==1))).unwrap_or((464,512));let to=super::super::segment_anchor(super::super::local_waypoint(to as u16,vehicle==1));let c=crate::original_collision::trace(to,from,vehicle==1,|cell,_|{let(x,y)=(cell&127,cell>>7);if mode==1&&(30..=34).contains(&x)&&(20..=40).contains(&y){-1}else{0}});c.axes==0&&c.position==from});
   for n in[vehicle,mode,fixture as i32,points.len()as i32]{h=(h^n as u32).wrapping_mul(16777619);}for p in points{h=(h^p as u32).wrapping_mul(16777619);}
  }}}
  assert_eq!(h,1088102601);
 }
 #[test]fn all_24_native_vehicle_fallback_schedules_reuse_marks_counter_and_seed_origin(){
  let starts=[(464i16,512i16,Some(4126),None),(560,512,Some(4130),None),(512,304,None,Some(2592)),(512,656,None,Some(5152))];let goals=[(648,520),(648,648),(168,520)];let mut h=2166136261u32;
  for previous in 0..2{for(si,&(x,y,cx,cy))in starts.iter().enumerate(){for(gi,&goal)in goals.iter().enumerate(){
   let alternate=previous==0;let offset=if alternate{(-7,-8)}else{(8,8)};let cell=i32::from((x+offset.0)>>4)+i32::from((y+offset.1)>>4)*128;let open=|j:i32|{let(x,y)=(j&127,j>>7);!((30..=34).contains(&x)&&(20..=40).contains(&y))};let mut mask=target_mask((x,y),goal,true,alternate,127,open);let(_,steps)=search_with_steps(cell,cx,cy,true,0,&mut mask,open);let more=target_mask((x,y),goal,true,alternate,2,open);for(a,b)in mask.iter_mut().zip(more){if b==255{*a=255;}}let(candidates,_)=search_with_steps(cell,cx,cy,true,steps,&mut mask,open);
   for n in[1,previous,si as i32,gi as i32,cell,i32::from(alternate)]{h=(h^n as u32).wrapping_mul(16777619);}for candidate in candidates{let(score,count,points)=candidate.map(|c|(c.score,c.count(),c.points)).unwrap_or((0,0,vec![]));for n in[i32::from(score),i32::from(count)]{h=(h^n as u32).wrapping_mul(16777619);}for p in points{h=(h^p as u32).wrapping_mul(16777619);}}
  }}}
  assert_eq!(h,327515614);
 }
}

#[cfg(test)]mod seed_tests{
 use super::*;
 #[test]fn all_1408_native_local_seed_cases_match_contact_axes_nudges_and_counter_bytes(){
  let mut h=2166136261u32;
  for vehicle in 0..2{for direction in 0..4{for offset in [-129,-128,-127,-2,-1,0,1,2,127,128,129]{for steps in [0u32,1,256,257]{for blocked in 0..2{for valid in 0..2{
   let mut mask=vec![65u8;16384];let contact=if valid==1{Some(4112)}else{None};let s=seed(4112+offset,contact,contact,direction,vehicle!=0,steps,&mut mask,|j|blocked!=1||(j-4112).abs()>1);
   let(cell,wall,step,next_steps,score,count,points)=if let Some(s)=s{(s.cell,s.wall,s.step,s.steps,s.score,s.count(),s.points)}else{(0,0,0,0,0,0,Vec::new())};
   for n in [vehicle,direction as i32,offset,steps as i32,blocked,valid,i32::from(next_steps==0&&step==0),cell,wall,step,next_steps as i32,i32::from(score),i32::from(count)]{h=(h^n as u32).wrapping_mul(16777619);}for p in points{h=(h^p as u32).wrapping_mul(16777619);}
  }}}}}}
  assert_eq!(h,1572824157);
 }
}
