'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/app.js'),'utf8');
const first=source.indexOf(' function continueFromResult(){'),last=source.indexOf(" $('newBattle').onclick=continueFromResult;",first);
assert.ok(first>=0&&last>first);
const transitionStart=source.indexOf(' let pendingCampaignFilm=false;');
assert.ok(transitionStart>=0&&transitionStart<first);
function beforeBattle(mission,{fresh=false,previous=-1,available=true}={}){
 const calls=[],offers=[],selection={value:String(mission)};
 const context={muted:true,$:id=>id==='mission'?selection:{value:40},
  window:{ResultFilm:{offerCampaignIntro:o=>{offers.push(['intro',o]);return available;},
   offerTransition:(m,w,o)=>{offers.push([m,w,o]);return available;}}},
  start:()=>calls.push('battle'),RadioAudio:{haltNativeChannels:()=>calls.push('halt'),setScene:s=>calls.push(s)}};
 vm.createContext(context);vm.runInContext(source.slice(transitionStart,first),context);
 context.beginCampaignBattle(previous,fresh);
 if(available&&offers.length){context.beginCampaignBattle(previous,fresh);assert.equal(offers.length,1);}
 return {calls,offers,selection};
}
const intro=beforeBattle(0,{fresh:true});
assert.deepEqual(intro.calls,['halt','movie']);assert.equal(intro.offers[0][0],'intro');
assert.equal(intro.offers[0][1].volume,.4);
intro.offers[0][1].onFinish();assert.deepEqual(intro.calls,['halt','movie','battle']);
const planet=beforeBattle(4,{previous:0});
assert.deepEqual(planet.offers[0].slice(0,2),[4,0]);assert.deepEqual(planet.calls,['halt','movie']);
planet.selection.value='19';planet.offers[0][2].onFinish();
assert.deepEqual(planet.calls,['halt','movie','battle']);assert.equal(planet.selection.value,'4');
assert.deepEqual(beforeBattle(4,{available:false}).calls,['battle']);
assert.deepEqual(beforeBattle(0).calls,['battle']);
assert.deepEqual(beforeBattle(0,{fresh:true,available:false}).calls,['battle']);
const introTrace=process.env.NATIVE_CAMPAIGN_FIRST_INTRO_TRACE;
if(introTrace){
 const rows=fs.readFileSync(introTrace,'utf8').trim().split(/\r?\n/).map(JSON.parse);
 assert.equal(rows.length,18);
 for(const n of rows){
  const expected=n.fresh!==0&&n.mission===1;
  const flow=beforeBattle(n.mission-1,{fresh:n.fresh!==0});
  if(n.mission===1)assert.equal(flow.offers.some(([id])=>id==='intro'),expected);
  assert.equal(n.movie,expected?'e_intro2':'');assert.equal(n.halt,Number(expected));
  assert.equal(n.played,expected&&n.media!==0?1:0);assert.equal(n.next,expected?20:16);
 }
}
function run({mission=19,winner=0,network=false,classic=true,film=true,original=true}={}){
 const actions=[],offers=[],nodes={result:{classList:{contains:()=>classic}},newBattle:{},mission:{value:String(mission)},music:{value:55},quit:{onclick:()=>actions.push('menu')}};
 const context={state:{mission,winner,map:{world:4,original}},mission,muted:true,resultAcknowledged:false,isNetwork:()=>network,$:id=>nodes[id],
  window:{OriginalResultView:{clear:()=>actions.push('clear')},ResultFilm:{offerOutro:options=>{offers.push(options);return film;}}},
  RadioAudio:{haltNativeChannels:()=>actions.push('halt'),setScene:(scene,world)=>actions.push([scene,world])},
  beginCampaignBattle:world=>actions.push(['transition',world]),start:()=>actions.push('start'),openMultiplayerMenu:()=>actions.push('lobby')};
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
const next=run({mission:18});assert.deepEqual(next.actions,['clear',['transition',2]]);assert.equal(next.nodes.mission.value,19);assert.equal(next.offers.length,0);
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
