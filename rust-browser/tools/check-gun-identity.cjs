// Real Wasm lifecycle check. Reads only the private campaign schema; every
// placement and terrain cell used here is synthetic, not an original level.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),options={
 campaign:path.join(root,'dist/original/campaign.json'),
 wasm:path.join(root,'dist/independent/engine.wasm'),
};
for(let i=2;i<process.argv.length;i+=2){
 const key=process.argv[i].replace(/^--/,'');
 assert(['campaign','wasm'].includes(key),`Unknown option ${process.argv[i]}`);
 assert(process.argv[i+1],`Missing value for ${process.argv[i]}`);
 options[key]=path.resolve(process.argv[i+1]);
}
assert(fs.existsSync(options.campaign),'Missing private campaign: extract your own local game first.');
const campaign=JSON.parse(fs.readFileSync(options.campaign,'utf8'));
assert(campaign.missions?.length,'Missing campaign input schema.');
const clone=v=>JSON.parse(JSON.stringify(v));
const guns=['gatling','cannon','howitzer','missile'];
const robots=['grunt','tough','sniper','psycho','pyro','laser'];
const expected={gatling:{hp:50,damage:6,range:240},cannon:{hp:100,damage:75,range:240},
 howitzer:{hp:75,damage:100,range:400},missile:{hp:100,damage:100,range:320}};

function fixture(type,crew,{fort=false,nearEnemy=false,corner=false}={}){
 const m=clone(campaign.missions[0]),size=64,count=size*size;
 const anchor=corner?320:330;
 m.number=1;m.name='Synthetic gun identity';
 Object.assign(m.map,{width:size,height:size,tileSize:32,world:0,seed:912,revision:0,
  original:true,tiles:Array(count).fill(0),territory:Array(count).fill(0),
  bridgeBase:Array(count).fill(-1),bridgeHp:[],originalTiles:Array(count).fill(0),terrainCodes:Array(count).fill(0)});
 m.sectors=[{id:0,x:1320,y:1440,owner:0,home:true,countsForProduction:true}];
 m.buildings=[];m.originalObjects=[];m.crates=[];
 if(fort){
  // Deliberately isolate the fort ID from roof geometry: this checks identity
  // metadata through the ABI, not the separate rooftop boarding path problem.
  m.buildings=[{id:0,kind:'fort',team:0,sector:0,stars:1,x:1200,y:1200,w:240,h:240,
   hp:9999,maxHp:9999,product:'grunt',progress:0,readyGuns:[],
   gate:[1320,1440],orientation:'south',rawBase:[600,600]}];
 }
 m.spawns=[{type,team:-1,x:anchor,y:anchor,squad:1,...(fort?{fortId:0}:{})},
  {type:crew,team:0,x:anchor,y:anchor+10,squad:2},
  {type:'heavy',team:1,x:nearEnemy?400:1700,y:nearEnemy?320:1700,squad:3}];
 return m;
}

(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(options.wasm),{});
 const e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function request(v){
  const bytes=enc.encode(JSON.stringify(v)),ptr=e.alloc(bytes.length);
  new Uint8Array(e.memory.buffer,ptr,bytes.length).set(bytes);
  let length;try{length=e.run(ptr,bytes.length);}finally{e.free(ptr,bytes.length);}
  const state=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),length)));
  assert(!state.error,state.error);return state;
 }
 const init=m=>request({op:'init',mission:0,difficulty:1,seed:918,aiTeams:[],originalMission:m});
 const tick=()=>request({op:'tick',dt:.05});
 const get=(s,id)=>{const u=s.units.find(u=>u.id===id);assert(u,`Missing unit ${id}`);return u;};
 const catalog=(s,type)=>{const d=s.catalog.find(d=>d.type===type);assert(d,`Missing catalog ${type}`);return d;};
 function invariant(state,id,type,fort,baseline,phase,anchor=330){
  const u=get(state,id);
  assert.equal(u.type,type,`${phase}: crew must not turn ${type} into another gun`);
  assert.equal(u.fortId,fort?0:null,`${phase}: fort attachment must survive crew changes`);
  assert.equal(u.hp,expected[type].hp,`${phase}: gun hull HP remains its hardware type`);
  assert.equal(u.maxHp,expected[type].hp);
  assert.deepEqual(catalog(state,u.type),baseline,`${phase}: HUD/catalog entry remains the gun's own data`);
  assert.equal(baseline.class,2);assert.equal(baseline.speed,0);
  for(const field of ['hp','range','damage'])assert.equal(baseline[field],expected[type][field]);
  assert.equal(u.x,anchor);assert.equal(u.y,anchor);
 }
 function crew(state,robot,gun,type){
  request({op:'command',kind:'enter',ids:[robot],target:gun});
  for(let i=0;i<160;i++){
   state=tick();
   assert.equal(state.winner,null,'Fixture must remain active during crewing');
   if(get(state,gun).team===0){
    assert(state.events.some(v=>v.kind==='crewed'&&v.type===type),
     'The crew event reports the hardware type, not its incoming robot');
    return state;
   }
  }
  assert.fail(`Robot did not crew ${type}: ${JSON.stringify(state.units.filter(u=>u.id===robot||u.id===gun).map(u=>({id:u.id,type:u.type,team:u.team,x:u.x,y:u.y,order:u.order,target:u.target,path:u.path,holdPosition:u.holdPosition})))}`);
 }
 let lifecycles=0;
 for(const type of guns)for(const robotType of robots)for(const fort of [false,true]){
  let state=init(fixture(type,robotType,{fort}));
  const gun=state.units[0].id,robot=state.units[1].id,baseline=clone(catalog(state,type));
  invariant(state,gun,type,fort,baseline,'neutral');
  assert.equal(get(state,gun).team,-1);assert.equal(get(state,gun).crew,null);
  state=crew(state,robot,gun,type);
  invariant(state,gun,type,fort,baseline,'crewed');
  assert.equal(get(state,gun).crew.kind,robotType);
  assert(!state.units.some(u=>u.id===robot),'Only the original robot is consumed by successful boarding');
  state=request({op:'command',kind:'unload',ids:[gun]});
  invariant(state,gun,type,fort,baseline,'unloaded');
  assert.equal(get(state,gun).team,-1);assert.equal(get(state,gun).crew,null);
  const driver=state.units.find(u=>u.team===0&&u.type===robotType);
  assert(driver,'Unloading returns the actual incoming robot type');
  assert.equal(driver.fortId,null,'The robot does not inherit its hardware attachment');
  state=crew(state,driver.id,gun,type);
  invariant(state,gun,type,fort,baseline,'recrewed');
  assert.equal(get(state,gun).crew.kind,robotType);
  lifecycles++;
 }
 let cornerCases=0;
 for(const type of guns){
  let state=init(fixture(type,'grunt',{corner:true}));
  const gun=state.units[0].id,robot=state.units[1].id,baseline=clone(catalog(state,type));
  state=crew(state,robot,gun,type);
  state=request({op:'command',kind:'unload',ids:[gun]});
  const driver=state.units.find(u=>u.team===0&&u.type==='grunt');assert(driver);
  assert(Math.hypot(driver.x-320,driver.y-320)>28,'Released robot initially lies outside boarding radius');
  assert.equal(Math.floor(driver.x/32),10);assert.equal(Math.floor(driver.y/32),10);
  state=crew(state,driver.id,gun,type);
  invariant(state,gun,type,false,baseline,'same-cell corner reentry',320);
  cornerCases++;
 }
 let firing=0;
 for(const type of guns){
  let state=init(fixture(type,'grunt',{nearEnemy:true}));
  const gun=state.units[0].id,robot=state.units[1].id;
  state=crew(state,robot,gun,type);
  let shot=null;
  for(let i=0;i<20&&!shot;i++){
   state=tick();shot=state.events.find(v=>v.kind==='shot'&&v.team===0);
  }
  assert(shot,`Crewed ${type} must retain a firing weapon`);
  assert.equal(shot.type,type,'The real shot event identifies the gun, never the robot or another gun');
  assert.equal(get(state,gun).type,type);firing++;
 }
 console.log(`Gun identity passed with real Wasm: ${lifecycles} neutral/crewed/unloaded/recrewed lifecycles (4 guns ×6 crews ×ground/fort), ${cornerCases} exact tile-corner reentries, stable type/id/hull/stats/catalog/fortId and ${firing} correctly typed firing events. No Rust gun-type conversion reproduced; sprite/portrait mappings require separate rendering verification.`);
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
