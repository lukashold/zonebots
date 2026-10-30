// Exercise automatic production through the real Wasm ABI and a private campaign.
// Original maps, placements and resources are read locally, never embedded here.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const options = {
  campaign: path.join(root, 'dist/original/campaign.json'),
  wasm: path.join(root, 'dist/independent/engine.wasm'),
};
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i].replace(/^--/, '');
  assert(['campaign', 'wasm', 'report'].includes(key), `Unknown option ${process.argv[i]}`);
  assert(process.argv[i + 1], `Missing value for ${process.argv[i]}`);
  options[key] = path.resolve(process.argv[i + 1]);
}
assert(fs.existsSync(options.campaign), 'Missing private campaign: extract your local game first');
assert(fs.existsSync(options.wasm), 'Missing Wasm: run independent-engine/build.ps1');
const campaign = JSON.parse(fs.readFileSync(options.campaign, 'utf8'));
const clone = value => JSON.parse(JSON.stringify(value));

(async () => {
  const {instance} = await WebAssembly.instantiate(fs.readFileSync(options.wasm), {});
  const e = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
  function request(value) {
    const bytes = enc.encode(JSON.stringify(value)), ptr = e.alloc(bytes.length);
    new Uint8Array(e.memory.buffer, ptr, bytes.length).set(bytes);
    let length;
    try { length = e.run(ptr, bytes.length); } finally { e.free(ptr, bytes.length); }
    const state = JSON.parse(dec.decode(new Uint8Array(e.memory.buffer, e.output(), length)));
    assert(!state.error, state.error);
    return state;
  }
  const init = m => request({op:'init', mission:m.number - 1, difficulty:1,
    seed:9821 + m.number, aiTeams:[], originalMission:m});
  const byId = (state, id) => {
    const building = state.buildings.find(b => b.id === id);
    assert(building, `Missing building ${id}`);
    return building;
  };
  const menu = (state, building) => state.buildMenus[state.buildings.indexOf(building)];
  const defaultProduct = (state, choices) => choices.includes('grunt') ? 'grunt' :
    choices.find(k => state.catalog.find(c => c.type === k)?.class === 0) ?? choices[0];
  function timer(state, building) {
    const seconds = state.productionSeconds[state.buildings.indexOf(building)];
    assert(Number.isInteger(seconds) && seconds >= 0, `${building.kind} ${building.id}: missing timer`);
  }
  // Keep the real map and factory geometry; replace combatants with an isolated
  // capturer and a distant stationary opponent so tests cannot end in victory.
  function captureFixture(source, id, owner = -1) {
    const m = clone(source), target = m.buildings.find(b => b.id === id);
    const flag = m.sectors.find(s => s.id === target.sector);
    assert(flag && !flag.home && flag.countsForProduction !== false, 'Factory flag must be capturable');
    m.buildings.forEach(b => { b.team = -1; b.product = null; b.progress = 0; b.remainingWork = null; });
    m.sectors.forEach(s => { s.owner = -1; });
    target.team = owner; flag.owner = owner;
    const size = m.map.tileSize, width = m.map.width * size, height = m.map.height * size;
    const corners = [[size, size], [width-size, size], [size, height-size], [width-size, height-size]];
    corners.sort((a, b) => Math.hypot(b[0]-flag.x, b[1]-flag.y) - Math.hypot(a[0]-flag.x, a[1]-flag.y));
    const [x, y] = corners[0];
    assert(Math.hypot(x-flag.x, y-flag.y) > 600, 'Fixture combatants must be separated');
    m.spawns = [{type:'grunt', team:0, x:flag.x, y:flag.y, squad:1},
      {type:'gatling', team:1, x, y, squad:2}];
    m.crates = []; m.originalObjects = [];
    return m;
  }

  let ownedFactories = 0, capturedRobots = 0, manualCase;
  const missionResults = [];
  for (const source of campaign.missions) {
    const initial = init(source);
    assert.equal(initial.map.original, true);
    let owned = 0, captured = 0;
    for (const building of initial.buildings) {
      const choices = menu(initial, building);
      if (building.team < 0 || building.hp <= 0 || !choices.length) continue;
      const input = source.buildings.find(b => b.id === building.id);
      const expected = choices.includes(input.product) ? input.product : defaultProduct(initial, choices);
      assert.equal(building.product, expected, `Mission ${source.number}, owned ${building.kind} ${building.id}`);
      assert(choices.includes(building.product));
      timer(initial, building);
      owned++; ownedFactories++;
    }
    for (const input of source.buildings.filter(b => b.team < 0 && b.hp > 0 && b.kind === 'robot')) {
      const fixture = captureFixture(source, input.id);
      let state = init(fixture), building = byId(state, input.id);
      assert.equal(building.product, null, 'Neutral factories stay inactive until captured');
      state = request({op:'tick', dt:0});
      building = byId(state, input.id);
      assert.equal(building.team, 0);
      assert.equal(building.product, 'grunt', `Mission ${source.number}, captured robot factory ${input.id}`);
      assert(state.events.some(event => event.kind === 'capture' && event.sector === building.sector && event.team === 0));
      timer(state, building);
      const work = building.remainingWork;
      state = request({op:'advance', steps:10});
      building = byId(state, input.id);
      assert.equal(state.winner, null);
      assert.equal(building.product, 'grunt');
      assert(building.remainingWork < work && building.progress > 0, 'Captured production must advance');
      timer(state, building);
      if (!manualCase) {
        const alternative = menu(state, building).find(k => k !== 'grunt' && state.catalog.find(c => c.type === k)?.class === 0);
        if (alternative) manualCase = {source, id:input.id, alternative};
      }
      captured++; capturedRobots++;
    }
    missionResults.push({mission:source.number, ownedFactories:owned, capturedRobotFactories:captured});
  }
  assert(ownedFactories > 0 && capturedRobots > 0 && manualCase, 'Campaign coverage unexpectedly empty');

  // Capture must preserve a valid enemy choice and its unfinished work.
  const {source, id, alternative} = manualCase;
  const manual = captureFixture(source, id, 1);
  let state = init(manual);
  const workload = state.catalog.find(c => c.type === alternative).workload;
  const manualInput = manual.buildings.find(b => b.id === id);
  manualInput.product = alternative;
  manualInput.remainingWork = Math.floor(workload / 2);
  manualInput.progress = 1 - manualInput.remainingWork / workload;
  state = init(manual);
  assert.equal(byId(state, id).product, alternative);
  state = request({op:'tick', dt:0});
  let building = byId(state, id);
  assert.equal(building.team, 0);
  assert.equal(building.product, alternative);
  assert.equal(building.remainingWork, manualInput.remainingWork);
  assert.equal(building.progress, manualInput.progress);

  // The old stop command now selects the robot default. Repeating it must not
  // restart work that is already being spent on that default.
  state = request({op:'produce', id, product:null});
  building = byId(state, id);
  assert.equal(building.product, 'grunt');
  assert.equal(building.progress, 0);
  assert.equal(building.remainingWork, null);
  state = request({op:'advance', steps:10});
  const beforeReset = byId(state, id);
  assert(beforeReset.progress > 0);
  state = request({op:'produce', id, product:null});
  assert.equal(byId(state, id).remainingWork, beforeReset.remainingWork);
  assert.equal(byId(state, id).progress, beforeReset.progress);
  state = request({op:'produce', id, product:'not-a-product'});
  assert.equal(byId(state, id).product, 'grunt');
  assert.equal(byId(state, id).remainingWork, beforeReset.remainingWork);

  // Two successive completed squads must leave the same factory producing.
  const repeated = captureFixture(source, id, 0);
  repeated.sectors.forEach(s => { s.owner = 0; });
  const repeatedInput = repeated.buildings.find(b => b.id === id);
  repeatedInput.product = 'grunt'; repeatedInput.remainingWork = 1;
  state = init(repeated);
  const members = state.catalog.find(c => c.type === 'grunt').members;
  const firstCount = state.units.filter(u => u.team === 0 && u.type === 'grunt').length;
  state = request({op:'tick', dt:0.1});
  assert.equal(state.units.filter(u => u.team === 0 && u.type === 'grunt').length, firstCount + members);
  assert(state.events.some(event => event.kind === 'ready' && event.type === 'grunt' && event.team === 0));
  building = byId(state, id);
  const total = state.sectors.filter(s => s.countsForProduction !== false).length;
  const owned = state.sectors.filter(s => s.countsForProduction !== false && s.owner === 0).length;
  const rate = Math.trunc(Math.trunc(building.hp * 100 / building.maxHp) * (1 + (Math.trunc(owned * 100 / total) >> 2)) / 100);
  assert(rate > 0);
  state = request({op:'advance', steps:Math.ceil((building.remainingWork + 1) / (rate * 17) * 10) + 1});
  assert.equal(state.winner, null);
  assert.equal(state.units.filter(u => u.team === 0 && u.type === 'grunt').length, firstCount + 2 * members);
  building = byId(state, id);
  assert.equal(building.product, 'grunt');
  assert(building.remainingWork > 0);
  timer(state, building);

  const report = {ok:true, missions:missionResults, ownedFactories, capturedRobotFactories:capturedRobots,
    preservedManualProduction:true, nullReselectPreservesProgress:true, completedRobotSquads:2};
  if (options.report) fs.writeFileSync(options.report, JSON.stringify(report, null, 2) + '\n');
  console.log(`Automatic production passed: ${campaign.missions.length} missions, ${ownedFactories} owned factories, ${capturedRobots} neutral robot captures, preserved choices, null reset and two successive squads.`);
})().catch(error => { console.error(error.stack || error); process.exitCode = 1; });
