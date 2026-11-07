use super::*;
fn field() -> Game {
    let mut g = Game::new(0, 1, 457590);
    g.map.original = true;
    g.map.width = 64;
    g.map.height = 64;
    g.map.tiles = vec![0; 4096];
    g.map.terrain_codes = vec![0; 4096];
    g.map.native_terrain_plane = vec![0; 16384];
    g.map.bridge_base = vec![-1; 4096];
    g.map.native_road_graph = Some(crate::original_route::RoadGraph {
        regions: vec![
            crate::original_route::RoadRegion {
                bounds: [0; 4],
                waypoint: 0,
                next: vec![255; 96]
            };
            96
        ],
    });
    g.units.clear();
    g.buildings.clear();
    g.objects.clear();
    g.crates.clear();
    g.sectors.clear();
    g.walls.clear();
    g.native_world = None;
    g.native_group_rosters = Default::default();
    g.ai_teams.clear();
    g.native_route_mode = true;
    g.winner = None;
    g.spawn("grunt", 0, 640., 640., 1, None);
    g.spawn("grunt", 0, 704., 640., 1, None);
    g.maintain_native_group_rosters();
    g.native_group_rosters.set_current_roster(0, 1);
    g.native_group_rosters.set_group_angle(0, 1, 0);
    g.import_native_world(&crate::Value::Null);
    let slot = g.units[1].native_slot.unwrap();
    g.native_routes.get_mut().managed[slot] = true;
    let group = g.native_group_rosters.current_roster.unwrap();
    g.native_group_rosters.pool[group].roster[63] = 0;
    g.native_group_rosters.pool[group].roster[9] = 1;
    g.native_group_rosters.pool[group].roster[59] = 255;
    g.start_native_walking_direct(1, [350, 320]);
    let u = &mut g.units[1];
    u.native_phase35 = 7;
    u.native_frame32 = 1;
    u.native_timer3c = 0;
    u.native_speed8b = 0;
    u.native_field62 = 1;
    u.native_flags64 &= !16;
    g.native_callback_mode = 0;
    g
}
fn scenario(n: usize) -> Game {
    let mut g = field();
    let slot = g.units[1].native_slot.unwrap();
    let group = g.native_group_rosters.current_roster.unwrap();
    let x = 304 + [0, 4, -4, 16, -16, 48, -48, 64][n % 8];
    let y = 335 + [0, 7, -7, 40][n / 8 % 4];
    {
        let u = &mut g.units[1];
        u.x = f64::from(x) * 2.;
        u.y = f64::from(y) * 2.;
        u.native_remaining48 = 0;
        u.native_frame32 = (n * 13 % 300) as i16;
        u.native_timer3c = (n % 5) as i8;
        u.native_speed8b = (n % 13) as i8;
        u.native_phase35 = 7;
        u.angle = native_math::angle((n % 8) as u8);
        u.native_flags64 = 0;
        u.native_flags9c = 0;
        u.native_initiative_scan.flags71 = 0;
        u.native_axis81 = 0;
        u.motion_bonus = 0;
        u.native_directiona2 = 0x34;
        u.native_directiona3 = 0x12;
        u.path.clear();
        let mut line = crate::original_motion::Segment::new(u.x, u.y, (u.x + 2., u.y));
        line.set_source_fixture([0, 123, 456, 789]);
        u.original_motion = Some(line);
    }
    g.native_group_rosters.members[slot]
        .as_mut()
        .unwrap()
        .flags31 = 4;
    g.native_group_rosters.pool[group].roster[17] = 6;
    g.native_hardware_dummy.field17 = 0;
    g.native_routes.get_mut().pool = Default::default();
    if n % 256 >= 128 {
        let head = g
            .native_routes
            .get_mut()
            .pool
            .allocate_goal([x + 12, y + 10])
            .unwrap();
        g.native_routes.get_mut().pool.links[slot].primary = head;
        g.units[1].path = g.native_owned_route_view(1).unwrap();
    }
    g
}
fn project(g: &Game) -> Vec<i64> {
    let slot = g.units[1].native_slot.unwrap();
    let u = &g.units[1];
    let line = u.original_motion.as_ref().unwrap().source_line_fields();
    let paths = g.native_routes.borrow();
    let head = paths.pool.links[slot].primary;
    let mut values = vec![
        (u.x / 2.) as i64,
        (u.y / 2.) as i64,
        i64::from(u.native_frame32),
        i64::from(u.native_phase35),
        i64::from(u.native_timer3c),
        i64::from(original_combat::facing(u.angle)),
        i64::from(
            g.native_group_rosters.members[slot]
                .as_ref()
                .unwrap()
                .flags31,
        ),
    ];
    values.extend(line.map(i64::from));
    values.extend([
        i64::from(u.native_axis81),
        i64::from(u.motion_bonus as u8),
        i64::from(i16::from_le_bytes([
            u.native_directiona2 as u8,
            u.native_directiona3 as u8,
        ])),
        i64::from(head),
        i64::from(g.native_hardware_dummy.field17),
    ]);
    if let Some(node) = paths.pool.node(head) {
        values.extend([
            i64::from(node.point[0]),
            i64::from(node.point[1]),
            i64::from(node.next),
        ]);
    }
    values
}
fn compare_coupled(name: &str, mode: u8, expected_count: usize, expected_hash: u32) {
    let rows = std::env::var_os(name).map(|path| {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<crate::Value>(l).unwrap())
            .collect::<Vec<_>>()
    });
    if let Some(rows) = &rows {
        assert_eq!(rows.len(), expected_count);
    }
    let mut hash = 2166136261u32;
    let mut count = 0;
    for n in 0..if mode == 2 { 512 } else { 256 } {
        let mut g = scenario(n);
        if mode == 3 {
            g.map.terrain_codes.fill(19);
            g.map.native_terrain_plane.fill(19);
        }
        if n >= 256 {
            let u = &mut g.units[1];
            u.kind = "jeep".into();
            u.native_class7c = 1;
            u.native_family30 = Some(34);
            u.native_phase35 = 9;
        }
        for tick in 0..=if matches!(mode, 1 | 3) { 12 } else { 0 } {
            if tick > 0 && g.units[1].native_phase35 != 7 {
                break;
            }
            g.units[1].native_field62 = if tick == 0 { 0 } else { (n % 4 + 1) as u8 };
            if mode == 2 {
                assert_eq!(g.native_common_follow_for_test(1), Ok(()));
            } else {
                assert!(g.dispatch_native_follow(1));
            }
            assert_eq!(g.native_route_boundary, None, "scenario{n}/tick{tick}");
            let values = project(&g);
            if let Some(rows) = &rows {
                let row = &rows[count];
                assert_eq!(row["scenario"], crate::json!(n));
                if matches!(mode, 1 | 3) {
                    assert_eq!(row["tick"], crate::json!(tick));
                }
                assert_eq!(
                    crate::json!(values),
                    row["values"],
                    "scenario{n}/tick{tick}"
                );
            }
            for v in [n as i64, tick as i64].into_iter().chain(values) {
                hash = (hash ^ (v as u32)).wrapping_mul(16777619);
            }
            count += 1;
        }
    }
    assert_eq!(count, expected_count);
    assert_eq!(hash, expected_hash, "real coupled original output");
}
// Numeric digests were produced by the original EXE with no intercepted
// leaves. Local traces optionally compare each field, scenario and callback.
#[test]
fn coupled_original_follower_start_matches_real_path_and_line_leaves() {
    compare_coupled("Z_NATIVE_COUPLED_FOLLOW_START", 0, 256, 3464735893);
}
#[test]
fn coupled_original_follower_motion_matches_real_callbacks() {
    compare_coupled("Z_NATIVE_COUPLED_FOLLOW_MOTION", 1, 976, 467180558);
}
#[test]
fn follower_callback_moves_once_before_rejoining_and_restores_animation() {
    let mut g = field();
    assert!(g.dispatch_native_follow(1));
    assert_eq!(g.native_route_boundary, None);
    assert_eq!((g.units[1].x, g.units[1].y), (702., 640.));
    assert_eq!(
        (g.units[1].native_frame32, g.units[1].native_timer3c),
        (2, 10)
    );
    assert_eq!(g.units[1].native_phase35, 2);
    let slot = g.units[1].native_slot.unwrap();
    assert_ne!(g.native_routes.borrow().pool.links[slot].primary, 0);
    assert!(g.units[1].original_motion.is_some());
}
#[test]
fn movement_suppression_does_not_suppress_the_phase7_animation_counter() {
    let mut g = field();
    g.units[1].native_remaining48 = 0;
    g.units[1].original_motion = None;
    let before = (g.units[1].x, g.units[1].y);
    assert!(g.dispatch_native_follow(1));
    assert_eq!(g.native_route_boundary, None);
    assert_eq!((g.units[1].x, g.units[1].y), before);
    assert_eq!(
        (g.units[1].native_frame32, g.units[1].native_timer3c),
        (2, 10)
    );
}

#[test]
fn coupled_original_hardware_follow_uses_real_path_and_line_leaves() {
    compare_coupled("Z_NATIVE_COUPLED_FOLLOW_DIRECT", 2, 512, 284322126);
}

#[test]
fn coupled_original_follower_blocked_callbacks_match_real_contact_handoffs() {
    compare_coupled("Z_NATIVE_COUPLED_FOLLOW_BLOCKED", 3, 868, 2836574662);
}
