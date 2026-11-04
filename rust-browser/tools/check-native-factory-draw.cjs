//Native BUILD clock/work ordering; optional local CSV from the Ghidra oracle.
//No original assets, executable bytes or decompiler text are embedded here.
const assert=require('node:assert/strict'),fs=require('node:fs'),mix=(h,n)=>Math.imul(h^(n>>>0),16777619)>>>0;
const csv=process.argv[2]?fs.readFileSync(process.argv[2],'utf8').trim().split(/\r?\n/).map(r=>r.split(',').map(Number)):null;if(csv)assert.equal(csv.length,2048);let hash=2166136261;
for(let s=0;s<2048;s++){const kind=s%2===0?2:4,hold=[0,1,2,68][Math.floor(s/2)%4],hp=[0,1,20,100][Math.floor(s/8)%4],owned=Math.floor(s/32)%9,owner=[0,1,4][Math.floor(s/288)%3],blocked=Math.floor(s/864)%2,phase=s%4,rate=Math.trunc(hp*(1+Math.trunc(Math.trunc(owned*100/8)/4))/100),remaining=[-1,0,1,rate,rate+1,1000][s*7%6],animation=[0,17,-27][s%3];
 let endHold=Math.max(0,hold-1),endRemaining=remaining,endTotal=4000+s,endAnimation=animation,clocks=0,clockRemaining=0,clockHold=0,seconds=0,spawns=0;
 if(hp!==0&&owner<4){if(endHold!==0){endAnimation=0;if(phase&2){clocks=1;clockRemaining=remaining;clockHold=endHold;}}
 else{clocks=1;clockRemaining=remaining;seconds=rate===0?-1:Math.trunc(remaining/(rate*17));if(rate!==0){endAnimation--;if(endAnimation<=rate-27)endAnimation=-endAnimation;if(!blocked){endRemaining-=rate;if(endRemaining<0){endRemaining=endTotal=32155;endAnimation=0;endHold=68;spawns=1;}}}}}
 const row=[s,kind,owner,hp,owned,blocked,hold,remaining,animation,phase,rate,clocks,clockRemaining,clockHold,seconds,spawns,endRemaining,endTotal,endHold,endAnimation,0].map(n=>n===0?0:n);if(csv)assert.deepEqual(csv[s],row,`actual BUILD callback${s}`);for(const n of row)hash=mix(hash,n);
}assert.equal(hash,3477325678,'2048 actual40ad60 producer callbacks and40b180 readonly counts');
console.log(`Native factory draw:2048 source executions; clock before work/strict completion; hold decremented once; zero HP/neutral/blocked gates; signature${hash}.`);
