use super::{retire,Registers};
use serde_json::Value;

fn raw(row: &Value) -> Vec<u8> {
    row.as_str().unwrap().as_bytes().chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(),16).unwrap()).collect()
}

#[test]
#[ignore = "requires private native stage 17 trace"]
fn original_fort_retirement_preserves_the_other_team_bytes_and_returns_continue() {
    let trace = std::env::var("NATIVE_FORT_END_TRACE").expect("private trace path");
    let text = std::fs::read_to_string(trace).unwrap();
    assert_eq!(text.lines().count(),128);
    for (id,line) in text.lines().enumerate() {
        let row:Value = serde_json::from_str(line).unwrap();
        let before = raw(&row["before"]);
        let mut projected = before.clone();
        let n = |key:&str| row[key].as_u64().unwrap() as u8;
        assert_eq!(usize::from(n("id")),id);
        let mut visibility = before[2..6].try_into().unwrap();
        let mut activity = before[6..10].try_into().unwrap();
        let mut regs = Registers { scene_dirty:(id*3) as u8,defeated_owner:(id*5) as u8,
            palette_row:(id*7) as i8,palette_cursor:(id*13) as u8 };
        let returned = retire(n("owner"),&mut activity,&mut visibility,&mut regs).unwrap();
        projected[2..6].copy_from_slice(&visibility);
        projected[6..10].copy_from_slice(&activity);
        assert_eq!(projected,raw(&row["after"]),"row {id}");
        assert_eq!([u8::from(returned),regs.scene_dirty,regs.defeated_owner,regs.palette_row as u8,regs.palette_cursor],
            ["returnAl","dirty","removedOwner","paletteRow","paletteCursor"].map(n));
        assert_eq!(row["rngUnchanged"],true);
    }
}
#[test]
#[ignore="requires private source scoreboard callback trace"]
fn original_score_clock_gates_one_pulse_and_main_clears_the_dirty_byte() {
    use super::{Registers,ScoreClock};
    let path=std::env::var("NATIVE_FORT_SCORE_TRACE").expect("private trace path");
    let source=std::fs::read_to_string(path).unwrap();
    assert_eq!(source.lines().count(),768);
    for (i,line) in source.lines().enumerate(){
        let r:serde_json::Value=serde_json::from_str(line).unwrap();
        assert_eq!(r["id"],i);
        let n=|key:&str,slot:usize|r[key][slot].as_i64().unwrap();
        let mut clock=ScoreClock{subtick:n("before",0) as i16,seconds:n("before",1) as i32};
        let mut registers=Registers{scene_dirty:n("before",2) as u8,..Default::default()};
        clock.ui_callback(registers.scene_dirty);
        assert_eq!([i64::from(clock.subtick),i64::from(clock.seconds),i64::from(registers.scene_dirty)],
                   std::array::from_fn(|j|n("afterUi",j)),"UI row {i}");
        if r["main"]==true{registers.begin_main();}
        assert_eq!(i64::from(registers.scene_dirty),r["afterMain"].as_i64().unwrap(),"Main row {i}");
    }
}
