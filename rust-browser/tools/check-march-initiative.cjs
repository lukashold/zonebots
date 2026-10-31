'use strict';
// Actual Rust/Wasm command lifecycle. All terrain, units and placements below
// are synthetic; no original map, sprite or extracted game file is embedded.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),args=process.argv.slice(2);
let wasm=path.join(root,'dist/independent/engine.wasm');
if(args.length){assert.equal(args.length,2,'Usage: node tools/check-march-initiative.cjs [--wasm filename]');assert.equal(args[0],'--wasm');wasm=path.resolve(args[1]);}
const goal=[1500,400],start=[320,400],clone=v=>JSON.parse(JSON.stringify(v));
function fixture({team=0,original=true,flags=true,hardware='jeep',squad=1,near=false}={}){
 const width=64,height=64,n=width*height;
 return{map:{width,height,tileSize:32,world:0,seed:7129,revision:0,original,
  tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0)},
  sectors:flags?[{id:0,x:560,y:500,owner:-1,home:false,countsForProduction:true},{id:1,x:820,y:300,owner:-1,home:false,countsForProduction:true}]:[],
  buildings:[],crates:[],originalObjects:[],walls:[],spawns:[
   ...Array.from({length:squad},(_,i)=>({type:'grunt',team,x:start[0]+i*12,y:start[1],squad:1})),
   ...(hardware?[{type:hardware,team:-1,x:near?460:1100,y:near?480:490,squad:2}]:[]),
   {type:'heavy',team:1-team,x:1900,y:1800,squad:3}]};
}

(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(wasm),{}),e=instance.exports,enc=new TextEncoder,dec=new TextDecoder;let state;
 function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let length;try{length=e.run(p,b.length);}finally{e.free(p,b.length);}const next=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),length)));assert(!next.error,next.error);state={...state,...next};return state;}
 const init=options=>{state=null;return req({op:'init',mission:0,difficulty:1,seed:7281,aiTeams:[],originalMission:fixture(options)});};
 const tick=()=>req({op:'tick',dt:.1,compact:true,knownMapSeed:7129,knownMapRevision:0});
 const get=id=>{const u=state.units.find(u=>u.id===id);assert(u,`Missing unit ${id}`);return u;};
 const move=(ids,team,destination=goal)=>req({op:'command',kind:'move',ids,team,x:destination[0],y:destination[1]});
 const at=(u,p)=>Math.hypot(u.x-p[0],u.y-p[1])<1;
 function until(predicate,limit,label,inspect=()=>{}){for(let i=0;i<limit;i++){tick();assert.equal(state.winner,null,`${label}: synthetic fixture remains active`);inspect();if(predicate())return;}assert.fail(`${label}: timed out: ${JSON.stringify({time:state.time,sectors:state.sectors,units:state.units.map(u=>({id:u.id,type:u.type,team:u.team,x:u.x,y:u.y,order:u.order,target:u.target,dest:u.dest,path:u.path,initiative:u.initiative}))})}`);}
 let scenarios=0,captures=0;
 // The flags sit 100px off the original straight route: touching the march
 // line cannot accidentally pass this test without actual opportunistic detours.
 for(const team of [0,1]){
  init({team});const robot=state.units[0].id,car=state.units.find(u=>u.team<0).id;
  move([robot],team);assert(get(robot).path.length>0,'Regression begins with an active explicit march.');
  const captured=new Set;let detours=0,crewed=false;
  until(()=>get(car).team===team&&at(get(car),goal),700,'flags then Jeep then original march',()=>{
   for(const s of state.sectors)if(s.owner===team)captured.add(s.id);
   const u=state.units.find(u=>u.id===robot);if(u?.initiative?.kind==='flag')detours++;
   if(get(car).team===team){crewed=true;assert.equal(get(car).crew.kind,'grunt');assert(!state.units.some(u=>u.id===robot),'Successful crewing consumes the incoming robot exactly once.');assert.deepEqual(get(car).dest,goal,'The occupied mobile vehicle inherits the original march destination.');}
  });
  assert.equal(captured.size,2,'The soldier must detour to both flags along the journey.');assert(detours>0,'A flag detour must have been observed before successful capture.');assert(crewed);assert.equal(get(car).type,'jeep');captures+=captured.size;scenarios++;
 }
 // A squad must not consume several soldiers for the same empty vehicle, and
 // the soldiers left outside keep their own formation-adjusted destinations.
 for(const team of [0,1]){
  init({team,flags:false,squad:3,near:true});const robots=state.units.filter(u=>u.type==='grunt').map(u=>u.id),car=state.units.find(u=>u.team<0).id;
  move(robots,team);const destinations=new Map(robots.map(id=>[id,clone(get(id).dest)]));let consumed;
  until(()=>get(car).team===team&&get(car).path.length===0&&state.units.filter(u=>robots.includes(u.id)).every(u=>at(u,destinations.get(u.id))),650,'squad crewing and march continuation',()=>{
   const claimants=state.units.filter(u=>u.initiative?.kind==='vehicle'&&u.target===car);assert(claimants.length<=1,'Only one automatic claimant may reserve a hardware object.');
   if(get(car).team===team){const missing=robots.filter(id=>!state.units.some(u=>u.id===id));assert.equal(missing.length,1);consumed=missing[0];assert.deepEqual(get(car).dest,destinations.get(consumed));}
  });
  assert(consumed);assert.equal(state.units.filter(u=>u.team===team&&u.type==='grunt').length,2);assert(at(get(car),destinations.get(consumed)));scenarios++;
 }
 // A gun can be crewed on the way, but its fixed mounting cannot inherit a
 // drive order. The old soldier's march must not become a path for the gun.
 for(const team of [0,1]){
  init({team,flags:false,hardware:'gatling',near:true});const robot=state.units[0].id,gun=state.units.find(u=>u.team<0).id,parked=[get(gun).x,get(gun).y];move([robot],team);
  until(()=>get(gun).team===team,180,'stationary hardware on march');assert(!state.units.some(u=>u.id===robot));assert.equal(get(gun).order,'guard');assert.equal(get(gun).dest,null);assert.equal(get(gun).path.length,0);assert(at(get(gun),parked));scenarios++;
 }
 // Stopping during an already active detour must release the claim and hold;
 // replacing it with a new march must never resurrect the old distant goal.
 init({flags:false,near:true});const robot=state.units[0].id,car=state.units.find(u=>u.team<0).id;move([robot],0);
 until(()=>get(robot).initiative?.kind==='vehicle',20,'enter detour acquired');req({op:'command',kind:'stop',ids:[robot],team:0});const stopped=[get(robot).x,get(robot).y];
 for(let i=0;i<50;i++)tick();assert(at(get(robot),stopped),'Stop holds position instead of finishing a saved march.');assert.equal(get(car).team,-1);assert.equal(get(robot).initiative,null);scenarios++;
 move([robot],0);until(()=>get(robot).initiative?.kind==='vehicle',20,'new detour acquired');const replacement=[120,400];move([robot],0,replacement);assert.deepEqual(get(robot).dest,replacement,'New command takes effect immediately.');
 until(()=>get(car).team===0&&at(get(car),replacement),450,'replacement march');assert.deepEqual(get(car).dest,replacement);assert(!at(get(car),goal));scenarios++;
 // Explicit combat is not an ordinary march and retains its chosen target,
 // even while passing objects that would be acquired by a move command.
 init({flags:false,near:true});const attacker=state.units[0].id,vehicle=state.units.find(u=>u.team<0).id,enemy=state.units.find(u=>u.team===1).id;
 req({op:'command',kind:'attack',ids:[attacker],target:enemy,team:0,x:1900,y:1800});for(let i=0;i<25;i++){tick();assert.equal(get(attacker).order,'attack');assert.equal(get(attacker).target,enemy);assert.equal(get(attacker).initiative,null);assert.equal(get(vehicle).team,-1);}scenarios++;
 // Original-map initiative changes must not alter the procedural rule set.
 for(const team of [0,1]){
  init({team,original:false});const soldier=state.units[0].id,car=state.units.find(u=>u.team<0).id;move([soldier],team);
  until(()=>at(get(soldier),goal),500,'procedural straight march');assert(state.sectors.every(s=>s.owner===-1));assert.equal(get(car).team,-1);assert.equal(get(soldier).initiative,null);scenarios++;
 }
 console.log(JSON.stringify({scenarios,capturedFlags:captures}));
 console.log('PASS: actual Wasm marching soldiers capture off-route flags, crew nearby hardware once, continue the saved march in mobile vehicles, preserve squad destinations, hold stationary guns, obey Stop/replacement/combat orders and leave procedural movement unchanged.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
