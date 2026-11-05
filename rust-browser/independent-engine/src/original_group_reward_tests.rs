use super::{apply, boost, clear_target_reports, World};
use serde_json::Value;

struct Raw {
    roster: Vec<u8>,
    aims: Vec<u8>,
    writes: Vec<Value>,
}

impl World for Raw {
    type Boundary = std::convert::Infallible;

    fn cooldown(&self) -> (u8, u8) { (self.roster[66], self.roster[67]) }
    fn set_count(&mut self, value: u8) {
        self.roster[67] = value;
        self.writes.push(serde_json::json!([0,value]));
    }
    fn member(&self, position: usize) -> Result<u8, Self::Boundary> { Ok(self.roster[19 + position]) }
    fn boost_aim(&mut self, slot: u8) -> Result<(), Self::Boundary> {
        let next = boost(self.aims[usize::from(slot)]);
        self.aims[usize::from(slot)] = next;
        self.writes.push(serde_json::json!([1,slot,next]));
        Ok(())
    }
    fn set_cooldown(&mut self, value: u8) {
        self.roster[66] = value;
        self.writes.push(serde_json::json!([2,value]));
    }
}

fn decode(value: &Value) -> Vec<u8> {
    value.as_str().unwrap().as_bytes().chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect()
}

#[test]
#[ignore = "requires private local instruction trace"]
fn original_reward_replays_all_ordered_writes_including_raw_alias_and_duplicate_members() {
    let path = std::env::var("NATIVE_GROUP_REWARD_TRACE").expect("private trace path");
    let file = std::fs::read_to_string(path).unwrap();
    let rows: Vec<Value> = file.lines().map(|s| serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(), 512);
    let mut applied = 0;
    for (id, pair) in rows.chunks_exact(2).enumerate() {
        let (before, after) = (&pair[0], &pair[1]);
        assert_eq!((before["id"].as_u64(), after["id"].as_u64()), (Some(id as u64), Some(id as u64)));
        assert_eq!((&before["stage"], &after["stage"]), (&Value::from("before"), &Value::from("after")));
        assert_eq!(before["group"], after["group"]);
        let mut raw = Raw { roster: decode(&before["roster"]), aims: decode(&before["aim"]), writes: vec![] };
        apply(&mut raw).unwrap();
        assert_eq!(raw.roster, decode(&after["roster"]), "roster {id}");
        assert_eq!(raw.aims, decode(&after["aim"]), "raw aim table {id}");
        assert_eq!(&raw.writes, after["writes"].as_array().unwrap(), "write order {id}");
        applied += usize::from(!raw.writes.is_empty());
    }
    assert_eq!(applied, 100);
}

#[test]
#[ignore = "requires private local fort removal trace"]
fn original_fort_cleanup_credits_only_the_last_reporting_leader() {
    let path = std::env::var("NATIVE_FORT_CREDIT_TRACE").expect("private trace path");
    let rows: Vec<Value> = std::fs::read_to_string(path).unwrap().lines()
        .map(|s| serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(), 128);
    let mut matches = [0;4];
    for (scenario, pair) in rows.chunks_exact(2).enumerate() {
        let (before, after) = (&pair[0], &pair[1]);
        assert_eq!((&before["stage"],&after["stage"]),(&Value::from("before"),&Value::from("after")));
        assert_eq!(before["scenario"],scenario);assert_eq!(after["scenario"],scenario);
        let mut raw: Vec<[u8;72]> = decode(&before["groups"]).chunks_exact(72)
            .map(|row| row.try_into().unwrap()).collect();
        assert_eq!(raw.len(),240);
        let credited: Vec<u8> = (0..3).filter_map(|victim| clear_target_reports(raw.iter_mut(), victim)).collect();
        let last: Vec<u8> = after["visits"].as_array().unwrap().iter().filter(|row| row[0]==0x44a5e0)
            .map(|row| row[1].as_u64().unwrap() as u8).collect();
        assert_eq!(credited,last,"{scenario}");
        assert_eq!(last.first().copied(),[None,Some(0),Some(3),Some(2)][scenario/16]);
        let native_after = decode(&after["groups"]);
        for (group, row) in raw.iter().enumerate() {
            assert_eq!(row[59],native_after[group*72+59],"source target byte {scenario}/{group}");
        }
        let objects=decode(&after["objects"]);
        assert_eq!(&objects[26*44+2..26*44+4],if scenario/16==1 {&[170,1]}else{&[0,0]});
        matches[scenario/16]+=1;
    }
    assert_eq!(matches,[16;4]);
}
