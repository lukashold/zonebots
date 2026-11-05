'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const app=fs.readFileSync(path.join(__dirname,'../dist/independent/app.js'),'utf8'),start=app.indexOf('let peerFortCamera='),end=app.indexOf('\n function start()',start);assert(start>=0&&end>start);
const snapshot=(winner=null,clock=0)=>({winner,time:10,units:[],events:[],nativeUiClock:clock,map:{original:true,world:0}});
function harness(network=false,initial=snapshot()){
 const nodes={},scenes=[],voices=[];let hudCalls=0;
 const context={state:initial,paused:false,menu:false,mergeSnapshot:s=>s,performance:{now:()=>100},previousUnits:new Map(),snapshotSpan:50,receivedAt:0,hurt:new Map(),placement:null,selected:new Set(),cam:{x:0,y:0},zoom:1,innerHeight:768,viewWidth:()=>1024,isNetwork:()=>network,$:id=>nodes[id]??=( {hidden:true,textContent:''}),hud:()=>hudCalls++,RadioAudio:{setScene:(...a)=>scenes.push(a),say:(...a)=>voices.push(a),nativeSample:()=>{},nativeVoice:()=>{},effect:()=>{}}};
 vm.createContext(context);vm.runInContext(app.slice(start,end),context);
 return{context,nodes,scenes,voices,get hudCalls(){return hudCalls;}};
}
function run(){for(const network of[false,true])for(const winner of[0,1]){
 const fixture=harness(network),{context,nodes,scenes,voices}=fixture;
 for(let n=0;n<5;n++)context.update(snapshot(null,n));assert.equal(voices.length,0);assert.equal(scenes.length,0);assert.equal(context.paused,false);
 for(let n=0;n<12;n++){const s=snapshot(winner,n+5);if(n===0)s.events=[{kind:'victory',team:winner}];context.update(s);}
 assert.equal(voices.length,1,'Repeated winner snapshots/UI-only returns must announce the result once');assert.equal(scenes.length,1,'Identical result snapshots must not restart scene audio');assert.equal(voices[0][0],winner===0?'victory':'defeat');
 assert.equal(context.paused,true);assert.equal(nodes.result.hidden,false);assert.equal(nodes.result.textContent,winner===0?'SIEG – Fort gesichert':'NIEDERLAGE');assert.equal(nodes.newBattle.hidden,false);if(network)assert.equal(nodes.newBattle.textContent,'Zur Mehrspieler-Lobby');assert.equal(fixture.hudCalls,17,'All result snapshots still refresh UI');
 context.paused=false;context.update(snapshot(winner,17));assert.equal(context.paused,true,'A completed game remains paused');assert.equal(voices.length,1);
 context.state=null;context.update(snapshot(winner,0));assert.equal(voices.length,2,'A freshly loaded game state may announce its result');
 context.update(snapshot(null,1));context.update(snapshot(winner,2));assert.equal(voices.length,3,'A later winner transition is a new result');context.update(snapshot(1-winner,3));assert.equal(voices.length,4,'A changed winner receives the corresponding result');
}
 const local=harness(),offers=[],stops=[];
 local.context.window={ResultFilm:{offer(...args){offers.push(args);return true;}}};
 local.context.muted=true;local.nodes.music={value:'0.65'};
 local.context.RadioAudio.haltNativeChannels=()=>stops.push('halt');
 const result={...snapshot(0),mission:3,nativeFortScene:{active:true,camera:null}};
 local.context.update(result);
 assert.deepEqual(local.scenes,[['movie',0]]);
 assert.deepEqual(stops,['halt']);assert.equal(local.voices.length,0);
 assert.deepEqual(offers[0].slice(0,3),[3,true,false]);
 assert.equal(offers[0][3].muted,true);assert.equal(offers[0][3].volume,.65);
 local.context.update({...result,nativeUiClock:1});assert.equal(offers.length,1);
 offers[0][3].onFinish();assert.deepEqual(local.scenes,[['movie',0],['victory',0]]);
 const scores=harness(),records=new Map(),element=()=>({children:[],append(...items){this.children.push(...items);}});
 scores.context.document={createElement:element};
 scores.context.localStorage={getItem:k=>records.get(k)??null,setItem:(k,v)=>records.set(k,v)};
 scores.nodes.result=element();
 scores.context.update({...snapshot(0),mission:0,nativeFortResult:{losses:[2,5,0,0],points:8900}});
 assert.equal(scores.nodes.result.children[0].children[0].textContent,'Gruppen verloren · Eigene 2 : Gegner 5');
 assert.equal(scores.nodes.result.children[0].children[1].textContent,'Punkte 8900 · Bestwert 8900');
 scores.context.state=null;scores.nodes.result=element();
 scores.context.update({...snapshot(0),mission:0,nativeFortResult:{losses:[1,1,0,0],points:7200}});
 assert.equal(scores.nodes.result.children[0].children[1].textContent,'Punkte 7200 · Bestwert 8900');
 assert.equal(records.get('z.native.fort.best.v1:0'),'8900');
 const classic=harness(true);const presented=[];
 classic.nodes.result={hidden:true,classList:{contains:name=>name==='classic-result'}};
 classic.context.window={OriginalResultView:{show(...args){presented.push(args);return true;}}};
 const ended={...snapshot(1),nativeFortScene:{scoreTime:3671},nativeFortResult:{losses:[17,53,0,0],points:null}};
 classic.context.update(ended);assert.equal(presented.length,1);assert.deepEqual(presented[0].slice(2),[ended.nativeFortResult,3671,false,true]);
 classic.nodes.result.hidden=classic.nodes.newBattle.hidden=true;classic.context.menu=true;
 classic.context.update(ended);assert.equal(presented.length,1,'a late peer snapshot must not reopen a dismissed result');
 assert.equal(classic.nodes.result.hidden,true);

 console.log('Result audio passed: actual App.update, offline/network, local cinematic, repeated UI states and pause/HUD.');
}
module.exports={harness};if(require.main===module)run();
