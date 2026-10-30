// Verify pickup hit rectangles against locally extracted artwork metadata.
// Original pixels and campaign geometry are not embedded in this test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
let requested=null;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
(async()=>{
 const art=env.window.OriginalArt,box={x:123,y:456,taken:false},map={original:true,world:0};let calls=[];
 const ctx={drawImage:(im,x,y,w,h)=>calls.push({frame:im.frameId,x,y,w,h})};
 assert.equal(art.crateBounds(box,map),null,'Missing resources cannot provide a target');
 assert.equal(art.crate(ctx,box,map),false);
 assert(await art.load());let drawCases=0;
 const source=metadata.objectTypes[10],frame=metadata.frames[source.sprite];assert.equal(source.sprite,351);
 for(let world=0;world<5;world++)for(const [x,y]of [[0,0],[123,456],[123.6,456.4],[-22.5,91.5]]){
  const target={x,y,taken:false},current={original:true,world},bounds=art.crateBounds(target,current);
  assert(bounds);assert.deepEqual({...bounds},{x:Math.round(x),y:Math.round(y),w:frame.w*2,h:frame.h*2});
  calls=[];assert(art.crate(ctx,target,current));assert.deepEqual(calls,[{frame:source.sprite,...bounds}]);drawCases++;
  target.taken=true;assert.equal(art.crateBounds(target,current),null);calls=[];assert.equal(art.crate(ctx,target,current),false);assert.deepEqual(calls,[]);
 }
 for(const current of [{original:false,world:0},undefined])assert.equal(art.crateBounds(box,current),null);
 for(const bad of [null,undefined,{x:NaN,y:1},{x:1,y:Infinity}])assert.equal(art.crateBounds(bad,map),null);
 assert.deepEqual(invalid,[]);
 console.log(JSON.stringify({worlds:5,drawCases}));
 console.log('PASS: grenade crate hit bounds match original sprite351, scale2 dimensions and rounded native origin; taken/unavailable/invalid targets have no bounds.');
})().catch(error=>{console.error(error);process.exitCode=1;});
