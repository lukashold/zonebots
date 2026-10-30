// Actual frontend input handlers and Rust/Wasm, using a wholly synthetic map.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),binary=fs.readFileSync(path.join(root,'dist/independent/engine.wasm'));
function mission(){const size=32,n=size*size;return {number:1,name:'Synthetic grenade pickup',
 map:{width:size,height:size,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:1,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0)},
 sectors:[{id:0,x:384,y:384,owner:0,home:true,countsForProduction:true}],
 buildings:[{id:0,kind:'fort',team:0,sector:0,x:96,y:96,w:192,h:192,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0}],
 spawns:[{type:'grunt',team:0,x:160,y:512,squad:1},{type:'grunt',team:0,x:192,y:512,squad:1},{type:'jeep',team:0,x:224,y:512,squad:2},{type:'gatling',team:1,x:960,y:960,squad:3}],
 crates:[{id:0,x:640,y:512,taken:false},{id:1,x:800,y:640,taken:false}],originalObjects:[]};}
async function check(mode){
 let response,frame,now=0,translation=[0,0],hands=[];const calls=[],spoken=[],elements=new Map(),listeners=new Map();
 const ctx=new Proxy({translate:(x,y)=>{translation=[x,y];}},{get:(o,k)=>k in o?o[k]:(()=>{}),set:(o,k,v)=>{o[k]=v;return true;}});
 function node(id=''){return {id,value:id==='classic'?mode:id==='difficulty'?'1':'0',style:{},dataset:{},children:[],width:200,height:220,hidden:true,disabled:false,textContent:'',classList:{toggle(){}},getContext:()=>ctx,
  getBoundingClientRect:()=>({left:1100,top:460,width:200,height:220}),setPointerCapture(){},replaceChildren(...children){this.children=children;},append(child){this.children.push(child);},querySelector:()=>null};}
 function element(id){if(!elements.has(id))elements.set(id,node(id));return elements.get(id);}
 const raw=(await WebAssembly.instantiate(binary,{})).instance.exports;
 const exports={...raw,run:(ptr,length)=>{const input=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,ptr,length)).toString());calls.push(input);
  const n=raw.run(ptr,length),reply=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,raw.output(),n)).toString());if(reply.buildings)response=reply;return n;}};
 const art=new Proxy({ready:true,load:async()=>true,unitBounds:()=>null,fortGuns:()=>[],flagBounds:()=>null,
  crateBounds:box=>box.taken?null:{x:box.x,y:box.y,w:32,h:32},captureCursor:()=> 'pickup-hand',captureHand:(c,x,y,map,elapsed)=>{hands.push({x,y,elapsed});return true;}},{get:(o,k)=>k in o?o[k]:(()=>true)});
 const env={window:{innerWidth:1280,addEventListener:(type,fn)=>listeners.set(type,fn)},innerHeight:720,devicePixelRatio:1,TextEncoder,TextDecoder,URL,Set,Map,performance:{now:()=>now},setTimeout:()=>0,clearTimeout(){},requestAnimationFrame:fn=>{frame=fn;},
  document:{getElementById:element,createElement:()=>node(),querySelector:()=>element('overlayButtons'),querySelectorAll:()=>[],currentScript:{src:'http://test/independent/app.js'}},WebAssembly:{instantiate:async()=>({instance:{exports}})},
  fetch:async()=>({ok:true,arrayBuffer:async()=>binary,json:async()=>({missions:[mission()]})}),OriginalArt:art,IndependentArt:new Proxy({},{get:()=>()=>true}),BattlefieldVisibility:{radarDetects:()=>true,radarRadius:()=>100},
  RadioAudio:new Proxy({load:async()=>true,say:(kind,detail)=>spoken.push({kind,detail})},{get:(o,k)=>o[k]||(()=>{})})};
 vm.createContext(env);for(const file of ['flag-command.js','vehicle-command.js','pickup-command.js','attack-command.js']){vm.runInContext(fs.readFileSync(path.join(root,'dist/independent',file),'utf8'),env);Object.assign(env,env.window);}
 vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8'),env);await new Promise(resolve=>setImmediate(resolve));assert.equal(element('start').disabled,false);
 element('start').onclick();element('classic').onchange();element('cycleV').onclick();
 const canvas=element('world'),render=(dt=1)=>{now+=dt;hands=[];frame(now);},commands=()=>calls.filter(c=>c.op==='command');
 const pointer=(x,y,button=0,shiftKey=false)=>({clientX:x+translation[0],clientY:y+translation[1],pointerId:1,button,shiftKey,ctrlKey:false});
 const click=(x,y,button=0,shift=false)=>{const p=pointer(x,y,button,shift);canvas.onpointerdown(p);canvas.onpointerup(p);};
 const hover=(x,y)=>{canvas.onpointermove(pointer(x,y));render();};
 render();hover(670,542);assert.notEqual(canvas.style.cursor,'pickup-hand','A vehicle cannot collect grenades');
 // Select the vehicle first, then add both infantry: the ineligible unit must
 // never take formation index zero away from the crate's actual collector.
 click(160,512,0,true);hover(670,542);assert.equal(canvas.style.cursor,'pickup-hand');
 const button=mode==='classic'?0:2;click(670,542,button);
 let order=commands().at(-1);assert.equal(order.kind,'move');assert.equal(order.x,640);assert.equal(order.y,512);
 assert.equal(order.ids.length,2);assert(order.ids.every(id=>response.units.find(u=>u.id===id)?.type==='grunt'));
 const first=response.units.find(u=>u.id===order.ids[0]);assert.deepEqual(first.dest,[640,512]);
 assert(response.crates.every(c=>!c.taken));assert.equal(spoken.filter(s=>s.kind==='grenades').length,0,'No pickup announcement on order issue');
 render();assert.equal(hands.length,1,'A shared target gets one hand marker');
 element('stop').onclick();render();assert.equal(hands.length,0,'Stop clears the pickup marker');
 hover(670,542);click(670,542,button);render();assert.equal(hands.length,1);
 let collected=false;for(let i=0;i<400;i++){render(50);if(response.crates[0].taken){collected=true;break;}}
 assert(collected,`${mode}: actual Rust movement must reach and collect the crate`);
 render(50);render(50);assert.equal(hands.length,0,'Collected crates lose their hand marker');assert.notEqual(canvas.style.cursor,'pickup-hand');
 const grenades=()=>response.units.filter(u=>u.team===0).reduce((n,u)=>n+u.grenades,0);
 assert.equal(grenades(),8);assert.equal(spoken.filter(s=>s.kind==='grenades').length,1,'Pickup sound follows the actual Rust event exactly once');
 for(let i=0;i<10;i++)render(50);assert.equal(grenades(),8,'A taken crate cannot grant grenades twice');
 hover(830,670);click(830,670,button);render();assert.equal(hands.length,1);
 click(700,700,button);render();assert.equal(hands.length,0,'A new ordinary move overrides pickup feedback');
 assert.equal(response.crates[1].taken,false);
 console.log(`PASS ${mode}: infantry-only hand, full crate hit rectangle, raw pickup destination, mixed selection, stop/override cleanup, actual +8 pickup and one event.`);
}
(async()=>{for(const mode of ['classic','modern'])await check(mode);})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
