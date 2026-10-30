//! Rust/Wasm reconstruction with Ghidra-verified original data and contact rules. This is not a clean-room implementation.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
const TILE: f64 = 32.;
mod original_stats;
pub mod original_combat;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tile_size: f64,
    pub world: usize,
    pub tiles: Vec<u8>,
    pub territory: Vec<usize>,
    pub bridge_base: Vec<i32>,
    pub bridge_hp: Vec<f64>,
    pub seed: u32,
    pub revision: u32,
    #[serde(default)] pub original:bool,
    #[serde(default)] pub original_tiles:Vec<u16>,
    #[serde(default)] pub terrain_codes:Vec<u8>,
    #[serde(default)] pub water_tile:u16,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Sector {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub owner: i32,
    pub home: bool,
    #[serde(default="production_sector")] pub counts_for_production:bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub kind: String,
    pub team: i32,
    pub sector: usize,
    pub stars: usize,
    pub hp: f64,
    pub max_hp: f64,
    pub product: Option<String>,
    pub progress: f64,
    #[serde(default)] pub gate:Option<(f64,f64)>,
    #[serde(default)] pub orientation:String,
    #[serde(default)] pub remaining_work:Option<f64>,
    #[serde(default)] pub raw_base:Option<(f64,f64)>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    pub kind: String,
    pub hp: f64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: usize,
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub team: i32,
    pub hp: f64,
    pub max_hp: f64,
    pub angle: f64,
    pub aim: f64,
    pub moving: bool,
    pub squad: usize,
    pub grenades: usize,
    pub crew: Option<Crew>,
    pub cargo: Vec<Crew>,
    pub order: String,
    pub target: Option<usize>,
    pub dest: Option<(f64, f64)>,
    pub cooldown: f64,
    #[serde(default)]
    pub original_combat: Option<original_combat::CombatState>,
    #[serde(skip_deserializing)]
    pub path: VecDeque<(f64, f64)>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Fx {
    pub x: f64,
    pub y: f64,
    pub age: f64,
    pub life: f64,
    pub kind: String,
    pub radius: f64,
    pub x2: f64,
    pub y2: f64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Crate {
    pub x: f64,
    pub y: f64,
    pub taken: bool,
}
#[derive(Clone, Copy)]
pub struct Stats {
    pub hp: f64,
    pub speed: f64,
    pub range: f64,
    pub damage: f64,
    pub reload: f64,
    pub seconds: f64,
    pub members: usize,
    pub stars: usize,
    pub cls: u8,
}
pub fn stats(k: &str) -> Stats {
    if let Some(d)=original_stats::data(k){return d;}
    let (hp, speed, range, damage, reload, seconds, members, stars, cls) = match k {
        "psycho" => (42., 42., 130., 6., 0.28, 30., 3, 2, 0),
        "tough" => (64., 31., 160., 30., 1.8, 42., 2, 3, 0),
        "sniper" => (36., 40., 260., 18., 1.7, 46., 2, 3, 0),
        "pyro" => (62., 37., 75., 9., 0.3, 48., 3, 4, 0),
        "laser" => (56., 43., 210., 15., 0.65, 58., 3, 5, 0),
        "jeep" => (150., 72., 150., 9., 0.48, 35., 1, 1, 1),
        "light" => (290., 46., 190., 36., 1.9, 49., 1, 1, 1),
        "medium" => (480., 35., 215., 58., 2.4, 66., 1, 2, 1),
        "heavy" => (720., 28., 230., 92., 2.9, 85., 1, 4, 1),
        "apc" => (320., 55., 0., 0., 1., 48., 1, 2, 1),
        "crane" => (250., 36., 0., 0., 1., 54., 1, 2, 1),
        "gatling" => (280., 0., 175., 8., 0.25, 39., 1, 1, 2),
        "cannon" => (430., 0., 230., 52., 2.2, 64., 1, 2, 2),
        "missile" => (400., 0., 300., 100., 3.8, 92., 1, 4, 2),
        _ => (40., 40., 120., 8., 0.75, 24., 3, 1, 0),
    };
    Stats {
        hp,
        speed,
        range,
        damage,
        reload,
        seconds,
        members,
        stars,
        cls,
    }
}
pub const TYPES: [&str; 19] = [
    "grunt", "psycho", "tough", "sniper", "pyro", "laser", "jeep", "light", "medium", "heavy",
    "apc", "crane", "gatling", "cannon", "missile", "launcher", "howitzer", "truck", "firetruck",
];
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub map: Map,
    pub sectors: Vec<Sector>,
    pub buildings: Vec<Building>,
    pub units: Vec<Unit>,
    pub effects: Vec<Fx>,
    pub crates: Vec<Crate>,
    pub events: Vec<Value>,
    #[serde(default)] pub objects:Vec<Value>,
    pub time: f64,
    pub winner: Option<i32>,
    pub difficulty: usize,
    pub mission: usize,
    #[serde(skip)]
    next_id: usize,
    #[serde(skip)]
    ai_clock: f64,
    #[serde(skip)]
    rng: u32,
    #[serde(skip)]
    ai_teams: Vec<i32>,
    #[serde(skip)] production_clock:f64,
}
impl Game {
    fn random(&mut self) -> f64 {
        advance_random(&mut self.rng) as f64 / u32::MAX as f64
    }
    pub fn new(mission: usize, difficulty: usize, seed: u32) -> Self {
        let (w, h) = (56, 40);
        let n = w * h;
        let mut g = Self {
            map: Map {
                width: w,
                height: h,
                tile_size: TILE,
                world: (mission / 4) % 5,
                tiles: vec![0; n],
                territory: vec![0; n],
                bridge_base: vec![-1; n],
                bridge_hp: vec![],
                seed,
                revision: 0, original:false,original_tiles:vec![],terrain_codes:vec![],water_tile:0,
            },
            sectors: vec![],
            buildings: vec![],
            units: vec![],
            effects: vec![],
            crates: vec![],
            events: vec![], objects:vec![],
            time: 0.,
            winner: None,
            difficulty: difficulty.min(3),
            mission,
            next_id: 1,
            ai_clock: 3.,
            rng: seed.max(1),
            ai_teams: vec![1], production_clock:0.,
        };
        // Own nine-territory layout. Voronoi edges and a meandering river avoid a square sector grid.
        let sites = [
            (7, 7),
            (25, 6),
            (48, 7),
            (10, 20),
            (28, 20),
            (45, 19),
            (7, 32),
            (30, 33),
            (48, 32),
        ];
        for (id, (x, y)) in sites.iter().enumerate() {
            g.sectors.push(Sector {
                id,
                x: (*x as f64 + 0.5) * TILE,
                y: (*y as f64 + 0.5) * TILE,
                owner: if id == 6 {
                    0
                } else if id == 2 {
                    1
                } else {
                    -1
                },
                home: id == 6 || id == 2, counts_for_production:true,
            });
        }
        for y in 0..h {
            for x in 0..w {
                let j = y * w + x;
                let river = 28. + (y as f64 / 5.).sin() * 2.;
                let mut t = if (x as f64 - river).abs() < 1.4 { 3 } else { 0 };
                if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                    t = 2;
                }
                if t == 0 && g.random() < 0.07 {
                    t = 4;
                }
                if t == 0
                    && x > 3
                    && x < w - 3
                    && y > 3
                    && y < h - 3
                    && ((x as f64 / 3.).sin() + (y as f64 / 2.8).cos() > 1.62)
                {
                    t = 2;
                }
                if (y.abs_diff(13) < 2 || y.abs_diff(28) < 2) && x > 5 && x < 51 {
                    t = if t == 3 { 5 } else { 1 };
                }
                g.map.tiles[j] = t;
                g.map.territory[j] = g
                    .sectors
                    .iter()
                    .min_by_key(|s| {
                        let dx = x as i32 - (s.x / TILE) as i32;
                        let dy = y as i32 - (s.y / TILE) as i32;
                        dx * dx + dy * dy
                    })
                    .unwrap()
                    .id;
            }
        }
        for j in 0..n / 2 {
            g.map.tiles[n - 1 - j] = g.map.tiles[j];
        }
        for s in g.sectors.clone() {
            let kind = if s.home {
                "fort"
            } else {
                match s.id {
                    0 | 4 | 8 => "robot",
                    1 | 7 => "vehicle",
                    3 | 5 => {
                        if mission % 2 == 0 {
                            "repair"
                        } else {
                            "radar"
                        }
                    }
                    _ => "radar",
                }
            };
            let (bw, bh) = if s.home { (144., 128.) } else { (96., 80.) };
            let x = s.x - bw / 2.;
            let y = s.y - bh - 18.;
            for ty in ((y / TILE) as i32 - 1)..=((s.y / TILE) as i32 + 2) {
                for tx in ((x / TILE) as i32 - 2)..=(((x + bw) / TILE) as i32 + 1) {
                    if tx > 0 && ty > 0 && tx < w as i32 - 1 && ty < h as i32 - 1 {
                        g.map.tiles[ty as usize * w + tx as usize] = 0;
                    }
                }
            }
            let stars = if s.home {
                1 + (mission / 5).min(4)
            } else {
                1 + (s.id.min(8 - s.id) + mission) % 5
            };
            let hp = if s.home { 2400. } else { 1200. };
            g.buildings.push(Building {
                id: g.buildings.len(),
                x,
                y,
                w: bw,
                h: bh,
                kind: kind.into(),
                team: s.owner,
                sector: s.id,
                stars,
                hp,
                max_hp: hp,
                product: if s.home { Some("grunt".into()) } else { None },
                progress: 0., gate:None,orientation:String::new(),remaining_work:None,raw_base:None,
            });
        }
        for j in 0..n {
            if g.map.tiles[j] == 5 && g.map.bridge_base[j] < 0 {
                let id = g.map.bridge_hp.len() as i32;
                g.map.bridge_hp.push(400.);
                let mut q = VecDeque::from([j]);
                g.map.bridge_base[j] = id;
                while let Some(k) = q.pop_front() {
                    for v in [k.wrapping_sub(1), k + 1, k.wrapping_sub(w), k + w] {
                        if v < n && g.map.tiles[v] == 5 && g.map.bridge_base[v] < 0 {
                            g.map.bridge_base[v] = id;
                            q.push_back(v);
                        }
                    }
                }
            }
        }
        for team in [0, 1] {
            let s = g.sectors[if team == 0 { 6 } else { 2 }].clone();
            for squad in 0..2 {
                g.spawn_group("grunt", team, s.x - 38. + squad as f64 * 56., s.y + 22.);
            }
            if mission > 0 {
                g.spawn_group("light", team, s.x, s.y + 68.);
            }
            g.spawn_group("gatling", team, s.x - 58., s.y - 114.);
            g.spawn_group("gatling", team, s.x + 58., s.y - 114.);
            if difficulty == 0 && team == 0 {
                g.spawn_group("grunt", 0, s.x + 8., s.y + 56.);
            }
        }
        for id in [0, 4, 8] {
            let s = g.sectors[id].clone();
            g.spawn_group(
                if id == 4 { "light" } else { "jeep" },
                -1,
                s.x + 72.,
                s.y + 32.,
            );
            g.crates.push(Crate {
                x: s.x - 64.,
                y: s.y + 24.,
                taken: false,
            });
        }
        g
    }
    fn spawn_group(&mut self, k: &str, team: i32, x: f64, y: f64) {
        let d = stats(k);
        let squad = self.next_id;
        for i in 0..d.members {
            self.spawn(
                k,
                team,
                x + (i % 3) as f64 * 17.,
                y + (i / 3) as f64 * 18.,
                squad,
                None,
            );
        }
    }
    fn spawn(&mut self, k: &str, team: i32, x: f64, y: f64, squad: usize, crew: Option<Crew>) {
        let d = stats(k);
        let crew = if d.cls > 0 && team >= 0 {
            crew.or(Some(Crew {
                kind: "grunt".into(),
                hp: if self.map.original {original_combat::initial_crew_hp(k)} else {40.},
            }))
        } else {
            crew
        };
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            kind: k.into(),
            x,
            y,
            team,
            hp: d.hp,
            max_hp: d.hp,
            angle: 0.,
            aim: 0.,
            moving: false,
            squad,
            grenades: 0,
            crew,
            cargo: vec![],
            order: "guard".into(),
            target: None,
            dest: None,
            cooldown: 0.,
            original_combat: self.map.original.then(||original_combat::CombatState::new(k)),
            path: VecDeque::new(),
        });
    }
    pub fn products(&self,b:&Building)->Vec<&'static str>{
      if b.hp<=0.{return vec![];}
      if self.map.original{return original_stats::MENU_ORDER.iter().filter_map(|id|original_stats::PRODUCTS.iter().find(|r|r.1==*id)).filter(|r|r.6&&original_stats::required(r.0,&b.kind)>0&&original_stats::required(r.0,&b.kind)<=b.stars).map(|r|r.0).collect();}
      TYPES.iter().copied().filter(|k|{let d=stats(k);d.stars<=b.stars&&(b.kind=="fort"||b.kind=="robot"&&d.cls==0||b.kind=="vehicle"&&d.cls!=0)}).collect()
    }
    pub fn apply_original(&mut self,v:&Value)->Result<(),String>{
      let map:Map=serde_json::from_value(v["map"].clone()).map_err(|e|e.to_string())?;
      if map.width<1||map.height<1||map.width>128||map.height>128||map.tiles.len()!=map.width*map.height{return Err("Invalid original campaign map".into());}
      let sectors:Vec<Sector>=serde_json::from_value(v["sectors"].clone()).map_err(|e|e.to_string())?;
      let buildings:Vec<Building>=serde_json::from_value(v["buildings"].clone()).map_err(|e|e.to_string())?;
      let crates:Vec<Crate>=serde_json::from_value(v["crates"].clone()).map_err(|e|format!("Invalid original crates: {e}"))?;
      self.map=map;self.objects=v["originalObjects"].as_array().cloned().unwrap_or_default();self.sectors=sectors;self.buildings=buildings;self.units.clear();self.crates=crates;self.effects.clear();self.next_id=1;
      self.map.water_tile=self.map.terrain_codes.iter().position(|c|[9,39,40].contains(c)).and_then(|i|self.map.original_tiles.get(i).copied()).unwrap_or(0);
      for spawn in v["spawns"].as_array().ok_or("Missing spawns")?{let kind=spawn["type"].as_str().ok_or("Missing unit type")?;let kind=if kind=="mobile_missile"{"launcher"}else{kind};if !TYPES.contains(&kind){return Err(format!("Unsupported original unit {kind}"));}let x=spawn["x"].as_f64().ok_or("Missing x")?;let y=spawn["y"].as_f64().ok_or("Missing y")?;let team=spawn["team"].as_i64().unwrap_or(-1)as i32;let squad=spawn["squad"].as_u64().unwrap_or(self.next_id as u64)as usize;self.spawn(kind,team,x,y,squad,None);}
      for i in 0..self.buildings.len(){if self.buildings[i].team>=0{let choices=self.products(&self.buildings[i]);if self.buildings[i].product.as_deref().map(|p|!choices.contains(&p)).unwrap_or(true){self.buildings[i].product=if choices.contains(&"grunt"){Some("grunt".into())}else{choices.first().map(|k|k.to_string())};}}}
      Ok(())
    }
    pub fn remaining_seconds(&self,b:&Building)->Option<u32>{let k=b.product.as_deref()?;if !self.map.original{return Some(((1.-b.progress)*stats(k).seconds)as u32);}let owned=self.sectors.iter().filter(|s|s.owner==b.team&&s.counts_for_production).count();let total=self.sectors.iter().filter(|s|s.counts_for_production).count();let rate=original_stats::rate(b.hp,b.max_hp,owned,total);if rate==0{return None;}Some((b.remaining_work.unwrap_or(original_stats::workload(k)as f64).max(0.)/(rate*17)as f64)as u32)}

    fn passable(&self, x: i32, y: i32, swim: bool) -> bool {
        if x < 1 || y < 1 || x >= self.map.width as i32 - 1 || y >= self.map.height as i32 - 1 {
            return false;
        }
        let t = self.map.tiles[y as usize * self.map.width + x as usize];
        if t == 2 || (t == 3 && !swim) {
            return false;
        }
        let (px, py) = ((x as f64 + 0.5) * TILE, (y as f64 + 0.5) * TILE);
        !self
            .buildings
            .iter()
            .any(|b| b.hp > 0. && px > b.x && px < b.x + b.w && py > b.y && py < b.y + b.h
                && !b.gate.map(|(gx,gy)|(px-gx).abs()<24.&&(if b.orientation=="north"{py<=gy+24.}else{py>=gy-24.})).unwrap_or(false))
    }
    fn route(&self, x: f64, y: f64, dx: f64, dy: f64, swim: bool) -> VecDeque<(f64, f64)> {
        let w = self.map.width;
        let (sx, sy) = ((x / TILE) as usize, (y / TILE) as usize);
        if sx >= w || sy >= self.map.height {
            return VecDeque::new();
        }
        let mut goal = ((dx / TILE) as i32, (dy / TILE) as i32);
        if !self.passable(goal.0, goal.1, swim) {
            let mut found = false;
            for r in 1i32..7 {
                for yy in -r..=r {
                    for xx in -r..=r {
                        if !found && self.passable(goal.0 + xx, goal.1 + yy, swim) {
                            goal = (goal.0 + xx, goal.1 + yy);
                            found = true;
                        }
                    }
                }
                if found {
                    break;
                }
            }
            if !found {
                return VecDeque::new();
            }
        }
        let start = sy * w + sx;
        let end = goal.1 as usize * w + goal.0 as usize;
        let mut prev = vec![usize::MAX; w * self.map.height];
        prev[start] = start;
        let mut q = VecDeque::from([start]);
        while let Some(j) = q.pop_front() {
            if j == end {
                break;
            }
            let (cx, cy) = ((j % w) as i32, (j / w) as i32);
            for (xx, yy) in [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
            ] {
                let (nx, ny) = (cx + xx, cy + yy);
                if !self.passable(nx, ny, swim)
                    || (xx != 0
                        && yy != 0
                        && (!self.passable(cx + xx, cy, swim) || !self.passable(cx, cy + yy, swim)))
                {
                    continue;
                }
                let k = ny as usize * w + nx as usize;
                if prev[k] == usize::MAX {
                    prev[k] = j;
                    q.push_back(k);
                }
            }
        }
        if prev[end] == usize::MAX {
            return VecDeque::new();
        }
        let mut p = vec![];
        let mut j = end;
        while j != start {
            p.push(((j % w) as f64 * TILE + 16., (j / w) as f64 * TILE + 16.));
            j = prev[j];
        }
        p.reverse();
        p.into()
    }
    fn los(&self, x: f64, y: f64, dx: f64, dy: f64) -> bool {
        let steps = ((dx - x).hypot(dy - y) / 12.) as usize;
        for i in 1..steps {
            let a = i as f64 / steps as f64;
            let (tx, ty) = (
                ((x + (dx - x) * a) / TILE) as usize,
                ((y + (dy - y) * a) / TILE) as usize,
            );
            if self.map.tiles.get(ty * self.map.width + tx) == Some(&2) {
                return false;
            }
        }
        true
    }
    pub fn command(&mut self, v: &Value) {
        let ids: Vec<usize> = v["ids"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_u64().map(|x| x as usize))
                    .collect()
            })
            .unwrap_or_default();
        let kind = v["kind"].as_str().unwrap_or("move");
        let team = v["team"].as_i64().unwrap_or(0) as i32;
        for (n, id) in ids.iter().enumerate() {
            let Some(i) = self
                .units
                .iter()
                .position(|u| u.id == *id && u.team == team && original_combat::alive(self.map.original,u.hp))
            else {
                continue;
            };
            if kind == "unload" {
                let u = self.units[i].clone();
                if u.kind == "apc" {
                    let cargo = std::mem::take(&mut self.units[i].cargo);
                    for (c, k) in cargo.into_iter().enumerate() {
                        self.spawn(
                            &k.kind,
                            team,
                            u.x + 24. + c as f64 * 9.,
                            u.y + 28.,
                            self.next_id,
                            None,
                        );
                        let last = self.units.len() - 1;
                        self.units[last].hp = k.hp;
                    }
                } else if stats(&u.kind).cls > 0 {
                    if let Some(c) = self.units[i].crew.take() {
                        self.units[i].team = -1;
                        self.units[i].order = "guard".into();
                        self.spawn(&c.kind, team, u.x + 26., u.y + 24., self.next_id, None);
                        let last = self.units.len() - 1;
                        self.units[last].hp = c.hp;
                    }
                }
                continue;
            }
            self.units[i].order = kind.into();
            self.units[i].target = v["target"].as_u64().map(|n| n as usize);
            self.units[i].path.clear();
            self.units[i].dest = None;
            if kind == "stop" {
                self.units[i].order = "guard".into();
                continue;
            }
            if let (Some(x), Some(y)) = (v["x"].as_f64(), v["y"].as_f64()) {
                let d = (x + (n % 4) as f64 * 12., y + (n / 4) as f64 * 14.);
                self.units[i].dest = Some(d);
                self.units[i].path = self.route(
                    self.units[i].x,
                    self.units[i].y,
                    d.0,
                    d.1,
                    stats(&self.units[i].kind).cls == 0 && self.map.world != 1,
                );
            }
        }
    }
    pub fn produce(&mut self, id: usize, k: Option<&str>) {
        if id >= self.buildings.len() || self.buildings[id].team != 0 {
            return;
        }
        if k.is_none() || self.products(&self.buildings[id]).contains(&k.unwrap()) {
            if self.buildings[id].product.as_deref() != k {
                self.buildings[id].progress = 0.;self.buildings[id].remaining_work=None;
            }
            self.buildings[id].product = k.map(str::to_string);
        }
    }
    fn ai(&mut self) {
        let teams = self.ai_teams.clone();
        for team in teams {
            for bi in 0..self.buildings.len() {
                let b = self.buildings[bi].clone();
                if b.team != team || b.hp <= 0. {
                    continue;
                }
                if b.product.is_none() {
                    let options = self.products(&b);
                    if !options.is_empty() {
                        let pick = (self.random() * options.len() as f64) as usize;
                        self.buildings[bi].product =
                            Some(options[pick.min(options.len() - 1)].into());
                    }
                }
            }
            let available: Vec<usize> = self
                .units
                .iter()
                .filter(|u| {
                    u.team == team && original_combat::alive(self.map.original,u.hp) && stats(&u.kind).speed > 0. && u.path.is_empty()
                })
                .map(|u| u.id)
                .collect();
            for id in available {
                let u = self.units.iter().find(|u| u.id == id).unwrap().clone();
                let goal = self
                    .sectors
                    .iter()
                    .filter(|s| s.owner != team && s.counts_for_production)
                    .min_by(|a, b| {
                        ((a.x - u.x).hypot(a.y - u.y)).total_cmp(&(b.x - u.x).hypot(b.y - u.y))
                    });
                if let Some(s) = goal {
                    let(x,y)=if s.home{self.buildings.iter().find(|b|b.sector==s.id&&b.kind=="fort").and_then(|b|b.gate).unwrap_or((s.x,s.y))}else{(s.x,s.y)};
                    self.command(&json!({"team":team,"ids":[id],"kind":"move","x":x,"y":y}));
                }
            }
        }
    }
    pub fn step(&mut self, dt: f64) {
        if self.winner.is_some() {
            return;
        }
        let dt = dt.clamp(0., 0.1);
        self.time += dt;
        self.effects.iter_mut().for_each(|e| e.age += dt);
        self.effects.retain(|e| e.age < e.life);
        self.ai_clock -= dt;
        if self.ai_clock <= 0. {
            self.ai();
            self.ai_clock = [8., 4.5, 2.5, 1.3][self.difficulty];
        }
        let snapshot = self.units.clone();
        let mut damage: Vec<(usize, f64, bool)> = vec![];
        let mut original_damage:Vec<original_combat::Attack>=vec![];
        let mut bdamage: Vec<(usize, f64)> = vec![];
        for i in 0..self.units.len() {
            let u = self.units[i].clone();
            if !original_combat::alive(self.map.original,u.hp) || u.team < 0 {
                continue;
            }
            let mut d = stats(&u.kind);
            if self.map.original {
                if let Some(state)=&u.original_combat {
                    d.speed*=state.speed_scale(&u.kind);
                    d.reload=state.reload_seconds();
                }
            }
            self.units[i].moving = false;
            self.units[i].cooldown -= dt;
            if u.order == "attackBridge" || u.order == "repairBridge" {
                if let Some((x, y)) = u.dest {
                    let j = (y / TILE) as usize * self.map.width + (x / TILE) as usize;
                    if let Some(id) = self.map.bridge_base.get(j).copied().filter(|id| *id >= 0) {
                        let id = id as usize;
                        let dist = (u.x - x).hypot(u.y - y);
                        if u.kind == "crane" && u.order == "repairBridge" && dist < 110. {
                            self.map.bridge_hp[id] = (self.map.bridge_hp[id] + dt * 65.).min(400.);
                            if self.map.bridge_hp[id] >= 400. {
                                for k in 0..self.map.tiles.len() {
                                    if self.map.bridge_base[k] == id as i32 {
                                        self.map.tiles[k] = 5;
                                    }
                                }
                                self.map.revision += 1;
                                self.events
                                    .push(json!({"kind":"bridgeRepaired","team":u.team}));
                                self.units[i].order = "guard".into();
                            }
                        }
                        if u.order == "attackBridge"
                            && dist < d.range
                            && self.units[i].cooldown <= 0.
                            && d.damage > 0.
                        {
                            self.units[i].cooldown = d.reload;
                            self.map.bridge_hp[id] -= d.damage;
                            if self.map.bridge_hp[id] <= 0. {
                                for k in 0..self.map.tiles.len() {
                                    if self.map.bridge_base[k] == id as i32 {
                                        self.map.tiles[k] = 3;
                                    }
                                }
                                for v in &mut self.units {
                                    let tile = (v.y / TILE) as usize * self.map.width
                                        + (v.x / TILE) as usize;
                                    if self.map.bridge_base.get(tile) == Some(&(id as i32)) {
                                        v.hp = if self.map.original {-1.} else {0.};
                                    }
                                }
                                self.map.revision += 1;
                                self.events
                                    .push(json!({"kind":"bridgeDestroyed","team":u.team}));
                                self.units[i].order = "guard".into();
                            }
                        }
                    }
                }
            }
            // Explicit hardware boarding and repair pursue moving targets without storing JS references.
            if ["enter", "board", "repair", "attack"].contains(&u.order.as_str()) {
                let mut dest = None;
                if let Some(target) = u.target {
                    if u.order == "repair" {
                        if let Some(b) = self.buildings.get(target) {
                            dest = Some((b.x + b.w / 2., b.y + b.h + 22.));
                            if u.kind == "crane"
                                && u.x.hypot(u.y).is_finite()
                                && (u.x - (b.x + b.w / 2.)).hypot(u.y - (b.y + b.h)) < 70.
                                && b.team == u.team
                            {
                                self.buildings[target].hp = (b.hp + dt * 48.).min(b.max_hp);
                            }
                        }
                    } else if let Some(v) = snapshot.iter().find(|v| v.id == target && original_combat::alive(self.map.original,v.hp)) {
                        dest = Some((v.x, v.y));
                        let dist = (u.x - v.x).hypot(u.y - v.y);
                        if dist < 28.
                            && d.cls == 0
                            && (u.order == "enter" && v.team < 0
                                || u.order == "board" && v.team == u.team && v.kind == "apc")
                        {
                            if let Some(vi) = self.units.iter().position(|v| v.id == target) {
                                if u.order == "enter" {
                                    self.units[vi].team = u.team;
                                    self.units[vi].crew = Some(Crew {
                                        kind: u.kind.clone(),
                                        hp: u.hp,
                                    });
                                    if let Some(state)=&mut self.units[vi].original_combat {
                                        state.crew_boarded(&u.kind);
                                    }
                                } else if self.units[vi].cargo.len() < 6 {
                                    self.units[vi].cargo.push(Crew {
                                        kind: u.kind.clone(),
                                        hp: u.hp,
                                    });
                                } else {
                                    continue;
                                }
                                self.units[i].hp = if self.map.original {-1.} else {0.};
                                self.events.push(json!({"kind":"crewed","team":u.team}));
                                continue;
                            }
                        }
                    }
                }
                if let Some(dest) = dest {
                    let need = u.path.is_empty()
                        && ((u.order != "attack")
                            || (u.x - dest.0).hypot(u.y - dest.1) > d.range * 0.8);
                    if need && d.speed > 0. {
                        self.units[i].path =
                            self.route(u.x, u.y, dest.0, dest.1, d.cls == 0 && self.map.world != 1);
                    }
                }
            }
            let enemy = snapshot
                .iter()
                .filter(|v| {
                    original_combat::alive(self.map.original,v.hp)
                        && v.team >= 0
                        && v.team != u.team
                        && (v.x - u.x).hypot(v.y - u.y) <= d.range
                        && self.los(u.x, u.y, v.x, v.y)
                })
                .min_by(|a, b| {
                    ((a.x - u.x).hypot(a.y - u.y)).total_cmp(&(b.x - u.x).hypot(b.y - u.y))
                });
            if let Some(v) =
                enemy.filter(|_| !(d.cls == 0 && u.order == "move" && !u.path.is_empty()))
            {
                self.units[i].aim = (v.y - u.y).atan2(v.x - u.x);
                if d.damage > 0. && self.units[i].cooldown <= 0. {
                    self.units[i].cooldown = d.reload;
                    let sniper = u.kind == "sniper" && stats(&v.kind).cls > 0 && v.crew.is_some();
                    let amt = if !self.map.original && u.grenades > 0 && stats(&v.kind).cls > 0 {
                        self.units[i].grenades -= 1;
                        d.damage + 38.
                    } else {
                        d.damage
                    };
                    if self.map.original {
                        original_damage.push(original_combat::Attack::from_unit(&u,v.id,amt,self.units[i].aim));
                    } else {damage.push((v.id, amt, sniper));}
                    self.effects.push(Fx {
                        x: u.x,
                        y: u.y,
                        age: 0.,
                        life: 0.14,
                        kind: "shot".into(),
                        radius: 2.,
                        x2: v.x,
                        y2: v.y,
                    });
                }
            } else if u.order == "attackBuilding" {
                if let Some(b) = u.target.and_then(|id| self.buildings.get(id)) {
                    if b.team != u.team && b.hp > 0. {
                        let (x, y) = (b.x + b.w / 2., b.y + b.h / 2.);
                        if (u.x - x).hypot(u.y - y) <= d.range && self.units[i].cooldown <= 0. {
                            self.units[i].cooldown = d.reload;
                            bdamage.push((b.id, d.damage));
                        } else if u.path.is_empty() {
                            self.units[i].path = self.route(
                                u.x,
                                u.y,
                                x,
                                b.y + b.h + 24.,
                                d.cls == 0 && self.map.world != 1,
                            );
                        }
                    }
                }
            }
            let road = self
                .map
                .tiles
                .get((u.y / TILE) as usize * self.map.width + (u.x / TILE) as usize)
                == Some(&1);
            let speed = d.speed
                * (if road && d.cls == 1 {
                    1.3
                } else if d.cls == 0 && u.order == "move" {
                    1.2
                } else {
                    1.
                });
            let hold = enemy.is_some() && u.order == "guard";
            if !hold && d.speed > 0. {
                if let Some(&(x, y)) = self.units[i].path.front() {
                    let dist = (x - u.x).hypot(y - u.y);
                    if dist < speed * dt + 1. {
                        self.units[i].x = x;
                        self.units[i].y = y;
                        self.units[i].path.pop_front();
                    } else {
                        self.units[i].angle = (y - u.y).atan2(x - u.x);
                        self.units[i].x += (x - u.x) / dist * speed * dt;
                        self.units[i].y += (y - u.y) / dist * speed * dt;
                        self.units[i].moving = true;
                        if enemy.is_none() {
                            self.units[i].aim = self.units[i].angle;
                        }
                    }
                }
            }
            if d.cls == 0 {
                for c in &mut self.crates {
                    if !c.taken && (u.x - c.x).hypot(u.y - c.y) < 22. {
                        c.taken = true;
                        self.units[i].grenades += 8;
                        self.events.push(json!({"kind":"grenades","team":u.team}));
                    }
                }
            }
            for b in &mut self.buildings {
                if b.kind == "repair"
                    && b.team == u.team
                    && d.cls == 1
                    && (u.x - (b.x + b.w / 2.)).hypot(u.y - (b.y + b.h + 20.)) < 65.
                {
                    let owned = self.sectors.iter().filter(|s| s.owner == u.team).count();
                    self.units[i].hp =
                        (self.units[i].hp + dt * (8. + owned as f64 * 4.)).min(u.max_hp);
                }
            }
        }
        for b in &self.buildings {
            if b.kind == "fort" && b.hp > 0. {
                if let Some(u) = self.units.iter().find(|u| {
                    original_combat::alive(self.map.original,u.hp)
                        && u.team >= 0
                        && u.team != b.team
                        && stats(&u.kind).cls == 0
                        && (u.x - b.gate.unwrap_or((b.x+b.w/2.,b.y+b.h+5.)).0).abs() < 24.
                        && (u.y - b.gate.unwrap_or((b.x+b.w/2.,b.y+b.h+5.)).1).abs() < 24.
                }) {
                    self.winner = Some(u.team);
                }
            }
        }
        if let Some(w) = self.winner {
            self.events.push(json!({"kind":"victory","team":w}));
            return;
        }
        for attack in original_damage {
            if let Some(i)=self.units.iter().position(|u|u.id==attack.target && original_combat::alive(true,u.hp)) {
                let mut rng=self.rng;
                original_combat::apply_attack(&mut self.units[i],attack,self.difficulty,||advance_random(&mut rng));
                self.rng=rng;
                if self.units[i].hp<0. {
                    let u=&self.units[i];
                    self.effects.push(Fx{x:u.x,y:u.y,age:0.,life:0.8,kind:"explosion".into(),
                        radius:if stats(&u.kind).cls==0 {12.} else {30.},x2:0.,y2:0.});
                }
            }
        }
        for (id, amount, sniper) in damage {
            if let Some(i) = self.units.iter().position(|u| u.id == id && u.hp > 0.) {
                if sniper {
                    if let Some(c) = &mut self.units[i].crew {
                        c.hp -= amount;
                        if c.hp <= 0. {
                            self.units[i].team = -1;
                            self.units[i].crew = None;
                            self.units[i].path.clear();
                        }
                    }
                } else {
                    self.units[i].hp -= amount;
                }
                if self.units[i].hp <= 0. {
                    let u = &self.units[i];
                    self.effects.push(Fx {
                        x: u.x,
                        y: u.y,
                        age: 0.,
                        life: 0.8,
                        kind: "explosion".into(),
                        radius: if stats(&u.kind).cls == 0 { 12. } else { 30. },
                        x2: 0.,
                        y2: 0.,
                    });
                }
            }
        }
        for (id, amount) in bdamage {
            let b = &mut self.buildings[id];
            b.hp -= amount;
            if b.hp <= 0. {
                self.events
                    .push(json!({"kind":"buildingLost","team":b.team}));
                if b.kind == "fort" {
                    self.winner = Some(1 - b.team);
                }
            }
        }
        // The public play guide describes immediate flag transfer, including an almost finished product.
        for s in &mut self.sectors {
            if s.home {
                continue;
            }
            if let Some(u) = self
                .units
                .iter()
                .find(|u| original_combat::alive(self.map.original,u.hp) && u.team >= 0 && (u.x - s.x).hypot(u.y - s.y) < 22.)
            {
                if s.owner != u.team {
                    let prev = s.owner;
                    s.owner = u.team;
                    for b in self.buildings.iter_mut().filter(|b| b.sector == s.id) {
                        b.team = u.team;
                    }
                    self.events.push(
                        json!({"kind":"capture","team":u.team,"previous":prev,"sector":s.id}),
                    );
                }
            }
        }

        self.production_clock+=dt*17.;let production_ticks=self.production_clock.floor()as i32;self.production_clock-=production_ticks as f64;
        let total=self.sectors.iter().filter(|s|s.counts_for_production).count();
        let mut completed = vec![];
        for b in &mut self.buildings {
            if b.team < 0 || b.hp <= 0. {
                continue;
            }
            if let Some(k) = b.product.clone() {
                let owned = self.sectors.iter().filter(|s| s.owner == b.team && (!self.map.original || s.counts_for_production)).count();
                let mult = if b.team == 1 {
                    [0.55, 1., 1.45, 1.85][self.difficulty]
                } else {
                    1.
                };
                if self.map.original{
                  let rate=original_stats::rate(b.hp,b.max_hp,owned,total);let workload=original_stats::workload(&k)as f64;
                  let work=b.remaining_work.get_or_insert(workload);*work-=(rate*production_ticks)as f64;
                  b.progress=(1.-*work/workload).clamp(0.,1.);
                  if *work<0.{b.remaining_work=Some(workload);b.progress=0.;let(gx,gy)=b.gate.unwrap_or((b.x+b.w/2.,b.y+b.h));completed.push((k,b.team,gx,gy+if b.orientation=="north"{-30.}else{30.}));}
                }else{
                  b.progress+=dt*(0.65+owned as f64*0.32)*(b.hp/b.max_hp).max(0.15)*mult/stats(&k).seconds;
                  if b.progress>=1.{b.progress-=1.;completed.push((k,b.team,b.x+b.w/2.,b.y+b.h+30.));}
                }
            }
        }
        for (k, team, x, y) in completed {
            if self
                .units
                .iter()
                .filter(|u| original_combat::alive(self.map.original,u.hp) && u.team == team)
                .count()
                < 120
            {
                self.spawn_group(&k, team, x, y);
                self.events
                    .push(json!({"kind":"ready","team":team,"type":k}));
            }
        }
        for team in [0, 1] {
            if !self.units.iter().any(|u| original_combat::alive(self.map.original,u.hp) && u.team == team) {
                self.winner = Some(1 - team);
            }
        }
        let original=self.map.original;
        self.units.retain(|u| original_combat::alive(original,u.hp));
        if let Some(w) = self.winner {
            self.events.push(json!({"kind":"victory","team":w}));
        }
    }
}
thread_local! {static GAME:RefCell<Option<Game>>=const{RefCell::new(None)};static OUT:RefCell<Vec<u8>>=const{RefCell::new(Vec::new())};}
pub fn request(v: Value) -> Value {
    GAME.with(|cell|{let mut state=cell.borrow_mut();if v["op"]=="init"{let mut g=Game::new(v["mission"].as_u64().unwrap_or(0)as usize,v["difficulty"].as_u64().unwrap_or(1)as usize,v["seed"].as_u64().unwrap_or(7163)as u32);if !v["originalMission"].is_null(){if let Err(e)=g.apply_original(&v["originalMission"]){return json!({"error":e});}}if let Some(a)=v["aiTeams"].as_array(){g.ai_teams=a.iter().filter_map(|x|x.as_i64().map(|x|x as i32)).collect();}*state=Some(g);}let Some(g)=state.as_mut()else{return json!({"error":"Initialize first"})};g.events.clear();match v["op"].as_str().unwrap_or(""){"tick"=>g.step(v["dt"].as_f64().unwrap_or(0.04)),"advance"=>for _ in 0..v["steps"].as_u64().unwrap_or(1).min(30000){g.step(0.1);},"command"=>g.command(&v),"produce"=>g.produce(v["id"].as_u64().unwrap_or(usize::MAX as u64)as usize,v["product"].as_str()),_=>{}}let mut value=serde_json::to_value(&*g).unwrap();value["catalog"]=json!(TYPES.iter().map(|k|{let d=stats(k);json!({"type":k,"hp":d.hp,"range":d.range,"speed":d.speed,"seconds":d.seconds,"members":d.members,"stars":d.stars,"workload":original_stats::workload(k),"class":d.cls,"requirements":{"vehicle":original_stats::required(k,"vehicle"),"robot":original_stats::required(k,"robot"),"fort":original_stats::required(k,"fort")}})}).collect::<Vec<_>>());value["productionSeconds"]=json!(g.buildings.iter().map(|b|g.remaining_seconds(b)).collect::<Vec<_>>());value["buildMenus"]=json!(g.buildings.iter().map(|b|g.products(b)).collect::<Vec<_>>());value})
}
#[no_mangle]
pub extern "C" fn alloc(n: usize) -> *mut u8 {
    let b = vec![0u8; n].into_boxed_slice();
    Box::into_raw(b) as *mut u8
}
#[no_mangle]
pub unsafe extern "C" fn free(p: *mut u8, n: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(p, n)));
}
#[no_mangle]
pub unsafe extern "C" fn run(p: *const u8, n: usize) -> usize {
    let v = serde_json::from_slice(std::slice::from_raw_parts(p, n));
    let value = match v {
        Ok(v) => request(v),
        Err(e) => json!({"error":e.to_string()}),
    };
    OUT.with(|o| {
        *o.borrow_mut() = serde_json::to_vec(&value).unwrap();
        o.borrow().len()
    })
}
#[no_mangle]
pub extern "C" fn output() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

#[cfg(test)]
mod tests;

fn production_sector()->bool{true}

fn advance_random(rng:&mut u32)->u32 {
    *rng^=*rng<<13;*rng^=*rng>>17;*rng^=*rng<<5;*rng
}
