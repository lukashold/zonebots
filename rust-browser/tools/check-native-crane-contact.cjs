// Actual published/candidate Wasm and unchanged external campaign; no private
// getters, embedded Original JSON or media. AI is disabled explicitly. Source
// timing is not inferred from these port callback counts. Canvas/GPU is outside.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const assert=require('node:assert/strict'),crypto=require('node:crypto');
const root=path.resolve(__dirname,'..');
const wasmPath=path.resolve(process.argv[2]||path.join(root,'dist/independent/engine.wasm'));
const campaignPath=path.resolve(process.argv[3]||process.env.Z_PRIVATE_CAMPAIGN||path.join(root,'dist/original/campaign.json'));
const jsDir=path.resolve(process.argv[4]||path.join(root,'dist/independent'));
const clone=v=>JSON.parse(JSON.stringify(v));
(async()=>{
 const wasm=fs.readFileSync(wasmPath),campaign=fs.readFileSync(campaignPath),mission=JSON.parse(campaign).missions[13],cases=[];
 async function engine(){
  const e=(await WebAssembly.instantiate(wasm,{})).instance.exports;let state;
  return {get state(){return state;},raw(v){
   const b=Buffer.from(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;
   try{n=e.run(p,b.length);}finally{e.free(p,b.length);}
   const r=JSON.parse(Buffer.from(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!r.error,r.error);state={...state,...r};return r;
  },run(v){this.raw(v);return state;},tick(){return this.run({op:'tick',dt:.059,compact:true,knownMapSeed:state.map.seed,knownMapRevision:state.map.revision});}};
 }
 const normal=(s,id)=>{const row=s.nativeDisplayQueue.find(r=>r.kind==='unit'&&r.id===id);assert(row);assert.equal(row.nativeCraneBoundary,undefined);assert.equal(row.nativeLayers?.length,3);return row;};
 function view(s,id){
  const u=s.units.find(u=>u.id===id),x=Math.floor(u.x/2),y=Math.floor(u.y/2),cell=((y*8+64)&0x7ff80)+((x+8)>>4),tx=cell&127,ty=cell>>7;
  const code=(tx<s.map.width&&ty<s.map.height?s.map.terrainCodes[ty*s.map.width+tx]:s.map.nativeTerrainPlane[cell])&127;
  const row=normal(s,id),arm=row.nativeLayers.find(l=>l.sprite>=5826&&l.sprite<=5889)?.sprite;
  return {x:u.x,y:u.y,cell,code,arm,phase:u.nativePhase35,hp:s.map.bridgeHp[4]};
 }
 async function take(multiplayer,missingFrames=false){
  const e=await engine(),m=clone(mission);if(missingFrames)delete m.nativeCraneBuildFrames;
  e.run({op:'init',mission:13,seed:0x4359,multiplayer,aiTeams:[],originalMission:m});
  const a=e.state.units.find(u=>u.type==='tough'&&u.team===0&&u.x===1312&&u.y===3360),c=e.state.units.find(u=>u.type==='crane'&&u.team<0&&u.x===1536&&u.y===3136);assert(a&&c,'unchanged authored actors');
  e.run({op:'command',kind:'enter',ids:[a.id],target:c.id,team:0});let taken=0;
  for(let callback=1;callback<=600;callback++){e.tick();assert.equal(e.state.nativeRouteBoundary,null);if(e.state.units.find(u=>u.id===c.id)?.team===0){taken=callback;break;}}
  assert.equal(taken,122,'unchanged authored takeover port control');
  for(let k=0;k<4;k++){e.tick();normal(e.state,c.id);}
  return {e,c,taken};
 }
 for(const multiplayer of[false,true]){
  const {e,c,taken}=await take(multiplayer),b=mission.bridges.find(b=>b.id===4);assert(b);
  const ownedBefore=e.state.units.find(u=>u.id===c.id);
  e.run({op:'command',kind:'repairBridge',ids:[c.id],team:1,x:b.x+b.w/2,y:b.y+b.h/2});
  const rejected=e.state.units.find(u=>u.id===c.id);
  for(const field of['order','target','dest','path','nativePhase35'])assert.deepEqual(rejected[field],ownedBefore[field],`ownership gate ${field}`);
  e.run({op:'command',kind:'repairBridge',ids:[c.id],team:0,x:b.x+b.w/2,y:b.y+b.h/2});
  let before=view(e.state,c.id),crossing=null,nextAfter=null,completed=0;
  for(let callback=1;callback<=2000;callback++){
   e.tick();assert.equal(e.state.nativeRouteBoundary,null);const after=view(e.state,c.id);
   if(crossing&&callback===crossing.callback+1)nextAfter=after;
   if(before.code===50&&after.code===47&&!crossing){
    crossing={callback,before,after};
    assert.equal(before.arm,5826,'pre-contact arm phase');
    assert.equal(after.arm,5826,'crossing cannot add post-Common contact/phase2');
   }
   if(after.hp>=600){completed=callback;break;}before=after;
  }
  assert(crossing&&nextAfter,'real50→47 crossing and following callback');
  assert.equal(nextAfter.arm,5828,'following Main preamble may activate47 before the work tick');
  assert(completed>0,'native work/finite retreat/shared late jobs must complete');
  for(let k=0;k<150;k++)e.tick();assert.equal(e.state.map.bridgeHp[4],600);assert.equal(e.state.nativeRouteBoundary,null);
  cases.push({kind:'authoredBridge',multiplayer,taken,crossing,nextAfter,completedPortCallbacks:completed,postCallbacks:150});
 }
 for(const multiplayer of[false,true])for(const missingFrames of[false,true]){
  const {e,c}=await take(multiplayer,missingFrames),before=view(e.state,c.id);
  e.run({op:'command',kind:'move',ids:[c.id],team:0,x:1550,y:3216});
  for(let k=0;k<20;k++){e.tick();assert.equal(e.state.nativeRouteBoundary,null);normal(e.state,c.id);}
  const after=view(e.state,c.id);assert(after.x!==before.x||after.y!==before.y,'unused BUILD frames cannot block normal crane Common/movement');
  cases.push({kind:'normalMove',multiplayer,missingFrames,before,after20:after});
 }
 // Actual MP controller/validator/perspective. One authoritative engine only;
 // fake transport routes messages, never a second world/RNG callback.
 let milliseconds=1000;const source=fs.readFileSync(path.join(jsDir,'multiplayer-game.js'),'utf8');
 function api(){const window={};vm.runInNewContext(source,{window,setTimeout,clearTimeout,Date:class extends Date{static now(){return milliseconds;}}});return window.MultiplayerGame;}
 const e=await engine(),toHost=[],toGuest=[];let guestRuns=0,frames=0;
 const host=api().create({run:v=>e.raw(v)}),guest=api().create({run(){guestRuns++;throw Error('guest must not simulate');}});
 host.bind({connected:true,queuedBytes:0,send(type,payload){toGuest.push(clone({type,payload}));return true;}},'host');
 guest.bind({connected:true,send(type,payload){toHost.push(clone({type,payload}));return true;}},'guest');
 function pump(){let n=500;while(toHost.length||toGuest.length){assert(n-->0);while(toGuest.length){const msg=toGuest.shift();const accepted=guest.handle(msg);assert(accepted,`guest rejected actual ${msg.type}`);if(msg.type==='game:frame')frames++;}while(toHost.length)host.handle(toHost.shift());}}
 assert(host.start({op:'init',mission:13,seed:0x4359,aiTeams:[],originalMission:mission},{name:'Crane Main contact'}));pump();
 const a=host.state.units.find(u=>u.type==='tough'&&u.team===0&&u.x===1312&&u.y===3360),c=host.state.units.find(u=>u.type==='crane'&&u.team<0&&u.x===1536&&u.y===3136);assert(a&&c);
 host.action({op:'command',kind:'enter',ids:[a.id],target:c.id,team:0});pump();
 function tick(){milliseconds+=59;host.tick(.059);pump();assert.equal(host.state.nativeRouteBoundary,null);guest.tick(.059);}
 for(let k=0;k<130;k++)tick();assert.equal(host.state.units.find(u=>u.id===c.id)?.team,0);
 const b=mission.bridges.find(b=>b.id===4);host.action({op:'command',kind:'repairBridge',ids:[c.id],team:0,x:b.x+b.w/2,y:b.y+b.h/2});pump();
 let completed=0;for(let k=1;k<=2000;k++){tick();if(host.state.map.bridgeHp[4]>=600){completed=k;break;}}assert(completed);
 for(let k=0;k<150;k++)tick();host.setPaused(true);pump();
 assert.equal(guest.state.time,host.state.time);assert.equal(guest.state.map.bridgeHp[4],600);assert.equal(guestRuns,0);
 const h=normal(host.state,c.id),g=normal(guest.viewState,c.id);assert(h.nativeLayers.every(l=>l.palette===0));assert(g.nativeLayers.every(l=>l.palette===1));
 assert.deepEqual(clone(guest.state.units),clone(host.state.units));
 cases.push({kind:'hostGuestRPC',frames,guestSimulationCalls:guestRuns,completedPortCallbacks:completed,canonicalHP:600});host.close();guest.close();
 console.log(JSON.stringify({result:'PASS',wasmSha256:crypto.createHash('sha256').update(wasm).digest('hex'),campaignSha256:crypto.createHash('sha256').update(campaign).digest('hex'),scope:'Actual ABI/RPC port counts, unchanged authored M14; no original human-admission/full-match/CRT/GPU parity claim',cases},null,2));
})().catch(error=>{console.error(error.stack);process.exitCode=1;});
