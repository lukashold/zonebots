use crate::model::{Map, Point};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
pub const TS: f64 = 32.0;
pub fn blocked(m: &Map, x: f64, y: f64) -> bool {
    let tx = (x / TS).floor() as i32;
    let ty = (y / TS).floor() as i32;
    tx < 0
        || ty < 0
        || tx >= m.w as i32
        || ty >= m.h as i32
        || m.block[ty as usize * m.w + tx as usize] != 0
}
pub fn free_tiles(m: &Map, tx: i32, ty: i32, n: usize) -> Vec<Point> {
    let mut out = vec![];
    for r in 0i32..12 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let p = Point {
                    x: (tx + dx) as f64 * TS + 16.0,
                    y: (ty + dy) as f64 * TS + 16.0,
                };
                if !blocked(m, p.x, p.y) {
                    out.push(p);
                    if out.len() >= n {
                        return out;
                    }
                }
            }
        }
    }
    out
}
#[derive(Copy, Clone)]
struct Node {
    n: usize,
    f: f64,
}
impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, o: &Self) -> bool {
        self.n == o.n && self.f == o.f
    }
}
impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.total_cmp(&self.f).then_with(|| o.n.cmp(&self.n))
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
fn clear(m: &Map, a: &Point, b: &Point) -> bool {
    let n = (a.distance(b) / 6.0).ceil() as usize;
    for i in 1..=n {
        let x = a.x + (b.x - a.x) * i as f64 / n as f64;
        let y = a.y + (b.y - a.y) * i as f64 / n as f64;
        for (dx, dy) in [(0., 0.), (7., 7.), (-7., 7.), (7., -7.), (-7., -7.)] {
            if blocked(m, x + dx, y + dy) {
                return false;
            }
        }
    }
    true
}
pub fn find(m: &Map, start: Point, end: Point) -> Option<Vec<Point>> {
    if m.w == 0 || m.h == 0 {
        return None;
    }
    let (sx, sy) = (
        ((start.x / TS).floor() as i32).clamp(0, m.w as i32 - 1),
        ((start.y / TS).floor() as i32).clamp(0, m.h as i32 - 1),
    );
    let mut e = Point {
        x: end.x.clamp(0.0, m.w as f64 * TS - 0.001),
        y: end.y.clamp(0.0, m.h as f64 * TS - 0.001),
    };
    if blocked(m, e.x, e.y) {
        e = free_tiles(m, (e.x / TS) as i32, (e.y / TS) as i32, 1)
            .into_iter()
            .next()?
    }
    let ex = (e.x / TS) as usize;
    let ey = (e.y / TS) as usize;
    let s = sy as usize * m.w + sx as usize;
    let goal = ey * m.w + ex;
    if s == goal {
        return Some(vec![e]);
    }
    let len = m.w * m.h;
    let mut g = vec![f64::INFINITY; len];
    let mut parent = vec![usize::MAX; len];
    let mut closed = vec![false; len];
    let mut heap = BinaryHeap::new();
    g[s] = 0.;
    heap.push(Node { n: s, f: 0. });
    while let Some(Node { n, .. }) = heap.pop() {
        if closed[n] {
            continue;
        }
        closed[n] = true;
        if n == goal {
            break;
        }
        let x = n % m.w;
        let y = n / m.w;
        for (dx, dy) in [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= m.w as i32 || ny >= m.h as i32 {
                continue;
            }
            let k = ny as usize * m.w + nx as usize;
            if m.block[k] != 0 || closed[k] {
                continue;
            }
            if dx != 0
                && dy != 0
                && (m.block[y * m.w + nx as usize] != 0 || m.block[ny as usize * m.w + x] != 0)
            {
                continue;
            }
            let ng = g[n]
                + if dx != 0 && dy != 0 { 1.414 } else { 1.0 }
                    * [1., 0.75, 9., 9., 1.7, 0.8, 9.][m.tiles[k].min(6) as usize];
            if ng < g[k] {
                g[k] = ng;
                parent[k] = n;
                let hx = (nx - ex as i32).unsigned_abs() as f64;
                let hy = (ny - ey as i32).unsigned_abs() as f64;
                heap.push(Node {
                    n: k,
                    f: ng + 0.75 * (hx.max(hy) + 0.414 * hx.min(hy)),
                });
            }
        }
    }
    if !closed[goal] {
        return None;
    }
    let mut pts = vec![e];
    let mut k = parent[goal];
    while k != s && k != usize::MAX {
        pts.push(Point {
            x: (k % m.w) as f64 * TS + 16.,
            y: (k / m.w) as f64 * TS + 16.,
        });
        k = parent[k]
    }
    pts.reverse();
    let mut out = vec![];
    let mut anchor = start;
    let mut i = 0;
    while i < pts.len() {
        let mut j = i;
        while j + 1 < pts.len() && clear(m, &anchor, &pts[j + 1]) {
            j += 1
        }
        anchor = pts[j].clone();
        out.push(anchor.clone());
        i = j + 1
    }
    Some(out)
}
