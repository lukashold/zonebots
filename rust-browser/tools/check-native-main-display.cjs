//Numeric main-call boundary traces. Optional private CSV is emitted by
//ZNativeMainDisplayOracle; no original media or disassembly is embedded.
const assert=require('node:assert/strict'),fs=require('node:fs');
const mix=(h,n)=>Math.imul((h^(n>>>0))>>>0,16777619)>>>0;
function trace(s){const mask=s%16,variation=Math.floor(s/48),out=[0x4675c0,0x41d3b0,0x40a8f0,0x460a00,0x4339f0,0x40a2a0,0x4224f0,0x412de0,0x40ad60,0x43f7d0,0x412c10,0x43a310,0x434c60,0x4384f0,0x438410,0x46f040,0x421500,0x46ebb0,0x46ec70,0x44b8b0,0x45a240,0x456a90,0x43e890,0x4064f0,0x45a9f0,0x45ccb0,0x40b180,0x44b280];
 for(const[k,slot]of[0,7,31,239].entries())if(mask&(1<<k))out.push((slot*0x1000000+0x45a080)>>>0);
 out.push(0x419fb0,0x461d10,0x41d110,0x43e780,0x4563c0);
 for(let slot=0;slot<8;slot++)if((slot+s)%3-1>0)out.push((slot*0x1000000+0x43f260)>>>0);
 out.push(0x420be0,0x461970,0x464d90,0x41d6a0);
 for(let slot=0;slot<8;slot++)if((slot+s+variation)%3-1!==-1)out.push((slot*0x1000000+0x405820)>>>0);
 out.push(0x406f80,0x41f520,0x407af0,0x422310,0x4212c0,0x4549f0);return out;
}
let total=2166136261;const rows=process.argv[2]?fs.readFileSync(process.argv[2],'utf8').trim().split(/\r?\n/).map(r=>r.split(',')):null;
if(rows)assert.equal(rows.length,96);
for(let s=0;s<96;s++){const out=trace(s),hash=out.reduce(mix,2166136261),mode=[0,1,16][Math.floor(s/16)%3],mask=s%16;
 if(rows){assert.deepEqual(rows[s].slice(0,5).map(Number),[s,mode,mask,out.length,hash]);assert.deepEqual(rows[s][5].split(';').map(Number),out,`executed native Main scenario${s}`);}
 for(const value of[s,mode,mask,out.length,hash])total=mix(total,value);
}
assert.equal(total,2084901077,'96 executed435c12..435e21 traces;5a240 loop executes, callees are explicit boundaries');
const prefix=process.argv[3]?fs.readFileSync(process.argv[3],'utf8').trim().split(/\r?\n/).map(r=>r.split(',').map(Number)):null;let prefixHash=2166136261;
if(prefix)assert.equal(prefix.length,1024);
for(let s=0;s<1024;s++){const value=(s<10?[0,1,2,68,32767,-32768,-1,-2,-32767,65535][s]:s*997)<<16>>16,after=Math.max(0,(value-1)<<16>>16),row=[s,s%35,value,1,after,after];if(prefix)assert.deepEqual(prefix[s],row,`executed40ad60 BUILD callback${s}`);for(const n of row)prefixHash=mix(prefixHash,n);}
assert.equal(prefixHash,3044181778,'1024 actual40ad60 loops see decremented word36 at the BUILD callback');
console.log(`Native Main display:96 real-call boundary scenarios, signature${total};1024 BUILD-prefix cases signature${prefixHash}; BUILD/production before UNIT; flags after UNIT/AI; OBJECT X>0 versus ambient X!=-1.`);
