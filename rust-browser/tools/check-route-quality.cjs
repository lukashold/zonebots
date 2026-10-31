// Behavior-level navigation checks. Original maps remain in the optional local
// resource pack; the public fixtures below contain only newly authored terrain.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),options={wasm:path.join(root,'dist/independent/engine.wasm'),campaign:path.join(root,'dist/original/campaign.json')};
for(let i=2;i<process.argv.length;i+=2){const key=process.argv[i].replace(/^--/,'');assert(['wasm','campaign'].includes(key));assert(process.argv[i+1]);options[key]=path.resolve(process.argv[i+1]);}
const copy=v=>JSON.parse(JSON.stringify(v)),xy=(x,y)=>[(x+.5)*32,(y+.5)*32],distance=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(options.wasm),{}),e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function request(v){const bytes=enc.encode(JSON.stringify(v)),p=e.alloc(bytes.length);new Uint8Array(e.memory.buffer,p,bytes.length).set(bytes);let n;try{n=e.run(p,bytes.length);}finally{e.free(p,bytes.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 function fixture(){const width=32,height=24,n=width*height;return {map:{width,height,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:9187,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),waterTile:0},sectors:[],buildings:[],crates:[],originalObjects:[],walls:[],spawns:[{type:'medium',team:0,x:112,y:336,squad:1},{type:'gatling',team:1,x:32,y:704,squad:2}]};}
 const init=m=>request({op:'init',mission:0,difficulty:1,seed:9187,aiTeams:[],originalMission:m});
 function walk(m,goals,inspect=()=>{}){let s=init(m),u=s.units.find(u=>u.team===0),id=u.id,previous=[u.x,u.y],length=0,ticks=0;const terrain=s.map;for(const goal of goals){s=request({op:'command',kind:'move',team:0,ids:[id],x:goal[0],y:goal[1]});u=s.units.find(u=>u.id===id);for(let i=0;i<4000&&u.path.length;i++){s=request({op:'tick',dt:.05,compact:true});u=s.units.find(u=>u.id===id);assert(u,'Mover must survive');ticks++;const p=[u.x,u.y];length+=distance(previous,p);previous=p;inspect(p,terrain,s);assert.equal(s.winner,null,'Fixture must not end while checking movement');}assert(!u.path.length,'Movement must eventually finish');assert(distance([u.x,u.y],goal)<.01,`Requested endpoint ${goal} was replaced by ${u.x},${u.y}`);}return {seconds:ticks*.05,length,end:[u.x,u.y]};}
 // A long oblique command should not bounce through every intervening tile
 // center, and arbitrary legal click positions must survive route construction.
 let m=fixture();m.spawns[0].x=179;m.spawns[0].y=340;const end=[771,549],direct=distance([179,340],end);
 const flat=walk(m,[end]);assert(flat.length<=direct*1.015,`Open-field detour ${flat.length} exceeds direct ${direct}`);
 // Streets are useful when their actual speed advantage pays for the approach.
 m=fixture();const start=xy(3,10),goal=xy(28,10);const plain=walk(m,[goal]);
 for(let x=3;x<=28;x++){const j=9*m.map.width+x;m.map.tiles[j]=1;m.map.terrainCodes[j]=7;}
 const road=walk(m,[goal]);assert(road.seconds<plain.seconds*.93,`Useful nearby road lost: ${road.seconds}s vs ground ${plain.seconds}s`);
 // A cheap route across water is still invalid. The only legal crossing is a
 // bridge displaced above the direct line; verify the actual moving positions.
 m=fixture();m.spawns[0].type='jeep';for(let y=0;y<m.map.height;y++){const j=y*m.map.width+12;m.map.tiles[j]=3;m.map.terrainCodes[j]=40;}
 const bridge=6*m.map.width+12;m.map.tiles[bridge]=5;m.map.terrainCodes[bridge]=23;m.map.bridgeBase[bridge]=0;m.map.bridgeHp=[400];let crossed=false;
 const crossing=walk(m,[goal],(p,map)=>{const x=Math.floor(p[0]/32),y=Math.floor(p[1]/32),j=y*map.width+x;assert.notEqual(map.terrainCodes[j],40,'Shortcut crossed deep water');if(x===12){assert.equal(y,6,'Cross only the bridge');crossed=true;}});assert(crossed,'The actual unit must traverse the displaced bridge');
 // No smoothing may cut a blocked coastal diagonal from an off-center anchor.
 m=fixture();m.spawns[0]={type:'jeep',team:0,x:127,y:127,squad:1};m.map.tiles.fill(2);m.map.terrainCodes.fill(19);for(const [x,y]of [[3,3],[4,4]]){m.map.tiles[y*32+x]=0;m.map.terrainCodes[y*32+x]=0;}
 let s=init(m),u=s.units.find(u=>u.team===0);s=request({op:'command',kind:'move',ids:[u.id],team:0,x:144,y:144});assert.equal(s.units.find(v=>v.id===u.id).path.length,0,'Blocked diagonal must remain unreachable');
 const real=[];
 if(fs.existsSync(options.campaign)){
  const campaign=JSON.parse(fs.readFileSync(options.campaign,'utf8'));
  const cases=[{mission:16,kind:'heavy',start:[1440,3616],end:[944,3760],oldSeconds:25.60,via:[[1392,3632],[1296,3664],[1200,3696],[1104,3728],[1008,3760],[944,3760]]},{mission:2,kind:'grunt',start:[1472,2432],end:[848,1936],oldSeconds:22.05,via:[[1392,2352],[1296,2288],[1200,2224],[1104,2128],[1008,2064],[912,2000],[848,1936]]}];
  for(const test of cases){const m=copy(campaign.missions.find(m=>m.number===test.mission));assert(m,`Missing private mission ${test.mission}`);m.spawns=[{type:test.kind,team:0,x:test.start[0],y:test.start[1],squad:1},{type:'gatling',team:1,x:32,y:32,squad:2}];m.sectors.forEach(s=>s.owner=-1);m.buildings.forEach(b=>{b.team=-1;b.product=null;});m.crates=[];m.originalObjects=[];
   const automatic=walk(m,[test.end]),manual=walk(m,test.via);assert(automatic.seconds<=manual.seconds+.25,`Mission ${test.mission}: one command ${automatic.seconds}s is slower than manual shortcuts ${manual.seconds}s`);assert(automatic.seconds<test.oldSeconds*.9,`Mission ${test.mission}: original excessive road detour was retained`);real.push({mission:test.mission,kind:test.kind,automatic,manual,oldSeconds:test.oldSeconds});
  }
 }else console.log('Private campaign unavailable: real-campaign regressions skipped; synthetic navigation still checked.');
 console.log(JSON.stringify({flat,plain,road,crossing,real}));
 console.log('Route quality passed: exact endpoints, oblique open-field shortcut, useful road, actual displaced bridge crossing, blocked coastal corner and available original-campaign shortcut regressions.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
