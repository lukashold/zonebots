'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/original-art.js'),'utf8');
const indices=[0,16,...Array.from({length:23},(_,i)=>164+i)];
const baseline=Array.from({length:768},(_,i)=>(i*7+13)&255);
const fortIds=[[809,1675],[2800,2845],[2127,2148],[2613,2725],[2304,2431]];

async function load(resultPalette){
 const frames=[];
 for(const sides of fortIds)for(const base of sides)for(let i=0;i<10;i++)frames[base+i]={x:0,y:0,w:indices.length,h:1};
 const pixels=Uint8ClampedArray.from(indices.flatMap(i=>[i,i,i,i?255:0]));
 const manifest={frames,palettes:fortIds.map(()=>baseline),fortResultPalettes:fortIds.map(()=>resultPalette)};
 class Image{width=indices.length;height=1;set src(_){this.onload();}}
 const window={};
 const document={currentScript:{src:'http://test/independent/original-art.js'},createElement(){
  const canvas={};
  const context={
   drawImage(){},getImageData:()=>({data:pixels}),
   createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),
   putImageData(img){canvas.pixels=Array.from(img.data);},
  };
  canvas.getContext=()=>context;return canvas;
 }};
 vm.runInNewContext(source,{window,document,Image,URL,fetch:async()=>({ok:true,json:async()=>manifest})});
 assert.equal(await window.OriginalArt.load(),true);
 return window.OriginalArt;
}
function fort(art,row,world=0,dead=true){
 const draws=[];
 const ctx={drawImage(im){draws.push(im);}};
 assert(art.building(ctx,{kind:'fort',x:0,y:0,hp:dead?-1:1,team:1,orientation:world&1?'north':'south'},0,{original:true,world},row));
 assert.equal(draws.length,6);
 return draws;
}
const pixel=(im,index)=>im.pixels.slice(indices.indexOf(index)*4,indices.indexOf(index)*4+4);
const band=im=>indices.filter(i=>i>=165&&i<=185).flatMap(i=>pixel(im,i).slice(0,3));
(async()=>{
 const base=Array.from({length:63},(_,i)=>(i*9+47)&255),art=await load(base);
 for(let world=0;world<5;world++){
  const initial=fort(art,null,world),one=fort(art,1,world),full=fort(art,20,world),alive=fort(art,20,world,false);
  assert.deepEqual(band(initial[0]),base);
  assert.deepEqual(pixel(one[0],165),[76,0,0,255]);
  assert.deepEqual(pixel(one[0],166),[...base.slice(0,3),255]);
  assert.deepEqual(pixel(full[0],184),[76,0,0,255]);
  assert.deepEqual(pixel(full[0],185),[...base.slice(0,3),255]);
  for(const index of [0,16,164,186])assert.deepEqual(pixel(full[0],index),pixel(initial[0],index));
  assert.deepEqual(pixel(full[0],16),[...baseline.slice(32*3,33*3),255],'team coloring still applies');
  assert.deepEqual(band(alive[0]),baseline.slice(165*3,186*3));
  for(let i=1;i<6;i++){
   assert.strictEqual(one[i],initial[i],'the other damaged parts reuse the unmodified texture');
   assert.deepEqual(band(one[i]),baseline.slice(165*3,186*3));
  }
  assert.strictEqual(fort(art,20,world)[0],full[0],'repeated rows use the same texture');
 }
 if(process.env.NATIVE_FORT_PALETTE_TRACE){
  const entries=fs.readFileSync(process.env.NATIVE_FORT_PALETTE_TRACE,'utf8').trim().split(/\r?\n/).map(JSON.parse);
  const tables=entries.filter(x=>x.kind==='table');
  assert.equal(tables.length,4);
  for(const table of tables){
   const b=Array.from(Buffer.from(table.base,'hex')),original=Buffer.from(table.table,'hex'),candidate=await load(b);
   for(let row=0;row<=20;row++)assert.deepEqual(Buffer.from(band(fort(candidate,row)[0])),original.subarray(row*63,(row+1)*63));
  }
  console.log('Damaged fort sprites match all 84 original RGB table rows.');
 }
 console.log('Fort palette affects one damaged texture and preserves adjacent, intact and team-colored pixels.');
})().catch(error=>{console.error(error);process.exitCode=1;});
