use super::{Force,World,remove_team};
use serde_json::Value;

struct Recorder {
    visible:u8,
    count:i16,
    marker:u32,
    flags:[u8;240],
    multiplayer:bool,
    suppressed:bool,
    visits:Vec<[i64;3]>,
}

impl Recorder {
    fn visit(&mut self,pc:i64,slot:u8,force:Force) {self.visits.push([pc,i64::from(slot),i64::from(force.0)]);}
}

impl World for Recorder {
    type Boundary=std::convert::Infallible;
    fn fort_visibility(&self,team:u8)->u8 {assert_eq!(team,1);self.visible}
    fn hide_fort(&mut self,team:u8){assert_eq!(team,1);self.visible=0;}
    fn team_count(&self)->i16{self.count}
    fn set_team_count(&mut self,value:i16){self.count=value;}
    fn set_team_marker(&mut self,value:u32){self.marker=value;}
    fn flags31(&self,slot:u8)->u8{self.flags[usize::from(slot)]}
    fn remove_actor(&mut self,slot:u8,force:&mut Force)->Result<(),Self::Boundary>{
        self.visit(0x458ec0,slot,*force);
        let wreck=force.begin_conversion(self.multiplayer);
        self.visit(0x458af0,slot,*force);
        // The bound APC controls exercise the nested suppressed converter
        // only when the outer unsuppressed conversion chooses to park it.
        if !wreck&&!self.suppressed {
            self.visit(0x458ec0,slot,*force);
            force.begin_conversion(self.multiplayer);
            self.visit(0x458af0,slot,*force);
            force.finish_conversion();
        }
        force.finish_conversion();
        self.flags[usize::from(slot)]&=!4;
        Ok(())
    }
}

fn decode(v:&Value)->Vec<u8>{v.as_str().unwrap().as_bytes().chunks_exact(2)
    .map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}

#[test]
#[ignore = "requires the private whole fort team removal trace"]
fn original_fort_team_cleanup_keeps_force_across_the_correct_conversion_boundaries() {
    let path=std::env::var("NATIVE_FORT_CREDIT_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    let rows:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(),128);
    for (id,pair) in rows.chunks_exact(2).enumerate() {
        let (b,a)=(&pair[0],&pair[1]);
        let unit=decode(&b["units"]);
        let mut recorder=Recorder{visible:b["teamActive"].as_u64().unwrap() as u8,count:b["teamCount"].as_u64().unwrap() as i16,
            marker:b["marker"].as_u64().unwrap() as u32,flags:std::array::from_fn(|i|unit[i*192+49]),
            multiplayer:b["mp"]!=0,suppressed:b["suppressed"]!=0,visits:vec![]};
        let mut force=Force(b["force"].as_u64().unwrap() as u8);
        remove_team(&mut recorder,1,&mut force).unwrap();
        let expected:Vec<[i64;3]>=a["visits"].as_array().unwrap().iter().filter(|e|e[0]==0x458ec0||e[0]==0x458af0)
            .map(|e|[0,1,2].map(|n|e[n].as_i64().unwrap())).collect();
        assert_eq!(recorder.visits,expected,"case {id}");
        assert_eq!([i64::from(recorder.visible),i64::from(recorder.count),i64::from(recorder.marker),i64::from(force.0)],
            ["teamActive","teamCount","marker","force"].map(|key|a[key].as_i64().unwrap()));
        let end=decode(&a["units"]);
        for (slot,flags) in recorder.flags.iter().enumerate(){assert_eq!(*flags,end[slot*192+49],"flags {id}/{slot}");}
    }
}
