'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/original-hud.js'),'utf8');
const settle=()=>new Promise(resolve=>setImmediate(resolve));
// Independent fixture: fresh Ghidra memory export0049c6c0+4*animationFamily,
// interpreted using00469478 and00469190 ASM: caption,image,nativeY,other.
const nativeDescriptors=[
 ['cannon',5,[94,106,8,15]],['medium',6,[99,111,1,30]],['missile',21,[88,100,4,22]],
 ['gatling',22,[92,104,5,21]],['howitzer',23,[90,102,5,21]],['launcher',24,[93,105,1,30]],
 ['heavy',25,[91,103,2,27]],['light',30,[96,108,1,30]],['apc',32,[98,110,4,25]],
 ['jeep',34,[89,101,4,22]],['crane',36,[95,107,3,26]],['firetruck',38,[97,109,5,22]],
];
const nativeHeights={100:44,101:46,102:42,103:54,104:42,105:60,106:30,107:52,108:60,109:44,110:50,111:60};
function fixture({missing=false,broken=false}={}){
 const nodes={},events={},calls=[],encoded=[];
 function element(id){
  const classes=new Set();const node={id,hidden:false,width:96,height:96,style:{setProperty(k,v){this[k]=v;}},textContent:'',parentElement:null,
   classList:{add(k){classes.add(k);},contains(k){return classes.has(k);}},
   appendChild(child){child.parentElement=this;},addEventListener(name,handler){this[name]=handler;},click(){this.clicks=(this.clicks||0)+1;},
   toDataURL(){encoded.push(id);return 'data:image/png;base64,'+id;},
   getContext(){return {clearRect(...args){calls.push([id,'clear',...args]);},createImageData(w,h){return {data:new Uint8ClampedArray(w*h*4)};},putImageData(image){node.pixelId=image.data[0];},getImageData(){const data=new Uint8ClampedArray(354*2*4);for(let y=0;y<2;y++)for(let x=0;x<354;x++){const at=(y*354+x)*4;data[at]=Math.floor(x/2);data[at+3]=255;}return {data};},drawImage(...args){calls.push([id,'draw',...args]);}};}};
  return node;
 }
 for(const id of ['hud','nativeHud','mini','hudMenu','pauseButton','portrait','hudCaption','hudUnitArt','hudHealth','unitName','unitInfo','unitOrderInfo','cycleR','cycleG','cycleV','cycleB','overlayT','overlayD','overlayZ','overlay'])nodes[id]=element(id);
 nodes.mini.parentElement=nodes.hud;nodes.hudHealth.width=80;nodes.hudHealth.height=6;
 let counter=0;const root=element('root');
 const document={currentScript:{src:'https://example.test/independent/original-hud.js'},documentElement:root,getElementById(id){return nodes[id];},querySelector(){return nodes.overlay;},createElement(){return element('canvas'+counter++);}};
 const manifest={atlas:'hud/atlas.png',frames:Array.from({length:177},(_,i)=>({x:i*2,y:0,w:i>=88&&i<=115?96:2,h:i>=88&&i<=99?14:nativeHeights[i]||2})),palettes:Array.from({length:5},()=>Array.from({length:768},(_,i)=>Math.floor(i/3))),backgroundStart:131,headStart:137};
 const window={innerWidth:1280,innerHeight:720,addEventListener(name,fn){events[name]=fn;},dispatchEvent(){}};
 const context={window,document,URL,Event:class{},Image:class{constructor(){this.width=354;this.height=2;}set src(url){this.url=url;queueMicrotask(()=>broken?this.onerror():this.onload());}},fetch:async()=>({ok:!missing,json:async()=>manifest})};
 vm.runInNewContext(source,context);return {api:window.OriginalHud,window,root,nodes,calls,encoded,events,manifest};
}
async function run(){
 let f=fixture({missing:true});assert.equal(f.api.sidebarWidth(),240);await settle();
 assert.equal(f.api.isReady(),false);assert.equal(f.api.sidebarWidth(),240);assert.equal(f.nodes.mini.parentElement,f.nodes.hud);
 f.window.innerWidth=600;f.api.resize();assert.equal(f.api.sidebarWidth(),0);assert.equal(f.nodes.mini.parentElement,f.nodes.hud);
 f=fixture({broken:true});await settle();assert.equal(f.api.isReady(),false);assert.equal(f.root.classList.contains('original-hud-ready'),false);
 f=fixture();await settle();assert.equal(f.api.isReady(),true);assert.equal(f.api.sidebarWidth(),144);assert.equal(f.nodes.mini.parentElement,f.nodes.nativeHud);
 const state={map:{width:100,height:50,world:0}},unit={type:'cannon',team:0,hp:25,maxHp:100};
 f.api.update(state,unit);assert.equal(f.nodes.hudCaption.hidden,false);assert.equal(f.nodes.mini.style.width,'88px');assert.equal(f.nodes.mini.style.height,'44px');
 const health=f.calls.filter(call=>call[0]==='hudHealth'&&call[1]==='draw').at(-1);assert.equal(health.at(-2),20,'quarter health has quarter bar width');
 const count=f.encoded.length;f.api.update(state,unit);assert.equal(f.encoded.length,count,'cached native skins must not reencode each simulation update');
 for(const [kind,family,descriptor] of nativeDescriptors){
  f.api.update(state,{...unit,type:kind});
  const art=f.calls.filter(call=>call[0]==='hudUnitArt'&&call[1]==='draw').at(-1),caption=f.calls.filter(call=>call[0]==='hudCaption'&&call[1]==='draw').at(-1);
  assert.equal(art[2].pixelId,descriptor[1],`${kind}/family${family}: original hardware image`);
  assert.equal(caption[2].pixelId,descriptor[0],`${kind}: original caption`);
  assert.equal(art[4],descriptor[2]*2,`${kind}: native image offset`);
  assert.equal(f.manifest.frames[caption[2].pixelId].h,14,`${kind}: caption is a text strip`);
  assert.ok(f.manifest.frames[art[2].pixelId].h>20,`${kind}: hardware picture must not be text`);
 }
 const privatePath=path.join(__dirname,'../dist/original/hud.json');
 if(fs.existsSync(privatePath)){
  const actual=JSON.parse(fs.readFileSync(privatePath,'utf8'));
  for(const [kind,,descriptor] of nativeDescriptors){assert.equal(actual.frames[descriptor[0]].h,14,kind);assert.equal(actual.frames[descriptor[1]].h,nativeHeights[descriptor[1]],kind);}
 }
 let painted=0;f.api.update(state,{...unit,type:'grunt'},()=>painted++);assert.equal(painted,1,'robot unit preview comes from the actual renderer, separate from communicator');
 f.api.update(state,null);assert.equal(f.nodes.hudCaption.hidden,true);
 f.window.innerWidth=600;f.api.resize();assert.equal(f.api.sidebarWidth(),0);assert.equal(f.nodes.mini.parentElement,f.nodes.hud);assert.equal(f.nodes.mini.style.left,'');assert.equal(f.nodes.mini.style.top,'');assert.equal(f.nodes.mini.style.width,'168px');assert.equal(f.nodes.mini.style.height,'84px');
 f.window.innerWidth=1280;f.api.resize();assert.equal(f.nodes.mini.parentElement,f.nodes.nativeHud);assert.equal(f.api.sidebarWidth(),144);
 f.nodes.hudMenu.click(); // Invoke the bound DOM listener through the event property fixture.
 f.nodes.hudMenu['click']();assert.equal(f.nodes.pauseButton.clicks,2);
 for(const file of ['dist/index.html','dist/independent/index.html']){
  const html=fs.readFileSync(path.join(__dirname,'..',file),'utf8');
  assert.ok(html.indexOf('id="mini"')>html.indexOf('</aside>'),'fallback minimap must remain outside hidden mobile sidebar');
  for(const id of ['nativeHud','hudCaption','hudUnitArt','hudHealth','hudMenu'])assert.equal([...html.matchAll(new RegExp('id="'+id+'"','g'))].length,1);
  assert.ok(html.includes('original-hud.js'));assert.ok(html.includes('original-hud.css'));
 }
 console.log('Original HUD checks passed: all12 hardware portrait/caption/offset mappings against independent Ghidra table fixtures, text/image dimension separation, optional/broken-pack fallback, native sidebar width, independent rendered-unit preview, health bar, cached skins, menu action, map fit and mobile minimap restoration.');
}
run().catch(error=>{console.error(error);process.exitCode=1;});
