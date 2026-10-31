// Real Wasm authority plus two isolated clients; no browser or private artwork required.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),source=fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8');
const clone=value=>JSON.parse(JSON.stringify(value));
function module(){const window={};vm.runInNewContext(source,{window,setTimeout,clearTimeout,Date:class extends Date{static now(){return 1000;}}});return window.MultiplayerGame;}
function fixture(){
 const width=32,height=32,n=width*height,tiles=Array(n).fill(0),bridgeBase=Array(n).fill(-1),terrainCodes=Array(n).fill(0),bridge=17*width+23;
 tiles[bridge]=5;bridgeBase[bridge]=0;terrainCodes[bridge]=23;
 return {map:{width,height,tileSize:32,world:0,tiles,territory:Array.from({length:n},(_,i)=>i%width<16?0:1),bridgeBase,bridgeHp:[1],seed:27,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes},
  sectors:[{id:0,x:128,y:900,owner:0,home:true,countsForProduction:true},{id:1,x:900,y:128,owner:1,home:true,countsForProduction:true}],
  buildings:[{id:0,kind:'fort',team:0,sector:0,x:64,y:64,w:128,h:160,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0,remainingWork:1},
   {id:1,kind:'fort',team:1,sector:1,x:800,y:800,w:128,h:160,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0,remainingWork:1},
   {id:2,kind:'robot',team:1,sector:1,x:704,y:64,w:96,h:128,hp:2000,maxHp:2000,stars:5,product:'grunt',progress:0,remainingWork:1,readyGuns:['cannon']}],
  spawns:[{type:'grunt',team:0,x:224,y:256,squad:1},{type:'grunt',team:1,x:736,y:512,squad:2},{type:'apc',team:1,x:800,y:512,squad:3},{type:'gatling',team:-1,x:680,y:320,squad:4}],crates:[],originalObjects:[]};
}
async function rawEngine(){
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{}),e=instance.exports,encoder=new TextEncoder(),decoder=new TextDecoder(),calls=[];
 return {calls,run(request){calls.push(clone(request));const b=encoder.encode(JSON.stringify(request)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const result=JSON.parse(decoder.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!result.error,result.error);return result;}};
}
async function run(){
 const engine=await rawEngine(),hostApi=module(),guestApi=module(),outbound=[],toGuest=[],toHost=[],hostViews=[],guestViews=[],hostStarts=[],guestStarts=[],hostPauses=[],guestPauses=[],previews=[],results=[],infos=[];
 const host=hostApi.create({run:v=>engine.run(v),apply:s=>hostViews.push(s),onStart:(m,r)=>hostStarts.push([m,r]),onPause:(p,r)=>hostPauses.push([p,r]),onInfo:i=>infos.push(i)});
 const guest=guestApi.create({run:()=>{throw Error('Guest must never execute the engine');},apply:s=>guestViews.push(s),onStart:(m,r)=>guestStarts.push([m,r]),onPause:(p,r)=>guestPauses.push([p,r]),onPreview:(r,id)=>previews.push([r,id]),onAction:(r,id)=>results.push([r,id]),onInfo:i=>infos.push(i)});
 const hostSession={connected:true,queuedBytes:0,send(type,payload){const m=clone({type,payload});outbound.push(m);toGuest.push(m);return true;}};
 const guestSession={connected:true,queuedBytes:0,send(type,payload){toHost.push(clone({type,payload}));return true;}};
 host.bind(hostSession,'host');guest.bind(guestSession,'guest');
 function pump(){let remaining=100;while(toGuest.length||toHost.length){assert(remaining-->0,'Network pump must settle');while(toGuest.length)guest.handle(toGuest.shift());while(toHost.length)host.handle(toHost.shift());}}
 assert.equal(host.start({op:'init',mission:0,difficulty:0,seed:529,originalMission:fixture()}, {name:'Synthetic PvP'}),true);
 assert.equal(host.active,true);assert.equal(host.waiting,true);assert.equal(host.paused,true);assert.equal(host.tick(.05),null);assert.equal(guest.tick(.05),null);
 assert.equal(hostStarts.length,1);assert.equal(hostStarts[0][1],'host');assert(hostViews.length,'Host state must be applied before onStart');
 const init=outbound.find(m=>m.type==='game:init');assert(init.payload.state.map&&init.payload.state.catalog&&init.payload.state.objects);assert.equal(init.payload.state.difficulty,1);assert.equal(init.payload.state.multiplayer,true);
 assert(!JSON.stringify(init).includes('sprites.png')&&!JSON.stringify(init).includes('audio.json'),'State transfer must not bundle original artwork/audio');
 pump();assert.equal(host.waiting,false);assert.equal(host.paused,false);assert.equal(guest.paused,false);assert.equal(guest.active,true);assert.equal(guestStarts[0][1],'guest');
 assert.equal(guestStarts[0][0].name,'Synthetic PvP');
 const red=host.state.units.find(u=>u.team===0).id,blue=host.state.units.find(u=>u.team===1&&u.type==='grunt').id,neutral=host.state.units.find(u=>u.team===-1).id;
 const unit=(s,id)=>s.units.find(u=>u.id===id);
 assert.equal(unit(guest.viewState,blue).team,0);assert.equal(unit(guest.viewState,red).team,1);assert.equal(unit(guest.viewState,neutral).team,-1);
 assert.equal(guest.viewState.sectors[0].owner,1);assert.equal(guest.viewState.sectors[1].owner,0);assert.equal(guest.viewState.buildings[2].team,0);
 const firstMap=guest.state.map;assert.equal(guest.viewState.map,firstMap);
 assert.equal(guest.action({op:'command',team:0,kind:'attackBridge',ids:[blue],x:736,y:544},'bridge'),null);pump();
 assert.equal(unit(host.state,blue).order,'attackBridge');assert.equal(results.at(-1)[1],'bridge');assert.equal(unit(guest.viewState,blue).order,'attackBridge');
 for(let i=0;i<3;i++)host.tick(.05);pump();
 assert(host.state.time>.14&&guest.state.time>.14);assert.equal(host.state.map.revision,1);assert.equal(guest.state.map.revision,1);assert.equal(guest.state.map.tiles[17*32+23],3);assert.notEqual(guest.state.map,firstMap,'A bridge revision must refresh geometry');
 for(const key of ['units','buildings','sectors','map'])assert.deepEqual(clone(guest.state[key]),clone(host.state[key]),`Both peers must share canonical ${key}`);
 const deliveredEvents=guestViews.flatMap(s=>s.events);
 assert.equal(deliveredEvents.filter(e=>e.kind==='bridgeDestroyed').length,1,'Events from intervening20Hz ticks must be aggregated into one8Hz frame');
 assert(deliveredEvents.some(e=>e.kind==='ready'),'Production events between frames must not be lost');
 const revisedMap=guest.state.map;
 for(let i=0;i<3;i++)host.tick(.05);pump();assert.equal(guest.state.map,revisedMap,'Unchanged geometry must preserve cache identity');
 const frame=outbound.filter(m=>m.type==='game:frame').at(-1);for(const field of ['map','catalog','objects'])assert(!Object.hasOwn(frame.payload.state,field),`Warm network frame must omit ${field}`);
 const beforeForged=engine.calls.length;
 for(const request of [{op:'tick',dt:10},{op:'init'},{op:'produce',id:0,product:'psycho'},{op:'command',kind:'move',ids:[red],x:600,y:600,team:0},{op:'command',kind:'move',ids:[blue],x:NaN,y:600},{op:'command',kind:'anything',ids:[blue],x:600,y:600}])host.handle({type:'game:action',payload:{epoch:1,requestId:'forged',request}});
 assert.equal(engine.calls.length,beforeForged,'Peer cannot tick/init, change the opponent factory, command the opponent or pass invalid commands');pump();
 guest.action({op:'produce',id:2,product:'psycho'},'production');pump();assert.equal(host.state.buildings[2].product,'psycho');assert.equal(guest.viewState.buildings[2].team,0);
 assert.equal(guest.requestPreview({op:'previewGun',id:2,slot:0,x:896,y:256},'preview'),null);pump();assert.equal(previews.at(-1)[1],'preview');assert.equal(previews.at(-1)[0].valid,true);
 guest.action({op:'placeGun',id:2,slot:0,x:896,y:256},'place');pump();assert.equal(results.at(-1)[1],'place');assert.equal(results.at(-1)[0].placementResult.valid,true);
 assert(host.state.units.some(u=>u.type==='cannon'&&u.team===1&&u.x===896&&u.y===256));assert(guest.viewState.units.some(u=>u.type==='cannon'&&u.team===0&&u.x===896&&u.y===256));
 assert.equal(host.state.buildings[2].readyGuns.length,0);
 host.tick(.05);pump();assert.equal(host.state.placementResult,undefined,'A placement acknowledgement must not become a permanent snapshot field');
 const time=host.state.time;guest.setPaused(true);assert.equal(guest.paused,true);pump();assert.equal(host.paused,true);assert.equal(host.tick(.05),null);assert.equal(host.state.time,time);
 guest.setPaused(false);assert.equal(guest.paused,true,'Guest must wait for host confirmation before resuming');pump();assert.equal(host.paused,false);assert.equal(guest.paused,false);
 const priorOps=engine.calls.length;assert.equal(guest.tick(.05),null);assert.equal(engine.calls.length,priorOps,'Guest simulation never advances');
 // A stale or forged state frame cannot change host authority or roll back the guest.
 assert.equal(host.handle({type:'game:frame',payload:frame.payload}),false);
 const guestTime=guest.state.time;assert.equal(guest.handle({type:'game:frame',payload:frame.payload}),false);assert.equal(guest.state.time,guestTime);
 const perspective=guestApi.perspective({...host.state,winner:0,events:[{kind:'capture',team:0,previous:1,owner:0,attackerTeam:1}]},'guest');assert.equal(perspective.winner,1);assert.deepEqual(clone(perspective.events),[{kind:'capture',team:1,previous:0,owner:1,attackerTeam:0}]);
 const actionsBefore=engine.calls.length;for(let i=0;i<60;i++)host.handle({type:'game:action',payload:{epoch:1,requestId:i,request:{op:'command',kind:'stop',ids:[blue]}}});
 assert(engine.calls.length-actionsBefore<=40,'Remote action rate must be bounded');pump();
 hostSession.connected=false;const disconnectedTime=host.state.time;assert.equal(host.tick(.05),null);assert.equal(host.state.time,disconnectedTime);assert.equal(host.paused,true,'Host must stop when the transport disconnects');
 host.close();pump();assert.equal(host.active,false);assert.equal(guest.active,false);assert.equal(host.tick(.05),null);assert(guestPauses.at(-1)[0]);
 const bytes=JSON.stringify(init.payload.state).length,warmBytes=JSON.stringify(frame.payload.state).length;
 console.log(`Multiplayer game passed: realWasm host/guest ready handshake, forced fairness, own-team perspective,8Hz dynamic frames/events, live bridge geometry, command ownership, production/gun RPCs, pause/resume, stale frames, bounded requests, guest never ticks; init${bytes}B/warm${warmBytes}B.`);
}
run().catch(error=>{console.error(error);process.exitCode=1;});
