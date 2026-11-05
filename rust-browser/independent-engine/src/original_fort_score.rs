//! Score arithmetic observed in the campaign result callback.
const MISSION_FACTOR:[f32;21]=[
    0.,225.,661.,672.,731.,696.,455.,829.,827.,713.,1000.,
    861.,982.,1016.,1452.,872.,1096.,1139.,1336.,2651.,1217.,
];

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct Loss {
    pub(crate) count:i16,
    pub(crate) value:i32,
}

impl Loss{
    pub(crate) fn record(&mut self,value:i32){
        self.count=self.count.wrapping_add(1);
        self.value=self.value.wrapping_add(value);
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct ResultScore{
    pub(crate) points:i32,
    pub(crate) record:i32,
}

fn result_seconds(seconds:i32)->i32{
    let hours=seconds/3600;
    let rest=seconds.wrapping_sub(i32::from(hours as i16)*3600);
    let minutes=(rest/60) as i8;
    let second=rest.wrapping_sub(i32::from(minutes)*60) as i8;
    let total=i32::from(second)+(i32::from(minutes)+i32::from(hours as i8)*60)*60;
    if total==0{1}else{total}
}

pub(crate) fn group_value(pointer:u32)->Option<i32>{
    if !(0x615498..0x61b640).contains(&pointer){return Some(0);}
    let relative=pointer.checked_sub(0x619ae8)?;
    if relative%82!=0{return None;}
    super::original_stats::PRODUCTS.iter().find(|p|p.1==relative as usize/82).map(|p|p.5)
}

#[derive(Default)]
pub(crate) struct Tally{
    pub(crate) by_team:[Loss;4],
    pub(crate) unknown_value:bool,
}
impl Tally{
    pub(crate) fn retire(&mut self,team:u8,pointer:u32){
        let loss=&mut self.by_team[usize::from(team&3)];
        if let Some(value)=group_value(pointer){loss.record(value);}else{
            self.unknown_value=true;loss.record(0);
        }
    }
    pub(crate) fn opponent_losses(&self,local:usize)->Loss{
        self.by_team.iter().enumerate().filter(|(team,_)|*team!=local).fold(Loss::default(),|a,(_,b)|Loss{
            count:a.count.wrapping_add(b.count),value:a.value.wrapping_add(b.value),
        })
    }
}

/// The score reads the low signed words of the two full-width loss values.
/// The source uses single-precision operations and truncates after adding
/// the mission bonus. The per-mission record changes only on improvement.
pub(crate) fn score(mission:usize,seconds:i32,enemy:Loss,own:Loss,record:i32)->Option<ResultScore>{
    let factor=*MISSION_FACTOR.get(mission)?;
    let advantage=(i32::from(enemy.value as i16)-i32::from(own.value as i16)).max(0);
    let points=((factor*advantage as f32/result_seconds(seconds) as f32+(mission*1000) as f32) as i32).min(99_999);
    Some(ResultScore{points,record:record.max(points)})
}

/// The win screen rates signed low words against the mission's par time.
/// The two terms truncate separately before they are added to the rating.
pub(crate) fn rank(mission:usize,seconds:i32,enemy:Loss,own:Loss,base:i32)->Option<(i32,u8)>{
    let par=*MISSION_FACTOR.get(mission)?;
    if par==0.{return None;}
    let seconds=seconds as i16 as f32;
    let enemy=if enemy.value as i16==0{1.}else{enemy.value as i16 as f32};
    let own=own.value as i16 as f32;
    let kills=((enemy-own)*100./enemy)as i32;
    let speed=((par-seconds)*100./par)as i32;
    let rating=base.wrapping_add(speed).wrapping_add(kills).clamp(0,100);
    Some((rating,(rating/10).min(9)as u8))
}

#[cfg(test)]
mod tests{
    use super::*;
    use serde_json::Value;

    #[test]
    #[ignore="requires private native campaign rank trace"]
    fn rank_matches_original_win_screen_arithmetic(){
        let path=std::env::var("NATIVE_FORT_RESULT_RANK_TRACE").unwrap();
        let source=std::fs::read_to_string(path).unwrap();
        assert_eq!(source.lines().count(),960);
        for (id,line) in source.lines().enumerate(){
            let row:Value=serde_json::from_str(line).unwrap();
            assert_eq!(row["id"],id);
            let n=|key:&str|row[key].as_i64().unwrap() as i32;
            let loss=|key:&str|Loss{count:0,value:n(key)};
            let actual=rank(n("mission") as usize,n("seconds"),loss("enemy"),loss("own"),n("base"));
            assert_eq!(actual,Some((n("rating"),n("rank") as u8)),"rank case {id}");
        }
    }

    #[test]
    #[ignore="requires private native result-score trace"]
    fn result_score_matches_signed_original_values_and_record_writes(){
        let source=std::fs::read_to_string(std::env::var("NATIVE_FORT_RESULT_SCORE_TRACE").unwrap()).unwrap();
        assert_eq!(source.lines().count(),2310);
        for (id,line) in source.lines().enumerate(){
            let r:Value=serde_json::from_str(line).unwrap();
            assert_eq!(r["id"],id);
            let n=|key:&str|r[key].as_i64().unwrap() as i32;
            let loss=|key:&str|Loss{count:0,value:n(key)};
            let result=score(n("mission") as usize,n("seconds"),loss("enemy"),loss("own"),n("old")).unwrap();
            assert_eq!((result.points,result.record),(n("score"),n("best")),"case {id}");
        }
    }
    #[test]
    #[ignore="requires private group loss and RINFO valuation trace"]
    fn raw_group_loss_value_matches_both_native_score_branches(){
        let source=std::fs::read_to_string(std::env::var("NATIVE_FORT_LOSS_SCORE_TRACE").unwrap()).unwrap();
        assert_eq!(source.lines().count(),168);
        for line in source.lines(){
            let r:Value=serde_json::from_str(line).unwrap();
            let n=|key:&str|r[key].as_u64().unwrap() as usize;
            let get=|key:&str,i:usize|r[key][i].as_i64().unwrap() as i32;
            let local=n("local");let other=local^1;
            let mut tally=Tally::default();
            tally.by_team[other]=Loss{count:get("before",0) as i16,value:get("before",2)};
            tally.by_team[local]=Loss{count:get("before",1) as i16,value:get("before",3)};
            let gid=n("gid");
            let pointer=match gid{0..=18=>0x619ae8+gid as u32*82,19=>0x615497,_=>0x61b640};
            if n("gate")==1{tally.retire(n("team") as u8,pointer);}
            let their=tally.opponent_losses(local);let own=tally.by_team[local];
            assert_eq!([i32::from(their.count),i32::from(own.count),their.value,own.value],
                       std::array::from_fn(|i|get("after",i)),"case {}",n("id"));
            assert!(!tally.unknown_value);
        }
    }
}
