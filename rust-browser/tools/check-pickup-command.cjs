// Portable behavioral tests; all positions and units are synthetic.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const env={window:{}};vm.createContext(env);
vm.runInContext(fs.readFileSync(path.join(__dirname,'../dist/independent/pickup-command.js'),'utf8'),env);
const {intent,createTracker}=env.window.PickupCommand;
function fixture(){return {map:{original:true},winner:null,time:12.5,
 catalog:[{type:'grunt',class:0},{type:'psycho',class:0},{type:'jeep',class:1},{type:'gatling',class:2}],
 units:[{id:1,type:'grunt',team:0,hp:30},{id:2,type:'jeep',team:0,hp:50},{id:3,type:'psycho',team:0,hp:50},
  {id:4,type:'grunt',team:1,hp:30},{id:5,type:'grunt',team:0,hp:0},{id:6,type:'grunt',team:0,hp:-1},
  {id:7,type:'gatling',team:0,hp:50},{id:8,type:'unknown',team:0,hp:30},{id:9,type:'grunt',team:-1,hp:30}],
 crates:[{x:300,y:200,taken:false},{x:500,y:300,taken:false},{x:700,y:400,taken:true}]};}
const bounds=c=>({x:c.x,y:c.y,w:64,h:32}),point={x:350,y:220};
let state=fixture(),before=JSON.stringify(state),selected=new Set([2,3,7,1,3]),result=intent(state,selected,point,{},bounds);
assert.equal(result.crate,state.crates[0]);assert.deepEqual(Array.from(result.ids),[3,1]);
assert.equal(result.x,300);assert.equal(result.y,200);assert(Math.hypot(point.x-result.x,point.y-result.y)>22);
assert.equal(JSON.stringify(state),before);assert.deepEqual([...selected],[2,3,7,1]);
assert.deepEqual(Array.from(intent(state,[1,1],point,{},bounds).ids),[1]);
for(const ids of [[],null,[2],[4],[6],[7],[8],[9],[999]])assert.equal(intent(state,ids,point,{},bounds),null);
assert(intent(state,[5],point,{},bounds),'Original zero-HP infantry can collect crates');
state.map.original=false;assert.equal(intent(state,[5],point,{},bounds),null);state.map.original=true;
for(const options of [{paused:true},{menu:true},{commandMode:'attack'},{placement:{}},{commandMode:'placement'}])assert.equal(intent(state,[1],point,options,bounds),null);
for(const winner of [0,1]){state.winner=winner;assert.equal(intent(state,[1],point,{},bounds),null);}state.winner=null;
assert.equal(intent(state,[1],{x:710,y:410},{},bounds),null,'Taken crates are not interactive');
assert.equal(intent(state,[1],{x:299,y:210},{},bounds),null,'Valid sprite bounds suppress larger fallback hits');
assert(intent(state,[1],{x:289,y:191}), 'Legacy fallback has a 24 by 20 click area');
assert.equal(intent(state,[1],{x:287,y:191}),null);
for(const bad of [{x:NaN,y:200},{x:300,y:Infinity}]){const s=fixture();s.crates=[bad];assert.equal(intent(s,[1],point,{},bounds),null);}
for(const box of [{x:300,y:200,w:0,h:32},{x:NaN,y:200,w:64,h:32},{x:300,y:200,w:64,h:Infinity}])assert.equal(intent(state,[1],point,{},()=>box),null);
for(const p of [null,{x:NaN,y:200},{x:300,y:Infinity}])assert.equal(intent(state,[1],p,{},bounds),null);
assert.equal(intent(null,[1],point,{},bounds),null);
const overlap=fixture();overlap.crates=[{x:300,y:210,taken:false},{x:300,y:200,taken:false}];
assert.equal(intent(overlap,[1],point,{},bounds).crate,overlap.crates[0],'Later Y-sorted crate takes precedence');
overlap.crates.push({...overlap.crates[0]});assert.equal(intent(overlap,[1],point,{},bounds).crate,overlap.crates[2],'Equal-depth crates keep reverse array order');

function moving(){const s=fixture();for(const [i,u]of s.units.entries()){u.order='move';u.dest=[300+i*12,200];u.path=[[280,200],[304,208]];}return s;}
function assigned(ids=[1,3]){const s=moving(),tracker=createTracker();tracker.assign(intent(s,ids,point,{},bounds),s);return {s,tracker};}
let t=assigned(),markers=t.tracker.targets(t.s,new Set([1,3]));
assert.equal(markers.length,1);assert.equal(markers[0].crate,t.s.crates[0]);
assert.deepEqual([markers[0].x,markers[0].y,markers[0].started],[300,200,12.5]);
assert.equal(t.tracker.targets(t.s,[]).length,0);assert.equal(t.tracker.targets(t.s,[1]).length,1,'Reselecting retains a valid active order');
t.tracker.clear([1]);assert.equal(t.tracker.targets(t.s,[1]).length,0);assert.equal(t.tracker.targets(t.s,[3]).length,1);
t.tracker.reset();assert.equal(t.tracker.targets(t.s,[3]).length,0);
for(const mutate of [
 s=>{s.units=s.units.filter(u=>u.id!==1);},s=>{s.units[0].hp=-1;},s=>{s.units[0].team=1;},
 s=>{s.units[0].type='jeep';},s=>{s.units[0].order='guard';},s=>{s.units[0].order='attack';},
 s=>{s.units[0].path=[];},s=>{delete s.units[0].path;},s=>{s.units[0].dest=[500,300];},
 s=>{s.units[0].dest=null;},s=>{s.crates[0].taken=true;},s=>{s.crates.shift();},
 s=>{s.crates[0].x=NaN;},s=>{s.time=0;}
]){
 t=assigned([1]);mutate(t.s);assert.equal(t.tracker.targets(t.s,[]).length,0);
 assert.equal(t.tracker.targets(moving(),[1]).length,0,'Expired orders cannot revive after a new same-ID move back');
}
t=assigned([1]);t.s.units[0].dest[0]+=12;assert.equal(t.tracker.targets(t.s,[1]).length,0,'Assignment snapshots destination values rather than sharing its array');
t=assigned([5]);assert.equal(t.tracker.targets(t.s,[5]).length,1);t.s.map.original=false;assert.equal(t.tracker.targets(t.s,[5]).length,0);
t=assigned([1]);t.s.crates.reverse();assert.equal(t.tracker.targets(t.s,[1]).length,1,'Crate lookup survives array reorder and serialized object replacement');
t.s.crates=t.s.crates.map(c=>({...c}));assert.equal(t.tracker.targets(t.s,[1]).length,1);
// Same-cell moves have no path. A first-frame acknowledgement is allowed, but
// an unreachable or completed empty route cannot leave an indefinite hand.
t={s:moving(),tracker:createTracker()};t.s.units[0].path=[];
t.tracker.assign(intent(t.s,[1],point,{},bounds),t.s);assert.equal(t.tracker.targets(t.s,[1]).length,1);
t.s.time+=.05;assert.equal(t.tracker.targets(t.s,[1]).length,0);
t={s:moving(),tracker:createTracker()};t.s.units[0].path=[];
t.tracker.assign(intent(t.s,[1],point,{},bounds),t.s);t.s.time+=.05;t.s.units[0].path=[[304,208]];
assert.equal(t.tracker.targets(t.s,[1]).length,1);t.s.units[0].path=[];assert.equal(t.tracker.targets(t.s,[1]).length,0);
t={s:moving(),tracker:createTracker()};t.tracker.assign({crate:t.s.crates[0],ids:[2,4,6,7,8,9,999]},t.s);
assert.equal(t.tracker.targets(t.s,[2,4,6,7,8,9,999]).length,0);
t.tracker.assign(null,t.s);t.tracker.clear(null);t.tracker.reset();
for(const winner of [0,1]){t=assigned([1]);t.s.winner=winner;assert.equal(t.tracker.targets(t.s,[1]).length,0);assert.equal(t.tracker.targets(moving(),[1]).length,0);}
console.log('PASS: pickup sprite/fallback hits, infantry filtering, HP0 and mode guards, depth order, fixed crate destinations, deduplicated persistent hands, selection/manual override, route/destination/death/taken/restart cleanup.');
