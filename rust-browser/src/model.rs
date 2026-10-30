use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn distance(&self, p: &Point) -> f64 {
        (self.x - p.x).hypot(self.y - p.y)
    }
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Door {
    pub tx: i32,
    pub ty: i32,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Def {
    #[serde(rename = "buildLevels")]
    pub build_levels: BTreeMap<String, i32>,
    pub cls: String,
    pub tech: i32,
    pub time: f64,
    pub n: usize,
    pub hp: f64,
    pub sp: f64,
    pub range: f64,
    pub dmg: f64,
    pub rl: f64,
    pub w: Option<String>,
    pub r: f64,
    pub armor: f64,
    pub splash: f64,
    pub min_range: f64,
    pub cargo: usize,
    pub crane: bool,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Target {
    pub kind: String,
    pub id: usize,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Order {
    pub k: String,
    pub x: f64,
    pub y: f64,
    pub target: Option<u32>,
    pub building: Option<usize>,
    pub st: Option<Target>,
    pub rp: f64,
    pub auto: bool,
    pub hx: f64,
    pub hy: f64,
    pub sec: Option<usize>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Unit {
    pub id: u32,
    pub t: String,
    pub team: i32,
    pub x: f64,
    pub y: f64,
    pub hp: f64,
    pub ang: f64,
    pub tang: f64,
    pub path: Option<Vec<Point>>,
    pub pi: usize,
    pub order: Option<Order>,
    pub cd: f64,
    pub tgt: Option<u32>,
    pub scan: f64,
    pub dead: bool,
    pub sel: bool,
    pub hit: f64,
    pub sector: i32,
    pub near: f64,
    pub walk: f64,
    pub moving: bool,
    pub gren: i32,
    pub gcd: f64,
    pub squad: u32,
    pub cargo: Vec<Unit>,
    pub on_fort: Option<f64>,
    pub working: bool,
}
impl Unit {
    pub fn pos(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Building {
    pub id: usize,
    pub kind: String,
    pub tech: i32,
    pub team: i32,
    pub sector: i32,
    pub tx: i32,
    pub ty: i32,
    pub w: i32,
    pub h: i32,
    pub door: Door,
    pub hp: f64,
    pub maxhp: f64,
    pub prod: Option<String>,
    pub progress: f64,
    pub dead: bool,
    pub hit: f64,
    pub cx: f64,
    pub cy: f64,
    pub rally: Option<Point>,
    pub paused: bool,
    pub held: Option<String>,
    pub taken: Option<i32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Sector {
    pub id: usize,
    pub owner: i32,
    pub flag: Option<Point>,
    pub fx: i32,
    pub fy: i32,
    pub home: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Bridge {
    pub id: usize,
    pub tiles: Vec<usize>,
    pub hp: f64,
    pub maxhp: f64,
    pub dead: bool,
    pub hit: f64,
    pub rep: f64,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Crate {
    pub x: f64,
    pub y: f64,
    pub taken: bool,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Spawn {
    pub t: String,
    pub team: i32,
    pub tx: i32,
    pub ty: i32,
    pub sector: Option<i32>,
    pub squad_key: Option<String>,
    pub on_fort: Option<Building>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Map {
    #[serde(rename = "W")]
    pub w: usize,
    #[serde(rename = "H")]
    pub h: usize,
    pub tiles: Vec<u8>,
    pub block: Vec<u8>,
    pub sectors: Vec<Sector>,
    pub buildings: Vec<Building>,
    pub spawns: Vec<Spawn>,
    pub crates: Vec<Crate>,
    pub rock_hp: Vec<f64>,
    pub bridges: Vec<Bridge>,
    pub bridge_at: Vec<i32>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Ai {
    pub ai_rate: f64,
    pub ai_int: f64,
    pub ai_wait: f64,
    pub ai_rush: usize,
    pub ai_batch: usize,
    pub ai_min: usize,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub kills: usize,
    pub losses: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outcome {
    pub winner: i32,
    pub time: f64,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Shot {
    pub k: String,
    pub x: f64,
    pub y: f64,
    pub tx: f64,
    pub ty: f64,
    pub v: Option<u32>,
    pub st: Option<Target>,
    pub src: u32,
    pub team: i32,
    pub dmg: f64,
    pub splash: f64,
    pub sp: f64,
    pub ang: f64,
    pub arc: bool,
    pub x0: f64,
    pub y0: f64,
}
pub struct Game {
    pub map: Map,
    pub defs: BTreeMap<String, Def>,
    pub unit_order: Vec<String>,
    pub units: Vec<Unit>,
    pub shots: Vec<Shot>,
    pub fx: Vec<Value>,
    pub ev: Vec<Value>,
    pub dirty: Vec<usize>,
    pub time: f64,
    pub over: Option<Outcome>,
    pub nid: u32,
    pub ai_teams: Vec<i32>,
    pub ai_t: f64,
    pub ai: Ai,
    pub idx: i32,
    pub rng: u32,
    pub alert_t: f64,
    pub stats: [Stats; 2],
}
impl Game {
    pub fn rnd(&mut self) -> f64 {
        self.rng = self.rng.wrapping_add(0x6d2b79f5);
        let mut t = self.rng;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
    pub fn def(&self, u: &Unit) -> Def {
        self.defs.get(&u.t).cloned().unwrap_or_default()
    }
    pub fn ui(&self, id: u32) -> Option<usize> {
        self.units.iter().position(|u| u.id == id && !u.dead)
    }
    pub fn is_ai(&self, team: i32) -> bool {
        self.ai_teams.contains(&team)
    }
    pub fn force(&self, team: i32) -> usize {
        self.units
            .iter()
            .filter(|u| !u.dead && u.team == team && self.def(u).sp > 0.0)
            .map(|u| 1 + u.cargo.len())
            .sum()
    }
    pub fn rate(&self, team: i32) -> f64 {
        let own = self.map.sectors.iter().filter(|s| s.owner == team).count();
        (0.55 + 1.1 * own as f64 / self.map.sectors.len().max(1) as f64)
            * if team == 1 && self.is_ai(1) {
                self.ai.ai_rate
            } else {
                1.0
            }
    }
}
