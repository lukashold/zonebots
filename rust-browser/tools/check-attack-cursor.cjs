// Validate attack artwork against the separately extracted private sprite pack.
// No original pixels or campaign geometry are embedded in this test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
let requested=null,encoded=0;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});c.toDataURL=()=>{encoded++;return 'data:image/png;base64,'+Buffer.from(JSON.stringify({width:c.width,height:c.height,frame:c.frameId})).toString('base64');};return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
function decodeCursor(value){const match=value.match(/^url\("data:image\/png;base64,([^"]+)"\)/);assert(match);return JSON.parse(Buffer.from(match[1],'base64').toString());}
(async()=>{
 const art=env.window.OriginalArt;let calls=[];const ctx={drawImage:(im,x,y,w,h)=>calls.push({id:im.frameId,x,y,w,h})},map={original:true,world:0};
 assert.equal(art.attackCursor(map),'crosshair');assert.equal(art.attackMarker(ctx,100,200,map),false);assert.deepEqual(calls,[]);
 assert(await art.load());let markerCases=0;
 for(let world=0;world<5;world++){
  const current={original:true,world},cursor=art.attackCursor(current);
  assert.match(cursor,/\) 16 16, crosshair$/,'Scale2 cursor must retain the native8/8 hotspot');
  assert.deepEqual(decodeCursor(cursor),{width:32,height:30,frame:4});assert.equal(art.attackCursor(current),cursor);
  const hand=art.captureCursor(current),entry=art.entryCursor(current);
  assert.equal(decodeCursor(hand).frame,45);assert.equal(decodeCursor(entry).frame,6);
  assert.notEqual(cursor,hand);assert.notEqual(cursor,entry);assert.equal(art.attackCursor(current),cursor,'Independent cursor caches must not overwrite one another');
  for(const elapsed of [-1,0,.124,.125,.249,.25,.374,.375,.5,1,2]){
   calls=[];assert(art.attackMarker(ctx,123,456,current,elapsed));
   assert.deepEqual(calls,[{id:2783+(Math.max(0,Math.floor(elapsed*8))%4),x:107,y:440,w:32,h:32}]);markerCases++;
  }
  calls=[];assert(art.attackMarker(ctx,123.7,456.2,current,0));assert.deepEqual(calls,[{id:2783,x:108,y:440,w:32,h:32}]);
 }
 assert.equal(encoded,15,'Five world-specific attack, hand and entry cursors are each encoded once');
 for(const unavailable of [{original:false,world:0},undefined,null]){calls=[];assert.equal(art.attackCursor(unavailable),'crosshair');assert.equal(art.attackMarker(ctx,0,0,unavailable),false);assert.deepEqual(calls,[]);}
 assert.deepEqual(invalid,[],'Every original cursor and acknowledgement frame must exist');
 console.log(JSON.stringify({worlds:5,markerCases,attackCursorEncodings:5,otherCursorEncodings:10}));
 console.log('PASS: original attack crosshair4, native hotspot16/16, acknowledgement2783..2786, fixed scale2 size, rounded anchors, independent cursor caches and unavailable-art fallbacks.');
})().catch(error=>{console.error(error);process.exitCode=1;});
