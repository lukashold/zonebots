use super::*;

const DIRECTIONS: [[i16; 2]; 8] = [
    [13, 0], [3, 13], [-13, 0], [-3, 13],
    [13, -3], [0, -13], [-13, -3], [-3, -13],
];

fn dodge_field(vehicle: bool, route: bool, blocked: bool) -> (Game, u32) {
    let mut game = Game::new(0, 1, 0x518e0);
    game.map.original = true;
    game.map.width = 64;
    game.map.height = 64;
    game.map.tiles = vec![0; 4096];
    game.map.terrain_codes = vec![0; 4096];
    game.map.native_terrain_plane = vec![0; 16384];
    game.map.native_road_graph = Some(original_route::RoadGraph { regions: vec![] });
    game.units.clear();
    game.buildings.clear();
    game.objects.clear();
    game.crates.clear();
    game.walls.clear();
    game.sectors.clear();
    game.native_group_rosters.clear();
    game.native_route_mode = true;
    game.spawn(if vehicle { "jeep" } else { "grunt" }, 0, 1026., 1026., 1, None);
    game.maintain_native_group_rosters();
    game.import_native_world(&Value::Null);
    let slot = game.units[0].native_slot.unwrap();
    game.native_group_rosters.observe_unit_context(slot);
    let member = game.native_group_rosters.members[slot].as_mut().unwrap();
    member.role = 0;
    member.flags31 = 4;
    member.flags69 = 0;
    game.prepare_native_ambient_queries();
    let unit = &mut game.units[0];
    unit.native_phase35 = if vehicle { 58 } else { 1 };
    unit.native_speed8b = 7;
    unit.native_timer3c = 2;
    unit.native_direction34 = 255;
    unit.native_flags64 = 0;
    unit.native_counter66 = 5;
    unit.native_counter67 = 7;
    unit.native_contact_pausebc = 1;
    unit.native_terrain_tag63 = 0;
    if let Some(w) = unit.native_hardware_slot {
        let mut record = game.native_hardware_pool[w].order;
        record.field17 = 0;
        game.native_hardware_pool[w].set_order(record);
    }
    if blocked {
        game.map.terrain_codes.fill(19);
        game.map.native_terrain_plane.fill(19);
    }
    let head = if route {
        let head = game.native_routes.get_mut().pool.allocate_goal([760, 652]).unwrap();
        game.native_routes.get_mut().pool.links[slot].primary = head;
        game.units[0].path = game.native_owned_route_view(0).unwrap();
        head
    } else {
        0
    };
    (game, head)
}

#[test]
fn authored_dodge_movement_matches_872_original_caller_rows() {
    let trace = std::env::var_os("Z_NATIVE_DODGE_MOVEMENT_TRACE").map(|path| {
        std::fs::read_to_string(path).unwrap().lines().map(|line| {
            serde_json::from_str::<Value>(line).unwrap()
        }).collect::<Vec<_>>()
    });
    let mut hash = 2166136261u32;
    let mut rows = 0;
    for scenario in 0..128 {
        let vehicle = scenario >= 64;
        let route = scenario & 32 != 0;
        let blocked = scenario & 16 != 0;
        let speed = if scenario & 8 == 0 { 1 } else { 4 };
        let [dx, dy] = DIRECTIONS[scenario % 8];
        let (mut game, head) = dodge_field(vehicle, route, blocked);
        let slot = game.units[0].native_slot.unwrap();
        game.install_native_dodge(0, u8::from(vehicle), [513 + dx, 513 + dy], true);
        for tick in 0..=16 {
            let step = if tick == 0 { 0 } else { speed };
            game.units[0].native_field62 = step;
            assert!(game.update_native_dodge(0, tick, 1));
            assert_eq!(game.native_route_boundary, None, "scenario {scenario} tick {tick}");
            let u = &game.units[0];
            let hw = u.native_hardware_slot.map_or(0, |w| game.native_hardware_pool[w].order.field17);
            let has_head = head != 0 && game.native_routes.borrow().pool.links[slot].primary == head;
            let values = [
                (u.x / 2.) as i32, (u.y / 2.) as i32, i32::from(u.native_phase35),
                i32::from(u.native_flags64), i32::from(u.native_remaining48), i32::from(u.native_axis81),
                i32::from(u.native_bucket38), i32::from(original_combat::facing(u.angle)),
                i32::from(u.motion_bonus), i32::from(hw), i32::from(u.native_pose7e),
                i32::from(u.native_contact_pausebc), i32::from(u.native_counter66), i32::from(u.native_counter67),
                i32::from(has_head), i32::from(u.native_frame32), i32::from(u.native_timer3c),
            ];
            // The hardware reference runs 4502f0 before the separate tread
            // and smoke callback. Its preserved animation fields are excluded.
            let fields = if vehicle { &values[..15] } else { &values[..] };
            if let Some(trace) = &trace {
                let row = &trace[rows];
                let expected = row["fields"].as_array().unwrap();
                assert_eq!(row["scenario"], json!(scenario));
                assert_eq!(row["tick"], json!(tick));
                assert_eq!(json!(fields), json!(&expected[..fields.len()]), "scenario {scenario} tick {tick}");
            }
            for value in [scenario as i32, tick as i32, i32::from(step)].iter().chain(fields) {
                hash = (hash ^ *value as u32).wrapping_mul(16777619);
            }
            rows += 1;
            if u.native_flags64 & 64 == 0 { break; }
        }
    }
    assert_eq!((rows, hash), (872, 1229059787), "ZNativeDodgeMovementOracle");
}
