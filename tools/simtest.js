// Headless sanity test: generates all 20 levels, checks reachability and lets two AIs play each one.
// Run: node tools/simtest.js
const path = require('path');
for (const f of ['data', 'map', 'game']) require(path.join(__dirname, '..', 'js', f + '.js'));
const ZG = globalThis.ZG, TS = ZG.TILE;
let bad = 0;
for (let i = 0; i < ZG.LEVELS.length; i++) {
  const G = ZG.newGame(i, { aiTeams: [0, 1] }), map = G.map;
  const forts = map.buildings.filter(b => b.kind === 'fort');
  const from = forts[0].door, px = t => (t + 0.5) * TS;
  let unreachable = 0;
  for (const s of map.sectors) if (s.flag && !ZG.findPath(map, px(from.tx), px(from.ty), s.flag.x, s.flag.y)) unreachable++;
  if (!ZG.findPath(map, px(from.tx), px(from.ty), px(forts[1].door.tx), px(forts[1].door.ty))) unreachable++;
  const t0 = Date.now();
  let steps = 0, maxUnits = 0;
  while (!G.over && G.time < 2400) { ZG.step(G, 0.05); steps++; G.ev.length = 0; if (G.units.length > maxUnits) maxUnits = G.units.length; }
  const own = [0, 1].map(t => map.sectors.filter(s => s.owner === t).length);
  if (unreachable) bad++;
  console.log(`L${String(i + 1).padStart(2)} ${map.W}x${map.H} sectors=${map.sectors.length} bld=${map.buildings.length} unreachable=${unreachable} ` +
    `winner=${G.over ? G.over.winner : '-'} t=${Math.round(G.time)}s own=${own} maxUnits=${maxUnits} ${(Date.now() - t0)}ms/${steps}`);
}
if (bad) { console.error('FAILED: unreachable targets in ' + bad + ' level(s)'); process.exit(1); }
console.log('OK');
