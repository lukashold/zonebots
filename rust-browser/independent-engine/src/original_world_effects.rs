//!402160/402770/4017b0 cosmetic particles. Separate from the OBJECT pool.
//!The viewport is explicit because native sprite phases advance only on draw.
#[path="original_water_spray.rs"]pub(crate) mod spray;
#[path="original_bridge_effects.rs"]pub(crate) mod bridge;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Kind{Hit,DirectHit,Destroyed,FallingObject(i16),CrateBurst,Poof{sprite:i16,maximum:i8},Fragment{sprite:i16,maximum:i8},AmbientFragment(i16)}
#[derive(Clone,Debug)]
pub struct Particle{
    pub position:[i16;3],pub origin:[i16;2],pub velocity:[i8;3],
    pub deceleration:i8,pub deceleration_left:i8,pub tint:u8,
    pub fade_left:i8,pub fade_interval:i8,pub lifetime:i16,pub flags:u8,
    pub phase:i8,pub maximum_phase:i8,pub sprite:i16,pub angle:i16,pub spin:i16,
    pub active:bool,pub expiry:i16,pub owner:u8,pub raw_tail:[u8;12],
}

#[derive(Clone,Copy,Debug)]
pub struct View{pub camera:[i16;2],pub clip:[i16;4],pub world:[i16;2],pub sine:i32,pub cosine:i32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Primitive{ColorPoint{color:u8}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Draw{pub point:[i16;2],pub screen:[i16;2],pub sprite:i16,pub height:i16,pub angle:u8,pub transformed:bool,pub owner:u8,pub primitive:Option<Primitive>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Action{
    Blast{point:[i16;2],strength:u8,suppress_visual:bool},
    DirectUnits{rect:[i16;4],strength:u8},
    Spawn{kind:Kind,origin:[i16;2],count:u8},
    RandomDestroyedBurst{origin:[i16;2]},
    Sound{point:[i16;2],id:u8},
    SetCrateSoundGate,QuietBlasts,ClearQuietBlasts,
    ProbeFalling{point:[i16;2]},ProbeCrate{point:[i16;2]},
    ProbeAmbient{point:[i16;2]},ProbeFragment{point:[i16;2],sprite:i16},
    RandomHitBurst{origin:[i16;2]},
    Retire{clear_expiry:bool},InvalidCrate,ResidualCrate,
}
#[derive(Clone,Copy,Debug)]
pub struct Step{pub draw:Option<Draw>,pub expiry:Option<i16>}
///Each successful allocation consumes6 RNG words for hit particles or3 for
///destruction. No RNG is consumed if the shared effects pool is full.
pub fn initialize(kind:Kind,origin:[i16;2],tint:u8,mut random:impl FnMut()->u32)->Particle{
    if let Kind::Fragment{sprite,maximum}=kind{
        let x=((random()&31)as i8-16)>>1;let y=((random()&31)as i8-16)>>1;random();let spin=(random()&63)as i16-32;
        return Particle{position:[0;3],origin,velocity:[x,y,-13],deceleration:1,deceleration_left:1,tint,fade_left:0,fade_interval:0,lifetime:60,flags:60,phase:0,maximum_phase:maximum,sprite,angle:0,spin,active:true,expiry:10,owner:0,raw_tail:[0;12]};
    }
    if matches!(kind,Kind::FallingObject(_)|Kind::CrateBurst|Kind::AmbientFragment(_)){
        let x=(random()&31)as i8-16;let y=(random()&31)as i8-16;let z=(if kind==Kind::CrateBurst{-5i8}else{-4i8}).wrapping_sub((random()&3)as i8);let spin=(random()&63)as i16-32;
        let velocity=if kind==Kind::CrateBurst{[(random()&6)as i8-3,(random()&7)as i8-3,z]}else{[x,y,z]};
        return Particle{position:[0;3],origin,velocity,deceleration:4,deceleration_left:4,tint,fade_left:0,fade_interval:0,lifetime:400,flags:if kind==Kind::CrateBurst{60}else{12},phase:0,maximum_phase:0,sprite:match kind{Kind::FallingObject(sprite)|Kind::AmbientFragment(sprite)=>sprite,_=>1336},angle:0,spin,active:true,expiry:match kind{Kind::CrateBurst=>6,Kind::AmbientFragment(_)=>1,_=>7},owner:0,raw_tail:[0;12]};
    }
    let(phase,velocity,deceleration,flags,sprite,maximum_phase)=match kind{
        Kind::Hit|Kind::Poof{..}=>{let phase=(random()&3)as i8;let x=(random()&15)as i8;let x=x.wrapping_add((random()&7)as i8).wrapping_sub(12);let y=(random()&15)as i8;let y=y.wrapping_add((random()&7)as i8).wrapping_sub(12);let z=(-4i8).wrapping_sub((random()&7)as i8);let(sprite,flags,maximum)=if let Kind::Poof{sprite,maximum}=kind{(sprite,12,maximum)}else{(250,4,0)};(phase,[x,y,z],2,flags,sprite,maximum)},
        Kind::DirectHit=>{let phase=(random()&3)as i8;let x=((random()&65535)%25)as i8-12;let y=((random()&65535)%25)as i8-12;let z=(-1i8).wrapping_sub((random()&3)as i8);(phase,[x,y,z],2,4,250,0)},
        Kind::Destroyed=>{let x=(random()&31)as i8-16;let y=(random()&31)as i8-16;let z=(-2i8).wrapping_sub((random()&3)as i8);(0,[x,y,z],3,12,557,5)},
        _=>unreachable!(),
    };
    Particle{position:[0;3],origin,velocity,deceleration,deceleration_left:if matches!(kind,Kind::Hit|Kind::DirectHit|Kind::Poof{..}){1}else{deceleration},flags,sprite,maximum_phase,phase,tint,fade_left:0,fade_interval:0,lifetime:200,angle:0,spin:0,active:true,expiry:0,owner:0,raw_tail:[0;12]}
}
///4027f0 uses the fixed camera471,-256,angle425,height1 installed at435e60.
pub fn project(position:[i16;3],sine:i32,cosine:i32)->[i16;2]{
    let y=i32::from(position[1])-471;let z=i32::from(position[2])+256;
    let mut depth=(sine.wrapping_mul(y)>>10).wrapping_add(cosine.wrapping_mul(z)>>10);if depth==0{depth=1;}
    let x=(i32::from(position[0])-1).wrapping_mul(256).wrapping_div(depth).wrapping_add(160)as i16;
    let y=(cosine.wrapping_mul(y)>>10).wrapping_sub(sine.wrapping_mul(z)>>10).wrapping_mul(256).wrapping_div(depth).wrapping_add(100)as i16;
    [x,y]
}
impl Particle{
    ///4028a0. The draw result precedes this tick's movement; reaching ground
    ///or negative lifetime dispatches mode0 and frees the slot immediately.
    pub fn advance(&mut self,view:View)->Option<Draw>{
        self.advance_native(view,&mut[0,0]).draw
    }
    pub fn advance_native(&mut self,view:View,last_projection:&mut[i16;2])->Step{
        if !self.active{return Step{draw:None,expiry:None};}self.lifetime=self.lifetime.wrapping_sub(1);if self.lifetime<0{return self.dispatch(None);}
        self.resume_native(view,last_projection)
    }
    fn dispatch(&mut self,draw:Option<Draw>)->Step{if self.expiry==0{self.active=false;Step{draw,expiry:None}}else{Step{draw,expiry:Some(self.expiry)}}}
    ///Called after a native expiry callback that left a nonnegativeZ and an
    ///occupied slot. The source resumes without decrementing lifetime again.
    pub fn resume_native(&mut self,view:View,last_projection:&mut[i16;2])->Step{
        let projection=project(self.position,view.sine,view.cosine);
        *last_projection=projection;
        let world=[self.origin[0].wrapping_add(projection[0]),self.origin[1].wrapping_add(projection[1])];
        let screen=[world[0].wrapping_sub(view.camera[0]),world[1].wrapping_sub(view.camera[1])];
        let color_point=self.flags&4==0;
        let visible=if self.flags&4==0||self.sprite==250{screen[0]>view.clip[0]&&screen[0]<view.clip[2]&&screen[1]>view.clip[1]&&screen[1]<view.clip[3]}
            else{screen[0].wrapping_add(200)>view.clip[0]&&screen[0].wrapping_sub(70)<view.clip[2]&&screen[1].wrapping_add(200)>view.clip[1]&&screen[1].wrapping_sub(70)<view.clip[3]};
        let draw=if visible{if color_point{
            Some(Draw{point:world,screen,sprite:0,height:0,angle:0,transformed:false,owner:self.owner,primitive:Some(Primitive::ColorPoint{color:self.tint})})
        }else if self.sprite==250{
            let height=((-i32::from(self.position[2]))>>5).min(4)as i16;let sprite=250i16.wrapping_add(height.wrapping_mul(4)).wrapping_add(i16::from(self.phase));self.phase=self.phase.wrapping_add(1)&3;
            Some(Draw{point:world,screen,sprite,height:0,angle:0,transformed:false,owner:self.owner,primitive:None})
        }else{let sprite=self.sprite.wrapping_add(i16::from(self.phase));let transformed=self.flags&8!=0;let angle=self.angle.wrapping_add(self.spin)as u8;
            if transformed{self.angle=i16::from(angle);}if self.fade_left==0{self.phase=self.phase.wrapping_add(1);if self.maximum_phase<self.phase{self.phase=0;}}
            Some(Draw{point:world,screen,sprite,height:((-i32::from(self.position[2]))>>3)as i16,angle,transformed,owner:self.owner,primitive:None})
        }}else{None};
        self.position[0]=self.position[0].wrapping_add(i16::from(self.velocity[0]));self.position[1]=self.position[1].wrapping_add(i16::from(self.velocity[1]));
        if world[0]< -32||world[0]>view.world[0]||world[1]< -32||world[1]>view.world[1]{self.active=false;return Step{draw,expiry:None};}
        self.position[2]=self.position[2].wrapping_add(i16::from(self.velocity[2]));
        if self.position[2]>0{self.position[2]=0;if self.flags&1==0{return self.dispatch(draw);}self.velocity[2]=1i8.wrapping_sub(self.velocity[2]);if self.velocity[2]==0{return self.dispatch(draw);}}
        self.deceleration_left=self.deceleration_left.wrapping_sub(1);if self.deceleration_left<0{self.deceleration_left=self.deceleration;
            if self.flags&16==0{self.velocity[0]=self.velocity[0].wrapping_sub(self.velocity[0].signum());}
            if self.flags&32==0{self.velocity[1]=self.velocity[1].wrapping_sub(self.velocity[1].signum());}
            if self.flags&64==0{self.velocity[2]=self.velocity[2].wrapping_add(1);}
        }
        self.fade_left=self.fade_left.wrapping_sub(1);if self.fade_left<0{self.fade_left=self.fade_interval;self.tint=self.tint.wrapping_sub(1);if self.tint==63{self.tint=64;}}
        Step{draw,expiry:None}
    }
    pub fn from_bytes(b:[u8;44])->Self{let word=|at|i16::from_le_bytes([b[at],b[at+1]]);Self{position:[word(0),word(2),word(4)],origin:[word(6),word(8)],velocity:[b[10]as i8,b[11]as i8,b[12]as i8],deceleration:b[13]as i8,deceleration_left:b[14]as i8,tint:b[15],fade_left:b[16]as i8,fade_interval:b[17]as i8,lifetime:word(18),expiry:word(20),flags:b[22],phase:b[23]as i8,maximum_phase:b[24]as i8,owner:b[25],sprite:word(26),angle:word(28),spin:word(30),active:word(0)!=-1000,raw_tail:b[32..44].try_into().unwrap()}}
    pub fn bytes(&self)->[u8;44]{let mut out=[0;44];out[32..44].copy_from_slice(&self.raw_tail);for(i,v)in self.position.iter().enumerate(){out[i*2..i*2+2].copy_from_slice(&v.to_le_bytes());}if !self.active{out[0..2].copy_from_slice(&(-1000i16).to_le_bytes());}
        for(i,v)in self.origin.iter().enumerate(){out[6+i*2..8+i*2].copy_from_slice(&v.to_le_bytes());}for(i,v)in self.velocity.iter().enumerate(){out[10+i]=*v as u8;}out[13]=self.deceleration as u8;out[14]=self.deceleration_left as u8;out[15]=self.tint;out[16]=self.fade_left as u8;out[17]=self.fade_interval as u8;out[18..20].copy_from_slice(&self.lifetime.to_le_bytes());out[20..22].copy_from_slice(&self.expiry.to_le_bytes());out[22]=self.flags;out[23]=self.phase as u8;out[24]=self.maximum_phase as u8;out[25]=self.owner;out[26..28].copy_from_slice(&self.sprite.to_le_bytes());out[28..30].copy_from_slice(&self.angle.to_le_bytes());out[30..32].copy_from_slice(&self.spin.to_le_bytes());out
    }
}

fn on_screen(point:[i16;2],view:View,double_camera:bool)->bool{let n=if double_camera{2}else{1};let x=point[0].wrapping_sub(view.camera[0].wrapping_mul(n));let y=point[1].wrapping_sub(view.camera[1].wrapping_mul(n));x> -49&&i32::from(x)<=i32::from(view.clip[2])+32&&y> -49&&i32::from(y)<=i32::from(view.clip[3])+32}
pub fn expiry_prefix(mode:i16,point:[i16;2],view:View,crate_sound_played:bool)->Vec<Action>{
    expiry_prefix_with_sprite(mode,point,0,view,crate_sound_played)
}
pub fn expiry_prefix_with_sprite(mode:i16,point:[i16;2],sprite:i16,view:View,crate_sound_played:bool)->Vec<Action>{
    if mode==1{let mut actions=Vec::new();if point[0]>-33{actions.push(Action::Blast{point,strength:10,suppress_visual:true});}actions.push(Action::DirectUnits{rect:[point[0].wrapping_sub(10),point[1].wrapping_sub(5),point[0].wrapping_add(54),point[1].wrapping_add(41)],strength:10});actions.push(Action::ProbeAmbient{point});return actions;}
    if mode==10{return vec![Action::ProbeFragment{point,sprite}];}
    let mut out=Vec::new();if mode==7{if point[0]> -33{out.push(Action::Blast{point,strength:0,suppress_visual:true});}out.push(Action::DirectUnits{rect:[point[0].wrapping_sub(10),point[1].wrapping_sub(5),point[0].wrapping_add(54),point[1].wrapping_add(41)],strength:0});out.push(Action::ProbeFalling{point});}
    else if mode==6{if on_screen(point,view,true){if !crate_sound_played{out.push(Action::Sound{point,id:30});}out.push(Action::SetCrateSoundGate);}out.push(Action::ProbeCrate{point});}out
}
pub fn ambient_suffix(point:[i16;2],terrain:u8)->Vec<Action>{
    let cell=((point[1]as u16).wrapping_mul(8)&0xff80).wrapping_add((point[0]>>4)as u16);
    if cell>0x3fff{return vec![Action::Retire{clear_expiry:true}];}
    if matches!(terrain&127,9|42|38|40|39){let mut out=vec![Action::Retire{clear_expiry:true}];if point[0]>-33{out.push(Action::Blast{point,strength:0,suppress_visual:false});}out}
    else{vec![Action::RandomHitBurst{origin:[point[0].wrapping_sub(154),point[1].wrapping_sub(80)]},Action::Retire{clear_expiry:false}]}
}
pub fn fragment_suffix(point:[i16;2],terrain:u8,sprite:i16,view:View,sound_played:bool)->Vec<Action>{
    //401800 uses Y&fff0, unlike the shorter cell expression in401af0.
    let cell=(point[1]as u16&0xfff0).wrapping_mul(8).wrapping_add((point[0]>>4)as u16);
    if cell>0x3fff{return vec![Action::Retire{clear_expiry:true}];}
    let mut out=Vec::new();if matches!(terrain&127,9|42|38|40|39){out.push(Action::Retire{clear_expiry:true});if on_screen(point,view,false)&&!sound_played{out.push(Action::Sound{point,id:8});}out.push(Action::QuietBlasts);if point[0]>-33{out.push(Action::Blast{point,strength:16,suppress_visual:false});}out.push(Action::ClearQuietBlasts);return out;}
    if matches!(sprite,0x166b|0x616|0x162f|0x16ba|0x74d|0x14d9|0x1637|0x200|0x14c5|0x735|0x1638|0x1640|0x163c){out.push(Action::QuietBlasts);if on_screen(point,view,false){out.push(Action::Sound{point,id:11});}}
    if point[0]>-33{out.push(Action::Blast{point,strength:127,suppress_visual:false});}
    out.push(Action::ResidualCrate);out.push(Action::ClearQuietBlasts);let origin=[point[0].wrapping_sub(152),point[1].wrapping_sub(100)];out.push(Action::Spawn{kind:Kind::Destroyed,origin,count:8});out.push(Action::Spawn{kind:Kind::Hit,origin,count:8});out
}
pub fn expiry_suffix(mode:i16,point:[i16;2],terrain:u8,view:View)->Vec<Action>{
    let cell=((point[1]as u16).wrapping_mul(8)&0xff80).wrapping_add((point[0]>>4)as u16);let mut out=Vec::new();
    if cell>0x3fff{return vec![if mode==6{Action::InvalidCrate}else{Action::Retire{clear_expiry:true}}];}
    let wet=matches!(terrain&127,9|42|38|40|39);
    if mode==7{if wet{out.push(Action::Retire{clear_expiry:true});if on_screen(point,view,false){out.push(Action::Sound{point,id:8});}}
        else if on_screen(point,view,false){out.push(Action::Sound{point,id:11});}
        out.push(Action::QuietBlasts);
        if point[0]> -33{out.push(Action::Blast{point,strength:0,suppress_visual:false});}
        if !wet{out.push(Action::Spawn{kind:Kind::Hit,origin:[point[0].wrapping_sub(154),point[1].wrapping_sub(80)],count:8});out.push(Action::Retire{clear_expiry:false});}
    }else if mode==6{if wet{out.push(Action::Retire{clear_expiry:true});}if point[0]> -33{out.push(Action::Blast{point,strength:40,suppress_visual:false});}
        if !wet{out.push(Action::ResidualCrate);let origin=[point[0].wrapping_sub(152),point[1].wrapping_sub(84)];out.push(Action::RandomDestroyedBurst{origin});out.push(Action::Spawn{kind:Kind::Hit,origin,count:8});}
    }out
}
pub fn apply_particle_action(p:&mut Particle,action:&Action){match action{Action::Retire{clear_expiry}=>{p.active=false;if *clear_expiry{p.expiry=0;}},Action::InvalidCrate=>{p.position[0]=-1;p.expiry=0;},Action::ResidualCrate=>{p.sprite=473;p.spin=0;p.position[2]=-1;p.lifetime=21;p.expiry=0;p.phase=0;p.maximum_phase=7;p.fade_left=2;p.fade_interval=2;p.velocity[2]=-5;},_=>{}}}
///4565a0: inclusive UNIT center geometry; hardware class1 uses16, all other
///classes8. The caller retains native bucket/slot order and32-per-bucket cap.
pub fn direct_unit_contains(position:[i16;2],class:u8,rect:[i16;4])->bool{let offset=if class==1{16}else{8};let x=position[0].wrapping_add(offset);let y=position[1].wrapping_add(offset);x>=rect[0]&&x<=rect[2]&&y>=rect[1]&&y<=rect[3]}

#[cfg(test)]mod tests{
    use super::*;
    fn hash_bytes(bytes:&[u8])->u32{bytes.iter().fold(2166136261,|h,&v|(h^u32::from(v)).wrapping_mul(16777619))}
    fn hash(h:&mut u32,values:&[i32]){for &v in values{*h=(*h^v as u32).wrapping_mul(16777619);}}
    #[test]fn death_and_ambient_constructors_match_192_executed_native_records(){let mut h=2166136261;
        for kind in 0..3{for seed in 0..64{let variant=match kind{0=>Kind::AmbientFragment(1234),1=>Kind::Fragment{sprite:5739,maximum:(seed%8)as i8},_=>Kind::Poof{sprite:1903,maximum:7}};let mut calls=0;let mut p=initialize(variant,[346,420],37,||{let r=seed*7919^calls*2027^0x7731a5c9;calls+=1;r as u32});if kind==0{p.maximum_phase=(seed%8)as i8;}p.owner=(seed&3)as u8;hash(&mut h,&[kind*64+seed,calls,hash_bytes(&p.bytes())as i32]);}}
        assert_eq!(h,121846309,"ZNativeDeathEffectOracle constructors");
    }
    #[test]fn ambient_and_hardware_fragment_expiry_contacts_match_512_executed_native_records(){let mut h=2166136261;let mut maximum=0;
        let sprites=[0x166b,0x616,0x162f,0x16ba,0x74d,0x14d9,0x1637,0x200,0x14c5,0x735,0x1638,0x1640,0x163c,0,250];let codes=[0,7,9,42,38,40,39,29,127];let points=[[-40i16,512],[-33,512],[-32,512],[-1,512],[0,0],[16,16],[512,512],[2032,2032],[2048,2048],[32767,32767],[-32768,-32768],[320,180],[160,100],[48,48],[-48,-48],[-49,-49]];let projections=[[0i16,0],[160,100],[-160,-100],[32767,-32768]];
        for kind in 0..2{for seed in 0..256{let sprite=if kind==0{1234}else{sprites[seed as usize%sprites.len()]};let variant=if kind==0{Kind::AmbientFragment(sprite)}else{Kind::Fragment{sprite,maximum:(seed%8)as i8}};let mut calls=0;let mut p=initialize(variant,[346,420],37,||{let r=seed*7919^calls*2027^0x7731a5c9;calls+=1;r as u32});if kind==0{p.maximum_phase=(seed%8)as i8;}p.owner=(seed&3)as u8;calls=0;
            let point=points[seed as usize%16];let projection=projections[seed as usize/16%4];p.position=[7,-13,if seed&1!=0{-1}else{0}];p.origin=[point[0].wrapping_sub(projection[0]),point[1].wrapping_sub(projection[1])];p.lifetime=(400-seed)as i16;let view=View{camera:[(seed%3*12)as i16,(seed%5*8)as i16],clip:[0,0,320,200],world:[2048,2048],sine:0,cosine:0};let gate=seed&1!=0;let mut quiet=seed>>1&1!=0;let mut events=0;let mut event_hash=2166136261;let mode=if kind==0{1}else{10};let mut pending=std::collections::VecDeque::from(expiry_prefix_with_sprite(mode,point,p.sprite,view,gate));
            while let Some(action)=pending.pop_front(){let mut event=None;match action{
                Action::Blast{point,strength,suppress_visual}=>event=Some(vec![2,i32::from(point[0]),i32::from(point[1]),i32::from(strength),i32::from(suppress_visual),i32::from(quiet)]),
                Action::DirectUnits{rect,strength}=>event=Some(vec![3,i32::from(rect[0]),i32::from(rect[1]),i32::from(rect[2]),i32::from(rect[3]),i32::from(strength)]),
                Action::Sound{id,..}=>event=Some(vec![1,i32::from(id)]),Action::QuietBlasts=>quiet=true,Action::ClearQuietBlasts=>quiet=false,
                Action::RandomHitBurst{origin}=>{let count=((seed*7919^calls*2027^0x7731a5c9)as u8&3)+4;calls+=1;event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),4,250,maximum,0x402770]);},
                Action::Spawn{kind,origin,count}=>{let(flags,sprite,constructor)=if kind==Kind::Destroyed{maximum=5;(12,557,0x4017b0)}else{(4,250,0x402770)};event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),flags,sprite,maximum,constructor]);},
                Action::ProbeAmbient{point}=>for a in ambient_suffix(point,codes[seed as usize%9]).into_iter().rev(){pending.push_front(a);},
                Action::ProbeFragment{point,sprite}=>for a in fragment_suffix(point,codes[seed as usize%9],sprite,view,gate).into_iter().rev(){pending.push_front(a);},
                _=>apply_particle_action(&mut p,&action),
            }if let Some(event)=event{events+=1;hash(&mut event_hash,&event);}}
            hash(&mut h,&[kind*256+seed,hash_bytes(&p.bytes())as i32,calls,events,event_hash as i32,0,0,i32::from(quiet),i32::from(gate)]);
        }}assert_eq!(h,2924193149,"ZNativeDeathEffectOracle expiry1/10");
    }
    fn particle(kind:Kind,seed:i32)->(Particle,i32){let mut calls=0;let origin=if seed<16{[346,420]}else if seed<24{[-180,-100]}else{[4080,4080]};let p=initialize(kind,origin,0,||{let value=(seed*7919)^(calls*2027)^0x7731a5c9;calls+=1;value as u32});(p,calls)}
    #[test]fn constructors_match_128_executed_native_cases(){let mut h=2166136261;for(kind,k)in[Kind::Hit,Kind::Destroyed].into_iter().zip(0..){for seed in 0..64{let(p,calls)=particle(kind,seed);hash(&mut h,&[k*64+seed,calls,hash_bytes(&p.bytes())as i32]);}}assert_eq!(h,401699989);}
    #[test]fn physics_projection_and_draws_match_6144_executed_native_updates(){let mut h=2166136261;let mut scenario=0;
        for kind in[Kind::Hit,Kind::Destroyed]{for seed in 0..32{let(mut p,_)=particle(kind,seed);let clip=match seed>>2&3{1=>[400,400,600,600],2=>[0,0,320,200],_=>[-32768,-32768,32767,32767]};let view=View{camera:[0,0],clip,world:[4096,4096],sine:crate::native_math::sine(425),cosine:crate::native_math::sine(553)};
            for _ in 0..96{let draw=p.advance(view);let(sprite,x,y,height,angle,transformed)=draw.map(|d|(i32::from(d.sprite),i32::from(d.point[0]),i32::from(d.point[1]),i32::from(d.height),i32::from(d.angle),i32::from(d.transformed))).unwrap_or((-1,-999,-999,0,0,0));hash(&mut h,&[scenario,hash_bytes(&p.bytes())as i32,sprite,x,y,height,angle,transformed]);scenario+=1;}
        }}assert_eq!(h,3417500266);
    }
    #[test]fn special_constructors_match_192_executed_native_cases(){let mut h=2166136261;for(kind,k)in[Kind::FallingObject(5960),Kind::CrateBurst,Kind::DirectHit].into_iter().zip(0..){for seed in 0..64{let mut calls=0;let p=initialize(kind,[346,420],37,||{let v=seed*7919^calls*2027^0x7731a5c9;calls+=1;v as u32});hash(&mut h,&[k*64+seed,calls,hash_bytes(&p.bytes())as i32]);}}assert_eq!(h,1354124282);}
    #[test]fn direct_unit_centers_match_512_executed_native_selector_cases(){let mut h=2166136261;for s in 0..512{let mut mask=0;for i in 0..8{let p=[(500+(s*13+i*7)%81-40)as i16,(500+(s*17+i*11)%81-40)as i16];if direct_unit_contains(p,((s+i)%8)as u8,[484,484,516,516]){mask|=1<<i;}}hash(&mut h,&[s,mask]);}assert_eq!(h,3634530395);}
    #[test]fn expiry_actions_and_record_mutations_match_256_executed_native_cases(){let mut h=2166136261;
        let points:[[_;2];12]=[[-40i16,512],[-33,512],[-32,512],[-1,512],[0,0],[16,16],[512,512],[2032,2032],[2048,2048],[32767,32767],[-32768,-32768],[320,180]];let projections=[[0i16,0],[160,100],[-160,-100],[32767,-32768]];let codes=[0,7,9,42,38,40,39,29,127];
        for(kind,k)in[Kind::FallingObject(5960),Kind::CrateBurst].into_iter().zip(0..){for seed in 0..128{let mut calls=0;let mut p=initialize(kind,[346,420],37,||{let v=seed*7919^calls*2027^0x7731a5c9;calls+=1;v as u32});calls=0;let point=points[seed as usize%points.len()];let projection=projections[seed as usize/12%4];p.position=[7,-13,if seed&1!=0{-1}else{0}];p.origin=[point[0].wrapping_sub(projection[0]),point[1].wrapping_sub(projection[1])];p.lifetime=(400-seed)as i16;
            let view=View{camera:[(seed%3*12)as i16,(seed%5*8)as i16],clip:[0,0,320,200],world:[2048,2048],sine:0,cosine:0};let mut gate=seed&1!=0;let mut quiet=false;let mode=if k==0{7}else{6};let mut actions=std::collections::VecDeque::from(expiry_prefix(mode,point,view,gate));let mut events=0;let mut eh=2166136261;let mut shared_maximum=0;
            while let Some(action)=actions.pop_front(){let mut event=None;match action{
                Action::Blast{point,strength,suppress_visual}=>event=Some(vec![2,i32::from(point[0]),i32::from(point[1]),i32::from(strength),i32::from(suppress_visual),i32::from(quiet)]),
                Action::DirectUnits{rect,strength}=>event=Some(vec![3,i32::from(rect[0]),i32::from(rect[1]),i32::from(rect[2]),i32::from(rect[3]),i32::from(strength)]),
                Action::Sound{id,..}=>event=Some(vec![1,i32::from(id)]),Action::SetCrateSoundGate=>gate=true,Action::QuietBlasts=>quiet=true,
                Action::Spawn{kind,origin,count}=>{let(flags,sprite,constructor)=if kind==Kind::Hit{(4,250,0x402770)}else{unreachable!()};event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),flags,sprite,shared_maximum,constructor]);},
                Action::RandomDestroyedBurst{origin}=>{let count=((seed*7919^calls*2027^0x7731a5c9)as u8&3)+5;calls+=1;shared_maximum=5;event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),12,557,5,0x4017b0]);},
                Action::ProbeFalling{point}|Action::ProbeCrate{point}=>{let suffix=expiry_suffix(mode,point,codes[seed as usize%9],view);for a in suffix.into_iter().rev(){actions.push_front(a);}},
                _=>apply_particle_action(&mut p,&action),
            }if let Some(event)=event{events+=1;hash(&mut eh,&event);}}
            hash(&mut h,&[k*128+seed,hash_bytes(&p.bytes())as i32,calls,events,eh as i32,0,0,i32::from(quiet),i32::from(gate)]);
        }}assert_eq!(h,3181430069);
    }
    #[test]fn special_flight_landing_and_residual_traces_match_8192_executed_native_updates(){let mut h=2166136261;let mut scenario=0;
        for kind in[Kind::FallingObject(5960),Kind::CrateBurst]{for seed in 0..32{let mut calls=0;let mut p=initialize(kind,[346,420],37,||{let v=seed*7919^calls*2027^0x7731a5c9;calls+=1;v as u32});let code=[0,9,29,42,38,40,39,7][seed as usize%8];let view=View{camera:[(seed%3*12)as i16,(seed%5*8)as i16],clip:[0,0,2048,2048],world:[4096,4096],sine:crate::native_math::sine(425),cosine:crate::native_math::sine(553)};let mut projection=[0,0];let mut quiet=false;
            for _ in 0..128{let mut draw=None;let mut events=0;let mut eh=2166136261;let mut gate=false;let mut step=p.advance_native(view,&mut projection);
                loop{if step.draw.is_some(){draw=step.draw;}let Some(mode)=step.expiry else{break;};let point=[p.origin[0].wrapping_add(projection[0]),p.origin[1].wrapping_add(projection[1])];let mut pending=std::collections::VecDeque::from(expiry_prefix(mode,point,view,gate));let mut maximum=0;
                    while let Some(action)=pending.pop_front(){let mut event=None;match action{
                        Action::Blast{point,strength,suppress_visual}=>event=Some(vec![2,i32::from(point[0]),i32::from(point[1]),i32::from(strength),i32::from(suppress_visual),i32::from(quiet)]),
                        Action::DirectUnits{rect,strength}=>event=Some(vec![3,i32::from(rect[0]),i32::from(rect[1]),i32::from(rect[2]),i32::from(rect[3]),i32::from(strength)]),
                        Action::Sound{id,..}=>event=Some(vec![1,i32::from(id)]),Action::SetCrateSoundGate=>gate=true,Action::QuietBlasts=>quiet=true,
                        Action::Spawn{kind,origin,count}=>{assert_eq!(kind,Kind::Hit);event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),4,250,maximum,0x402770]);},
                        Action::RandomDestroyedBurst{origin}=>{let count=((seed*7919^calls*2027^0x7731a5c9)as u8&3)+5;calls+=1;maximum=5;event=Some(vec![4,i32::from(count),i32::from(origin[0]),i32::from(origin[1]),12,557,5,0x4017b0]);},
                        Action::ProbeFalling{point}|Action::ProbeCrate{point}=>{for a in expiry_suffix(mode,point,code,view).into_iter().rev(){pending.push_front(a);}},
                        _=>apply_particle_action(&mut p,&action),
                    }if let Some(event)=event{events+=1;hash(&mut eh,&event);}}
                    if !p.active||p.position[2]<0{break;}step=p.resume_native(view,&mut projection);
                }
                let(sprite,x,y,height,angle,transformed)=draw.map(|d|(i32::from(d.sprite),i32::from(d.screen[0]),i32::from(d.screen[1]),i32::from(d.height),i32::from(d.angle),i32::from(d.transformed))).unwrap_or((-1,-999,-999,0,0,0));hash(&mut h,&[scenario,hash_bytes(&p.bytes())as i32,calls,events,eh as i32,i32::from(quiet),i32::from(gate),sprite,x,y,height,angle,transformed]);scenario+=1;
            }
        }}assert_eq!(h,859486516);
    }
}
