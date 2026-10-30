// Attack feedback behavior without a browser or original resources.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const env={window:{}};vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(__dirname,'../dist/independent/attack-command.js'),'utf8'),env);
const {intent,fighterIds,createTracker}=env.window.AttackCommand;
function fixture(){const map={original:true,width:32,height:32,tileSize:32,bridgeBase:Array(1024).fill(-1),bridgeHp:[400]};map.bridgeBase[20*32+20]=0;map.bridgeBase[20*32+21]=0;
 return {map,winner:null,time:12.5,catalog:[{type:'grunt',damage:5,range:240},{type:'gatling',damage:6,range:240,speed:0},
  {type:'crane',damage:0,range:240},{type:'apc',damage:0,range:240},{type:'truck',damage:1,range:0}],
  units:[{id:1,type:'grunt',team:0,hp:30},{id:2,type:'gatling',team:0,hp:50},{id:3,type:'crane',team:0,hp:100},
   {id:4,type:'apc',team:0,hp:200},{id:5,type:'grunt',team:0,hp:0},{id:6,type:'grunt',team:0,hp:-1},
   {id:7,type:'truck',team:0,hp:30},{id:8,type:'unknown',team:0,hp:30},
   {id:10,type:'grunt',team:1,hp:30,x:300,y:200},{id:11,type:'gatling',team:2,hp:0,x:500,y:200},
   {id:12,type:'gatling',team:-1,hp:50,x:700,y:200},{id:13,type:'grunt',team:1,hp:-1,x:900,y:200}],
  buildings:[{id:0,team:1,hp:2000,x:100,y:400,w:128,h:160,gate:[140,570]},
   {id:1,team:-1,hp:1000,x:300,y:400,w:128,h:160},
   {id:2,team:0,hp:1000,x:500,y:400,w:128,h:160},{id:3,team:1,hp:0,x:700,y:400,w:128,h:160}]};}
const unit=(s,id)=>s.units.find(u=>u.id===id),box=u=>u.id===10?{x:290,y:170,w:80,h:60}:null,point={x:360,y:180};
let s=fixture(),before=JSON.stringify(s),selection=new Set([3,2,4,1,2]),r=intent(s,selection,point,{},box);
assert.equal(r.kind,'attack');assert.equal(r.targetType,'unit');assert.equal(r.target,10);assert.equal(r.entity,unit(s,10));
assert.deepEqual(Array.from(r.ids),[2,1]);assert.deepEqual([r.x,r.y],[300,200]);assert.equal(JSON.stringify(s),before);
assert.deepEqual(Array.from(fighterIds(s,[1,1,2,3,4,5,6,7,8,10,12,999])),[1,2,5]);
assert.deepEqual([...selection],[3,2,4,1]);assert.deepEqual(Array.from(fighterIds(null,[1])),[]);
for(const selected of [[],null,[3],[4],[6],[7],[8],[10],[12],[999]])assert.equal(intent(s,selected,point,{},box),null);
assert(intent(s,[5],{x:500,y:200}), 'Original HP0 actors and enemy units are alive');
s.map.original=false;assert.equal(intent(s,[5],point,{},box),null);assert.equal(intent(s,[1],{x:500,y:200}),null);s.map.original=true;
for(const options of [{paused:true},{menu:true},{placement:{}},{commandMode:'placement'}])assert.equal(intent(s,[1],point,options,box),null);
for(const winner of [0,1]){s.winner=winner;assert.equal(intent(s,[1],point,{},box),null);}s.winner=null;
for(const p of [null,{x:NaN,y:200},{x:300,y:Infinity}])assert.equal(intent(s,[1],p,{},box),null);
assert.equal(intent(null,[1],point,{},box),null);
for(const x of [700,900])assert.equal(intent(s,[1],{x,y:200}),null,'Neutral and dead units are not attack targets');
assert.equal(intent(s,[1],{x:521.9,y:200}).target,11);assert.equal(intent(s,[1],{x:522,y:200}),null);
assert.equal(intent(s,[1],{x:300,y:200},{},()=>({x:NaN,y:0,w:2,h:2})).target,10,'Bad sprite bounds use radius fallback');
assert.equal(intent(s,[1],{x:280,y:200},{},box),null,'A valid sprite rectangle replaces the radius hit');
r=intent(s,[1],{x:120,y:420});assert.equal(r.kind,'attackBuilding');assert.equal(r.target,0);assert.deepEqual([r.x,r.y],[140,570]);
r=intent(s,[1],{x:320,y:420});assert.equal(r.target,1);assert.deepEqual([r.x,r.y],[364,584],'Neutral buildings keep legacy attack behavior and exit route');
for(const x of [520,720])assert.equal(intent(s,[1],{x,y:420}),null,'Own and destroyed buildings are not normal targets');
assert.equal(intent(s,[1],{x:650,y:650}),null,'Normal ground/bridge hover has no attack intent');
r=intent(s,[1],{x:650,y:650},{commandMode:'attack'});assert.equal(r.kind,'attackBridge');assert.equal(r.targetType,'bridge');assert.equal(r.target,0);
r=intent(s,[1],{x:800,y:800},{commandMode:'attack'});assert.equal(r.kind,'attack');assert.equal(r.targetType,'point');assert.equal(r.target,undefined);
for(const p of [{x:-1,y:800},{x:1024,y:800},{x:800,y:1024}])assert.equal(intent(s,[1],p,{commandMode:'attack'}),null);
s.map.bridgeHp[0]=0;assert.equal(intent(s,[1],{x:650,y:650},{commandMode:'attack'}).targetType,'point');

function assigned(type='unit',ids=[1,2]){
 const state=fixture(),tracker=createTracker(),p=type==='unit'?point:type==='building'?{x:120,y:420}:type==='bridge'?{x:650,y:650}:{x:800,y:800};
 const target=intent(state,ids,p,type==='bridge'||type==='point'?{commandMode:'attack'}:{},box);
 for(const [i,id]of target.ids.entries()){const u=unit(state,id);u.order=target.kind;u.target=target.target??null;u.dest=type==='unit'?null:[target.x+i*12,target.y];u.path=type==='point'?[[784,784],[816,816]]:[];}
 tracker.assign(target,state);return {state,tracker,target};
}
let t=assigned(),markers=t.tracker.targets(t.state,new Set([1,2]));
assert.equal(markers.length,1);assert.equal(markers[0].entity,unit(t.state,10));assert.equal(markers[0].started,12.5);
assert.equal(t.tracker.targets(t.state,[]).length,0);assert.equal(t.tracker.targets(t.state,[1]).length,1);
unit(t.state,10).x+=90;unit(t.state,10).y+=40;markers=t.tracker.targets(t.state,[2]);assert.deepEqual([markers[0].x,markers[0].y],[390,240]);
t.tracker.clear([1]);assert.equal(t.tracker.targets(t.state,[1]).length,0);assert.equal(t.tracker.targets(t.state,[2]).length,1);t.tracker.reset();assert.equal(t.tracker.targets(t.state,[2]).length,0);
for(const mutate of [
 state=>{state.units=state.units.filter(u=>u.id!==1);},state=>{unit(state,1).hp=-1;},state=>{unit(state,1).team=1;},
 state=>{unit(state,1).type='crane';},state=>{unit(state,1).order='move';},state=>{unit(state,1).target=11;},
 state=>{unit(state,1).dest=[300,200];},state=>{state.units=state.units.filter(u=>u.id!==10);},state=>{unit(state,10).hp=-1;},
 state=>{unit(state,10).team=0;},state=>{unit(state,10).team=-1;},state=>{unit(state,10).x=NaN;},state=>{state.time=0;}
]){t=assigned('unit',[1]);mutate(t.state);assert.equal(t.tracker.targets(t.state,[]).length,0);assert.equal(t.tracker.targets(assigned().state,[1]).length,0,'A stale same-ID target cannot resurrect an expired marker');}
t=assigned('unit',[2]);assert.equal(t.tracker.targets(t.state,[2]).length,1,'A stationary gun keeps an attack marker with an empty path');
t=assigned('unit',[5]);assert.equal(t.tracker.targets(t.state,[5]).length,1);t.state.map.original=false;assert.equal(t.tracker.targets(t.state,[5]).length,0);
t=assigned('unit',[1]);unit(t.state,10).hp=0;assert.equal(t.tracker.targets(t.state,[1]).length,1);t.state.map.original=false;assert.equal(t.tracker.targets(t.state,[1]).length,0);
t=assigned('building');markers=t.tracker.targets(t.state,[1,2]);assert.equal(markers.length,1);assert.equal(markers[0].entity,t.state.buildings[0]);assert.deepEqual([markers[0].x,markers[0].y],[164,480],'Building marker uses center while command routes to gate');
for(const mutate of [state=>{state.buildings[0].team=0;},state=>{state.buildings[0].hp=0;},state=>{state.buildings.shift();},state=>{unit(state,1).dest[0]++;}]){t=assigned('building',[1]);mutate(t.state);assert.equal(t.tracker.targets(t.state,[1]).length,0);}
t=assigned('bridge');markers=t.tracker.targets(t.state,[1,2]);assert.equal(markers.length,1);assert.deepEqual([markers[0].x,markers[0].y],[656,656]);
for(const mutate of [state=>{state.map.bridgeHp[0]=0;},state=>{state.map.bridgeBase[20*32+20]=-1;},state=>{unit(state,1).order='guard';},state=>{unit(state,1).dest[0]+=32;}]){t=assigned('bridge',[1]);mutate(t.state);assert.equal(t.tracker.targets(t.state,[1]).length,0);}
// Formation offsets outside the selected bridge cannot produce a valid marker.
t=assigned('bridge',[1]);const actor=unit(t.state,1);actor.dest=[740,650];t.tracker.reset();t.tracker.assign(t.target,t.state);assert.equal(t.tracker.targets(t.state,[1]).length,0);
t=assigned('point');markers=t.tracker.targets(t.state,[1,2]);assert.equal(markers.length,1);assert.equal(markers[0].targetType,'point');assert.deepEqual([markers[0].x,markers[0].y],[800,800]);
for(const mutate of [state=>{unit(state,1).path=[];},state=>{unit(state,1).target=10;},state=>{unit(state,1).dest[0]++;}]){t=assigned('point',[1]);mutate(t.state);assert.equal(t.tracker.targets(t.state,[1]).length,0);}
t=assigned('point',[1]);t.tracker.reset();unit(t.state,1).path=[];t.tracker.assign(t.target,t.state);assert.equal(t.tracker.targets(t.state,[1]).length,1);t.state.time+=.05;assert.equal(t.tracker.targets(t.state,[1]).length,0);
t=assigned('unit',[1]);t.state.time=20;const second=intent(t.state,[2],{x:320,y:420});unit(t.state,2).order='attackBuilding';unit(t.state,2).target=1;unit(t.state,2).dest=[second.x,second.y];t.tracker.assign(second,t.state);assert.equal(t.tracker.targets(t.state,[1,2]).length,2);
t.tracker.assign(null,t.state);t.tracker.clear(null);t.tracker.reset();
for(const winner of [0,1]){t=assigned();t.state.winner=winner;assert.equal(t.tracker.targets(t.state,[1]).length,0);assert.equal(t.tracker.targets(assigned().state,[1]).length,0);}
console.log('PASS: combat-only attack intent, original HP0, sprite/radius/building/bridge/point targets, route gates, moving targets, stationary guns, selected deduplication, override/loss/arrival/restart cleanup.');
