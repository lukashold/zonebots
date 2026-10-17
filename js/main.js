// Zonebots - browser front end: input, command bar, menus, sound, main loop
(function () {
  const ZG = window.ZG, TS = ZG.TILE, A = ZG.ART, $ = id => document.getElementById(id);
  const cv = $('game'), ctx = cv.getContext('2d'), mini = $('mini'), mctx = mini.getContext('2d');
  const lo = document.createElement('canvas'), lctx = lo.getContext('2d');
  const store = {
    get(k, d) { try { const v = localStorage.getItem('zonebots.' + k); return v == null ? d : v; } catch (e) { return d; } },
    set(k, v) { try { localStorage.setItem('zonebots.' + k, v); } catch (e) { /* private mode */ } }
  };
  let lang = store.get('lang', (navigator.language || 'en').slice(0, 2) === 'de' ? 'de' : 'en');
  let unlocked = +store.get('unlocked', 0), sound = store.get('sound', '1') === '1', diff = +store.get('diff', 1), skDef = null;
  let classic = store.get('ctrl', '0') === '0', zUnlocked = +store.get('zunlocked', 0), zIdx = -1;
  let L = ZG.TEXT[lang];
  let G = null, R = null, cur = 0, paused = false, speed = 1, zoom = 2, dpr = 1;
  const cam = { x: 0, y: 0 }, view = { x: 0, y: 0, w: 0, h: 0 };
  const ui = { selB: null, drag: null, marks: [], rally: null, paths: [] };
  const mouse = { x: 0, y: 0, inside: false }, keys = {}, groups = {};
  let mode = null, panelSig = '', lastClick = { t: 0, type: null }, miniDrag = false;

  // ---------- sound (synthesised) ----------
  let ac = null, lastShot = 0;
  function tone(type, f0, f1, dur, vol) {
    if (!sound) return;
    try {
      ac = ac || new (window.AudioContext || window.webkitAudioContext)();
      const t = ac.currentTime, g = ac.createGain();
      g.gain.setValueAtTime(vol, t); g.gain.exponentialRampToValueAtTime(0.001, t + dur); g.connect(ac.destination);
      if (type === 'noise') {
        const n = Math.floor(ac.sampleRate * dur), buf = ac.createBuffer(1, n, ac.sampleRate), d = buf.getChannelData(0);
        for (let i = 0; i < n; i++) d[i] = Math.random() * 2 - 1;
        const s = ac.createBufferSource(), f = ac.createBiquadFilter();
        f.type = 'lowpass'; f.frequency.setValueAtTime(f0, t); f.frequency.exponentialRampToValueAtTime(f1, t + dur);
        s.buffer = buf; s.connect(f); f.connect(g); s.start(t);
      } else {
        const o = ac.createOscillator(); o.type = type;
        o.frequency.setValueAtTime(f0, t); o.frequency.exponentialRampToValueAtTime(f1, t + dur);
        o.connect(g); o.start(t); o.stop(t + dur);
      }
    } catch (e) { /* audio unavailable */ }
  }
  // original sound effects and music, when the player loaded their own Z files
  const SHOT = { sniper: 'RIFLE3', jeep: 'JEEPMGUN', gatling: 'GATTGUN', light: 'LTANKGUN', medium: 'MTANKGUN', cannon: 'MTANKGUN', heavy: 'HTANKGUN', howitzer: 'HTANKGUN' };
  const WSHOT = { bullet: 'MACHGUN2', laser: 'LASERGUN', flame: 'FLAMER', rocket: 'MOBIMIS2', missile: 'MOBIMISS', shell: 'MTANKGUN' };
  const sfxBuf = {};
  function zsfx(name, vol) {
    if (!sound || !ZG.Z.files) return false;
    const raw = ZG.Z.files[name.toLowerCase() + '.raw'];
    if (!raw) return false;
    try {
      ac = ac || new (window.AudioContext || window.webkitAudioContext)();
      let b = sfxBuf[name];
      if (!b) { b = sfxBuf[name] = ac.createBuffer(1, raw.length, 11025); const ch = b.getChannelData(0); for (let i = 0; i < raw.length; i++) ch[i] = (raw[i] - 128) / 128; }
      const s = ac.createBufferSource(), g = ac.createGain(); g.gain.value = vol == null ? 0.6 : vol;
      s.buffer = b; s.connect(g); g.connect(ac.destination); s.start();
      return true;
    } catch (e) { return false; }
  }
  // voices of the original: COMPxx announcements and ROBxx robot replies, numbered as in the game
  const VOICE = {
    yes: ['ROB01', 'ROB02', 'ROB03', 'ROB04', 'ROB05', 'ROB06'],
    report: { grunt: 'ROB07', psycho: 'ROB08', sniper: 'ROB09', tough: 'ROB10', laser: 'ROB11', pyro: 'ROB12' },
    go: ['ROB13', 'ROB14', 'ROB15', 'ROB16', 'ROB17', 'ROB18', 'ROB19', 'ROB20', 'ROB21', 'ROB22', 'ROB23', 'ROB24'],
    attacked: ['ROB25', 'ROB26', 'ROB27', 'ROB28', 'ROB29'],
    hit: ['ROB37', 'ROB40', 'ROB41', 'ROB42', 'ROB43', 'ROB44', 'ROB45', 'ROB46'],
    taunt: ['COMP10', 'COMP11', 'COMP13', 'COMP15', 'COMP17', 'COMP18']
  };
  const pick = a => a[Math.random() * a.length | 0];
  let voiceSrc = null, voiceUntil = 0;
  const vbuf = {};
  function voice(name, urgent) {
    if (!sound || !ZG.Z.files || !name) return;
    const f = ZG.Z.files, key = name.toLowerCase() + '.raw', raw = f[(lang === 'de' ? 'de/' : 'en/') + key] || f['en/' + key];
    if (!raw) return;
    const now = performance.now();
    if (now < voiceUntil && !urgent) return;
    try {
      ac = ac || new (window.AudioContext || window.webkitAudioContext)();
      const id = lang + name;
      let b = vbuf[id];
      if (!b) { b = vbuf[id] = ac.createBuffer(1, raw.length, 11025); const ch = b.getChannelData(0); for (let i = 0; i < raw.length; i++) ch[i] = (raw[i] - 128) / 128; }
      if (voiceSrc) try { voiceSrc.stop(); } catch (e) { /* already ended */ }
      const s = ac.createBufferSource(), g = ac.createGain(); g.gain.value = 0.9;
      s.buffer = b; s.connect(g); g.connect(ac.destination); s.start();
      voiceSrc = s; voiceUntil = now + b.duration * 1000 + 150;
    } catch (e) { /* audio unavailable */ }
  }
  const lastSaid = {};
  // the same kind of remark at most every few seconds
  function say(kind, name, gap, urgent) {
    const now = performance.now();
    if (lastSaid[kind] && now - lastSaid[kind] < gap * 1000) return;
    lastSaid[kind] = now; voice(name, urgent);
  }
  const music = new Audio(); music.loop = true; music.volume = 0.45;
  let musicName = '';
  function playMusic(name, loop) {
    const url = sound && ZG.Z.music(name);
    if (!url) { music.pause(); musicName = ''; return; }
    if (musicName === name && !music.paused) return;
    musicName = name; music.src = url; music.loop = loop !== false; music.play().catch(() => { musicName = ''; });
  }
  const gameMusic = () => ['aa16.ogg', 'ac16.ogg', 'aj16.ogg'][(zIdx >= 0 ? zIdx : Math.max(0, cur)) % 3];
  const onScreen = e => e.x > view.x - 100 && e.y > view.y - 100 && e.x < view.x + view.w + 100 && e.y < view.y + view.h + 100;

  // ---------- view ----------
  function resize() {
    dpr = window.devicePixelRatio || 1;
    cv.width = Math.floor(innerWidth * dpr); cv.height = Math.floor(innerHeight * dpr);
    if (G) sizeMini();
  }
  function sizeMini() {
    const mh = Math.min(300, innerHeight * 0.45), mw = Math.min(innerWidth * 0.5, Math.round(mh * G.map.W / G.map.H)), h = Math.round(mw * G.map.H / G.map.W);
    mini.style.width = mw + 'px'; mini.style.height = h + 'px';
    mini.width = Math.round(mw * dpr); mini.height = Math.round(h * dpr);
  }
  const scale = () => zoom * dpr; // device pixels per art pixel
  function clampCam() {
    const S = scale();
    view.w = cv.width / S * A; view.h = cv.height / S * A;
    const mw = G.map.W * TS, mh = G.map.H * TS;
    cam.x = view.w >= mw ? (mw - view.w) / 2 : ZG.clamp(cam.x, 0, mw - view.w);
    cam.y = view.h >= mh ? (mh - view.h) / 2 : ZG.clamp(cam.y, 0, mh - view.h);
    cam.x = Math.round(cam.x / A) * A; cam.y = Math.round(cam.y / A) * A;
    view.x = cam.x; view.y = cam.y;
  }
  const centerOn = (x, y) => { cam.x = x - view.w / 2; cam.y = y - view.h / 2; clampCam(); };
  const world = (cx, cy) => ({ x: cam.x + cx * dpr / scale() * A, y: cam.y + cy * dpr / scale() * A });

  // ---------- selection ----------
  const selUnits = () => G.units.filter(u => u.sel && !u.dead);
  const mineSel = () => selUnits().filter(u => u.team === 0);
  function clearSel() { for (const u of G.units) u.sel = false; ui.selB = null; }
  // first click: the whole squad; clicking a member of the selected squad again: just that robot
  function selectUnit(u, add) {
    const squad = ZG.squadOf(G, u), sel = mineSel();
    const wholeSquad = squad.length > 1 && u.sel && sel.length === squad.length && squad.every(v => v.sel);
    if (add) { const on = !u.sel; u.sel = on; ui.selB = null; return; }
    clearSel();
    if (wholeSquad) u.sel = true; else for (const v of squad) v.sel = true;
    ui.selB = null;
    say('select', VOICE.report[u.t] && Math.random() < 0.5 ? VOICE.report[u.t] : pick(VOICE.yes), 0, true);
  }
  // units sent away from their squad form a squad of their own
  function splitSquads(us) {
    const by = {};
    for (const u of us) if (u.squad) (by[u.squad] = by[u.squad] || []).push(u);
    for (const k in by) {
      const part = by[k];
      if (part.length < ZG.squadOf(G, part[0]).length) { const id = G.nid++; for (const u of part) u.squad = id; }
    }
  }
  function unitAt(p, pred) {
    let best = null, bd = 1e9;
    for (const u of G.units) {
      if (u.dead || !pred(u)) continue;
      const oy = u.def.cls === 'robot' ? -8 : 0, d = Math.hypot(u.x - p.x, u.y + oy - p.y);
      if (d < u.def.r + 8 && d < bd) { bd = d; best = u; }
    }
    return best;
  }
  const buildingAt = p => G.map.buildings.find(b => p.x >= b.tx * TS && p.y >= b.ty * TS - 20 && p.x < (b.tx + b.w) * TS && p.y < (b.ty + b.h) * TS);
  function selOnScreen(type) {
    ui.selB = null;
    for (const v of G.units) if (v.team === 0 && v.def.sp && (!type || v.t === type) && v.x > view.x && v.y > view.y && v.x < view.x + view.w && v.y < view.y + view.h) v.sel = true;
  }
  // messages appear on the plaque under the clock, like in the original
  let msgTimer = 0;
  function toast(msg, cls) {
    const m = $('msgBox'); m.textContent = msg; m.className = 'plaque ' + (cls || '');
    clearTimeout(msgTimer); msgTimer = setTimeout(() => m.classList.add('fade'), 2600);
  }

  // ---------- orders ----------
  function order(p, attackMode) {
    const us = mineSel();
    if (!us.length) return false;
    splitSquads(us);
    const robots = us.some(u => u.def.cls === 'robot'), cranes = us.filter(u => u.def.crane);
    const enemy = unitAt(p, u => u.team === 1);
    const apc = unitAt(p, u => u.team === 0 && u.def.cargo && !u.sel);
    const empty = unitAt(p, u => u.team < 0);
    const st = ZG.structAt(G, p.x, p.y);
    let atk = true;
    if (enemy) ZG.orderAttack(G, us, enemy);
    else if (apc && robots) { ZG.orderBoard(G, us, apc); ZG.orderMove(G, us.filter(u => u.def.cls !== 'robot'), apc.x, apc.y); atk = false; }
    else if (empty && robots) { ZG.orderEnter(G, us, empty); atk = false; }
    else if (st && st.kind === 'b' && st.ref.team === 1 && !st.ref.dead) ZG.orderStruct(G, us, st);
    else if (cranes.length && st && ((st.kind === 'b' && st.ref.team === 0 && !st.ref.dead && st.ref.hp < st.ref.maxhp) || (st.kind === 'br' && st.ref.dead))) {
      ZG.orderRepair(G, cranes, st); ZG.orderMove(G, us.filter(u => !u.def.crane), st.x, st.y); atk = false;
    } else if (st && (st.kind === 'rock' || (attackMode && st.kind === 'br' && !st.ref.dead))) {
      ZG.orderStruct(G, us, st); ZG.orderMove(G, us.filter(u => !u.order || u.order.k !== 'hit'), st.x, st.y);
    } else { atk = attackMode; ZG.orderMove(G, us, p.x, p.y, attackMode ? 'amove' : 'move'); }
    ui.marks.push({ x: p.x, y: p.y, t: 0, atk: atk });
    // show the routes as dotted lines for a moment
    ui.paths = us.filter(u => u.path).slice(0, 12).map(u => ({ pts: [{ x: u.x, y: u.y }].concat(u.path), t: 0 }));
    if (!zsfx('CLICK1L', 0.5)) tone('square', 520, 620, 0.05, 0.04);
    say('order', st && st.kind === 'b' && st.ref.kind === 'fort' && st.ref.team === 1 ? 'ROB54' : pick(VOICE.go), 0, true);
    return true;
  }
  function setRally(p) {
    const b = ui.selB;
    if (b && b.team === 0 && !b.dead && ZG.products(b).length) { b.rally = { x: p.x, y: p.y }; ui.marks.push({ x: p.x, y: p.y, t: 0, atk: false }); toast(L.rally); }
  }
  // mouse pointer: the original arrow and cross-hairs when the player's Z files are loaded
  function pointer(p) {
    if (!R || !R.zs) return mode ? 'crosshair' : 'default';
    if (mode) return R.zs.cursor(5, 8, 8) || 'crosshair';
    if (p && mineSel().length) {
      const enemy = unitAt(p, u => u.team === 1);
      if (enemy) return R.zs.cursor(5, 8, 8) || 'crosshair';
      return R.zs.cursor(4, 8, 8) || 'default';
    }
    return R.zs.cursor(0, 0, 0) || 'default';
  }
  function setMode(m) {
    mode = m;
    $('mode').classList.toggle('hidden', !m);
    $('mode').textContent = m === 'attack' ? (lang === 'de' ? 'Angriff: Ziel anklicken' : 'Attack: click target') : m === 'rally' ? (lang === 'de' ? 'Sammelpunkt: Ort anklicken' : 'Rally point: click a spot') : '';
    cv.style.cursor = pointer(null);
    panelSig = '';
  }
  // a plain click on the map (no drag)
  function click(p, shift) {
    if (mode === 'attack') { order(p, true); setMode(null); return; }
    if (mode === 'rally') { setRally(p); setMode(null); return; }
    const own = unitAt(p, v => v.team === 0 && v.def.sp) || unitAt(p, v => v.team === 0);
    if (own && !(classic && mineSel().length && own.def.cargo && !own.sel && mineSel().some(u => u.def.cls === 'robot'))) {
      const now = performance.now();
      if (now - lastClick.t < 350 && lastClick.type === own.t) selOnScreen(own.t); else selectUnit(own, shift);
      lastClick = { t: now, type: own.t };
      return;
    }
    if (classic && mineSel().length) { order(p, false); return; }
    const b = buildingAt(p);
    if (b) { clearSel(); ui.selB = b; return; }
    if (!shift) clearSel();
  }

  // ---------- mouse ----------
  cv.addEventListener('contextmenu', e => e.preventDefault());
  cv.addEventListener('mousedown', function (e) {
    if (!G || paused || G.over) return;
    const p = world(e.clientX, e.clientY);
    if (e.button === 0) ui.drag = { x0: p.x, y0: p.y, x1: p.x, y1: p.y, sx: e.clientX, sy: e.clientY, shift: e.shiftKey };
    else if (e.button === 2) {
      if (mode) { setMode(null); return; }
      if (classic) { clearSel(); return; }
      if (mineSel().length) order(p, e.ctrlKey || e.altKey || e.metaKey);
      else setRally(p);
    }
  });
  window.addEventListener('mousemove', function (e) {
    mouse.x = e.clientX; mouse.y = e.clientY; mouse.inside = true;
    if (G && e.target === cv) { const cur = pointer(world(e.clientX, e.clientY)); if (cv.style.cursor !== cur) cv.style.cursor = cur; }
    if (G && ui.drag) { const p = world(e.clientX, e.clientY); ui.drag.x1 = p.x; ui.drag.y1 = p.y; }
    if (G && miniDrag) miniTo(e);
  });
  document.addEventListener('mouseleave', () => { mouse.inside = false; mouse.left = true; });
  document.addEventListener('mouseenter', () => { mouse.left = false; });
  window.addEventListener('blur', () => { mouse.inside = false; mouse.left = false; for (const k in keys) keys[k] = false; });
  window.addEventListener('mouseup', function (e) {
    miniDrag = false;
    if (!G || !ui.drag || e.button !== 0) return;
    const d = ui.drag; ui.drag = null;
    if (Math.hypot(e.clientX - d.sx, e.clientY - d.sy) < 6) { click({ x: d.x1, y: d.y1 }, d.shift); return; }
    const x0 = Math.min(d.x0, d.x1), x1 = Math.max(d.x0, d.x1), y0 = Math.min(d.y0, d.y1), y1 = Math.max(d.y0, d.y1);
    const hit = G.units.filter(u => u.team === 0 && u.def.sp && u.x >= x0 && u.x <= x1 && u.y >= y0 && u.y <= y1);
    if (!hit.length) return;
    if (!d.shift) clearSel();
    for (const u of hit) u.sel = true;
    ui.selB = null;
  });
  cv.addEventListener('wheel', function (e) {
    if (!G) return;
    e.preventDefault();
    const p = world(e.clientX, e.clientY), z = ZG.clamp(zoom + (e.deltaY < 0 ? 1 : -1), 1, 5);
    if (z === zoom) return;
    zoom = z; const q = world(e.clientX, e.clientY); cam.x += p.x - q.x; cam.y += p.y - q.y; clampCam();
  }, { passive: false });

  // ---------- touch: tap = like a click, drag = scroll, pinch = zoom ----------
  let touch = null;
  cv.addEventListener('touchstart', function (e) {
    if (!G || paused || G.over) return;
    e.preventDefault();
    const t = e.touches;
    if (t.length === 1) touch = { x: t[0].clientX, y: t[0].clientY, sx: t[0].clientX, sy: t[0].clientY, moved: false, pinch: 0 };
    else if (t.length === 2) touch = { moved: true, pinch: Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY), z: zoom };
  }, { passive: false });
  cv.addEventListener('touchmove', function (e) {
    if (!G || !touch) return;
    e.preventDefault();
    const t = e.touches;
    if (t.length === 2 && touch.pinch) {
      const d = Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY), cx = (t[0].clientX + t[1].clientX) / 2, cy = (t[0].clientY + t[1].clientY) / 2;
      const p = world(cx, cy), z = ZG.clamp(Math.round(touch.z * d / touch.pinch), 1, 5);
      if (z !== zoom) { zoom = z; const q = world(cx, cy); cam.x += p.x - q.x; cam.y += p.y - q.y; clampCam(); }
    } else if (t.length === 1 && !touch.pinch) {
      if (!touch.moved && Math.hypot(t[0].clientX - touch.sx, t[0].clientY - touch.sy) > 10) touch.moved = true;
      if (touch.moved) { const f = dpr / scale() * A; cam.x -= (t[0].clientX - touch.x) * f; cam.y -= (t[0].clientY - touch.y) * f; clampCam(); }
      touch.x = t[0].clientX; touch.y = t[0].clientY;
    }
  }, { passive: false });
  cv.addEventListener('touchend', function (e) {
    if (!G || !touch || e.touches.length) return;
    e.preventDefault();
    if (!touch.moved) {
      const was = classic; classic = true; // touch always works like the classic one-button scheme
      click(world(touch.sx, touch.sy), false);
      classic = was;
    }
    touch = null;
  }, { passive: false });

  // ---------- minimap ----------
  function miniPos(e) { const r = mini.getBoundingClientRect(); return { x: (e.clientX - r.left) / r.width * G.map.W * TS, y: (e.clientY - r.top) / r.height * G.map.H * TS }; }
  function miniTo(e) { const p = miniPos(e); centerOn(p.x, p.y); }
  mini.addEventListener('mousedown', function (e) {
    if (!G) return;
    e.preventDefault();
    if (e.button === 2 || (classic && e.button === 0 && mineSel().length && e.shiftKey)) order(miniPos(e), e.ctrlKey);
    else { miniDrag = true; miniTo(e); }
  });
  mini.addEventListener('contextmenu', e => e.preventDefault());

  // ---------- keys ----------
  window.addEventListener('keydown', function (e) {
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    if (!G || !$('menu').classList.contains('hidden')) return;
    if (k === 'Escape' || k === 'p') { if (!$('help').classList.contains('hidden')) { $('help').classList.add('hidden'); return; } if (mode) setMode(null); else if (!G.over) setPause(!paused); return; }
    if (paused || G.over) return;
    if (e.ctrlKey && k === 'a') { e.preventDefault(); clearSel(); for (const u of G.units) if (u.team === 0 && u.def.sp) u.sel = true; return; }
    if (k >= '1' && k <= '9') {
      e.preventDefault();
      if (e.ctrlKey || e.metaKey) groups[k] = mineSel();
      else if (groups[k]) {
        const g = groups[k].filter(u => !u.dead && u.team === 0);
        if (g.length) { const twice = g.every(u => u.sel) && mineSel().length === g.length; clearSel(); for (const u of g) u.sel = true; if (twice) centerOn(g[0].x, g[0].y); }
      }
      return;
    }
    if (e.ctrlKey || e.metaKey) return;
    if (k === 'f') { if (mineSel().length) setMode(mode === 'attack' ? null : 'attack'); return; }
    if (k === 'x') { ZG.orderStop(G, mineSel()); return; }
    if (k === 'u') { ZG.orderUnload(G, mineSel().filter(u => u.cargo)); return; }
    if (k === ' ') { e.preventDefault(); const f = G.map.buildings.find(b => b.kind === 'fort' && b.team === 0); centerOn(f.cx, f.cy + 40); return; }
    keys[k] = true;
    if (k.indexOf('Arrow') === 0) e.preventDefault();
  });
  window.addEventListener('keyup', e => { keys[e.key.length === 1 ? e.key.toLowerCase() : e.key] = false; });
  window.addEventListener('resize', function () { resize(); if (G) clampCam(); });

  // ---------- building panel and unit actions ----------
  const portCtx = $('bpPort').getContext('2d');
  // picture of a unit type: the original sprite when the player loaded Z, our own pixel art otherwise
  function unitPicture(t) {
    const d = ZG.UNITS[t];
    if (R && R.zs) {
      const L = R.zs.unit({ t: t, def: d, team: 0, ang: Math.PI / 2, tang: Math.PI / 2, moving: false, walk: 0, cd: 0, tgt: null });
      if (L) {
        const c = document.createElement('canvas'); c.width = 40; c.height = 36;
        const x = c.getContext('2d');
        for (const l of L) x.drawImage(l[0], 20 + l[1], (d.cls === 'robot' ? 30 : 18) + l[2]);
        return c;
      }
    }
    return ZG.unitIcon(t, 0);
  }
  function actBtn(label, fn, on) {
    const b = document.createElement('button'); b.textContent = label; if (on) b.className = 'on';
    b.onclick = fn; $('acts').appendChild(b);
  }
  function cycleProd(b, step) {
    const list = ZG.products(b);
    if (!list.length) return;
    const i = list.indexOf(b.prod);
    b.prod = list[((i < 0 ? 0 : i) + step + list.length) % list.length]; b.progress = 0; b.paused = false;
    say('prod', 'COMP04', 1, true);
    panelSig = '';
  }
  $('bpUp').onclick = () => ui.selB && cycleProd(ui.selB, -1);
  $('bpDown').onclick = () => ui.selB && cycleProd(ui.selB, 1);
  $('bpOk').onclick = () => { ui.selB = null; };
  $('bpRally').onclick = () => setMode(mode === 'rally' ? null : 'rally');
  $('bpCancel').onclick = function () {
    const b = ui.selB; if (!b || b.team !== 0) return;
    // stop or resume production; the chosen product is kept
    if (b.prod) { b.held = b.prod; b.prod = null; b.progress = 0; say('prod', 'COMP05', 0, true); } else { b.prod = b.held || ZG.products(b)[0]; say('prod', 'COMP04', 0, true); }
    panelSig = '';
  };
  function updatePanel() {
    const b = ui.selB, us = mineSel();
    const sig = (b ? b.id + ':' + b.team + ':' + b.prod + ':' + b.dead : '') + '|' + us.map(u => u.id + ':' + (u.cargo ? u.cargo.length : '')).join(',') + lang + mode + (R && R.zs ? 'z' : '');
    if (sig !== panelSig) {
      panelSig = sig;
      $('acts').innerHTML = '';
      if (us.length) {
        actBtn(L.stop, () => ZG.orderStop(G, mineSel()));
        actBtn(L.amoveBtn, () => setMode(mode === 'attack' ? null : 'attack'), mode === 'attack');
        if (us.some(u => u.cargo && u.cargo.length)) actBtn(L.unload, () => ZG.orderUnload(G, mineSel().filter(u => u.cargo)));
        actBtn(L.selNone, () => clearSel());
      }
      $('bpanel').classList.toggle('hidden', !b);
      if (b) {
        const mine = b.team === 0 && !b.dead, list = mine ? ZG.products(b) : [], shown = b.prod || b.held || (list[0] || null);
        $('bpName').textContent = L.b[b.kind];
        $('bpStars').textContent = b.kind === 'repair' || b.kind === 'radar' ? '' : '★'.repeat(b.tech);
        for (const id of ['bpUp', 'bpDown', 'bpRally', 'bpCancel']) $(id).classList.toggle('hidden', !list.length);
        $('bpRally').textContent = L.rally.split(' ')[0]; $('bpRally').className = mode === 'rally' ? 'on' : '';
        $('bpCancel').textContent = b.prod ? L.cancel : L.resume;
        portCtx.imageSmoothingEnabled = false;
        portCtx.fillStyle = '#1d2420'; portCtx.fillRect(0, 0, 40, 36);
        if (shown) {
          const img = unitPicture(shown), f = Math.min(36 / img.width, 34 / img.height);
          portCtx.drawImage(img, (40 - img.width * f) / 2, (36 - img.height * f) / 2, img.width * f, img.height * f);
        }
        $('bpUnit').textContent = shown ? L.units[shown] : (b.dead ? L.destroyed : b.team < 0 ? L.neutral : b.team === 1 ? L.enemy : L.b[b.kind]);
        $('bpCount').textContent = shown && ZG.UNITS[shown].n > 1 ? ZG.UNITS[shown].n : '';
      }
    }
    if (b) {
      $('bpPct').textContent = (b.dead ? 0 : Math.round(b.hp / b.maxhp * 100)) + '%';
      let tt = '';
      if (!b.dead && b.team === 0 && b.prod) {
        const left = Math.ceil((1 - b.progress) * ZG.UNITS[b.prod].time / (ZG.prodRate(G, 0) * (0.4 + 0.6 * b.hp / b.maxhp)));
        tt = L.time + ' ' + Math.floor(left / 60) + ':' + String(left % 60).padStart(2, '0');
      } else if (b.kind === 'repair') tt = '';
      if ($('bpTime').textContent !== tt) $('bpTime').textContent = tt;
      // keep the panel next to its building, inside the window
      const S = scale() / dpr, p = $('bpanel'), w = p.offsetWidth, h = p.offsetHeight;
      let x = ((b.tx + b.w) * TS - cam.x) / A * S + 8, y = (b.ty * TS - cam.y) / A * S;
      if (x + w > innerWidth - 8) x = (b.tx * TS - cam.x) / A * S - w - 8;
      x = ZG.clamp(x, 8, innerWidth - w - 8); y = ZG.clamp(y, 34, innerHeight - h - 60);
      p.style.left = Math.round(x) + 'px'; p.style.top = Math.round(y) + 'px';
    }
  }
  function updateSide() {
    const n = G.map.sectors.length; let a = 0, b = 0;
    for (const s of G.map.sectors) { if (s.owner === 0) a++; else if (s.owner === 1) b++; }
    if (b > a + 1 && G.time > 60 && !paused && !G.over) say('taunt', pick(VOICE.taunt), 120);
    $('terrR').style.width = (a / n * 100) + '%'; $('terrB').style.width = (b / n * 100) + '%';
    const tt = a + ':' + b; if ($('terrTxt').textContent !== tt) $('terrTxt').textContent = tt;
    const t = Math.floor(G.time), ct = Math.floor(t / 3600) + ':' + String(Math.floor(t / 60) % 60).padStart(2, '0') + ':' + String(t % 60).padStart(2, '0');
    if ($('clock').textContent !== ct) $('clock').textContent = ct;
  }
  $('miniBtn').onclick = function () { $('miniPanel').classList.toggle('hidden'); if (G) sizeMini(); };

  function handleEvents() {
    for (const e of G.ev) {
      if (e.k === 'shot') {
        const now = performance.now();
        if (now - lastShot > 55 && onScreen(e)) {
          lastShot = now;
          if (zsfx(SHOT[e.t] || WSHOT[e.w] || 'MACHGUN2', 0.35)) { /* original sound */ }
          else if (e.w === 'laser') tone('sawtooth', 1400, 300, 0.12, 0.03);
          else if (e.w === 'flame') tone('noise', 900, 300, 0.12, 0.03);
          else if (e.w === 'bullet') tone('noise', 3000, 900, 0.05, 0.05);
          else tone('noise', 700, 120, 0.22, 0.09);
        }
      } else if (e.k === 'boom') {
        const n = e.big ? ['ZEXPLO3', 'ZEXPLO4', 'ZEXPLO5'][Math.random() * 3 | 0] : (Math.random() < 0.5 ? 'EXP1' : 'EXP2');
        if (onScreen(e) && !zsfx(n, e.big ? 0.8 : 0.5)) tone('noise', e.big ? 500 : 900, 50, e.big ? 0.7 : 0.3, e.big ? 0.22 : 0.1);
      }
      else if (e.k === 'kill' && e.team === 0) say('hit', pick(VOICE.hit), 7);
      else if (e.k === 'hurt' && e.team === 0) say('attacked', pick(VOICE.attacked), 25);
      else if (e.k === 'capture') {
        if (e.team === 0) { toast(L.captured, 'good'); tone('triangle', 520, 1040, 0.25, 0.1); say('capture', e.radar ? 'COMP09' : 'ROB49', 0, true); }
        else if (e.prev === 0) { toast(L.lost, 'bad'); tone('triangle', 400, 160, 0.3, 0.1); say('lost', 'COMP08', 0, true); }
      } else if (e.k === 'ready' && e.team === 0) {
        toast(L.units[e.t] + ' ' + L.ready); if (!zsfx('ROBFACT5', 0.5)) tone('sine', 880, 1320, 0.12, 0.06);
        const cls = ZG.UNITS[e.t].cls; say('ready', cls === 'robot' ? 'COMP02' : cls === 'gun' ? 'COMP03' : 'COMP01', 0);
      }
      else if (e.k === 'alert') { toast(L.alert, 'bad'); tone('square', 660, 440, 0.4, 0.08); say('alert', e.fort ? 'COMP16' : pick(VOICE.attacked), 0, true); }
      else if (e.k === 'enter' && e.team === 0) { toast(L.crewed, 'good'); say('enter', e.gun ? 'ROB51' : 'ROB52', 0, true); }
      else if (e.k === 'crate' && e.team === 0) { toast(L.crate, 'good'); tone('sine', 660, 990, 0.15, 0.07); say('crate', 'ROB53', 0, true); }
      else if (e.k === 'driver' && e.team === 0) toast(L.driver, 'good');
      else if (e.k === 'bridge') { toast(e.up ? L.bridgeUp : L.bridgeDown, e.up ? 'good' : ''); say('bridge', e.up ? 'ROB47' : 'ROB38', 0, true); }
      else if (e.k === 'bdead') { zsfx('OBJDEST3', 0.8); toast(e.team === 0 ? L.ourBldDead : L.bldDead, e.team === 0 ? 'bad' : 'good'); if (e.team !== 0) say('bdead', 'ROB39', 0, true); }
      else if (e.k === 'stormed') toast(L.stormed, e.team === 0 ? 'good' : 'bad');
      else if (e.k === 'board' && e.team === 0) tone('sine', 500, 700, 0.08, 0.05);
      else if (e.k === 'unload' && e.team === 0) toast(L.unloaded);
      else if (e.k === 'fixed' && e.team === 0) { toast(L.fixed, 'good'); say('fixed', 'ROB48', 0, true); }
      else if (e.k === 'rock') { toast(L.rockGone); if (onScreen(e)) tone('noise', 400, 40, 0.6, 0.18); }
      else if (e.k === 'over') setTimeout(showEnd, 1300);
    }
    G.ev.length = 0;
  }

  // ---------- screens ----------
  function setPause(v) { paused = v; $('pause').classList.toggle('hidden', !v); }
  function showEnd() {
    if (!G || !G.over) return;
    const orig = zIdx >= 0, win = G.over.winner === 0, last = orig ? zIdx === ZG.Z.levels.length - 1 : cur === ZG.LEVELS.length - 1, camp = cur >= 0;
    if (win && camp && cur + 1 > unlocked) { unlocked = Math.min(cur + 1, ZG.LEVELS.length); store.set('unlocked', unlocked); }
    if (win && orig && zIdx + 1 > zUnlocked) { zUnlocked = zIdx + 1; store.set('zunlocked', zUnlocked); }
    let bestTxt = '';
    if (win && camp) {
      const key = 'best' + cur + 'd' + diff, old = +store.get(key, 0), now = Math.floor(G.over.time);
      if (!old || now < old) { store.set(key, now); bestTxt = L.newBest; } else bestTxt = L.best + ': ' + fmt(old);
    }
    $('endTitle').textContent = win ? L.win : L.lose;
    $('endTitle').style.color = win ? '#3ed43e' : '#ee3a24';
    $('endSub').textContent = win && last ? L.allDone : levelName(cur);
    const t = Math.floor(G.over.time);
    $('endStats').innerHTML = '<span>' + L.time + '</span><b>' + fmt(t) + '</b><span>' + L.diffLabel + '</span><b>' + L.diff[diff] + '</b><span>' + L.kills + '</span><b>' + G.stats[0].kills + '</b><span>' + L.losses + '</span><b>' + G.stats[0].losses + '</b>';
    $('endBest').textContent = bestTxt;
    $('btnNext').classList.toggle('hidden', !win || last || !(camp || orig));
    $('btnRetry').classList.toggle('primary', !win);
    $('end').classList.remove('hidden');
    playMusic(win ? 'ipwin.ogg' : 'iplose.ogg', false);
    voice(win ? 'ROB65' : 'COMP12', true);
    if (win) { tone('triangle', 523, 523, 0.15, 0.1); setTimeout(() => tone('triangle', 784, 784, 0.3, 0.1), 160); } else tone('sawtooth', 220, 80, 0.8, 0.08);
  }
  const fmt = t => Math.floor(t / 60) + ':' + String(t % 60).padStart(2, '0');
  const levelName = i => i < 0 ? (skDef.original ? 'Z ' + skDef.zlevel + ' · ' + skDef.zname : L.skirmish + ' · ' + L.worlds[skDef.theme] + ' #' + skDef.seed) : L.worlds[ZG.LEVELS[i].theme] + ' · ' + L.level + ' ' + (i + 1);

  function start(i, def) {
    cur = def ? -1 : i; skDef = def || null; zIdx = def && def.original ? i : -1;
    G = ZG.newGame(def || i, { diff: diff });
    const th = ZG.THEMES[G.lv.theme];
    if (G.lv.original) { const t = ZG.Z.terrainCanvas(G.map); R = { terrain: t, mini: ZG.Z.miniCanvas(G.map, t), th: th, zs: ZG.Z.spriteSet(G.map) }; }
    else R = { terrain: ZG.renderTerrain(G.map, th, G.lv.seed), mini: ZG.renderMini(G.map, th), th: th };
    ui.selB = null; ui.drag = null; ui.marks = []; ui.paths = []; panelSig = ''; paused = false; setMode(null);
    $('miniPanel').classList.add('hidden');
    for (const k in groups) delete groups[k];
    zoom = innerWidth > 2600 ? 4 : innerWidth > 1500 ? 3 : 2;
    for (const id of ['menu', 'end', 'pause', 'help']) $(id).classList.add('hidden');
    $('hud').classList.remove('hidden');
    $('lvTitle').textContent = levelName(cur);
    playMusic(gameMusic());
    resize(); clampCam();
    const f = G.map.buildings.find(b => b.kind === 'fort' && b.team === 0);
    centerOn(f.cx, f.cy + 40);
  }
  function showPanel(id) { for (const p of ['mMain', 'mPlay', 'mOrig', 'mSkirm', 'mOpts']) $(p).classList.toggle('hidden', p !== id); }
  for (const b of document.querySelectorAll('[data-go]')) b.addEventListener('click', () => showPanel(b.dataset.go));
  function showMenu() {
    showPanel('mMain');
    playMusic('ipoptions16.ogg');
    G = null; $('hud').classList.add('hidden');
    for (const id of ['end', 'pause', 'help']) $(id).classList.add('hidden');
    $('menu').classList.remove('hidden');
    buildMenu();
  }
  function buildMenu() {
    L = ZG.TEXT[lang]; document.documentElement.lang = lang;
    $('tagline').textContent = L.sub;
    $('miPlay').textContent = L.mPlay; $('miOrig').textContent = L.mOrig; $('miSkirm').textContent = L.mSkirm; $('miOpts').textContent = L.mOpts;
    $('optDiffTitle').textContent = L.diffLabel; $('langTitle').textContent = L.langTitle;
    for (const b of document.querySelectorAll('.mback')) b.textContent = L.back;
    // with the player's own Z files the menu uses the original background and panel plate
    const bg = ZG.Z.image('background.png'), plate = ZG.Z.image('boxcentrewide.png');
    for (const el of document.querySelectorAll('.screen')) el.style.backgroundImage = bg ? 'url(' + bg + ')' : '';
    for (const el of document.querySelectorAll('.box')) { el.classList.toggle('zskin', !!plate); el.style.backgroundImage = plate ? 'url(' + plate + ')' : ''; }
    $('btnHelp').textContent = L.help; $('btnLang').textContent = L.lang; $('helpTitle').textContent = L.help;
    $('btnHelpBack').textContent = L.back; $('helpBody').innerHTML = L.helpText.map(t => '<p>' + t + '</p>').join('');
    $('pauseTitle').textContent = L.paused; $('btnResume').textContent = L.resume; $('btnRestart').textContent = L.retry;
    $('btnPauseHelp').textContent = L.help; $('btnQuit').textContent = L.menu;
    $('btnNext').textContent = L.next; $('btnRetry').textContent = L.retry; $('btnEndMenu').textContent = L.menu;
    const box = $('worlds'); box.innerHTML = '';
    for (let w = 0; w < 5; w++) {
      const row = document.createElement('div'); row.className = 'world';
      const th = ZG.THEMES[w], nm = document.createElement('div'); nm.className = 'wn';
      nm.innerHTML = '<span class="sw" style="background:linear-gradient(135deg,' + th.g[0] + ' 50%,' + th.water + ' 50%)"></span>';
      nm.appendChild(document.createTextNode(L.worlds[w]));
      row.appendChild(nm);
      for (let k = 0; k < 4; k++) {
        const i = w * 4 + k, b = document.createElement('button');
        b.className = 'lvl' + (i < unlocked ? ' done' : '') + (i === unlocked ? ' primary' : '');
        const bt = +store.get('best' + i + 'd' + diff, 0);
        b.textContent = i + 1; b.disabled = i > unlocked; b.title = i > unlocked ? L.locked : levelName(i) + (bt ? ' · ' + L.best + ' ' + fmt(bt) : '');
        b.onclick = () => start(i);
        row.appendChild(b);
      }
      box.appendChild(row);
    }
    $('campTitle').textContent = L.campaign; $('skTitle').textContent = L.skirmish; $('diffLabel').textContent = L.diffLabel;
    const seg = (id, names, curIdx, fn) => {
      const db = $(id); db.innerHTML = '';
      names.forEach(function (name, k) { const b = document.createElement('button'); b.textContent = name; if (k === curIdx) b.className = 'primary'; b.onclick = () => fn(k); db.appendChild(b); });
    };
    seg('diffBtns', L.diff, diff, k => { diff = k; store.set('diff', k); buildMenu(); });
    $('zTitle').textContent = L.zTitle; $('zPickText').textContent = L.zPick; $('zForget').textContent = L.zForget;
    const zl = $('zLevels'); zl.innerHTML = '';
    $('zForget').classList.toggle('hidden', !ZG.Z.ready());
    if (ZG.Z.ready()) {
      $('zHint').textContent = L.zLoaded;
      ZG.Z.levels.forEach(function (lv, k) {
        const b = document.createElement('button');
        b.textContent = lv.n + ' · ' + lv.name; b.disabled = k > zUnlocked; b.title = k > zUnlocked ? L.locked : lv.name;
        if (k === zUnlocked) b.className = 'primary';
        b.onclick = () => start(k, ZG.Z.makeLevel(k));
        zl.appendChild(b);
      });
    } else if (!$('zHint').dataset.msg) $('zHint').textContent = L.zHint;
    $('ctrlTitle').textContent = L.ctrlTitle;
    seg('ctrlBtns', L.ctrl, classic ? 0 : 1, k => { classic = k === 0; store.set('ctrl', k); buildMenu(); });
    const fill = (id, label, names, keep) => {
      const sel = $(id), v = keep && sel.value !== '' ? sel.value : null; sel.innerHTML = '';
      names.forEach((n, k) => { const o = document.createElement('option'); o.value = k; o.textContent = n; sel.appendChild(o); });
      if (v != null) sel.value = v; else sel.value = Math.min(1, names.length - 1);
      sel.previousElementSibling.textContent = label;
    };
    fill('skSize', L.size, L.sizes, true); fill('skWorld', L.world, L.worlds, true); fill('skTech', L.strength, ['★', '★★', '★★★', '★★★★'], true);
    $('skSeed').previousElementSibling.textContent = L.seedLabel; $('btnSkirmish').textContent = L.startSk;
    if (!$('skSeed').value) $('skSeed').value = 1 + Math.floor(Math.random() * 9999);
  }
  const SK_SIZE = [[3, 2], [4, 3], [5, 4], [7, 4]], SK_STR = [1, 4, 8, 14];
  $('btnSkirmish').onclick = function () {
    const seed = Math.max(1, Math.floor(+$('skSeed').value) || 1), sz = SK_SIZE[+$('skSize').value];
    start(-1, ZG.makeLevel(SK_STR[+$('skTech').value], sz[0], sz[1], { theme: +$('skWorld').value, seed: seed, river: seed % 4, custom: true }));
  };
  $('btnSeed').onclick = () => { $('skSeed').value = 1 + Math.floor(Math.random() * 9999); };
  $('zPick').onchange = async function () {
    const list = this.files; if (!list || !list.length) return;
    $('zHint').textContent = L.zLoading;
    const files = await ZG.Z.readPicked(list);
    this.value = '';
    if (!ZG.Z.use(files)) { $('zHint').dataset.msg = 1; $('zHint').textContent = L.zNone; buildMenu(); return; }
    delete $('zHint').dataset.msg;
    try { await ZG.Z.save(files); } catch (e) { /* storage full or blocked: still usable this session */ }
    buildMenu();
  };
  $('zForget').onclick = async function () { await ZG.Z.forget(); buildMenu(); };
  $('btnLang').onclick = () => { lang = lang === 'de' ? 'en' : 'de'; store.set('lang', lang); buildMenu(); };
  $('btnHelp').onclick = $('btnPauseHelp').onclick = () => $('help').classList.remove('hidden');
  $('btnHelpBack').onclick = () => $('help').classList.add('hidden');
  $('btnMenu').onclick = () => setPause(true);
  $('btnResume').onclick = () => setPause(false);
  $('btnRestart').onclick = $('btnRetry').onclick = () => zIdx >= 0 ? start(zIdx, ZG.Z.makeLevel(zIdx)) : start(cur, skDef);
  $('btnQuit').onclick = $('btnEndMenu').onclick = showMenu;
  $('btnNext').onclick = () => zIdx >= 0 ? start(zIdx + 1, ZG.Z.makeLevel(zIdx + 1)) : start(cur + 1);
  $('btnSpeed').onclick = () => { speed = speed === 1 ? 2 : speed === 2 ? 0.5 : 1; $('btnSpeed').textContent = speed + '×'; };
  $('btnSound').onclick = () => {
    sound = !sound; store.set('sound', sound ? '1' : '0'); $('btnSound').style.opacity = sound ? 1 : 0.4;
    if (!sound) { music.pause(); musicName = ''; } else if (G) playMusic(gameMusic());
  };
  $('btnFull').onclick = function () {
    if (document.fullscreenElement) document.exitFullscreen(); else if (document.documentElement.requestFullscreen) document.documentElement.requestFullscreen().catch(() => {});
  };
  // the menu music needs a first click before browsers allow sound
  document.addEventListener('pointerdown', () => { if (!G && !musicName) playMusic('ipoptions16.ogg'); }, { capture: true });
  $('btnSound').style.opacity = sound ? 1 : 0.4;

  // ---------- main loop ----------
  let last = performance.now();
  function frame(now) {
    requestAnimationFrame(frame);
    const rdt = Math.min(0.1, (now - last) / 1000); last = now;
    if (!G) return;
    if (!paused) {
      const sp = 700 * rdt * 2 / zoom; let dx = 0, dy = 0;
      if (keys.ArrowLeft || keys.a) dx -= 1; if (keys.ArrowRight || keys.d) dx += 1;
      if (keys.ArrowUp || keys.w) dy -= 1; if (keys.ArrowDown || keys.s) dy += 1;
      // screen-edge scrolling; it goes on when the mouse slips past the edge of the window
      if ((mouse.inside || mouse.left) && !ui.drag && document.hasFocus()) {
        const E = 14;
        if (mouse.x < E) dx -= 1; else if (mouse.x > innerWidth - E) dx += 1;
        if (mouse.y < E) dy -= 1; else if (mouse.y > innerHeight - E) dy += 1;
      }
      if (dx || dy) { cam.x += dx * sp; cam.y += dy * sp; }
      let left = rdt * speed;
      while (left > 0.0001 && !G.over) { const st = Math.min(left, 0.04); ZG.step(G, st); left -= st; }
      if (G.over) for (const f of G.fx) f.t += rdt;
      for (const m of ui.marks) m.t += rdt;
      if (ui.paths) { for (const q of ui.paths) q.t += rdt; ui.paths = ui.paths.filter(q => q.t < 1.4); }
      ui.marks = ui.marks.filter(m => m.t < 0.5);
      if (G.dirty.length) { R.terrain.redraw(G.dirty); ZG.redrawMini(R, G.map, R.th, G.dirty); G.dirty.length = 0; }
      handleEvents();
    }
    clampCam();
    const S = scale(), lw = Math.ceil(cv.width / S) + 1, lh = Math.ceil(cv.height / S) + 1;
    if (lo.width !== lw || lo.height !== lh) { lo.width = lw; lo.height = lh; }
    lctx.setTransform(1, 0, 0, 1, 0, 0);
    lctx.fillStyle = '#0a0b0d'; lctx.fillRect(0, 0, lw, lh);
    lctx.setTransform(1, 0, 0, 1, -cam.x / A, -cam.y / A);
    lctx.imageSmoothingEnabled = false;
    ui.rally = ui.selB && ui.selB.team === 0 ? ui.selB.rally : null;
    ZG.draw(lctx, G, R, view, ui, G.time);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(lo, 0, 0, lw * S, lh * S);
    mctx.setTransform(1, 0, 0, 1, 0, 0);
    ZG.drawMini(mctx, G, R, view, mini.width, mini.height);
    updatePanel(); updateSide();
  }

  window.ZGdebug = { get G() { return G; }, start: start, get cam() { return cam; }, get view() { return view; } };
  resize(); buildMenu(); requestAnimationFrame(frame);
  // ?z=<folder url>: load the original Z files from a local server (see tools/zplay.py)
  async function loadZFrom(base) {
    const names = ['levels.dat', 'en.lproj/sprites.rsc', 'PNG/Background.png', 'PNG/BoxCentreWide.png', 'AA16.ogg', 'aC16.ogg', 'aJ16.ogg', 'ipOPTIONS16.ogg', 'ipWIN.ogg', 'ipLOSE.ogg'];
    for (const n of ZG.Z.SFX) names.push('audio/' + n + '.RAW');
    const voices = [];
    for (const lg of ['en', 'de']) {
      for (let i = 1; i <= 75; i++) voices.push(lg + '.lproj/ROB' + String(i).padStart(2, '0') + '.RAW');
      for (let i = 1; i <= 20; i++) voices.push(lg + '.lproj/COMP' + String(i).padStart(2, '0') + '.RAW');
    }
    for (let i = 1; i <= 20; i++) { const n = String(i).padStart(2, '0'); names.push('LEVEL' + n + '.MAP', 'BUILD' + n + '.DAT', 'OBJECT' + n + '.DAT', 'CPUPLR' + n + '.DAT', 'BRIDGE' + n + '.DAT'); }
    for (const w of ['DESERT', 'VOLCAN', 'ARCTIC', 'JUNGLE', 'CITY']) names.push(w + '.PAL', w + '.BLK');
    $('zHint').textContent = L.zLoading;
    const files = {};
    // a few downloads at a time; hundreds at once make browsers drop requests
    const queue = names.concat(voices);
    await Promise.all(Array.from({ length: 8 }, async function () {
      for (let n = queue.shift(); n; n = queue.shift()) if (!await fetchOne(n)) await fetchOne(n);
    }));
    async function fetchOne(n) {
      try {
        const r = await fetch(base + n);
        if (!r.ok) return r.status === 404;
        const key = n.indexOf('.lproj/') > 0 && /(ROB|COMP)\d\d/.test(n) ? n.slice(0, 2) + '/' + n.split('/').pop().toLowerCase() : n.split('/').pop().toLowerCase();
        files[key] = new Uint8Array(await r.arrayBuffer());
        return true;
      } catch (e) { return false; }
    }
    if (ZG.Z.use(files)) { try { await ZG.Z.save(files); } catch (e) { /* storage blocked */ } }
    if (G === null) buildMenu();
  }
  const zq = new URLSearchParams(location.search).get('z');
  if (zq && /^[\w\/.-]+$/.test(zq)) loadZFrom(zq.endsWith('/') ? zq : zq + '/');
  else ZG.Z.restore().then(n => { if (n && G === null) buildMenu(); });
})();
