// Zonebots - optional loader for the data files of the original 1996 game "Z".
//
// Nothing from the original game is part of this project. If a player owns the game, they can pick its
// installation folder in the menu; the files are read locally in the browser and kept in the browser's
// own storage. The format notes below come from analysing the data files of the GOG release.
//
//   LEVELxx.MAP  56433 bytes, little endian
//     0x2771  tileset name, 13 bytes ("DESERT.LBM", "ARTIC.LBM", ...); the part before the dot names .PAL/.BLK
//     0x278D  u16 width, u16 height (tiles)
//     0x2791  plane A, 128x128 bytes (stride 128): low 8 bits of the tile index
//     0x6791  plane B, 128x128 bytes: bit 7 = tile index bit 8, bits 0-6 = terrain code
//   <WORLD>.PAL  256 RGB entries, 6 bits per channel
//   <WORLD>.BLK  512 tiles, 16x16 pixels, 8 bits per pixel
//   BUILDxx.DAT  80-byte records: u16 x, u16 y (pixels), byte type; x = 0xFFFF marks an empty slot
//   OBJECTxx.DAT 10-byte records: u16 x, u16 y (pixels), byte type (0x0A flag, 0x0B rock, 0x21 vehicle)
//   CPUPLRxx.DAT starts with u16 tile rectangles of the computer player's area
//   levels.dat   240-byte records: name (20 bytes) + file names
(function () {
  const ZG = globalThis.ZG, T = ZG.T, TS = ZG.TILE;
  const Z = ZG.Z = { files: null, levels: null };

  const WORLDS = { DESERT: 0, VOLCAN: 1, ARTIC: 2, ARCTIC: 2, JUNGLE: 3, CITY: 4 };
  // terrain codes (plane B bits 0-6)
  const C_ROAD = 7, C_CLIFF = 19, WATER = { 9: 1, 38: 1, 39: 1 };
  const FORT = { 16: 1, 20: 1, 21: 1, 30: 1, 63: 1 }, FORT_GATE = { 30: 1, 63: 1 }, BLD = { 27: 1, 31: 1, 32: 1 };
  const O_FLAG = 0x0a, O_ROCK = 0x0b, O_VEHICLE = 0x21;
  // BUILD record type -> our building kind (best guess, see CONTRIBUTING for open questions)
  const BKIND = { 1: 'robot', 4: 'vehicle', 0: 'vehicle', 8: 'repair' };

  const u16 = (d, o) => d[o] | (d[o + 1] << 8);
  const str = (d, o, n) => { let s = ''; for (let i = 0; i < n && d[o + i]; i++) s += String.fromCharCode(d[o + i]); return s; };

  // ---------- reading the folder ----------
  const NEED = /^(level\d\d\.map|build\d\d\.dat|object\d\d\.dat|cpuplr\d\d\.dat|bridge\d\d\.dat|[a-z]+\.pal|[a-z]+\.blk|levels\.dat|sprites\.rsc)$/i;
  Z.wanted = name => NEED.test(name);
  // files: { lowercased name: Uint8Array }
  Z.use = function (files) {
    Z.files = files;
    Z.levels = [];
    const ld = files['levels.dat'];
    for (let i = 1; i <= 20; i++) {
      const nn = String(i).padStart(2, '0');
      if (!files['level' + nn + '.map']) continue;
      let name = 'Level ' + i;
      if (ld) {
        for (let r = 0; r * 240 < ld.length; r++) if (str(ld, r * 240 + 0x14, 13).toLowerCase() === 'level' + nn + '.map') name = str(ld, r * 240, 20) || name;
      }
      Z.levels.push({ n: i, nn: nn, name: name });
    }
    return Z.levels.length;
  };
  Z.ready = () => !!(Z.levels && Z.levels.length);

  // keep the picked files in IndexedDB so the folder does not have to be picked every time
  const DB = 'zonebots-z', STORE = 'files';
  function idb() {
    return new Promise(function (res, rej) {
      if (typeof indexedDB === 'undefined') return rej(new Error('no IndexedDB'));
      const q = indexedDB.open(DB, 1);
      q.onupgradeneeded = () => q.result.createObjectStore(STORE);
      q.onsuccess = () => res(q.result); q.onerror = () => rej(q.error);
    });
  }
  Z.save = async function (files) {
    const db = await idb();
    await new Promise((res, rej) => { const tx = db.transaction(STORE, 'readwrite'); tx.objectStore(STORE).put(files, 'all'); tx.oncomplete = res; tx.onerror = () => rej(tx.error); });
  };
  Z.restore = async function () {
    try {
      const db = await idb();
      const files = await new Promise((res, rej) => { const q = db.transaction(STORE).objectStore(STORE).get('all'); q.onsuccess = () => res(q.result); q.onerror = () => rej(q.error); });
      return files ? Z.use(files) : 0;
    } catch (e) { return 0; }
  };
  Z.forget = async function () {
    Z.files = null; Z.levels = null;
    try { const db = await idb(); await new Promise(res => { const tx = db.transaction(STORE, 'readwrite'); tx.objectStore(STORE).delete('all'); tx.oncomplete = res; tx.onerror = res; }); } catch (e) { /* nothing stored */ }
  };
  // from an <input type=file webkitdirectory> selection
  Z.readPicked = async function (fileList) {
    const files = {};
    for (const f of fileList) {
      const name = f.name.toLowerCase();
      if (!Z.wanted(name)) continue;
      // prefer files from the top folder if a name appears twice; sprites.rsc exists once per language
      const path = (f.webkitRelativePath || '').toLowerCase();
      if (name === 'sprites.rsc' ? files[name] && !path.includes('en.lproj') : files[name] && path.split('/').length > 2) continue;
      files[name] = new Uint8Array(await f.arrayBuffer());
    }
    return files;
  };

  // ---------- parsing ----------
  Z.parseMap = function (d) {
    const world = str(d, 0x2771, 13).split('.')[0].toUpperCase();
    const W = u16(d, 0x278d), H = u16(d, 0x278f), tile = new Uint16Array(W * H), code = new Uint8Array(W * H);
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const a = d[0x2791 + y * 128 + x], b = d[0x6791 + y * 128 + x];
      tile[y * W + x] = a | ((b & 0x80) << 1); code[y * W + x] = b & 0x7f;
    }
    return { world: world, W: W, H: H, tile: tile, code: code };
  };
  Z.parseRecords = function (d, size, fields) {
    const out = [];
    if (!d) return out;
    for (let o = 0; o + size <= d.length; o += size) { const x = u16(d, o); if (x === 0xffff) continue; out.push(fields(d, o)); }
    return out;
  };
  Z.palette = function (d) { const p = new Uint8Array(768); for (let i = 0; i < 768; i++) p[i] = Math.min(255, d[i] * 4 + (d[i] >> 4)); return p; };

  // ---------- building an engine map from a Z level ----------
  Z.genMap = function (lv) {
    const f = Z.files, nn = lv.zlevel, zm = Z.parseMap(f['level' + nn + '.map']);
    const W = zm.W, H = zm.H, N = W * H, tiles = new Uint8Array(N), block = new Uint8Array(N);
    for (let i = 0; i < N; i++) {
      const c = zm.code[i];
      tiles[i] = c === C_ROAD ? T.ROAD : c === C_CLIFF ? T.ROCK : WATER[c] ? T.WATER : T.GROUND;
      if (tiles[i] === T.ROCK || tiles[i] === T.WATER) block[i] = 1;
    }
    const objs = Z.parseRecords(f['object' + nn + '.dat'], 10, (d, o) => ({ x: u16(d, o), y: u16(d, o + 2), type: d[o + 4], b5: d[o + 5] }));
    const blds = Z.parseRecords(f['build' + nn + '.dat'], 80, (d, o) => ({ x: u16(d, o), y: u16(d, o + 2), type: d[o + 4] }));
    const cpu = [];
    const cd = f['cpuplr' + nn + '.dat'];
    if (cd) for (let o = 0; o + 8 <= Math.min(cd.length, 240); o += 30) { const r = [u16(cd, o), u16(cd, o + 2), u16(cd, o + 4), u16(cd, o + 6)]; if (r[2] > r[0] && r[3] > r[1] && r[2] < 200 && r[3] < 200) cpu.push(r); }

    // forts: connected areas of fort footprint codes
    const seen = new Uint8Array(N), forts = [];
    for (let i = 0; i < N; i++) {
      if (seen[i] || !(FORT[zm.code[i]] || zm.tile[i] === 494)) continue;
      const st = [i], cells = []; seen[i] = 1;
      while (st.length) {
        const j = st.pop(); cells.push(j);
        const x = j % W, y = (j / W) | 0;
        for (const dd of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
          const nx = x + dd[0], ny = y + dd[1], k = ny * W + nx;
          if (nx >= 0 && ny >= 0 && nx < W && ny < H && !seen[k] && (FORT[zm.code[k]] || zm.tile[k] === 494)) { seen[k] = 1; st.push(k); }
        }
      }
      if (cells.length < 12) continue;
      let x0 = W, y0 = H, x1 = 0, y1 = 0, gx = 0, gy = 0, gn = 0;
      for (const j of cells) {
        const x = j % W, y = (j / W) | 0;
        x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y);
        if (FORT_GATE[zm.code[j]]) { gx += x; gy += y; gn++; }
      }
      forts.push({ cells: cells, x0: x0, y0: y0, x1: x1, y1: y1, gate: gn ? [Math.round(gx / gn), Math.round(gy / gn)] : [Math.round((x0 + x1) / 2), y1] });
    }
    if (forts.length < 2) throw new Error('Z level ' + nn + ': forts not found');
    forts.sort((a, b) => (b.x1 - b.x0) * (b.y1 - b.y0) - (a.x1 - a.x0) * (a.y1 - a.y0));
    forts.length = 2;
    // the computer's fort lies inside its CPU area; otherwise the upper fort is the computer's
    const inCpu = ft => cpu.some(r => ft.gate[0] >= r[0] && ft.gate[0] <= r[2] && ft.gate[1] >= r[1] && ft.gate[1] <= r[3]);
    let eIdx = inCpu(forts[0]) && !inCpu(forts[1]) ? 0 : inCpu(forts[1]) && !inCpu(forts[0]) ? 1 : (forts[0].y0 < forts[1].y0 ? 0 : 1);

    const buildings = [], spawns = [], crates = [], sectors = [];
    const addBuilding = (kind, tech, team, x0, y0, w, h, door) => {
      const b = { id: buildings.length, kind: kind, tech: tech, team: team, sector: -1, tx: x0, ty: y0, w: w, h: h, door: { tx: door[0], ty: door[1] },
        hp: kind === 'fort' ? 4000 : 1500, maxhp: kind === 'fort' ? 4000 : 1500, prod: null, progress: 0, dead: false, hit: 0 };
      b.cx = (x0 + w / 2) * TS; b.cy = (y0 + h / 2) * TS;
      for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) if (x >= 0 && y >= 0 && x < W && y < H) block[y * W + x] = 1;
      buildings.push(b); return b;
    };
    const freeNear = (tx, ty) => { for (let r = 0; r < 8; r++) for (let dy = -r; dy <= r; dy++) for (let dx = -r; dx <= r; dx++) { const x = tx + dx, y = ty + dy; if (x >= 0 && y >= 0 && x < W && y < H && !block[y * W + x]) return [x, y]; } return [tx, ty]; };

    // the two forts: both get their own sector without a flag
    const fortTech = Math.min(lv.maxTech, lv.fortTech);
    [eIdx === 0 ? 1 : 0, eIdx].forEach(function (fi, team) {
      const ft = forts[fi];
      const b = addBuilding('fort', fortTech, team, ft.x0, ft.y0, ft.x1 - ft.x0 + 1, ft.y1 - ft.y0 + 1, [0, 0]);
      // the gate must stay walkable: the door is the first free tile in front of it
      const gate = freeNear(ft.gate[0], ft.y1 + 1);
      b.door = { tx: gate[0], ty: gate[1] };
      ft.b = b; ft.team = team;
    });
    for (const r of blds) {
      const tx = Math.floor(r.x / 16), ty = Math.floor(r.y / 16);
      const wh = r.type === 0 ? [5, 4] : r.type === 1 || r.type === 4 ? [4, 5] : [4, 3];
      const kind = BKIND[r.type] || 'robot';
      const door = freeNear(tx + (wh[0] >> 1), ty + wh[1]);
      addBuilding(kind, 1, -1, tx, ty, wh[0], wh[1], door);
    }

    // flags and territories: every flag and fort grows its territory over walkable ground
    const flags = objs.filter(o => o.type === O_FLAG);
    flags.forEach(function (o, k) {
      const tx = Math.floor((o.x + 8) / 16), ty = Math.floor((o.y + 8) / 16);
      sectors.push({ id: k, owner: -1, flag: { x: (tx + 0.5) * TS, y: (ty + 0.5) * TS }, fx: tx, fy: ty, home: false });
    });
    forts.forEach(function (ft) {
      const s = { id: sectors.length, owner: ft.team, flag: null, fx: ft.b.door.tx, fy: ft.b.door.ty, home: true };
      sectors.push(s); ft.b.sector = s.id;
    });
    const secAt = new Int16Array(N).fill(-1), q = [];
    for (const s of sectors) { const j = s.fy * W + s.fx; if (secAt[j] < 0) { secAt[j] = s.id; q.push(j); } }
    for (const ft of forts) for (const j of ft.cells) if (secAt[j] < 0) { secAt[j] = ft.b.sector; q.push(j); }
    for (let pass = 0; pass < 2; pass++) {
      for (let qi = 0; qi < q.length; qi++) {
        const j = q[qi], x = j % W, y = (j / W) | 0;
        for (const dd of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
          const nx = x + dd[0], ny = y + dd[1], k = ny * W + nx;
          if (nx < 0 || ny < 0 || nx >= W || ny >= H || secAt[k] >= 0) continue;
          if (pass === 0 && block[k] && !buildings.some(b => nx >= b.tx && ny >= b.ty && nx < b.tx + b.w && ny < b.ty + b.h)) continue;
          secAt[k] = secAt[j]; q.push(k);
        }
      }
    }
    // buildings belong to the territory they stand in; tech grows towards the middle of the map
    for (const b of buildings) {
      if (b.kind === 'fort') continue;
      b.sector = secAt[b.door.ty * W + b.door.tx];
      if (b.sector < 0) b.sector = secAt[(b.ty + (b.h >> 1)) * W + b.tx + (b.w >> 1)];
      const mid = 1 - Math.min(1, Math.hypot(b.cx / TS - W / 2, b.cy / TS - H / 2) / (Math.hypot(W, H) / 2));
      b.tech = b.kind === 'repair' ? 1 : ZG.clamp(Math.round(1 + (lv.maxTech - 1) * (0.3 + 0.8 * mid)), 1, lv.maxTech);
    }
    for (const s of sectors) s.b = buildings.find(b => b.sector === s.id && b.kind !== 'fort') || null;

    // rocks, vehicles, starting troops
    const rockHp = new Float32Array(N);
    for (const o of objs) {
      const tx = Math.floor((o.x + 8) / 16), ty = Math.floor((o.y + 8) / 16), j = ty * W + tx;
      if (tx < 0 || ty < 0 || tx >= W || ty >= H) continue;
      if (o.type === O_ROCK && !block[j]) { tiles[j] = T.BOULDER; block[j] = 1; rockHp[j] = 260; }
      else if (o.type === O_VEHICLE && !block[j]) spawns.push({ t: lv.maxTech >= 2 && (o.b5 & 1) ? 'light' : 'jeep', team: -1, tx: tx, ty: ty });
    }
    for (const ft of forts) {
      const d = ft.b.door, cnt = lv.start[ft.team] + (ft.team === 0 ? lv.bonus || 0 : 0);
      for (let k = 0; k < cnt; k++) { const p = freeNear(d.tx - 2 + (k % 5), d.ty + 1 + Math.floor(k / 5)); spawns.push({ t: 'grunt', team: ft.team, tx: p[0], ty: p[1] }); block[p[1] * W + p[0]] = 2; }
      const g1 = freeNear(ft.b.tx - 1, d.ty), g2 = freeNear(ft.b.tx + ft.b.w, d.ty);
      spawns.push({ t: 'gatling', team: ft.team, tx: g1[0], ty: g1[1], sector: ft.b.sector }); block[g1[1] * W + g1[0]] = 2;
      spawns.push({ t: 'gatling', team: ft.team, tx: g2[0], ty: g2[1], sector: ft.b.sector }); block[g2[1] * W + g2[0]] = 2;
    }
    for (let i = 0; i < N; i++) if (block[i] === 2) block[i] = 0;

    const map = { W: W, H: H, tiles: tiles, block: block, sectors: sectors, buildings: buildings, spawns: spawns, crates: crates,
      sw: W, sh: H, cols: 1, rows: 1, secAt: secAt, rockHp: rockHp, bridges: [], bridgeAt: new Int16Array(N).fill(-1), z: zm, world: WORLDS[zm.world] != null ? WORLDS[zm.world] : 0 };
    // forts first, as the generator does
    buildings.sort((a, b) => (b.kind === 'fort') - (a.kind === 'fort') || a.team - b.team);
    buildings.forEach((b, i) => { b.id = i; });
    return map;
  };

  // engine level definitions for the original campaign: our difficulty curve, their maps
  Z.makeLevel = function (k) {
    const zl = Z.levels[k], base = ZG.LEVELS[Math.min(k, ZG.LEVELS.length - 1)];
    const lv = Object.assign({}, base, { zlevel: zl.nn, zname: zl.name, original: true });
    const zm = Z.parseMap(Z.files['level' + zl.nn + '.map']);
    lv.theme = WORLDS[zm.world] != null ? WORLDS[zm.world] : 0;
    return lv;
  };

  // ---------- sprites.rsc ----------
  // u16 count, count x u32 offsets, count group bytes, 768-byte palette, then the sprites.
  // A sprite: byte height, byte width/8, one byte per row, padding to 4 bytes, then w*h pixels in
  // VGA "mode X" order (plane p holds columns p, p+4, ...). Colour 0 is transparent.
  Z.sprite = function (i) {
    const d = Z.files && Z.files['sprites.rsc'];
    if (!d) return null;
    const n = u16(d, 0);
    if (i < 0 || i >= n) return null;
    const off = (d[2 + i * 4] | d[3 + i * 4] << 8 | d[4 + i * 4] << 16 | d[5 + i * 4] << 24) >>> 0;
    const h = d[off], w = d[off + 1] * 8, hdr = (2 + h + 3) & ~3, q = w >> 2, px = new Uint8Array(w * h);
    for (let p = 0; p < 4; p++) for (let y = 0; y < h; y++) for (let x = 0; x < q; x++) px[y * w + x * 4 + p] = d[off + hdr + p * q * h + y * q + x] || 0;
    return { w: w, h: h, px: px };
  };
  // the sprite as a canvas in a world palette
  Z.spriteCanvas = function (i, world) {
    const s = Z.sprite(i);
    if (!s || !s.w || !s.h) return null;
    const nm = (world || 'DESERT').toLowerCase(), pd = Z.files[nm + '.pal'] || Z.files['desert.pal'], pal = Z.palette(pd);
    const c = document.createElement('canvas'); c.width = s.w; c.height = s.h;
    const x = c.getContext('2d'), id = x.createImageData(s.w, s.h), o = id.data;
    for (let k = 0; k < s.px.length; k++) { const v = s.px[k]; if (!v) continue; o[k * 4] = pal[v * 3]; o[k * 4 + 1] = pal[v * 3 + 1]; o[k * 4 + 2] = pal[v * 3 + 2]; o[k * 4 + 3] = 255; }
    x.putImageData(id, 0, 0);
    return c;
  };

  // ---------- original terrain image ----------
  Z.terrainCanvas = function (map) {
    const zm = map.z, pal = Z.palette(Z.files[zm.world.toLowerCase() + '.pal'] || Z.files[(zm.world === 'ARTIC' ? 'arctic' : zm.world.toLowerCase()) + '.pal']);
    const blk = Z.files[zm.world.toLowerCase() + '.blk'] || Z.files[(zm.world === 'ARTIC' ? 'arctic' : zm.world.toLowerCase()) + '.blk'];
    const W = map.W * 16, H = map.H * 16, cv = document.createElement('canvas'); cv.width = W; cv.height = H;
    const cx = cv.getContext('2d'), id = cx.createImageData(W, H), d = id.data;
    const paint = function (tx, ty, ti, target, tw) {
      for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
        const p = blk[ti * 256 + y * 16 + x] * 3, o = ((ty * 16 + y) * tw + tx * 16 + x) * 4;
        target[o] = pal[p]; target[o + 1] = pal[p + 1]; target[o + 2] = pal[p + 2]; target[o + 3] = 255;
      }
    };
    for (let ty = 0; ty < map.H; ty++) for (let tx = 0; tx < map.W; tx++) paint(tx, ty, zm.tile[ty * map.W + tx], d, W);
    cx.putImageData(id, 0, 0);
    // a tile picture to show where a bridge was destroyed: any water tile of this map
    let waterTile = -1;
    for (let i = 0; i < zm.code.length && waterTile < 0; i++) if (WATER[zm.code[i]]) waterTile = zm.tile[i];
    return {
      canvas: cv, th: ZG.THEMES[map.world], original: true,
      redraw: function (list) {
        for (const j of list) {
          const tx = j % map.W, ty = (j / map.W) | 0, tid = cx.createImageData(16, 16);
          const ti = map.tiles[j] === T.WATER && waterTile >= 0 ? waterTile : zm.tile[j];
          paint(0, 0, ti, tid.data, 16); cx.putImageData(tid, tx * 16, ty * 16);
        }
      }
    };
  };
  Z.miniCanvas = function (map, terrain) {
    const c = document.createElement('canvas'); c.width = map.W; c.height = map.H;
    c.getContext('2d').drawImage(terrain.canvas, 0, 0, map.W, map.H);
    return c;
  };
})();
