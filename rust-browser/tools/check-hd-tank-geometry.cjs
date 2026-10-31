'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/hd-units.js'),'utf8'),types=['light','medium','heavy'];
const unit=(type,team,d=0,aim=0)=>({id:1,type,team,x:80,y:90,hp:300,maxHp:300,angle:d*Math.PI/4,aim:aim*Math.PI/4,moving:false,cooldown:0,order:'guard',crew:team<0?null:{kind:'grunt',hp:30}});
const destination={x:42,y:53,w:72,h:66};
let reads=0,gradients=[];
function syntheticCanvas(){const image={width:0,height:0};const methods={createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),createLinearGradient:(...coords)=>{const item={coords,colors:[]};gradients.push(item);return{addColorStop(_offset,color){item.colors.push(color);}};},createRadialGradient:()=>({addColorStop(){}}),getImageData(){reads++;const data=new Uint8ClampedArray(image.width*image.height*4);for(let y=60;y<160;y++)for(let x=50;x<140;x++)data[(y*image.width+x)*4+3]=255;return{data};}};const context=new Proxy(methods,{get:(target,key)=>key in target?target[key]:(()=>{}),set:(target,key,value)=>(target[key]=value,true)});image.getContext=()=>context;return image;}
const fakeEnv={window:{},performance:{now:()=>0},document:{createElement:syntheticCanvas}};vm.runInNewContext(source,fakeEnv);const fake=fakeEnv.window.HDUnits;let calls=[];const ctx={drawImage:(...args)=>calls.push(args),save(){},restore(){}};
for(const type of types)for(const team of [-1,0,1]){
 const u=unit(type,team),before=reads;calls=[];gradients=[];assert(fake.unit(ctx,u,0,{},destination));assert.equal(reads,before+1,'Measure alpha once on a newly prepared tank pose.');
 const radius={light:9,medium:10.5,heavy:13}[type],turretX=team<0?47:49,turret=gradients.find(g=>JSON.stringify(g.coords)===JSON.stringify([turretX-radius,54-radius,turretX+radius,54+radius]));assert(turret,'Find the actual upper turret shell gradient, not an unrelated faction stripe.');
 assert.deepEqual(turret.colors,team<0?['#bfbfbf','#777777','#333b3a']:team===0?['#ca7b64','#a84338','#452527']:['#85a3ad','#46708b','#243640'],'Only occupied tanks paint the full turret in the owning faction colors.');
 assert.deepEqual(calls[0].slice(1),[46,56,98,108,42,53,72,66],'Tight source crop has two logical pixels of edge padding; destination remains native bounds.');
 const image=calls[0][0],rasters=fake.stats.rasters;calls=[];assert(fake.unit(ctx,u,0,{},destination));assert.equal(reads,before+1,'Cached tank draws must never read pixels again.');assert.deepEqual(calls[0].slice(1),[46,56,98,108,42,53,72,66]);
 if(team<0)for(let d=0;d<8;d++)for(let aim=0;aim<8;aim++){calls=[];assert(fake.unit(ctx,unit(type,team,d,aim),0,{},destination));assert.strictEqual(calls[0][0],image,'Every stale neutral hull/turret heading shares the same parked raster.');assert.equal(fake.stats.rasters,rasters,'Changing an empty tank heading must not create extra rasters.');assert.equal(reads,before+1);}
}
const beforeOther=reads;fake.unit(ctx,unit('jeep',0),0,{},destination);fake.unit(ctx,unit('grunt',0),0,{},destination);assert.equal(reads,beforeOther,'Other HD artwork keeps its existing crop and preparation cost.');
console.log('PASS: HD tank alpha-crop preparation, exact native destination, cached crops and unchanged non-tank raster work.');
// A real raster check is optional in portable source checkouts. Set
// CANVAS_MODULE to the installed @napi-rs/canvas directory to run it anywhere.
let Canvas;try{Canvas=require(process.env.CANVAS_MODULE||'@napi-rs/canvas');}catch{console.log('SKIP: optional real HD raster geometry (@napi-rs/canvas not installed).');}
if(Canvas){
 const env={window:{},performance:{now:()=>0},document:{createElement:()=>Canvas.createCanvas(1,1)}};vm.runInNewContext(source,env);const art=env.window.HDUnits,native={x:48,y:54,w:80,h:72};let cases=0,minWidth=1,minHeight=1;
 for(const type of types)for(const team of [-1,0,1])for(let d=0;d<8;d++)for(let aim=0;aim<8;aim++){
  const canvas=Canvas.createCanvas(192,192),c=canvas.getContext('2d');assert(art.unit(c,unit(type,team,d,aim),0,{},native));const pixels=c.getImageData(0,0,192,192).data;let l=192,t=192,r=-1,b=-1,red=0,blue=0;
  for(let y=0;y<192;y++)for(let x=0;x<192;x++){const at=(y*192+x)*4;if(pixels[at+3]>16){l=Math.min(l,x);r=Math.max(r,x);t=Math.min(t,y);b=Math.max(b,y);if(pixels[at]>50&&pixels[at]>pixels[at+1]*1.3&&pixels[at]>pixels[at+2]*1.2)red++;if(pixels[at+2]>50&&pixels[at+2]>pixels[at]*1.3&&pixels[at+2]>pixels[at+1]*1.1)blue++;}}
  assert(l>=native.x&&t>=native.y&&r<native.x+native.w&&b<native.y+native.h,'The opaque silhouette stays inside its native visual bounds.');const width=(r-l+1)/native.w,height=(b-t+1)/native.h;
  assert(width>=.85&&height>=.85,`${type}/team${team}/body${d}/aim${aim}: HD body must fill native bounds rather than shrink inside blank sprite margins (${width.toFixed(2)},${height.toFixed(2)}).`);
  if(team===0)assert(red>=25,'A red occupied tank has a visible painted turret, not just isolated colored bolts.');else if(team===1)assert(blue>=25,'A blue occupied tank has a visible painted turret, not just isolated colored bolts.');else assert(red<100&&blue<100,'Empty tank armor stays gray; small colored tail lamps are material details.');
  minWidth=Math.min(minWidth,width);minHeight=Math.min(minHeight,height);cases++;
 }
 console.log(JSON.stringify({realRasterCases:cases,minWidthFill:minWidth,minHeightFill:minHeight}));console.log('PASS: actual HD tank rasters fill native bounds across independent hull/turret directions and neutral/owned teams.');
}
