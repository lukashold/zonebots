//! Ordered45c5c0 common caller. All path, contact, hardware and random storage
//! remains with its existing owner. These are adapter outcomes, not native EAX.
#[path="original_hardware_common_game.rs"]mod game;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Shared{pub saved_control06:u8,pub gate:u8,pub movement_byte:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Boundary{MissingActor,MissingMember,MissingContext,MissingLine(usize),MissingObjects,HatchRelease(u8),AutomaticTarget(u8),MissingLoopSlot(u8),ObjectOrder(u8),ApcSideFire,#[cfg(test)]CrtPrefix}
#[derive(Clone,Copy,Debug,Default)]pub struct Frame{pub role:i8,pub control:u8,pub pattern:u8,pub remaining:i16,pub frame:i16,pub timer:i8,pub flags31:u8,pub flags69:u8,pub counter68:i8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Write{ElevationZero,Gate(bool),SaveControl(u8),Control(u8),Counter(i8),PhaseNine,FrameTimer(i16,i8)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Call{Occupancy,Body,Hatch,Enemy,Friendly,SelectMovementHardware,PreservedMove,Cosmetics,Follow,Resume,Check,NormalMove,Tail}
pub trait World{type Error;fn frame(&self)->Result<Frame,Self::Error>;fn write(&mut self,w:Write);fn saved_control(&self)->u8;fn w17(&self)->u8;fn write_w17(&mut self,n:u8);fn call(&mut self,c:Call)->Result<bool,Self::Error>;}
pub fn body_w4(b:u8)->u8{(b&240)|(((b as i8>>4)as i16+i16::from(b&15)).clamp(0,2)as u8)}
pub fn run<W:World>(w:&mut W)->Result<(),W::Error>{
 w.write(Write::ElevationZero);w.write(Write::Gate(false));for c in[Call::Occupancy,Call::Body,Call::Hatch,Call::Enemy,Call::Friendly]{w.call(c)?;}
 w.write(Write::Counter(w.frame()?.counter68.wrapping_sub(1).max(0)));w.call(Call::SelectMovementHardware)?;
 if w.frame()?.role>0{let s=w.frame()?;if s.remaining!=0{w.write(Write::SaveControl(s.control));w.write(Write::Control(s.pattern.wrapping_add(16)));w.call(Call::PreservedMove)?;w.write(Write::Control(w.saved_control()));w.call(Call::Cosmetics)?;}let s=w.frame()?;w.call(Call::Follow)?;w.write(Write::FrameTimer(i16::from(s.frame as u8 as i8),s.timer));w.write(Write::PhaseNine);}
 let flag=w.w17();if flag&2!=0{if w.frame()?.flags31&64==0{if w.call(Call::Resume)?{w.call(Call::Check)?;}w.write(Write::PhaseNine);w.write_w17((w.w17()&!2)|1);}}
 else if flag&1==0{w.write(Write::Gate(true));}
 else if w.frame()?.flags69&8==0{w.call(Call::NormalMove)?;}
 w.call(Call::Tail)?;Ok(())
}

///Signed41f4d0 expanded viewport, shared with44fa40's terminal writer.
pub fn expanded_view(point:[i16;2],camera:[i16;2],size:[i16;2])->bool{let q=std::array::from_fn::<_,2,_>(|i|point[i].wrapping_sub(camera[i]));q[0]>=-48&&i32::from(q[0])<=i32::from(size[0])+32&&q[1]>=-48&&i32::from(q[1])<=i32::from(size[1])+32}

///Existing coupled leaves retained for playability; these are not whole-caller
///Source proofs and are deliberately separate from fatal missing-raw boundaries.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum LegacyCall{Broadcast15fb0,Secondary5b210,Dodge44f730}
