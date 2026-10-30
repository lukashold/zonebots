// Portable native-fullscreen behavior checks; no browser or game resources required.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/fullscreen.js'),'utf8');
function element(){
  const listeners=new Map();
  return {textContent:'',hidden:true,dataset:{},attributes:{},listeners,
    setAttribute(name,value){this.attributes[name]=value;},
    addEventListener(name,listener){const list=listeners.get(name)||[];list.push(listener);listeners.set(name,list);},
    fire(name,event={preventDefault(){}}){return (listeners.get(name)||[]).map(listener=>listener(event));}};
}
function fixture(config={}){
  const buttons=[element(),element()],status=element(),help=element(),root=element(),doc=element();
  Object.assign(doc,{documentElement:root,fullscreenElement:null,fullscreenEnabled:true,
    querySelectorAll(selector){assert.equal(selector,'[data-fullscreen]');return buttons;},
    getElementById(id){return config.noPanels?null:({fullscreenStatus:status,fullscreenHelp:help}[id]||null);}});
  const state={buttons,status,help,root,doc,requests:0,exits:0};
  if(!config.unsupported){
    root.requestFullscreen=function(){assert.equal(this,root);state.requests++;doc.fullscreenElement=root;doc.fire('fullscreenchange');return Promise.resolve();};
    doc.exitFullscreen=function(){assert.equal(this,doc);state.exits++;doc.fullscreenElement=null;doc.fire('fullscreenchange');return Promise.resolve();};
  }
  if(config.setup)config.setup(state);
  const window={document:doc,innerHeight:900,screen:{height:900}};
  const context=vm.createContext({window});vm.runInContext(source,context);
  return Object.assign(state,{api:window.GameFullscreen,window});
}
function labels(state,active){
  state.buttons.forEach(button=>{
    assert.equal(button.textContent,active?'Vollbild beenden':'Vollbild');
    assert.equal(button.attributes['aria-pressed'],String(active));
    assert.notEqual(button.disabled,true,'fallback help must stay reachable');
  });
}
async function run(){
  let state=fixture();
  labels(state,false);assert.equal(state.root.dataset.fullscreenSupported,'true');
  state.api.init();state.api.init();
  assert.equal(state.doc.listeners.get('fullscreenchange').length,1);
  assert.equal(state.buttons[0].listeners.get('click').length,1);
  assert.equal(state.doc.listeners.has('keydown'),false,'browser F11 must not be intercepted');
  let prevented=false;
  const enter=state.buttons[0].fire('click',{preventDefault(){prevented=true;}})[0];
  assert.equal(state.requests,1,'request must run in the original click stack');assert.equal(prevented,true);
  assert.equal(await enter,true);labels(state,true);
  assert.match(state.status.textContent,/Vollbild aktiv/);assert.equal(state.status.hidden,false);assert.equal(state.help.hidden,true);
  assert.equal(await state.buttons[1].fire('click')[0],true);assert.equal(state.exits,1);labels(state,false);
  assert.match(state.status.textContent,/Vollbild beendet/);
  state.doc.fullscreenElement=state.root;state.doc.fire('fullscreenchange');labels(state,true);
  state.doc.fullscreenElement=null;state.doc.fire('fullscreenchange');labels(state,false); // Browser Esc.
  state.window.innerHeight=state.window.screen.height;state.doc.fire('fullscreenchange');labels(state,false); // F11/viewport is not Fullscreen API state.

  state=fixture({unsupported:true});labels(state,false);
  assert.equal(state.root.dataset.fullscreenSupported,'false');assert.equal(await state.buttons[0].fire('click')[0],false);
  assert.equal(state.help.hidden,false);assert.match(state.status.textContent,/bietet hier keinen Vollbildmodus/);labels(state,false);
  state=fixture({setup(s){s.doc.fullscreenEnabled=false;}});
  assert.equal(await state.api.toggle(),false);assert.equal(state.requests,0);assert.equal(state.help.hidden,false);
  assert.match(state.status.textContent,/erlaubt hier keinen Vollbildmodus/);labels(state,false);

  state=fixture({setup(s){s.root.requestFullscreen=function(){s.requests++;return Promise.reject(new Error('Permission denied'));};}});
  assert.equal(await state.buttons[0].fire('click')[0],false);assert.equal(state.help.hidden,false);labels(state,false);
  assert.match(state.status.textContent,/konnte nicht gestartet werden/);
  assert.equal(state.buttons[0].attributes['aria-busy'],'false');
  state=fixture({setup(s){s.root.requestFullscreen=function(){throw new TypeError('Blocked');};}});
  assert.equal(await state.api.toggle(),false);assert.equal(state.help.hidden,false);labels(state,false);
  state=fixture({setup(s){s.root.requestFullscreen=function(){return Promise.resolve();};}});
  assert.equal(await state.api.toggle(),false);assert.match(state.status.textContent,/wurde nicht aktiviert/);labels(state,false);

  let resolve;
  state=fixture({setup(s){s.root.requestFullscreen=function(){s.requests++;return new Promise(done=>{resolve=done;});};}});
  const pending=state.buttons[0].fire('click')[0];assert.equal(state.requests,1);labels(state,false);
  assert.equal(await state.buttons[1].fire('click')[0],false);assert.equal(state.requests,1,'pending request must not duplicate');
  state.doc.fullscreenElement=state.root;state.doc.fire('fullscreenchange');labels(state,true);
  resolve();assert.equal(await pending,true);assert.equal(state.buttons[0].attributes['aria-busy'],'false');

  state=fixture({setup(s){
    delete s.root.requestFullscreen;delete s.doc.exitFullscreen;
    s.root.webkitRequestFullscreen=function(){assert.equal(this,s.root);s.requests++;s.doc.webkitFullscreenElement=s.root;s.doc.fire('webkitfullscreenchange');};
    s.doc.webkitExitFullscreen=function(){assert.equal(this,s.doc);s.exits++;s.doc.webkitFullscreenElement=null;s.doc.fire('webkitfullscreenchange');};
  }});
  assert.equal(await state.api.toggle(),true);labels(state,true);assert.equal(await state.api.toggle(),true);labels(state,false);
  assert.equal(state.requests,1);assert.equal(state.exits,1);
  state=fixture({setup(s){
    delete s.root.requestFullscreen;s.doc.webkitFullscreenEnabled=false;
    s.root.webkitRequestFullscreen=function(){s.requests++;};
  }});
  assert.equal(await state.api.toggle(),false);assert.equal(state.requests,0);assert.equal(state.help.hidden,false);
  state=fixture({setup(s){s.doc.exitFullscreen=function(){return Promise.reject(new Error('Denied'));};}});
  await state.api.toggle();assert.equal(await state.api.toggle(),false);labels(state,true);assert.equal(state.help.hidden,false);
  assert.match(state.status.textContent,/konnte nicht beendet werden/);
  state=fixture({setup(s){s.doc.exitFullscreen=function(){return Promise.resolve();};}});
  await state.api.toggle();assert.equal(await state.api.toggle(),false);labels(state,true);assert.equal(state.help.hidden,false);
  assert.match(state.status.textContent,/konnte nicht beendet werden/);
  state=fixture({noPanels:true});assert.equal(await state.api.toggle(),true);assert.equal(await state.api.toggle(),true);
  state=fixture({unsupported:true,noPanels:true});assert.equal(await state.api.toggle(),false);
  console.log('Fullscreen checks passed: native enter/exit, actual-state labels, direct click activation, fallback help, rejection handling, WebKit, idempotence.');
}
run().catch(error=>{console.error(error);process.exitCode=1;});
