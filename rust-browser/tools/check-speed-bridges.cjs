// Real Wasm: relative speeds, faction equality, and the complete bridge cycle.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),args=process.argv.slice(2),at=args.indexOf('--wasm'),wasm=at<0?path.join(root,'dist/independent/engine.wasm'):path.resolve(args[at+1]);
function field(original=true){const width=100,height=48,n=width*height;return{map:{width,height,tileSize:32,world:0,seed:985,revision:0,original,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),territory:Array(n).fill(0),originalTiles:Array(n).fill(120),bridgeBase:Array(n).fill(-1),bridgeHp:[]},sectors:[],buildings:[],crates:[],originalObjects:[],walls:[],spawns:[]};}
(async()=>{const e=(await WebAssembly.instantiate(fs.readFileSync(wasm),{})).instance.exports,enc=new TextEncoder,dec=new TextDecoder;let state;
function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const r=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!r.error,r.error);state={...state,...r};return state;}
function init(m,difficulty=1){state=null;return req({op:'init',mission:0,difficulty,seed:985,aiTeams:[],originalMission:m});}
const near=(a,b)=>assert(Math.abs(a-b)<1e-6,`${a} != ${b}`),tick=()=>req({op:'tick',dt:.1,compact:true,knownMapSeed:state.map.seed,knownMapRevision:state.map.revision});
let speedCases=0;
for(const [kind,speed,mean]of [['grunt',16,0.5],['psycho',16,0.5],['tough',12,0.4],['sniper',22,2/3],['pyro',16,0.5],['laser',22,2/3],['jeep',26,0.8],['medium',16,0.5],['heavy',12,0.4],['light',22,2/3],['crane',22,2/3]])for(const team of [0,1])for(const difficulty of [0,1,2,3]){
 const m=field();m.spawns=[{type:kind,team,x:320,y:320,squad:1},{type:'gatling',team:1-team,x:2900,y:1400,squad:2}];init(m,difficulty);const id=state.units[0].id;
 near(state.catalog.find(u=>u.type===kind).speed,mean*2/.059);req({op:'command',kind:'move',ids:[id],team,x:2500,y:320});for(let i=0;i<10;i++)tick();near(state.units.find(u=>u.id===id).x-320,speed);speedCases++;
}
// A600HP original bridge must still be unbuilt at520HP; the old400 threshold
// may neither reject its repair order nor release vehicles prematurely.
for(const team of [0,1]){
 const m=field(),w=m.map.width;for(let y=0;y<m.map.height;y++)for(let x=12;x<15;x++){const j=y*w+x;m.map.tiles[j]=3;m.map.terrainCodes[j]=40;}
 m.map.bridgeHp=[0];m.map.bridgeMaxHp=[600];for(let x=12;x<15;x++){const j=12*w+x;m.map.bridgeBase[j]=0;m.map.terrainCodes[j]=23;}
 m.spawns=[{type:'jeep',team,x:272,y:400,squad:1},{type:'crane',team,x:336,y:400,squad:2},{type:'gatling',team:1-team,x:2900,y:1400,squad:3}];init(m);const car=state.units[0].id,crane=state.units[1].id;
 req({op:'command',kind:'move',ids:[car],team,x:656,y:400});for(let i=0;i<40;i++){tick();assert(state.units.find(u=>u.id===car).x<384);}
 req({op:'command',kind:'repairBridge',ids:[crane],team,x:432,y:400});for(let i=0;i<80;i++)tick();req({op:'snapshot'});near(state.map.bridgeHp[0],520);assert.equal(state.map.tiles[12*w+13],3);
 // Issuing repair again at520 verifies command validation uses600 as well.
 req({op:'command',kind:'repairBridge',ids:[crane],team,x:432,y:400});assert.equal(state.units.find(u=>u.id===crane).order,'repairBridge');for(let i=0;i<20;i++)tick();near(state.map.bridgeHp[0],600);assert.equal(state.map.tiles[12*w+13],5);assert.equal(state.map.revision,1);
 req({op:'command',kind:'move',ids:[car],team,x:656,y:400});for(let i=0;i<180;i++)tick();near(state.units.find(u=>u.id===car).x,656);
}
// All actual destroyed mission bridges must accept a crane from a legal nearby
// bank and become traversable. Local private map data is never embedded here.
const campaign=JSON.parse(fs.readFileSync(path.join(root,'dist/original/campaign.json')));assert(campaign.version>=3);let actual=0;
for(const mission of campaign.missions)for(const bridge of mission.bridges.filter(b=>b.initiallyDestroyed)){
 const m=JSON.parse(JSON.stringify(mission)),map=m.map;let pair;
 for(const cell of bridge.tiles){const tx=cell%map.width,ty=Math.floor(cell/map.width);for(let dy=-3;dy<=3&&!pair;dy++)for(let dx=-3;dx<=3;dx++){const x=tx+dx,y=ty+dy,j=y*map.width+x;if(x<1||y<1||x>=map.width-1||y>=map.height-1||![0,1].includes(map.tiles[j])||![0,7,51].includes(map.terrainCodes[j]&127))continue;const p=[x*32+16,y*32+16],target=[tx*32+16,ty*32+16];if(Math.hypot(p[0]-target[0],p[1]-target[1])>100||m.buildings.some(b=>b.hp>0&&p[0]>b.x&&p[0]<b.x+b.w&&p[1]>b.y&&p[1]<b.y+b.h))continue;pair={p,target};break;}if(pair)break;}
 assert(pair,`Mission${m.number} bridge${bridge.id} has a nearby dry bank`);m.spawns=[{type:'crane',team:0,x:pair.p[0],y:pair.p[1],squad:1},{type:'gatling',team:1,x:32,y:32,squad:2}];m.sectors.forEach(s=>s.owner=-1);m.buildings.forEach(b=>{b.team=-1;b.product=null;});init(m);const id=state.units[0].id;assert.equal(state.map.bridgeHp[bridge.id],0);assert(bridge.tiles.every(j=>state.map.tiles[j]===3));req({op:'command',kind:'repairBridge',ids:[id],team:0,x:pair.target[0],y:pair.target[1]});for(let i=0;i<250&&state.map.bridgeHp[bridge.id]<bridge.maxHp;i++)tick();near(state.map.bridgeHp[bridge.id],bridge.maxHp);assert(bridge.tiles.every(j=>state.map.tiles[j]===5));actual++;
}
assert.equal(actual,19);console.log(JSON.stringify({speedCases,syntheticBridgeTeams:2,repairedOriginalBridges:actual}));console.log('PASS: native relative speeds, GOG59ms movement clock, faction/difficulty equality,600HP crane repairs, blocked/reopened crossing and all19 real destroyed bridges.');
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
