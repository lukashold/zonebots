'use strict';
// Exercise the complete HD roster without loading private sprites or game assets.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const expected=['grunt','psycho','tough','sniper','pyro','laser','jeep','light','medium','heavy','apc','crane','gatling','cannon','missile','launcher','howitzer','truck','firetruck'];
let operations=0,allocated=0;
const numeric=(...values)=>{for(const value of values)if(typeof value==='number')assert(Number.isFinite(value),'Nonfinite Canvas argument');operations++;};
const gradient={addColorStop:numeric};
const context={createImageData(w,h){return{data:new Uint8ClampedArray(w*h*4)};},putImageData:numeric};
for(const method of ['beginPath','moveTo','lineTo','closePath','fill','stroke','ellipse','quadraticCurveTo','bezierCurveTo','save','restore','translate','scale','drawImage'])context[method]=numeric;
for(const method of ['createLinearGradient','createRadialGradient'])context[method]=(...values)=>{numeric(...values);return gradient;};
const sandbox={window:{},document:{createElement(tag){assert.equal(tag,'canvas');allocated++;return {getContext(type){assert.equal(type,'2d');return context;}};}}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/hd-units.js'),'utf8'),sandbox);
const art=sandbox.window.HDUnits;
assert.deepEqual(Array.from(art.types),expected,'The HD renderer covers the complete public unit roster');
let cases=0;
for(const type of expected)for(const team of [-1,0,1,3])for(let direction=0;direction<32;direction++){
 const u={id:direction,type,team,x:200,y:220,angle:direction*Math.PI/16,aim:(31-direction)*Math.PI/16,moving:true,hp:20,maxHp:100};
 assert.equal(art.unit(context,u,direction*.17,{}, {x:200,y:220,w:62,h:54}),true);cases++;
 assert(art.cacheSize<=art.cacheStats.maxEntries&&art.cacheStats.bytes<=art.cacheStats.maxBytes,'Changing pose must not grow the raster cache without bound');
}
art.clearCache();assert.equal(art.cacheSize,0);
const still={id:1,type:'jeep',team:0,x:0,y:0,angle:0,aim:Math.PI,moving:false,hp:100,maxHp:100};
art.unit(context,still,0);const afterFirst=allocated;
for(let i=0;i<60;i++)art.unit(context,still,i/60);
assert.equal(allocated,afterFirst,'Stationary art must reuse its high resolution raster across frames');
art.beginFrame(1);const beforeBurst=art.stats.rasters;for(let i=0;i<100;i++)art.unit(context,{...still,type:expected[i%expected.length],team:i%2,aim:i*Math.PI/8,moving:true},4);assert(art.stats.rasters-beforeBurst<=4,'A rendering frame may create at most four unit rasters');assert(art.stats.pending>0,'Unfinished pose work must be queued while units remain visible');
assert.equal(art.unit(context,{type:'unknown',x:0,y:0}),false);
assert.equal(art.unit(context,{type:'jeep',x:NaN,y:0}),false);
console.log(`HD units: ${cases} poses, all 19 types, finite Canvas geometry, cache bound and idle reuse passed (${operations} drawing operations).`);
