//! Integer geometry used by the original movement, aiming and formations.

/// 00459af0: eight directions, with the original asymmetric half-axis ties.
pub fn direction(dx:i32,dy:i32)->u8 {
    if dx<=0 {
        let x=-dx;
        if x<dy.abs() {
            let h=dy>>1;
            if h<0 {if x < -h {0}else{7}}else if x<h {4}else{5}
        }else if dy<0 {if -dy < x>>1 {6}else{7}}
        else if dy < x>>1 {6}else{5}
    }else if dy.abs()<=dx {
        if dy<0 {if dx>>1 < -dy {1}else{2}}
        else if dx>>1 < dy {3}else{2}
    }else {
        let h=dy>>1;
        if h>=0 {if dx<h {4}else{3}}else if dx < -h {0}else{1}
    }
}
pub fn angle(direction:u8)->f64 {(f64::from(direction)-2.)*std::f64::consts::FRAC_PI_4}

/// SINCOS.DAT is big-endian signed16, then arithmetic >>5 in00460800.
/// This mathematical reconstruction matches all640 runtime table entries.
pub fn sine(index:i32)->i32 {
    let q=index.rem_euclid(512);
    let raw=if q%128==0 {[0,32767,0,-32768][(q/128)as usize]}
        else{(f64::from(q)*std::f64::consts::PI/256.).sin().mul_add(32768.,0.).floor()as i32};
    raw>>5
}
pub fn isqrt(n:u64)->i32 {
    let mut x=(n as f64).sqrt()as u64;
    while x.saturating_mul(x)>n{x-=1;}
    while (x+1).saturating_mul(x+1)<=n{x+=1;}
    x as i32
}
/// 004643b0 truncates the float arctangent within each quadrant, then the
/// allocator quantizes its degree bearing to an unsigned 256-step rotation.
pub fn projectile_angle(dx:i32,dy:i32)->f64 {
    let bearing=if dx==0{if dy>0{180}else{0}}
        else if dy==0{if dx>0{90}else{270}}
        else{
            let degree=|ratio:f32|f64::from(ratio.atan())*57.296;
            if dx<0{if dy<0{270+degree(dy as f32/dx as f32)as i32}else{180+degree(-dx as f32/dy as f32)as i32}}
            else if dy<0{degree(-dx as f32/dy as f32)as i32}else{90+degree(dy as f32/dx as f32)as i32}
        };
    f64::from(255-(bearing*256/360))*std::f64::consts::TAU/256.
}

#[derive(Clone)]
pub struct Line {
    pub x:i32,pub y:i32,pub remaining:i32,
    major:i32,minor:i32,error:i32,sx:i32,sy:i32,x_major:bool,
}
impl Line {
    pub fn new(x:i32,y:i32,tx:i32,ty:i32)->Self {
        let dx=tx-x;let dy=ty-y;let major=dx.abs().max(dy.abs());
        Self{x,y,remaining:major,major,minor:dx.abs().min(dy.abs()),error:major/2,
            sx:dx.signum(),sy:dy.signum(),x_major:dx.abs()>=dy.abs()}
    }
    pub fn advance(&mut self,pixels:i32)->i32 {
        let n=pixels.min(self.remaining).max(0);
        for _ in 0..n {
            self.error-=self.minor;let diagonal=self.error<0;
            if diagonal {self.error+=self.major;}
            if self.x_major{self.x+=self.sx;if diagonal{self.y+=self.sy;}}
            else{self.y+=self.sy;if diagonal{self.x+=self.sx;}}
        }
        self.remaining-=n;n
    }
    pub fn extend(&mut self,pixels:i32){self.remaining+=pixels;}
    pub fn major_length(&self)->i32{self.major}
}

#[cfg(test)]
mod tests {
 use super::*;
 // Outputs obtained by executing00459af0 in Ghidra's x86 emulator.
 #[test]
 fn direction_matches_all_728_native_integer_vectors() {
  let mut hash=2166136261u32;
  for dx in -13..=13 {for dy in -13..=13 {if dx==0&&dy==0{continue;}hash=(hash^u32::from(direction(dx,dy))).wrapping_mul(16777619);}}
  assert_eq!(hash,2008442125);
 }
 #[test]
 fn four_steps_match_executed_native_bullet_routines_in_all_octants() {
  for (dx,dy) in [(60,22),(22,60),(-60,22),(-22,60),(60,-22),(22,-60),(-60,-22),(-22,-60)] {
   let mut ray=Line::new(500,500,500+dx,500+dy);
   for (tick,minor) in [5,11,16,22].into_iter().enumerate() {
    ray.advance(15);let major=(tick as i32+1)*15;
    let (x,y)=if dx.abs()>=dy.abs(){(major*dx.signum(),minor*dy.signum())}else{(minor*dx.signum(),major*dy.signum())};
    assert_eq!((ray.x,ray.y),(500+x,500+y),"{dx},{dy} tick{tick}");
   }
  }
 }
}
