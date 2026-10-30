// Vehicle-entry presentation behavior; all positions and units are synthetic.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const context={window:{}};vm.createContext(context);
vm.runInContext(fs.readFileSync(path.join(__dirname,'../dist/independent/vehicle-command.js'),'utf8'),context);
const {intent,createTracker}=context.window.VehicleCommand;
function fixture(){return {map:{original:true},winner:null,time:12.5,
 catalog:[{type:'grunt',class:0},{type:'tough',class:0},{type:'jeep',class:1},{type:'medium',class:1},{type:'gatling',class:2},{type:'apc',class:1}],
 units:[
  {id:1,type:'grunt',team:0,hp:30},{id:2,type:'tough',team:0,hp:100},{id:3,type:'jeep',team:0,hp:50},
  {id:4,type:'gatling',team:0,hp:50},{id:5,type:'grunt',team:1,hp:30},{id:6,type:'grunt',team:0,hp:0},
  {id:7,type:'grunt',team:0,hp:-1},{id:8,type:'unknown',team:0,hp:50},{id:9,type:'grunt',team:-1,hp:30},
  {id:10,type:'medium',team:-1,hp:200,x:300,y:200,crew:null,cargo:[]},
  {id:11,type:'gatling',team:-1,hp:50,x:500,y:200,crew:null,cargo:[]},
  {id:12,type:'apc',team:0,hp:200,x:700,y:200,crew:{kind:'grunt',hp:30},cargo:[{},{}]},
  {id:13,type:'apc',team:1,hp:200,x:900,y:200,crew:null,cargo:[]},
  {id:14,type:'jeep',team:0,hp:50,x:1000,y:200,crew:{kind:'grunt',hp:30},cargo:[]},
  {id:15,type:'apc',team:-1,hp:200,x:1100,y:200,crew:null,cargo:Array(6).fill({})},
  {id:16,type:'apc',team:0,hp:200,x:1300,y:200,crew:{kind:'grunt',hp:30},cargo:Array(6).fill({})},
  {id:17,type:'jeep',team:-1,hp:0,x:1500,y:200,crew:null,cargo:[]},
  {id:18,type:'jeep',team:-1,hp:-1,x:1700,y:200,crew:null,cargo:[]},
  {id:19,type:'jeep',team:-1,hp:50,x:1900,y:200,crew:{kind:'grunt',hp:30},cargo:[]},
  {id:20,type:'grunt',team:-1,hp:30,x:2000,y:200,crew:null},
  {id:21,type:'unknown',team:-1,hp:50,x:2100,y:200,crew:null},
  {id:22,type:'apc',team:0,hp:200,x:2300,y:200,moving:true,crew:{kind:'grunt',hp:30},cargo:[]}
 ]};}
const unit=(s,id)=>s.units.find(u=>u.id===id),at=(s,id)=>({x:unit(s,id).x,y:unit(s,id).y});
const bounds=u=>u.id===10?{x:280,y:170,w:80,h:60}:null;
let state=fixture(),before=JSON.stringify(state),selected=new Set([3,4,2,1,2]);
let result=intent(state,selected,{x:282,y:172},{},bounds);
assert.equal(result.unit,unit(state,10));assert.equal(result.kind,'enter');assert.deepEqual(Array.from(result.ids),[2,1]);
assert.equal(result.x,300);assert.equal(result.y,200);assert.equal(JSON.stringify(state),before);assert.deepEqual(Array.from(selected),[3,4,2,1]);
assert.equal(intent(state,[1],at(state,11),{},bounds).kind,'enter','Infantry may crew a vacant gun.');
assert.equal(intent(state,[1],at(state,12),{},bounds).kind,'board','Infantry may board a friendly APC.');
assert.equal(intent(state,[1],at(state,15),{},bounds).kind,'enter','Vacant APC driver is separate from passenger capacity.');
assert.equal(intent(state,[1],at(state,22),{},bounds).kind,'board','Moving friendly APC remains a supported boarding target.');
for(const id of [13,14,16,18,19,20,21])assert.equal(intent(state,[1],at(state,id),{},bounds),null,`Target ${id} is ineligible.`);
for(const id of [3,4,5,7,8,9,999])assert.equal(intent(state,[id],at(state,10),{},bounds),null,`Actor ${id} is ineligible.`);
assert.equal(intent(state,[],at(state,10),{},bounds),null);
assert(intent(state,[6],at(state,17),{},bounds),'Original HP0 infantry and hardware are usable.');
state.map.original=false;assert.equal(intent(state,[6],at(state,10),{},bounds),null);assert.equal(intent(state,[1],at(state,17),{},bounds),null);state.map.original=true;
for(const options of [{paused:true},{menu:true},{commandMode:'attack'}])assert.equal(intent(state,[1],at(state,10),options,bounds),null);
for(const winner of [0,1]){state.winner=winner;assert.equal(intent(state,[1],at(state,10),{},bounds),null);}state.winner=null;
assert.equal(intent(state,[1],{x:521.9,y:200},{},bounds).unit.id,11);
assert.equal(intent(state,[1],{x:522,y:200},{},bounds),null,'Fallback radius uses a strict22-pixel limit.');
assert.equal(intent(state,[1],at(state,11),{},()=>({x:NaN,y:0,w:20,h:20})).unit.id,11,'Invalid renderer bounds fall back to target radius.');
for(const point of [null,{x:NaN,y:200},{x:300,y:Infinity}])assert.equal(intent(state,[1],point,{},bounds),null);
assert.equal(intent(null,[1],{x:300,y:200},{},bounds),null);assert.equal(intent(state,null,{x:300,y:200},{},bounds),null);

function assigned(ids=[1],targetId=10){
 const s=fixture(),tracker=createTracker(),order=intent(s,ids,at(s,targetId),{},bounds);
 for(const id of order.ids){const u=unit(s,id);u.order=order.kind;u.target=targetId;u.path=[];}
 tracker.assign(order,s);return {s,tracker};
}
let tracked=assigned([1,2]),markers=tracked.tracker.targets(tracked.s,new Set([1,2]));
assert.equal(markers.length,1);assert.equal(markers[0].unit,unit(tracked.s,10));assert.equal(markers[0].kind,'enter');assert.equal(markers[0].x,300);assert.equal(markers[0].y,200);assert.equal(markers[0].started,12.5);
assert.equal(tracked.tracker.targets(tracked.s,[]).length,0);assert.equal(tracked.tracker.targets(tracked.s,[1]).length,1,'Reselecting an active actor restores its arrow.');
tracked.tracker.clear([1]);assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0);assert.equal(tracked.tracker.targets(tracked.s,[2]).length,1);
tracked.tracker.reset();assert.equal(tracked.tracker.targets(tracked.s,[2]).length,0);

for(const mutate of [
 s=>{s.units=s.units.filter(u=>u.id!==1);},s=>{unit(s,1).hp=-1;},s=>{unit(s,1).team=1;},
 s=>{unit(s,1).order='guard';},s=>{unit(s,1).order='move';},s=>{unit(s,1).target=11;},
 s=>{s.units=s.units.filter(u=>u.id!==10);},s=>{unit(s,10).hp=-1;},
 s=>{unit(s,10).team=0;unit(s,10).crew={kind:'grunt',hp:30};},
 s=>{unit(s,10).team=1;},s=>{unit(s,10).crew={kind:'grunt',hp:30};}
]){
 const t=assigned();mutate(t.s);assert.equal(t.tracker.targets(t.s,[]).length,0);
 const restored=assigned().s;assert.equal(t.tracker.targets(restored,[1]).length,0,'Expired entry arrows must not resurrect.');
}
tracked=assigned([1],22);const carrier=unit(tracked.s,22);carrier.x+=80;carrier.y+=40;
markers=tracked.tracker.targets(tracked.s,[1]);assert.equal(markers[0].x,2380);assert.equal(markers[0].y,240,'Boarding arrow follows live carrier position.');
carrier.cargo=Array(6).fill({});assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0,'A full carrier cancels pending arrow feedback.');
tracked=assigned([1],12);unit(tracked.s,12).team=1;assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0);
tracked=assigned([6],17);assert.equal(tracked.tracker.targets(tracked.s,[6]).length,1);tracked.s.map.original=false;assert.equal(tracked.tracker.targets(tracked.s,[6]).length,0);

tracked=assigned([1]);tracked.s.time=20;
let second=intent(tracked.s,[2],at(tracked.s,12),{},bounds);unit(tracked.s,2).order='board';unit(tracked.s,2).target=12;tracked.tracker.assign(second,tracked.s);
markers=tracked.tracker.targets(tracked.s,[1,2]);assert.deepEqual(Array.from(markers,m=>[m.unit.id,m.kind,m.started]),[[10,'enter',12.5],[12,'board',20]]);
second=intent(tracked.s,[2],at(tracked.s,10),{},bounds);unit(tracked.s,2).order='enter';unit(tracked.s,2).target=10;tracked.tracker.assign(second,tracked.s);
markers=tracked.tracker.targets(tracked.s,[1,2]);assert.equal(markers.length,1);assert.equal(markers[0].started,12.5);
for(const winner of [0,1]){tracked=assigned();tracked.s.winner=winner;assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0);assert.equal(tracked.tracker.targets(assigned().s,[1]).length,0);}
tracked={s:fixture(),tracker:createTracker()};tracked.tracker.assign(null,tracked.s);tracked.tracker.clear(null);tracked.tracker.reset();
console.log('PASS: vacant vehicle/gun entry and friendly APC boarding intent, infantry/HP0 eligibility, renderer bounds/fallback, selection/order lifecycle, full/occupied/dead targets, moving-carrier following and game-end cleanup.');
