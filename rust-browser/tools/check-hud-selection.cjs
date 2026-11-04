//The real app.hud path against the built Wasm, including absent/not-ready
//OriginalHud. Artwork pixels are unnecessary; campaign metadata stays private.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),binary=fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),campaign=JSON.parse(fs.readFileSync(path.join(root,'dist/original/campaign.json'),'utf8'));
async function check(nativeHud){const elements=new Map(),events={};let updates=0;
 const ctx=new Proxy({}, {get:()=>()=>{},set:()=>true});
 function element(id){if(!elements.has(id))elements.set(id,{id,value:id==='classic'?'classic':id==='difficulty'?'1':'0',style:{},dataset:{},width:200,height:220,hidden:false,disabled:false,textContent:'',classList:{toggle(){}},getContext:()=>ctx,getBoundingClientRect:()=>({left:1100,top:460,width:200,height:220}),setPointerCapture(){},replaceChildren(){},append(){},querySelector:()=>null});return elements.get(id);}
 const raw=(await WebAssembly.instantiate(binary,{})).instance.exports;
 const art=new Proxy({ready:true,load:async()=>true,unitBounds:()=>null,fortGuns:()=>[],flagBounds:()=>null},{get:(o,k)=>k in o?o[k]:(()=>true)});
 const window={innerWidth:1280,addEventListener(name,fn){events[name]=fn;}};if(nativeHud)window.OriginalHud={update(){updates++;},resize(){},sidebarWidth:()=>240};
 const env={window,innerHeight:720,devicePixelRatio:1,TextEncoder,TextDecoder,URL,Set,Map,performance:{now:()=>0},setTimeout:()=>0,clearTimeout(){},requestAnimationFrame(){},document:{getElementById:element,createElement:()=>({}),querySelector:()=>element('overlayButtons'),querySelectorAll:()=>[],currentScript:{src:'http://test/independent/app.js'}},WebAssembly:{instantiate:async()=>({instance:{exports:raw}})},fetch:async()=>({ok:true,arrayBuffer:async()=>binary,json:async()=>campaign}),OriginalArt:art,IndependentArt:new Proxy({},{get:()=>()=>true}),BattlefieldVisibility:{radarDetects:()=>true,radarRadius:()=>100},RadioAudio:new Proxy({load:async()=>true},{get:(o,k)=>o[k]||(()=>{})})};
 vm.createContext(env);for(const name of['flag-command','vehicle-command','pickup-command','attack-command']){vm.runInContext(fs.readFileSync(path.join(root,'dist/independent',name+'.js'),'utf8'),env);Object.assign(env,window);}vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/app.js'),'utf8'),env);await new Promise(resolve=>setImmediate(resolve));assert.equal(element('start').disabled,false);element('start').onclick();
 const empty=()=>{assert.equal(element('unitInfo').textContent,'','app must not write a second empty-selection instruction');assert.equal(element('unitInfo').hidden,true);assert.match(element('unitOrderInfo').textContent,/Einheit oder Fabrik/);};empty();
 element('cycleR').onclick();assert.equal(element('unitInfo').hidden,false);assert.match(element('unitInfo').textContent,/HP.*Granaten/);assert.match(element('unitOrderInfo').textContent,/Auftrag:/);
 const space={key:' ',target:{matches:()=>false},preventDefault(){},repeat:false};events.keydown(space);empty();events.keydown(space);assert.equal(element('unitInfo').hidden,false);assert.match(element('unitInfo').textContent,/HP.*Granaten/);
 assert(nativeHud?updates>=4:updates===0);console.log(`HUD selection app/Wasm passed: ${nativeHud?'OriginalHud loaded but update is not ready':'OriginalHud unavailable'}, one empty-selection hint, robot HP/order restored, Space clear/restore.`);
}
(async()=>{await check(false);await check(true);})().catch(e=>{console.error(e);process.exitCode=1;});
