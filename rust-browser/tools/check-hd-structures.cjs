// Exercise scenery without browser pixels: finite draw geometry, visible combat
// kinds, taken-crate suppression and the large-fort cache's memory budget.
'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
let canvases=0,drawCalls=0,paintCalls=0;
const geometry=[];
const ctx=new Proxy({}, {
  set(target,key,value){if(typeof value==='number')assert(Number.isFinite(value),`${key} must be finite`);target[key]=value;return true;},
  get(target,key){if(key in target)return target[key];
    if(key==='createLinearGradient'||key==='createRadialGradient')return(...args)=>{args.forEach(x=>assert(Number.isFinite(x),`${key} coordinate`));paintCalls++;return{addColorStop(offset){assert(Number.isFinite(offset)&&offset>=0&&offset<=1);}};};
    if(key==='drawImage')return(image,...args)=>{args.forEach(x=>assert(Number.isFinite(x),'image position'));drawCalls++;};
    return(...args)=>{for(const x of args)if(typeof x==='number')assert(Number.isFinite(x),`${key} coordinate`);geometry.push([key,...args]);paintCalls++;};
  }
});
const window={},document={createElement(tag){assert.equal(tag,'canvas');canvases++;return{width:0,height:0,getContext(){return ctx;}};}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/hd-structures.js'),'utf8'),{window,document,Math,Number});
const art=window.HDStructures;
for(const kind of ['fort','robot','vehicle','radar','repair'])for(const team of [-1,0,1])for(const hp of [0,20,100]){
  assert.equal(art.building(ctx,{kind,team,hp,maxHp:100,x:32,y:64,w:kind==='fort'?320:128,h:kind==='fort'?352:kind==='radar'?96:160,stars:3,rawBase:kind==='fort'?[16,48]:null},.6,{world:0}),true);
}
for(let type=0;type<76;type++)art.object(ctx,{x:32,y:32,type},{world:0});
assert.equal(art.crate(ctx,{x:32,y:32,taken:true},{}),false);
assert.equal(art.crate(ctx,{x:32,y:32,taken:false},{}),true);
assert.equal(art.flag(ctx,{x:32,y:32,owner:1,id:1},1,{}),true);
for(const kind of ['shot','grenade','smoke','explosion']){
  const before=paintCalls;art.effects(ctx,[{kind,x:20,y:20,x2:200,y2:100,age:.12,life:.8,radius:22}],1,{});assert(paintCalls>before,`${kind} must remain visible`);
  for(const age of [undefined,NaN,Infinity,-1])for(const life of [undefined,NaN,Infinity,0,-1])art.effects(ctx,[{kind,x:20,y:20,age,life,radius:NaN,angle:Infinity}],1,{});
}
// Enough fort variants to exceed a count-only cache by over 4x the byte budget.
for(let i=0;i<40;i++){
  art.building(ctx,{kind:'fort',team:i%4,hp:100,maxHp:100,x:0,y:0,rawBase:[0,0],w:320,h:352,stars:i,orientation:i%2?'north':'south'},1,{});
  assert(art.cacheStats.bytes<=art.cacheStats.maxBytes,'cache memory budget');assert(art.cacheStats.entries<=art.cacheStats.maxEntries,'cache entry budget');
}
assert(art.cacheStats.bytes>0&&art.cacheStats.entries<40,'large forts must cause memory eviction');
const before=canvases;
art.building(ctx,{kind:'fort',team:3,hp:100,maxHp:100,x:0,y:0,rawBase:[0,0],w:320,h:352,stars:39,orientation:'north'},1,{});
assert.equal(canvases,before,'recently used fort stays cached');
assert(drawCalls>80,'scenery images drawn');
console.log(`HD scenery passed: all structures/objects/combat effects finite; ${art.cacheStats.entries} cached images, ${(art.cacheStats.bytes/1024/1024).toFixed(1)} MiB / 48 MiB.`);
