// Real Rust/Wasm integration; synthetic fixtures require no copyrighted assets.
// If an owned campaign exists locally, additionally verify all imported maps.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');let wasm=path.join(root,'dist/independent/engine.wasm');
if(process.argv[2]){assert.equal(process.argv[2],'--wasm');assert(process.argv[3]);wasm=path.resolve(process.argv[3]);}
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(wasm),{}),e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();let bytes=0;
 function request(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}bytes=n;const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 const lookup={version:1,headTiles:Array.from({length:16},(_,i)=>100+i),interiorTiles:Array.from({length:16},(_,i)=>200+i),frontRemap:{115:300},frontStep:20,rubbleTiles:Array.from({length:8},(_,i)=>400+i),rubbleCode:22};
 const fragment=(id,x,y,hp=5,bit=0,record=0)=>({id,x,y,w:32,h:96,hp,maxHp:hp,wallRecord:record,wallBit:bit,height:2,nativeHp:hp,variant:3});
 function fixture({type='light',x=480,y=336,hp=5,walls}={}){const width=64,height=40,n=width*height;return{
  map:{width,height,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:1729,revision:0,original:true,originalTiles:Array(n).fill(17),terrainCodes:Array(n).fill(0),waterTile:0,wallTileLookup:lookup},
  sectors:[{id:0,x:1936,y:1168,owner:1,home:true,countsForProduction:false}],buildings:[],crates:[],originalObjects:[],walls:walls||[fragment(7,640,320,hp)],
  spawns:[{type,team:0,x,y,squad:1},{type:'gatling',team:1,x:1936,y:1168,squad:2}]};}
 const init=m=>request({op:'init',mission:0,difficulty:1,seed:1729,aiTeams:[],multiplayer:true,originalMission:m});
 const wall=(s,id=7)=>{const w=s.walls.find(w=>w.id===id);assert(w,`Missing wall ${id}`);return w;};
 const unit=s=>{const u=s.units.find(u=>u.team===0);assert(u,'Attacker disappeared');return u;};
 const command=(s,kind,extra={})=>request({op:'command',team:0,ids:[unit(s).id],kind,...extra});
 const tick=(steps=1,extra={})=>{let s;for(let i=0;i<steps;i++)s=request({op:'tick',dt:0.1,...extra});return s;};
 let s=init(fixture({hp:80})),j=10*s.map.width+20;
 assert.equal(s.map.tiles[j],6);assert.equal(s.map.terrainCodes[j],2);assert.equal(s.map.terrainCodes[j+s.map.width],18);assert.equal(s.map.originalTiles[j+s.map.width],320);
 assert.equal(s.catalog.find(d=>d.type==='light').canDestroyWalls,true);assert.equal(s.catalog.find(d=>d.type==='jeep').canDestroyWalls,false);assert.equal(s.catalog.find(d=>d.type==='laser').wallGrenades,true);
 const known={compact:true,knownMapSeed:s.mapSeed,knownMapRevision:s.mapRevision};
 s=command(s,'attackWall',{target:7});s=tick(1,known);assert(wall(s).hp<80&&wall(s).hp>0);assert(!s.map,'HP-only change should retain compact map omission');
 for(let i=0;i<160&&wall(s).hp>0;i++)s=tick(1,known);
 assert.equal(wall(s).hp,0);assert(s.map,'Collapse must transmit revised collision/artwork map');assert.equal(s.mapRevision,1);assert.equal(unit(s).order,'guard');
 for(let dy=0;dy<3;dy++){assert.equal(s.map.tiles[j+dy*s.map.width],0);assert.equal(s.map.terrainCodes[j+dy*s.map.width],22);assert.equal(s.map.originalTiles[j+dy*s.map.width],400+((7+dy)&7));}
 s=command(s,'move',{x:656,y:336});s=tick(80);assert(Math.hypot(unit(s).x-656,unit(s).y-336)<20,'Tank must be able to enter its opened breach');
 // HP=0 in the original header is still intact, then falls on its first hit.
 s=init(fixture({hp:0}));assert.equal(wall(s).hp,1);assert.equal(s.map.tiles[j],6);s=command(s,'attackWall',{target:7});s=tick();assert.equal(wall(s).hp,0);
 for(const type of ['grunt','psycho','sniper','pyro','laser','jeep','gatling','apc','crane']){s=init(fixture({type}));s=command(s,'attackWall',{target:7});s=tick(5);assert.equal(wall(s).hp,5,`${type} without explosives must not damage stone`);}
 // Acquire real grenade inventory through the existing pickup path.
 for(const type of ['grunt','psycho','sniper','pyro','laser']){const m=fixture({type,x:608,hp:80});m.crates=[{id:21,x:608,y:336,taken:false}];s=init(m);s=tick();assert.equal(unit(s).grenades,8);s=command(s,'attackWall',{target:7});s=tick();assert.equal(wall(s).hp,30);assert.equal(unit(s).grenades,7);assert(s.events.some(e=>e.weapon==='grenade'));}
 // Lower composed heads block fire at upper fragments, despite shared fronts.
 s=init(fixture({type:'cannon',x:656,y:464,walls:[fragment(7,640,320),fragment(9,640,352,5,32)]}));
 s=command(s,'attackWall',{target:7});s=tick();assert.equal(wall(s).hp,5);assert.equal(wall(s,9).hp,5);assert(!s.events.some(e=>e.kind==='shot'));
 s=command(s,'attackWall',{target:9});s=tick();assert.equal(wall(s).hp,5);assert.equal(wall(s,9).hp,0);
 // Distant orders find a reachable firing cell and stop outside wall geometry.
 s=init(fixture({x:96,walls:[fragment(7,1152,320)]}));s=command(s,'attackWall',{target:7});assert(unit(s).path.length>0);
 for(let i=0;i<700&&wall(s).hp>0;i++)s=tick();assert.equal(wall(s).hp,0);assert(unit(s).x<1152);
 // Immutable cliffs cannot be targeted away or shot through.
 const cliff=fixture();for(let y=0;y<cliff.map.height;y++)cliff.map.tiles[y*cliff.map.width+17]=2;
 s=init(cliff);s=command(s,'attackWall',{target:7});s=tick(20);assert.equal(wall(s).hp,5);assert.equal(unit(s).x,480);
 const owned=path.join(root,'dist/original/campaign.json');
 if(fs.existsSync(owned)){
  const campaign=JSON.parse(fs.readFileSync(owned,'utf8'));assert.equal(campaign.missions.length,20);let count=0,maxBytes=0;
  for(const m of campaign.missions){assert(Array.isArray(m.walls),'Regenerate owned campaign with wall metadata first');s=init(m);count+=s.walls.length;maxBytes=Math.max(maxBytes,bytes);assert(bytes<4*1024*1024,'Full state exceeds network snapshot limit');assert.deepEqual(s.map.originalTiles,m.map.originalTiles);assert.deepEqual(s.map.terrainCodes,m.map.terrainCodes);assert.deepEqual(s.map.tiles,m.map.tiles);assert.equal(s.walls.length,m.walls.length);}
  assert.equal(count,6038,'All original bitmap wall pieces must be imported');
  const largest=campaign.missions.reduce((a,b)=>a.map.width*a.map.height>b.map.width*b.map.height?a:b),m=structuredClone(largest),width=m.map.width,height=m.map.height,n=width*height;
  // Preserve actual map/wall geometry but isolate actors from campaign combat.
  const owners=Array(n).fill(0);for(const w of m.walls)for(const cell of w.cells)owners[cell]++;
  let candidate;for(const w of m.walls){for(const cell of w.cells){if(owners[cell]!==1)continue;const x=cell%width,y=Math.floor(cell/width);for(const [dx,dy]of[[-1,0],[1,0],[0,-1],[0,1]]){const nx=x+dx,ny=y+dy,j=ny*width+nx;if(nx>=0&&ny>=0&&nx<width&&ny<height&&[0,1].includes(m.map.tiles[j])&&[0,7,22].includes(m.map.terrainCodes[j])){candidate={wall:w,cell,from:[nx*32+16,ny*32+16],to:[x*32+16,y*32+16]};break;}}if(candidate)break;}if(candidate)break;}
  assert(candidate,'Largest original map must contain a reachable isolated wall face');
  m.buildings=[];m.crates=[];m.originalObjects=[];m.map.territory.fill(0);m.sectors=[{id:0,x:-1000,y:-1000,owner:-1,home:true,countsForProduction:false}];
  m.spawns=[{type:'light',team:0,x:candidate.from[0],y:candidate.from[1],squad:1},{type:'gatling',team:1,x:-1000,y:-1000,squad:2}];
  s=init(m);const id=candidate.wall.id;s=command(s,'attackWall',{target:id});for(let i=0;i<100&&wall(s,id).hp>0;i++)s=tick();
  assert.equal(wall(s,id).hp,0,'Actual largest-map wall must collapse on tank fire');assert.equal(s.map.tiles[candidate.cell],0);assert.equal(s.map.terrainCodes[candidate.cell],22);
  s=command(s,'move',{x:candidate.to[0],y:candidate.to[1]});s=tick(50);assert(Math.hypot(unit(s).x-candidate.to[0],unit(s).y-candidate.to[1])<20,'Original terrain breach must become reachable');
  console.log(JSON.stringify({ownedMaps:campaign.missions.length,originalWallPieces:count,maxFullSnapshotBytes:maxBytes,largestMap:[width,height],largestMapTankBreach:true}));
 }else console.log('Owned campaign absent: optional original-map checks skipped.');
 console.log('PASS: Wasm wall import, compact HP/revision updates, exact rubble/front forms, tank breach/pathing, weapon/grenade gates, overlapping-wall LOS and immutable cliffs.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
