// Validate original hand artwork and its frontend wiring using a private resource pack.
// No original artwork or campaign geometry is embedded in this public test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
let requested=null,encoded=0;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(image){c.firstColor=Array.from(image.data.slice(0,4));},drawImage(){},getImageData:()=>{const data=new Uint8ClampedArray(c.width*c.height*4);for(const f of metadata.frames){if(!f)continue;const at=(f.y*c.width+f.x)*4;data[at]=16;data[at+3]=255;}return {data};}});c.toDataURL=()=>{encoded++;return 'data:image/png;base64,'+Buffer.from(JSON.stringify({width:c.width,height:c.height,frame:c.frameId})).toString('base64');};return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
(async()=>{
 const art=env.window.OriginalArt;let calls=[],lastColor;const ctx={drawImage:(im,x,y,w,h)=>{lastColor=im.firstColor;calls.push({id:im.frameId,x,y,w,h});}};
 const sector={x:100,y:200,owner:0},map={original:true,world:0};
 assert.equal(art.captureCursor(map),'pointer');assert.equal(art.captureHand(ctx,100,200,map),false);assert.equal(art.flagBounds(sector,0,map),null);assert.equal(art.flagShadows(ctx,[sector],0,map),false);
 assert(await art.load());let handCases=0,flagCases=0,shadowCases=0;
 for(let world=0;world<5;world++){
  const current={original:true,world},cursor=art.captureCursor(current),match=cursor.match(/^url\("(data:image\/png;base64,[^"]+)"\) 18 14, pointer$/);
  assert(match,'Original capture cursor must supply its native scale2 hotspot');
  const image=JSON.parse(Buffer.from(match[1].split(',')[1],'base64').toString());
  assert.deepEqual(image,{width:32,height:28,frame:45});assert.equal(art.captureCursor(current),cursor,'Cursor is cached per world');
  for(const elapsed of [-1,0,.125,.25,.375,.5,1,2]){
   calls=[];assert(art.captureHand(ctx,123,456,current,elapsed));
   const frame=2755+(Math.max(0,Math.floor(elapsed*8))%4);
   assert.deepEqual(calls,[{id:frame,x:105,y:442,w:32,h:32}]);handCases++;
  }
  for(const owner of [-1,0,1])for(const time of [-1,0,1/6,2/6,3/6,4/6,1,1.25]){
   const s={...sector,owner};calls=[];assert(art.flag(ctx,s,time,current));
   //0041d110: four 16x16 map flags, at tile origin; large fort banners and
   // their shadows must never be substituted for neutral territory flags.
   const phase=Math.max(0,Math.floor(time*6))%4,id=480+phase,f=metadata.frames[id];
   const box=art.flagBounds(s,time,current);assert.deepEqual({...box},{x:84,y:184,w:f.w*2,h:f.h*2});
   assert.deepEqual(calls,[{id,...box}]);
   const palette=metadata.palettes[world],index=owner<0?80:owner===1?32:16;
   assert.deepEqual(lastColor,[palette[index*3],palette[index*3+1],palette[index*3+2],255],'Synthetic cloth pixel must follow the original neutral/team palette mapping');flagCases++;
   calls=[];assert(art.flagShadows(ctx,[s],time,current));
   const shadow=metadata.frames[484+phase];assert.deepEqual(calls,[{id:484+phase,x:84,y:184,w:shadow.w*2,h:shadow.h*2}]);shadowCases++;
  }
  const fractional={x:100.6,y:200.4,owner:-1};calls=[];assert(art.flag(ctx,fractional,0,current));
  assert.deepEqual({...art.flagBounds(fractional,0,current)},{x:85,y:184,w:32,h:32});assert.deepEqual(calls,[{id:480,x:85,y:184,w:32,h:32}]);
  calls=[];assert(art.flagShadows(ctx,[sector,{x:300,y:400,owner:1}],0,current));assert.deepEqual(calls,[{id:484,x:84,y:184,w:32,h:32},{id:484,x:284,y:384,w:32,h:32}]);
 }
 assert.equal(encoded,5,'Repeated cursor requests must reuse their cached data URL');
 assert.equal(art.captureCursor({original:false,world:0}),'pointer');assert.equal(art.captureHand(ctx,0,0,{original:false}),false);assert.equal(art.flagBounds(sector,0,{original:false}),null);assert.equal(art.flagShadows(ctx,[sector],0,{original:false}),false);
 assert.deepEqual(invalid,[],'All hand and flag frames must exist in the atlas');
 for(const relative of ['dist/index.html','dist/independent/index.html']){
  const html=fs.readFileSync(path.join(root,relative),'utf8'),module=html.indexOf('flag-command.js'),app=html.indexOf('app.js');
  assert(module>=0&&app>module,`${relative}: flag intent module must load before app`);
 }
 const app=fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8');
 assert(app.includes('function context(p){if(captureAt(p))return;'),'Modern context command must test flag intent before building/unit context');
 const up=app.slice(app.indexOf('canvas.onpointerup='),app.indexOf('canvas.onpointercancel='));
 assert(up.indexOf('captureAt(p)')>=0&&up.indexOf('captureAt(p)')<up.indexOf('state.units.find'),'Classic click must test explicit flag intent before ordinary selection');
 console.log(JSON.stringify({worlds:5,handCases,flagCases,shadowCases,cursorEncodings:encoded}));
 console.log('PASS: original hover/grasp hands, native hotspot and fixed size, four-frame map flags and ground shadows, neutral grayscale, matching hit rectangles, cursor cache and classic/modern entrypoint wiring.');
})().catch(error=>{console.error(error);process.exitCode=1;});
