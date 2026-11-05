//! Browser contextual wall/bridge orders after acceptance. This is not the
//!441d80 human mouse classifier or437a60 automatic-contact opcode12.
//! Mobile hardware uses the existing53290 literal-ground approach; fixed
//! hardware uses52b70. Their actual callbacks own W, finite paths and shots.
use crate::{original_orders_game::Event, Game};
impl Game {
    pub(crate) fn accept_native_context_ground(
        &mut self,
        index: usize,
        kind: &str,
        target: Option<usize>,
        click: Option<(f64, f64)>,
    ) -> bool {
        if !self.map.original
            || !self.native_route_mode
            || self.map.native_road_graph.is_none()
            || !self.owns_native_route(index)
        {
            return false;
        }
        let Some(actor) = self.units.get(index) else {
            return false;
        };
        if !matches!(actor.native_class7c, 1 | 2) || actor.native_hardware_slot.is_none() {
            return false;
        }
        let point = match kind {
            "attackWall" => {
                let Some(wall) = target.and_then(|id| self.wall(id)) else {
                    return false;
                };
                wall.center()
            }
            "attackBridge" => {
                let Some(p) = click.filter(|(x, y)| x.is_finite() && y.is_finite()) else {
                    return false;
                };
                if self.bridge_at_point(p.0, p.1).is_none() {
                    return false;
                }
                p
            }
            _ => return false,
        };
        let point = [(point.0 / 2.).floor() as i16, (point.1 / 2.).floor() as i16];
        let event = if actor.native_class7c == 2 {
            Event::classified(2, point, self.native_order_arguments[2])
        } else {
            crate::original_orders_game::building_target_event(point)
        };
        if !self.record_native_order_event(index, event) {
            return false;
        }
        //Keep the accepted UI caption separately. Do not clear an active inline
        //line, finite heads, W, dodge or constructor data before54160 runs.
        let actor = &mut self.units[index];
        actor.order = kind.into();
        actor.target = if kind == "attackWall" { target } else { None };
        actor.dest = Some((f64::from(point[0]) * 2., f64::from(point[1]) * 2.));
        actor.hold_position = false;
        actor.explicit_stop = false;
        true
    }
}
#[cfg(test)]
mod tests {
    use crate::*;
    fn field(kind: &str) -> Game {
        let mut g = Game::new(0, 1, 4901);
        g.map.original = true;
        g.map.width = 128;
        g.map.height = 128;
        g.map.tiles = vec![0; 16384];
        g.map.terrain_codes = vec![0; 16384];
        g.map.native_terrain_plane = vec![0; 16384];
        g.map.original_tiles = vec![0; 16384];
        g.map.bridge_base = vec![-1; 16384];
        g.map.territory = vec![0; 16384];
        g.map.native_road_graph = Some(original_route::RoadGraph {
            regions: vec![
                original_route::RoadRegion {
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
        g.walls.clear();
        g.sectors.clear();
        g.native_world = None;
        g.native_ambient = None;
        g.native_group_rosters.clear();
        g.ai_teams.clear();
        g.winner = None;
        g.spawn(kind, 0, 640., 640., 10, None);
        g.spawn("gatling", 1, 3900., 3900., 20, None);
        g.maintain_native_group_rosters();
        g.native_group_rosters.set_current_roster(0, 10);
        g.import_native_world(&Value::Null);
        g.native_route_mode = true;
        let slot = g.units[0].native_slot.unwrap();
        g.native_routes.get_mut().managed[slot] = true;
        g.import_walls(&json!({"walls":[{"id":9,"x":736.,"y":640.,"w":32.,"h":32.,"hp":1.,"nativeHp":0,"wallRecord":0,"wallBit":0,"cells":[2583]}]})).unwrap();
        g.map.terrain_codes[20 * 128 + 23] = 16;
        g.map.native_terrain_plane[20 * 128 + 23] = 16;
        g.events.clear();
        g
    }
    #[test]
    fn accepted_context_wall_records_real_native_command_without_early_line_cleanup() {
        for (kind, opcode) in [
            ("medium", 5),
            ("light", 5),
            ("heavy", 5),
            ("launcher", 5),
            ("cannon", 2),
            ("missile", 2),
            ("howitzer", 2),
        ] {
            let mut g = field(kind);
            let id = g.units[0].id;
            g.units[0].native_remaining48 = 17;
            g.units[0].original_motion =
                Some(original_motion::Segment::new(640., 640., (720., 640.)));
            let line = g.units[0]
                .original_motion
                .as_ref()
                .unwrap()
                .source_line_fields();
            let hw = g.units[0].native_hardware_slot.unwrap();
            let old = g.native_hardware_pool[hw].attributes.bytes;
            g.command(&json!({"kind":"attackWall","target":9,"ids":[id]}));
            assert_eq!(
                g.units[0].native_order5c, opcode,
                "{kind}: accepted contextual command must reach54160"
            );
            assert_eq!(
                g.units[0]
                    .original_motion
                    .as_ref()
                    .unwrap()
                    .source_line_fields(),
                line,
                "{kind}: callback owns path cleanup"
            );
            assert_eq!(g.units[0].native_remaining48, 17);
            assert_eq!(g.native_hardware_pool[hw].attributes.bytes, old);
            let m = g.native_group_rosters.members[g.units[0].native_slot.unwrap()]
                .as_ref()
                .unwrap();
            assert_eq!(
                (
                    m.promotion.target_x,
                    m.promotion.target_y,
                    m.promotion.target_aux
                ),
                if opcode == 5 {
                    (3456, 376, 328)
                } else {
                    (376, 328, 0)
                }
            );
        }
    }

    #[test]
    fn fixed_hardware_dispatches2_into_its_existing_w_record_and_releases_real_projectiles() {
        for kind in ["cannon", "missile", "howitzer"] {
            let mut g = field(kind);
            let id = g.units[0].id;
            let slot = g.units[0].native_slot.unwrap();
            let hw = g.units[0].native_hardware_slot.unwrap();
            g.native_order_arguments[2] = 123;
            g.command(&json!({"kind":"attackWall","target":9,"ids":[id]}));
            assert_eq!(g.units[0].native_order5c, 2);
            g.dispatch_recorded_native_order(0);
            assert_eq!(g.units[0].native_order5c, 0);
            assert_eq!(g.native_hardware_pool[hw].order.target, [376, 328]);
            assert_eq!(
                g.native_group_rosters.members[slot]
                    .as_ref()
                    .unwrap()
                    .flags31
                    & 16,
                16
            );
            assert_eq!(
                g.native_group_rosters.members[slot]
                    .as_ref()
                    .unwrap()
                    .promotion
                    .motion,
                3
            );
            assert_eq!(g.units[0].native_initiative_scan.flags71 & 32, 32);
            let mut fired = false;
            for _ in 0..90 {
                g.step(0.059);
                assert_eq!(g.native_route_boundary, None, "{kind}");
                fired |= !g.projectiles.is_empty();
                if fired {
                    break;
                }
            }
            assert!(fired, "{kind}: original fixed-gun release");
        }
    }
    #[test]
    fn fixed_hardware_outside_range_keeps_its_native_stationary_target_without_an_invented_approach(
    ) {
        for kind in ["cannon", "missile", "howitzer"] {
            let mut g = field(kind);
            g.import_walls(&json!({"walls":[{"id":10,"x":1408.,"y":640.,"w":32.,"h":32.,"hp":1.,"nativeHp":0,"cells":[2604]}]})).unwrap();
            let id = g.units[0].id;
            let slot = g.units[0].native_slot.unwrap();
            let w = g.units[0].native_hardware_slot.unwrap();
            g.command(&json!({"kind":"attackWall","target":10,"ids":[id]}));
            assert_eq!(g.units[0].native_order5c, 2);
            for _ in 0..90 {
                g.step(0.059);
                assert_eq!(g.native_route_boundary, None);
                assert_eq!([g.units[0].x, g.units[0].y], [640., 640.]);
                assert_eq!(g.native_routes.borrow().pool.links[slot].primary, 0);
                assert!(!g.projectiles.iter().any(|p| p.team == 0));
            }
            assert_eq!(g.native_hardware_pool[w].order.target, [712, 328]);
        }
    }
    #[test]
    fn accepted_context_bridge_preserves_real_click_until_dispatch_and_rejects_invalid_targets() {
        let mut g = field("medium");
        let id = g.units[0].id;
        g.map.bridge_base[20 * 128 + 26] = 0;
        g.map.bridge_hp = vec![600.];
        g.map.bridge_max_hp = vec![600.];
        g.command(&json!({"kind":"attackBridge","x":848.,"y":656.,"ids":[id]}));
        assert_eq!(g.units[0].native_order5c, 5);
        assert_eq!(g.native_ground_attack_point(0), Some([424, 328]));
        assert_eq!(g.units[0].dest, Some((848., 656.)));
        let before = g.units[0].native_order5c;
        g.command(&json!({"kind":"attackBridge","x":1200.,"y":1200.,"ids":[id]}));
        assert_eq!(g.units[0].native_order5c, before);
        g.command(&json!({"kind":"attackWall","target":999,"ids":[id]}));
        assert_eq!(g.units[0].native_order5c, before);
    }
    #[test]
    fn compatibility_explicit_event_and_weapon_owner_gates_are_preserved() {
        for mode in 0..7 {
            let mut g = field(if mode == 6 { "jeep" } else { "medium" });
            let id = g.units[0].id;
            let mut v = json!({"kind":"attackWall","target":9,"ids":[id]});
            match mode {
                0 => {
                    v["nativeEvent"] = json!(false);
                }
                1 => g.map.native_road_graph = None,
                2 => g.native_route_mode = false,
                3 => {
                    let slot = g.units[0].native_slot.unwrap();
                    g.native_routes.get_mut().managed[slot] = false;
                }
                4 => {
                    v["team"] = json!(1);
                }
                5 => g.walls[0].hp = 0.,
                _ => {}
            }
            g.command(&v);
            assert_eq!(g.units[0].native_order5c, 0, "mode{mode}");
            assert!(g.projectiles.is_empty());
        }
    }
    #[test]
    #[ignore = "requires the unmodified private campaign metadata; set Z_NATIVE_CAMPAIGN"]
    fn authored_wall_and_bridge_commands_use_the_existing_native_callbacks_and_real_impacts() {
        let path = std::env::var("Z_NATIVE_CAMPAIGN").expect("private campaign path");
        let campaign: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for mp in [false, true] {
            for (mission, xy, kind, target) in [
                (17, [2528., 1280.], "attackWall", 131),
                (7, [1120., 3104.], "attackWall", 407),
                (17, [2528., 1280.], "attackWall", 114),
                (7, [1120., 3104.], "attackWall", 351),
                (6, [1056., 3168.], "attackBridge", 0),
                (19, [608., 3008.], "attackBridge", 2),
            ] {
                let m = campaign["missions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|m| m["number"] == mission)
                    .unwrap();
                let mut g = Game::new(mission as usize - 1, 1, 4807);
                g.multiplayer = mp;
                g.original_rng = original_combat::OriginalRng::for_original_mission(None, mp);
                g.apply_original(m).unwrap();
                g.ai_teams.clear();
                if g.native_strategy.is_some() && !mp {
                    g.update_original_strategy(true);
                }
                let index = g
                    .units
                    .iter()
                    .position(|u| u.team == 0 && [u.x, u.y] == xy)
                    .unwrap();
                let id = g.units[index].id;
                let slot = g.units[index].native_slot.unwrap();
                let hw = g.units[index].native_hardware_slot.unwrap();
                assert!(g.owns_native_route(index));
                let p = if kind == "attackWall" {
                    g.wall(target).unwrap().center()
                } else {
                    let b = &m["bridges"][target];
                    (
                        b["x"].as_f64().unwrap() + b["w"].as_f64().unwrap() / 2.,
                        b["y"].as_f64().unwrap() + b["h"].as_f64().unwrap() / 2.,
                    )
                };
                g.command(
                    &json!({"kind":kind,"target":target,"x":p.0,"y":p.1,"ids":[id],"team":0}),
                );
                assert_eq!(
                    g.units[index].native_order5c, 5,
                    "mission{mission}/{kind}/{target}/mp{mp}: accepted native command"
                );
                let mut shots = false;
                let mut finite = false;
                let mut moved = false;
                let mut elapsed = 0;
                for tick in 1..=800 {
                    g.step(0.059);
                    elapsed = tick;
                    shots |= g.projectiles.iter().any(|p| p.team == 0);
                    finite |= g.native_routes.borrow().pool.links[slot].primary != 0
                        || g.units[index].native_remaining48 > 0;
                    moved |= [g.units[index].x, g.units[index].y] != xy;
                    let hp = if kind == "attackWall" {
                        g.walls.iter().find(|w| w.id == target).unwrap().hp
                    } else {
                        g.map.bridge_hp[target]
                    };
                    if g.native_route_boundary.is_some()
                        || hp < if kind == "attackWall" { 1. } else { 600. }
                    {
                        break;
                    }
                }
                let hp = if kind == "attackWall" {
                    g.walls.iter().find(|w| w.id == target).unwrap().hp
                } else {
                    g.map.bridge_hp[target]
                };
                println!("native-context mission={mission} target={kind}/{target} mp={mp} ticks={elapsed} hp={hp} finite={finite} moved={moved} shots={shots} xy={:?} Wtarget={:?} op={} phase={} remaining={} boundary={:?}",[g.units[index].x,g.units[index].y],g.native_hardware_pool[hw].order.target,g.units[index].native_order5c,g.units[index].native_phase35,g.units[index].native_remaining48,g.native_route_boundary);
                assert_eq!(g.native_route_boundary, None);
                assert_eq!(
                    g.native_hardware_pool[hw].order.target,
                    [(p.0 / 2.) as i16, (p.1 / 2.) as i16]
                );
                assert!(shots);
                assert!(hp < if kind == "attackWall" { 1. } else { 600. });
                if kind == "attackBridge" || target == 114 || target == 351 {
                    assert!(finite && moved, "native finite approach");
                } else {
                    assert!(!moved, "near command keeps its source position");
                }
            }
        }
    }
}
