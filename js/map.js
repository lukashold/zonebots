// Zonebots - map generation and pathfinding
(function () {
  const ZG = globalThis.ZG, T = ZG.T, TS = ZG.TILE;

  function noise(rnd, W, H, cell) {
    const gw = Math.ceil(W / cell) + 2, gh = Math.ceil(H / cell) + 2;
    const g = new Float32Array(gw * gh);
    for (let i = 0; i < g.length; i++) g[i] = rnd();
    return function (x, y) {
      const fx = x / cell, fy = y / cell, ix = fx | 0, iy = fy | 0;
      let tx = fx - ix, ty = fy - iy;
      tx = tx * tx * (3 - 2 * tx); ty = ty * ty * (3 - 2 * ty);
      const a = g[iy * gw + ix], b = g[iy * gw + ix + 1], c = g[(iy + 1) * gw + ix], d = g[(iy + 1) * gw + ix + 1];
      return a + (b - a) * tx + (c - a) * ty + (a - b - c + d) * tx * ty;
    };
  }

  ZG.genMap = function (lv) {
    const rnd = ZG.rng(lv.seed), cols = lv.cols, rows = lv.rows, sw = lv.sw, sh = lv.sh;
    const W = cols * sw, H = rows * sh, N = W * H, tiles = new Uint8Array(N);
    const n1 = noise(rnd, W, H, 4), n2 = noise(rnd, W, H, 5);
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const i = y * W + x;
      if (n1(x, y) > lv.rock) tiles[i] = T.ROCK;
      else if (n2(x, y) > lv.rough) tiles[i] = T.ROUGH;
    }
    const setT = (x, y, t) => { if (x >= 0 && y >= 0 && x < W && y < H) tiles[y * W + x] = t; };
    const wob = (t, len, k, A) => Math.round(A * Math.sin((t - len / 2) / len * Math.PI * 2 * k));
    const rw = lv.idx > 11 ? 3 : 2;
    if (lv.river & 1) {
      const xs = cols % 2 === 0 ? [W / 2] : [sw * Math.floor(cols / 2), sw * Math.ceil(cols / 2)];
      for (const bx of xs) for (let y = 0; y < H; y++) {
        const cx = bx + wob(y, H, 1.5, 2);
        for (let d = 0; d < rw; d++) setT(cx - 1 + d, y, T.WATER);
      }
    }
    if (lv.river & 2) {
      const ys = rows % 2 === 0 ? [H / 2] : [sh * Math.floor(rows / 2), sh * Math.ceil(rows / 2)];
      for (const by of ys) for (let x = 0; x < W; x++) {
        const cy = by + wob(x, W, 2, 2);
        for (let d = 0; d < rw; d++) setT(x, cy - 1 + d, T.WATER);
      }
    }
    // point symmetry keeps both sides fair
    for (let i = 0; i < N >> 1; i++) tiles[N - 1 - i] = tiles[i];

    const sectors = [], n = cols * rows;
    for (let r = 0; r < rows; r++) for (let c = 0; c < cols; c++)
      sectors.push({ id: r * cols + c, c: c, r: r, x0: c * sw, y0: r * sh, x1: (c + 1) * sw, y1: (r + 1) * sh, owner: -1, flag: null, b: null, fx: 0, fy: 0 });
    const hr = lv.idx % 3 === 0 ? rows - 1 : lv.idx % 3 === 1 ? Math.floor(rows / 2) : 0;
    const pid = hr * cols, eid = n - 1 - pid;
    for (const s of sectors) {
      const m = sectors[n - 1 - s.id];
      if (s.id > m.id) continue;
      s.fx = s.x0 + Math.floor(sw / 2) + Math.floor(rnd() * 3) - 1;
      s.fy = s.y0 + Math.floor(sh / 2) + 1 - Math.floor(rnd() * 2);
      if (m !== s) { m.fx = W - 1 - s.fx; m.fy = m.y0 + (s.fy - s.y0); }
    }
    for (const s of sectors)
      for (let y = s.fy - 5; y <= s.fy + 2; y++) for (let x = s.fx - 3; x <= s.fx + 4; x++) setT(x, y, T.GROUND);

    // roads: random spanning tree plus some extra links, mirrored
    const road = (x, y) => {
      if (x < 0 || y < 0 || x >= W || y >= H) return;
      const i = y * W + x; tiles[i] = (tiles[i] === T.WATER || tiles[i] === T.BRIDGE) ? T.BRIDGE : T.ROAD;
    };
    const carveH = (xa, xb, y) => { for (let x = Math.min(xa, xb); x <= Math.max(xa, xb) + 1; x++) { road(x, y); road(x, y + 1); } };
    const carveV = (x, ya, yb) => { for (let y = Math.min(ya, yb); y <= Math.max(ya, yb) + 1; y++) { road(x, y); road(x + 1, y); } };
    const done = new Set();
    const carveEdge = (ia, ib) => {
      if (ia > ib) { const t = ia; ia = ib; ib = t; }
      const key = ia * 1000 + ib; if (done.has(key)) return; done.add(key);
      const a = sectors[ia], b = sectors[ib];
      if (a.r === b.r) { carveH(a.fx, b.fx, a.fy + 1); carveV(b.fx, a.fy + 1, b.fy + 1); }
      else { const vx = b.fx + 3; carveH(a.fx, vx, a.fy + 1); carveV(vx, a.fy + 1, b.fy + 1); carveH(b.fx, vx, b.fy + 1); }
    };
    const edges = [];
    for (const s of sectors) {
      if (s.c < cols - 1) edges.push([s.id, s.id + 1]);
      if (s.r < rows - 1) edges.push([s.id, s.id + cols]);
    }
    for (let i = edges.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); const t = edges[i]; edges[i] = edges[j]; edges[j] = t; }
    const uf = sectors.map((s, i) => i), find = i => uf[i] === i ? i : (uf[i] = find(uf[i]));
    for (const e of edges) {
      const a = find(e[0]), b = find(e[1]), join = a !== b;
      if (join || rnd() < 0.45) {
        carveEdge(e[0], e[1]); carveEdge(n - 1 - e[1], n - 1 - e[0]);
        uf[a] = b; uf[find(n - 1 - e[0])] = find(n - 1 - e[1]);
      }
    }

    // buildings
    const buildings = [], spawns = [], crates = [];
    const addB = (s, kind, tech, team) => {
      const fort = kind === 'fort';
      const b = { id: buildings.length, kind: kind, tech: tech, team: team, sector: s.id,
        tx: fort ? s.fx - 2 : s.fx - 1, ty: fort ? s.fy - 4 : s.fy - 3, w: fort ? 4 : 3, h: fort ? 3 : 2,
        door: { tx: s.fx, ty: s.fy - 1 }, hp: fort ? 4000 : 0, maxhp: fort ? 4000 : 0, prod: null, progress: 0, dead: false, hit: 0 };
      b.cx = (b.tx + b.w / 2) * TS; b.cy = (b.ty + b.h / 2) * TS;
      buildings.push(b); s.b = b; return b;
    };
    const fortTech = Math.min(lv.maxTech, lv.fortTech);
    [[pid, 0], [eid, 1]].forEach(function (h) {
      const s = sectors[h[0]], team = h[1];
      s.owner = team; s.home = true;
      addB(s, 'fort', fortTech, team);
      spawns.push({ t: 'gatling', team: team, tx: s.fx - 3, ty: s.fy - 1, sector: s.id });
      spawns.push({ t: 'gatling', team: team, tx: s.fx + 3, ty: s.fy - 1, sector: s.id });
      if (lv.idx >= 8) spawns.push({ t: 'cannon', team: team, tx: s.fx + 3, ty: s.fy - 3, sector: s.id });
      const cnt = lv.start[team] + (team === 0 ? lv.bonus || 0 : 0);
      for (let k = 0; k < cnt; k++) spawns.push({ t: 'grunt', team: team, tx: s.fx - 2 + (k % 5), ty: s.fy + Math.floor(k / 5) });
      if (lv.idx >= 6) spawns.push({ t: 'jeep', team: team, tx: s.fx - 2, ty: s.fy + 2 });
      if (lv.idx >= 12) spawns.push({ t: 'light', team: team, tx: s.fx + 2, ty: s.fy + 2 });
    });
    const pairs = [];
    for (const s of sectors) {
      if (s.home) continue;
      s.flag = { x: (s.fx + 0.5) * TS, y: (s.fy + 0.5) * TS };
      if (s.id < n - 1 - s.id) pairs.push(s);
    }
    for (let i = pairs.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); const t = pairs[i]; pairs[i] = pairs[j]; pairs[j] = t; }
    const seq = ['robot', 'vehicle', 'none', 'robot', 'repair', 'vehicle', 'none', 'robot', 'vehicle', 'none', 'repair', 'robot', 'vehicle'];
    const techFor = s => {
      const close = cols > 1 ? 1 - Math.abs(s.c - (cols - 1) / 2) / ((cols - 1) / 2) : 1;
      return ZG.clamp(1 + Math.round((lv.maxTech - 1) * (0.25 + 0.75 * close) * (0.7 + rnd() * 0.5)), 1, lv.maxTech);
    };
    pairs.forEach(function (s, k) {
      const m = sectors[n - 1 - s.id], kind = seq[k % seq.length], tech = techFor(s);
      if (kind !== 'none') { addB(s, kind, kind === 'repair' ? 1 : tech, -1); addB(m, kind, kind === 'repair' ? 1 : tech, -1); }
      if (k < lv.guns) {
        const g = lv.maxTech >= 2 && rnd() < 0.4 ? 'cannon' : 'gatling';
        spawns.push({ t: g, team: -1, tx: s.fx + 3, ty: s.fy - 1, sector: s.id });
        spawns.push({ t: g, team: -1, tx: m.fx + 3, ty: m.fy - 1, sector: m.id });
      }
      if (k % 2 === 1 && crates.length < lv.crates * 2) {
        crates.push({ x: (s.fx - 2.5) * TS, y: (s.fy - 2.5) * TS, taken: false });
        crates.push({ x: (m.fx - 2.5) * TS, y: (m.fy - 2.5) * TS, taken: false });
      }
      if (pairs.length - 1 - k < lv.neutral) {
        const r = rnd(), v = lv.maxTech >= 3 && r < 0.2 ? 'medium' : lv.maxTech >= 2 && r < 0.55 ? 'light' : 'jeep';
        spawns.push({ t: v, team: -1, tx: s.fx - 2, ty: s.fy });
        spawns.push({ t: v, team: -1, tx: m.fx - 2, ty: m.fy });
      }
    });
    if (n % 2 === 1) { const s = sectors[(n - 1) / 2]; if (!s.home) addB(s, 'vehicle', lv.maxTech, -1); }

    const block = new Uint8Array(N);
    for (let i = 0; i < N; i++) if (tiles[i] === T.ROCK || tiles[i] === T.WATER) block[i] = 1;
    for (const b of buildings)
      for (let y = b.ty; y < b.ty + b.h; y++) for (let x = b.tx; x < b.tx + b.w; x++) block[y * W + x] = 1;

    return { W: W, H: H, tiles: tiles, block: block, sectors: sectors, buildings: buildings, spawns: spawns, crates: crates, pid: pid, eid: eid, sw: sw, sh: sh, cols: cols, rows: rows };
  };

  // ---------- A* on the tile grid ----------
  const COST = [1, 0.75, 9, 9, 1.7, 0.8];
  const DX = [1, -1, 0, 0, 1, 1, -1, -1], DY = [0, 0, 1, -1, 1, -1, 1, -1];
  let stamp = null, gs, par, cl, gen = 0;
  const hn = [], hf = [];
  function hpush(n, f) {
    let i = hn.length; hn.push(n); hf.push(f);
    while (i > 0) { const p = (i - 1) >> 1; if (hf[p] <= f) break; hn[i] = hn[p]; hf[i] = hf[p]; i = p; }
    hn[i] = n; hf[i] = f;
  }
  function hpop() {
    const n = hn[0], ln = hn.pop(), lf = hf.pop(), len = hn.length;
    if (len) {
      let i = 0;
      for (;;) {
        let c = 2 * i + 1; if (c >= len) break;
        if (c + 1 < len && hf[c + 1] < hf[c]) c++;
        if (hf[c] >= lf) break;
        hn[i] = hn[c]; hf[i] = hf[c]; i = c;
      }
      hn[i] = ln; hf[i] = lf;
    }
    return n;
  }

  ZG.nearFree = function (map, tx, ty) {
    const W = map.W, H = map.H;
    for (let r = 1; r < 10; r++)
      for (let dy = -r; dy <= r; dy++) for (let dx = -r; dx <= r; dx++) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) !== r) continue;
        const x = tx + dx, y = ty + dy;
        if (x >= 0 && y >= 0 && x < W && y < H && !map.block[y * W + x]) return [x, y];
      }
    return null;
  };

  // pixel coordinates in, list of pixel waypoints out (null if unreachable)
  ZG.findPath = function (map, sx, sy, ex, ey) {
    const W = map.W, H = map.H, blk = map.block, N = W * H;
    const stx = ZG.clamp(Math.floor(sx / TS), 0, W - 1), sty = ZG.clamp(Math.floor(sy / TS), 0, H - 1);
    let etx = ZG.clamp(Math.floor(ex / TS), 0, W - 1), ety = ZG.clamp(Math.floor(ey / TS), 0, H - 1), exact = true;
    if (blk[ety * W + etx]) { const f = ZG.nearFree(map, etx, ety); if (!f) return null; etx = f[0]; ety = f[1]; exact = false; }
    const last = exact ? { x: ex, y: ey } : { x: (etx + 0.5) * TS, y: (ety + 0.5) * TS };
    const s = sty * W + stx, e = ety * W + etx;
    if (s === e) return [last];
    if (!stamp || stamp.length < N) { stamp = new Uint32Array(N); gs = new Float32Array(N); par = new Int32Array(N); cl = new Uint32Array(N); }
    gen++; hn.length = 0; hf.length = 0;
    stamp[s] = gen; gs[s] = 0; par[s] = -1; hpush(s, 0);
    while (hn.length) {
      const nd = hpop();
      if (cl[nd] === gen) continue;
      cl[nd] = gen;
      if (nd === e) break;
      const x = nd % W, y = (nd / W) | 0, g = gs[nd];
      for (let d = 0; d < 8; d++) {
        const dx = DX[d], dy = DY[d], nx = x + dx, ny = y + dy;
        if (nx < 0 || ny < 0 || nx >= W || ny >= H) continue;
        const m = ny * W + nx;
        if (blk[m] || cl[m] === gen) continue;
        if (dx && dy && (blk[y * W + nx] || blk[ny * W + x])) continue;
        const ng = g + (dx && dy ? 1.414 : 1) * COST[map.tiles[m]];
        if (stamp[m] !== gen || ng < gs[m]) {
          stamp[m] = gen; gs[m] = ng; par[m] = nd;
          const hx = Math.abs(nx - etx), hy = Math.abs(ny - ety);
          hpush(m, ng + 0.75 * (Math.max(hx, hy) + 0.414 * Math.min(hx, hy)));
        }
      }
    }
    if (cl[e] !== gen) return null;
    const out = [last];
    for (let nd = par[e]; nd !== -1 && nd !== s; nd = par[nd]) out.push({ x: (nd % W + 0.5) * TS, y: (((nd / W) | 0) + 0.5) * TS });
    return out.reverse();
  };
})();
