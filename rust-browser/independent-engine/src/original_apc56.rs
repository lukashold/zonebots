//!447d50 wrapper order. The automatic passenger backend remains explicitly
//!Legacy; this module adds no W,passenger,target or random storage.
#[path="original_apc56_game.rs"]mod game;
pub trait World{type Error;fn phase(&mut self,value:u8);fn scope(&mut self,value:u8);fn common(&mut self)->Result<(),Self::Error>;fn words(&self)->[i16;2];fn restore_words(&mut self,value:[i16;2]);fn legacy_automatic(&mut self)->Result<(),Self::Error>;}
pub fn run<W:World>(w:&mut W)->Result<(),W::Error>{w.phase(9);w.scope(1);w.common()?;let words=w.words();w.legacy_automatic()?;w.restore_words(words);w.phase(56);w.scope(0);Ok(())}

#[cfg(test)]pub(crate)fn take_trace()->Vec<(usize,&'static str)>{game::take_trace()}

#[cfg(test)]mod tests{
 use super::*;
 struct Probe{trace:Vec<&'static str>,phase:u8,guard:u8,words:[i16;2],fail:bool}
 impl World for Probe{type Error=&'static str;fn phase(&mut self,n:u8){self.phase=n;self.trace.push(if n==9{"Phase9"}else{"Phase56"});}fn scope(&mut self,n:u8){self.guard=n;self.trace.push(if n==1{"Guard1"}else{"Guard0"});}fn common(&mut self)->Result<(),Self::Error>{self.trace.push("Common");self.words=[1234,-1234];if self.fail{Err("prefix")}else{Ok(())}}fn words(&self)->[i16;2]{self.words}fn restore_words(&mut self,n:[i16;2]){self.trace.push("Restore");self.words=n;}fn legacy_automatic(&mut self)->Result<(),Self::Error>{self.trace.push("LegacyAutomatic");self.words=[99,88];Ok(())}}
 #[test]fn saves_post_common_words_and_restores_only_after_the_single_automatic_tail(){let mut p=Probe{trace:vec![],phase:56,guard:5,words:[7,8],fail:false};assert_eq!(run(&mut p),Ok(()));assert_eq!(p.trace,["Phase9","Guard1","Common","LegacyAutomatic","Restore","Phase56","Guard0"]);assert_eq!(p.words,[1234,-1234]);assert_eq!((p.phase,p.guard),(56,0));}
 #[test]fn a_named_common_boundary_preserves_known_prefix_writes_without_fabricated_tail_return(){let mut p=Probe{trace:vec![],phase:56,guard:5,words:[7,8],fail:true};assert_eq!(run(&mut p),Err("prefix"));assert_eq!(p.trace,["Phase9","Guard1","Common"]);assert_eq!(p.words,[1234,-1234]);assert_eq!((p.phase,p.guard),(9,1));}
}
