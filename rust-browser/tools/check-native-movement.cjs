// Independent, behavior-level traces through the shipped Rust/Wasm ABI.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const patterns=[[1,0,0,0,0],[1,0,0,0],[1,0,0],[1,0,0,1,0],[1,0],[1,1,0],[1,1,1,1,0],[1],[2,1,1],[2,1],[2],[3,2,2],[3,2]];
const controls={grunt:40,psycho:40,tough:30,sniper:50,pyro:40,laser:50,jeep:60,medium:40,heavy:30,light:50,launcher:20,apc:50,crane:50,truck:60,firetruck:50};
function field(kind='grunt',code=0){const width=100,height=48,n=width*height;return{map:{width,height,tileSize:32,world:0,seed:1951,revision:0,original:true,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(code),territory:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[]},sectors:[],buildings:[],crates:[],originalObjects:[],spawns:[{type:kind,team:0,x:320,y:320,squad:1},{type:'gatling',team:1,x:2900,y:1400,squad:2}]};}
(async()=>{const e=(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{})).instance.exports,enc=new TextEncoder,dec=new TextDecoder;let state;
function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);state={...state,...s};return state;}
function init(m){state=null;return req({op:'init',mission:0,difficulty:1,seed:1951,aiTeams:[],originalMission:m});}
const move=(id,x,y,team=0)=>req({op:'command',kind:'move',ids:[id],team,x,y});
const tick=dt=>req({op:'tick',dt,compact:true,knownMapSeed:1951,knownMapRevision:0});
const get=id=>state.units.find(u=>u.id===id),near=(a,b)=>assert(Math.abs(a-b)<1e-7,`${a} != ${b}`);
let traces=0,positions=0;
for(const [kind,control]of Object.entries(controls))for(const code of [0,7]){
 init(field(kind,code));const id=state.units[0].id,p=patterns[control/10+(code===7?2:0)];move(id,2500,320);let x=320;
 for(let t=1;t<=120;t++){tick(.059);x+=2*p[t%p.length];near(get(id).x,x);near(get(id).y,320);positions++;}traces++;
}
// Integer clocks do not advance at58ms. Both factions sample the shared phase,
// including an idle unit first commanded after five elapsed native updates.
init(field('jeep'));let id=state.units[0].id;move(id,2500,320);tick(.058);near(get(id).x,320);tick(.001);near(get(id).x,322);
const m=field();m.spawns[1]={type:'grunt',team:1,x:320,y:1100,squad:2};init(m);const a=state.units[0].id,b=state.units[1].id;move(a,2500,320);for(let i=0;i<5;i++)tick(.059);move(b,2500,1100,1);for(let i=0;i<5;i++)tick(.059);near(get(a).x,330);near(get(b).x,326);
// Bresenham reference uses a closed form for the minor coordinate. This does
// not use the error-accumulator algorithm of the implementation.
for(const [dx,dy]of [[200,100],[100,200],[-200,100],[-100,200],[200,-100],[100,-200],[-200,-100],[-100,-200]]){
 const m=field('jeep',7);m.spawns[0].x=900;m.spawns[0].y=900;init(m);id=state.units[0].id;move(id,900+dx,900+dy);let n=0;
 for(let t=1;t<=65;t++){tick(.059);n=Math.min(Math.max(Math.abs(dx),Math.abs(dy))/2,n+patterns[8][t%3]);const major=Math.max(Math.abs(dx),Math.abs(dy))/2,minor=Math.min(Math.abs(dx),Math.abs(dy))/2,q=Math.floor((2*n*minor+major-1)/(2*major));near(get(id).x,900+2*Math.sign(dx)*(Math.abs(dx)>=Math.abs(dy)?n:q));near(get(id).y,900+2*Math.sign(dy)*(Math.abs(dx)>=Math.abs(dy)?q:n));positions++;}traces++;
}
function partition(parts){init(field('sniper'));const id=state.units[0].id;move(id,2500,320);for(const dt of parts)tick(dt);return[get(id).x,get(id).y];}
assert.deepEqual(partition(Array(100).fill(.01)),partition(Array(10).fill(.1)),'Frame partitions must not change movement');
console.log(JSON.stringify({traces,positions,clockMilliseconds:59,sharedLateStart:true,partitionInvariant:true}));
console.log('PASS: GOG native tick traces,15 mobile types, terrain selection, clock threshold, shared idle/marching phases, eight diagonal octants and frame partition invariance.');
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
