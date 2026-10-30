// Menu routing and original-art loading checks without private resources or a browser.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/menu-shell.js'),'utf8');
function element(dataset={}){
 const classes=new Set(),listeners=new Map();
 return {dataset,hidden:false,classList:{add(name){classes.add(name);},contains(name){return classes.has(name);}},listeners,
  addEventListener(name,fn){listeners.set(name,fn);},fire(name){return listeners.get(name)?.();}};
}
function fixture(options={}){
 const pages=['home','new','options','help'].map(name=>element({menuPage:name}));
 const links=['new','options','help','home'].map(name=>element({menuOpen:name}));
 const menu=element(),root=element(),help=element(),close=element(),label=element({originalLabel:'newGame'});
 const draws=[],clears=[],properties=new Map(),fetches=[],images=[];
 const context2d={clearRect(...args){clears.push(args);},drawImage(...args){draws.push(args);}};
 const canvas={width:260,height:48,getContext(name){assert.equal(name,'2d');return context2d;}};
 label.querySelector=selector=>{assert.equal(selector,'canvas');return canvas;};
 root.style={setProperty(name,value){properties.set(name,value);}};
 const manifest={assets:{
  background:{path:'menu/background.png'},title:{path:'menu/title.png'},
  panel:{path:'menu/panel.png',width:1024,height:1024,content:[0,1,676,591]},
  menuText:{path:'menu/menuText.png'}
 },labels:{newGame:{normal:[12,14,215,26],hover:[5,49,226,36]}}};
 class Image {
  set src(value){
   this.url=String(value);images.push(this);
   if(options.deferTitle&&this.url.endsWith('/title.png'))return;
   const failed=options.imageError||(options.titleError&&this.url.endsWith('/title.png'))||(options.labelError&&this.url.endsWith('/menuText.png'));
   queueMicrotask(()=>failed?this.onerror(new Error('Image unavailable')):this.onload());
  }
 }
 const document={currentScript:{src:options.script||'https://example.test/independent/menu-shell.js'},documentElement:root,
  getElementById(id){return {menu,fullscreenHelp:help,fullscreenHelpClose:close}[id]||null;},
  querySelectorAll(selector){return {'[data-menu-page]':pages,'[data-menu-open]':links,'[data-original-label]':[label]}[selector]||[];}};
 const window={};
 const context=vm.createContext({window,document,Image,URL,
  fetch:async url=>{fetches.push(String(url));if(options.networkError)throw new Error('Offline');return {ok:!options.missing,json:async()=>manifest};}});
 vm.runInContext(source,context);
 return {api:window.MenuShell,pages,links,menu,root,help,close,label,canvas,draws,clears,properties,fetches,images,manifest,context2d};
}
function visiblePage(state,name){
 assert.equal(state.menu.dataset.page,name);
 assert.deepEqual(state.pages.filter(page=>!page.hidden).map(page=>page.dataset.menuPage),[name]);
}
async function settled(){await new Promise(resolve=>setImmediate(resolve));}
function crop(draw,expected){assert.deepEqual(draw.slice(1,5),expected);assert.equal(draw.length,9);}
async function run(){
 let state=fixture({missing:true});await settled();visiblePage(state,'home');
 for(const name of ['new','options','help','home']){state.links.find(link=>link.dataset.menuOpen===name).fire('click');visiblePage(state,name);}
 state.api.show('not-a-page');visiblePage(state,'home');
 state.close.fire('click');assert.equal(state.help.hidden,true);
 assert.equal(await state.api.load(),false);assert.equal(state.root.classList.contains('original-menu-ready'),false);
 assert.equal(state.label.classList.contains('has-original-label'),false);assert.equal(state.draws.length,0);
 state=fixture({networkError:true});await settled();assert.equal(await state.api.load(),false);visiblePage(state,'home');
 state.links[1].fire('click');visiblePage(state,'options');

 state=fixture();await settled();
 assert.equal(state.fetches[0],'https://example.test/original/menu.json');
 assert.equal(state.properties.get('--menu-background'),'url("https://example.test/original/menu/background.png")');
 assert.equal(state.properties.get('--menu-title'),'url("https://example.test/original/menu/title.png")');
 assert.equal(state.properties.get('--menu-panel-size'),`${1024/676*100}% ${1024/591*100}%`);
 assert.equal(state.root.classList.contains('original-menu-ready'),true);
 assert.equal(state.label.classList.contains('has-original-label'),true);
 assert.ok(state.images.some(image=>image.url==='https://example.test/original/menu/menuText.png'));
 assert.equal(state.context2d.imageSmoothingEnabled,false);
 crop(state.draws.at(-1),[12,14,215,26]);
 state.label.fire('pointerenter');crop(state.draws.at(-1),[5,49,226,36]);
 state.label.fire('pointerleave');crop(state.draws.at(-1),[12,14,215,26]);
 state.label.fire('focus');crop(state.draws.at(-1),[5,49,226,36]);
 state.label.fire('blur');crop(state.draws.at(-1),[12,14,215,26]);
 assert.equal(state.clears.length,state.draws.length);
 assert.ok(state.draws.every(draw=>draw[5]>=0&&draw[6]>=0&&draw[5]+draw[7]<=260&&draw[6]+draw[8]<=48),'every label crop must fit its canvas');
 state=fixture({deferTitle:true});await settled();
 assert.equal(state.root.classList.contains('original-menu-ready'),false,'fallback Z must stay visible until title image has loaded');
 const title=state.images.find(image=>image.url.endsWith('/title.png'));assert.ok(title);title.onload();await settled();
 assert.equal(state.root.classList.contains('original-menu-ready'),true);
 assert.equal(state.label.classList.contains('has-original-label'),true);
 state=fixture({titleError:true});await settled();
 assert.equal(state.root.classList.contains('original-menu-ready'),false,'broken private title must retain fallback Z');
 assert.equal(state.label.classList.contains('has-original-label'),true,'broken title must not block working label artwork');
 crop(state.draws.at(-1),[12,14,215,26]);state.label.fire('pointerenter');crop(state.draws.at(-1),[5,49,226,36]);
 state=fixture({labelError:true});await settled();
 assert.equal(state.root.classList.contains('original-menu-ready'),true,'a broken bitmap label must not hide a working title');
 assert.equal(state.label.classList.contains('has-original-label'),false);assert.equal(state.draws.length,0);
 state=fixture({imageError:true});await settled();
 assert.equal(state.root.classList.contains('original-menu-ready'),false);assert.equal(state.label.classList.contains('has-original-label'),false);assert.equal(state.draws.length,0);

 for(const file of ['dist/index.html','dist/independent/index.html']){
  const html=fs.readFileSync(path.join(__dirname,'..',file),'utf8'),prefix=file==='dist/index.html'?'independent/':'';
  const ids=[...html.matchAll(/\bid="([^"]+)"/g)].map(match=>match[1]);assert.equal(ids.length,new Set(ids).size,`${file}: duplicate IDs`);
  for(const id of ['mission','difficulty','classic','music','effects','voice','start','continueBattle','loading','fullscreenHelpClose'])assert.ok(ids.includes(id),`${file}: missing #${id}`);
  assert.equal([...html.matchAll(/data-fullscreen\b/g)].length,2,`${file}: menu and game fullscreen buttons`);
  assert.ok(html.includes(`href="${prefix}menu.css"`));
  for(const script of ['menu-shell.js','fullscreen.js'])assert.ok(html.includes(`src="${prefix}${script}"`));
 }
 console.log('Menu checks passed: view navigation/back, offline/missing-resource fallback, deferred/failed title retaining Z fallback, independent label/title failure, private asset URLs, original label normal/hover/focus crops, both HTML entrypoints and controls.');
}
run().catch(error=>{console.error(error);process.exitCode=1;});
