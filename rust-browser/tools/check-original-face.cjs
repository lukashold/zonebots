'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const env={window:{}};vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/original-face.js'),'utf8'),env);
const face=env.window.OriginalFace,layout={headStarts:[137,177,217,257,297],backgroundStart:131};let hash=2166136261,count=0;
const value=n=>{hash=Math.imul(hash^(n>>>0),16777619)>>>0;};
for(let head=0;head<5;head++)for(let world=0;world<5;world++)for(let vehicle=0;vehicle<2;vehicle++)for(let seed=0;seed<256;seed++){
 const word=(seed*40503+17)&65535,decoded=face.decode(word,layout.headStarts[head]),draws=face.layers(word,head,world,vehicle,layout,[12,20]);
 [head,world,vehicle,seed,word,...decoded,draws.length,...draws.flatMap(d=>[d.sprite,d.x,d.y])].forEach(value);count++;
}
assert.equal(count,12800);assert.equal(hash,2076380861,'ZNativePortraitOracle: complete68c80→68d30 sprite/position traces, GPU texture submission isolated');
console.log('PASS:12800 native portrait layer traces, five distinct head sets, five native worlds, hardware background, packed face poses and every selected sprite/offset.');

let phraseHash=2166136261,phraseCount=0;
for(let scenario=0;scenario<256;scenario++){const words=Array.from({length:scenario%181},(_,f)=>(f*40503+scenario*17)&65535);for(let tick=0;tick<64;tick++){for(const n of [scenario,tick,face.phraseWord(words,tick)])phraseHash=Math.imul(phraseHash^(n>>>0),16777619)>>>0;phraseCount++;}}
assert.equal(phraseCount,16384);assert.equal(phraseHash,2326867798,'ZNativePortraitClockOracle:46ad70/46ad10 precedes72/17 frame-index update');
console.log('PASS:16384 original speech-portrait clock callbacks and phrase termination.');
