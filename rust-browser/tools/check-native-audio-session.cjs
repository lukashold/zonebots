'use strict';
// Actual Audio.js/App/controller paths. Controlled decode returns an instrumented
// buffer after reading original WAV bytes; this is not a PCM/device-mixer test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
function arg(name,fallback){const i=process.argv.indexOf('--'+name);return i<0?fallback:process.argv[i+1];}
const root=path.resolve(arg('source',path.join(__dirname,'..'))),dir=path.join(root,'dist/independent'),assets=path.resolve(arg('assets',process.env.ORIGINAL_PRIVATE_ROOT||path.join(root,'dist/original'))),wasmPath=path.resolve(arg('wasm',path.join(dir,'engine.wasm')));
const hash=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex'),clone=x=>JSON.parse(JSON.stringify(x));
const manifest=JSON.parse(fs.readFileSync(path.join(assets,'audio.json'),'utf8')),source=fs.readFileSync(path.join(dir,'audio.js'),'utf8'),app=fs.readFileSync(path.join(dir,'app.js'),'utf8'),mp=fs.readFileSync(path.join(dir,'multiplayer-game.js'),'utf8');
const cases=[],measurements=[];async function flush(){for(let i=0;i<48;i++)await Promise.resolve();}function deferred(){let resolve;const promise=new Promise(r=>resolve=r);return{promise,resolve};}
async function test(name,f){try{await f();cases.push({name,ok:true});}catch(e){cases.push({name,ok:false,error:e.message});}}
function audio(stage='decode'){
 const started=[],stopped=[],decodeRequests=[],reads=[],attempts=[],m=deferred(),w=deferred(),d=deferred();
 if(stage!=='manifest')m.resolve();if(stage!=='wave')w.resolve();if(stage==='none')d.resolve();
 class Gain{constructor(){this.gain={value:1,setTargetAtTime(){}};}connect(){}disconnect(){}}
 class Source{connect(){}start(){started.push(this.buffer.url);}stop(){stopped.push(this.buffer?.url);this.onended?.();}}
 class Context{constructor(){this.currentTime=0;this.state='running';this.destination={};}createGain(){return new Gain();}createBufferSource(){return new Source();}resume(){return Promise.resolve();}decodeAudioData(bytes){decodeRequests.push(bytes.url);return d.promise.then(()=>({url:bytes.url,duration:1}));}}
 class Audio{constructor(){this.paused=true;}canPlayType(){return'probably';}play(){this.paused=false;return Promise.resolve();}pause(){this.paused=true;}addEventListener(){}}
 const c={URL,Audio,Date,Math,Set,Map,Promise,console,SpeechSynthesisUtterance:class{},CustomEvent:class{},document:{currentScript:{src:'https://private.example/independent/audio.js'}},AudioContext:Context,dispatchEvent(){},speechSynthesis:{cancel(){},speak(){}},fetch:async url=>{const u=new URL(url);if(u.pathname==='/original/audio.json')return{ok:true,json:async()=>{await m.promise;return manifest;}};const bytes=fs.readFileSync(path.join(assets,u.pathname.replace(/^\/original\//,'')));reads.push({url:u.pathname,bytes:bytes.length});const b=bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength);b.url=u.pathname;return{ok:true,arrayBuffer:async()=>{await w.promise;return b;}};}};
 c.window=c;vm.createContext(c);vm.runInContext(source,c);const a=c.RadioAudio;for(const method of['nativeSample','nativeVoice']){const f=a[method];a[method]=e=>{attempts.push({method,event:clone(e)});return f(e);};}
 return{a,started,stopped,decodeRequests,reads,attempts,manifest:m,wave:w,decode:d,release(){m.resolve();w.resolve();d.resolve();}};
}
const event=(lane,id='1',recipientTeam=0)=>lane==='effects'?{kind:'nativeSample',sampleId:22,clip:'MTANKGUN',lane:'effects',playAudio:true,nativeAudioId:id,recipientTeam,actorTeam:null,team:recipientTeam}:{kind:'nativeVoice',clip:'ROB01',playAudio:true,nativeAudioId:id,recipientTeam,actorTeam:null,team:recipientTeam};
function play(p,lane,id='1'){return p.a[lane==='effects'?'nativeSample':'nativeVoice'](event(lane,id));}
async function unitCases(){
 for(const lane of['effects','voice'])for(const stage of['manifest','wave','decode-resolved'])await test(lane+': reset while '+stage,async()=>{
  const p=audio(stage==='decode-resolved'?'decode':stage);p.a.unlock();if(stage!=='manifest')await p.a.load();const old=play(p,lane);await flush();
  if(stage==='decode-resolved'){assert.equal(p.decodeRequests.length,1);p.decode.resolve();}
  const timeline=[{at:'old-accepted',starts:p.started.length}];p.a.resetNativeEvents();timeline.push({at:'session-reset',starts:p.started.length});
  const next=play(p,lane),duplicate=play(p,lane);p.release();await Promise.all([old,next,duplicate]);await flush();timeline.push({at:'old-and-new-loads-resolved',starts:p.started.length});
  measurements.push({case:lane+':'+stage,timeline,reads:p.reads,starts:p.started,stops:p.stopped});assert.equal(p.started.length,1,'Exactly new-session id1 must start; old accepted async event must not start');assert.equal(p.stopped.length,0);
 });
 for(const lane of['effects','voice']){
  await test(lane+': same session ID dedupe unchanged',async()=>{const p=audio('none');await p.a.load();p.a.unlock();await Promise.all([play(p,lane,'1'),play(p,lane,'1'),play(p,lane,'0')]);assert.equal(p.started.length,1);await play(p,lane,'2');assert.equal(p.started.length,2);});
  await test(lane+': reset does not stop already started clip',async()=>{const p=audio('none');await p.a.load();p.a.unlock();await play(p,lane);p.a.resetNativeEvents();assert.equal(p.started.length,1);assert.equal(p.stopped.length,0);await Promise.all([play(p,lane),play(p,lane)]);assert.equal(p.started.length,2);assert.equal(p.stopped.length,0);});
  await test(lane+': mute invalidates pending async clip',async()=>{const p=audio();await p.a.load();p.a.unlock();const old=play(p,lane);await flush();p.a.mute(true);p.a.mute(false);p.release();await old;assert.equal(p.started.length,0);await play(p,lane,'2');assert.equal(p.started.length,1);});
 }
 await test('legacy effect async path is not changed by native reset',async()=>{const p=audio();await p.a.load();p.a.unlock();const old=p.a.effect('GYRO5');await flush();p.a.resetNativeEvents();p.release();await old;assert.equal(p.started.length,1);});
}
function fixture(){const width=64,n=4096;return{map:{width,height:width,tileSize:32,world:0,original:true,seed:4300,revision:0,tiles:Array(n).fill(0),territory:Array.from({length:n},(_,i)=>i%width<32?0:1),terrainCodes:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],nativeRoadGraph:{regions:[]},nativeTerrainPlane:Array(16384).fill(0)},sectors:[0,1].map(team=>({id:team,x:team?1600:300,y:400,owner:team,home:true,countsForProduction:true})),buildings:[0,1].map(team=>({id:team,kind:'fort',team,sector:team,x:team?1700:64,y:team?300:64,w:128,h:160,hp:9999,maxHp:9999,stars:1,product:'grunt',progress:0,remainingWork:1000000})),spawns:[{type:'medium',team:0,x:320,y:320,squad:1},{type:'gatling',team:1,x:440,y:320,squad:2},{type:'grunt',team:0,x:100,y:1800,squad:3},{type:'grunt',team:1,x:1800,y:1800,squad:4}],crates:[],originalObjects:[],walls:[],nativeWorldBuildings:Array.from({length:35},(_,nativeSlot)=>({nativeSlot,kind:-1,x:-1,y:-1,hp:-1,maxHp:-1,flags6:0,buildingId:null}))};}
function controller(){const window={};vm.runInNewContext(mp,{window,Date,setTimeout,clearTimeout});return window.MultiplayerGame;}
function appContext(engine,radio){const nodes={},c={engine,enc:new TextEncoder(),dec:new TextDecoder(),RadioAudio:radio,state:null,muted:false,selected:new Set(),nativeSentSelection:'',nativeSentVersion:0,nativeSelectionVersion:0,cam:{x:0,y:0},zoom:1,innerHeight:2048,viewWidth:()=>2048,isNetwork:()=>true,performance:{now:()=>0},engineTimings:[],mergeSnapshot:s=>s,paused:false,previousUnits:new Map(),snapshotSpan:50,receivedAt:0,hurt:new Map(),placement:null,$:id=>nodes[id]??=({hidden:true,textContent:''}),hud(){}};vm.createContext(c);let s=app.indexOf('function rawCall(v)'),t=app.indexOf('\n function call(v)',s);vm.runInContext(app.slice(s,t),c);s=app.indexOf('function update(s)');t=app.indexOf('\n function start()',s);vm.runInContext(app.slice(s,t),c);return c;}
async function actualEndpoint(){
 const settings=a=>({...a.nativeSettings(),...(mp.includes('refreshNativeAudioSettings')?{view:{camera:[0,0],size:[1024,1024]}}:{})});
 const bytes=fs.readFileSync(wasmPath),{instance}=await WebAssembly.instantiate(bytes,{}),ha=audio(),ga=audio();await Promise.all([ha.a.load(),ga.a.load()]);ha.a.unlock();ga.a.unlock();await flush();const hc=appContext(instance.exports,ha.a),gc=appContext(null,ga.a),toHost=[],toGuest=[],allMessages=[];let guestRuns=0,hostResets=0,guestResets=0,hostRuns=0;
 const host=controller().create({nativeAudioSettings:()=>settings(ha.a),onNativeHardwareAudioContext:v=>ha.a.setNativeHardwareAudioContext?.(v),onNativeAudioSession(){hostResets++;ha.a.resetNativeEvents();},run:v=>{hostRuns++;return hc.rawCall(v);},apply:s=>hc.update(s)}),guest=controller().create({nativeAudioSettings:()=>settings(ga.a),onNativeHardwareAudioContext:v=>ga.a.setNativeHardwareAudioContext?.(v),onNativeAudioSession(){guestResets++;ga.a.resetNativeEvents();},run(){guestRuns++;throw Error('Guest simulation is forbidden');},apply:s=>gc.update(s)});
 host.bind({connected:true,queuedBytes:0,send(type,payload){const m=clone({type,payload});toGuest.push(m);allMessages.push(m);return true;}},'host');guest.bind({connected:true,queuedBytes:0,send(type,payload){toHost.push(clone({type,payload}));return true;}},'guest');
 const pump=()=>{let budget=100;while(toHost.length||toGuest.length){assert(budget-->0);while(toGuest.length)assert(guest.handle(toGuest.shift()));while(toHost.length)assert(host.handle(toHost.shift()));}};
 try{
  assert(host.start({op:'init',originalMission:fixture()}));pump();host.setPaused(true);pump();for(let i=0;i<130;i++){host.tickUi(1/17);pump();}host.setPaused(false);pump();
  for(let i=0;i<120&&!ga.decodeRequests.some(x=>x.endsWith('/MTANKGUN.wav'));i++){assert(host.tick(.059));pump();await flush();}
  assert(ha.decodeRequests.some(x=>x.endsWith('/MTANKGUN.wav')));assert(ga.decodeRequests.some(x=>x.endsWith('/MTANKGUN.wav')));assert.equal(ha.started.length+ga.started.length,0);
  const currentEpoch=allMessages.find(m=>m.type==='game:init').payload.epoch;
  // An explicit voice transport input complements the actual native shot. The
  // original world did emit22; it did not emit this selected ROB01 fixture.
  const oldVoiceHost=clone(host.viewState);oldVoiceHost.events=[event('voice','1000',0)];hc.update(oldVoiceHost);
  const oldVoiceGuest=clone(host.state);oldVoiceGuest.events=[event('voice','1000',1)];assert(guest.handle({type:'game:frame',payload:{epoch:currentEpoch,seq:100000,state:oldVoiceGuest}}));await flush();assert(ga.decodeRequests.some(x=>x.endsWith('/ROB01.wav')));
  assert(host.start({op:'init',originalMission:fixture()}));const nextInit=toGuest.find(m=>m.type==='game:init');assert(nextInit);const nextEpoch=nextInit.payload.epoch;
  const newFrame=clone(host.state);newFrame.events=[event('effects','1',1),event('voice','2',1)];
  const future={type:'game:frame',payload:{epoch:nextEpoch,seq:1,state:newFrame}};const attemptsBefore=ga.attempts.length;
  assert.equal(guest.handle(clone(future)),false,'Future frame before its game:init must be rejected');assert.equal(ga.attempts.length,attemptsBefore);assert.equal(guestResets,1);
  pump();assert.equal(hostResets,2);assert.equal(guestResets,2);assert(guest.handle(clone(future)));
  const newHost=clone(host.viewState);newHost.events=[event('effects','1',0),event('voice','2',0)];hc.update(newHost);await flush();
  assert.equal(guest.handle(clone(future)),false,'Duplicate sequence is rejected');assert.equal(guest.handle({type:'game:frame',payload:{...future.payload,epoch:currentEpoch,seq:100001}}),false,'Old epoch rejected');
  assert.equal(guest.handle({type:'game:frame',payload:{...future.payload,epoch:nextEpoch+1}}),false,'Early next epoch rejected');
  ha.release();ga.release();await flush();
  const count=p=>Object.fromEntries(['MTANKGUN','ROB01'].map(n=>[n,p.started.filter(x=>x.endsWith('/'+n+'.wav')).length]));
  measurements.push({case:'actual-App-Host-Guest-two-epochs',wasm:hash(wasmPath),hostResets,guestResets,guestRuns,hostRuns,host:count(ha),guest:count(ga),hostStarted:ha.started,guestStarted:ga.started,hostStopped:ha.stopped,guestStopped:ga.stopped,detail:'Actual first MTANKGUN source callback; explicit ROB01/new-ID1 frame inputs exercise transport, not a world voice emission claim'});
  assert.equal(guestRuns,0);assert.deepEqual(count(ha),{MTANKGUN:1,ROB01:1});assert.deepEqual(count(ga),{MTANKGUN:1,ROB01:1});assert.equal(ha.stopped.length+ga.stopped.length,0);
 }finally{host.close(false);guest.close(false);}
}
(async()=>{await unitCases();await test('actual App/Host/Guest epoch + rejected future-frame sequence',actualEndpoint);const output={sourceRoot:root,sourceHashes:Object.fromEntries(['audio.js','app.js','multiplayer-game.js'].map(n=>[n,hash(path.join(dir,n))])),wasm:hash(wasmPath),cases,measurements,scope:'Only pending native session audio. Actual original WAV reads/instrumented WebAudio; no PCM/browser/device decode equivalence. No camera/legacy/world/source clock changes.'};const out=arg('output',null);if(out)fs.writeFileSync(out,JSON.stringify(output,null,2));console.log(JSON.stringify({passed:cases.filter(x=>x.ok).length,failed:cases.filter(x=>!x.ok),measurements},null,2));if(cases.some(x=>!x.ok))process.exitCode=1;})().catch(e=>{console.error(e);process.exitCode=1;});
