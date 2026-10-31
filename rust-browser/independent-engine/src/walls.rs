//! Destructible scenery imported separately from the immutable cliff mask.
//! A* routing and immediate shot contact are browser-engine adapters. Original
//! placement, artwork and durability metadata come from the local extractor.
use super::{original_combat, original_terrain, pursuit, Fx, Game, Unit, TILE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wall {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub hp: f64,
    #[serde(default)] pub max_hp: f64,
    #[serde(default)] pub cells: Vec<usize>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub object_type: Option<usize>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub sprite: Option<u16>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub destroyed_sprite: Option<u16>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub source_offset: Option<usize>,
    #[serde(default)] pub wall_record: Option<usize>,
    #[serde(default)] pub wall_bit: Option<usize>,
    #[serde(default)] pub height: u8,
    #[serde(default)] pub native_hp: Option<i16>,
    #[serde(default)] pub neighbor_mask: u16,
    #[serde(default)] pub variant: u8,
    #[serde(default)] pub wall_tile_index: Option<u16>,
    #[serde(default, skip_serializing)] pub ground_tiles: Vec<u16>,
    #[serde(default, skip_serializing)] pub ground_codes: Vec<u8>,
    #[serde(default, skip_serializing)] pub ground_classes: Vec<u8>,
}

impl Wall {
    pub fn alive(&self) -> bool { self.hp > 0. }
    pub(crate) fn point_from(&self, x: f64, y: f64) -> (f64, f64) {
        (x.clamp(self.x, self.x + self.w), y.clamp(self.y, self.y + self.h))
    }
    pub(crate) fn center(&self) -> (f64, f64) { (self.x + self.w / 2., self.y + self.h / 2.) }
}

/// Primary projectile families capable of breaking stone; infantry grenade
/// eligibility remains separate from primary gun ammunition.
pub fn primary_capable(kind: &str) -> bool {
    matches!(kind, "tough" | "light" | "medium" | "heavy" | "launcher" | "cannon" | "missile" | "howitzer")
}
//0044a040 terrain orders route classes0 and16 through inventory+0x99 to
// state49. Pyro's terrain-table2 likewise prefers its grenade inventory. The
// common engine contact/cooldown adapter does not model squad donor selection.
pub fn grenade_capable(kind: &str) -> bool { matches!(kind, "grunt" | "psycho" | "sniper" | "laser" | "pyro") }
pub fn capable(unit: &Unit) -> bool { primary_capable(&unit.kind) || grenade_capable(&unit.kind) && unit.grenades > 0 }

impl Game {
    pub(crate) fn import_walls(&mut self, mission: &Value) -> Result<(), String> {
        let mut walls: Vec<Wall> = match mission.get("walls") {
            None | Some(Value::Null) => Vec::new(),
            Some(value) => serde_json::from_value(value.clone()).map_err(|e| format!("Invalid original walls: {e}"))?,
        };
        if walls.len() > 16384 { return Err("Too many original walls".into()); }
        let total = self.map.width * self.map.height;
        let mut ids = HashSet::new();
        for wall in &mut walls {
            if !ids.insert(wall.id) || ![wall.x, wall.y, wall.w, wall.h, wall.hp, wall.max_hp].iter().all(|n| n.is_finite())
                || wall.x < 0. || wall.y < 0. || wall.w <= 0. || wall.h <= 0.
                || wall.w > 512. || wall.h > 512. || wall.x + wall.w > self.map.width as f64 * TILE
                || wall.y + wall.h > self.map.height as f64 * TILE {
                return Err("Invalid original wall geometry".into());
            }
            if wall.hp==0. && wall.native_hp==Some(0) { wall.hp=1.; }
            if wall.max_hp <= 0. { wall.max_hp = wall.hp.max(0.); }
            if wall.hp > wall.max_hp { wall.max_hp = wall.hp; }
            if wall.cells.is_empty() {
                for y in (wall.y / TILE).floor() as usize..((wall.y + wall.h) / TILE).ceil() as usize {
                    for x in (wall.x / TILE).floor() as usize..((wall.x + wall.w) / TILE).ceil() as usize {
                        wall.cells.push(y * self.map.width + x);
                    }
                }
            }
            wall.cells.sort_unstable(); wall.cells.dedup();
            if wall.cells.is_empty() || wall.cells.iter().any(|&cell| cell >= total) {
                return Err("Invalid original wall cells".into());
            }
            let classes: Vec<_> = wall.cells.iter().map(|&j| self.map.tiles[j]).collect();
            if wall.ground_classes.len() != wall.cells.len() {
                wall.ground_classes = classes.iter().map(|&tile| if tile == 2 || tile == 6 { 0 } else { tile }).collect();
            }
            if wall.ground_codes.len() != wall.cells.len() {
                wall.ground_codes = wall.cells.iter().zip(&classes).map(|(&j, &tile)| {
                    if tile == 2 || tile == 6 { 0 } else { self.map.terrain_codes.get(j).copied().unwrap_or(0) }
                }).collect();
            }
            if wall.ground_tiles.len() != wall.cells.len() {
                wall.ground_tiles = wall.cells.iter().map(|&j| self.map.original_tiles.get(j).copied().unwrap_or(0)).collect();
            }
        }
        self.walls = walls;
        if !self.walls.is_empty() {
            if self.map.wall_ground_tiles.len()!=total {self.map.wall_ground_tiles=self.map.original_tiles.clone();}
            if self.map.wall_ground_codes.len()!=total {self.map.wall_ground_codes=self.map.terrain_codes.clone();}
            if self.map.wall_ground_classes.len()!=total {self.map.wall_ground_classes=self.map.tiles.clone();}
            for wall in &self.walls { for(n,&j)in wall.cells.iter().enumerate() {
                if let Some(tile)=self.map.wall_ground_tiles.get_mut(j){*tile=wall.ground_tiles[n];}
                if let Some(code)=self.map.wall_ground_codes.get_mut(j){*code=wall.ground_codes[n];}
                self.map.wall_ground_classes[j]=wall.ground_classes[n];
            }}
        }
        self.rebuild_walls();
        Ok(())
    }

    pub(crate) fn wall(&self, id: usize) -> Option<&Wall> { self.walls.iter().find(|w| w.id == id && w.alive()) }

    /// Restore the one shared raw ground map, then repeat the native record/bit
    /// order. A wall head can overlap another fragment's two southern fronts.
    fn rebuild_walls(&mut self) {
        if self.walls.is_empty() { return; }
        let total=self.map.tiles.len();
        self.wall_owners=vec![usize::MAX;total];
        for wall in &self.walls { for &cell in &wall.cells {
            if cell>=total {continue;}
            if let Some(&tile)=self.map.wall_ground_tiles.get(cell) {if let Some(value)=self.map.original_tiles.get_mut(cell){*value=tile;}}
            if let Some(&code)=self.map.wall_ground_codes.get(cell) {if let Some(value)=self.map.terrain_codes.get_mut(cell){*value=code;}}
            if let Some(&class)=self.map.wall_ground_classes.get(cell) {self.map.tiles[cell]=class;}
        }}
        let lookup=self.map.wall_tile_lookup.clone();
        let native=lookup["version"].as_u64()==Some(1);
        // Native rubble uses a random variant. The fixed persisted fragment ID
        // selects the same local artwork without claiming original RNG ordering.
        if native { for wall in self.walls.iter().filter(|w|!w.alive()) {
            for (dy,&cell)in wall.cells.iter().enumerate() {
                if cell>=total {continue;}
                if let Some(tile)=lookup["rubbleTiles"][(wall.id+dy)&7].as_u64() {if let Some(value)=self.map.original_tiles.get_mut(cell){*value=tile as u16;}}
                if let Some(value)=self.map.terrain_codes.get_mut(cell){*value=lookup["rubbleCode"].as_u64().unwrap_or(22)as u8;}
                self.map.tiles[cell]=0;
            }
        } }
        let live:HashSet<_>=self.walls.iter().filter(|w|w.alive()).filter_map(|w|Some((w.wall_record?,w.wall_bit?))).collect();
        let mut order:Vec<_>=(0..self.walls.len()).filter(|&i|self.walls[i].alive()).collect();
        order.sort_unstable_by_key(|&i|(self.walls[i].wall_record.unwrap_or(usize::MAX),self.walls[i].wall_bit.unwrap_or(self.walls[i].id)));
        for index in order {
            let wall=&mut self.walls[index];
            let mut mask=15u16;
            if let (Some(record),Some(bit))=(wall.wall_record,wall.wall_bit) {
                let (x,y)=(bit%32,bit/32);
                for (flag,neighbor)in [(1,(y>0).then(||bit-32)),(2,(x<31).then(||bit+1)),(4,(y<31).then(||bit+32)),(8,(x>0).then(||bit-1))] {
                    if neighbor.is_some_and(|n|live.contains(&(record,n))){mask&=!flag;}
                }
            }
            wall.neighbor_mask=mask;
            let head=if native {
                let value=if mask==0 {&lookup["interiorTiles"][usize::from(wall.variant&15)]} else {&lookup["headTiles"][usize::from(mask)]};
                value.as_u64().map(|n|n as u16).or(wall.wall_tile_index)
            }else{wall.wall_tile_index};
            wall.wall_tile_index=head;
            let head_cell=(wall.y/TILE)as usize*self.map.width+(wall.x/TILE)as usize;
            let height=if wall.height>0 {usize::from(wall.height)}else{((wall.h/TILE).ceil()as usize).saturating_sub(1)};
            for &cell in &wall.cells {if cell<total {self.map.tiles[cell]=6;self.wall_owners[cell]=wall.id;}}
            if let Some(tile)=head {
                if let Some(value)=self.map.original_tiles.get_mut(head_cell){*value=tile;}
                let remapped=lookup["frontRemap"][tile.to_string()].as_u64().map(|n|n as u16).unwrap_or(tile);
                let step=lookup["frontStep"].as_u64().unwrap_or(20)as u16;
                for dy in 1..=height {
                    let cell=head_cell+dy*self.map.width;if cell>=total {break;}
                    let front=(remapped&256)|((remapped.wrapping_add(step*dy as u16))&255);
                    if let Some(value)=self.map.original_tiles.get_mut(cell){*value=front;}
                }
            }
            if let Some(value)=self.map.terrain_codes.get_mut(head_cell){*value=wall.height.max(1);}
            for dy in 1..=height {if let Some(value)=self.map.terrain_codes.get_mut(head_cell+dy*self.map.width){*value=18;}}
        }
    }

    /// Terrain at the selected wall may terminate a shot. All intervening walls
    /// and permanent cliffs remain opaque; the ordinary LOS contract is intact.
    pub(crate) fn wall_los(&self, x: f64, y: f64, dx: f64, dy: f64, target: usize) -> bool {
        if self.wall(target).is_none() { return false; }
        if ![x, y, dx, dy].iter().all(|n| n.is_finite()) { return false; }
        let steps = ((dx - x).hypot(dy - y) / 4.).ceil().max(1.) as usize;
        for n in 1..=steps {
            let t = n as f64 / steps as f64;
            let (px, py) = (x + (dx - x) * t, y + (dy - y) * t);
            if px < 0. || py < 0. || px >= self.map.width as f64 * TILE || py >= self.map.height as f64 * TILE { return false; }
            let cell = (py / TILE) as usize * self.map.width + (px / TILE) as usize;
            if self.map.tiles[cell]==2 || self.map.tiles[cell]==6&&self.wall_owners.get(cell).copied()!=Some(target) { return false; }
        }
        true
    }

    pub(crate) fn route_to_wall_attack(&self, unit: &Unit, wall: &Wall, range: f64) -> VecDeque<(f64, f64)> {
        let control = unit.original_combat.as_ref().map(|s| s.movement_control)
            .unwrap_or_else(|| original_combat::CombatState::new(&unit.kind).movement_control);
        let (x, y) = wall.center();
        self.route_profile(unit.x, unit.y, x, y, original_terrain::profile(&unit.kind), control, Some(range.max(0.)), Some(wall.id))
    }

    pub(crate) fn damage_wall(&mut self, id: usize, amount: f64, team: i32) -> bool {
        if !amount.is_finite() || amount <= 0. { return false; }
        let Some(index) = self.walls.iter().position(|w| w.id == id && w.alive()) else { return false; };
        self.walls[index].hp = (self.walls[index].hp - amount).max(0.);
        if self.walls[index].alive() { return false; }
        let wall = self.walls[index].clone();
        self.rebuild_walls();
        self.map.revision = self.map.revision.wrapping_add(1);
        let (x, y) = wall.center();
        self.effects.push(Fx { x, y, age: 0., life: 0.8, kind: "explosion".into(), radius: (wall.w.max(wall.h) * 0.45).clamp(14., 42.), x2: 0., y2: 0. });
        self.events.push(json!({"kind":"wallDestroyed","id":id,"x":x,"y":y,"team":team}));
        true
    }

    /// An explicit wall attack retains its target over many shots and moves to
    /// an actual reachable firing position instead of trying to enter the wall.
    pub(crate) fn update_wall_attack(&mut self, index: usize, range: f64, damage: f64, reload: f64, speed: f64) -> bool {
        let unit = self.units[index].clone();
        if unit.order != "attackWall" { return false; }
        let target = unit.target.and_then(|id| self.wall(id)).cloned();
        let Some(wall) = target.filter(|_| capable(&unit) && damage > 0. && range > 0.) else {
            self.clear_wall_order(index); return false;
        };
        let point = wall.point_from(unit.x, unit.y);
        self.units[index].dest = Some(wall.center());
        if (unit.x - point.0).hypot(unit.y - point.1) <= range && self.wall_los(unit.x, unit.y, point.0, point.1, wall.id) {
            self.units[index].path.clear(); self.units[index].pursuit = None;
            self.units[index].aim = (point.1 - unit.y).atan2(point.0 - unit.x);
            if self.units[index].cooldown <= 0. {
                let grenade = !primary_capable(&unit.kind);
                let amount = if grenade { f64::from(super::original_grenades::CONTACT_DAMAGE) } else { damage };
                if grenade { self.units[index].grenades -= 1; }
                self.units[index].cooldown = reload;
                self.effects.push(Fx { x: unit.x, y: unit.y, age: 0., life: 0.18, kind: "shot".into(), radius: 2., x2: point.0, y2: point.1 });
                self.events.push(json!({"kind":"shot","type":unit.kind,"weapon":if grenade {"grenade"}else{"primary"},"x":unit.x,"y":unit.y,"team":unit.team,"wall":wall.id}));
                if self.damage_wall(wall.id, amount, unit.team) { self.clear_wall_order(index); }
            }
            return true;
        }
        if speed <= 0. { self.units[index].path.clear(); return false; }
        let previous = unit.pursuit.as_ref();
        let revised = previous.is_some_and(|p| p.revision != self.map.revision || p.target != wall.id);
        let due = previous.is_none_or(|p| self.time >= p.next_plan);
        if revised || due && unit.path.is_empty() {
            let path = self.route_to_wall_attack(&unit, &wall, range);
            let delay = if path.is_empty() { 0.75 } else { 0.25 };
            self.units[index].path = path;
            self.units[index].pursuit = Some(pursuit::State { target: wall.id, planned_goal: wall.center(), next_plan: self.time + delay, revision: self.map.revision });
        }
        false
    }

    fn clear_wall_order(&mut self, index: usize) {
        let unit = &mut self.units[index];
        unit.order = "guard".into(); unit.target = None; unit.dest = None; unit.path.clear(); unit.pursuit = None;
    }
}
