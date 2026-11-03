use super::*;

// Small public fixtures isolate target choice from campaign data, AI orders,
// automatic boarding and return fire by the two target robots.
fn engagement(original: bool) -> Game {
    let mut game = Game::new(0, 1, 117);
    game.map.original = original;
    game.map.tiles.fill(0);
    game.map.territory.fill(0);
    game.map.bridge_base.fill(-1);
    game.map.bridge_hp.clear();
    game.sectors = vec![Sector {
        id: 0, x: 1500., y: 1000., owner: 0, home: true,
        counts_for_production: true,
    }];
    game.buildings.clear();
    game.objects.clear();
    game.crates.clear();
    game.units.clear();
    game.effects.clear();
    game.events.clear();
    game.ai_teams.clear();
    game.ai_clock = 100.;
    game.next_id = 1;
    for (team, x, y) in [(0, 320., 320.), (1, 400., 320.), (1, 500., 320.)] {
        game.spawn("grunt", team, x, y, game.next_id, None);
        let unit = game.units.last_mut().unwrap();
        unit.hold_position = true;
        unit.cooldown = if team == 0 { 0. } else { 100. };
    }
    game
}

fn assert_shot_at(game: &mut Game, x: f64, y: f64) {
    if game.map.original {
        //Native phase0 initializes idle first. Automatic acquisition belongs
        //to a later directional search, rather than the first browser frame.
        for _ in 0..1024{if game.units[0].native_fire.is_some(){break;}game.step(0.059);}
        let target=game.units.iter().find(|u|u.team==1&&u.x==x&&u.y==y).unwrap().id;
        assert_eq!(game.units[0].native_fire.as_ref().unwrap().target,target);
        assert!(!game.events.iter().any(|e|e["kind"]=="shot"),"acquisition is not a hit");
        for _ in 0..30 {game.step(0.059);if game.events.iter().any(|e|e["kind"]=="shot"){break;}}
        assert_eq!(game.events.iter().filter(|e|e["kind"]=="shot").count(),1);
        assert_eq!(game.units[0].native_fire.as_ref().unwrap().target,target);return;
    }
    let shots: Vec<_> = game.effects.iter().filter(|effect| effect.kind == "shot").collect();
    assert_eq!(shots.len(), 1, "Only the command recipient should fire");
    assert_eq!((shots[0].x2, shots[0].y2), (x, y));
}

#[test]
fn manual_unit_target_wins_over_a_nearer_enemy_in_weapon_range() {
    for original in [false, true] {
        let mut game = engagement(original);
        let (attacker, chosen) = (game.units[0].id, game.units[2].id);
        let near_hp = game.units[1].hp;
        let chosen_hp = game.units[2].hp;
        game.command(&json!({"ids":[attacker],"kind":"attack","target":chosen}));
        game.step(0.059);
        assert_shot_at(&mut game, 500., 320.);
        assert_eq!(game.units[1].hp, near_hp, "The closer unit must not steal the aimed shot");
        if !original {
            assert_eq!(game.units[2].hp, chosen_hp - stats("grunt").damage);
        }
    }
}

#[test]
fn building_target_receives_fire_even_with_an_enemy_unit_nearby() {
    for original in [false, true] {
        for owner in [-1, 1] {
            let mut game = engagement(original);
            game.units.pop();
            let building = Building {
                id: 0, x: 468., y: 288., w: 64., h: 64., kind: "robot".into(),
                team: owner, sector: 0, stars: 1, hp: 1000., max_hp: 1000.,
                product: None, progress: 0., ready_guns: vec![], gate: None,
                orientation: String::new(), remaining_work: None, completion_ticks:0, raw_base: None,
            };
            game.buildings.push(building);
            let attacker = game.units[0].id;
            let near_hp = game.units[1].hp;
            game.command(&json!({"ids":[attacker],"kind":"attackBuilding","target":0}));
            game.step(0.059);
            if original{
                //The native rifle bullet pool has no BUILD contact callback.
                //A grunt without grenades cannot inflict instant point damage.
                assert_eq!(game.buildings[0].hp,1000.);
                assert!(!game.events.iter().any(|e|e["kind"]=="shot"));
            }else{assert_eq!(game.buildings[0].hp,1000.-stats("grunt").damage);}
            assert_eq!(game.units[1].hp, near_hp);
            if !original{assert!(game.events.iter().any(|event|event["kind"]=="shot"&&event["team"]==0));}
        }
    }
}

#[test]
fn out_of_range_manual_target_keeps_pursuit_and_nearby_return_fire() {
    for original in [false, true] {
        let mut game = engagement(original);
        game.units[2].x = 800.;
        let (attacker, chosen) = (game.units[0].id, game.units[2].id);
        let chosen_hp = game.units[2].hp;
        game.command(&json!({"ids":[attacker],"kind":"attack","target":chosen}));
        game.step(0.1);game.step(0.1);
        assert_shot_at(&mut game, 400., 320.);
        assert_eq!(game.units[2].hp, chosen_hp);
        assert!(!game.units[0].path.is_empty());
        if original {
            assert!(!game.units[0].moving,"Native infantry attack phases cannot also advance the walking handler");
            assert_eq!(game.units[0].x,320.);
        }else{
            assert!(game.units[0].moving);
            assert!(game.units[0].x > 320.);
        }
    }
}

#[test]
fn manual_unit_priority_respects_line_of_sight() {
    for original in [false, true] {
        let mut game = engagement(original);
        game.units[2].x = 320.;
        game.units[2].y = 500.;
        game.map.tiles[12 * game.map.width + 10] = 2;
        let (attacker, chosen) = (game.units[0].id, game.units[2].id);
        let chosen_hp = game.units[2].hp;
        game.command(&json!({"ids":[attacker],"kind":"attack","target":chosen}));
        game.step(0.059);
        assert_shot_at(&mut game, 400., 320.);
        assert_eq!(game.units[2].hp, chosen_hp);
    }
}

#[test]
fn automatic_targeting_and_ground_attack_still_choose_the_nearest_enemy() {
    for original in [false, true] {
        for ground_order in [false, true] {
            let mut game = engagement(original);
            if ground_order {
                let attacker = game.units[0].id;
                game.command(&json!({"ids":[attacker],"kind":"attack","x":650,"y":500}));
            }
            game.step(0.059);
            assert_shot_at(&mut game, 400., 320.);
        }
    }
}
