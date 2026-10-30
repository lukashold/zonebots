// Same browser adapter, tested without DOM against generated and owned original maps.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const base=path.resolve(__dirname,'..');
for(const file of ['data','map','zdata','game'])require(path.join(base,'dist','js',file+'.js'));
(async()=>{
 await ZRust.load(fs.readFileSync(path.join(base,'dist','engine.wasm')));
 const ZG=globalThis.ZG;
 const results=[];
 const original=process.argv[2];
 if(original){const files={};for(const name of fs.readdirSync(original))if(ZG.Z.wanted(name))files[name.toLowerCase()]=new Uint8Array(fs.readFileSync(path.join(original,name)));ZG.Z.use(files);}
 for(let k=0;k<20;k++){
  const g=ZG.newGame(original?ZG.Z.makeLevel(k):k,{aiTeams:[0,1]});
  assert.equal(g.engine,'rust-wasm');
  const before=g.units[0], p={x:before.x,y:before.y};
  ZG.orderMove(g,[before],p.x+64,p.y+64);
  for(let i=0;i<5;i++)ZG.step(g,0.04);
  assert.strictEqual(g.units.find(u=>u.id===before.id),before,'UI unit identity must be stable');
  const r=ZRust.call({op:'advance',steps:12000,dt:0.1});
  for(const u of r.units){assert(Number.isFinite(u.x)&&Number.isFinite(u.y)&&Number.isFinite(u.hp));assert(u.x>=0&&u.y>=0&&u.x<=g.map.W*32&&u.y<=g.map.H*32);}
  assert(r.over,`Level ${k+1}: AI game did not finish within 1200 s`);
  results.push({level:k+1,world:g.lv.theme,winner:r.over.winner,seconds:Math.round(r.time),units:r.units.length});
  console.log(JSON.stringify(results.at(-1)));
 }
 console.log('PASS: 20 maps, Rust/Wasm simulation, finite state and completed matches.');
})().catch(e=>{console.error(e);process.exitCode=1});
