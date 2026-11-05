use super::{Lane,Notice,Radio,Speaker,World,message};
use serde_json::Value;

struct Recorder {
    actors:[Option<Speaker>;241],
    phase:i32,
    hold:i32,
    random:u16,
    multiplayer:bool,
    draws:u8,
    messages:Vec<Value>,
    current:u16,
    next_mode:u16,
    next_message:u16,
    phrase:u16,
    length:u16,
    sound:u16,
}
impl World for Recorder {
    type Boundary=std::convert::Infallible;
    fn scene_gate(&self)->i32 {self.phase}
    fn voice_hold(&self)->i32 {self.hold}
    fn set_voice_hold(&mut self,value:i32){self.hold=value;}
    fn physical_speaker(&mut self,slot:usize)->Result<Option<Speaker>,Self::Boundary>{Ok(self.actors[slot])}
    fn announce(&mut self,kind:Notice,slot:usize)->Result<(),Self::Boundary>{
        self.phase=i32::from(self.phase as i16);
        let message=message(kind,if self.multiplayer{0}else{self.draws+=1;self.random});
        self.messages.push(serde_json::json!([slot,4,message]));
        self.current=message;self.next_mode=0;self.next_message=message;
        self.phrase=1;self.length=48;self.sound=81+(message&255)-0x3d;
        self.hold=[90,73,90,63,69,78,54,60,66][usize::from((message&255)-0x3d)];
        Ok(())
    }
}

#[test]
#[ignore = "requires private original fort radio trace"]
fn fort_result_radio_matches_both_native_lanes_and_announce_continuations() {
    let path=std::env::var("NATIVE_FORT_RADIO_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    assert_eq!(text.lines().count(),1176);
    let mut messages=std::collections::BTreeSet::new();
    for (id,line) in text.lines().enumerate() {
        let row:Value=serde_json::from_str(line).unwrap();
        let n=|key:&str|row[key].as_i64().unwrap();
        assert_eq!(n("id"),id as i64);
        let lane=|key:&str|{let x=|i:usize|row[key][i].as_i64().unwrap();
            Lane{flag:x(0) as i32,counter:x(1) as u8,cursor:x(2) as usize,owner:x(3) as u8}};
        let mut radio=Radio{win:lane("win"),loss:lane("loss")};
        let mut r=Recorder{actors:[None;241],phase:n("phase") as i32,hold:n("hold") as i32,random:n("rand") as u16,
            multiplayer:n("mp")!=0,draws:0,messages:vec![],current:0x419,next_mode:17,next_message:19,phrase:65535,length:0,sound:0};
        for (i,slot) in [0usize,2,120,239].into_iter().enumerate(){r.actors[slot]=Some(Speaker{
            flags31:row["flags"][i].as_u64().unwrap() as u8,role:row["roles"][i].as_u64().unwrap() as i8});}
        radio.advance(&mut r).unwrap();
        let fields=|l:Lane|[i64::from(l.flag),i64::from(l.counter),l.cursor as i64];
        for (key,value) in [("winAfter",fields(radio.win)),("lossAfter",fields(radio.loss))] {
            assert_eq!(value,[0usize,1,2].map(|i|row[key][i].as_i64().unwrap()),"{key} {id}");
        }
        assert_eq!(r.messages.as_slice(),row["messages"].as_array().unwrap(),"messages {id}");
        for msg in r.messages{messages.insert(msg[2].as_u64().unwrap());}
        assert_eq!([i64::from(r.draws),i64::from(r.phase),i64::from(r.hold),i64::from(r.current),i64::from(r.next_mode),
            i64::from(r.next_message),i64::from(r.phrase),i64::from(r.length),i64::from(r.sound)],
            ["draws","phaseAfter","holdAfter","current","nextMode","nextMessage","phrase","length","sound"].map(n),"row {id}");
    }
    assert_eq!(messages.len(),9);
}
