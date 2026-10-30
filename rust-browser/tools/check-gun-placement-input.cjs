// Real frontend pointer/keyboard handlers + Rust/Wasm. Entire fixture is synthetic.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),binary=fs.readFileSync(path.join(root,'dist/independent/engine.wasm'));
const clone=value=>JSON.parse(JSON.stringify(value));
function mission(){
 const size=32,cells=size*size;
 return {number:1,name:'Synthetic placement test',map:{width:size,height:size,tileSize:32,world:0,
  tiles:Array(cells).fill(0),territory:Array(cells).fill(0),bridgeBase:Array(cells).fill(-1),bridgeHp:[],
  seed:1,revision:0,original:true,originalTiles:Array(cells).fill(0),terrainCodes:Array(cells).fill(0)},
  sectors:[{id:0,x:384,y:384,owner:0,home:true,countsForProduction:true}],
  buildings:[{id:0,kind:'fort',team:0,sector:0,x:128,y:128,w:192,h:192,hp:2000,maxHp:2000,
   stars:3,product:'grunt',progress:0,remainingWork:null,readyGuns:['gatling'],gate:[224,320]}],
  spawns:[{type:'grunt',team:0,x:96,y:512,squad:1},{type:'gatling',team:1,x:928,y:928,squad:2}],
  crates:[],originalObjects:[]};
}
async function check(mode){
 let response,frame,now=0,translation=[0,0];const calls=[],ghosts=[],listeners=new Map(),elements=new Map();
 const ctx=new Proxy({translate:(x,y)=>{translation=[x,y];}},{get:(o,k)=>k in o?o[k]:(()=>{}),set:(o,k,v)=>{o[k]=v;return true;}});
 function node(id='') {return {id,value:id==='classic'?mode:id==='difficulty'?'1':'0',style:{},dataset:{},children:[],width:200,height:220,
  hidden:true,disabled:false,textContent:'',classList:{toggle(){}},getContext:()=>ctx,
  getBoundingClientRect:()=>({left:1100,top:460,width:200,height:220}),setPointerCapture(){},
  replaceChildren(...children){this.children=children;},append(child){this.children.push(child);},querySelector:()=>null};}
 function element(id){if(!elements.has(id))elements.set(id,node(id));return elements.get(id);}
 const raw=(await WebAssembly.instantiate(binary,{})).instance.exports;
 const exports={...raw,run:(ptr,length)=>{
  const input=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,ptr,length)).toString());calls.push(input);
  const n=raw.run(ptr,length),reply=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,raw.output(),n)).toString());
  if(reply.buildings)response=reply;return n;
 }};
 function direct(input){const bytes=Buffer.from(JSON.stringify(input)),ptr=raw.alloc(bytes.length);new Uint8Array(raw.memory.buffer,ptr,bytes.length).set(bytes);try{exports.run(ptr,bytes.length);}finally{raw.free(ptr,bytes.length);}return response;}
 const art=new Proxy({ready:true,load:async()=>true,unitBounds:()=>null,fortGuns:()=>[],flagBounds:()=>null,
  unit:(c,u)=>{if(u.id===-1)ghosts.push(u);return true;}},{get:(o,k)=>k in o?o[k]:(()=>true)});
 const source=mission(),env={window:{innerWidth:1280,addEventListener:(type,fn)=>listeners.set(type,fn)},innerHeight:720,
  devicePixelRatio:1,TextEncoder,TextDecoder,URL,Set,Map,performance:{now:()=>now},setTimeout:()=>0,clearTimeout(){},
  requestAnimationFrame:fn=>{frame=fn;},document:{getElementById:element,createElement:()=>node(),querySelector:()=>element('overlayButtons'),
   querySelectorAll:()=>[],currentScript:{src:'http://test/independent/app.js'}},
  WebAssembly:{instantiate:async()=>({instance:{exports}})},fetch:async()=>({ok:true,arrayBuffer:async()=>binary,json:async()=>({missions:[source]})}),
  OriginalArt:art,IndependentArt:new Proxy({},{get:()=>()=>true}),BattlefieldVisibility:{radarDetects:()=>true,radarRadius:()=>100},
  RadioAudio:new Proxy({load:async()=>true},{get:(o,k)=>o[k]||(()=>{})})};
 vm.createContext(env);
 for(const file of ['flag-command.js','vehicle-command.js','pickup-command.js','attack-command.js']){vm.runInContext(fs.readFileSync(path.join(root,'dist/independent',file),'utf8'),env);Object.assign(env,env.window);}
 vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8'),env);
 await new Promise(resolve=>setImmediate(resolve));assert.equal(element('start').disabled,false);
 element('start').onclick();element('classic').onchange();element('cycleR').onclick();
 const canvas=element('world'),queued=()=>response.buildings[0].readyGuns;
 const render=()=>{now+=110;frame(now);};
 const pointer=(x,y,button=0)=>({clientX:x+translation[0],clientY:y+translation[1],pointerId:1,button,shiftKey:false,ctrlKey:false});
 const click=(x,y,button=0)=>{const e=pointer(x,y,button);canvas.onpointerdown(e);canvas.onpointerup(e);};
 const move=(x,y)=>{canvas.onpointermove(pointer(x,y));render();};
 const commands=()=>calls.filter(c=>c.op==='command');
 const placements=()=>calls.filter(c=>c.op==='placeGun');
 const preview=()=>[...calls].reverse().find(c=>c.op==='previewGun');
 assert.deepEqual(queued(),['gatling']);assert.equal(element('placeReadyGun').hidden,false);
 const initialCount=response.units.length;direct({op:'advance',steps:10});render();
 assert.equal(response.units.length,initialCount,'Ready gun must wait rather than spawn automatically');
 assert.deepEqual(queued(),['gatling']);
 element('placeReadyGun').onclick();assert.match(element('orderMode').textContent,/platzieren/);
 move(512,512);assert.equal(preview().id,0);assert.equal(preview().slot,0);
 assert(ghosts.some(u=>u.type==='gatling'),'Placement mode renders an actual gun ghost');
 assert.equal(canvas.style.cursor,'crosshair');
 // A blocked factory tile must keep placement active and never order selected robots.
 click(160,160);assert.deepEqual(queued(),['gatling']);assert.equal(placements().length,0);
 assert.equal(commands().length,0);assert.match(element('orderMode').textContent,/platzieren/);
 // Right click cancels before either control scheme can send a contextual order.
 click(512,512,2);assert.equal(element('orderMode').textContent,'');
 assert.equal(element('factory').hidden,false);assert.deepEqual(queued(),['gatling']);assert.equal(commands().length,0);
 // The per-factory ready button starts the same mode; Esc returns without pausing.
 assert.equal(element('readyGuns').children.length,1);element('readyGuns').children[0].onclick();
 let prevented=false;listeners.get('keydown')({key:'Escape',target:{matches:()=>false},preventDefault(){prevented=true;}});
 assert(prevented);assert.equal(element('orderMode').textContent,'');assert.equal(element('pause').hidden,true);
 assert.deepEqual(queued(),['gatling']);assert.equal(placements().length,0);
 // Confirmation is a LEFT click in both classic and modern controls.
 element('placeReadyGun').onclick();move(512,512);click(512,512);
 assert.equal(placements().length,1);assert.equal(commands().length,0);
 assert.equal(response.placementResult?.valid,true);assert.deepEqual(queued(),[]);
 assert.equal(response.units.length,initialCount+1);assert.equal(element('orderMode').textContent,'');
 const placed=response.units.find(u=>u.type==='gatling'&&u.team===0);assert(placed);
 assert.equal(placed.x%32,0);assert.equal(placed.y%32,0);assert.equal(element('placeReadyGun').hidden,true);
 // A source factory lost while choosing a spot must invalidate the mode and queue.
 element('start').onclick();element('placeReadyGun').onclick();
 const lost=clone(source);lost.buildings[0].team=1;lost.sectors[0].owner=1;
 direct({op:'init',mission:0,difficulty:1,seed:1,aiTeams:[],originalMission:lost});render();
 assert.equal(element('orderMode').textContent,'');assert.deepEqual(queued(),['gatling']);
 assert.equal(element('placeReadyGun').hidden,true);assert.equal(placements().length,1);
 console.log(`PASS ${mode}: waiting queue, ghost preview, invalid-click isolation, right/Esc cancel, left placement, queue consumption and ownership-loss cleanup.`);
}
(async()=>{for(const mode of ['classic','modern'])await check(mode);})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
