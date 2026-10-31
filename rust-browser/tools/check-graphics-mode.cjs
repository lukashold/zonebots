// Verify switching presentation never initializes or mutates the game simulation.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const project=path.join(__dirname,'..'),source=fs.readFileSync(path.join(project,'dist/independent/graphics.js'),'utf8');
function fixture(saved,{blocked=false}={}){
 const calls=[],listeners={},events=[],select={value:'',addEventListener(n,f){this[n]=f;}},radios=['original','hd'].map(value=>({value,checked:false,addEventListener(n,f){this[n]=f;}})),label={textContent:''},root={dataset:{}};
 const bounds={x:3,y:4,w:40,h:50},storage=new Map(saved===undefined?[]:[['z.graphics.v1',saved]]);
 const original=new Proxy({ready:true,attackBounds:()=>bounds,unitBounds:()=>bounds,crateBounds:()=>bounds},{get(target,name){return target[name]??((...args)=>{calls.push(['original',name,args]);return true;});}});
 const module=name=>new Proxy({},{get:(_,method)=>(...args)=>{calls.push([name,method,args]);return true;}});
 const window={OriginalArt:original,HDUnits:module('units'),HDStructures:module('structures'),HDTerrain:module('terrain'),addEventListener:(name,fn)=>listeners[name]=fn,dispatchEvent:event=>events.push(event.type)};
 const document={documentElement:root,querySelectorAll:selector=>({'[data-graphics-select]':[select],'[data-graphics-choice]':radios,'[data-graphics-current]':[label],'[data-graphics-preview]':[]}[selector]||[])};
 vm.runInNewContext(source,{window,document,localStorage:{getItem(k){if(blocked)throw Error();return storage.get(k)??null;},setItem(k,v){if(blocked)throw Error();storage.set(k,v);}},Event:class{constructor(type){this.type=type;}}});
 return {api:window.GameGraphics,calls,listeners,events,select,radios,label,root,bounds,storage};
}
for(const saved of [undefined,'garbage','original']){
 const f=fixture(saved);assert.equal(f.api.mode,'original');assert.equal(f.root.dataset.graphicsMode,'original');assert.equal(f.radios[0].checked,true);
 f.api.art.unit({}, {id:17},12,{});assert.equal(f.calls[0][0],'original');
 assert.equal(f.api.setMode('wrong'),false);assert.equal(f.api.mode,'original');
}
const f=fixture('hd');assert.equal(f.api.mode,'hd');assert.equal(f.select.value,'hd');assert.equal(f.label.textContent,'HD-Remake');
const ctx={},u={id:3},map={revision:1};f.api.art.unit(ctx,u,5,map);
assert.deepEqual(f.calls.at(-1),['units','unit',[ctx,u,5,map,f.bounds]]);
for(const [name,target] of [['building','structures'],['object','structures'],['flag','structures'],['objectShadows','structures'],['flagShadows','structures'],['effects','structures'],['terrain','terrain'],['minimap','terrain']]){
 f.api.art[name](ctx,map);assert.equal(f.calls.at(-1)[0],target);assert.equal(f.calls.at(-1)[1],name);
}
f.api.art.crate(ctx,u,map);assert.deepEqual(f.calls.at(-1),['structures','crate',[ctx,u,map,f.bounds]]);
f.api.art.production(ctx,u,90,map);assert.equal(f.calls.at(-1)[0],'original');assert.equal(f.calls.at(-1)[1],'production');
const drawCount=f.calls.length;f.select.value='original';f.select.change();
assert.equal(f.calls.length,drawCount,'changing mode must not invoke rendering or game initialization');assert.equal(f.api.mode,'original');assert.deepEqual(f.events,['graphicschange']);assert.equal(f.storage.get('z.graphics.v1'),'original');
f.radios[1].checked=true;f.radios[1].change();assert.equal(f.api.mode,'hd');assert.equal(f.select.value,'hd');assert.equal(f.radios[0].checked,false);
f.listeners.storage({key:'unrelated',newValue:'original'});assert.equal(f.api.mode,'hd');f.listeners.storage({key:'z.graphics.v1',newValue:'original'});assert.equal(f.api.mode,'original');
const b=fixture('hd',{blocked:true});assert.equal(b.api.mode,'original');assert.equal(b.api.setMode('hd'),true);assert.equal(b.api.mode,'hd','graphics must remain usable when browser storage is blocked');
for(const entry of ['dist/index.html','dist/independent/index.html']){
 const html=fs.readFileSync(path.join(project,entry),'utf8'),scripts=[...html.matchAll(/<script src="([^"]+)"/g)].map(m=>path.basename(m[1]));
 for(const module of ['hd-terrain.js','hd-structures.js','hd-units.js','graphics.js'])assert.ok(scripts.indexOf(module)>scripts.indexOf('original-art.js')&&scripts.indexOf(module)<scripts.indexOf('app.js'));
 assert.match(html,/data-graphics-select/);assert.equal([...html.matchAll(/data-graphics-choice/g)].length,2);assert.equal([...html.matchAll(/data-graphics-preview/g)].length,2);
}
console.log('Graphics mode passed: original default, saved HD preference, linked menu controls, invalid/blocked storage, cross-tab changes, renderer dispatch with native geometry, production clocks, no simulation calls on switching, both entrypoints.');
