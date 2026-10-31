// Real CPU raster timings with Skia Canvas, optional local development dependency.
// node tools/check-hd-performance.cjs --canvas-module /path/to/@napi-rs/canvas
const {pathToFileURL,fileURLToPath}=require('node:url');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),{performance}=require('node:perf_hooks');
const args=process.argv.slice(2),argument=name=>{const index=args.indexOf(name);return index<0?undefined:args[index+1];};
let Canvas;try{Canvas=require(argument('--canvas-module')||'@napi-rs/canvas');}catch{throw Error('Install @napi-rs/canvas for this optional real-raster benchmark, or pass --canvas-module. Normal graphics regressions need no package.');}
(async()=>{
const directory=argument('--source')||path.join(__dirname,'../dist/independent');let allocations=0;
class LocalImage extends Canvas.Image{set src(url){const filename=fileURLToPath(url),override=argument('--desert-png');super.src=fs.readFileSync(override&&path.basename(filename)==='desert-ground-v2.png'?override:filename);}}
const sandbox={window:{},Image:LocalImage,URL,document:{currentScript:{src:pathToFileURL(path.resolve(directory,'hd-terrain.js')).href},createElement(tag){assert.equal(tag,'canvas');allocations++;return Canvas.createCanvas(1,1);}},performance,Math,Number,Uint8ClampedArray,Map,Set};
for(const name of ['hd-terrain.js','hd-units.js','hd-structures.js'])vm.runInNewContext(fs.readFileSync(path.join(directory,name),'utf8'),sandbox);
const {HDTerrain,HDUnits}=sandbox.window,loadingAt=performance.now();const texturesReady=await HDTerrain.load?.(),assetDecodeMs=+(performance.now()-loadingAt).toFixed(2);const ctx=Canvas.createCanvas(1280,720).getContext('2d');
const map={width:80,height:80,world:0,seed:409,revision:0,original:true,tiles:Array(6400).fill(0),terrainCodes:Array(6400).fill(0),bridgeBase:Array(6400).fill(-1)};
for(let y=0;y<80;y++)for(let x=0;x<80;x++){const i=y*80+x;if(y>12&&y<27){map.tiles[i]=3;map.terrainCodes[i]=38;}if(x>7&&x<11){map.tiles[i]=1;map.terrainCodes[i]=7;}if(x>60){map.tiles[i]=2;map.terrainCodes[i]=19;}}
const terrain=[],units=[];
for(let frame=0;frame<80;frame++){
 const at=performance.now();HDUnits.beginFrame?.(frame);ctx.clearRect(0,0,1280,720);const view={x:frame<14?frame*70:980,y:160,w:1280,h:720};ctx.save();ctx.translate(-view.x,-view.y);HDTerrain.terrain(ctx,map,frame*.05,view);ctx.restore();const terrainDone=performance.now();
 for(let i=0;i<120;i++){const u={id:i,type:HDUnits.types[i%19],team:i%2,x:40+i%20*60,y:40+Math.floor(i/20)*100,angle:Math.floor(i%8)*Math.PI/4,aim:((i*17+Math.floor(frame/12))%16)*Math.PI/8,moving:true,hp:100,maxHp:100};HDUnits.unit(ctx,u,frame*.05,map,{x:u.x,y:u.y,w:48,h:50});}
 const done=performance.now();terrain.push(terrainDone-at);units.push(done-terrainDone);
}
function describe(values){const a=values.slice().sort((x,y)=>x-y);return{p50:+a[Math.floor(a.length*.5)].toFixed(2),p95:+a[Math.floor(a.length*.95)].toFixed(2),max:+a[a.length-1].toFixed(2),total:+a.reduce((n,x)=>n+x,0).toFixed(2)};}
if(argument('--image'))fs.writeFileSync(argument('--image'),ctx.canvas.toBuffer('image/png'));
const result={renderer:directory,assetDecodeMs,texturesReady,backend:'Skia Canvas CPU, 1280x720, synthetic80x80 map and120 moving units, 80 frames',terrainColdMs:describe(terrain.slice(0,14)),terrainWarmMs:describe(terrain.slice(24)),unitsMs:describe(units),allocations,terrainStats:HDTerrain.stats,unitStats:HDUnits.stats};
if(argument('--report'))fs.writeFileSync(argument('--report'),JSON.stringify(result,null,2));
console.log(JSON.stringify(result,null,2));

})().catch(error=>{console.error(error);process.exitCode=1;});
