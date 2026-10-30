// Verify the current Wasm ABI with privately extracted campaign input.
// No original maps, campaign placements, or assets are embedded in this file.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const options = {campaign:path.join(root,'dist/original/campaign.json'),wasm:path.join(root,'dist/independent/engine.wasm'),steps:200};
for(let i=2;i<process.argv.length;i+=2){
  const key=process.argv[i].replace(/^--/,'');
  assert(['campaign','wasm','steps','report'].includes(key),`Unknown option ${process.argv[i]}`);
  assert(process.argv[i+1],`Missing value for ${process.argv[i]}`);
  options[key]=key==='steps'?Number(process.argv[i+1]):path.resolve(process.argv[i+1]);
}
assert(Number.isInteger(options.steps)&&options.steps>0&&options.steps<=30000,'--steps must be 1..30000');
assert(fs.existsSync(options.campaign),'Missing private campaign: run python tools/extract-original-campaign.py --game <local-game-directory>');
assert(fs.existsSync(options.wasm),'Missing Wasm: run independent-engine/build.ps1');
const campaign=JSON.parse(fs.readFileSync(options.campaign,'utf8'));
const order=['gatling','cannon','howitzer','missile','grunt','psycho','tough','sniper','pyro','laser','jeep','apc','light','medium','heavy','launcher','crane'];
const clone=v=>JSON.parse(JSON.stringify(v));
const groups=new Map(campaign.groups.map(g=>[g.type,g]));
const expectedMenu=b=>b.hp<=0?[]:order.filter(k=>{
  const required=groups.get(k).buildLevels[b.kind]||0;
  return required>0&&required<=b.stars;
});
function finiteState(value,at='state'){
  if(typeof value==='number')assert(Number.isFinite(value),`${at} is not finite`);
  else if(Array.isArray(value))value.forEach((v,i)=>finiteState(v,`${at}[${i}]`));
  else if(value&&typeof value==='object')for(const [k,v]of Object.entries(value))finiteState(v,`${at}.${k}`);
}
function inBounds(r){
  assert(Number.isFinite(r.time),'Simulation time is not finite');
  for(const u of r.units){
    for(const field of ['x','y','hp','maxHp','angle','aim','cooldown'])assert(Number.isFinite(u[field]),`Unit ${u.id}.${field} is not finite`);
    assert(u.x>=0&&u.x<r.map.width*r.map.tileSize&&u.y>=0&&u.y<r.map.height*r.map.tileSize,`Unit ${u.id} outside mission ${r.mission+1}`);
  }
  for(const b of r.buildings)for(const field of ['x','y','hp','maxHp','progress'])assert(Number.isFinite(b[field]),`Building ${b.id}.${field} is not finite`);
}
(async()=>{
  const {instance}=await WebAssembly.instantiate(fs.readFileSync(options.wasm),{});
  const e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
  function request(v){
    const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);
    new Uint8Array(e.memory.buffer,p,b.length).set(b);
    let n;
    try{n=e.run(p,b.length);}finally{e.free(p,b.length);}
    const r=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));
    assert(!r.error,r.error);return r;
  }
  function init(m,extra={}){return request({op:'init',mission:m.number-1,difficulty:1,seed:9821+m.number,aiTeams:[],originalMission:m,...extra});}
  assert.equal(campaign.missions.length,20);
  const results=[];
  for(const m of campaign.missions){
    const first=init(m);
    assert.equal(first.map.original,true);
    for(const field of ['width','height','tileSize','world'])assert.equal(first.map[field],m.map[field]);
    for(const field of ['tiles','territory','originalTiles','terrainCodes','bridgeBase','bridgeHp'])assert.deepEqual(first.map[field],m.map[field],`Mission ${m.number} map ${field}`);
    assert.equal(first.catalog.length,19);
    const catalog=new Map(first.catalog.map(d=>[d.type,d]));
    assert.equal(first.units.length,m.spawns.length,`Mission ${m.number}: initial unit count`);
    first.units.forEach((u,i)=>{
      const source=m.spawns[i],d=catalog.get(source.type);
      assert.equal(u.type,source.type);assert.equal(u.team,source.team);
      assert.equal(u.x,source.x);assert.equal(u.y,source.y);
      assert.equal(u.hp,d.hp);assert.equal(u.maxHp,d.hp);
    });
    assert.equal(first.buildings.length,m.buildings.length);
    assert.equal(first.crates.length,m.crates.length,`Mission ${m.number}: crates must deserialize`);
    assert(first.buildMenus,'Rebuild Wasm: buildMenus is absent');
    first.buildings.forEach((b,i)=>{
      const source=m.buildings[i];
      for(const field of ['kind','team','sector','x','y','w','h','hp','maxHp','stars'])assert.equal(b[field],source[field],`Mission ${m.number} building ${i}.${field}`);
      assert.deepEqual(first.buildMenus[i],expectedMenu(b),`Mission ${m.number} building ${i}: menu order/technology`);
    });
    // Verify authoritative eligibility through a mutating command as well as menu output.
    const factory=first.buildings.find(b=>b.team===0&&first.buildMenus[b.id].length);
    assert(factory,'No controllable factory');
    const allowed=first.buildMenus[factory.id],choice=allowed.at(-1);
    let state=request({op:'produce',id:factory.id,product:choice});
    assert.equal(state.buildings[factory.id].product,choice);
    state=request({op:'produce',id:factory.id,product:'truck'});
    assert.equal(state.buildings[factory.id].product,choice,'Disabled product must be rejected');
    state=init(m,{aiTeams:[0,1]});
    state=request({op:'advance',steps:options.steps});
    finiteState(state);inBounds(state);assert(state.time>0);
    const row={mission:m.number,width:state.map.width,height:state.map.height,initialUnits:first.units.length,crates:first.crates.length,buildings:first.buildings.length,time:state.time,remainingUnits:state.units.length,winner:state.winner};
    results.push(row);console.log(JSON.stringify(row));
  }
  // Controlled damage and territory cases exercise integer truncation and 17 Hz
  // work consumption. Spatial input still comes exclusively from the private map.
  const formulaCases=[];
  for(const fraction of [1,0.5,0.001])for(const ownAll of [false,true]){
    const m=clone(campaign.missions[0]);
    if(ownAll)for(const sector of m.sectors)if(sector.countsForProduction!==false)sector.owner=0;
    const b=m.buildings.find(b=>b.kind==='fort'&&b.team===0);
    b.hp=b.maxHp*fraction;b.product='grunt';b.progress=0;b.remainingWork=null;
    const first=init(m),index=first.buildings.findIndex(x=>x.id===b.id);
    const workload=first.catalog.find(d=>d.type==='grunt').workload;
    assert(Number.isInteger(workload)&&workload>0,'Rebuild Wasm: catalog.workload is absent');
    const total=first.sectors.filter(s=>s.countsForProduction!==false).length;
    const owned=first.sectors.filter(s=>s.owner===0&&s.countsForProduction!==false).length;
    const territory=1+(Math.trunc(owned*100/total)>>2);
    const rate=Math.trunc(Math.trunc(b.hp*100/b.maxHp)*territory/100);
    assert.equal(first.productionSeconds[index],rate?Math.trunc(workload/(rate*17)):null);
    const after=request({op:'advance',steps:10});
    assert.deepEqual(after.sectors.map(s=>s.owner),first.sectors.map(s=>s.owner),'Formula fixture unexpectedly captured a sector');
    assert.equal(after.buildings[index].remainingWork,workload-rate*17);
    assert.equal(after.productionSeconds[index],rate?Math.trunc((workload-rate*17)/(rate*17)):null);
    formulaCases.push({fraction,ownAll,total,owned,rate,workload,remainingWork:after.buildings[index].remainingWork});
  }
  const report={missions:results,productionCases:formulaCases,wasmSha256:require('node:crypto').createHash('sha256').update(fs.readFileSync(options.wasm)).digest('hex')};
  if(options.report){fs.mkdirSync(path.dirname(options.report),{recursive:true});fs.writeFileSync(options.report,JSON.stringify(report,null,2));}
  console.log('PASS: 20 original missions, map arrays, initial unit/building/crate counts, HP, technology menus, production arithmetic, and finite state.');
})().catch(error=>{console.error(error);process.exitCode=1;});
