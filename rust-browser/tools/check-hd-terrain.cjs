// Renderer regressions that matter across Rust JSON snapshots and live bridges.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
let allocations=0,draws=0,finiteCalls=0,projection=1;
const fillStyles=[];
function finite(args,operation){
  for(const value of args)if(typeof value==='number')assert(Number.isFinite(value),`${operation} received ${value}`);
  finiteCalls++;
}
function context(){
  const state={createImageData(w,h){return{data:new Uint8ClampedArray(w*h*4)};},createPattern(){return {};},putImageData(){},fillStyle:'',getTransform(){return{a:projection,b:0};},drawImage(...args){finite(args,'drawImage');draws++;},
    createLinearGradient(...args){finite(args,'linearGradient');return{addColorStop(...stops){finite(stops,'colorStop');}};},
    createRadialGradient(...args){finite(args,'radialGradient');return{addColorStop(...stops){finite(stops,'colorStop');}};},
    fillRect(...args){finite(args,'fillRect');fillStyles.push(this.fillStyle);}};
  return new Proxy(state,{get(target,key){if(key in target)return target[key];return(...args)=>finite(args,String(key));},set(target,key,value){target[key]=value;return true;}});
}
const sandbox={window:{},document:{createElement(){allocations++;return{width:0,height:0,getContext:context};}}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/hd-terrain.js'),'utf8'),sandbox);
const art=sandbox.window.HDTerrain,ctx=context(),copy=value=>JSON.parse(JSON.stringify(value));
function fixture(){
  const n=64*64,map={width:64,height:64,world:0,seed:912,revision:0,original:true,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),bridgeBase:Array(n).fill(-1)};
  for(let y=0;y<64;y++){
    map.tiles[y*64+8]=3;map.terrainCodes[y*64+8]=38;
    map.tiles[y*64+12]=2;map.terrainCodes[y*64+12]=19;
    for(let x=0;x<64;x++)if(y>=4&&y<=6){map.tiles[y*64+x]=1;map.terrainCodes[y*64+x]=7;}
  }
  const bridge=5*64+8;map.tiles[bridge]=5;map.terrainCodes[bridge]=23;map.bridgeBase[bridge]=0;
  return map;
}
const map=fixture(),view={x:.375,y:.625,w:511,h:511},initial=JSON.stringify(map);
assert.equal(art.terrain(ctx,map,0,view),true);assert.equal(JSON.stringify(map),initial,'Drawing must not change physics');
const firstBuilds=art.stats.chunksBuilt;assert.equal(firstBuilds,1,'Opening a viewport must build one chunk, not the whole viewport');for(let frame=1;frame<8;frame++)art.terrain(ctx,map,frame/60,view);
const warm=allocations;art.terrain(ctx,copy(map),1,view);
assert.equal(allocations,warm,'Fresh Rust JSON snapshots must reuse existing terrain rasters');
art.minimap(ctx,map);const miniWarm=allocations;art.minimap(ctx,copy(map));
assert.equal(allocations,miniWarm,'Minimap must reuse its overview across engine snapshots');
assert(fillStyles.includes('#142d3444'),'Fixture should contain an intact rendered bridge');
fillStyles.length=0;map.tiles[5*64+8]=3;map.revision++;for(let frame=0;frame<8;frame++)art.terrain(ctx,map,2+frame/60,view);
assert(allocations>miniWarm,'A geometry revision must refresh visible terrain');
assert(!fillStyles.includes('#142d3444'),'Destroyed bridge must remain water despite its stale native bridge code');
for(let world=0;world<5;world++){
  map.world=world;map.seed=1000+world;art.terrain(ctx,map,world,view);art.minimap(ctx,map);
}
// Exercise enough distinct visible areas to exceed the raster memory budget.
for(let i=0;i<7;i++){
  map.seed=2000+i;projection=i%2?.5:1;
  art.terrain(ctx,map,i,{x:0,y:0,w:1023,h:1023});
  assert(art.cacheBytes<=64*1024*1024,'Terrain raster cache exceeds64MiB');
}
assert(art.cacheSize>0);assert(draws>0);assert(finiteCalls>1000);
art.clear();assert.equal(art.cacheBytes,0);assert.equal(art.cacheSize,0);
console.log(`HD terrain passed: cloned snapshots, cached minimap, live bridge revision, nonmutation, five biomes, finite Canvas calls and bounded raster memory (${draws} draws).`);

async function assetLoading(){
 const images=[],events=[],document={currentScript:{src:'https://example.test/independent/hd-terrain.js'},createElement(){return{width:0,height:0,getContext:context};}};
 class Image{set src(value){this.url=String(value);images.push(this);}}
 const window={dispatchEvent(event){events.push(event.type);}};
 vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/hd-terrain.js'),'utf8'),{window,document,Image,URL,Event:class{constructor(type){this.type=type;}}});
 const renderer=window.HDTerrain,promise=renderer.load();assert.equal(renderer.load(),promise,'asset loads are idempotent');assert.equal(images.length,20,'five worlds each load four baked materials');
 assert(images.some(image=>image.url.endsWith('/hd-materials/desert-ground-v2.png')),'desert uses the new detailed generated ground');
 renderer.terrain(ctx,fixture(),0,view);const before=renderer.stats.chunksBuilt;
 const desert=images.find(image=>image.url.endsWith('/desert-ground-v2.png'));desert.onerror();assert(desert.url.endsWith('/0-ground.webp'),'a failed generated image has a lightweight material fallback');
 for(const image of new Set(images)){image.width=image.height=256;image.onload();}
 assert.equal(await promise,true);assert.equal(events.length,5,'late material availability notifies menu previews');
 renderer.terrain(ctx,fixture(),1,view);assert(renderer.stats.chunksBuilt>before,'late decoded textures replace fallback chunks without changing map revisions');
 console.log('HD texture loading passed: asynchronous20material preload, idempotence, generated-desert fallback, menu notification, late texture revision.');
}
assetLoading().catch(error=>{console.error(error);process.exitCode=1;});
