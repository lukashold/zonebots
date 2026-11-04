const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm'),path=require('node:path');
const palettes=new WeakMap(),window={OriginalArt:{ready:true,nativePalette:c=>palettes.get(c)??0,selectNativePalette:(c,p)=>palettes.set(c,p)},GameGraphics:{isHD:()=>false}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/original-display.js'),'utf8'),{window});
const display=window.OriginalDisplay,mix=(h,n)=>Math.imul((h^(n>>>0))>>>0,16777619)>>>0;
let all=2166136261;
for(let s=0;s<512;s++){
 const count=s===0?0:1+s%17,mode=s%3,forced=Math.floor(s/3)%2!==0;
 let current=[0,4,255][Math.floor(s/6)%3],uploaded=[0,1,4,255][Math.floor(s/18)%4],trace=2166136261,uploads=0,draws=0;
 const records=Array.from({length:count},(_,i)=>{const y=s%4===0?i%4:s%4===1?count-i:s%4===2?(i*7313+s*37)<<16>>16:-100,p=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,255][(s+i*7)%17];let offset=[-128,-1,0,16,127][(s+i)%5];if(p===255&&offset===-1)offset=0;return{x:100+i,y,sprite:1000+i,p,offset};});
 display.sort(records,r=>r.y);let bytes=2166136261;for(const r of records){const b=Buffer.alloc(8);b.writeInt16LE(r.x,0);b.writeInt16LE(r.y,2);b.writeInt16LE(r.sprite,4);b.writeUInt8(r.p,6);b.writeInt8(r.offset,7);for(const n of b)bytes=mix(bytes,n);const next=display.palette(r.p,current===4?-1:current,forced);current=next<0?4:next;if(current!==uploaded){uploaded=current;uploads++;trace=mix(mix(trace,0),current);}draws++;for(const n of[mode===0?1:2,r.x,(r.y+r.offset)<<16>>16,r.sprite,current])trace=mix(trace,n);}
 for(const n of[s,count,bytes,current,uploaded,trace,uploads,draws])all=mix(all,n);
}
assert.equal(all,131070041,'512 actual545b0/4549f0 palette/submission gold traces');
const ctx={},calls=[],state={map:{original:true},units:[{id:7,team:1,x:100,y:200}],buildings:[{id:9}],nativeDisplayQueue:[{stage:'pre',kind:'building',id:9},{stage:'queue',kind:'unit',id:7,palette:4,sprite:127,x:100,y:200},{stage:'queue',kind:'sprite',id:3,palette:255,sprite:352,x:20,y:30},{stage:'queue',kind:'sprite',id:4,palette:8,sprite:353,x:20,y:30},{stage:'queue',kind:'effect',draw:{sprite:354,nativeInheritedPalette:true}}]};
assert(display.draw(ctx,state,{building:b=>{calls.push(['building',b.id]);window.OriginalArt.selectNativePalette(ctx,1);},objectShadows:()=>calls.push(['shadows']),unit:(u,sprite)=>{calls.push(['unit',u.id,u.team,u.nativeDrawPalette,sprite.team]);},sprite:f=>calls.push(['sprite',f.sprite,f.team??window.OriginalArt.nativePalette(ctx)])}));
assert.deepEqual(calls,[['building',9],['shadows'],['unit',7,1,255,255],['sprite',352,255],['sprite',353,-1],['sprite',354,-1]]);
assert.equal(state.units[0].team,1,'palette changes must never change occupied/neutral identity');
const saved={type:'medium',team:1,x:400,y:500,angle:2,aim:3,hp:200,maxHp:200,moving:true,nativeMoveFrame:2,nativeTerrainCode:7,cooldown:1,originalCombat:{fireRate:80},nativeHardware:{hatchFrame:2,hatchBody:1}},capturedCalls=[];
const captured={map:{original:true},units:[{id:8,type:'medium',team:1,x:1000,y:1200,angle:7,hp:1}],buildings:[],nativeDisplayQueue:[{stage:'queue',kind:'unit',id:7,nativeSlot:1,palette:0,sprite:315,x:100,y:200},{stage:'queue',kind:'unit',id:8,nativeSlot:2,palette:1,sprite:null,x:400,y:500,art:saved,time:1.25}]};
assert(display.draw(ctx,captured,{sprite:f=>capturedCalls.push(['sprite',f.sprite,f.x,f.y]),unit:(u,sprite,time)=>capturedCalls.push(['hardware',u.id,u.x,u.y,u.hp,u.angle,u.nativeTerrainCode,u.nativeHardware.hatchFrame,time])}));
assert.deepEqual(capturedCalls,[['sprite',315,100,200],['hardware',8,400,500,200,2,7,2,1.25]],'retired normal UNIT sprites and captured hardware inputs survive later state changes');
assert.equal(captured.units[0].x,1000);assert.equal(saved.team,1,'display palette must not mutate captured ownership');
const buildingCalls=[],buildingArt={id:19,kind:'robot',team:0,x:400,y:500,hp:100,maxHp:100,product:'grunt',completionTicks:3,nativeFlagFrame:2};
const buildingState={map:{original:true},units:[],buildings:[],nativeDisplayQueue:[{kind:'building',stage:'pre',id:19,time:1.25,seconds:0,radarDishQueued:true,art:buildingArt}]};
assert(display.draw(ctx,buildingState,{building:(b,time,seconds)=>buildingCalls.push([b.id,b.team,b.hp,b.product,b.nativeFlagFrame,b.nativeRadarDishQueued,time,seconds])}));
assert.deepEqual(buildingCalls,[[19,0,100,'grunt',2,true,1.25,0]],'the earlier BUILD draw survives subsequent owner/HP/production changes and complete removal');
assert.equal(buildingArt.nativeRadarDishQueued,undefined,'captured BUILD payload is immutable to the renderer');
const workshopCalls=[];window.OriginalArt.repairOverlay=(c,r,map)=>workshopCalls.push(['overlay',r.team,r.x,r.y]);window.OriginalArt.clock=(c,x,y,seconds,map)=>workshopCalls.push(['clock',x,y,seconds,map.original]);
const workshopState={map:{original:true},units:[],buildings:[],nativeDisplayQueue:[{kind:'building',stage:'pre',id:3,art:{id:3}},{kind:'repairOverlay',stage:'pre',id:3,nativeSlot:7,team:1,x:100,y:296},{kind:'repairClock',stage:'pre',id:3,nativeSlot:7,x:152,y:282,seconds:null},{kind:'sprite',stage:'queue',id:1,palette:255,x:0,y:0,sprite:100}]};
assert(display.draw(ctx,workshopState,{building:b=>workshopCalls.push(['building',b.id]),objectShadows:()=>workshopCalls.push(['shadows']),sprite:f=>workshopCalls.push(['sprite',f.sprite])}));
assert.deepEqual(workshopCalls,[['building',3],['overlay',1,100,296],['clock',152,282,null,true],['shadows'],['sprite',100]],'workshop time is retained in BUILD order before the UNIT queue, including stalled rate');
window.GameGraphics.isHD=()=>true;assert.equal(display.draw(ctx,state,{}),false,'HD keeps its presentation adapter');
for(const entry of ['dist/index.html','dist/independent/index.html']){const html=fs.readFileSync(path.join(__dirname,'..',entry),'utf8'),prefix=entry==='dist/index.html'?'independent/':'',displayAt=html.indexOf(`src="${prefix}original-display.js"`),appAt=html.indexOf(`src="${prefix}app.js"`);assert(displayAt>=0&&displayAt<appAt,`${entry}: native display must load before the app in both hosted and local entries`);}
console.log('native display: 512 native queue/palette traces; direct BUILD pass, inherited colors, stable identities and retained callback pictures passed');
