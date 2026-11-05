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
