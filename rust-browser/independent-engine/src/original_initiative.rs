//! Local object initiative, based on GOG Z.exe 0044d920/0044d290/004578b0.
//! The original rotates directional scans through an animation/state machine.
//! Here an idle/arrived unit scans all directions every 0.35 simulation seconds;
//! this cadence and the explicit Stop hold are deliberate browser adaptations.
//! Native range quantization, nearest-object choice, infantry-only crewing and
//! exclusive object reservation follow the verified code. This is not full AI.
use super::{original_combat, stats, Game};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum Initiative {
    Flag(usize),
    Vehicle(usize),
}

/// 0044d290 uses native integer deltas >>3, then integer Euclidean distance.
/// 0044d920 requires distance*2 < unit[4], and unit[4]=(native_range>>3)*2.
pub fn object_distance(x: f64, y: f64, tx: f64, ty: f64) -> u32 {
    let dx = ((tx / 2.) as i32 - (x / 2.) as i32) >> 3;
    let dy = ((ty / 2.) as i32 - (y / 2.) as i32) >> 3;
    ((dx * dx + dy * dy) as f64).sqrt() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{json, Crew, Sector};

    fn game() -> Game {
        let mut g = Game::new(0, 1, 337);
        g.ai_teams.clear();
        g.map.original = true;
        g.map.world = 0;
        g.map.width = 24;
        g.map.height = 24;
        g.map.tiles = vec![0; 24 * 24];
        g.map.territory = vec![0; 24 * 24];
        g.map.bridge_base = vec![-1; 24 * 24];
        g.map.bridge_hp.clear();
        g.buildings.clear();
        g.units.clear();
        g.crates.clear();
        g.next_id = 1;
        g.sectors = vec![Sector {
            id: 0,
            x: 400.,
            y: 320.,
            owner: -1,
            home: false,
            counts_for_production: true,
        }];
        g.spawn("grunt", 0, 320., 320., 1, None);
        g.spawn("grunt", 1, 700., 700., 2, None);
        g
    }
    fn run(g: &mut Game, steps: usize) {
        for _ in 0..steps {
            g.step(0.1);
        }
    }

    #[test]
    fn idle_zero_hp_robot_walks_to_nearby_flag_for_either_team() {
        for team in [0, 1] {
            let mut g = game();
            g.units[0].team = team;
            g.units[1].team = 1 - team;
            g.units[0].hp = 0.;
            g.step(0.1);
            assert_eq!(g.units[0].initiative, Some(Initiative::Flag(0)));
            assert!(g.units[0].x > 320.);
            run(&mut g, 30);
            assert_eq!(g.sectors[0].owner, team);
        }
    }
    #[test]
    fn nearby_hardware_takes_exactly_one_squad_member() {
        let mut g = game();
        g.sectors[0].owner = 0;
        g.spawn("grunt", 0, 336., 320., 1, None);
        g.spawn("grunt", 0, 352., 320., 1, None);
        g.spawn("jeep", -1, 400., 320., 3, None);
        let vehicle = g.units.last().unwrap().id;
        g.step(0.1);
        assert_eq!(
            g.units
                .iter()
                .filter(|u| u.initiative == Some(Initiative::Vehicle(vehicle)))
                .count(),
            1
        );
        run(&mut g, 30);
        let car = g.units.iter().find(|u| u.id == vehicle).unwrap();
        assert_eq!(car.team, 0);
        assert!(car.crew.is_some());
        assert_eq!(
            g.units
                .iter()
                .filter(|u| u.team == 0 && stats(&u.kind).cls == 0)
                .count(),
            2
        );
    }
    #[test]
    fn explicit_squad_entry_does_not_overwrite_driver_or_consume_followers() {
        let mut g = game();
        g.sectors[0].owner = 0;
        g.units[0].x = 400.;
        g.units[0].hp = 27.;
        g.spawn("grunt", 0, 400., 320., 1, None);
        g.spawn("grunt", 0, 400., 320., 1, None);
        g.spawn("jeep", -1, 400., 320., 3, None);
        let vehicle = g.units.last().unwrap().id;
        let ids: Vec<usize> = g
            .units
            .iter()
            .filter(|u| u.team == 0)
            .map(|u| u.id)
            .collect();
        g.command(&json!({"kind":"enter","ids":ids,"target":vehicle}));
        g.step(0.1);
        let car = g.units.iter().find(|u| u.id == vehicle).unwrap();
        assert_eq!(car.crew.as_ref().unwrap().hp, 27.);
        assert_eq!(
            g.units
                .iter()
                .filter(|u| u.team == 0 && stats(&u.kind).cls == 0)
                .count(),
            2
        );
        assert!(g
            .events
            .iter()
            .any(|e| e["kind"] == "crewed" && e["type"] == "jeep"));
    }
    #[test]
    fn explicit_move_and_stop_override_local_initiative() {
        let mut g = game();
        let robot = g.units[0].id;
        g.command(&json!({"kind":"move","ids":[robot],"x":100,"y":320}));
        g.step(0.1);
        assert_eq!(g.units[0].initiative, None);
        assert_eq!(g.units[0].dest, Some((100., 320.)));
        g.command(&json!({"kind":"stop","ids":[robot]}));
        let x = g.units[0].x;
        run(&mut g, 40);
        assert_eq!(g.units[0].x, x);
        assert!(g.units[0].hold_position);
        assert_eq!(g.sectors[0].owner, -1);
        g.command(&json!({"kind":"move","ids":[robot],"x":x,"y":320}));
        run(&mut g, 50);
        assert_eq!(g.sectors[0].owner, 0);
    }
    #[test]
    fn chooses_nearest_object_and_releases_lost_vehicle_claim() {
        let mut g = game();
        g.spawn("jeep", -1, 360., 320., 3, None);
        let car = g.units.last().unwrap().id;
        g.step(0.1);
        assert_eq!(g.units[0].initiative, Some(Initiative::Vehicle(car)));
        g.sectors[0].owner = 0;
        let v = g.units.iter_mut().find(|u| u.id == car).unwrap();
        v.team = 1;
        v.crew = Some(Crew {
            kind: "grunt".into(),
            hp: 30.,
        });
        g.step(0.1);
        assert_eq!(g.units[0].initiative, None);
        assert!(g.units[0].path.is_empty());
    }
    #[test]
    fn does_not_crew_enemy_or_friendly_hardware_or_cross_a_wall() {
        let mut g = game();
        g.sectors[0].owner = 0;
        g.spawn("jeep", 0, 360., 320., 3, None);
        g.spawn("jeep", 1, 400., 320., 4, None);
        g.step(0.1);
        assert_eq!(g.units[0].initiative, None);
        let mut g = game();
        for y in 0..24 {
            g.map.tiles[y * 24 + 11] = 2;
        }
        run(&mut g, 10);
        assert_eq!(g.units[0].initiative, None);
        assert_eq!(g.sectors[0].owner, -1);
    }
    #[test]
    fn native_quantized_range_uses_strict_idle_boundary() {
        let mut g = game();
        g.sectors[0].x = 560.; // 240 browser pixels =120 native =15 quantized.
        assert_eq!(object_distance(320., 320., 560., 320.), 15);
        g.step(0.1);
        assert_eq!(g.units[0].initiative, None);
        g.sectors[0].x = 558.;
        run(&mut g, 4);
        assert_eq!(g.units[0].initiative, Some(Initiative::Flag(0)));
    }
}

impl Game {
    fn initiative_valid(&self, team: i32, goal: Initiative) -> bool {
        match goal {
            Initiative::Flag(id) => self
                .sectors
                .get(id)
                .map(|s| !s.home && s.counts_for_production && s.owner != team)
                .unwrap_or(false),
            Initiative::Vehicle(id) => self.units.iter().any(|u| {
                u.id == id
                    && u.team < 0
                    && u.crew.is_none()
                    && stats(&u.kind).cls > 0
                    && original_combat::alive(true, u.hp)
            }),
        }
    }
    pub(super) fn update_initiative(&mut self, dt: f64) {
        if !self.map.original {
            return;
        }
        // Another unit may capture the object while its previous claimant is walking.
        // Release it immediately, rather than endlessly chasing an occupied vehicle.
        for i in 0..self.units.len() {
            if let Some(goal) = self.units[i].initiative {
                let stopped_short = match goal {
                    Initiative::Flag(id) => self
                        .sectors
                        .get(id)
                        .map(|s| {
                            self.units[i].path.is_empty()
                                && (s.x - self.units[i].x).hypot(s.y - self.units[i].y) >= 22.
                        })
                        .unwrap_or(false),
                    Initiative::Vehicle(_) => false,
                };
                if !original_combat::alive(true, self.units[i].hp)
                    || !self.initiative_valid(self.units[i].team, goal)
                    || stopped_short
                {
                    let u = &mut self.units[i];
                    u.initiative = None;
                    u.order = "guard".into();
                    u.path.clear();
                    u.dest = None;
                    u.target = None;
                }
            }
        }
        self.initiative_clock -= dt;
        if self.initiative_clock > 0. {
            return;
        }
        self.initiative_clock = 0.35;
        let mut reserved: Vec<Initiative> =
            self.units.iter().filter_map(|u| u.initiative).collect();
        // An explicit boarding command reserves its target against automatic claimants.
        reserved.extend(
            self.units
                .iter()
                .filter(|u| u.order == "enter")
                .filter_map(|u| u.target.map(Initiative::Vehicle)),
        );
        for i in 0..self.units.len() {
            let u = self.units[i].clone();
            let d = stats(&u.kind);
            if u.team < 0
                || !original_combat::alive(true, u.hp)
                || u.hold_position
                || u.initiative.is_some()
                || d.speed <= 0.
                || !u.path.is_empty()
            {
                continue;
            }
            // Complete a player's destination before acquiring another objective.
            let arrived = u.order == "move"
                && u.dest
                    .map(|(x, y)| (x - u.x).hypot(y - u.y) <= 48.)
                    .unwrap_or(false);
            if u.order != "guard" && !arrived {
                continue;
            }
            let limit = ((d.range / 2.) as u32) >> 3;
            if limit == 0 {
                continue;
            }
            let mut candidates = vec![];
            for s in &self.sectors {
                let goal = Initiative::Flag(s.id);
                if !s.home
                    && s.counts_for_production
                    && s.owner != u.team
                    && !reserved.contains(&goal)
                {
                    let distance = object_distance(u.x, u.y, s.x, s.y);
                    if distance < limit {
                        candidates.push((distance, (u.x - s.x).hypot(u.y - s.y), goal, s.x, s.y));
                    }
                }
            }
            if d.cls == 0 {
                for v in &self.units {
                    let goal = Initiative::Vehicle(v.id);
                    if v.team < 0
                        && v.crew.is_none()
                        && stats(&v.kind).cls > 0
                        && original_combat::alive(true, v.hp)
                        && !reserved.contains(&goal)
                    {
                        let distance = object_distance(u.x, u.y, v.x, v.y);
                        if distance < limit {
                            candidates.push((
                                distance,
                                (u.x - v.x).hypot(u.y - v.y),
                                goal,
                                v.x,
                                v.y,
                            ));
                        }
                    }
                }
            }
            candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
            for (_, distance, goal, x, y) in candidates {
                if !self.los(u.x, u.y, x, y) {
                    continue;
                }
                let route = self.route_unit(&u,x,y);
                if route.is_empty() && distance >= 22. {
                    continue;
                }
                let unit = &mut self.units[i];
                unit.initiative = Some(goal);
                unit.dest = Some((x, y));
                unit.path = route;
                match goal {
                    Initiative::Flag(_) => {
                        unit.order = "move".into();
                        unit.target = None;
                    }
                    Initiative::Vehicle(id) => {
                        unit.order = "enter".into();
                        unit.target = Some(id);
                    }
                }
                reserved.push(goal);
                break;
            }
        }
    }
}
