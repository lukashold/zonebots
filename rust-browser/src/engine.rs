use crate::model::*;
use crate::path::{self, TS};
use serde_json::{json, Value};
use std::collections::BTreeMap;
fn num(v: &Value, k: &str, d: f64) -> f64 {
    v[k].as_f64().unwrap_or(d)
}
fn explosive(w: &str) -> bool {
    matches!(w, "shell" | "rocket" | "missile" | "grenade")
}
impl Game {
    pub fn init(v: &Value) -> Result<Self, String> {
        let map: Map = serde_json::from_value(v["map"].clone()).map_err(|e| e.to_string())?;
        let defs: BTreeMap<String, Def> =
            serde_json::from_value(v["defs"].clone()).map_err(|e| e.to_string())?;
        if map.w == 0
            || map.h == 0
            || map.w > 512
            || map.h > 512
            || map.tiles.len() != map.w * map.h
            || map.block.len() != map.w * map.h
            || defs.is_empty()
        {
            return Err("Invalid map or unit definitions".into());
        }
        let lv = &v["lv"];
        let diff = &v["diff"];
        let mut g = Self {
            map,
            defs,
            unit_order: serde_json::from_value(v["unitOrder"].clone()).unwrap_or_default(),
            units: vec![],
            shots: vec![],
            fx: vec![],
            ev: vec![],
            dirty: vec![],
            time: 0.,
            over: None,
            nid: 1,
            ai_teams: serde_json::from_value(v["aiTeams"].clone()).unwrap_or(vec![1]),
            ai_t: 1.,
            idx: num(lv, "idx", 0.) as i32,
            rng: num(lv, "seed", 1000.) as u32 ^ 0x9e3779b9,
            alert_t: 0.,
            stats: [Stats::default(), Stats::default()],
            ai: Ai {
                ai_rate: num(lv, "aiRate", 1.) * num(diff, "rate", 1.),
                ai_int: num(lv, "aiInt", 3.) * num(diff, "int", 1.),
                ai_wait: num(lv, "aiWait", 5.) * num(diff, "wait", 1.),
                ai_rush: num(lv, "aiRush", 12.) as usize,
                ai_batch: num(lv, "aiBatch", 3.) as usize,
                ai_min: num(lv, "aiMin", 6.) as usize,
            },
        };
        let mut keyed = BTreeMap::new();
        let mut counts = [0usize; 2];
        let mut sq = [0u32; 2];
        for s in g.map.spawns.clone() {
            if !g.defs.contains_key(&s.t) {
                return Err(format!("Unknown unit {}", s.t));
            }
            let id = g.spawn(
                &s.t,
                s.team,
                s.tx as f64 * TS + 16.,
                s.ty as f64 * TS + 16.,
                s.sector.unwrap_or(-1),
            );
            let i = g.ui(id).unwrap();
            if let Some(b) = s.on_fort {
                g.units[i].on_fort = Some((b.ty + b.h) as f64 * TS + 1.)
            }
            if g.def(&g.units[i]).cls == "robot" && (0..=1).contains(&s.team) {
                let team = s.team as usize;
                let squad = if let Some(key) = s.squad_key {
                    *keyed.entry(key).or_insert_with(|| {
                        g.nid += 1;
                        g.nid - 1
                    })
                } else {
                    if counts[team] % 4 == 0 {
                        sq[team] = g.nid;
                        g.nid += 1
                    }
                    counts[team] += 1;
                    sq[team]
                };
                g.units[i].squad = squad;
            }
        }
        for i in 0..g.map.buildings.len() {
            if let Some(p) = g.map.buildings[i]
                .extra
                .get("zprod")
                .and_then(Value::as_str)
            {
                g.map.buildings[i].prod = Some(p.to_string())
            }
            if g.map.buildings[i].team >= 0 {
                g.map.buildings[i].prod = g.pick_prod(i)
            }
        }
        Ok(g)
    }
    pub fn snapshot(&self) -> Value {
        json!({"units":self.units,"shots":self.shots,"fx":self.fx,"ev":self.ev,"dirty":self.dirty,"time":self.time,"over":self.over,"nid":self.nid,"stats":self.stats,"ai":self.ai,"aiT":self.ai_t,"rng":self.rng,"buildings":self.map.buildings,"sectors":self.map.sectors,"bridges":self.map.bridges,"crates":self.map.crates,"changes":self.dirty.iter().map(|&j|json!([j,self.map.tiles[j],self.map.block[j],self.map.rock_hp.get(j).copied().unwrap_or(0.)])).collect::<Vec<_>>()})
    }
    pub fn spawn(&mut self, t: &str, team: i32, x: f64, y: f64, sector: i32) -> u32 {
        let d = self.defs[t].clone();
        let id = self.nid;
        self.nid += 1;
        let cd = self.rnd() * d.rl;
        let scan = self.rnd() * 0.3;
        self.units.push(Unit {
            id,
            t: t.into(),
            team,
            x,
            y,
            hp: d.hp,
            ang: std::f64::consts::FRAC_PI_2,
            tang: std::f64::consts::FRAC_PI_2,
            cd,
            scan,
            sector,
            ..Default::default()
        });
        if d.cls == "gun" {
            if let Some(j) = self.tile(x, y) {
                self.map.block[j] = 1;
                self.dirty.push(j)
            }
        }
        id
    }
    pub fn tile(&self, x: f64, y: f64) -> Option<usize> {
        let tx = (x / TS).floor() as i32;
        let ty = (y / TS).floor() as i32;
        if tx < 0 || ty < 0 || tx >= self.map.w as i32 || ty >= self.map.h as i32 {
            None
        } else {
            Some(ty as usize * self.map.w + tx as usize)
        }
    }
    fn products(&self, b: &Building) -> Vec<String> {
        if b.dead || b.kind == "radar" || b.kind == "repair" {
            return vec![];
        }
        self.unit_order
            .iter()
            .filter(|k| {
                let d = &self.defs[*k];
                if !d.build_levels.is_empty() {
                    let required = *d.build_levels.get(&b.kind).unwrap_or(&0);
                    required > 0 && required <= b.tech
                } else {
                    d.tech <= b.tech && (b.kind == "fort" || d.cls == b.kind || d.cls == "gun")
                }
            })
            .cloned()
            .collect()
    }
    fn gun_count(&self, sec: i32) -> usize {
        self.units
            .iter()
            .filter(|u| !u.dead && self.def(u).cls == "gun" && u.sector == sec)
            .count()
    }
    fn pick_prod(&mut self, i: usize) -> Option<String> {
        let b = self.map.buildings[i].clone();
        let list = self.products(&b);
        if !self.is_ai(b.team) {
            return b
                .prod
                .filter(|p| list.contains(p))
                .or_else(|| list.first().cloned());
        }
        let guns = self.gun_count(b.sector);
        let need = self.repair_job(b.team, None).is_some()
            && !self
                .units
                .iter()
                .any(|u| !u.dead && u.team == b.team && self.def(u).crane);
        let mut best = None;
        let mut bs = -1.;
        for k in list {
            let d = self.defs[&k].clone();
            let mut s = d.tech as f64 * (0.6 + self.rnd());
            if d.cls == "gun" {
                s = if guns >= 2 { -1. } else { s * 0.45 }
            }
            if d.cls == "vehicle" {
                s *= 1.15
            }
            if d.crane {
                s = if need { 9. } else { -1. }
            }
            if d.cargo > 0 {
                s *= 0.25
            }
            if s > bs {
                bs = s;
                best = Some(k)
            }
        }
        best
    }
    fn path_to(&self, u: &mut Unit, x: f64, y: f64) -> bool {
        u.path = path::find(&self.map, u.pos(), Point { x, y });
        u.pi = 0;
        u.near = 0.;
        u.path.is_some()
    }
    fn move_units(&mut self, ids: &[u32], x: f64, y: f64, kind: &str) {
        let free = path::free_tiles(
            &self.map,
            (x / TS).floor() as i32,
            (y / TS).floor() as i32,
            ids.len().max(1),
        );
        if free.is_empty() {
            return;
        }
        let mut ti = 0;
        let mut q = 0;
        for id in ids {
            if let Some(i) = self.ui(*id) {
                let mut u = self.units[i].clone();
                let d = self.def(&u);
                if d.sp == 0. || u.team < 0 {
                    continue;
                }
                let mut p = if ids.len() == 1 && !path::blocked(&self.map, x, y) {
                    Point { x, y }
                } else {
                    free[ti.min(free.len() - 1)].clone()
                };
                if ids.len() > 1 && d.cls == "robot" {
                    p.x += [-8., 8., -8., 8.][q];
                    p.y += [-8., -8., 8., 8.][q];
                    q += 1;
                    if q == 4 {
                        q = 0;
                        ti += 1
                    }
                } else {
                    ti += 1
                }
                if self.path_to(&mut u, p.x, p.y) {
                    let last = u.path.as_ref().unwrap().last().unwrap();
                    u.order = Some(Order {
                        k: kind.into(),
                        x: last.x,
                        y: last.y,
                        ..Default::default()
                    })
                } else {
                    u.order = None
                }
                self.units[i] = u
            }
        }
    }
    pub fn command(&mut self, v: &Value) {
        let op = v["kind"].as_str().unwrap_or("");
        let ids: Vec<u32> = serde_json::from_value(v["ids"].clone()).unwrap_or_default();
        let target = v["target"].as_u64().map(|x| x as u32);
        let st: Option<Target> = serde_json::from_value(v["st"].clone()).ok();
        let x = num(v, "x", 0.);
        let y = num(v, "y", 0.);
        if op == "move" || op == "amove" {
            self.move_units(&ids, x, y, op);
            return;
        }
        let mut enter = ids.clone();
        if op == "enter" {
            if let Some(t) = target.and_then(|id| self.ui(id)) {
                let p = self.units[t].pos();
                enter.sort_by(|a, b| {
                    let da = self
                        .ui(*a)
                        .filter(|&i| self.def(&self.units[i]).cls == "robot")
                        .map(|i| self.units[i].pos().distance(&p))
                        .unwrap_or(f64::INFINITY);
                    let db = self
                        .ui(*b)
                        .filter(|&i| self.def(&self.units[i]).cls == "robot")
                        .map(|i| self.units[i].pos().distance(&p))
                        .unwrap_or(f64::INFINITY);
                    da.total_cmp(&db)
                });
                if enter.len() > 1 {
                    self.move_units(&enter[1..], p.x, p.y, "move");
                    enter.truncate(1)
                }
            }
        }
        for id in enter {
            let Some(i) = self.ui(id) else { continue };
            let mut u = self.units[i].clone();
            let d = self.def(&u);
            if u.team < 0 {
                continue;
            }
            match op {
                "stop" => {
                    u.order = None;
                    u.path = None
                }
                "attack" => {
                    if d.dmg > 0. && target != Some(id) {
                        u.order = Some(Order {
                            k: "attack".into(),
                            target,
                            ..Default::default()
                        });
                        u.path = None
                    }
                }
                "enter" | "board" => {
                    if d.cls == "robot" {
                        u.order = Some(Order {
                            k: op.into(),
                            target,
                            ..Default::default()
                        });
                        u.path = None
                    }
                }
                "unload" => self.unload(&mut u),
                "hit" | "fix" => {
                    if let Some(s) = &st {
                        if op == "fix" && !d.crane {
                            continue;
                        }
                        if op == "hit" && d.sp == 0. {
                            continue;
                        }
                        if op == "hit"
                            && s.kind != "b"
                            && !explosive(d.w.as_deref().unwrap_or(""))
                            && u.gren == 0
                        {
                            continue;
                        }
                        let storm = op == "hit"
                            && s.kind == "b"
                            && d.cls == "robot"
                            && self
                                .map
                                .buildings
                                .get(s.id)
                                .is_some_and(|b| b.kind == "fort");
                        u.order = Some(Order {
                            k: if storm { "storm" } else { op }.into(),
                            building: if storm { Some(s.id) } else { None },
                            st: if storm { None } else { Some(s.clone()) },
                            ..Default::default()
                        });
                        u.path = None
                    }
                }
                _ => {}
            }
            self.units[i] = u;
        }
    }
    pub fn sync_controls(&mut self, v: &Value) {
        if let Some(n) = v["nid"].as_u64() {
            self.nid = self.nid.max(n as u32)
        }
        if let Some(us) = v["units"].as_array() {
            for a in us {
                if let Some(i) = a[0].as_u64().and_then(|id| self.ui(id as u32)) {
                    self.units[i].sel = a[1].as_bool().unwrap_or(false);
                    self.units[i].squad = a[2].as_u64().unwrap_or(0) as u32
                }
            }
        }
        if let Some(bs) = v["buildings"].as_array() {
            for c in bs {
                let id = c["id"].as_u64().unwrap_or(usize::MAX as u64) as usize;
                let Some(i) = self.map.buildings.iter().position(|b| b.id == id) else {
                    continue;
                };
                if self.map.buildings[i].team != 0 {
                    continue;
                }
                let b = self.map.buildings[i].clone();
                let prod = c["prod"].as_str().map(str::to_string);
                if prod.is_none() || self.products(&b).contains(prod.as_ref().unwrap()) {
                    if prod != b.prod {
                        self.map.buildings[i].progress = 0.
                    }
                    self.map.buildings[i].prod = prod;
                }
                self.map.buildings[i].rally = serde_json::from_value(c["rally"].clone()).ok();
                self.map.buildings[i].held = c["held"].as_str().map(str::to_string);
                self.map.buildings[i].paused = c["paused"].as_bool().unwrap_or(false);
            }
        }
    }
    fn unload(&mut self, u: &mut Unit) {
        let tiles = path::free_tiles(
            &self.map,
            (u.x / TS) as i32,
            (u.y / TS) as i32,
            u.cargo.len() + 1,
        );
        if tiles.len() < 2 {
            return;
        }
        for (i, mut c) in std::mem::take(&mut u.cargo).into_iter().enumerate() {
            let p = &tiles[1 + i % (tiles.len() - 1)];
            c.x = p.x + [-4., 4., -4., 4.][i % 4];
            c.y = p.y + [-4., -4., 4., 4.][i % 4];
            c.dead = false;
            c.sel = false;
            c.order = None;
            c.path = None;
            c.tgt = None;
            self.units.push(c)
        }
        self.ev
            .push(json!({"k":"unload","team":u.team,"x":u.x,"y":u.y}));
    }
    fn end(&mut self, winner: i32) {
        if self.over.is_none() {
            self.over = Some(Outcome {
                winner,
                time: self.time,
            });
            self.ev.push(json!({"k":"over","winner":winner}));
        }
    }
    fn retile(&mut self, j: usize, t: u8) {
        self.map.tiles[j] = t;
        self.map.block[j] = if matches!(t, 2 | 3 | 6) { 1 } else { 0 };
        self.dirty.push(j)
    }
    fn target_alive(&self, st: &Target) -> bool {
        match st.kind.as_str() {
            "b" => self.map.buildings.get(st.id).is_some_and(|b| !b.dead),
            "br" => self.map.bridges.get(st.id).is_some_and(|b| !b.dead),
            "rock" => self.map.tiles.get(st.id) == Some(&6),
            _ => false,
        }
    }
    fn target_dist(&self, u: &Unit, st: &Target) -> f64 {
        if st.kind == "b" {
            if let Some(b) = self.map.buildings.get(st.id) {
                return (u.x - u.x.clamp(b.tx as f64 * TS, (b.tx + b.w) as f64 * TS))
                    .hypot(u.y - u.y.clamp(b.ty as f64 * TS, (b.ty + b.h) as f64 * TS));
            }
        }
        if st.kind == "br" {
            if let Some(b) = self.map.bridges.get(st.id) {
                return b
                    .tiles
                    .iter()
                    .map(|&j| {
                        (u.x - ((j % self.map.w) as f64 * TS + 16.))
                            .hypot(u.y - ((j / self.map.w) as f64 * TS + 16.))
                    })
                    .fold(f64::INFINITY, f64::min)
                    .sub(16.)
                    .max(0.);
            }
        }
        (u.x - st.x).hypot(u.y - st.y).sub(16.).max(0.)
    }
    fn repair_job(&self, team: i32, from: Option<Point>) -> Option<Target> {
        let mut all = vec![];
        for b in &self.map.buildings {
            if !b.dead && b.team == team && b.hp < b.maxhp * 0.8 {
                all.push(Target {
                    kind: "b".into(),
                    id: b.id,
                    x: b.cx,
                    y: b.cy,
                })
            }
        }
        for b in &self.map.bridges {
            if b.dead {
                all.push(Target {
                    kind: "br".into(),
                    id: b.id,
                    x: b.x,
                    y: b.y,
                })
            }
        }
        all.into_iter().min_by(|a, b| {
            let p = from.clone().unwrap_or_default();
            p.distance(&Point { x: a.x, y: a.y })
                .total_cmp(&p.distance(&Point { x: b.x, y: b.y }))
        })
    }
    fn damage(&mut self, id: u32, amt: f64, src: Option<(u32, i32)>) {
        let Some(i) = self.ui(id) else { return };
        let mut u = self.units[i].clone();
        let d = self.def(&u);
        u.hp -= amt;
        u.hit = 0.12;
        if u.hp <= 0. {
            u.hp = 0.;
            u.dead = true;
            let big = d.cls != "robot";
            self.fx.push(json!({"k":if big{"boom"}else{"die"},"x":u.x,"y":u.y,"t":0,"life":0.7,"r":if big{26}else{12},"team":u.team}));
            self.ev.push(json!({"k":"boom","x":u.x,"y":u.y,"big":big}));
            if d.cls == "gun" && u.on_fort.is_none() {
                if let Some(j) = self.tile(u.x, u.y) {
                    self.map.block[j] = 0;
                    self.dirty.push(j)
                }
            }
            if (0..=1).contains(&u.team) {
                let n = 1 + u.cargo.len();
                self.stats[u.team as usize].losses += n;
                if let Some((_, team)) = src {
                    if (0..=1).contains(&team) && team != u.team {
                        self.stats[team as usize].kills += n;
                        self.ev
                            .push(json!({"k":"kill","team":team,"x":u.x,"y":u.y}));
                    }
                }
            }
            u.cargo.clear();
        } else if let Some((sid, team)) = src {
            if team >= 0 && team != u.team && u.team >= 0 {
                self.ev
                    .push(json!({"k":"hurt","team":u.team,"x":u.x,"y":u.y}));
                if u.order.is_none() && d.sp > 0. && d.dmg > 0. {
                    u.order = Some(Order {
                        k: "attack".into(),
                        target: Some(sid),
                        ..Default::default()
                    })
                }
            }
        }
        self.units[i] = u;
    }
    fn hit_structure(&mut self, st: &Target, amt: f64, src: Option<(u32, i32)>) {
        if !self.target_alive(st) {
            return;
        }
        match st.kind.as_str() {
            "b" => {
                let b = &mut self.map.buildings[st.id];
                if b.team < 0 {
                    return;
                }
                b.hp -= amt;
                b.hit = 0.12;
                if b.team == 0 && self.alert_t <= 0. {
                    self.alert_t = 12.;
                    self.ev
                        .push(json!({"k":"alert","x":b.cx,"y":b.cy,"fort":b.kind=="fort"}));
                }
                if b.hp <= 0. {
                    b.hp = 0.;
                    b.dead = true;
                    b.prod = None;
                    b.progress = 0.;
                    let (x, y, team, kind) = (b.cx, b.cy, b.team, b.kind.clone());
                    self.fx.push(json!({"k":"boom","x":x,"y":y,"t":0,"life":1.2,"r":if kind=="fort"{80}else{50}}));
                    self.ev
                        .push(json!({"k":"bdead","team":team,"kind":kind,"x":x,"y":y}));
                    if kind == "fort" {
                        self.end(1 - team)
                    }
                }
            }
            "br" => {
                let b = &mut self.map.bridges[st.id];
                b.hp -= amt;
                b.hit = 0.12;
                if b.hp <= 0. {
                    b.dead = true;
                    b.rep = 0.;
                    let b = b.clone();
                    for j in b.tiles {
                        self.retile(j, 3);
                        let ids: Vec<_> = self
                            .units
                            .iter()
                            .filter(|u| {
                                !u.dead && self.def(u).sp > 0. && self.tile(u.x, u.y) == Some(j)
                            })
                            .map(|u| u.id)
                            .collect();
                        for id in ids {
                            self.damage(id, 1e6, None)
                        }
                    }
                    self.ev
                        .push(json!({"k":"bridge","x":b.x,"y":b.y,"up":false}));
                }
            }
            "rock" => {
                if let Some(hp) = self.map.rock_hp.get_mut(st.id) {
                    *hp -= amt;
                    if *hp <= 0. {
                        self.retile(st.id, 1);
                        self.fx
                            .push(json!({"k":"boom","x":st.x,"y":st.y,"t":0,"life":0.5,"r":20}));
                        self.ev.push(json!({"k":"rock","x":st.x,"y":st.y}));
                    }
                }
            }
            _ => {}
        }
        let _ = src;
    }
    fn nearest_enemy(&self, u: &Unit, range: f64, min: f64) -> Option<u32> {
        self.units
            .iter()
            .filter(|v| !v.dead && v.team >= 0 && v.team != u.team)
            .filter_map(|v| {
                let d = u.pos().distance(&v.pos());
                if d < range && d >= min {
                    Some((v.id, d))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|p| p.0)
    }
    fn struct_mul(w: &str, st: &Target) -> f64 {
        if explosive(w) {
            1.
        } else if st.kind != "b" {
            if w == "laser" {
                0.3
            } else {
                0.
            }
        } else if w == "laser" {
            0.6
        } else {
            0.2
        }
    }
    fn fire(&mut self, u: &mut Unit, v: Option<Unit>, st: Option<Target>) {
        let d = self.def(u);
        let w = d.w.as_deref().unwrap_or("");
        let (tx, ty) = if let Some(v) = &v {
            (v.x, v.y)
        } else if let Some(s) = &st {
            (s.x, s.y)
        } else {
            return;
        };
        u.cd = d.rl * (0.9 + self.rnd() * 0.2);
        let a = (ty - u.y).atan2(tx - u.x);
        let (mx, my) = (u.x + a.cos() * (d.r + 3.), u.y + a.sin() * (d.r + 3.));
        self.ev
            .push(json!({"k":"shot","w":w,"t":u.t,"x":u.x,"y":u.y}));
        if explosive(w) {
            self.shots.push(Shot {
                k: w.into(),
                x: mx,
                y: my,
                tx,
                ty,
                v: if w == "shell" { None } else { v.map(|v| v.id) },
                st,
                src: u.id,
                team: u.team,
                dmg: d.dmg,
                splash: d.splash,
                sp: if w == "shell" {
                    if u.t == "howitzer" {
                        260.
                    } else {
                        430.
                    }
                } else if w == "rocket" {
                    300.
                } else {
                    270.
                },
                ang: a,
                arc: u.t == "howitzer",
                x0: mx,
                y0: my,
            });
        } else {
            let jx = (self.rnd() - 0.5) * 8.;
            let jy = (self.rnd() - 0.5) * 8.;
            self.fx.push(json!({"k":match w{"flame"=>"flame","laser"=>"beam",_=>"line"},"x":mx,"y":my,"x2":tx+jx,"y2":ty+jy,"t":0,"life":match w{"flame"=>0.3,"laser"=>0.14,_=>0.07}}));
            if let Some(v) = v {
                if w != "bullet" || self.rnd() < 0.88 {
                    let armor = self.def(&v).armor;
                    self.damage(
                        v.id,
                        d.dmg
                            * match w {
                                "bullet" | "flame" => 1. - armor,
                                "laser" => 1. - armor / 2.,
                                _ => 1.,
                            },
                        Some((u.id, u.team)),
                    );
                    if u.t == "sniper" && self.def(&v).cls != "robot" {
                        let chance = match v.t.as_str() {
                            "jeep" | "crane" => 0.3,
                            "apc" | "light" | "launcher" => 0.15,
                            "medium" => 0.08,
                            "heavy" => 0.04,
                            _ => 0.2,
                        };
                        if self.rnd() < chance {
                            if let Some(i) = self.ui(v.id) {
                                let mut v = self.units[i].clone();
                                v.team = -1;
                                v.order = None;
                                v.path = None;
                                v.tgt = None;
                                v.sel = false;
                                self.unload(&mut v);
                                self.units[i] = v;
                                self.ev
                                    .push(json!({"k":"driver","x":tx,"y":ty,"team":u.team}));
                            }
                        }
                    }
                }
            } else if let Some(st) = st {
                self.hit_structure(&st, d.dmg * Self::struct_mul(w, &st), Some((u.id, u.team)))
            }
        }
    }
}
use std::ops::Sub;
impl Game {
    fn think_unit(&mut self, i: usize, dt: f64) {
        let mut u = self.units[i].clone();
        let d = self.def(&u);
        u.cd -= dt;
        u.scan -= dt;
        u.hit = (u.hit - dt).max(0.);
        u.moving = false;
        u.working = false;
        if u.team < 0 {
            self.units[i] = u;
            return;
        }
        let mut hold = false;
        let mut tst = None;
        if let Some(mut o) = u.order.clone() {
            match o.k.as_str() {
                "attack" => {
                    if let Some(v) = o
                        .target
                        .and_then(|id| self.ui(id))
                        .map(|i| self.units[i].clone())
                        .filter(|v| {
                            v.team != u.team
                                && v.team >= 0
                                && (!o.auto || (v.x - o.hx).hypot(v.y - o.hy) <= d.range + 300.)
                        })
                    {
                        let dd = u.pos().distance(&v.pos());
                        if dd <= d.range * 0.92 && dd >= d.min_range {
                            hold = true;
                            u.path = None;
                            u.tgt = Some(v.id);
                        } else if d.sp == 0. {
                            u.order = None
                        } else {
                            o.rp -= dt;
                            if u.path.is_none() || o.rp <= 0. {
                                o.rp = 0.8;
                                if !self.path_to(&mut u, v.x, v.y) {
                                    u.order = None
                                } else {
                                    u.order = Some(o)
                                }
                            } else {
                                u.order = Some(o)
                            }
                        }
                    } else {
                        u.order = None;
                        u.path = None;
                        if o.auto
                            && (u.x - o.hx).hypot(u.y - o.hy) > 40.
                            && self.path_to(&mut u, o.hx, o.hy)
                        {
                            u.order = Some(Order {
                                k: "move".into(),
                                x: o.hx,
                                y: o.hy,
                                ..Default::default()
                            })
                        }
                    }
                }
                "enter" | "board" => {
                    let board = o.k == "board";
                    if let Some(vi) = o.target.and_then(|id| self.ui(id)) {
                        let mut v = self.units[vi].clone();
                        let vd = self.def(&v);
                        if (board
                            && (v.team != u.team || vd.cargo == 0 || v.cargo.len() >= vd.cargo))
                            || (!board && v.team >= 0)
                        {
                            u.order = None;
                            u.path = None;
                        } else if u.pos().distance(&v.pos()) < d.r + vd.r + 8. {
                            u.order = None;
                            u.path = None;
                            u.tgt = None;
                            u.sel = false;
                            u.dead = true;
                            if board {
                                v.cargo.push(u.clone());
                                self.ev
                                    .push(json!({"k":"board","x":v.x,"y":v.y,"team":u.team}));
                            } else {
                                v.team = u.team;
                                v.order = None;
                                v.hp = v.hp.max(vd.hp * 0.5);
                                self.ev.push(json!({"k":"enter","x":v.x,"y":v.y,"team":u.team,"gun":vd.cls=="gun"}));
                            }
                            self.units[vi] = v;
                            self.units[i] = u;
                            return;
                        } else {
                            o.rp -= dt;
                            if u.path.is_none() || o.rp <= 0. {
                                o.rp = 1.5;
                                if !self.path_to(&mut u, v.x, v.y) {
                                    u.order = None
                                } else {
                                    u.order = Some(o)
                                }
                            } else {
                                u.order = Some(o)
                            }
                        }
                    } else {
                        u.order = None;
                        u.path = None
                    }
                }
                "storm" => {
                    if let Some(b) = o
                        .building
                        .and_then(|id| self.map.buildings.get(id))
                        .cloned()
                        .filter(|b| !b.dead && b.team != u.team)
                    {
                        let p = Point {
                            x: b.door.tx as f64 * TS + 16.,
                            y: b.door.ty as f64 * TS + 16.,
                        };
                        if u.pos().distance(&p) < 13. {
                            self.map.buildings[b.id].taken = Some(u.team);
                            self.map.buildings[b.id].dead = true;
                            self.fx.push(
                                json!({"k":"boom","x":b.cx,"y":b.cy,"t":0,"life":1.2,"r":60}),
                            );
                            self.ev
                                .push(json!({"k":"stormed","team":u.team,"x":p.x,"y":p.y}));
                            self.end(u.team);
                        } else if u.path.is_none() && !self.path_to(&mut u, p.x, p.y) {
                            u.order = None
                        }
                        if b.team == 0 && self.alert_t <= 0. && u.pos().distance(&p) < 260. {
                            self.alert_t = 12.;
                            self.ev
                                .push(json!({"k":"alert","x":b.cx,"y":b.cy,"fort":true}));
                        }
                    } else {
                        u.order = None;
                        u.path = None
                    }
                }
                "hit" => {
                    if let Some(st) = o.st {
                        if !self.target_alive(&st)
                            || (st.kind == "b"
                                && self
                                    .map
                                    .buildings
                                    .get(st.id)
                                    .is_some_and(|b| b.team == u.team || b.team < 0))
                        {
                            u.order = None;
                            u.path = None
                        } else {
                            let dd = self.target_dist(&u, &st);
                            let reach = if d.dmg > 0. { d.range * 0.9 } else { 120. };
                            if dd <= reach && dd >= d.min_range {
                                hold = true;
                                u.path = None;
                                tst = Some(st)
                            } else if u.path.is_none() && !self.path_to(&mut u, st.x, st.y) {
                                u.order = None
                            }
                        }
                    }
                }
                "fix" => {
                    if let Some(st) = o.st {
                        let valid = if st.kind == "b" {
                            self.map
                                .buildings
                                .get(st.id)
                                .is_some_and(|b| !b.dead && b.team == u.team && b.hp < b.maxhp)
                        } else {
                            self.map.bridges.get(st.id).is_some_and(|b| b.dead)
                        };
                        if !valid {
                            u.order = None;
                            u.path = None
                        } else if self.target_dist(&u, &st) <= 40. {
                            hold = true;
                            u.path = None;
                            u.working = true;
                            if st.kind == "b" {
                                let b = &mut self.map.buildings[st.id];
                                b.hp = (b.hp + 70. * dt).min(b.maxhp);
                                if b.hp >= b.maxhp {
                                    u.order = None;
                                    self.ev
                                        .push(json!({"k":"fixed","team":u.team,"x":b.cx,"y":b.cy}));
                                }
                            } else {
                                let br = &mut self.map.bridges[st.id];
                                br.rep += dt / (2. + br.tiles.len() as f64 * 0.6);
                                if br.rep >= 1. {
                                    let br = br.clone();
                                    let occupied = self.units.iter().any(|v| {
                                        !v.dead
                                            && self.def(v).cls == "gun"
                                            && self
                                                .tile(v.x, v.y)
                                                .is_some_and(|j| br.tiles.contains(&j))
                                    });
                                    if !occupied {
                                        for j in br.tiles {
                                            self.retile(j, 5)
                                        }
                                        let b = &mut self.map.bridges[st.id];
                                        b.dead = false;
                                        b.hp = b.maxhp;
                                        b.rep = 0.;
                                        self.ev.push(json!({"k":"bridge","x":br.x,"y":br.y,"up":true,"team":u.team}));
                                    }
                                    u.order = None;
                                }
                            }
                        } else if u.path.is_none() && !self.path_to(&mut u, st.x, st.y) {
                            u.order = None
                        }
                    }
                }
                "amove" => {
                    if o.sec
                        .is_some_and(|s| self.map.sectors.get(s).is_some_and(|s| s.owner == u.team))
                    {
                        u.order = None;
                        u.path = None
                    } else if u.tgt.and_then(|id| self.ui(id)).is_some() {
                        hold = true
                    } else if u.path.is_none() {
                        if (u.x - o.x).hypot(u.y - o.y) < 24. || !self.path_to(&mut u, o.x, o.y) {
                            u.order = None
                        }
                    }
                }
                "move" => {
                    if u.path.is_none() {
                        u.order = None
                    }
                }
                _ => {}
            }
        }
        if d.dmg > 0. {
            if !u.tgt.and_then(|id| self.ui(id)).is_some_and(|i| {
                let v = &self.units[i];
                let dd = u.pos().distance(&v.pos());
                v.team >= 0 && v.team != u.team && dd <= d.range && dd >= d.min_range
            }) {
                u.tgt = None
            }
            if u.scan <= 0. {
                u.scan = 0.3;
                let ov = u
                    .order
                    .as_ref()
                    .filter(|o| o.k == "attack")
                    .and_then(|o| o.target)
                    .and_then(|id| self.ui(id))
                    .filter(|&i| {
                        let v = &self.units[i];
                        let dd = u.pos().distance(&v.pos());
                        v.team >= 0 && v.team != u.team && dd <= d.range && dd >= d.min_range
                    });
                u.tgt = ov
                    .map(|i| self.units[i].id)
                    .or_else(|| self.nearest_enemy(&u, d.range, d.min_range));
                if u.tgt.is_none() && u.order.is_none() && d.sp > 0. && d.cargo == 0 {
                    if let Some(id) = self.nearest_enemy(&u, d.range + 150., d.min_range) {
                        u.order = Some(Order {
                            k: "attack".into(),
                            target: Some(id),
                            auto: true,
                            hx: u.x,
                            hy: u.y,
                            ..Default::default()
                        })
                    }
                }
            }
            if let Some(v) = u
                .tgt
                .and_then(|id| self.ui(id))
                .map(|i| self.units[i].clone())
            {
                u.tang = (v.y - u.y).atan2(v.x - u.x);
                if d.cls == "robot" {
                    u.ang = u.tang
                }
                if u.cd <= 0. {
                    self.fire(&mut u, Some(v), None)
                }
            } else if let Some(st) = &tst {
                u.tang = (st.y - u.y).atan2(st.x - u.x);
                if d.cls == "robot" {
                    u.ang = u.tang
                }
                if u.cd <= 0. && Self::struct_mul(d.w.as_deref().unwrap_or(""), st) > 0. {
                    self.fire(&mut u, None, Some(st.clone()))
                }
            } else if d.cls != "gun" {
                u.tang = u.ang
            }
        }
        if u.gren > 0 {
            u.gcd -= dt;
            if u.gcd <= 0. {
                let gv = u
                    .tgt
                    .and_then(|id| self.ui(id))
                    .map(|i| self.units[i].clone())
                    .filter(|v| self.def(v).cls != "robot" && u.pos().distance(&v.pos()) <= 140.);
                let gs = tst
                    .clone()
                    .filter(|s| gv.is_none() && self.target_dist(&u, s) <= 140.);
                if gv.is_some() || gs.is_some() {
                    u.gren -= 1;
                    u.gcd = 2.5;
                    let (tx, ty) = gv.map(|v| (v.x, v.y)).unwrap_or_else(|| {
                        let s = gs.as_ref().unwrap();
                        (s.x, s.y)
                    });
                    self.shots.push(Shot {
                        k: "grenade".into(),
                        x: u.x,
                        y: u.y,
                        tx,
                        ty,
                        st: gs,
                        src: u.id,
                        team: u.team,
                        dmg: 70.,
                        splash: 30.,
                        sp: 230.,
                        arc: true,
                        x0: u.x,
                        y0: u.y,
                        ..Default::default()
                    });
                }
            }
        }
        if !hold && d.sp > 0. {
            if let Some(pts) = u.path.clone() {
                if let Some(p) = pts.get(u.pi) {
                    let dx = p.x - u.x;
                    let dy = p.y - u.y;
                    let dd = dx.hypot(dy);
                    let ti = self.tile(u.x, u.y).unwrap_or(0);
                    let speed = d.sp
                        * [0.8, 1., 0.8, 0.8, 0.5, 0.95, 0.8][self.map.tiles[ti].min(6) as usize]
                        * dt;
                    let last = u.pi + 1 == pts.len();
                    if path::blocked(&self.map, p.x, p.y) && !last {
                        let end = pts.last().unwrap();
                        self.path_to(&mut u, end.x, end.y);
                    } else {
                        if last && dd < 26. {
                            u.near += dt
                        }
                        if dd <= speed || (last && u.near > 1.2) {
                            if dd <= speed {
                                u.x = p.x;
                                u.y = p.y
                            }
                            u.pi += 1;
                            if u.pi >= pts.len() {
                                u.path = None
                            }
                        } else {
                            u.x += dx / dd * speed;
                            u.y += dy / dd * speed;
                            u.ang = dy.atan2(dx);
                            u.moving = true;
                            u.walk += speed * 0.2;
                            if u.tgt.is_none() && tst.is_none() {
                                u.tang = u.ang
                            }
                        }
                    }
                } else {
                    u.path = None
                }
            }
        }
        // Damage can have been applied to this unit by bridge explosions during its action.
        if self.units[i].dead {
            u.dead = true;
            u.hp = 0.
        }
        self.units[i] = u;
    }
    fn separate(&mut self) {
        for i in 0..self.units.len() {
            if self.units[i].dead {
                continue;
            }
            for j in i + 1..self.units.len() {
                if self.units[j].dead {
                    continue;
                }
                let a = &self.units[i];
                let b = &self.units[j];
                let ad = self.def(a);
                let bd = self.def(b);
                let rr = ad.r + bd.r;
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let dd = dx.hypot(dy);
                if dd >= rr {
                    continue;
                }
                let af = ad.sp == 0. || a.team < 0;
                let bf = bd.sp == 0. || b.team < 0;
                if af && bf {
                    continue;
                }
                if a.path.is_none() && b.path.is_none() && rr - dd < 3. {
                    continue;
                }
                let (nx, ny) = if dd < 0.01 {
                    let an = (a.id as f64 * 2.4 + b.id as f64) % 6.28;
                    (an.cos(), an.sin())
                } else {
                    (dx / dd, dy / dd)
                };
                let push = 1.5f64.min((rr - dd) * 0.35);
                let pa = if af {
                    0.
                } else if bf {
                    push * 2.
                } else {
                    push
                };
                let pb = if bf {
                    0.
                } else if af {
                    push * 2.
                } else {
                    push
                };
                let (ax, ay, bx, by) = (a.x - nx * pa, a.y - ny * pa, b.x + nx * pb, b.y + ny * pb);
                if pa > 0. && !path::blocked(&self.map, ax, ay) {
                    self.units[i].x = ax;
                    self.units[i].y = ay
                }
                if pb > 0. && !path::blocked(&self.map, bx, by) {
                    self.units[j].x = bx;
                    self.units[j].y = by
                }
            }
        }
    }
    fn capture(&mut self) {
        for si in 0..self.map.sectors.len() {
            let s = self.map.sectors[si].clone();
            let Some(flag) = s.flag else { continue };
            let team = self
                .units
                .iter()
                .find(|u| {
                    !u.dead
                        && u.team >= 0
                        && u.team != s.owner
                        && self.def(u).sp > 0.
                        && (u.x - flag.x).abs() < 20.
                        && (u.y - flag.y).abs() < 20.
                })
                .map(|u| u.team);
            if let Some(team) = team {
                self.map.sectors[si].owner = team;
                let mut radar = false;
                for bi in 0..self.map.buildings.len() {
                    if self.map.buildings[bi].sector != s.id as i32
                        || self.map.buildings[bi].kind == "fort"
                    {
                        continue;
                    }
                    let b = &mut self.map.buildings[bi];
                    b.team = team;
                    if !b.dead {
                        b.progress = 0.;
                        b.rally = None;
                        radar |= b.kind == "radar";
                        self.map.buildings[bi].prod = self.pick_prod(bi)
                    }
                }
                self.ev.push(json!({"k":"capture","team":team,"prev":s.owner,"x":flag.x,"y":flag.y,"radar":radar}));
            }
        }
        for ci in 0..self.map.crates.len() {
            let c = self.map.crates[ci].clone();
            if c.taken {
                continue;
            }
            if let Some(u) = self
                .units
                .iter()
                .find(|u| {
                    !u.dead
                        && u.team >= 0
                        && self.def(u).cls == "robot"
                        && (u.x - c.x).abs() <= 18.
                        && (u.y - c.y).abs() <= 18.
                })
                .cloned()
            {
                self.map.crates[ci].taken = true;
                let ids: Vec<_> = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| {
                        !v.dead
                            && v.team == u.team
                            && self.def(v).cls == "robot"
                            && u.pos().distance(&v.pos()) < 80.
                    })
                    .map(|(i, _)| i)
                    .collect();
                for i in ids {
                    self.units[i].gren = 3
                }
                self.ev
                    .push(json!({"k":"crate","team":u.team,"x":c.x,"y":c.y}));
            }
        }
    }
    fn update_shots(&mut self, dt: f64) {
        for mut s in std::mem::take(&mut self.shots) {
            if let Some(i) = s.v.and_then(|id| self.ui(id)) {
                s.tx = self.units[i].x;
                s.ty = self.units[i].y
            }
            let dx = s.tx - s.x;
            let dy = s.ty - s.y;
            let dd = dx.hypot(dy);
            if dd > s.sp * dt {
                s.x += dx / dd * s.sp * dt;
                s.y += dy / dd * s.sp * dt;
                s.ang = dy.atan2(dx);
                self.shots.push(s);
                continue;
            }
            self.fx
                .push(json!({"k":"boom","x":s.tx,"y":s.ty,"t":0,"life":0.3,"r":s.splash*0.7+6.}));
            self.ev
                .push(json!({"k":"boom","x":s.tx,"y":s.ty,"big":false}));
            if let Some(st) = &s.st {
                self.hit_structure(st, s.dmg, Some((s.src, s.team)))
            }
            let hits: Vec<_> = self
                .units
                .iter()
                .filter(|u| !u.dead && u.team >= 0 && u.team != s.team)
                .filter_map(|u| {
                    let dd = (u.x - s.tx).hypot(u.y - s.ty) - self.def(u).r;
                    if dd < s.splash {
                        Some((
                            u.id,
                            s.dmg
                                * if dd <= 4. {
                                    1.
                                } else {
                                    1. - 0.6 * dd / s.splash
                                },
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            for (id, dmg) in hits {
                self.damage(id, dmg, Some((s.src, s.team)))
            }
            if s.st.is_none() {
                if let Some(j) = self.tile(s.tx, s.ty) {
                    let bi = self.map.bridge_at.get(j).copied().unwrap_or(-1);
                    let kind = if bi >= 0 { "br" } else { "rock" };
                    if bi >= 0 || self.map.tiles[j] == 6 {
                        self.hit_structure(
                            &Target {
                                kind: kind.into(),
                                id: if bi >= 0 { bi as usize } else { j },
                                x: s.tx,
                                y: s.ty,
                            },
                            s.dmg * 0.3,
                            Some((s.src, s.team)),
                        )
                    }
                }
            }
        }
    }
    fn production(&mut self, dt: f64) {
        let mut counts = [self.force(0), self.force(1)];
        for bi in 0..self.map.buildings.len() {
            self.map.buildings[bi].hit = (self.map.buildings[bi].hit - dt).max(0.);
            let b = self.map.buildings[bi].clone();
            if b.dead || !(0..=1).contains(&b.team) {
                continue;
            }
            if b.kind == "radar" {
                continue;
            }
            if b.kind == "repair" {
                let p = Point {
                    x: b.door.tx as f64 * TS + 16.,
                    y: b.door.ty as f64 * TS + 16.,
                };
                for i in 0..self.units.len() {
                    let u = &self.units[i];
                    let d = self.def(u);
                    if !u.dead && u.team == b.team && u.hp < d.hp && u.pos().distance(&p) < 100. {
                        self.units[i].hp = (u.hp + 22. * dt).min(d.hp)
                    }
                }
                continue;
            }
            let Some(prod) = b.prod.clone() else { continue };
            let Some(d) = self.defs.get(&prod).cloned() else {
                continue;
            };
            if !self.products(&b).contains(&prod) || b.paused {
                continue;
            }
            let gun = d.cls == "gun";
            if (gun && self.gun_count(b.sector) >= 3) || (!gun && counts[b.team as usize] >= 50) {
                if gun && self.is_ai(b.team) {
                    self.map.buildings[bi].prod = self.pick_prod(bi)
                }
                continue;
            }
            self.map.buildings[bi].progress +=
                dt * self.rate(b.team) * (0.4 + 0.6 * b.hp / b.maxhp) / d.time.max(0.01);
            if self.map.buildings[bi].progress < 1. {
                continue;
            }
            let mut made = vec![];
            if gun {
                if let Some(s) = self.map.sectors.get(b.sector.max(0) as usize).cloned() {
                    for (dx, dy) in [
                        (-3, 0),
                        (3, 0),
                        (-3, -2),
                        (3, -2),
                        (4, -1),
                        (-3, -4),
                        (4, -4),
                        (3, -1),
                        (-3, -1),
                    ] {
                        let (x, y) = ((s.fx + dx) as f64 * TS + 16., (s.fy + dy) as f64 * TS + 16.);
                        if !path::blocked(&self.map, x, y) {
                            made.push(self.spawn(&prod, b.team, x, y, b.sector));
                            break;
                        }
                    }
                }
            } else {
                let tiles = path::free_tiles(&self.map, b.door.tx, b.door.ty, d.n);
                if !tiles.is_empty() {
                    let squad = if d.cls == "robot" {
                        self.nid += 1;
                        self.nid - 1
                    } else {
                        0
                    };
                    for k in 0..d.n {
                        let p = &tiles[k % tiles.len()];
                        let id = self.spawn(&prod, b.team, p.x, p.y, -1);
                        let i = self.ui(id).unwrap();
                        self.units[i].squad = squad;
                        made.push(id)
                    }
                    counts[b.team as usize] += d.n;
                    let p = b.rally.unwrap_or(Point {
                        x: b.door.tx as f64 * TS + 16.,
                        y: b.door.ty as f64 * TS + 112.,
                    });
                    self.move_units(&made, p.x, p.y, "move");
                }
            }
            if made.is_empty() {
                self.map.buildings[bi].progress = 1.;
                continue;
            }
            self.map.buildings[bi].progress = 0.;
            self.ev
                .push(json!({"k":"ready","team":b.team,"t":prod,"x":b.cx,"y":b.cy}));
            if self.is_ai(b.team) {
                self.map.buildings[bi].prod = self.pick_prod(bi)
            }
        }
    }
    pub fn ai_think(&mut self, team: i32) {
        let enemy = 1 - team;
        let mine: Vec<u32> = self
            .units
            .iter()
            .filter(|u| !u.dead && u.team == team && self.def(u).sp > 0.)
            .map(|u| u.id)
            .collect();
        let ef = self
            .map
            .buildings
            .iter()
            .find(|b| b.kind == "fort" && b.team == enemy && !b.dead)
            .cloned();
        let mf = self
            .map
            .buildings
            .iter()
            .find(|b| b.kind == "fort" && b.team == team && !b.dead)
            .cloned();
        if let Some(b) = mf {
            if let Some(v) = self
                .units
                .iter()
                .filter(|u| !u.dead && u.team == enemy && (u.x - b.cx).hypot(u.y - b.cy) < 280.)
                .min_by(|a, c| {
                    (a.x - b.cx)
                        .hypot(a.y - b.cy)
                        .total_cmp(&(c.x - b.cx).hypot(c.y - b.cy))
                })
                .cloned()
            {
                for id in &mine {
                    let i = self.ui(*id).unwrap();
                    let u = &self.units[i];
                    if self.def(u).dmg > 0.
                        && u.order.as_ref().is_none_or(|o| o.k == "amove")
                        && (u.x - b.cx).hypot(u.y - b.cy) < 520.
                    {
                        self.units[i].order = Some(Order {
                            k: "attack".into(),
                            target: Some(v.id),
                            ..Default::default()
                        });
                        self.units[i].path = None;
                    }
                }
            }
        }
        let open: Vec<_> = self
            .map
            .sectors
            .iter()
            .filter(|s| s.flag.is_some() && s.owner != team)
            .cloned()
            .collect();
        let mut assigned = BTreeMap::<usize, usize>::new();
        let mut claimed = vec![];
        for u in &self.units {
            if u.team == team {
                if let Some(o) = &u.order {
                    if let Some(s) = o.sec {
                        *assigned.entry(s).or_default() += 1
                    }
                    if o.k == "enter" {
                        if let Some(t) = o.target {
                            claimed.push(t)
                        }
                    }
                }
            }
        }
        let rush = mine.len() >= self.ai.ai_rush;
        let mut budget = if self.time < self.ai.ai_wait {
            0
        } else {
            self.ai.ai_batch
        };
        for id in &mine {
            let Some(i) = self.ui(*id) else { continue };
            let mut u = self.units[i].clone();
            if u.order.is_some() {
                continue;
            }
            let d = self.def(&u);
            if d.crane {
                if let Some(job) = self.repair_job(team, Some(u.pos())) {
                    u.order = Some(Order {
                        k: "fix".into(),
                        st: Some(job),
                        ..Default::default()
                    });
                    self.units[i] = u;
                }
                continue;
            }
            if !u.cargo.is_empty() {
                self.unload(&mut u);
                self.units[i] = u;
                continue;
            }
            if budget == 0 {
                break;
            }
            budget -= 1;
            if d.cls == "robot" {
                if let Some(v) = self
                    .units
                    .iter()
                    .filter(|v| {
                        !v.dead
                            && v.team < 0
                            && !claimed.contains(&v.id)
                            && u.pos().distance(&v.pos()) < 340.
                    })
                    .min_by(|a, b| {
                        u.pos()
                            .distance(&a.pos())
                            .total_cmp(&u.pos().distance(&b.pos()))
                    })
                    .cloned()
                {
                    claimed.push(v.id);
                    u.order = Some(Order {
                        k: "enter".into(),
                        target: Some(v.id),
                        ..Default::default()
                    });
                    self.units[i] = u;
                    continue;
                }
            }
            if let Some(b) = &ef {
                if open.is_empty() || (rush && self.rnd() < 0.4) {
                    if mine.len() < self.ai.ai_min {
                        break;
                    }
                    u.order = if d.cls == "robot" && self.idx >= 2 {
                        Some(Order {
                            k: "storm".into(),
                            building: Some(b.id),
                            ..Default::default()
                        })
                    } else if d.dmg > 0. {
                        Some(Order {
                            k: "hit".into(),
                            st: Some(Target {
                                kind: "b".into(),
                                id: b.id,
                                x: b.cx,
                                y: b.cy,
                            }),
                            ..Default::default()
                        })
                    } else {
                        None
                    };
                    self.units[i] = u;
                    continue;
                }
            }
            let score = |s: &Sector| {
                let p = s.flag.as_ref().unwrap();
                u.pos().distance(p) + 150. * assigned.get(&s.id).copied().unwrap_or(0) as f64
                    - if s.owner == enemy { 60. } else { 0. }
            };
            if let Some(s) = open.iter().min_by(|a, b| score(a).total_cmp(&score(b))) {
                let p = s.flag.as_ref().unwrap();
                if self.path_to(&mut u, p.x, p.y) {
                    u.order = Some(Order {
                        k: "amove".into(),
                        x: p.x,
                        y: p.y,
                        sec: Some(s.id),
                        ..Default::default()
                    });
                    *assigned.entry(s.id).or_default() += 1;
                } else if explosive(d.w.as_deref().unwrap_or("")) || u.gren > 0 {
                    let rock = self
                        .map
                        .tiles
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| **t == 6)
                        .map(|(j, _)| Target {
                            kind: "rock".into(),
                            id: j,
                            x: (j % self.map.w) as f64 * TS + 16.,
                            y: (j / self.map.w) as f64 * TS + 16.,
                        })
                        .min_by(|a, b| {
                            (u.x - a.x)
                                .hypot(u.y - a.y)
                                .total_cmp(&(u.x - b.x).hypot(u.y - b.y))
                        });
                    u.order = rock.map(|st| Order {
                        k: "hit".into(),
                        st: Some(st),
                        ..Default::default()
                    });
                }
            }
            self.units[i] = u;
        }
    }
    pub fn step(&mut self, dt: f64) {
        if self.over.is_some() {
            return;
        }
        let dt = dt.clamp(0., 0.1);
        self.time += dt;
        self.alert_t -= dt;
        self.production(dt);
        let n = self.units.len();
        for i in 0..n {
            if !self.units[i].dead {
                self.think_unit(i, dt)
            }
        }
        self.separate();
        self.capture();
        self.update_shots(dt);
        for b in &mut self.map.bridges {
            b.hit = (b.hit - dt).max(0.)
        }
        self.ai_t -= dt;
        if self.ai_t <= 0. {
            self.ai_t = self.ai.ai_int.max(0.1);
            for team in self.ai_teams.clone() {
                self.ai_think(team)
            }
        }
        self.units.retain(|u| !u.dead);
        for f in &mut self.fx {
            let t = num(f, "t", 0.) + dt;
            f["t"] = json!(t)
        }
        self.fx.retain(|f| num(f, "t", 0.) < num(f, "life", 0.));
        if self.over.is_none() {
            let a = self.force(0);
            let b = self.force(1);
            if a == 0 && b > 0 {
                self.end(1)
            } else if b == 0 && a > 0 {
                self.end(0)
            }
        }
    }
}
