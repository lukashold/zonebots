'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
for(const prefix of ['', 'independent/']){
 const html=fs.readFileSync(path.join(__dirname,'../dist',prefix,'index.html'),'utf8');
 const assets=prefix?'':'independent/';
 const camera=html.indexOf(`<script src="${assets}fort-camera.js"></script>`);
 const app=html.indexOf(`<script src="${assets}app.js"></script>`);
 assert(camera>=0&&camera<app,`camera controller must load before app.js in ${prefix}index.html`);
}
const window={};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/fort-camera.js'),'utf8'),{window});
const create=window.FortCamera.create;
const positions=(camera,scene,point)=>{const value=camera.advance(scene,point);return value===null?null:Array.from(value);};

const first=create(),second=create(),scene={active:true,frame:2,panRequest:[368,96]};
assert.equal(positions(first,scene,[240,140]),null);
assert.equal(positions(second,scene,[920,840]),null);
assert.notDeepEqual(positions(first,{...scene,frame:4},[240,140]),positions(second,{...scene,frame:4},[920,840]));
assert.equal(positions(first,{...scene,frame:3},[0,0]),null,'a stale frame does not restart the pan');
for(const camera of [first,second])assert.deepEqual(positions(camera,{...scene,frame:18},[0,0]),[336,104]);
assert.equal(positions(first,{...scene,frame:20},[0,0]),null,'return control after the last step');
assert.equal(positions(first,{active:false},[0,0]),null);
assert.deepEqual(positions(first,{...scene,frame:3},[336,104]),[336,104],'start a new independent mission pan');

if(process.env.NATIVE_FORT_CAMERA_STEP_TRACE){
 const rows=fs.readFileSync(process.env.NATIVE_FORT_CAMERA_STEP_TRACE,'utf8').trim().split(/\r?\n/).map(JSON.parse);
 assert.equal(rows.length,128*18);
 let trajectories=0;
 for(let offset=0;offset<rows.length;offset+=18){
  const start=rows[offset],point=start.before.slice(0,2),panRequest=start.before.slice(2,4);
  if(panRequest[0]===-1)continue;
  trajectories++;
  for(const frames of [[...Array(16)].map((_,i)=>i+3),[4,7,12,18]]){
   const camera=create();
   for(const frame of frames)assert.deepEqual(positions(camera,{active:true,frame,panRequest},point),rows[offset+frame-3].after.slice(0,2),`original row ${start.id}/${frame}`);
  }
 }
 assert.ok(trajectories>0);
 console.log(`Fort camera: ${trajectories} original trajectories replayed with full and coalesced peer snapshots.`);
}
