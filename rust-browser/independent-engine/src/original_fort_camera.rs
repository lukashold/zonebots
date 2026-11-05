#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Camera {
    pub current:[i16;2],
    pub request:[i16;2],
    pub delta_eighth:[i16;2],
    pub remaining:i16,
    pub mode:i16,
    pub target:[i16;2],
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct ModeBoundary(pub i16);

impl Camera {
    /// The requested fort position is consumed on the UI callback. Its
    /// remaining-count divisor and final snap are separate from rendering.
    pub(crate) fn tick(&mut self)->Result<(),ModeBoundary> {
        if self.request[0]!=-1 {
            self.delta_eighth=std::array::from_fn(|axis|((i32::from(self.current[axis])-i32::from(self.request[axis]))/8) as i16);
            self.target=[self.request[0].wrapping_sub(32),self.request[1].wrapping_add(8)];
            self.request[0]=-1;
            self.mode=1;
            self.remaining=16;
        } else if self.mode==0||self.remaining==0 {return Ok(());}
        if self.mode!=1{return Err(ModeBoundary(self.mode));}
        for (current,goal) in self.current.iter_mut().zip(self.target) {
            let difference=i32::from(goal)-i32::from(*current);
            *current=current.wrapping_add(((2*difference)/i32::from(self.remaining)) as i16);
        }
        self.remaining=self.remaining.wrapping_sub(1);
        if self.remaining==0 {self.current=self.target;}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    #[ignore = "requires private native camera tick trace"]
    fn fort_camera_matches_all_original_smooth_steps_and_final_snap() {
        let path=std::env::var("NATIVE_FORT_CAMERA_STEP_TRACE").expect("private trace path");
        let text=std::fs::read_to_string(path).unwrap();
        assert_eq!(text.lines().count(),128*18);
        for (i,line) in text.lines().enumerate() {
            let row:Value=serde_json::from_str(line).unwrap();
            assert_eq!(row["id"],i/18);assert_eq!(row["frame"],i%18);
            let before=|n:usize|row["before"][n].as_i64().unwrap() as i16;
            let mut camera=Camera{current:[before(0),before(1)],request:[before(2),before(3)],delta_eighth:[before(4),before(5)],
                remaining:before(6),mode:before(7),target:[before(8),before(9)]};
            camera.tick().unwrap();
            assert_eq!([camera.current[0],camera.current[1],camera.request[0],camera.request[1],camera.delta_eighth[0],camera.delta_eighth[1],
                camera.remaining,camera.mode,camera.target[0],camera.target[1]],
                std::array::from_fn(|n|row["after"][n].as_i64().unwrap() as i16),"row {i}");
        }
    }
}
