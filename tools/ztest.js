// Checks the original-data mode against your own copy of the 1996 game "Z".
// Run: node tools/ztest.js "C:/GOG Games/Z"
// Nothing is copied; the files are only read.
const path = require('path'), fs = require('fs');
for (const f of ['data', 'map', 'zdata', 'game']) require(path.join(__dirname, '..', 'js', f + '.js'));
const ZG = globalThis.ZG, TS = ZG.TILE;
const dir = process.argv[2];
if (!dir) { console.error('usage: node tools/ztest.js <folder of your Z installation>'); process.exit(2); }

const files = {};
for (const name of fs.readdirSync(dir)) if (ZG.Z.wanted(name)) files[name.toLowerCase()] = new Uint8Array(fs.readFileSync(path.join(dir, name)));
const n = ZG.Z.use(files);
console.log('levels found:', n);
let bad = 0;
for (let k = 0; k < n; k++) {
  const lv = ZG.Z.makeLevel(k);
  let G;
  try { G = ZG.newGame(lv, { aiTeams: [0, 1] }); } catch (e) { console.log(lv.zlevel, 'ERROR', e.message); bad++; continue; }
  const map = G.map, forts = map.buildings.filter(b => b.kind === 'fort'), d = forts[0].door, px = t => (t + 0.5) * TS;
  // flags on islands are reported but allowed; the enemy fort must be reachable
  let unreachable = 0, fortOk = !!ZG.findPath(map, px(d.tx), px(d.ty), px(forts[1].door.tx), px(forts[1].door.ty));
  for (const s of map.sectors) if (s.flag && !ZG.findPath(map, px(d.tx), px(d.ty), s.flag.x, s.flag.y)) unreachable++;
  const kinds = {};
  for (const b of map.buildings) kinds[b.kind] = (kinds[b.kind] || 0) + 1;
  while (!G.over && G.time < 1500) { ZG.step(G, 0.05); G.ev.length = 0; }
  if (!fortOk) bad++;
  console.log(`L${lv.zlevel} ${lv.zname.padEnd(16)} ${map.z.world.padEnd(6)} ${map.W}x${map.H} flags=${map.sectors.filter(s => s.flag).length} ` +
    `bld=${JSON.stringify(kinds)} rocks=${map.tiles.filter(t => t === ZG.T.BOULDER).length} bridges=${map.bridges.length} islandFlags=${unreachable} fort=${fortOk ? 'ok' : 'UNREACHABLE'} winner=${G.over ? G.over.winner : '-'} t=${Math.round(G.time)}s`);
}
if (bad) { console.error('problems in ' + bad + ' level(s)'); process.exit(1); }
console.log('OK');
