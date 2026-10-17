// Zonebots - pixel-art renderer. Everything is drawn by code: there are no image files.
//
// The world is drawn into a low-resolution canvas where one "art pixel" covers ZG.ART world units
// (one 32-unit tile = 16 art pixels) and the result is scaled up without smoothing.
(function () {
  const ZG = globalThis.ZG, TS = ZG.TILE, T = ZG.T;
  const A = ZG.ART = 2, TP = TS / A;

  // ---------- palettes ----------
  const TEAMPAL = [
    { T: '#d23b2a', t: '#82180f', L: '#ff8a66' },
    { T: '#2f62d6', t: '#162f78', L: '#7aa8ff' },
    { T: '#9b978b', t: '#58554c', L: '#d2cdbf' }
  ];
  const BASE = { k: '#141414', K: '#3b3b3b', m: '#cdd1d5', M: '#8e939a', n: '#5b6068', v: '#1e3442', V: '#79e0ff',
    y: '#ffd23a', o: '#ff8a1c', r: '#c8261a', g: '#5a8c32', G: '#36561e', w: '#ffffff', c: '#66ffff', b: '#6b4a2a', B: '#3e2a17' };
  ZG.TEAMCOL = ['#d23b2a', '#2f62d6'];
  const palOf = team => Object.assign({}, BASE, TEAMPAL[team < 0 ? 2 : team]);

  function canvas(w, h) { const c = document.createElement('canvas'); c.width = Math.max(1, w); c.height = Math.max(1, h); return c; }
  function fromGrid(rows, p, w) {
    const h = rows.length; w = w || rows[0].length;
    const c = canvas(w, h), x = c.getContext('2d');
    for (let j = 0; j < h; j++) for (let i = 0; i < rows[j].length; i++) {
      const ch = rows[j][i];
      if (ch !== '.' && p[ch]) { x.fillStyle = p[ch]; x.fillRect(i, j, 1, 1); }
    }
    return c;
  }
  function mirror(src) {
    const c = canvas(src.width, src.height), x = c.getContext('2d');
    x.translate(src.width, 0); x.scale(-1, 1); x.drawImage(src, 0, 0);
    return c;
  }
  // nearest-neighbour rotation keeps the pixel look
  function rotate(src, ang) {
    const w = src.width, h = src.height, S = Math.ceil(Math.hypot(w, h)) + 2;
    const out = canvas(S, S), oc = out.getContext('2d');
    const sd = src.getContext('2d').getImageData(0, 0, w, h).data, od = oc.createImageData(S, S), o = od.data;
    const ca = Math.cos(-ang), sa = Math.sin(-ang);
    for (let y = 0; y < S; y++) for (let x = 0; x < S; x++) {
      const dx = x + 0.5 - S / 2, dy = y + 0.5 - S / 2;
      const sx = Math.floor(dx * ca - dy * sa + w / 2), sy = Math.floor(dx * sa + dy * ca + h / 2);
      if (sx < 0 || sy < 0 || sx >= w || sy >= h) continue;
      const si = (sy * w + sx) * 4, di = (y * S + x) * 4;
      o[di] = sd[si]; o[di + 1] = sd[si + 1]; o[di + 2] = sd[si + 2]; o[di + 3] = sd[si + 3];
    }
    oc.putImageData(od, 0, 0);
    return out;
  }
  const DIRS = 16, dirOf = a => ((Math.round(a / (Math.PI * 2) * DIRS) % DIRS) + DIRS) % DIRS;
  const cache = {};
  function cached(key, make) { return cache[key] || (cache[key] = make()); }

  // ---------- robots: hand-set pixel grids, 14x14 ----------
  const R_DOWN = [
    '.....kkkk.....', '....kmmmmk....', '...kmmmmmmk...', '...kVvvvvVk...', '...kmmmmmmk...', '....kkkkkk....',
    '...kLTTTTLk...', '..kTkTTTTkTk..', '..kTktTTtkTk..', '..kmk.tt.kmk..', '.....kttk.....',
    '....kKk.kKk...', '....kKk.kKk...', '....kk...kk...'];
  const R_UP = [
    '.....kkkk.....', '....kmmmmk....', '...kmmmmmmk...', '...kmMMMMmk...', '...kmmmmmmk...', '....kkkkkk....',
    '...kLTTTTLk...', '..kTkKnnKkTk..', '..kTkKnnKkTk..', '..kmk.tt.kmk..', '.....kttk.....',
    '....kKk.kKk...', '....kKk.kKk...', '....kk...kk...'];
  const R_SIDE = [
    '.....kkkk.....', '....kmmmmk....', '....kmmmmmk...', '....kmmmVVk...', '....kmmmmmk...', '.....kkkkk....',
    '.....kTTTk....', '....kTTTTk....', '....kTtTTk....', '....ktTTmk....', '.....kttk.....',
    '.....kKk......', '.....kKk......', '.....kkk......'];
  const LEGS = {
    down: [['....kKk.kKk...', '....kk..kKk...', '........kk....'], ['....kKk.kKk...', '....kKk..kk...', '....kk........']],
    side: [['....kKKk......', '...kKk.kKk....', '...kk...kk....'], ['.....kKk......', '.....kKk......', '.....kkk......']]
  };
  // weapon pixels per robot type and view: [x, y, colour key]
  const WEAPON = {
    grunt: { down: [[10, 10, 'K'], [10, 11, 'K']], side: [[10, 9, 'K'], [11, 9, 'K']], up: [] },
    psycho: { down: [[9, 10, 'K'], [10, 10, 'K'], [9, 11, 'K'], [10, 11, 'K'], [10, 12, 'K']], side: [[9, 8, 'K'], [10, 8, 'K'], [11, 8, 'K'], [10, 9, 'K'], [11, 9, 'K'], [12, 9, 'K']], up: [] },
    tough: { down: [[10, 5, 'k'], [10, 6, 'G'], [10, 7, 'G'], [11, 6, 'G'], [11, 7, 'G'], [10, 8, 'k'], [11, 8, 'k']], side: [[4, 6, 'G'], [5, 6, 'G'], [6, 6, 'G'], [7, 6, 'G'], [8, 6, 'G'], [9, 6, 'G'], [10, 6, 'G'], [11, 6, 'k'], [4, 5, 'G'], [5, 5, 'G'], [6, 5, 'G'], [7, 5, 'G'], [8, 5, 'G'], [9, 5, 'G'], [10, 5, 'G'], [11, 5, 'k']], up: [[10, 5, 'G'], [10, 6, 'G'], [11, 5, 'G'], [11, 6, 'G'], [10, 7, 'k'], [11, 7, 'k']] },
    sniper: { down: [[10, 9, 'K'], [10, 10, 'K'], [10, 11, 'K'], [10, 12, 'K'], [10, 13, 'K']], side: [[9, 9, 'K'], [10, 9, 'K'], [11, 9, 'K'], [12, 9, 'K'], [13, 9, 'K'], [10, 8, 'K']], up: [] },
    pyro: { down: [[10, 10, 'K'], [10, 11, 'o']], side: [[10, 9, 'K'], [11, 9, 'K'], [12, 9, 'o'], [3, 7, 'o'], [3, 8, 'o'], [3, 9, 'r']], up: [[5, 7, 'o'], [6, 7, 'o'], [7, 7, 'o'], [8, 7, 'o'], [5, 8, 'r'], [6, 8, 'o'], [7, 8, 'o'], [8, 8, 'r']] },
    laser: { down: [[10, 10, 'K'], [10, 11, 'c'], [10, 12, 'w']], side: [[10, 9, 'K'], [11, 9, 'c'], [12, 9, 'w']], up: [] }
  };
  function robotSprite(t, team, view, frame) {
    return cached('r' + t + team + view + frame, function () {
      const p = palOf(team);
      let rows = (view === 'up' ? R_UP : view === 'down' ? R_DOWN : R_SIDE).slice();
      if (frame) { const lg = LEGS[view === 'up' ? 'down' : view === 'down' ? 'down' : 'side'][frame - 1]; rows.splice(11, 3, lg[0], lg[1], lg[2]); }
      const c = fromGrid(rows, p, 14), x = c.getContext('2d');
      const wp = WEAPON[t] ? WEAPON[t][view === 'left' || view === 'right' ? 'side' : view] : [];
      for (const q of wp) { x.fillStyle = p[q[2]]; x.fillRect(q[0], q[1], 1, 1); }
      return view === 'left' ? mirror(c) : c;
    });
  }

  // ---------- vehicles: drawn east-facing with rectangles, then rotated ----------
  function px(x, c, col, X, Y, W, H) { x.fillStyle = c[col] || col; x.fillRect(X, Y, W, H); }
  function boxo(x, p, col, X, Y, W, H) { px(x, p, 'k', X, Y, W, H); px(x, p, col, X + 1, Y + 1, W - 2, H - 2); }
  function tracks(x, p, W, H, th) {
    boxo(x, p, 'K', 0, 0, W, th); boxo(x, p, 'K', 0, H - th, W, th);
    for (let i = 1; i < W - 1; i += 2) { px(x, p, 'k', i, 1, 1, th - 2); px(x, p, 'k', i, H - th + 1, 1, th - 2); }
  }
  const BODY = {
    jeep: [16, 11, function (x, p) {
      for (const q of [[2, 0], [10, 0], [2, 8], [10, 8]]) boxo(x, p, 'K', q[0], q[1], 4, 3);
      boxo(x, p, 'T', 1, 2, 14, 7); px(x, p, 'L', 11, 3, 3, 5); px(x, p, 'v', 9, 3, 2, 5); px(x, p, 'V', 9, 3, 1, 1);
      px(x, p, 'K', 3, 3, 2, 2); px(x, p, 'K', 3, 6, 2, 2); px(x, p, 't', 1, 2, 1, 7);
    }],
    light: [18, 13, function (x, p) {
      tracks(x, p, 18, 13, 3); boxo(x, p, 'T', 1, 2, 16, 9); px(x, p, 'L', 13, 3, 3, 7); px(x, p, 't', 2, 3, 2, 7);
    }],
    medium: [20, 15, function (x, p) {
      tracks(x, p, 20, 15, 4); boxo(x, p, 'T', 1, 3, 18, 9); px(x, p, 'L', 15, 4, 3, 7); px(x, p, 't', 2, 4, 3, 7); px(x, p, 'n', 6, 4, 1, 7);
    }],
    heavy: [22, 17, function (x, p) {
      tracks(x, p, 22, 17, 4); boxo(x, p, 'T', 1, 3, 20, 11); px(x, p, 'L', 17, 4, 3, 9); px(x, p, 't', 2, 4, 3, 9); px(x, p, 'n', 6, 4, 1, 9); px(x, p, 'n', 15, 4, 1, 9);
    }],
    apc: [20, 13, function (x, p) {
      for (const q of [[2, 0], [8, 0], [14, 0], [2, 10], [8, 10], [14, 10]]) boxo(x, p, 'K', q[0], q[1], 4, 3);
      boxo(x, p, 'T', 1, 2, 18, 9); px(x, p, 'L', 15, 3, 3, 7); px(x, p, 'v', 16, 4, 1, 5);
      boxo(x, p, 'n', 4, 4, 8, 5); px(x, p, 'M', 5, 5, 6, 3); px(x, p, 't', 2, 3, 1, 7);
    }],
    crane: [18, 13, function (x, p) {
      tracks(x, p, 18, 13, 3); boxo(x, p, 'T', 1, 2, 16, 9); boxo(x, p, 'n', 11, 3, 5, 7); px(x, p, 'v', 14, 4, 1, 5);
      px(x, p, 'k', 1, 5, 12, 3); px(x, p, 'y', 2, 6, 10, 1); for (let i = 3; i < 12; i += 3) px(x, p, 'k', i, 6, 1, 1);
      px(x, p, 'K', 0, 4, 2, 5);
    }],
    launcher: [20, 13, function (x, p) {
      for (const q of [[2, 0], [13, 0], [2, 10], [13, 10]]) boxo(x, p, 'K', q[0], q[1], 4, 3);
      boxo(x, p, 'T', 1, 2, 18, 9); boxo(x, p, 'n', 13, 3, 5, 7); px(x, p, 'v', 16, 4, 1, 5); px(x, p, 't', 2, 3, 1, 7);
    }]
  };
  const TURRET = {
    jeep: [12, 5, function (x, p) { px(x, p, 'k', 2, 1, 10, 3); px(x, p, 'K', 3, 2, 8, 1); boxo(x, p, 'K', 1, 0, 4, 5); }],
    light: [16, 9, function (x, p) { px(x, p, 'k', 6, 3, 10, 3); px(x, p, 'K', 7, 4, 8, 1); boxo(x, p, 'T', 1, 1, 8, 7); px(x, p, 'L', 2, 2, 3, 2); }],
    medium: [19, 11, function (x, p) { px(x, p, 'k', 7, 4, 12, 3); px(x, p, 'K', 8, 5, 10, 1); boxo(x, p, 'T', 1, 1, 10, 9); px(x, p, 'L', 2, 2, 4, 2); px(x, p, 't', 2, 7, 8, 2); }],
    heavy: [22, 13, function (x, p) {
      px(x, p, 'k', 9, 2, 13, 3); px(x, p, 'k', 9, 8, 13, 3); px(x, p, 'K', 10, 3, 11, 1); px(x, p, 'K', 10, 9, 11, 1);
      boxo(x, p, 'T', 0, 1, 12, 11); px(x, p, 'L', 1, 2, 5, 2); px(x, p, 't', 1, 9, 10, 2);
    }],
    launcher: [12, 9, function (x, p) { boxo(x, p, 'n', 0, 0, 10, 9); px(x, p, 'w', 1, 1, 9, 2); px(x, p, 'w', 1, 6, 9, 2); px(x, p, 'r', 9, 1, 2, 2); px(x, p, 'r', 9, 6, 2, 2); }],
    gatling: [18, 9, function (x, p) { for (const yy of [1, 4, 7]) px(x, p, 'k', 6, yy, 12, 2); px(x, p, 'K', 7, 2, 10, 1); boxo(x, p, 'n', 2, 1, 7, 7); }],
    cannon: [20, 9, function (x, p) { px(x, p, 'k', 6, 2, 14, 5); px(x, p, 'K', 7, 3, 12, 3); boxo(x, p, 'n', 1, 0, 8, 9); }],
    howitzer: [24, 11, function (x, p) { px(x, p, 'k', 6, 2, 18, 7); px(x, p, 'M', 7, 3, 16, 5); px(x, p, 'k', 21, 2, 2, 7); boxo(x, p, 'n', 0, 0, 10, 11); }],
    mturret: [14, 11, function (x, p) { boxo(x, p, 'n', 0, 0, 12, 11); px(x, p, 'w', 1, 1, 11, 3); px(x, p, 'w', 1, 7, 11, 3); px(x, p, 'r', 11, 1, 2, 3); px(x, p, 'r', 11, 7, 2, 3); }]
  };
  // pivot of each turret: where it sits on the hull (x offset from vehicle centre) and its own rotation point
  const PIVOT = { jeep: [-3, 2], light: [-1, 4], medium: [-1, 5], heavy: [-2, 6], launcher: [-3, 4] };
  function partSprite(table, t, team, dir) {
    return cached(table + t + team + dir, function () {
      const def = (table === 'b' ? BODY : TURRET)[t], c = canvas(def[0], def[1]);
      def[2](c.getContext('2d'), palOf(team));
      return rotate(c, dir / DIRS * Math.PI * 2);
    });
  }
  function turretSprite(t, team, dir) {
    // rotate around the pivot by padding the turret so the pivot is the centre
    return cached('tp' + t + team + dir, function () {
      const def = TURRET[t], pv = PIVOT[t] ? PIVOT[t][1] : def[1] / 2 | 0, w = Math.max(def[0] - pv, pv) * 2;
      const c = canvas(w, def[1]); def[2]((function () { const x = c.getContext('2d'); x.translate(w / 2 - pv, 0); return x; })(), palOf(team));
      return rotate(c, dir / DIRS * Math.PI * 2);
    });
  }
  function gunBase(team) {
    return cached('gb' + team, function () {
      const p = palOf(team), c = canvas(16, 16), x = c.getContext('2d');
      px(x, p, 'k', 3, 0, 10, 16); px(x, p, 'k', 0, 3, 16, 10); px(x, p, 'k', 1, 1, 14, 14);
      px(x, p, 'n', 4, 1, 8, 14); px(x, p, 'n', 1, 4, 14, 8); px(x, p, 'n', 2, 2, 12, 12);
      px(x, p, 'M', 4, 2, 8, 2); px(x, p, 'M', 2, 4, 2, 8);
      px(x, p, 'T', 5, 5, 6, 6); px(x, p, 't', 5, 10, 6, 1);
      return c;
    });
  }

  // ---------- small sprites ----------
  function flagSprite(team, f) {
    return cached('f' + team + f, function () {
      const p = palOf(team), c = canvas(10, 16), x = c.getContext('2d');
      px(x, p, 'k', 0, 0, 2, 16); px(x, p, 'm', 0, 0, 1, 15);
      const wave = [[0, 0, 1, 1, 0, 0, 1], [0, 1, 1, 0, 0, 1, 1], [1, 1, 0, 0, 1, 1, 0]][f];
      for (let i = 0; i < 7; i++) { const o = wave[i]; px(x, p, 'k', 2 + i, 1 + o, 1, 6); px(x, p, i < 2 ? 'T' : 'T', 2 + i, 2 + o, 1, 4); px(x, p, 'L', 2 + i, 2 + o, 1, 1); px(x, p, 't', 2 + i, 5 + o, 1, 1); }
      return c;
    });
  }
  const crateSprite = () => cached('crate', function () {
    const c = canvas(10, 9), x = c.getContext('2d'), p = BASE;
    px(x, p, 'k', 0, 0, 10, 9); px(x, p, 'g', 1, 1, 8, 7); px(x, p, 'G', 1, 4, 8, 1); px(x, p, 'G', 4, 1, 1, 7);
    px(x, p, 'y', 6, 2, 2, 2); px(x, p, 'w', 2, 2, 1, 1);
    return c;
  });
  function boulderPixels(img, ox, oy, W, seed) {
    // a lumpy rock pile sitting on the tile
    for (let y = 2; y < 15; y++) for (let x = 1; x < 15; x++) {
      const dx = (x - 7.5) / 6.5, dy = (y - 9) / 6.5, n = hash(ox + x, oy + y, seed) * 0.25;
      const d = dx * dx + dy * dy * 1.2 + n;
      if (d > 1) continue;
      const shade = (dx + dy) < -0.4 ? 0 : (dx + dy) < 0.5 ? 1 : 2;
      img.put(ox + x, oy + y, d > 0.82 ? img.c.k : shade === 0 ? img.c.rockHi : shade === 1 ? img.c.rock : img.c.rockLo);
    }
  }

  // ---------- terrain into an ImageData ----------
  function hash(x, y, s) {
    let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177); h ^= h >>> 16;
    return (h >>> 0) / 4294967296;
  }
  const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5].map(v => v / 16);
  function vnoise(x, y, s) {
    const ix = Math.floor(x), iy = Math.floor(y), fx = x - ix, fy = y - iy, sx = fx * fx * (3 - 2 * fx), sy = fy * fy * (3 - 2 * fy);
    const a = hash(ix, iy, s), b = hash(ix + 1, iy, s), c = hash(ix, iy + 1, s), d = hash(ix + 1, iy + 1, s);
    return a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy;
  }
  function rgb(hex) { const n = parseInt(hex.slice(1), 16); return [n >> 16 & 255, n >> 8 & 255, n & 255]; }
  function shadeC(c, f) { return [Math.min(255, Math.round(c[0] * f)), Math.min(255, Math.round(c[1] * f)), Math.min(255, Math.round(c[2] * f))]; }
  function themeColors(th) {
    const o = {};
    for (const k in th) if (typeof th[k] === 'string') o[k] = rgb(th[k]);
    o.g = th.g.map(rgb); o.k = [20, 20, 20];
    o.rockFace = shadeC(o.rockLo, 0.8); o.waterDeep = shadeC(o.water, 0.8); o.plank = [139, 104, 64]; o.plankLo = [96, 70, 40]; o.rail = [60, 44, 26];
    return o;
  }

  ZG.renderTerrain = function (map, th, seed) {
    const W = map.W * TP, H = map.H * TP, cv = canvas(W, H), cx = cv.getContext('2d');
    const id = cx.createImageData(W, H), d = id.data, C = themeColors(th);
    const img = { c: C, put(x, y, c) { const i = (y * W + x) * 4; d[i] = c[0]; d[i + 1] = c[1]; d[i + 2] = c[2]; d[i + 3] = 255; } };
    for (let ty = 0; ty < map.H; ty++) for (let tx = 0; tx < map.W; tx++) drawTile(map, img, C, tx, ty, seed, th);
    cx.putImageData(id, 0, 0);
    const R = { canvas: cv, C: C, seed: seed, th: th };
    // redraw single tiles after bridges or boulders change
    R.redraw = function (list) {
      const done = {};
      for (const j of list) {
        const x0 = j % map.W, y0 = (j / map.W) | 0;
        for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
          const x = x0 + dx, y = y0 + dy, k = y * map.W + x;
          if (x < 0 || y < 0 || x >= map.W || y >= map.H || done[k]) continue;
          done[k] = 1;
          const tid = cx.createImageData(TP, TP), td = tid.data;
          const timg = { c: C, put(px2, py2, c) { const lx = px2 - x * TP, ly = py2 - y * TP; if (lx < 0 || ly < 0 || lx >= TP || ly >= TP) return; const i = (ly * TP + lx) * 4; td[i] = c[0]; td[i + 1] = c[1]; td[i + 2] = c[2]; td[i + 3] = 255; } };
          drawTile(map, timg, C, x, y, seed, th);
          cx.putImageData(tid, x * TP, y * TP);
        }
      }
    };
    return R;
  };

  function drawTile(map, img, C, tx, ty, seed, th) {
    const W = map.W, H = map.H, t = map.tiles[ty * W + tx];
    const at = (x, y) => x < 0 || y < 0 || x >= W || y >= H ? -1 : map.tiles[y * W + x];
    const rock = v => v === T.ROCK, wet = v => v === T.WATER || v === T.BRIDGE, path = v => v === T.ROAD || v === T.BRIDGE || v === T.BOULDER;
    const ox = tx * TP, oy = ty * TP, lava = th.key === 'volcano';
    for (let y = 0; y < TP; y++) for (let x = 0; x < TP; x++) {
      const gx = ox + x, gy = oy + y, h = hash(gx, gy, seed);
      // smooth patches with ordered dithering at their edges, sparse grit on top
      const v = vnoise(gx / 14, gy / 14, seed + 7) * (C.g.length - 0.01) + (BAYER[(gy & 3) * 4 + (gx & 3)] - 0.5) * 0.35;
      let c = C.g[ZG.clamp(Math.floor(v), 0, C.g.length - 1)];
      if (h < 0.025) c = C.speck; else if (h > 0.992) c = shadeC(c, 1.12);
      if (t === T.ROUGH) {
        const cell = vnoise(gx / 2.5, gy / 2.5, seed + 3);
        c = cell < 0.45 ? C.rough : cell < 0.6 ? C.roughHi : c;
        if (cell > 0.92) c = shadeC(C.roughHi, 0.75);
      } else if (path(t)) {
        if (t !== T.BRIDGE) {
          c = h < 0.12 ? shadeC(C.road, 0.9) : h > 0.94 ? shadeC(C.road, 1.08) : C.road;
          const edge = (x === 0 && !path(at(tx - 1, ty))) || (x === TP - 1 && !path(at(tx + 1, ty))) || (y === 0 && !path(at(tx, ty - 1))) || (y === TP - 1 && !path(at(tx, ty + 1)));
          if (edge) c = C.roadEdge;
          if (th.key === 'city' && !edge && ((path(at(tx - 1, ty)) && path(at(tx + 1, ty)) && y === 8 && (gx & 7) < 4))) c = [214, 200, 115];
        }
      } else if (rock(t)) {
        // plateau with a cliff face where the rock ends to the south
        const faceH = 6, face = !rock(at(tx, ty + 1)) && y >= TP - faceH;
        if (face) {
          const fy = y - (TP - faceH);
          c = fy === 0 ? C.rockLo : ((gx + (fy >> 1)) % 4 === 0 ? shadeC(C.rockFace, 0.75) : C.rockFace);
          if (fy === faceH - 1) c = shadeC(C.rockFace, 0.55);
        } else {
          const n = vnoise(gx / 5, gy / 5, seed + 11) + (BAYER[(gy & 3) * 4 + (gx & 3)] - 0.5) * 0.3;
          c = n < 0.3 ? C.rockHi : n < 0.8 ? C.rock : C.rockLo;
          if (y === 0 && !rock(at(tx, ty - 1))) c = shadeC(C.rockHi, 1.15);
          if (x === 0 && !rock(at(tx - 1, ty))) c = C.rockHi;
          if (x === TP - 1 && !rock(at(tx + 1, ty))) c = C.rockLo;
          if (h > 0.985) c = shadeC(C.rockLo, 0.7);
        }
      } else if (wet(t)) {
        const sh = (y < 2 && !wet(at(tx, ty - 1)) && at(tx, ty - 1) !== -1) || (y > TP - 3 && !wet(at(tx, ty + 1)) && at(tx, ty + 1) !== -1) ||
          (x < 2 && !wet(at(tx - 1, ty)) && at(tx - 1, ty) !== -1) || (x > TP - 3 && !wet(at(tx + 1, ty)) && at(tx + 1, ty) !== -1);
        const wv = hash(gx >> 2, gy, seed + 5);
        c = sh ? C.shore : wv < 0.12 ? C.waterHi : wv < 0.5 ? C.water : C.waterDeep;
        if (lava && !sh && hash(gx >> 1, gy >> 1, seed + 9) > 0.8) c = shadeC(C.water, 0.55);
        if (t === T.BRIDGE) {
          const horiz = path(at(tx - 1, ty)) || path(at(tx + 1, ty)) || at(tx - 1, ty) === T.BRIDGE;
          const along = horiz ? x : y, across = horiz ? y : x;
          c = (along % 4 === 3) ? C.plankLo : C.plank;
          if (h < 0.1) c = shadeC(C.plank, 0.85);
          const railA = horiz ? (y < 2 && wet(at(tx, ty - 1)) && at(tx, ty - 1) !== T.BRIDGE) : (x < 2 && wet(at(tx - 1, ty)) && at(tx - 1, ty) !== T.BRIDGE);
          const railB = horiz ? (y > TP - 3 && wet(at(tx, ty + 1)) && at(tx, ty + 1) !== T.BRIDGE) : (x > TP - 3 && wet(at(tx + 1, ty)) && at(tx + 1, ty) !== T.BRIDGE);
          if (railA || railB) c = (along % 6 === 0) ? C.k : C.rail;
          void across;
        }
      }
      img.put(gx, gy, c);
    }
    if (t === T.BOULDER) boulderPixels(img, ox, oy, W, seed);
    // soft shadow cast by a cliff onto the tile below
    if (!rock(t) && rock(at(tx, ty - 1))) for (let x = 0; x < TP; x++) for (let y = 0; y < 2; y++) img.put(ox + x, oy + y, shadeC(C.g[0], 0.6 + 0.15 * y));
  }

  ZG.renderMini = function (map, th) {
    const cv = canvas(map.W, map.H), c = cv.getContext('2d'), cols = [th.g[0], th.road, th.rock, th.water, th.rough, '#8b6b45', th.rockHi];
    for (let y = 0; y < map.H; y++) for (let x = 0; x < map.W; x++) { c.fillStyle = cols[map.tiles[y * map.W + x]]; c.fillRect(x, y, 1, 1); }
    return cv;
  };
  ZG.redrawMini = function (R, map, th, list) {
    const c = R.mini.getContext('2d'), cols = [th.g[0], th.road, th.rock, th.water, th.rough, '#8b6b45', th.rockHi];
    for (const j of list) { c.fillStyle = cols[map.tiles[j]]; c.fillRect(j % map.W, (j / map.W) | 0, 1, 1); }
  };

  // ---------- buildings ----------
  const LIFT = 10; // buildings rise this many art pixels above their footprint
  function buildingSprite(b) {
    const key = 'B' + b.kind + b.team + b.tech + (b.dead ? 'x' : '') + (b.taken != null ? 't' : '');
    return cached(key, function () {
      const p = palOf(b.team), W = b.w * TP, H = b.h * TP + LIFT, c = canvas(W, H), x = c.getContext('2d');
      x.fillStyle = 'rgba(0,0,0,0.35)'; x.fillRect(3, LIFT + 3, W - 2, H - LIFT - 1);
      if (b.dead) {
        for (let i = 0; i < 60; i++) {
          const rx = (hash(i, b.id, 3) * (W - 8)) | 0, ry = LIFT + ((hash(i, b.id, 5) * (H - LIFT - 6)) | 0), s = 2 + ((hash(i, b.id, 7) * 5) | 0);
          px(x, p, i % 3 ? 'K' : 'n', rx, ry, s, s - 1); px(x, p, 'k', rx, ry + s - 1, s, 1);
        }
        px(x, p, 'k', W / 2 - 6, H - 10, 12, 6);
        return c;
      }
      if (b.kind === 'fort') {
        boxo(x, p, 'n', 2, LIFT + 4, W - 4, H - LIFT - 6);
        // wall top with battlements
        boxo(x, p, 'M', 2, 4, W - 4, LIFT + 6);
        for (let i = 4; i < W - 6; i += 6) px(x, p, 'k', i, 4, 3, 2);
        // keep
        boxo(x, p, 'm', 14, 8, W - 28, H - 26); px(x, p, 'T', 16, 10, W - 32, 8); px(x, p, 'L', 16, 10, W - 32, 1); px(x, p, 't', 16, 17, W - 32, 1);
        for (let i = 18; i < W - 18; i += 4) px(x, p, 't', i, 11, 1, 6);
        // towers
        for (const q of [[0, 0], [W - 14, 0], [0, H - 18], [W - 14, H - 18]]) {
          boxo(x, p, 'n', q[0], q[1] + 2, 14, 16); boxo(x, p, 'M', q[0], q[1], 14, 10); px(x, p, 'T', q[0] + 4, q[1] + 3, 6, 4); px(x, p, 'L', q[0] + 4, q[1] + 3, 6, 1);
        }
        // front wall and gate
        for (let i = 16; i < W - 16; i += 4) px(x, p, 'K', i, H - 14, 1, 10);
        boxo(x, p, 'k', W / 2 - 9, H - 16, 18, 16); px(x, p, 'K', W / 2 - 7, H - 14, 14, 14);
        for (let i = 0; i < 14; i += 3) px(x, p, 'y', W / 2 - 7 + i, H - 15, 2, 1);
        return c;
      }
      // factories and workshop: roof, front face, door
      const face = 12;
      boxo(x, p, 'n', 0, H - face - 2, W, face + 2);
      for (let i = 3; i < W - 2; i += 5) px(x, p, 'K', i, H - face, 1, face - 2);
      boxo(x, p, 'M', 0, 2, W, H - face - 2);
      px(x, p, 'm', 1, 3, W - 2, 2);
      for (let i = 6; i < W - 4; i += 8) px(x, p, 'n', i, 6, 1, H - face - 9);
      px(x, p, 'T', 1, H - face - 6, W - 2, 3); px(x, p, 'L', 1, H - face - 6, W - 2, 1);
      if (b.kind === 'robot') {
        boxo(x, p, 'k', W / 2 - 6, H - 11, 12, 11); px(x, p, 'K', W / 2 - 4, H - 9, 8, 9);
        boxo(x, p, 'K', 5, 7, 9, 9); px(x, p, 'n', 7, 9, 5, 5); px(x, p, 'k', 9, 9, 1, 5); px(x, p, 'k', 7, 11, 5, 1);
        px(x, p, 'k', W - 8, 0, 1, 8); px(x, p, 'r', W - 9, 0, 3, 2);
      } else if (b.kind === 'vehicle') {
        boxo(x, p, 'k', W / 2 - 10, H - 12, 20, 12);
        for (let i = 0; i < 4; i++) px(x, p, 'K', W / 2 - 9, H - 11 + i * 3, 18, 2);
        boxo(x, p, 'K', 4, 0, 6, 12); px(x, p, 'k', 5, 0, 4, 2);
      } else {
        px(x, p, 'w', W / 2 - 7, 6, 14, 12); px(x, p, 'k', W / 2 - 7, 6, 14, 1); px(x, p, 'g', W / 2 - 2, 7, 4, 10); px(x, p, 'g', W / 2 - 6, 10, 12, 4);
        boxo(x, p, 'k', W - 16, H - 11, 12, 11); px(x, p, 'K', W - 14, H - 9, 8, 9);
      }
      if (b.kind !== 'repair') for (let i = 0; i < b.tech; i++) { px(x, p, 'k', W - 7 - i * 6, 7, 5, 5); px(x, p, 'y', W - 6 - i * 6, 8, 3, 3); }
      return c;
    });
  }

  ZG.buildingIcon = b => buildingSprite(b);

  // ---------- drawing helpers in art pixels ----------
  function line(c, x0, y0, x1, y1, col) {
    x0 = Math.round(x0); y0 = Math.round(y0); x1 = Math.round(x1); y1 = Math.round(y1);
    const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1;
    let err = dx + dy, n = 0;
    c.fillStyle = col;
    for (;;) {
      c.fillRect(x0, y0, 1, 1);
      if ((x0 === x1 && y0 === y1) || n++ > 400) break;
      const e2 = 2 * err;
      if (e2 >= dy) { err += dy; x0 += sx; }
      if (e2 <= dx) { err += dx; y0 += sy; }
    }
  }
  function disc(c, cx, cy, r, col) {
    c.fillStyle = col; cx = Math.round(cx); cy = Math.round(cy);
    for (let y = -r; y <= r; y++) { const w = Math.round(Math.sqrt(r * r - y * y)); c.fillRect(cx - w, cy + y, 2 * w + 1, 1); }
  }
  function bar(c, x, y, w, f) {
    x = Math.round(x); y = Math.round(y);
    c.fillStyle = '#000'; c.fillRect(x - 1, y - 1, w + 2, 3);
    c.fillStyle = f > 0.5 ? '#3ed43e' : f > 0.25 ? '#f0c419' : '#ee3a24'; c.fillRect(x, y, Math.max(1, Math.round(w * f)), 1);
  }
  function brackets(c, x, y, w, h, col) {
    x = Math.round(x); y = Math.round(y); c.fillStyle = col;
    for (const q of [[x, y, 1, 1], [x + w - 1, y, -1, 1], [x, y + h - 1, 1, -1], [x + w - 1, y + h - 1, -1, -1]]) {
      c.fillRect(q[0], q[1], 1, 1); c.fillRect(q[0] + q[2], q[1], 1, 1); c.fillRect(q[0] + q[2] * 2, q[1], 1, 1);
      c.fillRect(q[0], q[1] + q[3], 1, 1); c.fillRect(q[0], q[1] + q[3] * 2, 1, 1);
    }
  }

  function robotView(a) {
    const d = ((Math.round(a / (Math.PI / 2)) % 4) + 4) % 4;
    return ['right', 'down', 'left', 'up'][d];
  }

  // draw a unit at art-pixel position (ax, ay)
  function drawUnitAt(c, u, ax, ay, time) {
    const d = u.def, team = u.team;
    if (d.cls === 'robot') {
      const view = robotView(u.ang), frame = u.moving ? 1 + (Math.floor(u.walk / 2) & 1) : 0;
      c.fillStyle = 'rgba(0,0,0,0.35)'; c.fillRect(ax - 3, ay + 1, 7, 2);
      const s = robotSprite(u.t, team, view, frame);
      c.drawImage(s, ax - 7, ay - 12);
      if (u.hit > 0) { c.globalCompositeOperation = 'lighter'; c.drawImage(s, ax - 7, ay - 12); c.globalCompositeOperation = 'source-over'; }
      if (u.gren > 0) { c.fillStyle = '#26401c'; c.fillRect(ax + 2, ay - 4, 2, 2); }
      return;
    }
    if (d.cls === 'vehicle') {
      const dir = dirOf(u.ang), body = partSprite('b', u.t, team, dir);
      c.fillStyle = 'rgba(0,0,0,0.3)'; c.fillRect(ax - body.width / 2 + 3, ay - body.height / 2 + 4, body.width - 4, body.height - 4);
      c.drawImage(body, ax - body.width / 2 | 0, ay - body.height / 2 | 0);
      if (TURRET[u.t]) {
        const pv = PIVOT[u.t] || [0, 0], ca = Math.cos(u.ang), sa = Math.sin(u.ang);
        const tx = ax + ca * pv[0], ty = ay + sa * pv[0], tur = turretSprite(u.t, team, dirOf(u.tang));
        c.drawImage(tur, Math.round(tx - tur.width / 2), Math.round(ty - tur.height / 2));
      }
      if (u.working) { const f = (time * 8 | 0) & 1; c.fillStyle = f ? '#ffd23a' : '#fff'; c.fillRect(ax - 1, ay - 9, 2, 2); }
      if (u.hit > 0) { c.fillStyle = 'rgba(255,255,255,0.5)'; c.fillRect(ax - 6, ay - 5, 12, 10); }
      return;
    }
    const base = gunBase(team);
    c.drawImage(base, ax - 8, ay - 8);
    const tur = turretSprite(u.t, team, dirOf(u.tang));
    c.drawImage(tur, Math.round(ax - tur.width / 2), Math.round(ay - tur.height / 2));
    if (u.hit > 0) { c.fillStyle = 'rgba(255,255,255,0.5)'; c.fillRect(ax - 6, ay - 6, 12, 12); }
  }
  // product icons for the HUD (native pixel size, scaled by CSS)
  ZG.unitIcon = function (t, team) {
    return cached('icon' + t + team, function () {
      const c = canvas(28, 28), x = c.getContext('2d'), d = ZG.UNITS[t];
      drawUnitAt(x, { t: t, def: d, team: team, ang: d.cls === 'robot' ? Math.PI / 2 : -Math.PI / 4, tang: -Math.PI / 4, moving: false, walk: 0, hit: 0 }, 14, d.cls === 'robot' ? 20 : 14, 0);
      return c;
    });
  };

  // ---------- main draw ----------
  // c: low-res context already translated so that art pixel (0,0) is the map's top-left corner.
  // view: visible area in world units. ui: { selB, drag, marks, hover }
  ZG.draw = function (c, G, R, view, ui, time) {
    const map = G.map, vx0 = view.x / A - 24, vy0 = view.y / A - 24, vx1 = (view.x + view.w) / A + 24, vy1 = (view.y + view.h) / A + 40;
    c.drawImage(R.terrain.canvas, 0, 0);
    // shimmering water
    const W = map.W;
    for (let ty = Math.max(0, vy0 / TP | 0); ty < Math.min(map.H, vy1 / TP + 1); ty++)
      for (let tx = Math.max(0, vx0 / TP | 0); tx < Math.min(W, vx1 / TP + 1); tx++) {
        if (map.tiles[ty * W + tx] !== T.WATER) continue;
        const ph = (time * 3 + tx * 1.7 + ty * 2.3) % 6;
        if (ph < 2) { c.fillStyle = R.terrain.th.waterHi; c.fillRect(tx * TP + 3 + ((tx * 7) % 8), ty * TP + 4 + ((ty * 5) % 8), 3, 1); }
      }
    // sector borders: dotted lines like survey markers
    c.fillStyle = 'rgba(0,0,0,0.28)';
    const sw = map.sw * TP, sh = map.sh * TP;
    for (let k = 1; k < map.cols; k++) for (let y = Math.max(0, vy0 | 0); y < Math.min(map.H * TP, vy1); y += 4) c.fillRect(k * sw, y - (y % 4), 1, 2);
    for (let k = 1; k < map.rows; k++) for (let x = Math.max(0, vx0 | 0); x < Math.min(W * TP, vx1); x += 4) c.fillRect(x - (x % 4), k * sh, 2, 1);

    // everything that stands on the map, sorted by its foot line
    const items = [];
    for (const b of map.buildings) items.push({ y: (b.ty + b.h) * TP, b: b });
    for (const s of map.sectors) if (s.flag) items.push({ y: s.flag.y / A + 3, f: s });
    for (const k of map.crates) if (!k.taken) items.push({ y: k.y / A + 4, crate: k });
    for (const u of G.units) {
      const ax = u.x / A, ay = u.y / A;
      if (ax < vx0 || ay < vy0 || ax > vx1 || ay > vy1) continue;
      items.push({ y: ay + (u.def.cls === 'gun' ? -4 : 0), u: u, ax: ax, ay: ay });
    }
    items.sort((a, b) => a.y - b.y);
    const fr = (time * 6 | 0) % 3;
    for (const it of items) {
      if (it.b) {
        const b = it.b, s = buildingSprite(b);
        c.drawImage(s, b.tx * TP, b.ty * TP - LIFT);
        if (b.hit > 0) { c.fillStyle = 'rgba(255,255,255,0.35)'; c.fillRect(b.tx * TP, b.ty * TP - LIFT, b.w * TP, b.h * TP + LIFT); }
        if (b.kind === 'fort' && !b.dead) c.drawImage(flagSprite(b.team, fr), b.tx * TP + b.w * TP / 2 - 1, b.ty * TP - LIFT - 14);
        if (b.dead && (time * 5 + b.id) % 3 < 1) { c.fillStyle = '#ff6a1c'; c.fillRect(b.tx * TP + 8 + (b.id * 7) % 20, b.ty * TP + 6, 2, 2); }
        if (ui.selB === b) brackets(c, b.tx * TP - 2, b.ty * TP - LIFT - 2, b.w * TP + 4, b.h * TP + LIFT + 4, '#fff');
        if (!b.dead && b.team >= 0 && (ui.selB === b || b.hp < b.maxhp)) bar(c, b.tx * TP + 4, b.ty * TP - LIFT - 4, b.w * TP - 8, b.hp / b.maxhp);
        if (!b.dead && b.team >= 0 && b.prod) {
          const x0 = b.tx * TP + 2, y0 = (b.ty + b.h) * TP + 2, w = b.w * TP - 4;
          c.fillStyle = '#000'; c.fillRect(x0, y0, w, 2); c.fillStyle = '#ffd23a'; c.fillRect(x0, y0, Math.round(w * b.progress), 2);
        }
      } else if (it.f) {
        const s = it.f;
        c.fillStyle = 'rgba(0,0,0,0.35)'; c.fillRect(s.flag.x / A - 2, s.flag.y / A + 2, 6, 2);
        c.drawImage(flagSprite(s.owner, (fr + s.id) % 3), Math.round(s.flag.x / A) - 1, Math.round(s.flag.y / A) - 13);
      } else if (it.crate) {
        c.drawImage(crateSprite(), Math.round(it.crate.x / A) - 5, Math.round(it.crate.y / A) - 5);
      } else {
        const u = it.u;
        drawUnitAt(c, u, Math.round(it.ax), Math.round(it.ay), time);
      }
    }
    // selection marks and health on top
    for (const u of G.units) {
      if (!u.sel && u.hp >= u.def.hp) continue;
      const ax = u.x / A, ay = u.y / A, r = u.def.cls === 'robot' ? 6 : u.def.r / A + 2;
      if (ax < vx0 || ay < vy0 || ax > vx1 || ay > vy1) continue;
      const top = u.def.cls === 'robot' ? ay - 13 : ay - r;
      if (u.sel) brackets(c, ax - r - 1, top - 1, 2 * r + 3, (u.def.cls === 'robot' ? 17 : 2 * r) + 3, '#ffffff');
      if (u.sel || u.hp < u.def.hp) bar(c, ax - 5, top - 4, 10, u.hp / u.def.hp);
      if (u.sel && u.cargo && u.cargo.length) { c.fillStyle = '#ffd23a'; for (let i = 0; i < u.cargo.length; i++) c.fillRect(ax - 5 + i * 2, top - 6, 1, 1); }
    }
    // projectiles
    for (const s of G.shots) {
      let x = s.x / A, y = s.y / A;
      if (s.arc) { const tot = Math.hypot(s.tx - s.x0, s.ty - s.y0) + 1, done = Math.hypot(s.x - s.x0, s.y - s.y0), p = Math.min(1, done / tot); y -= Math.sin(p * Math.PI) * Math.min(40, tot / A * 0.35); }
      if (s.k === 'grenade') { c.fillStyle = '#26401c'; c.fillRect(Math.round(x) - 1, Math.round(y) - 1, 2, 2); }
      else if (s.k === 'shell') { c.fillStyle = '#ffe9a0'; c.fillRect(Math.round(x) - 1, Math.round(y) - 1, 2, 2); c.fillStyle = '#000'; c.fillRect(Math.round(x), Math.round(y), 1, 1); }
      else {
        const ca = Math.cos(s.ang), sa = Math.sin(s.ang);
        line(c, x - ca * 3, y - sa * 3, x + ca * 1, y + sa * 1, '#ffffff');
        line(c, x - ca * 6, y - sa * 6, x - ca * 4, y - sa * 4, '#ff9b2a');
        c.fillStyle = 'rgba(200,200,200,0.5)'; c.fillRect(Math.round(x - ca * 9), Math.round(y - sa * 9), 2, 2);
      }
    }
    // effects
    for (const f of G.fx) {
      const p = f.t / f.life;
      if (f.k === 'line') { if (p < 0.6) line(c, f.x / A, f.y / A - 3, f.x2 / A, f.y2 / A - 3, '#fff3b0'); c.fillStyle = '#ffd23a'; c.fillRect(Math.round(f.x / A) - 1, Math.round(f.y / A) - 4, 2, 2); }
      else if (f.k === 'beam') { line(c, f.x / A, f.y / A - 3, f.x2 / A, f.y2 / A - 3, p < 0.5 ? '#ffffff' : '#66ffff'); }
      else if (f.k === 'flame') {
        for (let k = 0; k < 5; k++) {
          const q = Math.min(1, p + k * 0.18), x = f.x / A + (f.x2 - f.x) / A * q, y = f.y / A + (f.y2 - f.y) / A * q - 3;
          disc(c, x, y, 1 + Math.round(2 * q), q < 0.4 ? '#ffe36b' : q < 0.7 ? '#ff8a1c' : '#c8261a');
        }
      } else {
        const r = Math.max(1, Math.round(f.r / A * (0.35 + 0.65 * p)));
        const cols = ['#ffffff', '#ffe36b', '#ff8a1c', '#c8261a', '#555555'];
        const ci = Math.min(4, Math.floor(p * 5));
        disc(c, f.x / A, f.y / A - 2, r, cols[ci]);
        if (ci < 3) disc(c, f.x / A, f.y / A - 2, Math.max(0, r - 2), cols[ci + 1 > 3 ? 3 : ci + 1]);
        if (p > 0.5) { c.fillStyle = 'rgba(80,80,80,' + (1 - p) + ')'; c.fillRect(Math.round(f.x / A) - 2, Math.round(f.y / A - r - 4 * p), 4, 3); }
      }
    }
    // order markers and drag box
    for (const m of ui.marks) {
      const r = 2 + Math.round(6 * m.t / 0.5);
      brackets(c, m.x / A - r, m.y / A - r, 2 * r + 1, 2 * r + 1, m.atk ? '#ff5a46' : '#7dff7d');
    }
    if (ui.drag) {
      const d = ui.drag, x0 = Math.round(Math.min(d.x0, d.x1) / A), y0 = Math.round(Math.min(d.y0, d.y1) / A), x1 = Math.round(Math.max(d.x0, d.x1) / A), y1 = Math.round(Math.max(d.y0, d.y1) / A);
      c.fillStyle = '#ffffff';
      for (let x = x0; x <= x1; x += 2) { c.fillRect(x, y0, 1, 1); c.fillRect(x, y1, 1, 1); }
      for (let y = y0; y <= y1; y += 2) { c.fillRect(x0, y, 1, 1); c.fillRect(x1, y, 1, 1); }
    }
    if (ui.rally) { const r = ui.rally; c.drawImage(flagSprite(0, fr), Math.round(r.x / A) - 1, Math.round(r.y / A) - 13); }
  };

  ZG.drawMini = function (c, G, R, view, cw, ch) {
    const map = G.map, sx = cw / (map.W * TS), sy = ch / (map.H * TS);
    c.imageSmoothingEnabled = false;
    c.drawImage(R.mini, 0, 0, cw, ch);
    for (const s of map.sectors) if (s.owner >= 0) { c.fillStyle = s.owner === 0 ? 'rgba(210,59,42,0.32)' : 'rgba(47,98,214,0.36)'; c.fillRect(s.x0 * TS * sx, s.y0 * TS * sy, map.sw * TS * sx, map.sh * TS * sy); }
    for (const b of map.buildings) { c.fillStyle = b.dead ? '#222' : b.team < 0 ? '#bbb' : ZG.TEAMCOL[b.team]; c.fillRect(b.tx * TS * sx, b.ty * TS * sy, b.w * TS * sx, b.h * TS * sy); }
    for (const s of map.sectors) if (s.flag) { c.fillStyle = s.owner < 0 ? '#fff' : s.owner === 0 ? '#ff9a88' : '#9ec0ff'; c.fillRect(s.flag.x * sx - 1, s.flag.y * sy - 1, 3, 3); }
    for (const u of G.units) { c.fillStyle = u.team < 0 ? '#ddd' : u.team === 0 ? '#ff5544' : '#5c8fff'; const z = u.def.cls === 'robot' ? 2 : 3; c.fillRect(u.x * sx - z / 2, u.y * sy - z / 2, z, z); }
    c.strokeStyle = '#fff'; c.lineWidth = 1; c.strokeRect(view.x * sx + 0.5, view.y * sy + 0.5, view.w * sx, view.h * sy);
  };
})();
