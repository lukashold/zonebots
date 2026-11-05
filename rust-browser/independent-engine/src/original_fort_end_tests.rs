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

#[test]
#[ignore="requires private original fort palette selector and cache trace"]
fn original_fort_palette_selector_advances_as_a_signed_byte() {
    let path=std::env::var("NATIVE_FORT_PALETTE_TRACE").expect("private trace path");
    let source=std::fs::read_to_string(path).unwrap();
    let mut tables=Vec::new();
    let mut steps=0;
    for line in source.lines(){
        let r:Value=serde_json::from_str(line).unwrap();
        if r["kind"]=="table"{
            let table=raw(&r["table"]);
            assert_eq!(table.len(),21*63);
            assert_eq!(table[..63],raw(&r["base"]));
            tables.push(table);
            continue;
        }
        let n=|key:&str,i:usize|r[key][i].as_i64().unwrap();
        let mut regs=Registers{defeated_owner:r["owner"].as_u64().unwrap() as u8,
            palette_row:n("before",0) as i8,palette_cursor:n("before",1) as u8,..Default::default()};
        regs.advance_palette();
        assert_eq!([i64::from(regs.palette_row),i64::from(regs.palette_cursor as i8)],
                   [n("after",0),n("after",1)],"step {steps}");
        let selected=regs.palette_copy_row().map(|row|{
            let from=usize::from(row)*63;
            &tables[r["seed"].as_u64().unwrap() as usize][from..from+63]
        });
        let previous=raw(&r["previous"]);
        assert_eq!(selected.unwrap_or(&previous),raw(&r["applied"]),"step {steps}");
        assert_eq!([n("cache",0),n("cache",1),n("cache",2)],
                   [0x4488,if selected.is_some(){65534}else{0x2266},0x3377]);
        steps+=1;
    }
    assert_eq!((tables.len(),steps),(4,1376));
}
