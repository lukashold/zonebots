'use strict';
// Static layer IDs/origins independently recovered from Z.exe in Ghidra:
// Light00455290, Medium00455c30, Heavy00455110. No original pixels embedded.
// The local private metadata supplies only actual frame sizes and palette RGB.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadataPath=path.join(root,'dist/original/sprites.json');
assert(fs.existsSync(metadataPath),'Extract the local original artwork first.');
const metadata=JSON.parse(fs.readFileSync(metadataPath,'utf8')),invalid=[];
let requested=null;
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
// Every synthetic atlas texel is opaque faction index16. The decoder still
// indexes the real frame rectangles and applies the real five world palettes.
const atlasPixels=new Proxy({},{get(_target,key){return Number(key)%4===3?255:16;}});
function canvas(){const image={width:0,height:0,frameId:requested};image.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(data){image.color=Array.from(data.data.slice(0,4));},drawImage(){},getImageData:()=>({data:atlasPixels})});return image;}
class Image{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;queueMicrotask(()=>this.onload());}}
const env={window:{},URL,Image,document:{currentScript:{src:'https://local.test/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
const fixtures={
 light:{neutral:5172,anchor:[-6,-6],hull:5719,damaged:5747,hullOffsets:[[-1,0],[-4,0],[-5,0],[-5,0]],turret:5731,turretOffsets:[[-9,1],[-9,0],[-9,0],[-9,0],[-9,3],[-10,2],[-9,2],[-10,2]],hatchOffsets:[[9,4],[8,3],[8,2],[8,2],[9,3],[9,2],[9,2],[10,3]]},
 medium:{neutral:5165,anchor:[-6,0],hull:488,damaged:500,hullOffsets:[[0,0],[-2,-7],[-2,0],[0,-7]],turret:513,turretOffsets:[[0,0],[-2,0],[-3,-3],[-2,-3],[0,-3],[0,-3],[0,-3],[-1,0]],barrel:521,barrelOffsets:[[7,-6],[20,0],[19,3],[13,10],[6,11],[-2,9],[-6,3],[-2,-4]],hatchOffsets:Array.from({length:8},()=>[11,-6])},
 // Heavy has no health-dependent alternate hull in00455110.
 heavy:{neutral:5168,anchor:[-6,-6],hull:5305,hullOffsets:[[0,0],[0,0],[0,0],[0,0]],turret:5329,turretOffsets:[[-7,0],[-6,0],[-2,2],[-6,1],[-7,2],[-8,1],[-12,2],[-8,0]],hatchOffsets:[[15,6],[13,9],[8,8],[8,5],[8,3],[9,0],[14,2],[15,3]]}
};
const angle=i=>i*Math.PI/4-Math.PI/2,plain=value=>JSON.parse(JSON.stringify(value));
const rounded=layers=>layers.map(({id,x,y})=>({id,x:Math.round(x),y:Math.round(y)}));
function union(draws){const x=Math.min(...draws.map(p=>p.x)),y=Math.min(...draws.map(p=>p.y));return{x,y,w:Math.max(...draws.map(p=>p.x+p.w))-x,h:Math.max(...draws.map(p=>p.y+p.h))-y};}
function expectedLayers(u,d,aim,phase){const f=fixtures[u.type],x=u.x+f.anchor[0]*2,y=u.y+f.anchor[1]*2,h=f.hullOffsets[d&3],t=f.turretOffsets[aim],tx=6+t[0],ty=-3+t[1],base=f.damaged&&u.hp<Math.floor(u.maxHp/4)?f.damaged:f.hull;
 const layers=[{id:base+(d&3)*3+phase,x:x+h[0]*2,y:y+h[1]*2},{id:f.turret+aim,x:x+tx*2,y:y+ty*2}];
 if(f.barrel)layers.push({id:f.barrel+aim,x:x+(tx+f.barrelOffsets[aim][0])*2,y:y+(ty+f.barrelOffsets[aim][1])*2});
 //00454ff0 sees initial hatch state00FF;00454f70 appends533+3*aim.
 layers.push({id:533+3*aim,x:x+f.hatchOffsets[aim][0]*2,y:y+f.hatchOffsets[aim][1]*2});return rounded(layers);
}
(async()=>{
 const art=env.window.OriginalArt;assert(await art.load());assert.equal(typeof art.vehicleLayers,'function','Expose the shared vehicle composition for regression checks.');let draws=[],owned=0,neutral=0;
 const ctx={drawImage(image,x,y,w,h){draws.push({id:image.frameId,x,y,w,h,color:image.color});}};
 // The two headings vary independently: the turret never inherits the hull's
 // four-way folding, and only hulls advance through their three track frames.
 for(let world=0;world<5;world++)for(const team of [0,1])for(const type of Object.keys(fixtures))for(let d=0;d<8;d++)for(let aim=0;aim<8;aim++)for(const sample of [{moving:false,time:.25,phase:0},{moving:true,time:0,phase:0},{moving:true,time:.125,phase:1},{moving:true,time:.25,phase:2}])for(const hp of [300,75,74]){
  const u={id:1,type,team,x:100.4,y:200.6,hp,maxHp:300,angle:angle(d),aim:angle(aim),moving:sample.moving,cooldown:0,order:'guard',crew:{kind:'grunt',hp:30}},map={original:true,world},before=JSON.stringify(u),label=`${type}/world${world}/team${team}/body${d}/aim${aim}/phase${sample.phase}/hp${hp}`;
  draws=[];assert(art.unit(ctx,u,sample.time,map),label);const expected=expectedLayers(u,d,aim,sample.phase);
  assert.deepEqual(draws.map(({id,x,y})=>({id,x,y})),expected,label);
  assert.deepEqual(rounded(plain(art.vehicleLayers(u,sample.time,map))),expected,`${label}: shared composition`);
  const bounds=union(draws);assert.deepEqual(plain(art.unitBounds(u,map,sample.time)),bounds,`${label}: selection/portrait bounds include every actual rounded layer`);assert.deepEqual(plain(art.attackBounds(u,map,sample.time)),bounds,`${label}: attack hitbox agrees with artwork`);
  const pal=metadata.palettes[world],index=team===1?32:16,color=[pal[index*3],pal[index*3+1],pal[index*3+2],255];for(const draw of draws)assert.deepEqual(draw.color,color,`${label}: actual world/team palette`);
  assert.equal(JSON.stringify(u),before,'Rendering must not mutate type, ownership, health or coordinates.');owned++;
 }
 // Empty hardware uses the original neutral object template, not a red body,
 // crew overlay or damage frame. It is static even when old headings remain.
 for(let world=0;world<5;world++)for(const type of Object.keys(fixtures))for(let d=0;d<8;d++)for(const hp of [300,74]){
  const u={id:2,type,team:-1,x:100.4,y:200.6,hp,maxHp:300,angle:angle(d),aim:angle(7-d),moving:false,cooldown:0,order:'guard',crew:null},map={original:true,world};
  draws=[];assert(art.unit(ctx,u,.25,map));assert.deepEqual(draws.map(({id,x,y})=>({id,x,y})),[{id:fixtures[type].neutral,x:100,y:201}]);
  assert.deepEqual(plain(art.unitBounds(u,map,.25)),union(draws));assert.deepEqual(plain(art.attackBounds(u,map,.25)),union(draws));
  const pal=metadata.palettes[world],color=[pal[80*3],pal[80*3+1],pal[80*3+2],255];for(const draw of draws)assert.deepEqual(draw.color,color,'Empty tank faction paint uses native palette4 grayscale.');neutral++;
 }
 assert.deepEqual(invalid,[],'Every requested original frame must exist.');
 console.log(JSON.stringify({occupiedCases:owned,neutralCases:neutral}));
 console.log('PASS: original Light/Medium/Heavy tank composition, independent hull/turret headings, tracks, damage gates, neutral templates, world/team palettes and complete rounded visual bounds.');
})().catch(error=>{console.error(error);process.exitCode=1;});
