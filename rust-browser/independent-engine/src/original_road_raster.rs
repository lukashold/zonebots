//! Persistent grid ray used by the original route price comparison (45cf90).
use super::{Point,Price};

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct Raster {
    /// 6330a8..b4: destination followed by the advancing position.
    pub(crate) destination:[i16;3],
    pub(crate) position:[i16;3],
    /// Actual offsets 62d5d0/d4/dc are Z, X, Y respectively.
    pub(crate) delta:[i16;3],
    pub(crate) steps:i16,
    pub(crate) error:[i16;3],
    pub(crate) direction:[i16;3],
    pub(crate) axis:u8,
    pub(crate) special_tiles:i16,
}
impl Raster {
    fn begin(&mut self,from:Point,to:Point)->bool{
        self.position=[from.0,from.1,0];self.destination=[to.0,to.1,0];
        if [from.0,from.1,to.0,to.1].iter().any(|&n|n<0){return false;}
        self.prepare_grid();
        true
    }
    fn prepare_grid(&mut self){
        for i in 0..2{self.position[i]>>=4;self.destination[i]>>=4;}
        if self.position[0]>=self.destination[0]{std::mem::swap(&mut self.position,&mut self.destination);}
        let dx=self.destination[0]-self.position[0];let dy=self.destination[1]-self.position[1];
        self.delta=[dx,dy.abs(),0];self.direction=[1,if dy<0{-1}else{1},1];
        self.axis=u8::from(self.delta[1]>=self.delta[0]);
        self.steps=self.delta[self.axis as usize];let half=self.steps>>1;
        self.error=if self.axis==0{[half,half,0]}else{[0,half,half]};
    }
    /// 43b090 temporarily reuses the delta words as endpoint scan controls.
    pub(crate) fn road_endpoint(&mut self,point:Point,on_road:bool){
        self.delta[0]=point.0;self.delta[1]=point.1;
        if !on_road{self.delta[2]=-16;}
    }
    /// 43b950 leaves its last compacted pair prepared, without raster steps.
    pub(crate) fn road_compaction(&mut self,from:Point,to:Point){
        self.position=[from.0,from.1,0];self.destination=[to.0,to.1,0];
        self.prepare_grid();
    }
    fn step(&mut self){
        let main=self.axis as usize;let cross=1-main;let error=if main==0{0}else{2};
        self.position[main]+=self.direction[main];self.error[error]-=self.delta[cross];
        if self.error[error]<0{self.error[error]+=self.steps;self.position[cross]+=self.direction[cross];}
    }
    pub(crate) fn price(&mut self,points:&[Point],mut terrain:impl FnMut(Point)->u8,mut price:impl FnMut(u8)->u8)->Price{
        let mut result=Price::default();
        for pair in points.windows(2){
            if pair[0].0==-1||pair[1].0==-1{break;}
            if !self.begin(pair[0],pair[1]){continue;}
            let mut high=false;
            for _ in 0..self.steps {
                let code=terrain((self.position[0],self.position[1]))&127;
                if matches!(code,40|52){result.special_tiles=result.special_tiles.wrapping_add(1);}
                let raw=price(code);let reduced=if high&&raw>=10{raw>>1}else{raw};
                result.cost=result.cost.wrapping_add(i32::from(reduced));high=raw>=10;
                self.step();
            }
        }
        self.special_tiles=self.special_tiles.wrapping_add(result.special_tiles);
        result
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]
    #[ignore="requires the private original route raster trace"]
    fn pricing_preserves_original_warm_raster_context(){
        let path=std::env::var("NATIVE_ROAD_RASTER_CONTEXT").unwrap();
        let trace=std::fs::read_to_string(path).unwrap();assert_eq!(trace.lines().count(),240);
        let starts=[(16,16),(31,47),(127,64),(1023,1023)];
        let deltas=[(-47,-32),(-32,0),(-17,31),(0,-47),(0,0),(0,63),(16,32),(17,31),(64,-15),(79,48)];
        let prices=[2,5,10,12,72];
        for (row,line) in trace.lines().enumerate(){
            let record:serde_json::Value=serde_json::from_str(line).unwrap();
            let v=|k:&str|record[k].as_i64().unwrap()as usize;
            let mode=v("mode");let start=starts[v("start")];let delta=deltas[v("delta")];
            let mut points=vec![start,(start.0+delta.0,start.1+delta.1)];
            if v("legs")==2{points.push((start.0+32,start.1+16));}
            let b=0x3540+mode as i16*256;
            let mut raster=Raster{destination:[b,b+1,b+2],position:[b+3,b+4,b+5],delta:[b+7,b+8,b+6],steps:b+9,
                error:[b+10,b+11,b+12],direction:[b+13,b+14,b+15],axis:9,special_tiles:if mode==2{32767}else{mode as i16*103}};
            let p=raster.price(&points,|(x,y)|((x as usize*7+y as usize*11+mode*3)%67)as u8,|c|prices[(c as usize+mode)%5]);
            let words=[raster.destination[0],raster.destination[1],raster.destination[2],raster.position[0],raster.position[1],raster.position[2],
                raster.delta[2],raster.delta[0],raster.delta[1],raster.steps,raster.error[0],raster.error[1],raster.error[2],raster.direction[0],raster.direction[1],raster.direction[2]];
            assert_eq!(serde_json::json!(words),record["raster"],"raster case {row}");
            assert_eq!(raster.axis as usize,v("step"),"axis case {row}");
            assert_eq!(p.cost as usize,v("price"),"price case {row}");
            assert_eq!(raster.special_tiles as usize,v("special"),"special case {row}");
        }
    }
}
