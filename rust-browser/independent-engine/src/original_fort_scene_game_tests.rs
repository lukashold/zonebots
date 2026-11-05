use super::*;
use crate::original_group_lifecycle::Member;
use std::io::{BufRead, BufReader};

#[test]
#[ignore = "private executed GOG fort pan trace; supply NATIVE_FORT_PAN_TRACE"]
fn fort_pan_matches_384_original_camera_and_selection_contexts() {
    let path = std::env::var_os("NATIVE_FORT_PAN_TRACE").expect("NATIVE_FORT_PAN_TRACE");
    let mut g = Game::new(0, 1, 0x4229e0);
    let camera = Some(([700,900],[1024,768]));
    let bases = [[100,200],[320,370],[-32760,32765],[760,-32760]];
    let mut count = 0;
    for row in BufReader::new(std::fs::File::open(path).unwrap()).lines() {
        let v: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        let n = |field: &str| v[field].as_i64().unwrap();
        assert_eq!(count,n("scenario"));
        let selection = (n("selected") as usize).checked_sub(1);
        let group = n("group") as u8;
        let base = bases[n("owner") as usize - 1];
        g.native_view = camera;
        g.native_selected_slot = selection;
        g.native_selection.insertion = 19;
        g.native_feedback.local_team = n("local") as u8;
        g.native_group_rosters.pool[usize::from(group)].roster[9] = 255;
        if let Some(slot) = selection {
            g.native_group_rosters.members[slot] = Some(Member {
                slot:slot as u8, group, role:n("role") as i8, flags31:4, flags69:0, flags8a:0, flags9c:0, flagsb7:0,
                primary_path:false, secondary_path:false, cursor:0, contact_order:0, position:[0;2], reserved_object:None,
                promotion:Default::default(),
            });
        }
        let target = g.prepare_native_fort_pan(base, n("mode") as i32).unwrap();
        assert_eq!(target, std::array::from_fn(|i| v["position"][i].as_i64().unwrap() as i16));
        assert_eq!(g.native_group_rosters.pool[usize::from(group)].roster[9], n("group9") as u8);
        assert_eq!(g.native_selected_slot.map_or(0,|slot|slot+1), n("selector") as usize);
        assert_eq!(g.native_selection.insertion, n("insertion") as u8);
        assert_eq!(g.native_feedback.local_team, n("localAfter") as u8);
        assert_eq!(g.native_view, camera);
        count += 1;
    }
    assert_eq!(count,384);
}
