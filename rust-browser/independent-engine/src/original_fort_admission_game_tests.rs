use super::*;
use super::super::{BuildingSlot, State};
use std::io::{BufRead, BufReader};

fn byte(hex: &str, offset: usize) -> u8 {
    u8::from_str_radix(&hex[2 * offset..2 * offset + 2], 16).unwrap()
}

#[test]
#[ignore = "private executed GOG fort admission trace; supply NATIVE_FORT_ADMISSION_TRACE"]
fn admitted_fort_world_writes_match_original_terrain_masks_and_build_bytes() {
    let path = std::env::var_os("NATIVE_FORT_ADMISSION_TRACE").expect("NATIVE_FORT_ADMISSION_TRACE");
    let rows = BufReader::new(std::fs::File::open(path).unwrap());
    let mut game = Game::new(0, 1, 0x422850);
    game.map.original = true;
    game.map.width = 128;
    game.map.height = 128;
    let mut contexts = 0;
    let mut invalidated = 0;
    for row in rows.lines() {
        let v: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        let n = |field: &str| v[field].as_i64().unwrap();
        if n("gateBefore") != 0 || matches!(n("visibleBefore"), 0 | 2) { continue; }
        let scenario = n("scenario") as usize;
        let team = n("side") as u8;
        let proxy = n("proxy") as usize;
        let hp = n("proxyHp") as i16;
        let base: [i16;2] = if team == 0 { [400,300] } else { [800,900] };
        game.map.native_terrain_plane = (0..16384).map(|i| [16,144,19,147,0,255][i % 6]).collect();
        game.map.terrain_codes = game.map.native_terrain_plane.iter().map(|&v| v & 127).collect();
        game.native_collision_masks.initialize(0);
        for i in 0..1296 { game.native_collision_masks.write(0x6323c8 + i as u32, (scenario * 31 + i * 19) as u8).unwrap(); }
        let mut state = State::empty();
        for slot in 0..35 {
            let mut raw = std::array::from_fn(|i| (scenario * 29 + (slot * 88 + i) * 13) as u8);
            raw[4] = 255;
            if slot == proxy {
                raw[..2].copy_from_slice(&base[0].to_le_bytes());
                raw[2..4].copy_from_slice(&base[1].to_le_bytes());
                raw[4] = 2;
                raw[30..32].copy_from_slice(&hp.to_le_bytes());
            }
            let record = native::Building {
                position: [i16::from_le_bytes(raw[..2].try_into().unwrap()), i16::from_le_bytes(raw[2..4].try_into().unwrap())],
                kind: raw[4] as i8, hp: i16::from_le_bytes(raw[30..32].try_into().unwrap()),
            };
            state.buildings.push(BuildingSlot {record, maximum:200, binding:None, fort_proxy:slot==proxy, repair:None, constructor:None, display:Default::default()});
            state.trail.known_builds[slot] = true;
            state.trail.builds[slot] = raw;
        }
        game.native_world = Some(state);
        let rng = game.original_rng.words;
        let slot = game.apply_native_fort_admission_world(team, base).unwrap();
        assert_eq!(slot, (hp!=0).then_some(proxy));
        assert_eq!(game.original_rng.words, rng);
        let terrain = v["terrain"].as_str().unwrap();
        for (i, &got) in game.map.native_terrain_plane.iter().enumerate() {
            assert_eq!(got, byte(terrain, i), "scenario {scenario}, terrain cell {i}");
            assert_eq!(game.map.terrain_codes[i], got & 127);
        }
        let masks = v["masks"].as_str().unwrap();
        for i in 0..1296 {
            assert_eq!(game.native_collision_masks.read(0x6323c8 + i as u32).unwrap(), byte(masks, i));
        }
        let state = game.native_world.as_ref().unwrap();
        let build = v["build"].as_str().unwrap();
        for (i, record) in state.buildings.iter().enumerate() {
            assert_eq!(record.record.kind as u8, byte(build, i * 88 + 4));
            for (j,&got) in state.trail.builds[i].iter().enumerate() { assert_eq!(got, byte(build, i * 88 + j)); }
        }
        contexts += 1;
        invalidated += usize::from(slot.is_some());
    }
    assert_eq!((contexts, invalidated), (64,40));
}
