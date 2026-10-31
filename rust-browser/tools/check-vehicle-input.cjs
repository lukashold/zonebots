// Exercise the real frontend input handlers against Rust/Wasm, without a browser.
// Requires the separately extracted private campaign and sprite metadata.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),campaign=JSON.parse(fs.readFileSync(path.join(root,'dist/original/campaign.json'))),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json')));
const binary=fs.readFileSync(path.join(root,'dist/independent/engine.wasm'));
async function check(mode){
 let response,frame,translation=[0,0],arrows=[],commands=[];
 const ctx=new Proxy({translate:(x,y)=>{translation=[x,y];}}, {get:(o,k)=>k in o?o[k]:(()=>{}),set:(o,k,v)=>{o[k]=v;return true;}});
 const elements=new Map();
 function element(id){if(!elements.has(id))elements.set(id,{id,value:id==='classic'?mode:id==='difficulty'?'1':'0',style:{},dataset:{},width:200,height:220,hidden:true,disabled:false,textContent:'',classList:{toggle(){}},getContext:()=>ctx,getBoundingClientRect:()=>({left:1100,top:460,width:200,height:220}),setPointerCapture(){},replaceChildren(){},append(){},querySelector:()=>null});return elements.get(id);}
 const raw=(await WebAssembly.instantiate(binary,{})).instance.exports;
 const exports={...raw,run:(ptr,length)=>{
  const request=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,ptr,length)).toString());
  if(request.op==='command')commands.push(request);
  const n=raw.run(ptr,length);const part=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,raw.output(),n)).toString());response=part.map?part:{...response,...part};return n;
 }};
 const bounds=u=>{const f=u.team<0&&metadata.frames[metadata.neutralUnits[u.type]];return f?{x:Math.round(u.x),y:Math.round(u.y),w:f.w*2,h:f.h*2}:null;};
 const art=new Proxy({ready:true,load:async()=>true,unitBounds:bounds,fortGuns:()=>[],flagBounds:()=>null,entryCursor:()=> 'entry-arrow',entryArrow:(c,x,y,map,elapsed)=>{arrows.push({x,y,elapsed});return true;}},{get:(o,k)=>k in o?o[k]:(()=>true)});
 const env={window:{innerWidth:1280,addEventListener(){}},innerHeight:720,devicePixelRatio:1,TextEncoder,TextDecoder,URL,Set,Map,performance:{now:()=>0},setTimeout:()=>0,clearTimeout(){},requestAnimationFrame:fn=>{frame=fn;},document:{getElementById:element,createElement:()=>({}),querySelector:()=>element('overlayButtons'),querySelectorAll:()=>[],currentScript:{src:'http://test/independent/app.js'}},WebAssembly:{instantiate:async()=>({instance:{exports}})},fetch:async url=>({ok:true,arrayBuffer:async()=>binary,json:async()=>campaign}),OriginalArt:art,IndependentArt:new Proxy({},{get:()=>()=>true}),BattlefieldVisibility:{radarDetects:()=>true,radarRadius:()=>100},RadioAudio:new Proxy({load:async()=>true},{get:(o,k)=>o[k]||(()=>{})})};
 vm.createContext(env);
 for(const file of ['flag-command.js','vehicle-command.js','pickup-command.js','attack-command.js']){vm.runInContext(fs.readFileSync(path.join(root,'dist/independent',file),'utf8'),env);Object.assign(env,env.window);}
 vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8'),env);
 await new Promise(resolve=>setImmediate(resolve));assert.equal(element('start').disabled,false);
 element('start').onclick();element('classic').onchange();element('cycleR').onclick();
 const target=response.units.find(u=>u.team<0&&u.type==='jeep');assert(target,'Campaign must provide vacant hardware');
 const box=bounds(target),world={x:box.x+box.w/2,y:box.y+box.h/2};
 const map=response.map;
 element('mini').onpointerdown({clientX:1100+world.x/(map.width*32)*200,clientY:460+world.y/(map.height*32)*220,pointerId:1});element('mini').onpointerup();
 frame(1);const p={clientX:world.x+translation[0],clientY:world.y+translation[1],pointerId:1,button:mode==='classic'?0:2,shiftKey:false,ctrlKey:false};
 const canvas=element('world');canvas.onpointermove(p);frame(2);assert.equal(canvas.style.cursor,'entry-arrow',`${mode}: eligible hardware gets entry cursor`);
 canvas.onpointerdown(p);canvas.onpointerup(p);
 assert.equal(commands.length,1,`${mode}: exactly one command per click`);
 const command=commands[0];assert.equal(command.kind,'enter');assert.equal(command.target,target.id);assert(command.ids.length>0);
 for(const id of command.ids){const u=response.units.find(u=>u.id===id);assert.equal(response.catalog.find(d=>d.type===u.type).class,0);assert.equal(u.order,'enter');assert.equal(u.target,target.id);assert.equal(u.path.length,0,'The first tick has not built the path yet');}
 assert.match(element('unitOrderInfo').textContent,/Bemannen/);
 arrows=[];frame(3);assert.equal(arrows.length,1,`${mode}: shared target produces one marker even before the first tick`);assert.equal(arrows[0].x,world.x+translation[0]);assert.equal(arrows[0].y,world.y+translation[1]);
 element('stop').onclick();arrows=[];frame(4);assert.equal(arrows.length,0,`${mode}: new command removes the old entry marker`);
 // A completed crew change must remove the marker without any UI command.
 canvas.onpointerdown(p);canvas.onpointerup(p);arrows=[];frame(5);assert.equal(arrows.length,1);
 let completed=false;
 for(let i=0;i<1200;i++){
  const request=Buffer.from(JSON.stringify({op:'tick',dt:.05})),ptr=raw.alloc(request.length);new Uint8Array(raw.memory.buffer,ptr,request.length).set(request);exports.run(ptr,request.length);raw.free(ptr,request.length);
  if(response.units.find(u=>u.id===target.id)?.team===0){completed=true;break;}
 }
 assert(completed,`${mode}: the actual Rust engine must crew the commanded hardware`);
 // Refresh the frontend from the shared Wasm instance without advancing time.
 frame(65);arrows=[];frame(66);assert.equal(arrows.length,0,`${mode}: live crew change clears the marker`);
 console.log(`PASS ${mode}: cursor, click -> Rust enter order, infantry filter, single destination marker, stop cleanup and real crew completion.`);
}
(async()=>{for(const mode of ['classic','modern'])await check(mode);})().catch(error=>{console.error(error);process.exitCode=1;});
