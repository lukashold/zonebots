const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
let draws=0,operations=0,created=0,hd=true;
const gradient={addColorStop(offset,color){assert.ok(Number.isFinite(offset)&&offset>=0&&offset<=1);assert.equal(typeof color,'string');}};
const ctx=new Proxy({},{get(_,name){if(name==='createLinearGradient'||name==='createRadialGradient')return (...args)=>{args.forEach(v=>assert.ok(Number.isFinite(v)));return gradient;};return(...args)=>{operations++;for(const arg of args)if(typeof arg==='number')assert.ok(Number.isFinite(arg),`${String(name)} requires finite geometry`);if(name==='drawImage')draws++;};},set(){return true;}});
const window={GameGraphics:{isHD:()=>hd}},document={createElement(tag){assert.equal(tag,'canvas');created++;return {width:0,height:0,getContext:()=>ctx};}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../dist/independent/wall-art.js'),'utf8'),{window,document});const art=window.WallArt;
const make=(mask,id,hp=5)=>({id,x:id%12*32,y:Math.floor(id/12)*32,w:32,h:96,hp,maxHp:5,wallRecord:0,neighborMask:mask,variant:id%4});
for(let world=0;world<5;world++)for(let mask=0;mask<16;mask++)for(const hp of [5,2,0]){
 const state={map:{world},walls:[make(mask,mask,hp)]},before=JSON.stringify(state);art.draw(ctx,state);assert.equal(JSON.stringify(state),before,'painting must not mutate wall state');assert.ok(art.cacheSize<=128);
}
const state={map:{world:0},walls:[make(0,0)]};art.clear();art.draw(ctx,state);const count=created;art.draw(ctx,state);assert.equal(created,count,'warm wall draws reuse cached surfaces');
let before=draws;art.draw(ctx,state,{x:5000,y:5000,w:100,h:100});assert.equal(draws,before,'offscreen walls must not be painted');hd=false;art.draw(ctx,state);assert.equal(draws,before,'original mode uses original terrain tiles');art.draw(ctx,state,undefined,true);assert.ok(draws>before,'HD menu preview can paint independently of selected mode');
console.log(`Wall artwork passed: five worlds,16forms,damage/rubble,finite geometry (${operations} operations),immutable state,bounded cache,warm reuse,viewport clipping and native-mode isolation.`);
