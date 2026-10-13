// Zonebots - all drawing is procedural canvas code, there are no image assets
(function () {
  const ZG = globalThis.ZG, TS = ZG.TILE, T = ZG.T;
  const TEAM = ['#d8362f', '#2f6fd8'], TEAMD = ['#8c1f1a', '#1b3f85'], NEUT = '#9a9a92', NEUTD = '#5e5e58';
  ZG.TEAMCOL = TEAM;
  const col = t => t < 0 ? NEUT : TEAM[t], cold = t => t < 0 ? NEUTD : TEAMD[t];

  ZG.renderTerrain = function (map, th, seed) {
    const cv = document.createElement('canvas'); cv.width = map.W * TS; cv.height = map.H * TS;
    const c = cv.getContext('2d'), rnd = ZG.rng(seed + 77), W = map.W, H = map.H, tl = map.tiles;
    const at = (x, y) => x < 0 || y < 0 || x >= W || y >= H ? -1 : tl[y * W + x];
    const wet = t => t === T.WATER || t === T.BRIDGE, rd = t => t === T.ROAD || t === T.BRIDGE;
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const t = tl[y * W + x], px = x * TS, py = y * TS;
      c.fillStyle = th.g[Math.floor(rnd() * th.g.length)]; c.fillRect(px, py, TS, TS);
      c.fillStyle = th.speck;
      for (let k = 0; k < 3; k++) c.fillRect(px + rnd() * 30, py + rnd() * 30, 2, 2);
      if (t === T.ROUGH) {
        c.fillStyle = th.rough; c.fillRect(px, py, TS, TS);
        c.fillStyle = th.roughHi;
        for (let k = 0; k < 6; k++) { const a = px + rnd() * 27, b = py + rnd() * 26; c.fillRect(a, b, 2, 5); c.fillRect(a + 2, b + 2, 2, 3); }
      } else if (t === T.ROAD) {
        c.fillStyle = th.road; c.fillRect(px, py, TS, TS);
        c.fillStyle = th.roadEdge;
        if (!rd(at(x, y - 1))) c.fillRect(px, py, TS, 2);
        if (!rd(at(x, y + 1))) c.fillRect(px, py + TS - 2, TS, 2);
        if (!rd(at(x - 1, y))) c.fillRect(px, py, 2, TS);
        if (!rd(at(x + 1, y))) c.fillRect(px + TS - 2, py, 2, TS);
        c.fillStyle = 'rgba(0,0,0,0.08)';
        for (let k = 0; k < 2; k++) c.fillRect(px + 4 + rnd() * 22, py + 4 + rnd() * 22, 3, 2);
      } else if (t === T.ROCK) {
        c.fillStyle = th.rock; c.fillRect(px, py, TS, TS);
        c.fillStyle = th.rockHi;
        for (let k = 0; k < 2; k++) {
          const a = px + 3 + rnd() * 16, b = py + 3 + rnd() * 14, w = 6 + rnd() * 8;
          c.beginPath(); c.moveTo(a, b + w * 0.6); c.lineTo(a + w * 0.5, b); c.lineTo(a + w, b + w * 0.5); c.lineTo(a + w * 0.4, b + w); c.closePath(); c.fill();
        }
        if (at(x, y - 1) !== T.ROCK) c.fillRect(px, py, TS, 4);
        if (at(x - 1, y) !== T.ROCK) c.fillRect(px, py, 3, TS);
        c.fillStyle = th.rockLo;
        if (at(x, y + 1) !== T.ROCK) c.fillRect(px, py + TS - 7, TS, 7);
        if (at(x + 1, y) !== T.ROCK) c.fillRect(px + TS - 4, py, 4, TS);
      } else if (wet(t)) {
        c.fillStyle = th.water; c.fillRect(px, py, TS, TS);
        c.fillStyle = th.waterHi;
        for (let k = 0; k < 2; k++) c.fillRect(px + 3 + rnd() * 18, py + 4 + rnd() * 24, 7, 1.5);
        c.fillStyle = th.shore;
        if (!wet(at(x, y - 1)) && at(x, y - 1) !== -1) c.fillRect(px, py, TS, 3);
        if (!wet(at(x, y + 1)) && at(x, y + 1) !== -1) c.fillRect(px, py + TS - 3, TS, 3);
        if (!wet(at(x - 1, y)) && at(x - 1, y) !== -1) c.fillRect(px, py, 3, TS);
        if (!wet(at(x + 1, y)) && at(x + 1, y) !== -1) c.fillRect(px + TS - 3, py, 3, TS);
        if (t === T.BRIDGE) {
          c.fillStyle = '#8b6b45'; c.fillRect(px, py, TS, TS);
          c.fillStyle = '#6d5133';
          const horiz = at(x, y - 1) === T.WATER || at(x, y + 1) === T.WATER;
          for (let k = 0; k < TS; k += 8) { if (horiz) c.fillRect(px + k, py, 1.5, TS); else c.fillRect(px, py + k, TS, 1.5); }
          c.fillStyle = '#3f2e1c';
          if (at(x, y - 1) === T.WATER) c.fillRect(px, py, TS, 3);
          if (at(x, y + 1) === T.WATER) c.fillRect(px, py + TS - 3, TS, 3);
          if (at(x - 1, y) === T.WATER) c.fillRect(px, py, 3, TS);
          if (at(x + 1, y) === T.WATER) c.fillRect(px + TS - 3, py, 3, TS);
        }
      }
    }
    return cv;
  };

  ZG.renderMini = function (map, th) {
    const cv = document.createElement('canvas'); cv.width = map.W; cv.height = map.H;
    const c = cv.getContext('2d'), cols = [th.g[0], th.road, th.rock, th.water, th.rough, '#8b6b45'];
    for (let y = 0; y < map.H; y++) for (let x = 0; x < map.W; x++) { c.fillStyle = cols[map.tiles[y * map.W + x]]; c.fillRect(x, y, 1, 1); }
    return cv;
  };

  function rr(c, x, y, w, h, r) { c.beginPath(); c.moveTo(x + r, y); c.arcTo(x + w, y, x + w, y + h, r); c.arcTo(x + w, y + h, x, y + h, r); c.arcTo(x, y + h, x, y, r); c.arcTo(x, y, x + w, y, r); c.closePath(); }

  const VEH = { jeep: [22, 13], light: [26, 18], medium: [30, 21], heavy: [35, 25], launcher: [29, 18] };
  ZG.drawUnit = function (c, u) {
    const d = u.def, t = u.t, main = col(u.team), dark = cold(u.team);
    const flash = u.hit > 0;
    c.save(); c.translate(u.x, u.y);
    c.fillStyle = 'rgba(0,0,0,0.28)';
    c.beginPath(); c.ellipse(2, 3, d.r, d.r * 0.8, 0, 0, 6.3); c.fill();
    if (d.cls === 'robot') {
      c.rotate(u.ang); c.scale(1.35, 1.35);
      const sw = u.moving ? Math.sin(u.walk) * 3 : 0;
      c.fillStyle = '#2b2b2b'; c.fillRect(-2 + sw, -6, 4, 3); c.fillRect(-2 - sw, 3, 4, 3);
      // weapon
      c.lineCap = 'butt';
      if (t === 'grunt') { c.fillStyle = '#333'; c.fillRect(2, 2, 8, 2); }
      else if (t === 'psycho') { c.fillStyle = '#333'; c.fillRect(2, 3, 8, 2); c.fillRect(2, -5, 8, 2); }
      else if (t === 'tough') { c.fillStyle = '#3f4a2c'; c.fillRect(-3, 3, 14, 4); c.fillStyle = '#222'; c.fillRect(10, 3, 2, 4); }
      else if (t === 'sniper') { c.fillStyle = '#222'; c.fillRect(1, 2, 14, 1.6); }
      else if (t === 'pyro') { c.fillStyle = '#e08a1e'; c.fillRect(-8, -3, 4, 6); c.fillStyle = '#333'; c.fillRect(2, 2, 7, 3); }
      else if (t === 'laser') { c.fillStyle = '#2c3e50'; c.fillRect(2, 2, 9, 3); c.fillStyle = '#5ff'; c.fillRect(10, 2, 3, 3); }
      c.fillStyle = flash ? '#fff' : main; c.strokeStyle = dark; c.lineWidth = 1.5;
      c.beginPath(); c.arc(0, 0, 5.2, 0, 6.3); c.fill(); c.stroke();
      c.fillStyle = '#d9dde0'; c.beginPath(); c.arc(1.5, 0, 3, 0, 6.3); c.fill();
      c.fillStyle = '#222'; c.fillRect(3, -2, 1.5, 4);
      if (u.gren > 0) { c.fillStyle = '#3d7a2a'; c.beginPath(); c.arc(-3.5, 0, 1.8, 0, 6.3); c.fill(); }
    } else if (d.cls === 'vehicle') {
      const s = VEH[t], L = s[0], Wd = s[1];
      c.rotate(u.ang);
      if (t === 'jeep') {
        c.fillStyle = '#1f1f1f';
        c.fillRect(-L / 2 + 1, -Wd / 2 - 2, 6, 3); c.fillRect(L / 2 - 7, -Wd / 2 - 2, 6, 3); c.fillRect(-L / 2 + 1, Wd / 2 - 1, 6, 3); c.fillRect(L / 2 - 7, Wd / 2 - 1, 6, 3);
        c.fillStyle = flash ? '#fff' : main; rr(c, -L / 2, -Wd / 2, L, Wd, 3); c.fill();
        c.fillStyle = dark; c.fillRect(-L / 2 + 2, -Wd / 2 + 2, 7, Wd - 4);
        c.fillStyle = '#9cc7d8'; c.fillRect(3, -Wd / 2 + 2, 3, Wd - 4);
      } else {
        c.fillStyle = '#242424'; c.fillRect(-L / 2, -Wd / 2, L, 5); c.fillRect(-L / 2, Wd / 2 - 5, L, 5);
        c.fillStyle = '#4a4a4a';
        const off = (u.walk * 1.5) % 5;
        for (let k = -L / 2 + off; k < L / 2 - 1; k += 5) { c.fillRect(k, -Wd / 2, 1.5, 5); c.fillRect(k, Wd / 2 - 5, 1.5, 5); }
        c.fillStyle = flash ? '#fff' : main; rr(c, -L / 2 + 1, -Wd / 2 + 3, L - 2, Wd - 6, 2); c.fill();
        c.fillStyle = dark; c.fillRect(-L / 2 + 2, -Wd / 2 + 4, 4, Wd - 8);
      }
      c.rotate(u.tang - u.ang);
      if (t === 'jeep') { c.fillStyle = '#2a2a2a'; c.fillRect(-2, -1.2, 11, 2.4); c.beginPath(); c.arc(-2, 0, 3, 0, 6.3); c.fill(); }
      else if (t === 'launcher') {
        c.fillStyle = '#3c3c3c'; c.fillRect(-11, -7, 20, 14);
        c.fillStyle = '#eee'; c.fillRect(-9, -5, 17, 3); c.fillRect(-9, 2, 17, 3);
        c.fillStyle = '#c33'; c.fillRect(6, -5, 3, 3); c.fillRect(6, 2, 3, 3);
      } else {
        const br = t === 'heavy' ? 9 : t === 'medium' ? 7.5 : 6.5, bl = t === 'heavy' ? 24 : t === 'medium' ? 21 : 17;
        c.fillStyle = '#2a2a2a';
        if (t === 'heavy') { c.fillRect(0, -4.5, bl, 3); c.fillRect(0, 1.5, bl, 3); } else c.fillRect(0, -1.6, bl, 3.2);
        c.fillStyle = dark; c.strokeStyle = '#1a1a1a'; c.lineWidth = 1;
        c.beginPath(); c.arc(0, 0, br, 0, 6.3); c.fill(); c.stroke();
        c.fillStyle = flash ? '#fff' : main; c.beginPath(); c.arc(0, 0, br - 3, 0, 6.3); c.fill();
      }
    } else {
      c.fillStyle = '#4b4b4b'; c.strokeStyle = '#222'; c.lineWidth = 1.5;
      c.beginPath(); for (let k = 0; k < 6; k++) { const a = k * Math.PI / 3 + 0.52; c.lineTo(Math.cos(a) * 13, Math.sin(a) * 13); } c.closePath(); c.fill(); c.stroke();
      c.fillStyle = flash ? '#fff' : main; c.beginPath(); c.arc(0, 0, 8, 0, 6.3); c.fill();
      c.rotate(u.tang);
      c.fillStyle = '#1f1f1f';
      if (t === 'gatling') { c.fillRect(0, -3.5, 15, 2); c.fillRect(0, -1, 16, 2); c.fillRect(0, 1.5, 15, 2); }
      else if (t === 'cannon') c.fillRect(0, -2.5, 20, 5);
      else { c.fillStyle = '#3c3c3c'; c.fillRect(-7, -7, 16, 14); c.fillStyle = '#eee'; c.fillRect(-5, -5, 14, 3); c.fillRect(-5, 2, 14, 3); }
      c.fillStyle = cold(u.team); c.beginPath(); c.arc(0, 0, 4, 0, 6.3); c.fill();
    }
    c.restore();
  };

  function star(c, x, y, r) {
    c.beginPath();
    for (let k = 0; k < 10; k++) { const a = -Math.PI / 2 + k * Math.PI / 5, q = k % 2 ? r * 0.45 : r; c.lineTo(x + Math.cos(a) * q, y + Math.sin(a) * q); }
    c.closePath(); c.fill();
  }

  function drawBuilding(c, b, time, selected) {
    const x = b.tx * TS, y = b.ty * TS, w = b.w * TS, h = b.h * TS, main = col(b.team), dark = cold(b.team);
    c.fillStyle = 'rgba(0,0,0,0.3)'; c.fillRect(x + 5, y + 6, w, h);
    if (b.kind === 'fort') {
      c.fillStyle = b.dead ? '#3a3a3a' : '#6f6b63'; c.fillRect(x, y, w, h);
      c.fillStyle = b.dead ? '#2a2a2a' : '#8d887e'; c.fillRect(x + 8, y + 8, w - 16, h - 16);
      if (!b.dead) {
        c.fillStyle = b.hit > 0 ? '#fff' : main; c.fillRect(x + 20, y + 20, w - 40, h - 44);
        c.fillStyle = dark; for (let k = x + 26; k < x + w - 26; k += 12) c.fillRect(k, y + 20, 4, h - 44);
        c.fillStyle = '#58544d'; c.strokeStyle = '#2f2c28'; c.lineWidth = 2;
        for (const p of [[x, y], [x + w - 22, y], [x, y + h - 22], [x + w - 22, y + h - 22]]) { c.fillRect(p[0], p[1], 22, 22); c.strokeRect(p[0], p[1], 22, 22); c.fillStyle = main; c.fillRect(p[0] + 7, p[1] + 7, 8, 8); c.fillStyle = '#58544d'; }
        c.fillStyle = '#1c1a17'; c.fillRect(x + w / 2 - 2, y + h - 16, 36, 16);
        // banner
        c.fillStyle = '#ddd'; c.fillRect(x + w / 2 - 1, y - 14, 2, 34);
        c.fillStyle = main; c.beginPath(); c.moveTo(x + w / 2 + 1, y - 14); c.lineTo(x + w / 2 + 22 + Math.sin(time * 4) * 2, y - 8); c.lineTo(x + w / 2 + 1, y - 2); c.fill();
      } else {
        c.fillStyle = '#111'; for (let k = 0; k < 7; k++) c.fillRect(x + 12 + (k * 37) % (w - 30), y + 10 + (k * 23) % (h - 24), 14, 9);
      }
      c.strokeStyle = '#2f2c28'; c.lineWidth = 2; c.strokeRect(x, y, w, h);
    } else {
      c.fillStyle = '#5f646b'; c.fillRect(x, y, w, h);
      c.fillStyle = '#7d838b'; c.fillRect(x + 4, y + 4, w - 8, h - 20);
      c.fillStyle = main; c.fillRect(x + 4, y + 4, w - 8, 7); c.fillRect(x + 4, y + h - 23, w - 8, 4);
      c.strokeStyle = 'rgba(0,0,0,0.25)'; c.lineWidth = 1;
      for (let k = x + 16; k < x + w - 8; k += 12) { c.beginPath(); c.moveTo(k, y + 12); c.lineTo(k, y + h - 24); c.stroke(); }
      if (b.kind === 'robot') {
        c.fillStyle = '#1c1c1c'; c.fillRect(x + w / 2 + 8, y + h - 14, 16, 14);
        c.fillStyle = '#333'; c.beginPath(); c.arc(x + 18, y + 26, 8, 0, 6.3); c.fill();
        c.fillStyle = b.team >= 0 && Math.sin(time * 6) > 0 ? '#ffe36b' : '#777'; c.beginPath(); c.arc(x + 18, y + 26, 4, 0, 6.3); c.fill();
      } else if (b.kind === 'vehicle') {
        c.fillStyle = '#1c1c1c'; c.fillRect(x + w / 2 - 10, y + h - 15, 52, 15);
        c.fillStyle = '#444'; for (let k = 0; k < 3; k++) c.fillRect(x + w / 2 - 10, y + h - 14 + k * 5, 52, 2);
        c.fillStyle = '#333'; c.fillRect(x + 10, y + 16, 9, 22);
        c.fillStyle = 'rgba(210,210,210,' + (b.team >= 0 ? 0.5 : 0) + ')'; c.beginPath(); c.arc(x + 14 + Math.sin(time * 2) * 3, y + 8 - (time * 8 % 12), 5, 0, 6.3); c.fill();
      } else {
        c.fillStyle = '#eee'; c.beginPath(); c.arc(x + w / 2, y + h / 2 - 4, 15, 0, 6.3); c.fill();
        c.fillStyle = '#2a9d55'; c.fillRect(x + w / 2 - 3.5, y + h / 2 - 15, 7, 22); c.fillRect(x + w / 2 - 11, y + h / 2 - 7.5, 22, 7);
        c.fillStyle = '#1c1c1c'; c.fillRect(x + w / 2 + 8, y + h - 14, 16, 14);
      }
      c.strokeStyle = '#2b2e33'; c.lineWidth = 2; c.strokeRect(x, y, w, h);
      if (b.kind !== 'repair') { c.fillStyle = '#ffd94a'; for (let k = 0; k < b.tech; k++) star(c, x + w - 9 - k * 11, y + 20, 5); }
    }
    if (selected) { c.strokeStyle = '#fff'; c.lineWidth = 2; c.setLineDash([6, 4]); c.strokeRect(x - 3, y - 3, w + 6, h + 6); c.setLineDash([]); }
    if (b.kind === 'fort' && !b.dead) bar(c, x + 6, y - 22, w - 12, b.hp / b.maxhp, 5);
    if (b.team >= 0 && b.prod && !b.dead) {
      c.fillStyle = 'rgba(0,0,0,0.6)'; c.fillRect(x, y + h + 2, w, 5);
      c.fillStyle = '#ffd94a'; c.fillRect(x + 1, y + h + 3, (w - 2) * b.progress, 3);
    }
  }

  function bar(c, x, y, w, f, h) {
    c.fillStyle = 'rgba(0,0,0,0.65)'; c.fillRect(x - 1, y - 1, w + 2, h + 2);
    c.fillStyle = f > 0.5 ? '#4bd04b' : f > 0.25 ? '#e6c12f' : '#e0432f'; c.fillRect(x, y, w * Math.max(0, f), h);
  }

  function drawFlag(c, s, time) {
    const x = s.flag.x, y = s.flag.y + 8, main = col(s.owner);
    c.fillStyle = 'rgba(0,0,0,0.3)'; c.beginPath(); c.ellipse(x + 3, y + 1, 7, 3, 0, 0, 6.3); c.fill();
    c.fillStyle = '#e8e8e8'; c.fillRect(x - 1, y - 26, 2.5, 27);
    const w1 = Math.sin(time * 5 + s.id) * 2.5, w2 = Math.cos(time * 5 + s.id) * 2.5;
    c.fillStyle = main; c.strokeStyle = 'rgba(0,0,0,0.5)'; c.lineWidth = 1;
    c.beginPath(); c.moveTo(x + 1.5, y - 26); c.quadraticCurveTo(x + 10, y - 28 + w1, x + 19, y - 24 + w2);
    c.lineTo(x + 19, y - 13 + w2); c.quadraticCurveTo(x + 10, y - 17 + w1, x + 1.5, y - 14); c.closePath(); c.fill(); c.stroke();
  }

  // view: {x,y,w,h} in world pixels. ui: {selB, drag, hover}
  ZG.draw = function (c, G, R, view, ui) {
    const map = G.map, time = G.time, sw = map.sw * TS, sh = map.sh * TS;
    c.drawImage(R.terrain, 0, 0);
    // sector ownership tint and borders
    for (const s of map.sectors) {
      if (s.owner < 0) continue;
      c.fillStyle = s.owner === 0 ? 'rgba(216,54,47,0.09)' : 'rgba(47,111,216,0.10)';
      c.fillRect(s.x0 * TS, s.y0 * TS, sw, sh);
    }
    c.strokeStyle = 'rgba(0,0,0,0.35)'; c.lineWidth = 2; c.setLineDash([10, 8]);
    c.beginPath();
    for (let k = 1; k < map.cols; k++) { c.moveTo(k * sw, 0); c.lineTo(k * sw, map.H * TS); }
    for (let k = 1; k < map.rows; k++) { c.moveTo(0, k * sh); c.lineTo(map.W * TS, k * sh); }
    c.stroke(); c.setLineDash([]);

    for (const b of map.buildings) drawBuilding(c, b, time, ui.selB === b);
    for (const s of map.sectors) if (s.flag) drawFlag(c, s, time);
    for (const k of map.crates) if (!k.taken) {
      c.fillStyle = 'rgba(0,0,0,0.3)'; c.fillRect(k.x - 8, k.y - 6, 20, 16);
      c.fillStyle = '#4c6b32'; c.fillRect(k.x - 10, k.y - 8, 20, 16);
      c.strokeStyle = '#26381a'; c.lineWidth = 1.5; c.strokeRect(k.x - 10, k.y - 8, 20, 16);
      c.fillStyle = '#e9d36a'; c.beginPath(); c.arc(k.x, k.y, 3.5, 0, 6.3); c.fill(); c.fillRect(k.x - 1, k.y - 6, 2, 3);
    }
    const sb = ui.selB;
    if (sb && sb.rally && sb.team === 0) {
      c.strokeStyle = 'rgba(255,217,74,0.8)'; c.lineWidth = 1.5; c.setLineDash([5, 5]);
      c.beginPath(); c.moveTo((sb.door.tx + 0.5) * TS, (sb.door.ty + 0.5) * TS); c.lineTo(sb.rally.x, sb.rally.y); c.stroke(); c.setLineDash([]);
      c.fillStyle = '#ffd94a'; c.beginPath(); c.arc(sb.rally.x, sb.rally.y, 5, 0, 6.3); c.fill();
    }

    const vis = u => u.x > view.x - 40 && u.y > view.y - 40 && u.x < view.x + view.w + 40 && u.y < view.y + view.h + 40;
    for (const u of G.units) if (u.sel && vis(u)) {
      c.strokeStyle = '#7dff7d'; c.lineWidth = 1.5; c.beginPath(); c.ellipse(u.x, u.y + 1, u.def.r + 4, u.def.r + 2, 0, 0, 6.3); c.stroke();
    }
    for (let pass = 0; pass < 2; pass++)
      for (const u of G.units) if ((u.def.cls === 'robot') === (pass === 1) && vis(u)) ZG.drawUnit(c, u);
    for (const u of G.units) if (vis(u) && (u.sel || u.hp < u.def.hp)) bar(c, u.x - 9, u.y - u.def.r - 8, 18, u.hp / u.def.hp, 2.5);

    for (const s of G.shots) {
      c.save(); c.translate(s.x, s.y); c.rotate(s.ang);
      if (s.k === 'grenade') { c.fillStyle = '#26401c'; c.beginPath(); c.arc(0, 0, 3, 0, 6.3); c.fill(); c.fillStyle = '#ffd36b'; c.fillRect(-1, -4, 2, 2); }
      else if (s.k === 'shell') { c.fillStyle = '#222'; c.fillRect(-4, -1.5, 8, 3); c.fillStyle = '#ffd36b'; c.fillRect(-7, -1, 3, 2); }
      else { c.fillStyle = '#eee'; c.fillRect(-5, -1.5, 10, 3); c.fillStyle = '#c33'; c.fillRect(3, -1.5, 3, 3); c.fillStyle = 'rgba(255,170,60,0.9)'; c.fillRect(-11, -1.5, 6, 3); c.fillStyle = 'rgba(200,200,200,0.4)'; c.fillRect(-22, -1, 11, 2); }
      c.restore();
    }
    for (const f of G.fx) {
      const p = f.t / f.life;
      if (f.k === 'line') { c.strokeStyle = 'rgba(255,240,170,' + (1 - p) + ')'; c.lineWidth = 1.2; c.beginPath(); c.moveTo(f.x, f.y); c.lineTo(f.x2, f.y2); c.stroke(); c.fillStyle = 'rgba(255,220,120,' + (1 - p) + ')'; c.fillRect(f.x - 2, f.y - 2, 4, 4); }
      else if (f.k === 'beam') { c.strokeStyle = 'rgba(120,255,255,' + (1 - p) + ')'; c.lineWidth = 3; c.beginPath(); c.moveTo(f.x, f.y); c.lineTo(f.x2, f.y2); c.stroke(); c.strokeStyle = 'rgba(255,255,255,' + (1 - p) + ')'; c.lineWidth = 1; c.stroke(); }
      else if (f.k === 'flame') {
        for (let k = 0; k < 5; k++) { const q = Math.min(1, p + k * 0.18), x = f.x + (f.x2 - f.x) * q, y = f.y + (f.y2 - f.y) * q; c.fillStyle = 'rgba(255,' + Math.floor(210 - 150 * q) + ',40,' + (0.7 * (1 - p)) + ')'; c.beginPath(); c.arc(x, y, 3 + 6 * q, 0, 6.3); c.fill(); }
      } else {
        c.fillStyle = 'rgba(255,' + Math.floor(230 - 170 * p) + ',' + Math.floor(120 * (1 - p)) + ',' + (1 - p) + ')'; c.beginPath(); c.arc(f.x, f.y, f.r * (0.3 + 0.7 * p), 0, 6.3); c.fill();
        c.fillStyle = 'rgba(60,60,60,' + 0.5 * (1 - p) + ')'; c.beginPath(); c.arc(f.x + 3, f.y - f.r * p, f.r * 0.5 * p, 0, 6.3); c.fill();
      }
    }
    if (ui.drag) { const d = ui.drag; c.strokeStyle = '#7dff7d'; c.lineWidth = 1; c.fillStyle = 'rgba(125,255,125,0.12)'; c.fillRect(d.x0, d.y0, d.x1 - d.x0, d.y1 - d.y0); c.strokeRect(d.x0, d.y0, d.x1 - d.x0, d.y1 - d.y0); }
    for (const m of ui.marks) { const p = m.t / 0.5; c.strokeStyle = (m.atk ? 'rgba(255,90,70,' : 'rgba(125,255,125,') + (1 - p) + ')'; c.lineWidth = 2; c.beginPath(); c.arc(m.x, m.y, 4 + 14 * p, 0, 6.3); c.stroke(); }
  };

  ZG.drawMini = function (c, G, R, view, cw, ch) {
    const map = G.map, sx = cw / (map.W * TS), sy = ch / (map.H * TS);
    c.imageSmoothingEnabled = false;
    c.drawImage(R.mini, 0, 0, cw, ch);
    for (const s of map.sectors) if (s.owner >= 0) { c.fillStyle = s.owner === 0 ? 'rgba(216,54,47,0.35)' : 'rgba(47,111,216,0.38)'; c.fillRect(s.x0 * TS * sx, s.y0 * TS * sy, map.sw * TS * sx, map.sh * TS * sy); }
    c.strokeStyle = 'rgba(0,0,0,0.4)'; c.lineWidth = 1; c.beginPath();
    for (let k = 1; k < map.cols; k++) { const x = Math.round(k * map.sw * TS * sx) + 0.5; c.moveTo(x, 0); c.lineTo(x, ch); }
    for (let k = 1; k < map.rows; k++) { const y = Math.round(k * map.sh * TS * sy) + 0.5; c.moveTo(0, y); c.lineTo(cw, y); }
    c.stroke();
    for (const b of map.buildings) { c.fillStyle = b.dead ? '#222' : col(b.team); c.fillRect(b.tx * TS * sx, b.ty * TS * sy, b.w * TS * sx, b.h * TS * sy); c.strokeStyle = '#000'; c.strokeRect(b.tx * TS * sx, b.ty * TS * sy, b.w * TS * sx, b.h * TS * sy); }
    for (const u of G.units) { c.fillStyle = u.team < 0 ? '#ddd' : u.team === 0 ? '#ff7a70' : '#7fb0ff'; const z = u.def.cls === 'robot' ? 2 : 3; c.fillRect(u.x * sx - z / 2, u.y * sy - z / 2, z, z); }
    c.strokeStyle = '#fff'; c.lineWidth = 1; c.strokeRect(view.x * sx + 0.5, view.y * sy + 0.5, view.w * sx, view.h * sy);
  };
})();
