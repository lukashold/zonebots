// Verify the Ghidra-derived entry arrow against the user's private sprite pack.
// No original pixels or campaign geometry are embedded in this public test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
let requested=null,encoded=0;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});c.toDataURL=()=>{encoded++;return 'data:image/png;base64,'+Buffer.from(JSON.stringify({width:c.width,height:c.height,frame:c.frameId})).toString('base64');};return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
(async()=>{
 const art=env.window.OriginalArt;let calls=[];const ctx={drawImage:(im,x,y,w,h)=>calls.push({id:im.frameId,x,y,w,h})},map={original:true,world:0};
 assert.equal(art.entryCursor(map),'pointer');assert.equal(art.entryArrow(ctx,100,200,map),false);assert.deepEqual(calls,[]);
 assert(await art.load());let arrowCases=0,neutralBoundsCases=0,jeepBoundsCases=0;
 for(let world=0;world<5;world++){
  const current={original:true,world},cursor=art.entryCursor(current),match=cursor.match(/^url\("(data:image\/png;base64,[^"]+)"\) 14 22, pointer$/);
  assert(match,'Entry cursor must use the original native hotspot at scale2');
  const image=JSON.parse(Buffer.from(match[1].split(',')[1],'base64').toString());
  assert.deepEqual(image,{width:32,height:30,frame:6});assert.equal(art.entryCursor(current),cursor,'Repeated requests reuse their per-world cursor');
  const hand=art.captureCursor(current),handImage=JSON.parse(Buffer.from(hand.match(/base64,([^"]+)/)[1],'base64').toString());
  assert.equal(handImage.frame,45);assert.notEqual(hand,cursor);assert.equal(art.entryCursor(current),cursor,'Hand and entry cursor caches must not overwrite one another');
  for(const elapsed of [-1,0,.124,.125,.249,.25,.374,.375,.5,1,2]){
   calls=[];assert(art.entryArrow(ctx,123,456,current,elapsed));
   const id=2833+(Math.max(0,Math.floor(elapsed*8))%4);
   assert.deepEqual(calls,[{id,x:109,y:434,w:32,h:32}]);arrowCases++;
  }
  calls=[];assert(art.entryArrow(ctx,123.7,456.2,current,0));assert.deepEqual(calls,[{id:2833,x:110,y:434,w:32,h:32}]);
  for(const [type,id]of Object.entries(metadata.neutralUnits)){
   const unit={type,team:-1,x:123.7,y:456.2,angle:0,aim:0,hp:100,maxHp:100},f=metadata.frames[id];
   calls=[];assert(art.unit(ctx,unit,0,current));const box=art.unitBounds(unit,current,0);
   assert.deepEqual({...box},{x:124,y:456,w:f.w*2,h:f.h*2});assert.deepEqual(calls,[{id,...box}],'Neutral bounds must exactly match the native draw for every hardware type');
   const center={x:box.x+box.w/2,y:box.y+box.h/2};calls=[];assert(art.entryArrow(ctx,center.x,center.y,current));
   const arrow=calls[0];assert.equal(arrow.x+14,center.x);assert.equal(arrow.y+22,center.y);
   assert(arrow.x<box.x+box.w&&arrow.x+arrow.w>box.x&&arrow.y<box.y+box.h&&arrow.y+arrow.h>box.y,'Center-anchored entry feedback must overlap the rendered body');neutralBoundsCases++;
  }
  for(const angle of [0,Math.PI/2,Math.PI,Math.PI*1.5]){
   const unit={type:'jeep',team:0,x:123.7,y:456.2,angle,aim:angle,hp:100,maxHp:100,cooldown:0,moving:true};
   calls=[];assert(art.unit(ctx,unit,.125,current));const box=art.unitBounds(unit,current,.125),x=Math.min(...calls.map(p=>p.x)),y=Math.min(...calls.map(p=>p.y));
   assert.deepEqual({...box},{x,y,w:Math.max(...calls.map(p=>p.x+p.w))-x,h:Math.max(...calls.map(p=>p.y+p.h))-y},'Occupied Jeep bounds must use the rounded origins of its actual composite layers');jeepBoundsCases++;
  }
 }
 assert.equal(encoded,10,'Five world-specific entry and hand cursors are each encoded once');
 for(const unavailable of [{original:false,world:0},undefined]){calls=[];assert.equal(art.entryCursor(unavailable),'pointer');assert.equal(art.entryArrow(ctx,0,0,unavailable),false);assert.deepEqual(calls,[]);}
 assert.deepEqual(invalid,[],'Every cursor and acknowledgement frame must exist');
 console.log(JSON.stringify({worlds:5,arrowCases,neutralBoundsCases,jeepBoundsCases,entryCursorEncodings:5,handCursorEncodings:5}));
 console.log('PASS: original entry cursor6, native hotspot14/22, grayscale acknowledgement2833..2836, fixed draw size, all neutral hardware bounds and body-centered markers, rounded Jeep layers, independent cursor caches and unavailable-art fallbacks.');
})().catch(error=>{console.error(error);process.exitCode=1;});
