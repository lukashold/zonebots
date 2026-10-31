// Two isolated controllers and the actual host Wasm: a guest uses its own workshop.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),source=fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),copy=x=>JSON.parse(JSON.stringify(x));
function api(){const window={};vm.runInNewContext(source,{window,Date,setTimeout,clearTimeout});return window.MultiplayerGame;}
function mission(){const n=32*32;return{map:{width:32,height:32,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(1),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:125,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0)},sectors:[{id:0,x:64,y:64,owner:0,home:true},{id:1,x:800,y:64,owner:1,home:true}],buildings:[{id:0,kind:'repair',team:0,sector:0,x:96,y:288,w:160,h:128,hp:1500,maxHp:1500,stars:0,product:null,progress:0,gate:[176,416],orientation:'south'},{id:1,kind:'repair',team:1,sector:1,x:560,y:288,w:160,h:128,hp:1500,maxHp:1500,stars:0,product:null,progress:0,gate:[640,416],orientation:'south'}],spawns:[{type:'grunt',team:0,x:544,y:640,squad:1},{type:'apc',team:1,x:480,y:640,squad:2},{type:'gatling',team:0,x:32,y:928,squad:3}],crates:[{x:544,y:640,taken:false}],originalObjects:[],walls:[]};}
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{}),e=instance.exports,calls=[];
 function run(v){calls.push(copy(v));const bytes=Buffer.from(JSON.stringify(v)),ptr=e.alloc(bytes.length);new Uint8Array(e.memory.buffer,ptr,bytes.length).set(bytes);let n;try{n=e.run(ptr,bytes.length);}finally{e.free(ptr,bytes.length);}const s=JSON.parse(Buffer.from(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 const toGuest=[],toHost=[],delivered=[],outbound=[];let guestRuns=0;
 const host=api().create({run}),guest=api().create({run(){guestRuns++;throw Error('Guest must never simulate');},apply:s=>delivered.push(copy(s))});
 host.bind({connected:true,queuedBytes:0,send(type,payload){const m=copy({type,payload});outbound.push(m);toGuest.push(m);return true;}},'host');guest.bind({connected:true,send(type,payload){toHost.push(copy({type,payload}));return true;}},'guest');
 function pump(){let limit=100;while(toGuest.length||toHost.length){assert(limit-->0);while(toGuest.length)guest.handle(toGuest.shift());while(toHost.length)host.handle(toHost.shift());}}
 assert.equal(host.start({op:'init',originalMission:mission(),seed:125},{name:'Workshop replication'}),true);pump();
 const car=host.state.units.find(u=>u.type==='apc'),shooter=host.state.units.find(u=>u.type==='grunt');
 for(let i=0;i<80&&host.state.units.find(u=>u.id===car.id).hp===car.maxHp;i++){host.tick(.05);pump();}assert(host.state.units.find(u=>u.id===car.id).hp<car.maxHp,'Combat must produce real repairable damage');
 host.action({op:'command',kind:'move',ids:[shooter.id],x:928,y:928});for(let i=0;i<3;i++){host.tick(.05);pump();}
 assert.equal(guest.viewState.units.find(u=>u.id===car.id).team,0);assert.equal(guest.viewState.buildings.find(b=>b.id===1).team,0);
 const before=calls.length;guest.action({op:'command',kind:'repairVehicle',ids:[car.id],target:0},'wrong-workshop');pump();assert.equal(calls.length,before,'The guest cannot use its opponent workshop');
 guest.action({op:'command',kind:'repairVehicle',ids:[car.id],target:1},'repair-own');pump();assert.equal(host.state.units.find(u=>u.id===car.id).order,'repairVehicle');assert.equal(guest.viewState.units.find(u=>u.id===car.id).order,'repairVehicle');
 let repairing=false,complete=false;for(let i=0;i<4000;i++){host.tick(.1);pump();const u=host.state.units.find(u=>u.id===car.id);assert(u);repairing ||= u.workshop?.phase==='repairing';if(u.hp===u.maxHp&&u.order!=='repairVehicle'){complete=true;break;}}
 for(let i=0;i<3;i++){host.tick(.05);pump();}assert(repairing);assert(complete);
 // A pause flush is a real network state barrier: between8Hz frames, the guest
 // is intentionally allowed to lag the host's20Hz cooldown counters.
 host.setPaused(true);pump();assert.equal(guest.state.time,host.state.time);const actual=host.state.units.find(u=>u.id===car.id),remote=guest.state.units.find(u=>u.id===car.id);assert.deepEqual(copy(remote),copy(actual),'Both peers must agree on the restored vehicle and completion state');assert.equal(remote.hp,remote.maxHp);assert.equal(delivered.flatMap(s=>s.events).filter(v=>v.kind==='vehicleRepaired').length,1,'Exactly one repair completion reaches the guest');assert(outbound.some(m=>m.type==='game:frame'&&m.payload.state.units.some(u=>u.id===car.id&&u.workshop?.phase==='repairing')));assert.equal(guestRuns,0);host.close();guest.close();
 console.log('Workshop multiplayer passed: real combat damage, guest ownership/perspective, validated workshop order, authoritative entry/repair/exit, identical restored vehicle, one completion event and no guest simulation.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
