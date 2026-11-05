use super::{feedback, Game};
use serde_json::Value;

#[test]
#[ignore = "requires private original retirement portrait trace"]
fn original_retirement_notice_matches_live_portrait_and_crt_controls() {
    let path=std::env::var("NATIVE_RETIREMENT_NOTICE_TRACE").expect("private trace path");
    let mut g=Game::new(0,1,12);
    let mut requests=0;
    let mut admitted=0;
    let mut rows=0;
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let row:Value=serde_json::from_str(line).unwrap();
        let n=|name:&str| row[name].as_i64().unwrap();
        assert_eq!(n("id"),rows);
        rows+=1;
        g.events.clear();
        g.multiplayer=n("mp")!=0;
        let owner=n("owner") as u8;
        g.native_feedback=feedback::State {
            scene_gate:n("gate") as i32,local_team:n("local") as u8,
            portrait_mode:n("modeBefore") as u8,current_message:n("messageBefore") as i32,
            next_mode:17,next_message:19,ui_clock:30,phrase_counter:19,phrase_frame:23,start_clock:47,
            ..Default::default()
        };
        let next=(n("rand") as u32)<<16;
        let seed=next.wrapping_sub(2531011).wrapping_mul(0xb9b33155);
        g.native_cosmetic_rng=seed;
        g.native_retirement_credit_for_listener(5,owner,owner^1);
        let f=&g.native_feedback;
        let tuple=[i64::from(f.speaker==Some(5)), i64::from(f.portrait_mode), i64::from(f.current_message),
            i64::from(f.next_mode), i64::from(f.next_message), i64::from(f.phrase_index as u16),
            i64::from(f.phrase_length), i64::from(f.phrase_counter), i64::from(f.phrase_frame), i64::from(f.start_clock)];
        assert_eq!(tuple,["speaker","mode","message","nextMode","nextMessage","phrase","length","counter","frame","start"].map(n),"row {}",n("id"));
        assert_eq!(g.native_cosmetic_rng,if n("draws")==1 {next}else{seed});
        assert_eq!(g.events.len() as i64,n("voiceCalls"));
        requests+=i64::from(f.speaker==Some(5));
        if let Some(voice)=g.events.first() {
            admitted+=1;
            assert_eq!(voice["message"],n("request"));
            assert_eq!(voice["portraitType"],n("portrait"));
            assert_eq!(voice["maximum"],n("maximum"));
            assert_eq!(voice["clip"],format!("ROB{}",n("sound")));
            assert_eq!(voice["playAudio"],false);
        }
    }
    assert_eq!((rows,requests,admitted),(448,112,84));
}
