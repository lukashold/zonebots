// Behavioral checks for capture intent; all positions below are synthetic.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const context={window:{}};vm.createContext(context);
vm.runInContext(fs.readFileSync(path.join(__dirname,'../dist/independent/flag-command.js'),'utf8'),context);
const {intent,createTracker}=context.window.FlagCommand;
function fixture(){return {
 map:{original:true},winner:null,
 catalog:[{type:'grunt',speed:40},{type:'jeep',speed:72},{type:'gatling',speed:0}],
 units:[{id:1,type:'grunt',team:0,hp:30},{id:2,type:'gatling',team:0,hp:50},{id:3,type:'jeep',team:0,hp:50},
  {id:4,type:'grunt',team:1,hp:30},{id:5,type:'grunt',team:0,hp:0},{id:6,type:'grunt',team:0,hp:-1},
  {id:7,type:'jeep',team:-1,hp:50},{id:8,type:'unknown',team:0,hp:50}],
 sectors:[{id:0,owner:0,x:100,y:100},{id:1,owner:1,x:300,y:200},{id:2,owner:-1,x:500,y:300}]
};}
const bounds=s=>({x:s.x-2,y:s.y-26,w:64,h:s.owner<0?22:48}),cloth={x:350,y:180};
let state=fixture(),before=JSON.stringify(state),selected=new Set([2,3,1,3]);
let result=intent(state,selected,cloth,{},bounds);
assert.equal(result.sector,state.sectors[1]);assert.deepEqual(Array.from(result.ids),[3,1]);
assert.equal(result.x,300);assert.equal(result.y,200);
assert(Math.hypot(cloth.x-result.x,cloth.y-result.y)>22,'Fixture must exercise a cloth click outside contact radius.');
assert.equal(JSON.stringify(state),before);assert.deepEqual(Array.from(selected),[2,3,1]);

assert.deepEqual(Array.from(intent(state,[1],cloth,{},bounds).ids),[1],'Array selections work as well as Sets.');
assert.equal(intent(state,[],cloth,{},bounds),null);
for(const id of [2,4,6,7,8,999])assert.equal(intent(state,[id],cloth,{},bounds),null,`Ineligible unit ${id}`);
assert.deepEqual(Array.from(intent(state,[5],cloth,{},bounds).ids),[5],'Original HP0 units remain eligible.');
state.map.original=false;assert.equal(intent(state,[5],cloth,{},bounds),null);state.map.original=true;

for(const options of [{paused:true},{menu:true},{commandMode:'attack'}])assert.equal(intent(state,[1],cloth,options,bounds),null);
for(const winner of [0,1]){state.winner=winner;assert.equal(intent(state,[1],cloth,{},bounds),null);}state.winner=null;
result=intent(state,[1],{x:110,y:90},{},bounds);
assert.equal(result.sector.id,0);assert.equal(result.x,100);assert.equal(result.y,100,'Owned flags remain valid original move targets.');
assert.equal(intent(state,[1],{x:510,y:280},{},bounds).sector.id,2,'Neutral cloth hit is accepted.');
assert.equal(intent(state,[1],{x:510,y:320},{},bounds),null,'Neutral flag uses its shorter actual bounds.');
assert.equal(intent(state,[1],{x:400,y:400},{},bounds),null);
for(const change of [{home:true},{countsForProduction:false},{x:NaN},{y:Infinity}]){
 const other=fixture();Object.assign(other.sectors[1],change);assert.equal(intent(other,[1],cloth,{},bounds),null);
}
assert(intent(state,[1],cloth,{},bounds),'A missing countsForProduction field means an ordinary valid sector.');
for(const b of [null,{x:0,y:0,w:Infinity,h:40},{x:300,y:170,w:0,h:40}])assert.equal(intent(state,[1],cloth,{},()=>b),null);
for(const point of [null,{x:NaN,y:180},{x:350,y:Infinity}])assert.equal(intent(state,[1],point,{},bounds),null);
assert.equal(intent(null,[1],cloth,{},bounds),null);assert.equal(intent(state,null,cloth,{},bounds),null);

const overlap=fixture();overlap.sectors.push({...overlap.sectors[1],id:3,owner:-1});
assert.equal(intent(overlap,[1],cloth,{},()=>({x:290,y:170,w:64,h:48})).sector.id,3,'Last drawn overlapping flag has priority.');

function movingFixture(){const s=fixture();s.time=12.25;for(const u of s.units){u.order='move';u.path=[[280,200],[300,200]];}return s;}
function assigned(ids=[1],point=cloth){const s=movingFixture(),tracker=createTracker();tracker.assign(intent(s,ids,point,{},bounds),s);return {s,tracker};}
let tracked=assigned([1,3]),marker=tracked.tracker.targets(tracked.s,new Set([1,3]));
assert.equal(marker.length,1,'Selected units sharing a flag draw one destination hand.');
assert.equal(marker[0].sector,tracked.s.sectors[1]);assert.equal(marker[0].x,300);assert.equal(marker[0].y,200);assert.equal(marker[0].started,12.25);
assert.equal(tracked.tracker.targets(tracked.s,[]).length,0,'Unselected orders do not draw hands.');
assert.equal(tracked.tracker.targets(tracked.s,[1]).length,1,'Reselecting an active order restores its hand.');
tracked.tracker.clear(new Set([1]));assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0);
assert.equal(tracked.tracker.targets(tracked.s,[3]).length,1,'Overriding one unit leaves another order intact.');
tracked.tracker.reset();assert.equal(tracked.tracker.targets(tracked.s,[3]).length,0);

// Each of these conditions ends an order, even while its unit is unselected.
for(const mutate of [
 s=>{s.units=s.units.filter(u=>u.id!==1);},
 s=>{s.units[0].hp=-1;},
 s=>{s.units[0].team=1;},
 s=>{s.units[0].order='guard';},
 s=>{s.units[0].order='attack';},
 s=>{s.units[0].path=[];},
 s=>{delete s.units[0].path;},
 s=>{s.sectors=s.sectors.filter(sector=>sector.id!==1);},
 s=>{s.sectors[1].owner=0;},
 s=>{s.sectors[1].home=true;},
 s=>{s.sectors[1].countsForProduction=false;}
]){
 const t=assigned();mutate(t.s);assert.equal(t.tracker.targets(t.s,[]).length,0);
 // Restoring a live fixture must not resurrect a marker for the expired order.
 assert.equal(t.tracker.targets(movingFixture(),[1]).length,0);
}
tracked=assigned([5]);assert.equal(tracked.tracker.targets(tracked.s,[5]).length,1,'HP0 original units retain active markers.');
tracked.s.map.original=false;assert.equal(tracked.tracker.targets(tracked.s,[5]).length,0);

tracked=assigned([1],{x:110,y:90});
assert.equal(tracked.tracker.targets(tracked.s,[1]).length,1,'Moving to an already owned flag remains marked.');
tracked.s.units[0].path=[];assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0,'Owned flag marker ends on arrival.');

tracked=assigned([1]);tracked.s.sectors[1].owner=-1;
assert.equal(tracked.tracker.targets(tracked.s,[1]).length,1,'A foreign-owner change still leaves a valid target.');
tracked.s.sectors[1]={...tracked.s.sectors[1],x:310,y:210};
marker=tracked.tracker.targets(tracked.s,[1]);assert.equal(marker[0].sector,tracked.s.sectors[1]);assert.equal(marker[0].x,310);assert.equal(marker[0].y,210);

tracked=assigned([1]);tracked.s.time=20;
tracked.tracker.assign(intent(tracked.s,[3],{x:510,y:280},{},bounds),tracked.s);
marker=tracked.tracker.targets(tracked.s,[1,3]);assert.equal(marker.length,2);
assert.deepEqual(Array.from(marker,m=>[m.sector.id,m.started]),[[1,12.25],[2,20]]);
tracked.tracker.assign(intent(tracked.s,[3],cloth,{},bounds),tracked.s);
marker=tracked.tracker.targets(tracked.s,[1,3]);assert.equal(marker.length,1);assert.equal(marker[0].started,12.25);

tracked={s:movingFixture(),tracker:createTracker()};
tracked.tracker.assign({sector:tracked.s.sectors[1],ids:[2,4,6,7,8,999]},tracked.s);
assert.equal(tracked.tracker.targets(tracked.s,[2,4,6,7,8,999]).length,0,'Assignment rejects ineligible IDs.');
tracked.tracker.assign(null,tracked.s);tracked.tracker.clear(null);tracked.tracker.reset();
for(const winner of [0,1]){tracked=assigned();tracked.s.winner=winner;assert.equal(tracked.tracker.targets(tracked.s,[1]).length,0);assert.equal(tracked.tracker.targets(movingFixture(),[1]).length,0,'Ended games do not retain stale orders.');}
console.log('PASS: flag intent and persistent destination markers, mobile/HP0 eligibility, owned/foreign targets, selection changes, deduplication, manual override, arrival/capture/death pruning and input immutability.');
