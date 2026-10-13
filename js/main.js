// Zonebots - browser front end: input, HUD, menus, sound, main loop
(function () {
  const ZG = window.ZG, TS = ZG.TILE, $ = id => document.getElementById(id);
  const cv = $('game'), ctx = cv.getContext('2d'), mini = $('mini'), mctx = mini.getContext('2d');
  const store = {
    get(k, d) { try { const v = localStorage.getItem('zonebots.' + k); return v == null ? d : v; } catch (e) { return d; } },
    set(k, v) { try { localStorage.setItem('zonebots.' + k, v); } catch (e) { /* private mode */ } }
  };
  let lang = store.get('lang', (navigator.language || 'en').slice(0, 2) === 'de' ? 'de' : 'en');
  let unlocked = +store.get('unlocked', 0), sound = store.get('sound', '1') === '1', diff = +store.get('diff', 1), skDef = null;
  let L = ZG.TEXT[lang];
  let G = null, R = null, cur = 0, paused = false, speed = 1, zoom = 1, dpr = 1;
  const cam = { x: 0, y: 0 }, view = { x: 0, y: 0, w: 0, h: 0 };
  const ui = { selB: null, drag: null, marks: [] };
  const mouse = { x: 0, y: 0, inside: false }, keys = {}, groups = {};
  let amove = false, panelSig = '', lastClick = { t: 0, type: null }, miniDrag = false;

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
  const onScreen = e => e.x > view.x - 100 && e.y > view.y - 100 && e.x < view.x + view.w + 100 && e.y < view.y + view.h + 100;

  // ---------- helpers ----------
  function resize() {
    dpr = window.devicePixelRatio || 1;
    cv.width = Math.floor(innerWidth * dpr); cv.height = Math.floor(innerHeight * dpr);
    if (G) sizeMini();
  }
  function sizeMini() {
    const mw = Math.min(210, innerWidth * 0.3), mh = mw * G.map.H / G.map.W;
    mini.style.width = mw + 'px'; mini.style.height = mh + 'px';
    mini.width = Math.round(mw * dpr); mini.height = Math.round(mh * dpr);
  }
  function clampCam() {
    const S = zoom * dpr, top = 40 / zoom;
    view.w = cv.width / S; view.h = cv.height / S;
    const mw = G.map.W * TS, mh = G.map.H * TS;
    cam.x = view.w >= mw ? (mw - view.w) / 2 : ZG.clamp(cam.x, 0, mw - view.w);
    cam.y = view.h - top >= mh ? (mh - view.h - top) / 2 : ZG.clamp(cam.y, -top, mh - view.h);
    view.x = cam.x; view.y = cam.y;
  }
  const centerOn = (x, y) => { cam.x = x - view.w / 2; cam.y = y - view.h / 2; clampCam(); };
  const world = e => ({ x: e.clientX / zoom + cam.x, y: e.clientY / zoom + cam.y });
  const selUnits = () => G.units.filter(u => u.sel && !u.dead);
  function clearSel() { for (const u of G.units) u.sel = false; ui.selB = null; }
  function unitAt(p, pred) {
    let best = null, bd = 1e9;
    for (const u of G.units) {
      if (u.dead || !pred(u)) continue;
      const d = Math.hypot(u.x - p.x, u.y - p.y);
      if (d < u.def.r + 6 && d < bd) { bd = d; best = u; }
    }
    return best;
  }
  const buildingAt = p => G.map.buildings.find(b => p.x >= b.tx * TS && p.y >= b.ty * TS && p.x < (b.tx + b.w) * TS && p.y < (b.ty + b.h) * TS);
  function toast(msg, cls) {
    const d = document.createElement('div'); d.className = 'toast ' + (cls || ''); d.textContent = msg;
    $('toasts').appendChild(d); setTimeout(() => d.remove(), 3200);
    while ($('toasts').children.length > 4) $('toasts').firstChild.remove();
  }

  // ---------- orders ----------
  function order(p, forceAmove) {
    const us = selUnits().filter(u => u.team === 0);
    if (!us.length) {
      const sb = ui.selB;
      if (sb && sb.team === 0 && !sb.dead && ZG.products(sb).length) { sb.rally = { x: p.x, y: p.y }; ui.marks.push({ x: p.x, y: p.y, t: 0, atk: false }); toast(L.rally); }
      return;
    }
    const enemy = unitAt(p, u => u.team === 1), neutral = unitAt(p, u => u.team < 0 && u.def.sp), b = buildingAt(p);
    let atk = true;
    if (enemy) ZG.orderAttack(G, us, enemy);
    else if (neutral && us.some(u => u.def.cls === 'robot')) { ZG.orderEnter(G, us, neutral); atk = false; }
    else if (b && b.kind === 'fort' && b.team === 1 && !b.dead) ZG.orderFort(G, us, b);
    else { atk = forceAmove; ZG.orderMove(G, us, p.x, p.y, forceAmove ? 'amove' : 'move'); }
    ui.marks.push({ x: p.x, y: p.y, t: 0, atk: atk });
    tone('square', 520, 620, 0.05, 0.04);
  }

  // ---------- input ----------
  cv.addEventListener('contextmenu', e => e.preventDefault());
  cv.addEventListener('mousedown', function (e) {
    if (!G || paused || G.over) return;
    const p = world(e);
    if (e.button === 0) {
      if (amove) { order(p, true); setAmove(false); return; }
      ui.drag = { x0: p.x, y0: p.y, x1: p.x, y1: p.y, sx: e.clientX, sy: e.clientY, shift: e.shiftKey };
    } else if (e.button === 2) {
      if (amove) { setAmove(false); return; }
      order(p, e.ctrlKey || e.altKey || e.metaKey);
    }
  });
  window.addEventListener('mousemove', function (e) {
    mouse.x = e.clientX; mouse.y = e.clientY; mouse.inside = true;
    if (G && ui.drag) { const p = world(e); ui.drag.x1 = p.x; ui.drag.y1 = p.y; }
    if (G && miniDrag) miniTo(e);
  });
  document.addEventListener('mouseleave', () => { mouse.inside = false; });
  window.addEventListener('blur', () => { mouse.inside = false; for (const k in keys) keys[k] = false; });
  window.addEventListener('mouseup', function (e) {
    miniDrag = false;
    if (!G || !ui.drag || e.button !== 0) return;
    const d = ui.drag; ui.drag = null;
    if (!d.shift) clearSel();
    if (Math.hypot(e.clientX - d.sx, e.clientY - d.sy) < 6) {
      const p = { x: d.x1, y: d.y1 }, u = unitAt(p, v => v.team === 0);
      if (u) {
        const now = performance.now();
        if (now - lastClick.t < 350 && lastClick.type === u.t) selOnScreen(u.t);
        else u.sel = !(d.shift && u.sel);
        lastClick = { t: now, type: u.t }; ui.selB = null;
      } else {
        const b = buildingAt(p);
        if (b && !b.dead) { for (const v of G.units) v.sel = false; ui.selB = b; }
      }
    } else {
      const x0 = Math.min(d.x0, d.x1), x1 = Math.max(d.x0, d.x1), y0 = Math.min(d.y0, d.y1), y1 = Math.max(d.y0, d.y1);
      let any = false;
      for (const u of G.units) if (u.team === 0 && u.def.sp && u.x >= x0 && u.x <= x1 && u.y >= y0 && u.y <= y1) { u.sel = true; any = true; }
      if (any) ui.selB = null;
    }
  });
  cv.addEventListener('wheel', function (e) {
    if (!G) return;
    e.preventDefault();
    const p = world(e), z = ZG.clamp(zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1), 0.5, 2.5);
    zoom = z; cam.x = p.x - e.clientX / zoom; cam.y = p.y - e.clientY / zoom; clampCam();
  }, { passive: false });

  function selOnScreen(type) {
    ui.selB = null;
    for (const v of G.units) if (v.team === 0 && v.def.sp && (!type || v.t === type) && v.x > view.x && v.y > view.y && v.x < view.x + view.w && v.y < view.y + view.h) v.sel = true;
  }

  // touch: tap = select / order, drag = scroll, pinch = zoom
  let touch = null;
  function tap(p) {
    if (amove) { order(p, true); setAmove(false); return; }
    const u = unitAt(p, v => v.team === 0 && v.def.sp), b = buildingAt(p), have = selUnits().length > 0;
    if (u) {
      const now = performance.now();
      if (now - lastClick.t < 400 && lastClick.type === u.t) selOnScreen(u.t); else { u.sel = !u.sel; ui.selB = null; }
      lastClick = { t: now, type: u.t };
    } else if (b && !b.dead && !have && b !== ui.selB) { clearSel(); ui.selB = b; }
    else if (have || (ui.selB && ui.selB.team === 0)) order(p, false);
    else clearSel();
  }
  cv.addEventListener('touchstart', function (e) {
    if (!G || paused || G.over) return;
    e.preventDefault();
    const t = e.touches;
    if (t.length === 1) touch = { x: t[0].clientX, y: t[0].clientY, sx: t[0].clientX, sy: t[0].clientY, moved: false, pinch: 0 };
    else if (t.length === 2) touch = { moved: true, pinch: Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY) };
  }, { passive: false });
  cv.addEventListener('touchmove', function (e) {
    if (!G || !touch) return;
    e.preventDefault();
    const t = e.touches;
    if (t.length === 2 && touch.pinch) {
      const d = Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY), cx = (t[0].clientX + t[1].clientX) / 2, cy = (t[0].clientY + t[1].clientY) / 2;
      const p = { x: cx / zoom + cam.x, y: cy / zoom + cam.y };
      zoom = ZG.clamp(zoom * d / touch.pinch, 0.5, 2.5); touch.pinch = d;
      cam.x = p.x - cx / zoom; cam.y = p.y - cy / zoom; clampCam();
    } else if (t.length === 1 && !touch.pinch) {
      if (!touch.moved && Math.hypot(t[0].clientX - touch.sx, t[0].clientY - touch.sy) > 10) touch.moved = true;
      if (touch.moved) { cam.x -= (t[0].clientX - touch.x) / zoom; cam.y -= (t[0].clientY - touch.y) / zoom; clampCam(); }
      touch.x = t[0].clientX; touch.y = t[0].clientY;
    }
  }, { passive: false });
  cv.addEventListener('touchend', function (e) {
    if (!G || !touch || e.touches.length) return;
    e.preventDefault();
    if (!touch.moved) tap({ x: touch.sx / zoom + cam.x, y: touch.sy / zoom + cam.y });
    touch = null;
  }, { passive: false });

  function miniTo(e) {
    const r = mini.getBoundingClientRect();
    centerOn((e.clientX - r.left) / r.width * G.map.W * TS, (e.clientY - r.top) / r.height * G.map.H * TS);
  }
  mini.addEventListener('mousedown', function (e) {
    if (!G) return;
    e.preventDefault();
    if (e.button === 2) { const r = mini.getBoundingClientRect(); order({ x: (e.clientX - r.left) / r.width * G.map.W * TS, y: (e.clientY - r.top) / r.height * G.map.H * TS }, e.ctrlKey); }
    else { miniDrag = true; miniTo(e); }
  });
  mini.addEventListener('contextmenu', e => e.preventDefault());

  function setAmove(v) { amove = v; $('mode').classList.toggle('hidden', !v); $('mode').textContent = lang === 'de' ? 'Angriffsbewegung: Ziel anklicken' : 'Attack-move: click target'; cv.style.cursor = v ? 'crosshair' : 'default'; }

  window.addEventListener('keydown', function (e) {
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    if (!G || !$('menu').classList.contains('hidden')) return;
    if (k === 'Escape' || k === 'p') { if (!$('help').classList.contains('hidden')) { $('help').classList.add('hidden'); return; } if (amove) setAmove(false); else if (!G.over) setPause(!paused); return; }
    if (paused || G.over) return;
    if (e.ctrlKey && k === 'a') { e.preventDefault(); clearSel(); for (const u of G.units) if (u.team === 0 && u.def.sp) u.sel = true; return; }
    if (k >= '1' && k <= '9') {
      e.preventDefault();
      if (e.ctrlKey || e.metaKey) groups[k] = selUnits();
      else if (groups[k]) {
        const g = groups[k].filter(u => !u.dead && u.team === 0);
        if (g.length) { const twice = g.every(u => u.sel) && selUnits().length === g.length; clearSel(); for (const u of g) u.sel = true; if (twice) centerOn(g[0].x, g[0].y); }
      }
      return;
    }
    if (e.ctrlKey || e.metaKey) return;
    if (k === 'f') { if (selUnits().length) setAmove(!amove); return; }
    if (k === 'x') { ZG.orderStop(G, selUnits().filter(u => u.team === 0)); return; }
    if (k === ' ') { e.preventDefault(); const f = G.map.buildings.find(b => b.kind === 'fort' && b.team === 0); centerOn(f.cx, f.cy + 60); return; }
    keys[k] = true;
    if (k.indexOf('Arrow') === 0) e.preventDefault();
  });
  window.addEventListener('keyup', e => { keys[e.key.length === 1 ? e.key.toLowerCase() : e.key] = false; });
  window.addEventListener('resize', resize);

  // ---------- HUD ----------
  const iconCache = {};
  function icon(t) {
    if (iconCache[t]) return iconCache[t];
    const c = document.createElement('canvas'); c.width = c.height = 88;
    const x = c.getContext('2d'), d = ZG.UNITS[t], sc = d.cls === 'robot' ? 2.8 : 2;
    x.scale(sc, sc);
    ZG.drawUnit(x, { x: 44 / sc, y: 46 / sc, def: d, t: t, team: 0, ang: -Math.PI / 2, tang: -Math.PI / 2, hit: 0, moving: false, walk: 0 });
    return (iconCache[t] = c);
  }
  function updatePanel() {
    const b = ui.selB, us = selUnits();
    const sig = (b ? 'b' + b.id + ':' + b.team + ':' + b.prod : '') + '|' + us.length + ':' + us.map(u => u.t).sort().join(',') + lang;
    const panel = $('panel');
    if (sig !== panelSig) {
      panelSig = sig;
      panel.classList.toggle('hidden', !b && !us.length);
      $('bldInfo').classList.toggle('hidden', !b);
      $('selinfo').innerHTML = '';
      if (b) {
        $('bName').textContent = L.b[b.kind];
        $('bTech').textContent = b.kind === 'repair' ? '' : '★'.repeat(b.tech);
        const mine = b.team === 0, list = mine ? ZG.products(b) : [];
        $('bSub').textContent = b.kind === 'repair' ? L.repairHint : mine && b.prod ? L.builds + ': ' + L.units[b.prod] : b.team === 1 ? '—' : '';
        const box = $('prods'); box.innerHTML = '';
        for (const k of list) {
          const btn = document.createElement('button'); btn.className = 'prod' + (k === b.prod ? ' on' : '');
          const d = ZG.UNITS[k], im = document.createElement('canvas'); im.width = im.height = 88; im.getContext('2d').drawImage(icon(k), 0, 0);
          btn.appendChild(im);
          btn.appendChild(document.createTextNode(L.units[k] + (d.n > 1 ? ' ×' + d.n : '')));
          btn.title = L.units[k] + ' · ' + d.time + ' s';
          btn.onclick = () => { if (b.prod !== k) { b.prod = k; b.progress = 0; } };
          box.appendChild(btn);
        }
        $('pbar').classList.toggle('hidden', !mine || !list.length); b._left = -1;
      } else if (us.length) {
        const cnt = {};
        for (const u of us) cnt[u.t] = (cnt[u.t] || 0) + 1;
        for (const k in cnt) { const s = document.createElement('span'); s.textContent = cnt[k] + '× ' + L.units[k]; $('selinfo').appendChild(s); }
      }
    }
    if (b) {
      $('pbar').firstElementChild.style.width = (b.progress * 100) + '%';
      if (b.team === 0 && b.prod) {
        const left = Math.ceil((1 - b.progress) * ZG.UNITS[b.prod].time / ZG.prodRate(G, 0));
        if (left !== b._left) { b._left = left; $('bSub').textContent = L.builds + ': ' + L.units[b.prod] + ' · ' + L.left + ' ' + left + ' s'; }
      }
    }
  }
  function updateTop() {
    const n = G.map.sectors.length; let a = 0, b = 0;
    for (const s of G.map.sectors) { if (s.owner === 0) a++; else if (s.owner === 1) b++; }
    $('terrR').style.width = (a / n * 100) + '%'; $('terrB').style.width = (b / n * 100) + '%';
    $('terrTxt').textContent = a + ' : ' + b;
    const t = Math.floor(G.time); $('clock').textContent = Math.floor(t / 60) + ':' + String(t % 60).padStart(2, '0');
  }

  function handleEvents() {
    for (const e of G.ev) {
      if (e.k === 'shot') {
        const now = performance.now();
        if (now - lastShot > 55 && onScreen(e)) {
          lastShot = now;
          if (e.w === 'laser') tone('sawtooth', 1400, 300, 0.12, 0.03);
          else if (e.w === 'flame') tone('noise', 900, 300, 0.12, 0.03);
          else if (e.w === 'bullet') tone('noise', 3000, 900, 0.05, 0.05);
          else tone('noise', 700, 120, 0.22, 0.09);
        }
      } else if (e.k === 'boom') { if (onScreen(e)) tone('noise', e.big ? 500 : 900, 50, e.big ? 0.7 : 0.3, e.big ? 0.22 : 0.1); }
      else if (e.k === 'capture') {
        if (e.team === 0) { toast(L.captured, 'good'); tone('triangle', 520, 1040, 0.25, 0.1); }
        else if (e.prev === 0) { toast(L.lost, 'bad'); tone('triangle', 400, 160, 0.3, 0.1); }
      } else if (e.k === 'ready' && e.team === 0) { toast(L.units[e.t] + ' ' + L.ready); tone('sine', 880, 1320, 0.12, 0.06); }
      else if (e.k === 'alert') { toast(L.alert, 'bad'); tone('square', 660, 440, 0.4, 0.08); }
      else if (e.k === 'enter' && e.team === 0) toast(L.crewed, 'good');
      else if (e.k === 'crate' && e.team === 0) { toast(L.crate, 'good'); tone('sine', 660, 990, 0.15, 0.07); }
      else if (e.k === 'driver' && e.team === 0) toast(L.driver, 'good');
      else if (e.k === 'over') setTimeout(showEnd, 1300);
    }
    G.ev.length = 0;
  }

  // ---------- screens ----------
  function setPause(v) { paused = v; $('pause').classList.toggle('hidden', !v); }
  function showEnd() {
    if (!G || !G.over) return;
    const win = G.over.winner === 0, last = cur === ZG.LEVELS.length - 1, camp = cur >= 0;
    if (win && camp && cur + 1 > unlocked) { unlocked = Math.min(cur + 1, ZG.LEVELS.length); store.set('unlocked', unlocked); }
    let bestTxt = '';
    if (win && camp) {
      const key = 'best' + cur + 'd' + diff, old = +store.get(key, 0), now = Math.floor(G.over.time);
      if (!old || now < old) { store.set(key, now); bestTxt = L.newBest; } else bestTxt = L.best + ': ' + fmt(old);
    }
    $('endTitle').textContent = win ? L.win : L.lose;
    $('endTitle').style.color = win ? '#4bd04b' : '#e0432f';
    $('endSub').textContent = win && last ? L.allDone : levelName(cur);
    const t = Math.floor(G.over.time);
    $('endStats').innerHTML = '<span>' + L.time + '</span><b>' + fmt(t) + '</b><span>' + L.diffLabel + '</span><b>' + L.diff[diff] + '</b><span>' + L.kills + '</span><b>' + G.stats[0].kills + '</b><span>' + L.losses + '</span><b>' + G.stats[0].losses + '</b>';
    $('endBest').textContent = bestTxt;
    $('btnNext').classList.toggle('hidden', !win || last || !camp);
    $('btnRetry').classList.toggle('primary', !win);
    $('end').classList.remove('hidden');
    if (win) { tone('triangle', 523, 523, 0.15, 0.1); setTimeout(() => tone('triangle', 784, 784, 0.3, 0.1), 160); } else tone('sawtooth', 220, 80, 0.8, 0.08);
  }
  const fmt = t => Math.floor(t / 60) + ':' + String(t % 60).padStart(2, '0');
  const levelName = i => i < 0 ? L.skirmish + ' · ' + L.worlds[skDef.theme] + ' #' + skDef.seed : L.worlds[ZG.LEVELS[i].theme] + ' · ' + L.level + ' ' + (i + 1);

  function start(i, def) {
    cur = def ? -1 : i; skDef = def || null; G = ZG.newGame(def || i, { diff: diff });
    const th = ZG.THEMES[G.lv.theme];
    R = { terrain: ZG.renderTerrain(G.map, th, G.lv.seed), mini: ZG.renderMini(G.map, th) };
    ui.selB = null; ui.drag = null; ui.marks = []; panelSig = ''; paused = false; setAmove(false);
    for (const k in groups) delete groups[k];
    zoom = innerWidth > 2600 ? 2 : innerWidth > 1700 ? 1.5 : 1;
    for (const id of ['menu', 'end', 'pause', 'help']) $(id).classList.add('hidden');
    $('hud').classList.remove('hidden');
    $('lvTitle').textContent = levelName(cur);
    for (const a of [['actAll', L.selAll], ['actNone', L.selNone], ['actStop', L.stop], ['actAtk', L.amoveBtn]]) $(a[0]).textContent = a[1];
    resize(); clampCam();
    const f = G.map.buildings.find(b => b.kind === 'fort' && b.team === 0);
    centerOn(f.cx, f.cy + 60);
  }
  function showMenu() {
    G = null; $('hud').classList.add('hidden');
    for (const id of ['end', 'pause', 'help']) $(id).classList.add('hidden');
    $('menu').classList.remove('hidden');
    buildMenu();
  }
  function buildMenu() {
    L = ZG.TEXT[lang]; document.documentElement.lang = lang;
    $('tagline').textContent = L.sub;
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
    const db = $('diffBtns'); db.innerHTML = '';
    L.diff.forEach(function (name, k) {
      const b = document.createElement('button'); b.textContent = name; if (k === diff) b.className = 'primary';
      b.onclick = () => { diff = k; store.set('diff', k); buildMenu(); };
      db.appendChild(b);
    });
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
  $('actAll').onclick = () => { if (G) selOnScreen(null); };
  $('actNone').onclick = () => { if (G) clearSel(); };
  $('actStop').onclick = () => { if (G) ZG.orderStop(G, selUnits().filter(u => u.team === 0)); };
  $('actAtk').onclick = () => { if (G && selUnits().length) setAmove(!amove); };
  $('btnLang').onclick = () => { lang = lang === 'de' ? 'en' : 'de'; store.set('lang', lang); buildMenu(); };
  $('btnHelp').onclick = $('btnPauseHelp').onclick = () => $('help').classList.remove('hidden');
  $('btnHelpBack').onclick = () => $('help').classList.add('hidden');
  $('btnMenu').onclick = () => setPause(true);
  $('btnResume').onclick = () => setPause(false);
  $('btnRestart').onclick = $('btnRetry').onclick = () => start(cur, skDef);
  $('btnQuit').onclick = $('btnEndMenu').onclick = showMenu;
  $('btnNext').onclick = () => start(cur + 1);
  $('btnSpeed').onclick = () => { speed = speed === 1 ? 2 : speed === 2 ? 0.5 : 1; $('btnSpeed').textContent = speed + '×'; };
  $('btnSound').onclick = () => { sound = !sound; store.set('sound', sound ? '1' : '0'); $('btnSound').style.opacity = sound ? 1 : 0.4; };
  $('btnSound').style.opacity = sound ? 1 : 0.4;

  // ---------- main loop ----------
  let last = performance.now();
  function frame(now) {
    requestAnimationFrame(frame);
    const rdt = Math.min(0.1, (now - last) / 1000); last = now;
    if (!G) return;
    if (!paused) {
      // scrolling
      const sp = 900 / zoom * rdt; let dx = 0, dy = 0;
      if (keys.ArrowLeft || keys.a) dx -= 1; if (keys.ArrowRight || keys.d) dx += 1;
      if (keys.ArrowUp || keys.w) dy -= 1; if (keys.ArrowDown || keys.s) dy += 1;
      if (mouse.inside && !ui.drag && document.hasFocus()) {
        if (mouse.x < 4) dx -= 1; else if (mouse.x > innerWidth - 5) dx += 1;
        if (mouse.y < 4) dy -= 1; else if (mouse.y > innerHeight - 5) dy += 1;
      }
      if (dx || dy) { cam.x += dx * sp; cam.y += dy * sp; }
      let left = rdt * speed;
      while (left > 0.0001 && !G.over) { const st = Math.min(left, 0.04); ZG.step(G, st); left -= st; }
      if (G.over) for (const f of G.fx) f.t += rdt;
      for (const m of ui.marks) m.t += rdt;
      ui.marks = ui.marks.filter(m => m.t < 0.5);
      handleEvents();
    }
    clampCam();
    const S = zoom * dpr;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#0c0e11'; ctx.fillRect(0, 0, cv.width, cv.height);
    ctx.setTransform(S, 0, 0, S, -Math.round(cam.x * S), -Math.round(cam.y * S));
    ctx.imageSmoothingEnabled = zoom !== 1;
    ZG.draw(ctx, G, R, view, ui);
    mctx.setTransform(1, 0, 0, 1, 0, 0);
    ZG.drawMini(mctx, G, R, view, mini.width, mini.height);
    updatePanel(); updateTop();
  }

  window.ZGdebug = { get G() { return G; }, start: start };
  resize(); buildMenu(); requestAnimationFrame(frame);
})();
