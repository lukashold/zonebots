const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=require('node:path').resolve(__dirname,'..');
const metadata=JSON.parse(fs.readFileSync(root+'/dist/original/sprites.json','utf8'));
let requested=null;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(root+'/dist/independent/original-art.js','utf8'),env);
(async()=>{
 const art=env.window.OriginalArt;assert(await art.load());let calls=[];const ctx={drawImage:(im,x,y,w,h)=>calls.push({id:im.frameId,x,y,w,h})},map={original:true,world:0};
 const b={x:100,y:200,hp:100,team:0,product:'grunt',kind:'robot'};
 assert.equal(art.productionText(b,383),'6:23');assert.equal(art.productionText(b,0),'0:00');assert.equal(art.productionText(b,599.9),'9:59');assert.equal(art.productionText(b,600),'9:59');assert.equal(art.productionText(b,null),'*:**');
 const completed={...b,completionTicks:68};
 // Global phase increments every other native update: four visible ticks then four blank.
 for(let tick=0;tick<24;tick++)assert.equal(art.productionText(completed,383,tick/17),(((tick+1)>>1)&2)?'0:00':null);
 assert.equal(art.productionText({...completed,completionTicks:0},383,3/17),'6:23');
 assert.equal(art.productionText(completed,null,3/17),'0:00','Completion flash precedes later storage/rate stall');
 for(const change of [{team:-1},{hp:0},{product:null},{kind:'radar'}])assert.equal(art.productionText({...completed,...change},383,3/17),null);
 for(const value of [undefined,NaN,Infinity])assert.equal(art.productionText(b,value),null);
 for(const change of [{team:-1},{hp:0},{product:null},{kind:'radar'},{kind:'repair'}])assert.equal(art.productionText({...b,...change},383),null);
 assert(art.production(ctx,b,383,map));
 assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),[[586,170,316],[590,178,316],[582,184,316],[583,192,316]]);
 calls=[];art.production(ctx,{...b,kind:'vehicle'},0,map);assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),[[580,164,314],[590,172,314],[580,178,314],[580,186,314]]);
 calls=[];art.production(ctx,{...b,kind:'fort',rawBase:[50,100]},383,map);assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),[[586,248,380],[590,256,380],[582,262,380],[583,270,380]]);
 calls=[];art.production(ctx,b,null,map);assert.deepEqual(calls.map(c=>c.id),[574,590,574,574]);
 for(let world=0;world<5;world++)for(const kind of ['vehicle','robot','fort'])for(const seconds of [0,8,59,60,599,600,9999,null])assert(art.production(ctx,{...b,kind},seconds,{...map,world}));
 calls=[];assert(art.production(ctx,completed,383,map,3/17));assert.deepEqual(calls.map(c=>c.id),[580,590,580,580]);
 calls=[];assert.equal(art.production(ctx,completed,383,map,7/17),false);assert.equal(calls.length,0,'Blank phase draws no glyphs');
 assert.deepEqual(invalid,[]);console.log('PASS: original timer glyphs, native factory/fort anchors, per-character spacing, m:ss,9:59 cap,*:** stall, cancelled/destroyed/neutral states, original completion 0:00 blink and all world frame bounds.');
})().catch(e=>{console.error(e);process.exitCode=1;});
