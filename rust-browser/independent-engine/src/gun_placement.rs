//! Finished artillery remains in its source building until it is placed.
//! The four queue slots, 16-native-pixel grid, source territory and six fort
//! sockets follow the original executable. Other clearance bounds are adapted
//! to the browser engine's unit/object collision representation.
use super::*;

pub(crate) const READY_GUN_CAPACITY: usize = 4;
const SIZE: f64 = 64.;
const SOCKETS: [(f64, f64); 6] = [(17., 170.), (113., 1.), (113., 170.), (17., 1.), (113., 49.), (17., 49.)];
// FUN00412640: FF entries of the terrain-placement table at00499788.
const BLOCKED_TERRAIN: &[u8] = &[1,2,3,4,5,6,7,9,11,12,13,14,15,16,17,18,19,20,21,23,24,25,26,27,28,29,32,33,34,35,36,37,38,39,40,41,42,43,44,46,47,48,49,50,51,52,56,57,58,60,61];

pub(crate) fn is_gun(kind: &str) -> bool {
    ["gatling", "cannon", "howitzer", "missile"].contains(&kind)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GunPlacementPreview {
    pub valid: bool,
    pub reason: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: u32,
    pub height: u32,
    pub fort_id: Option<usize>,
}

impl GunPlacementPreview {
    fn reject(mut self, reason: &str) -> Self {
        self.reason = reason.into();
        self
    }
}

fn overlap(ax: f64, ay: f64, aw: f64, ah: f64, bx: f64, by: f64, bw: f64, bh: f64) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn gun_sockets(fort: &Building, kind: &str) -> Vec<(f64, f64)> {
    let Some((bx, by)) = fort.raw_base else { return Vec::new(); };
    let (dx, dy) = if kind == "cannon" { (0., 0.) } else { (-2., -8.) };
    SOCKETS.iter().map(|(x, y)| ((bx + x + dx) * 2., (by + y + dy) * 2.)).collect()
}

fn selected_socket(fort: &Building, x: f64, y: f64) -> Option<usize> {
    let (bx, by) = fort.raw_base?;
    let nearest = SOCKETS.iter().enumerate().min_by(|(_, a), (_, b)|
        ((bx+a.0)*2.-x).hypot((by+a.1)*2.-y).total_cmp(&((bx+b.0)*2.-x).hypot((by+b.1)*2.-y)))?;
    let (index, socket) = nearest;
    // The imported physical building box starts below the top roof sockets.
    // Accept direct roof clicks too, and choose the same socket for every type
    // before applying the original per-type drawing offset.
    let inside = x >= fort.x && x < fort.x+fort.w && y >= fort.y && y < fort.y+fort.h;
    (inside || ((bx+socket.0)*2.-x).hypot((by+socket.1)*2.-y) <= 48.).then_some(index)
}

impl Game {
    pub(crate) fn gun_placement_preview(&self, id: usize, slot: usize, x: f64, y: f64, team: i32) -> GunPlacementPreview {
        let mut result = GunPlacementPreview {
            valid: false, reason: String::new(), kind: None,
            x: if x.is_finite() { (x / TILE).floor() * TILE } else { 0. },
            y: if y.is_finite() { (y / TILE).floor() * TILE } else { 0. },
            width: SIZE as u32, height: SIZE as u32, fort_id: None,
        };
        let Some(factory) = self.buildings.iter().find(|b| b.id == id) else { return result.reject("Fabrik nicht vorhanden"); };
        if factory.team != team || team < 0 || factory.hp <= 0. { return result.reject("Keine eigene einsatzbereite Fabrik"); }
        let Some(kind) = factory.ready_guns.get(slot).filter(|k| is_gun(k)) else { return result.reject("Kein Geschütz bereit"); };
        result.kind = Some(kind.clone());
        if self.winner.is_some() { return result.reject("Schlacht beendet"); }
        if !x.is_finite() || !y.is_finite() { return result.reject("Ungültige Position"); }
        if self.units.iter().filter(|u| u.team == team && original_combat::alive(self.map.original, u.hp)).count() >= 120 {
            return result.reject("Einheitenlimit erreicht");
        }
        if !self.sectors.iter().any(|s| s.id == factory.sector && s.owner == team) {
            return result.reject("Fabrikgebiet gehört nicht mehr dir");
        }

        // A fort click chooses a roof socket, not a ground gun beside the fort.
        if self.map.original {
            if let Some((fort, socket)) = self.buildings.iter().filter(|b| b.kind == "fort" && b.hp > 0. && b.team == team && b.sector == factory.sector)
                .find_map(|b| selected_socket(b, x, y).map(|socket| (b, socket))) {
                if let Some(&(sx, sy)) = gun_sockets(fort, kind).get(socket) {
                    result.x = sx; result.y = sy; result.fort_id = Some(fort.id);
                    if sx < 0. || sy < 0. || sx + SIZE > self.map.width as f64 * TILE || sy + SIZE > self.map.height as f64 * TILE {
                        return result.reject("Außerhalb der Karte");
                    }
                    if self.units.iter().any(|u| is_gun(&u.kind) && original_combat::alive(self.map.original, u.hp)
                        && overlap(sx, sy, SIZE, SIZE, u.x, u.y, SIZE, SIZE)) {
                        return result.reject("Turmplatz ist besetzt");
                    }
                    result.valid = true;
                    return result;
                }
            }
        }

        let (px, py) = (result.x, result.y);
        if px < 0. || py < 0. || px + SIZE > self.map.width as f64 * TILE || py + SIZE > self.map.height as f64 * TILE {
            return result.reject("Außerhalb der Karte");
        }
        let (tx, ty) = ((px / TILE) as usize, (py / TILE) as usize);
        for yy in ty..ty + 2 {
            for xx in tx..tx + 2 {
                let index = yy * self.map.width + xx;
                if self.map.territory.get(index).copied() != Some(factory.sector) {
                    return result.reject("Nur im Gebiet dieser Fabrik platzierbar");
                }
                if self.map.tiles.get(index).is_some_and(|tile|matches!(tile,2|6)) { return result.reject("Fels oder Steinwand blockiert den Platz"); }
                if let Some(code) = self.map.terrain_codes.get(index).filter(|_| self.map.original) {
                    if BLOCKED_TERRAIN.contains(&(code & 127)) { return result.reject("Gelände ist ungeeignet"); }
                } else if !self.map.tiles.get(index).is_some_and(|t| *t != 2 && *t != 3 && *t != 5) {
                    return result.reject("Gelände ist ungeeignet");
                }
            }
        }
        if self.buildings.iter().any(|b| b.hp > 0. && overlap(px, py, SIZE, SIZE, b.x, b.y, b.w, b.h)) {
            return result.reject("Gebäude blockiert den Platz");
        }
        if self.units.iter().any(|u| original_combat::alive(self.map.original, u.hp) && if is_gun(&u.kind) {
            overlap(px, py, SIZE, SIZE, u.x, u.y, SIZE, SIZE)
        } else {
            let radius = if stats(&u.kind).cls == 0 { 12. } else { 24. };
            overlap(px, py, SIZE, SIZE, u.x-radius, u.y-radius, radius*2., radius*2.)
        }) {
            return result.reject("Einheit blockiert den Platz");
        }
        if self.sectors.iter().any(|s| !s.home && s.counts_for_production && overlap(px, py, SIZE, SIZE, s.x-16., s.y-16., 32., 32.)) {
            return result.reject("Fahne blockiert den Platz");
        }
        if self.crates.iter().any(|c| !c.taken && overlap(px, py, SIZE, SIZE, c.x-12., c.y-12., 24., 24.)) {
            return result.reject("Kiste blockiert den Platz");
        }
        if self.objects.iter().any(|o| {
            // Hardware and grenade pickups are represented by units/crates and
            // must not leave stale collision obstacles after being collected.
            if matches!(o["templateFlags"].as_u64(), Some(0x31 | 0x32)) || o["type"].as_u64() == Some(10) { return false; }
            let (Some(ox), Some(oy)) = (o["x"].as_f64(), o["y"].as_f64()) else { return false; };
            overlap(px, py, SIZE, SIZE, ox-16., oy-16., 32., 32.)
        }) {
            return result.reject("Hindernis blockiert den Platz");
        }
        result.valid = true;
        result
    }

    pub(crate) fn place_gun_for_team(&mut self, id: usize, slot: usize, x: f64, y: f64, team: i32) -> GunPlacementPreview {
        let result = self.gun_placement_preview(id, slot, x, y, team);
        if !result.valid { return result; }
        let kind = result.kind.as_deref().unwrap();
        let index = self.buildings.iter().position(|b| b.id == id).unwrap();
        self.buildings[index].ready_guns.remove(slot);
        self.spawn(kind, team, result.x, result.y, self.next_id, None);
        self.units.last_mut().unwrap().fort_id = result.fort_id;
        self.events.push(json!({"kind":"placed","team":team,"type":kind,"factory":id,"x":result.x,"y":result.y}));
        result
    }

    pub(crate) fn place_ai_guns(&mut self, team: i32) {
        let factories: Vec<usize> = self.buildings.iter().filter(|b| b.team == team && b.hp > 0. && !b.ready_guns.is_empty()).map(|b| b.id).collect();
        for id in factories {
            for _ in 0..READY_GUN_CAPACITY {
                let Some(factory) = self.buildings.iter().find(|b| b.id == id) else { break; };
                let Some(kind) = factory.ready_guns.first() else { break; };
                let (sector, cx, cy) = (factory.sector, factory.x+factory.w/2., factory.y+factory.h/2.);
                let mut candidates = Vec::new();
                if self.map.original {
                    for fort in self.buildings.iter().filter(|b| b.kind == "fort" && b.team == team && b.sector == sector && b.hp > 0.) {
                        for (sx, sy) in gun_sockets(fort, kind) {
                            candidates.push((sx.clamp(fort.x+1., fort.x+fort.w-1.), sy.clamp(fort.y+1., fort.y+fort.h-1.)));
                        }
                    }
                }
                let mut ground: Vec<(f64, f64)> = self.map.territory.iter().enumerate().filter(|(_, s)| **s == sector).map(|(i, _)| ((i % self.map.width) as f64 * TILE, (i / self.map.width) as f64 * TILE)).collect();
                ground.sort_by(|a, b| (a.0-cx).hypot(a.1-cy).total_cmp(&(b.0-cx).hypot(b.1-cy)));
                candidates.extend(ground);
                let mut placed = false;
                for (x, y) in candidates.into_iter().take(4096) {
                    if self.gun_placement_preview(id, 0, x, y, team).valid {
                        self.place_gun_for_team(id, 0, x, y, team);
                        placed = true;
                        break;
                    }
                }
                if !placed { break; }
            }
        }
    }
}
