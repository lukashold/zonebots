// Zonebots - simulation (no DOM access, also runs headless in Node)
(function () {
  const ZG = globalThis.ZG, TS = ZG.TILE;
  const SPD = [0.8, 1, 0.8, 0.8, 0.5, 0.95];
  const CAP = 50, GUNCAP = 3;
  const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);

  ZG.newGame = function (idx, opt) {
    opt = opt || {};
    const df = ZG.DIFF[opt.diff == null ? 1 : opt.diff];
    const lv = Object.assign({}, typeof idx === 'object' ? idx : ZG.LEVELS[idx], { bonus: df.bonus }), map = ZG.genMap(lv);
    const G = { lv: lv, map: map, units: [], shots: [], fx: [], ev: [], time: 0, over: null, nid: 1,
      aiTeams: opt.aiTeams || [1], aiT: 1, rnd: ZG.rng(lv.seed ^ 0x9e3779b9), alertT: 0,
      stats: [{ kills: 0, losses: 0 }, { kills: 0, losses: 0 }],
      ai: { aiRate: lv.aiRate * df.rate, aiInt: lv.aiInt * df.int, aiWait: lv.aiWait * df.wait, aiRush: lv.aiRush, aiBatch: lv.aiBatch, aiMin: lv.aiMin } };
    for (const s of map.spawns) ZG.spawn(G, s.t, s.team, (s.tx + 0.5) * TS, (s.ty + 0.5) * TS, s.sector);
    for (const b of map.buildings) if (b.team >= 0) b.prod = pickProd(G, b);
    return G;
  };

  ZG.spawn = function (G, t, team, x, y, sector) {
    const d = ZG.UNITS[t];
    const u = { id: G.nid++, t: t, def: d, team: team, x: x, y: y, hp: d.hp, ang: Math.PI / 2, tang: Math.PI / 2,
      path: null, pi: 0, order: null, cd: G.rnd() * d.rl, tgt: null, scan: G.rnd() * 0.3, dead: false, sel: false,
      hit: 0, sector: sector == null ? -1 : sector, near: 0, walk: 0, moving: false, gren: 0, gcd: 0 };
    if (d.cls === 'gun') G.map.block[Math.floor(y / TS) * G.map.W + Math.floor(x / TS)] = 1;
    G.units.push(u);
    return u;
  };

  ZG.products = function (b) {
    if (b.kind === 'repair') return [];
    const out = [];
    for (const k in ZG.UNITS) {
      const d = ZG.UNITS[k];
      if (d.tech <= b.tech && (b.kind === 'fort' || d.cls === b.kind || d.cls === 'gun')) out.push(k);
    }
    return out;
  };

  function isAI(G, team) { return G.aiTeams.indexOf(team) >= 0; }

  function pickProd(G, b) {
    const list = ZG.products(b);
    if (!list.length) return null;
    if (!isAI(G, b.team)) return list.indexOf(b.prod) >= 0 ? b.prod : list[0];
    const guns = gunCount(G, b.sector);
    let best = null, bs = -1;
    for (const k of list) {
      const d = ZG.UNITS[k];
      let s = d.tech * (0.6 + G.rnd());
      if (d.cls === 'gun') s = guns >= 2 ? -1 : s * 0.45;
      if (d.cls === 'vehicle') s *= 1.15;
      if (s > bs) { bs = s; best = k; }
    }
    return best;
  }

  function gunCount(G, sec) {
    let n = 0;
    for (const u of G.units) if (!u.dead && u.def.cls === 'gun' && u.sector === sec) n++;
    return n;
  }
  function mobileCount(G, team) {
    let n = 0;
    for (const u of G.units) if (!u.dead && u.team === team && u.def.sp) n++;
    return n;
  }
  ZG.prodRate = function (G, team) {
    let own = 0;
    for (const s of G.map.sectors) if (s.owner === team) own++;
    return (0.55 + 1.1 * own / G.map.sectors.length) * (team === 1 && isAI(G, 1) ? G.ai.aiRate : 1);
  };

  function blockedPx(map, x, y) {
    const tx = Math.floor(x / TS), ty = Math.floor(y / TS);
    return tx < 0 || ty < 0 || tx >= map.W || ty >= map.H || map.block[ty * map.W + tx] === 1;
  }

  function setPath(G, u, x, y) {
    const p = ZG.findPath(G.map, u.x, u.y, x, y);
    u.path = p; u.pi = 0; u.near = 0;
    return p ? p[p.length - 1] : null;
  }

  // free tiles in a spiral around a tile
  function freeTiles(map, tx, ty, n) {
    const out = [];
    for (let r = 0; r < 12 && out.length < n; r++)
      for (let dy = -r; dy <= r; dy++) for (let dx = -r; dx <= r; dx++) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) !== r) continue;
        const x = tx + dx, y = ty + dy;
        if (x >= 0 && y >= 0 && x < map.W && y < map.H && !map.block[y * map.W + x]) out.push([x, y]);
      }
    return out;
  }

  const QUAD = [[-8, -8], [8, -8], [-8, 8], [8, 8]];
  ZG.orderMove = function (G, units, x, y, kind) {
    units = units.filter(u => !u.dead && u.def.sp);
    if (!units.length) return;
    const map = G.map, tx = ZG.clamp(Math.floor(x / TS), 0, map.W - 1), ty = ZG.clamp(Math.floor(y / TS), 0, map.H - 1);
    if (units.length === 1 && !map.block[ty * map.W + tx]) {
      const p = setPath(G, units[0], x, y);
      units[0].order = p ? { k: kind || 'move', x: p.x, y: p.y } : null;
      return;
    }
    const tiles = freeTiles(map, tx, ty, units.length);
    let ti = 0, q = 4, qt = null;
    for (const u of units) {
      let sx, sy;
      if (u.def.cls === 'robot') {
        if (q >= 4) { qt = tiles[Math.min(ti++, tiles.length - 1)]; q = 0; }
        if (!qt) return;
        sx = (qt[0] + 0.5) * TS + QUAD[q][0]; sy = (qt[1] + 0.5) * TS + QUAD[q][1]; q++;
      } else {
        const t = tiles[Math.min(ti++, tiles.length - 1)];
        if (!t) return;
        sx = (t[0] + 0.5) * TS; sy = (t[1] + 0.5) * TS;
      }
      const p = setPath(G, u, sx, sy);
      u.order = p ? { k: kind || 'move', x: p.x, y: p.y } : null;
    }
  };

  ZG.orderAttack = function (G, units, v) {
    for (const u of units) if (!u.dead && u.def.dmg && u !== v) { u.order = { k: 'attack', u: v, rp: 0 }; u.path = null; }
  };
  ZG.orderEnter = function (G, units, v) {
    // the closest robot crews the vehicle, everybody else walks along
    let best = null, bd = 1e9;
    for (const u of units) if (!u.dead && u.def.cls === 'robot') { const d = dist(u, v); if (d < bd) { bd = d; best = u; } }
    if (best) { best.order = { k: 'enter', u: v, rp: 0 }; best.path = null; }
    ZG.orderMove(G, units.filter(u => u !== best), v.x, v.y);
  };
  ZG.orderStop = function (G, units) { for (const u of units) { u.order = null; u.path = null; } };
  ZG.orderFort = function (G, units, b) {
    for (const u of units) if (!u.dead && u.def.sp) { u.order = { k: 'fort', b: b, enter: u.def.cls === 'robot' }; u.path = null; }
  };

  function distRect(u, b) {
    const x0 = b.tx * TS, y0 = b.ty * TS, x1 = x0 + b.w * TS, y1 = y0 + b.h * TS;
    const dx = u.x < x0 ? x0 - u.x : u.x > x1 ? u.x - x1 : 0, dy = u.y < y0 ? y0 - u.y : u.y > y1 ? u.y - y1 : 0;
    return Math.hypot(dx, dy);
  }

  function nearestEnemy(G, u, range) {
    let best = null, bd = range * range;
    for (const v of G.units) {
      if (v.dead || v.team < 0 || v.team === u.team) continue;
      const dx = v.x - u.x, dy = v.y - u.y, d2 = dx * dx + dy * dy;
      if (d2 < bd) { bd = d2; best = v; }
    }
    return best;
  }

  function endGame(G, winner) { if (!G.over) { G.over = { winner: winner, time: G.time }; G.ev.push({ k: 'over', winner: winner }); } }

  function captureSector(G, s, team) {
    const prev = s.owner;
    s.owner = team;
    const b = s.b;
    if (b) { b.team = team; b.progress = 0; b.rally = null; b.prod = pickProd(G, b); }
    for (const u of G.units) if (!u.dead && u.def.cls === 'gun' && u.sector === s.id) { u.team = team; u.tgt = null; }
    G.ev.push({ k: 'capture', team: team, prev: prev, x: s.flag.x, y: s.flag.y });
  }

  function damage(G, u, amt, src) {
    if (u.dead) return;
    u.hp -= amt; u.hit = 0.12;
    if (u.hp <= 0) {
      u.dead = true;
      const big = u.def.cls !== 'robot';
      G.fx.push({ k: 'boom', x: u.x, y: u.y, t: 0, life: big ? 0.6 : 0.35, r: big ? 26 : 12 });
      G.ev.push({ k: 'boom', x: u.x, y: u.y, big: big });
      if (u.def.cls === 'gun') G.map.block[Math.floor(u.y / TS) * G.map.W + Math.floor(u.x / TS)] = 0;
      if (u.team >= 0) G.stats[u.team].losses++;
      if (src && src.team >= 0 && src.team !== u.team) G.stats[src.team].kills++;
    } else if (!u.order && u.def.sp && u.team >= 0 && src && !src.dead && src.team !== u.team && src.team >= 0) {
      u.order = { k: 'attack', u: src, rp: 0 };
    }
  }

  function hitFort(G, b, amt, src) {
    if (b.dead) return;
    b.hp -= amt; b.hit = 0.12;
    if (b.team === 0 && G.alertT <= 0) { G.alertT = 12; G.ev.push({ k: 'alert', x: b.cx, y: b.cy }); }
    if (b.hp <= 0) {
      b.dead = true;
      G.fx.push({ k: 'boom', x: b.cx, y: b.cy, t: 0, life: 1.2, r: 80 });
      G.ev.push({ k: 'boom', x: b.cx, y: b.cy, big: true });
      endGame(G, 1 - b.team);
    }
  }

  function armorMul(w, def) {
    const a = def.armor || 0;
    return w === 'bullet' || w === 'flame' ? 1 - a : w === 'laser' ? 1 - a / 2 : 1;
  }

  function fire(G, u, v, b) {
    const d = u.def, w = d.w, tx = v ? v.x : b.cx, ty = v ? v.y : b.cy;
    u.cd = d.rl * (0.9 + G.rnd() * 0.2);
    const a = Math.atan2(ty - u.y, tx - u.x), mx = u.x + Math.cos(a) * (d.r + 3), my = u.y + Math.sin(a) * (d.r + 3);
    G.ev.push({ k: 'shot', w: w, x: u.x, y: u.y });
    if (w === 'bullet' || w === 'laser' || w === 'flame') {
      const jx = (G.rnd() - 0.5) * 8, jy = (G.rnd() - 0.5) * 8;
      if (w === 'flame') G.fx.push({ k: 'flame', x: mx, y: my, x2: tx + jx, y2: ty + jy, t: 0, life: 0.3 });
      else G.fx.push({ k: w === 'laser' ? 'beam' : 'line', x: mx, y: my, x2: tx + jx, y2: ty + jy, t: 0, life: w === 'laser' ? 0.14 : 0.07 });
      if (v) {
        if (w !== 'bullet' || G.rnd() < 0.88) {
          damage(G, v, d.dmg * armorMul(w, v.def), u);
          if (u.t === 'sniper' && v.t === 'jeep' && !v.dead && G.rnd() < 0.25) {
            v.team = -1; v.order = null; v.path = null; v.tgt = null;
            G.ev.push({ k: 'driver', x: v.x, y: v.y, team: u.team });
          }
        }
      } else hitFort(G, b, d.dmg * (w === 'laser' ? 0.6 : 0.3), u);
    } else {
      G.shots.push({ k: w, x: mx, y: my, tx: tx, ty: ty, v: w === 'shell' ? null : v, b: b || null, src: u, team: u.team,
        dmg: d.dmg, splash: d.splash || 0, sp: w === 'shell' ? 430 : w === 'rocket' ? 300 : 270, ang: a });
    }
  }

  function unitThink(G, u, dt) {
    const d = u.def;
    u.cd -= dt; u.scan -= dt; if (u.hit > 0) u.hit -= dt;
    u.moving = false;
    if (u.team < 0) return;
    let hold = false, tb = null, o = u.order;
    if (o) {
      if (o.k === 'attack') {
        const v = o.u;
        if (v.dead || v.team === u.team || v.team < 0) { u.order = null; u.path = null; }
        else {
          const dd = dist(u, v);
          if (dd <= d.range * 0.92) { hold = true; u.path = null; u.tgt = v; }
          else if (!d.sp) u.order = null;
          else if (!u.path || (o.rp -= dt) <= 0) { o.rp = 0.8; if (!setPath(G, u, v.x, v.y)) u.order = null; }
        }
      } else if (o.k === 'enter') {
        const v = o.u;
        if (v.dead || v.team >= 0) { u.order = null; u.path = null; }
        else if (dist(u, v) < d.r + v.def.r + 8) {
          v.team = u.team; v.order = null; v.hp = Math.max(v.hp, v.def.hp * 0.5); u.dead = true;
          G.ev.push({ k: 'enter', x: v.x, y: v.y, team: u.team });
          return;
        } else if (!u.path || (o.rp -= dt) <= 0) { o.rp = 1.5; if (!setPath(G, u, v.x, v.y)) u.order = null; }
      } else if (o.k === 'fort') {
        const b = o.b;
        if (b.dead || b.team === u.team) { u.order = null; u.path = null; }
        else {
          const door = { x: (b.door.tx + 0.5) * TS, y: (b.door.ty + 0.5) * TS };
          if (o.enter) {
            if (dist(u, door) < 13) { u.dead = true; G.ev.push({ k: 'sabotage', x: door.x, y: door.y, team: u.team }); hitFort(G, b, 400, u); return; }
            if (!u.path && !setPath(G, u, door.x, door.y)) u.order = null;
          } else if (distRect(u, b) <= d.range * 0.9) { hold = true; u.path = null; tb = b; }
          else if (!u.path && !setPath(G, u, door.x, door.y)) u.order = null;
          if (b.team === 0 && G.alertT <= 0 && dist(u, door) < 260) { G.alertT = 12; G.ev.push({ k: 'alert', x: b.cx, y: b.cy }); }
        }
      } else if (o.k === 'amove') {
        if (o.sec != null && G.map.sectors[o.sec].owner === u.team) { u.order = null; u.path = null; }
        else if (u.tgt && !u.tgt.dead) hold = true;
        else if (!u.path) {
          if (Math.hypot(u.x - o.x, u.y - o.y) < 24) u.order = null;
          else { const p = setPath(G, u, o.x, o.y); if (p) { o.x = p.x; o.y = p.y; } else u.order = null; }
        }
      } else if (o.k === 'move') {
        if (!u.path) u.order = null;
      }
    }

    // weapons
    if (d.dmg) {
      const v = u.tgt;
      if (v && (v.dead || v.team === u.team || v.team < 0 || dist(u, v) > d.range)) u.tgt = null;
      if (u.scan <= 0) {
        u.scan = 0.3;
        const ov = u.order && u.order.k === 'attack' ? u.order.u : null;
        u.tgt = ov && dist(u, ov) <= d.range ? ov : nearestEnemy(G, u, d.range);
      }
      if (u.gren > 0 && (u.gcd -= dt) <= 0) {
        const gv = u.tgt && u.tgt.def.cls !== 'robot' && dist(u, u.tgt) <= 140 ? u.tgt : null, gb = !gv && tb && distRect(u, tb) <= 140 ? tb : null;
        if (gv || gb) {
          u.gren--; u.gcd = 2.5;
          G.shots.push({ k: 'grenade', x: u.x, y: u.y, tx: gv ? gv.x : gb.cx, ty: gv ? gv.y : gb.cy, v: null, b: gb, src: u, team: u.team, dmg: 70, splash: 30, sp: 230, ang: 0 });
        }
      }
      if (u.tgt) {
        u.tang = Math.atan2(u.tgt.y - u.y, u.tgt.x - u.x);
        if (d.cls === 'robot') u.ang = u.tang;
        if (u.cd <= 0) fire(G, u, u.tgt, null);
      } else if (tb) {
        u.tang = Math.atan2(tb.cy - u.y, tb.cx - u.x);
        if (d.cls === 'robot') u.ang = u.tang;
        if (u.cd <= 0) fire(G, u, null, tb);
      } else if (d.cls !== 'gun') u.tang = u.ang;
    }

    // movement
    if (u.path && !hold && d.sp) {
      const p = u.path[u.pi], dx = p.x - u.x, dy = p.y - u.y, dd = Math.hypot(dx, dy);
      const map = G.map, ti = ZG.clamp(Math.floor(u.y / TS), 0, map.H - 1) * map.W + ZG.clamp(Math.floor(u.x / TS), 0, map.W - 1);
      const st = d.sp * SPD[map.tiles[ti]] * dt, last = u.pi === u.path.length - 1;
      if (last && dd < 26) { u.near += dt; } // crowded destination: give up after a moment
      if (dd <= st || (last && u.near > 1.2)) {
        if (dd <= st) { u.x = p.x; u.y = p.y; }
        if (++u.pi >= u.path.length) u.path = null;
      } else {
        u.x += dx / dd * st; u.y += dy / dd * st;
        u.ang = Math.atan2(dy, dx); u.moving = true; u.walk += dt * 10;
        if (!u.tgt && !tb) u.tang = u.ang;
      }
    }
  }

  function separate(G) {
    const us = G.units, map = G.map;
    for (let i = 0; i < us.length; i++) {
      const a = us[i]; if (a.dead) continue;
      const af = !a.def.sp || a.team < 0;
      for (let j = i + 1; j < us.length; j++) {
        const b = us[j]; if (b.dead) continue;
        const rr = a.def.r + b.def.r, dx = b.x - a.x, dy = b.y - a.y;
        if (dx > rr || dx < -rr || dy > rr || dy < -rr) continue;
        const d2 = dx * dx + dy * dy;
        if (d2 >= rr * rr) continue;
        const bf = !b.def.sp || b.team < 0;
        if (af && bf) continue;
        let d = Math.sqrt(d2), nx, ny;
        if (d < 0.01) { const an = (a.id * 2.4 + b.id) % 6.28; nx = Math.cos(an); ny = Math.sin(an); d = 0; } else { nx = dx / d; ny = dy / d; }
        const push = (rr - d) * 0.5, pa = af ? 0 : bf ? push * 2 : push, pb = bf ? 0 : af ? push * 2 : push;
        if (pa) { const x = a.x - nx * pa, y = a.y - ny * pa; if (!blockedPx(map, x, y)) { a.x = x; a.y = y; } }
        if (pb) { const x = b.x + nx * pb, y = b.y + ny * pb; if (!blockedPx(map, x, y)) { b.x = x; b.y = y; } }
      }
    }
  }

  function flags(G) {
    for (const s of G.map.sectors) {
      if (!s.flag) continue;
      for (const u of G.units) {
        if (u.dead || u.team < 0 || u.team === s.owner || !u.def.sp) continue;
        const dx = u.x - s.flag.x, dy = u.y - s.flag.y;
        if (dx < 20 && dx > -20 && dy < 20 && dy > -20) { captureSector(G, s, u.team); break; }
      }
    }
  }

  function crates(G) {
    for (const c of G.map.crates) {
      if (c.taken) continue;
      for (const u of G.units) {
        if (u.dead || u.team < 0 || u.def.cls !== 'robot' || Math.abs(u.x - c.x) > 18 || Math.abs(u.y - c.y) > 18) continue;
        c.taken = true;
        for (const v of G.units) if (!v.dead && v.team === u.team && v.def.cls === 'robot' && dist(u, v) < 80) v.gren = 3;
        G.ev.push({ k: 'crate', team: u.team, x: c.x, y: c.y });
        break;
      }
    }
  }

  function shots(G, dt) {
    for (const s of G.shots) {
      if (s.v && !s.v.dead) { s.tx = s.v.x; s.ty = s.v.y; }
      const dx = s.tx - s.x, dy = s.ty - s.y, dd = Math.hypot(dx, dy), st = s.sp * dt;
      if (dd > st) { s.x += dx / dd * st; s.y += dy / dd * st; s.ang = Math.atan2(dy, dx); continue; }
      s.done = true;
      G.fx.push({ k: 'boom', x: s.tx, y: s.ty, t: 0, life: 0.3, r: s.splash * 0.7 + 6 });
      G.ev.push({ k: 'boom', x: s.tx, y: s.ty, big: false });
      if (s.b) { hitFort(G, s.b, s.dmg, s.src); continue; }
      for (const v of G.units) {
        if (v.dead || v.team === s.team || v.team < 0) continue;
        const d = Math.hypot(v.x - s.tx, v.y - s.ty) - v.def.r;
        if (d < s.splash) damage(G, v, s.dmg * (d <= 4 ? 1 : 1 - 0.6 * d / s.splash), s.src);
      }
    }
    if (G.shots.length) G.shots = G.shots.filter(s => !s.done);
  }

  const GUNSPOTS = [[-3, 0], [3, 0], [-3, -2], [3, -2], [4, -1], [-3, -4], [4, -4], [3, -1], [-3, -1]];
  function production(G, dt) {
    const map = G.map, counts = [mobileCount(G, 0), mobileCount(G, 1)];
    for (const b of map.buildings) {
      if (b.hit > 0) b.hit -= dt;
      if (b.dead || b.team < 0) continue;
      const s = map.sectors[b.sector];
      if (b.kind === 'repair') {
        const px = (b.door.tx + 0.5) * TS, py = (b.door.ty + 0.5) * TS;
        for (const u of G.units) if (!u.dead && u.team === b.team && u.hp < u.def.hp && Math.hypot(u.x - px, u.y - py) < 100) u.hp = Math.min(u.def.hp, u.hp + 22 * dt);
        continue;
      }
      if (!b.prod) continue;
      const d = ZG.UNITS[b.prod], gun = d.cls === 'gun';
      if (gun ? gunCount(G, s.id) >= GUNCAP : counts[b.team] >= CAP) {
        if (isAI(G, b.team) && gun) b.prod = pickProd(G, b);
        continue;
      }
      b.progress += dt * ZG.prodRate(G, b.team) / d.time;
      if (b.progress < 1) continue;
      b.progress = 0;
      if (gun) {
        for (const g of GUNSPOTS) {
          const tx = s.fx + g[0], ty = s.fy + g[1];
          if (tx < 0 || ty < 0 || tx >= map.W || ty >= map.H || map.block[ty * map.W + tx]) continue;
          const x = (tx + 0.5) * TS, y = (ty + 0.5) * TS;
          ZG.spawn(G, b.prod, b.team, x, y, s.id);
          // nobody may be standing inside the new emplacement
          for (const u of G.units) if (u.def.sp && Math.floor(u.x / TS) === tx && Math.floor(u.y / TS) === ty) { const f = ZG.nearFree(map, tx, ty); if (f) { u.x = (f[0] + 0.5) * TS; u.y = (f[1] + 0.5) * TS; u.path = null; } }
          break;
        }
      } else {
        const made = [], tiles = freeTiles(map, b.door.tx, b.door.ty, d.n);
        for (let i = 0; i < d.n; i++) { const t = tiles[i % tiles.length]; made.push(ZG.spawn(G, b.prod, b.team, (t[0] + 0.5) * TS, (t[1] + 0.5) * TS)); }
        counts[b.team] += d.n;
        if (b.rally) ZG.orderMove(G, made, b.rally.x, b.rally.y);
        else ZG.orderMove(G, made, (b.door.tx + 0.5) * TS, (b.door.ty + 3.5) * TS);
      }
      G.ev.push({ k: 'ready', team: b.team, t: b.prod, x: b.cx, y: b.cy });
      if (isAI(G, b.team)) b.prod = pickProd(G, b);
    }
  }

  function aiThink(G, team) {
    const map = G.map, enemy = 1 - team;
    const mine = G.units.filter(u => u.team === team && u.def.sp && !u.dead);
    const eFort = map.buildings.find(b => b.kind === 'fort' && b.team === enemy && !b.dead);
    const myFort = map.buildings.find(b => b.kind === 'fort' && b.team === team && !b.dead);
    if (myFort) {
      let threat = null, bd = 280;
      for (const u of G.units) if (u.team === enemy && !u.dead) { const d = Math.hypot(u.x - myFort.cx, u.y - myFort.cy); if (d < bd) { bd = d; threat = u; } }
      if (threat) for (const u of mine)
        if ((!u.order || u.order.k === 'amove') && Math.hypot(u.x - myFort.cx, u.y - myFort.cy) < 520) { u.order = { k: 'attack', u: threat, rp: 0 }; u.path = null; }
    }
    const open = map.sectors.filter(s => s.flag && s.owner !== team);
    const assigned = {}, claimed = {};
    for (const u of mine) if (u.order) {
      if (u.order.sec != null) assigned[u.order.sec] = (assigned[u.order.sec] || 0) + 1;
      if (u.order.k === 'enter') claimed[u.order.u.id] = 1;
    }
    const rush = mine.length >= G.ai.aiRush;
    let budget = G.time < G.ai.aiWait ? 0 : G.ai.aiBatch;
    for (const u of mine) {
      if (u.order) continue;
      if (budget-- <= 0) break;
      if (u.def.cls === 'robot') {
        let nv = null, bd = 340;
        for (const v of G.units) if (v.team < 0 && !v.dead && v.def.sp && !claimed[v.id]) { const d = dist(u, v); if (d < bd) { bd = d; nv = v; } }
        if (nv) { claimed[nv.id] = 1; u.order = { k: 'enter', u: nv, rp: 0 }; continue; }
      }
      if (eFort && (!open.length || (rush && G.rnd() < 0.4))) {
        if (mine.length < G.ai.aiMin) break; // gather a strike force first
        u.order = { k: 'fort', b: eFort, enter: u.def.cls === 'robot' && G.lv.idx >= 3 };
        continue;
      }
      let best = null, bs = 1e9;
      for (const s of open) {
        const sc = Math.hypot(u.x - s.flag.x, u.y - s.flag.y) + 150 * (assigned[s.id] || 0) - (s.owner === enemy ? 60 : 0);
        if (sc < bs) { bs = sc; best = s; }
      }
      if (best && setPath(G, u, best.flag.x, best.flag.y)) {
        u.order = { k: 'amove', x: best.flag.x, y: best.flag.y, sec: best.id };
        assigned[best.id] = (assigned[best.id] || 0) + 1;
      }
    }
  }
  ZG.aiThink = aiThink;

  ZG.step = function (G, dt) {
    if (G.over) return;
    G.time += dt; G.alertT -= dt;
    production(G, dt);
    const us = G.units;
    for (let i = 0; i < us.length; i++) if (!us[i].dead) unitThink(G, us[i], dt);
    separate(G);
    flags(G);
    crates(G);
    shots(G, dt);
    if ((G.aiT -= dt) <= 0) { G.aiT = G.ai.aiInt; for (const t of G.aiTeams) aiThink(G, t); }
    let dead = false;
    for (const u of us) if (u.dead) { dead = true; break; }
    if (dead) G.units = us.filter(u => !u.dead);
    for (const f of G.fx) f.t += dt;
    if (G.fx.length) G.fx = G.fx.filter(f => f.t < f.life);
  };
})();
