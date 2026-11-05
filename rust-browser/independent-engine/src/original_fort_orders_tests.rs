use super::{disperse,Actor,Bounds,Move,World};
use serde_json::Value;

struct Recorder {
    bounds: Bounds,
    actors: Vec<Option<Actor>>,
    visits: Vec<[i64;3]>,
    routed: Vec<Move>,
    words: [u32;2],
    forced: u8,
    reentry: Option<u8>,
}

impl World for Recorder {
    type Boundary = std::convert::Infallible;
    fn bounds(&self)->Bounds { self.bounds }
    fn store_bounds(&mut self,bounds:Bounds) { self.bounds=bounds; }
    fn actor(&self,slot:u8)->Option<Actor> { self.actors[usize::from(slot)] }
    fn store_order_arguments(&mut self,command:Move) {
        for (word,value) in self.words.iter_mut().zip(command.target) {
            *word=(*word&0xffff0000)|u32::from(value as u16);
        }
    }
    fn update_order_symbol(&mut self,slot:u8,symbol:u16)->Result<(),Self::Boundary> {
        assert_eq!(symbol,0x0abb);
        self.visits.push([0x438740,0x5e30c8+i64::from(slot)*192,i64::from(self.forced)]);
        Ok(())
    }
    fn set_forced_order(&mut self,enabled:bool) { self.forced=u8::from(enabled); }
    fn route_order(&mut self,command:Move)->Result<(),Self::Boundary> {
        self.visits.push([0x45ef00,0x5e30c8+i64::from(command.slot)*192,i64::from(self.forced)]);
        self.routed.push(command);
        match (self.reentry,self.routed.len()) {
            (Some(1),1) => self.bounds=Bounds{left:130,top:130,right:180,bottom:180},
            (Some(1),2) => {self.bounds.left=0;self.bounds.top=0;},
            (Some(2),1) => {self.bounds.top=100;self.bounds.bottom=160;self.bounds.right=180;},
            (Some(3),1) => self.bounds.right=-1,
            (Some(4),1) => {self.actors[2].as_mut().unwrap().flags31=0;self.actors[3].as_mut().unwrap().position=[70,70];},
            (Some(5),1) => {self.bounds.left=50;self.bounds.top=50;},
            _ => (),
        }
        Ok(())
    }
}

fn decode(hex:&Value)->Vec<u8> {
    hex.as_str().unwrap().as_bytes().chunks_exact(2)
        .map(|pair|u8::from_str_radix(std::str::from_utf8(pair).unwrap(),16).unwrap()).collect()
}

#[test]
#[ignore = "requires private original forced-order call trace"]
fn original_fort_sweep_preserves_physical_order_boundaries_and_router_arguments() {
    let path=std::env::var("NATIVE_FORT_SWEEP_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    let rows:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(),192);
    let mut routed=0;
    for (id,pair) in rows.chunks_exact(2).enumerate() {
        let (before,after)=(&pair[0],&pair[1]);
        assert_eq!(before["scenario"],id);assert_eq!(after["scenario"],id);
        let rect=|i:usize| before["rect"][i].as_i64().unwrap() as i32;
        let bounds=Bounds{left:rect(0),top:rect(1),right:rect(2),bottom:rect(3)};
        let bytes=decode(&before["units"]);
        let actors=bytes.chunks_exact(192).map(|u| Some(Actor{flags31:u[49],class:u[124] as i8,
            position:[i16::from_le_bytes([u[44],u[45]]),i16::from_le_bytes([u[46],u[47]])]})).collect();
        let before_fields=|i:usize| before["fields"][i].as_u64().unwrap() as u32;
        let mut recorder=Recorder{bounds,actors,visits:vec![],routed:vec![],
            words:[before_fields(15),before_fields(16)],forced:before_fields(6) as u8,reentry:None};
        disperse(&mut recorder,bounds,(id%4) as u16,before["direction"].as_i64().unwrap()!=0).unwrap();
        let expected:Vec<[i64;3]>=after["visits"].as_array().unwrap().iter()
            .filter(|row|row[0]==0x438740||row[0]==0x45ef00).map(|row|[0,1,2].map(|i|row[i].as_i64().unwrap())).collect();
        assert_eq!(recorder.visits,expected,"row {id}");
        assert_eq!(recorder.forced as u64,after["fields"][6].as_u64().unwrap());
        assert_eq!(recorder.words.map(u64::from),[15usize,16].map(|i|after["fields"][i].as_u64().unwrap()));
        routed+=recorder.routed.len();
    }
    assert_eq!(routed,504);
}

#[test]
#[ignore = "requires private reentry trace with injected router mutations"]
fn original_fort_sweep_rereads_live_slots_and_bounds_without_changing_the_move_target() {
    let path=std::env::var("NATIVE_FORT_SWEEP_REENTRY_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    assert_eq!(text.lines().count(),96);
    for (id,line) in text.lines().enumerate() {
        let row:Value=serde_json::from_str(line).unwrap();
        let n=|key:&str| row[key].as_i64().unwrap();
        assert_eq!(n("id"),id as i64);
        let owner=n("owner") as u8;
        let bounds=Bounds{left:0,top:0,right:n("upper") as i32,bottom:120};
        let mut actors=vec![None;240];
        for (i,position) in [[30,30],[0,5],[90,90],[150,150],[170,170],[5,5],[70,70],[40,40]].into_iter().enumerate() {
            actors[i]=Some(Actor{flags31:4|((owner+u8::from(i!=6))&3),class:if i==7 {2}else{0},position});
        }
        let mut recorder=Recorder{bounds,actors,visits:vec![],routed:vec![],words:[0,0],forced:0,reentry:Some(n("control") as u8)};
        disperse(&mut recorder,bounds,u16::from(owner),n("reverse")!=0).unwrap();
        let found:Vec<[i64;4]>=recorder.routed.iter().zip(recorder.visits.iter().filter(|v|v[0]==0x45ef00))
            .map(|(command,entry)|[i64::from(command.slot),i64::from(command.target[0]),i64::from(command.target[1]),entry[2]]).collect();
        let expected:Vec<[i64;4]>=row["moves"].as_array().unwrap().iter().map(|r|[0,1,2,3].map(|i|r[i].as_i64().unwrap())).collect();
        assert_eq!(found,expected,"row {id}");
        assert_eq!(i64::from(recorder.forced),n("forcedAfter"));
    }
}
