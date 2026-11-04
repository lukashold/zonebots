// Browser adapter. Simulation, AI, combat, production and pathfinding execute in Rust/Wasm.
// The retained JS map/resource readers and renderer do not advance game state.
(function () {
  'use strict';
  const ZG = globalThis.ZG, encoder = new TextEncoder(), decoder = new TextDecoder();
  let wasm;
  const baseDefs = ZG.UNITS;
  function call(message) {
    const bytes = encoder.encode(JSON.stringify(message));
    const ptr = wasm.z_alloc(bytes.length);
    new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
    let length;
    try { length = wasm.z_request(ptr, bytes.length); }
    finally { wasm.z_free(ptr, bytes.length); }
    const result = JSON.parse(decoder.decode(new Uint8Array(wasm.memory.buffer, wasm.z_output(), length)));
    if (result.error) throw new Error(result.error);
    return result;
  }
  function controls(G) {
    return { nid: G.nid, units: G.units.map(u => [u.id, !!u.sel, u.squad || 0]),
      buildings: G.map.buildings.filter(b => b.team === 0).map(b => ({ id: b.id, prod: b.prod, held: b.held, rally: b.rally, paused: !!b.paused })) };
  }
  function apply(G, data) {
    const old = new Map(G.units.map(u => [u.id, u]));
    const hydrate = raw => {
      const u = old.get(raw.id) || {};
      Object.assign(u, raw); u.def = ZG.UNITS[u.t];
      u.cargo = u.def.cargo ? raw.cargo.map(hydrate) : null;
      return u;
    };
    G.units = data.units.map(hydrate);
    const byId = new Map(G.units.map(u => [u.id, u]));
    for (const b of data.buildings) Object.assign(G.map.buildings[b.id], b);
    for (const s of data.sectors) Object.assign(G.map.sectors[s.id], s);
    for (const b of data.bridges) Object.assign(G.map.bridges[b.id], b);
    G.map.crates = data.crates;
    const target = st => st && { kind: st.kind, ref: st.kind === 'b' ? G.map.buildings[st.id] : st.kind === 'br' ? G.map.bridges[st.id] : st.id, x: st.x, y: st.y };
    for (const u of G.units) {
      u.tgt = byId.get(u.tgt) || null;
      if (u.order) {
        if (u.order.target != null) u.order.u = byId.get(u.order.target) || { dead: true };
        if (u.order.building != null) u.order.b = G.map.buildings[u.order.building];
        if (u.order.st) u.order.st = target(u.order.st);
      }
    }
    G.shots = data.shots.map(s => Object.assign(s, { v: byId.get(s.v) || null, src: byId.get(s.src) || null, st: target(s.st) }));
    G.fx = data.fx; G.time = data.time; G.over = data.over; G.nid = data.nid; G.stats = data.stats; G.ai = data.ai;
    G.ev.push(...data.ev); G.dirty.push(...data.dirty);
    for (const [j, tile, block, hp] of data.changes) { G.map.tiles[j] = tile; G.map.block[j] = block; if (G.map.rockHp) G.map.rockHp[j] = hp; }
    return G;
  }
  ZG.newGame = function (idx, opt) {
    if (!wasm) throw new Error('Rust engine has not loaded');
    opt = opt || {};
    const df = ZG.DIFF[opt.diff == null ? 1 : opt.diff] || ZG.DIFF[1];
    const lv = Object.assign({}, typeof idx === 'object' ? idx : ZG.LEVELS[idx], { bonus: df.bonus });
    lv.aiRush = Math.max(3, Math.round(lv.aiRush * (df.rush || 1)));
    lv.aiBatch += df.batch || 0;
    ZG.UNITS = lv.original ? ZG.Z.unitDefs(baseDefs) : baseDefs;
    const map = lv.original ? ZG.Z.genMap(lv) : ZG.genMap(lv);
    const G = { lv, map, units: [], shots: [], fx: [], ev: [], dirty: [], time: 0, over: null, nid: 1, aiTeams: opt.aiTeams || [1], engine: 'rust-wasm' };
    const m = { W: map.W, H: map.H, tiles: Array.from(map.tiles), block: Array.from(map.block),
      sectors: map.sectors.map(({ b, ...s }) => s), buildings: map.buildings, spawns: map.spawns, crates: map.crates,
      rockHp: Array.from(map.rockHp), bridges: map.bridges, bridgeAt: Array.from(map.bridgeAt) };
    return apply(G, call({ op: 'init', map: m, lv, diff: df, defs: ZG.UNITS, unitOrder: Object.keys(ZG.UNITS), aiTeams: G.aiTeams }));
  };
  ZG.step = (G, dt) => apply(G, call({ op: 'tick', dt, controls: controls(G) }));
  function command(G, units, kind, extra) {
    return apply(G, call(Object.assign({ op: 'command', kind, ids: units.map(u => u.id), controls: controls(G) }, extra)));
  }
  const stData = st => ({ kind: st.kind, id: st.kind === 'rock' ? st.ref : st.ref.id, x: st.x, y: st.y });
  ZG.orderMove = (G, us, x, y, kind) => command(G, us, kind || 'move', { x, y });
  ZG.orderAttack = (G, us, v) => command(G, us, 'attack', { target: v.id });
  ZG.orderEnter = (G, us, v) => command(G, us, 'enter', { target: v.id });
  ZG.orderBoard = (G, us, v) => command(G, us, 'board', { target: v.id });
  ZG.orderUnload = (G, us) => command(G, us, 'unload');
  ZG.orderStop = (G, us) => command(G, us, 'stop');
  ZG.orderStruct = (G, us, st) => command(G, us, 'hit', { st: stData(st) });
  ZG.orderFort = (G, us, b) => ZG.orderStruct(G, us, { kind: 'b', ref: b, x: b.cx, y: b.cy });
  ZG.orderRepair = (G, us, st) => command(G, us, 'fix', { st: stData(st) });
  ZG.squadOf = (G, u) => u.squad ? G.units.filter(v => !v.dead && v.squad === u.squad && v.team === u.team) : [u];
  ZG.products = b => b.dead || b.kind === 'repair' || b.kind === 'radar' ? [] : Object.keys(ZG.UNITS).filter(k => {
    const d = ZG.UNITS[k];
    if (d.buildLevels) return d.buildLevels[b.kind] > 0 && d.buildLevels[b.kind] <= b.tech;
    return d.tech <= b.tech && (b.kind === 'fort' || d.cls === b.kind || d.cls === 'gun');
  });
  ZG.prodRate = (G, team) => (0.55 + 1.1 * G.map.sectors.filter(s => s.owner === team).length / G.map.sectors.length) * (team === 1 && G.aiTeams.includes(1) ? G.ai.aiRate : 1);
  ZG.structAt = (G, x, y) => {
    const tx = Math.floor(x / 32), ty = Math.floor(y / 32), m = G.map;
    if (tx < 0 || ty < 0 || tx >= m.W || ty >= m.H) return null;
    const j = ty * m.W + tx;
    for (const b of m.buildings) if (tx >= b.tx && ty >= b.ty && tx < b.tx + b.w && ty < b.ty + b.h) return { kind: 'b', ref: b, x: b.cx, y: b.cy };
    if (m.tiles[j] === ZG.T.BOULDER) return { kind: 'rock', ref: j, x: tx * 32 + 16, y: ty * 32 + 16 };
    if (m.bridgeAt[j] >= 0) return { kind: 'br', ref: m.bridges[m.bridgeAt[j]], x: tx * 32 + 16, y: ty * 32 + 16 };
    return null;
  };
  globalThis.ZRust = { async load(bytes) { const result = await WebAssembly.instantiate(bytes, {}); wasm = result.instance.exports; return wasm.z_engine_version(); }, call, controls, apply };
})();
