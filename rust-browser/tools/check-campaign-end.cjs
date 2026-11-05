'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/app.js'),'utf8');
const first=source.indexOf(' function continueFromResult(){'),last=source.indexOf(" $('newBattle').onclick=continueFromResult;",first);
assert.ok(first>=0&&last>first);
function run({mission=19,winner=0,network=false,classic=true,film=true,original=true}={}){
 const actions=[],offers=[],nodes={result:{classList:{contains:()=>classic}},newBattle:{},mission:{value:String(mission)},music:{value:0.55},quit:{onclick:()=>actions.push('menu')}};
 const context={state:{mission,winner,map:{world:4,original}},mission,muted:true,resultAcknowledged:false,isNetwork:()=>network,$:id=>nodes[id],
  window:{OriginalResultView:{clear:()=>actions.push('clear')},ResultFilm:{offerOutro:options=>{offers.push(options);return film;}}},
  RadioAudio:{haltNativeChannels:()=>actions.push('halt'),setScene:(scene,world)=>actions.push([scene,world])},start:()=>actions.push('start'),openMultiplayerMenu:()=>actions.push('lobby')};
 vm.createContext(context);vm.runInContext(source.slice(first,last),context);
 context.continueFromResult();context.continueFromResult();
 assert.equal(context.resultAcknowledged,true);return {actions,offers,nodes};
}
const end=run();assert.deepEqual(end.actions,['clear','halt',['movie',4]]);
assert.equal(end.offers.length,1);assert.equal(end.offers[0].muted,true);assert.equal(end.offers[0].volume,.55);
assert.equal(end.nodes.result.hidden,true);assert.equal(end.nodes.newBattle.hidden,true);
assert.equal(end.nodes.mission.value,'19');end.offers[0].onFinish();
assert.equal(end.nodes.mission.value,'0');assert.equal(end.actions.at(-1),'menu');
assert.ok(!end.actions.includes('start'),'Mission 20 must not repeat');
const fallback=run({film:false});assert.deepEqual(fallback.actions,['clear','menu']);assert.equal(fallback.nodes.mission.value,'0');
const next=run({mission:18});assert.deepEqual(next.actions,['clear','start']);assert.equal(next.nodes.mission.value,19);assert.equal(next.offers.length,0);
const lost=run({winner:1});assert.deepEqual(lost.actions,['clear','menu']);assert.equal(lost.offers.length,0);
const mp=run({network:true});assert.deepEqual(mp.actions,['clear','lobby']);assert.equal(mp.offers.length,0);
const trace=process.env.NATIVE_CAMPAIGN_END_TRACE;
if(trace){
 const rows=fs.readFileSync(trace,'utf8').trim().split(/\r?\n/).map(JSON.parse);assert.equal(rows.length,144);
 const finals=rows.filter(r=>r.mission===20&&!r.mp&&r.defeated!==r.local+1&&r.pending===0);
 assert.equal(finals.length,6);
 for(const n of finals){assert.equal(n.movie,'e_outro');assert.equal(n.movieMode,4);assert.equal(n.nextMission,1);assert.equal(n.movieStarts,n.media===0?0:1);}
 assert.ok(rows.filter(r=>r.mp).every(r=>r.movie!=='e_outro'));
}
const campaignFile=process.env.Z_PRIVATE_CAMPAIGN??path.join(__dirname,'../dist/original/campaign.json');
if(fs.existsSync(campaignFile)){
 const original=JSON.parse(fs.readFileSync(campaignFile,'utf8')).missions[19];
 assert.equal(original.number,20);
 const missionData=structuredClone(original),actor=missionData.spawns.find(u=>u.team===0&&u.type==='psycho');
 assert.ok(actor);Object.assign(actor,{x:3440,y:656});
 const wasm=new WebAssembly.Module(fs.readFileSync(path.join(__dirname,'../dist/independent/engine.wasm')));
 const engine=new WebAssembly.Instance(wasm,{}).exports;
 let camera=[0,0],map=null;
 function send(payload){
  const data=Buffer.from(JSON.stringify({compact:true,knownMapSeed:map?.seed,knownMapRevision:map?.revision,
   nativeAudio:{soundEnabled:false},nativeView:{camera,size:[568,360]},...payload}));
  const ptr=engine.alloc(data.length);
  new Uint8Array(engine.memory.buffer,ptr,data.length).set(data);
  let length;try{length=engine.run(ptr,data.length);}finally{engine.free(ptr,data.length);}
  assert.ok(engine.memory.buffer.byteLength<64*1024*1024,'an expiring effect must not grow the Wasm heap without bound');
  const state=JSON.parse(Buffer.from(new Uint8Array(engine.memory.buffer,engine.output(),length)).toString());
  assert.equal(state.error,undefined);map=state.map??map;camera=state.nativeFortScene?.camera??camera;return state;
 }
 let state=send({op:'init',mission:19,difficulty:1,seed:29510,originalMission:missionData});
 assert.equal(state.winner,null);
 for(let frame=2;frame<=170;frame++){
  state=send({op:'tick',dt:0.059});
  assert.equal(state.nativeFortScene.boundary,null);
  assert.equal(state.nativeFortScene.frame,frame<170?frame:null);
  assert.equal(state.winner,frame<170?null:0);
 }
 assert.deepEqual(state.nativeFortResult,{losses:[0,14,0,0],points:20000,rank:9});
 console.log('Shipping Wasm: authored Mission 20, one red robot on the original gate, all 169 fort ticks, bounded heap, 20,000 points.');
}
console.log('Campaign completion: final OK, optional original outro, skip/fallback, reset to mission 1, ordinary next/defeat/lobby.');
