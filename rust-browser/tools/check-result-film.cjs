'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const {ReadableStream}=require('node:stream/web');
const zlib=require('node:zlib');
const window={},source=fs.readFileSync(path.join(__dirname,'../dist/independent/result-film.js'),'utf8');
vm.runInNewContext(source,{window,Uint8Array,Uint32Array,DataView,DecompressionStream});
const {open,load,timeline,paint,select,stepMs}=window.ResultFilm;
const pixels=320*200,header=Buffer.alloc(14);
header.write('ZVF1');header.writeUInt16LE(320,4);header.writeUInt16LE(200,6);
header.writeUInt16LE(4,8);header.writeUInt16LE(84,10);header[12]=2;
const palette=Buffer.alloc(768);palette.set([200,5,0],3);
const first=Buffer.alloc(pixels,1),unchanged=Buffer.alloc(pixels),delta=Buffer.alloc(pixels);
delta[2]=3;
const bytes=Buffer.concat([header,Buffer.from([1]),palette,first,Buffer.from([0]),unchanged,
 Buffer.from([0]),delta,Buffer.from([0]),unchanged]);
const copy=frame=>({frame:frame.frame,indices:Array.from(frame.pixels.slice(0,4)),color:Array.from(frame.palette.slice(3,6))});
(async()=>{
 const zip=zlib.gzipSync(bytes),fetcher=async()=>({ok:true,body:new ReadableStream({start(c){
  c.enqueue(zip.subarray(0,20));c.enqueue(zip.subarray(20));c.close();
 }})});
 const film=await load('local:fixture',fetcher);
 assert.deepEqual([film.width,film.height,film.frames,film.frameMs,stepMs],[320,200,4,84,40]);
 const runtime=timeline(film);
 assert.deepEqual(copy(runtime.tick(0)),{frame:0,indices:[1,1,1,1],color:[0,0,0]});
 assert.deepEqual(copy(runtime.tick(84)),{frame:1,indices:[1,1,1,1],color:[200,5,0]});
 assert.deepEqual(copy(runtime.tick(120)),{frame:1,indices:[1,1,1,1],color:[200,5,0]});
 assert.deepEqual(copy(runtime.tick(168)),{frame:2,indices:[1,1,2,1],color:[200,5,0]});
 const frame=runtime.tick(252);
 assert.deepEqual(copy(frame),{frame:3,indices:[1,1,2,1],color:[228,3,0]});
 assert.equal(runtime.tick(300),null);
 const image={data:new Uint8ClampedArray(pixels*4)};
 let submitted;paint({putImageData:data=>submitted=data},image,frame);
 assert.strictEqual(submitted,image);
 assert.deepEqual(Array.from(image.data.slice(0,12)),[228,3,0,255,228,3,0,255,0,0,0,255]);
 assert.throws(()=>open(bytes.subarray(0,bytes.length-1)),/Filmlänge/);
 assert.throws(()=>open(Buffer.concat([bytes,Buffer.alloc(1)])),/Filmlänge/);
 const noFrame=Buffer.from(bytes);noFrame[14]=2;
 assert.throws(()=>open(noFrame),/Filmbild/);
 assert.equal(select(0,true),'e_dwint1');assert.equal(select(0,false),'e_dlost1');
 assert.equal(select(3,true),'e_dwinpl');assert.equal(select(19,true),'e_cwint1');
 assert.equal(select(0,true,true),null);assert.equal(select(20,true),null);
 const privateWindow={location:{hostname:'game.example',search:'?original-films=1'}},requests=[];
 vm.runInNewContext(source,{window:privateWindow,document:{currentScript:{src:'https://game.example/independent/result-film.js'}},
  URL,URLSearchParams,fetch:async url=>{requests.push(url.href);return{ok:true,json:async()=>({version:1,films:[]})};}});
 assert.equal(requests.length,0,'a URL parameter alone cannot enable the hosted pack');
 privateWindow.ResultFilm.enablePrivatePack();privateWindow.ResultFilm.enablePrivatePack();
 assert.deepEqual(requests,['https://game.example/original/films/films.json']);
 if(process.env.NATIVE_FORT_RESULT_SELECTION){
  const rows=fs.readFileSync(process.env.NATIVE_FORT_RESULT_SELECTION,'utf8').trim().split(/\r?\n/).map(JSON.parse);
  assert.equal(rows.length,480);
  for(const row of rows)assert.equal(select(row.mission-1,row.local!==row.defeated-1,row.mp),row.movie||null,`Original result case ${row.id}`);
 }
 console.log('Result film: bounded gzip decoding, held image, delayed palette, signed fade and RGBA output.');
})().catch(error=>{console.error(error);process.exitCode=1;});
