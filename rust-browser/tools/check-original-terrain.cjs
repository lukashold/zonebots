// Private original-campaign references remain local. Synthetic maps verify raw
// native terrain codes even when compatibility tile colours intentionally differ.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const options={wasm:path.join(root,'dist/independent/engine.wasm'),campaign:path.join(root,'dist/original/campaign.json')};
for(let i=2;i<process.argv.length;i+=2){const key=process.argv[i].replace(/^--/,'');assert(['wasm','campaign'].includes(key));assert(process.argv[i+1]);options[key]=path.resolve(process.argv[i+1]);}
const copy=v=>JSON.parse(JSON.stringify(v));
const wet=new Set([9,38,39,40,42]),vehicles=new Set(['jeep','light','medium','heavy','apc','crane','launcher','truck','firetruck']);
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(options.wasm),{}),e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function request(v){const data=enc.encode(JSON.stringify(v)),p=e.alloc(data.length);new Uint8Array(e.memory.buffer,p,data.length).set(data);let n;try{n=e.run(p,data.length);}finally{e.free(p,data.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 function fixture(code,type='jeep',world=0){
  const width=64,height=20,n=width*height;
  const map={width,height,tileSize:32,world,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:9187,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),waterTile:0};
  // A full-height barrier cannot be bypassed through the impassable outer edge.
  for(let y=0;y<height;y++)map.terrainCodes[y*width+10]=code;
  return {map,sectors:[{id:0,x:1984,y:592,owner:1,home:true,countsForProduction:true}],buildings:[],crates:[],originalObjects:[],spawns:[{type,team:0,x:144,y:336,squad:1},{type:'gatling',team:1,x:1968,y:592,squad:2}]};
 }
 const init=m=>request({op:'init',mission:0,difficulty:1,seed:9187,aiTeams:[],originalMission:m});
 const player=s=>s.units.find(u=>u.team===0);
 function order(s,x=496,y=336){return request({op:'command',team:0,kind:'move',ids:[player(s).id],x,y});}
 function crosses(s){return player(s).path.some(([x])=>Math.floor(x/32)>10);}
 // Sample the geometric route, not the presence of a redundant bridge waypoint.
 function routeCells(u){const cells=[];let a=[u.x,u.y];for(const b of u.path){const n=Math.max(1,Math.ceil(Math.hypot(b[0]-a[0],b[1]-a[1])));for(let i=1;i<=n;i++){const t=i/n;cells.push([Math.floor((a[0]+(b[0]-a[0])*t)/32),Math.floor((a[1]+(b[1]-a[1])*t)/32)]);}a=b;}return cells;}
 function pathAvoidsWater(s,u=player(s)){
  for(const [x,y]of routeCells(u)){const j=y*s.map.width+x,code=s.map.terrainCodes[j];assert(!wet.has(code),`${u.type} path enters native wet code${code} at cell(${x},${y})`);}
 }

 let shallowCases=0;
 for(const world of [0,2,3,4])for(const code of [9,38,39,42]){
  let s=order(init(fixture(code,'grunt',world)));assert(crosses(s),`Infantry should cross code${code},world${world}`);
  s=order(init(fixture(code,'jeep',world)));assert(!crosses(s),`Vehicle crossed raw code${code},world${world}`);pathAvoidsWater(s);shallowCases++;
 }
 for(const world of [0,1,2,3,4])for(const type of ['grunt','jeep']){
  const s=order(init(fixture(40,type,world)));assert(!crosses(s),`${type} crossed deep water/lava40,world${world}`);
 }
 for(const code of [9,38,39,40,42]){const s=order(init(fixture(code,'grunt',1)));assert(!crosses(s),`Infantry crossed volcanic wet code${code}`);}
 for(const code of [7,51,23])for(const type of ['grunt','jeep']){
  const s=order(init(fixture(code,type)));assert(crosses(s),`${type} failed dry road/intact bridge code${code}`);
 }
 for(const code of [19,43])for(const type of ['grunt','jeep']){const s=order(init(fixture(code,type)));assert(!crosses(s),`${type} crossed blocked native terrain${code}`);}
 // Mixed commands must apply the swimmer predicate per unit, not per selection.
 const mixed=fixture(9,'grunt');mixed.spawns.push({type:'jeep',team:0,x:176,y:336,squad:3});let s=init(mixed);
 s=request({op:'command',team:0,kind:'move',ids:s.units.filter(u=>u.team===0).map(u=>u.id),x:496,y:336});
 assert(s.units.find(u=>u.type==='grunt'&&u.team===0).path.some(([x])=>x>352));pathAvoidsWater(s,s.units.find(u=>u.type==='jeep'));
 // Plan across a live bridge, destroy it while the vehicle is still on shore,
 // then ensure movement cannot consume the now-invalid path into the water.
 const bridge=fixture(9);const bj=10*bridge.map.width+10;
 bridge.map.terrainCodes[bj]=23;bridge.map.tiles[bj]=5;bridge.map.bridgeBase[bj]=0;bridge.map.bridgeHp=[1];
 bridge.spawns.push({type:'heavy',team:0,x:272,y:336,squad:3});s=order(init(bridge));
 const car=player(s).id,attacker=s.units.find(u=>u.type==='heavy').id;
 assert(routeCells(s.units.find(u=>u.id===car)).some(([x,y])=>x===10&&y===10),'Vehicle should initially use the intact bridge');
 s=request({op:'command',team:0,kind:'attackBridge',ids:[attacker],x:336,y:336});
 s=request({op:'tick',dt:0.1});assert(s.map.bridgeHp[0]<=0,'Fixture shot must destroy bridge');assert.equal(s.map.tiles[bj],3);
 for(let i=0;i<160;i++){
  s=request({op:'tick',dt:0.1});const u=s.units.find(u=>u.id===car);assert(u,'Vehicle on shore must survive');assert(u.x<320,`Stale bridge path drove vehicle into water at${u.x},${u.y}`);
 }
 const campaign=JSON.parse(fs.readFileSync(options.campaign,'utf8'));let realCases=0;
 for(const [number,target]of [[3,[1360,2064]],[9,[1168,2864]],[9,[2960,2064]]]){
  const m=copy(campaign.missions.find(m=>m.number===number)),unit=m.spawns.find(u=>u.team===0&&vehicles.has(u.type));assert(unit);
  const targetIndex=Math.floor(target[1]/32)*m.map.width+Math.floor(target[0]/32);assert(wet.has(m.map.terrainCodes[targetIndex]));
  assert.equal(m.map.tiles[targetIndex],3,'Private campaign compatibility map must label native38/42 as water');
  const far=[32,32];m.spawns=[{...unit,squad:1},{type:'gatling',team:1,x:far[0],y:far[1],squad:2}];m.crates=[];m.originalObjects=[];
  m.buildings.forEach(b=>{b.team=-1;b.product=null;});m.sectors.forEach(s=>{s.owner=-1;});
  s=order(init(m),...target);pathAvoidsWater(s);realCases++;
 }
 console.log(`Original terrain passed: ${shallowCases} shallow-water/world combinations, deep water/lava, volcanic restrictions, road/bridge, mixed orders, stale destroyed-bridge path and ${realCases} actual campaign regressions.`);
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
