// Exercise real frontend input handlers and Wasm using only synthetic data.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),binary=fs.readFileSync(path.join(root,'dist/independent/engine.wasm'));
function mission(){const size=32,n=size*size;return {number:1,name:'Synthetic attack feedback',
 map:{width:size,height:size,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:1,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0)},
 sectors:[{id:0,x:384,y:384,owner:0,home:true,countsForProduction:true},{id:1,x:640,y:640,owner:-1,home:false,countsForProduction:true}],
 buildings:[{id:0,kind:'fort',team:0,sector:0,x:96,y:96,w:192,h:192,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0},
  {id:1,kind:'robot',team:1,sector:0,x:704,y:384,w:96,h:96,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0,gate:[752,512]}],
 spawns:[{type:'grunt',team:0,x:320,y:512,squad:1},{type:'apc',team:0,x:256,y:512,squad:2},{type:'grunt',team:1,x:512,y:512,squad:3}],
 crates:[{x:640,y:512,taken:false}],originalObjects:[]};}
async function check(mode){
 let response,frame,now=0,translation=[0,0],markers=[];const calls=[],elements=new Map(),listeners=new Map();
 const ctx=new Proxy({translate:(x,y)=>{translation=[x,y];}},{get:(o,k)=>k in o?o[k]:(()=>{}),set:(o,k,v)=>{o[k]=v;return true;}});
 function node(id=''){return {id,value:id==='classic'?mode:id==='difficulty'?'1':'0',style:{},dataset:{},children:[],width:200,height:220,hidden:true,disabled:false,textContent:'',classList:{toggle(){}},getContext:()=>ctx,
  getBoundingClientRect:()=>({left:1100,top:460,width:200,height:220}),setPointerCapture(){},replaceChildren(...children){this.children=children;},append(child){this.children.push(child);},querySelector:()=>null};}
 function element(id){if(!elements.has(id))elements.set(id,node(id));return elements.get(id);}
 const raw=(await WebAssembly.instantiate(binary,{})).instance.exports;
 const exports={...raw,run:(ptr,length)=>{const input=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,ptr,length)).toString());calls.push(input);
  const n=raw.run(ptr,length),reply=JSON.parse(Buffer.from(new Uint8Array(raw.memory.buffer,raw.output(),n)).toString());if(reply.buildings)response=reply;return n;}};
 function direct(input){const bytes=Buffer.from(JSON.stringify(input)),ptr=raw.alloc(bytes.length);new Uint8Array(raw.memory.buffer,ptr,bytes.length).set(bytes);try{exports.run(ptr,bytes.length);}finally{raw.free(ptr,bytes.length);}}
 const art=new Proxy({ready:true,load:async()=>true,unitBounds:()=>null,fortGuns:()=>[],
  attackBounds:u=>u.team>0?{x:u.x-12,y:u.y-20,w:48,h:48}:null,flagBounds:s=>s.home?null:{x:s.x,y:s.y-32,w:32,h:32},
  crateBounds:b=>b.taken?null:{x:b.x,y:b.y,w:32,h:32},captureCursor:()=> 'hand',entryCursor:()=> 'entry',attackCursor:()=> 'aim',
  attackMarker:(c,x,y,map,elapsed)=>{markers.push({x,y,elapsed});return true;}},{get:(o,k)=>k in o?o[k]:(()=>true)});
 const env={window:{innerWidth:1280,addEventListener:(type,fn)=>listeners.set(type,fn)},innerHeight:720,devicePixelRatio:1,TextEncoder,TextDecoder,URL,Set,Map,performance:{now:()=>now},setTimeout:()=>0,clearTimeout(){},requestAnimationFrame:fn=>{frame=fn;},
  document:{getElementById:element,createElement:()=>node(),querySelector:()=>element('overlayButtons'),querySelectorAll:()=>[],currentScript:{src:'http://test/independent/app.js'}},WebAssembly:{instantiate:async()=>({instance:{exports}})},
  fetch:async()=>({ok:true,arrayBuffer:async()=>binary,json:async()=>({missions:[mission()]})}),OriginalArt:art,IndependentArt:new Proxy({},{get:()=>()=>true}),BattlefieldVisibility:{radarDetects:()=>true,radarRadius:()=>100},
  RadioAudio:new Proxy({load:async()=>true},{get:(o,k)=>o[k]||(()=>{})})};
 vm.createContext(env);for(const file of ['flag-command.js','vehicle-command.js','pickup-command.js','attack-command.js']){vm.runInContext(fs.readFileSync(path.join(root,'dist/independent',file),'utf8'),env);Object.assign(env,env.window);}
 vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8'),env);await new Promise(resolve=>setImmediate(resolve));assert.equal(element('start').disabled,false);
 element('start').onclick();element('classic').onchange();element('cycleV').onclick();
 assert.equal(response.catalog.find(d=>d.type==='apc').damage,0,'Use rebuilt Wasm with the real combat catalog');
 const canvas=element('world'),render=(dt=1)=>{now+=dt;markers=[];frame(now);},commands=()=>calls.filter(c=>c.op==='command'&&c.team!==1);
 const pointer=(x,y,button=0,shiftKey=false)=>({clientX:x+translation[0],clientY:y+translation[1],pointerId:1,button,shiftKey,ctrlKey:false});
 const click=(x,y,button=0,shift=false)=>{const p=pointer(x,y,button,shift);canvas.onpointerdown(p);canvas.onpointerup(p);};
 const hover=(x,y)=>{canvas.onpointermove(pointer(x,y));render();};
 const key=value=>listeners.get('keydown')({key:value,target:{matches:()=>false},preventDefault(){}});
 const button=mode==='classic'?0:2,enemy=()=>response.units.find(u=>u.team===1);
 render();assert.equal(element('attack').disabled,true);hover(542,527);assert.notEqual(canvas.style.cursor,'aim','APC has no attack cursor');
 key('f');assert.doesNotMatch(element('orderMode').textContent,/Angriffsziel/);click(542,527,button);
 assert(!commands().some(c=>c.kind.startsWith('attack')),'Noncombat-only selection cannot issue an attack');
 // Mixed selection must send only the soldier and hit the visible sprite edge.
 click(320,512,0,true);hover(542,527);assert.equal(element('attack').disabled,false);assert.equal(canvas.style.cursor,'aim');
 click(542,527,button);let order=commands().at(-1);assert.equal(order.kind,'attack');assert.equal(order.target,enemy().id);assert.equal(order.ids.length,1);
 assert.equal(response.units.find(u=>u.id===order.ids[0]).type,'grunt');render();assert.equal(markers.length,1);
 assert.deepEqual([markers[0].x,markers[0].y],[enemy().x+12+translation[0],enemy().y+4+translation[1]],'Marker anchors at actual attack bounds center');
 // Advance a real enemy movement order, then refresh from that shared Wasm state.
 const oldX=enemy().x;direct({op:'command',kind:'move',team:1,ids:[enemy().id],x:720,y:512});direct({op:'advance',steps:4});render(50);
 assert(enemy().x>oldX);assert.equal(markers.length,1);assert.equal(markers[0].x,enemy().x+12+translation[0]);
 element('stop').onclick();render();assert.equal(markers.length,0,'Stop clears the persistent target marker');
 hover(740,420);assert.equal(canvas.style.cursor,'aim');click(740,420,button);order=commands().at(-1);
 assert.equal(order.kind,'attackBuilding');assert.equal(order.target,1);assert.deepEqual([order.x,order.y],[752,512]);
 render();assert.equal(markers.length,1);assert.deepEqual([markers[0].x,markers[0].y],[752+translation[0],432+translation[1]],'Building marker stays at center, independently of its route gate');
 element('stop').onclick();render();assert.equal(markers.length,0);
 hover(850,650);assert.notEqual(canvas.style.cursor,'aim');element('attack').onclick();hover(850,650);assert.equal(canvas.style.cursor,'aim');
 key('Escape');assert.equal(element('orderMode').textContent,'');assert.equal(element('pause').hidden,true,'Escape cancels aim without pausing');hover(850,650);assert.notEqual(canvas.style.cursor,'aim');
 // Forced attack must outrank the ordinary crate and flag hands in both schemes.
 for(const p of [{x:650,y:522},{x:650,y:625}]){
  key('f');hover(p.x,p.y);assert.equal(canvas.style.cursor,'aim');click(p.x,p.y,button);order=commands().at(-1);
  assert.equal(order.kind,'attack');assert.equal(order.target,undefined);assert.deepEqual([order.x,order.y],[p.x,p.y]);
  assert.equal(order.ids.length,1);render();assert.equal(markers.length,1);element('stop').onclick();render();assert.equal(markers.length,0);
 }
 assert.equal(response.crates[0].taken,false);assert.equal(response.sectors[1].owner,-1);
 console.log(`PASS ${mode}: combat-only cursor/orders, mixed selection, sprite-edge targeting, moving target marker, building route/center, Stop, forced ground/hand priority and Escape without pause.`);
}
(async()=>{for(const mode of ['classic','modern'])await check(mode);})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
