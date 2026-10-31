'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.join(__dirname,'..'),source=fs.readFileSync(path.join(root,'dist/independent/multiplayer-ui.js'),'utf8');
function fixture(){
 const nodes={},events={},statuses=[],game={opened:[],started:[],handled:[],left:0},sessions=[];
 const ids=['mpLobby','mpMission','mpStatus','mpHost','mpJoin','mpName','mpMode','mpTurnUrl','mpTurnUser','mpTurnPassword','mpTurn','mpRoleButtons','mpLocalSection','mpLocalCode','mpRemoteSection','mpRemoteCode','mpAccept','mpReady','mpStart','mpCancel','mpRemoteLabel','mpLocalLabel','mpReadyText','mpCopy'];
 function element(id){return {id,value:'',options:[],hidden:false,disabled:false,textContent:'',dataset:{},listeners:{},append(value){this.options.push(value);if(this.options.length===1)this.value=value.value;},replaceChildren(){this.options=[];},addEventListener(name,fn){this.listeners[name]=fn;},async fire(name='click'){return this.listeners[name]?.();},focus(){this.focused=true;},select(){this.selected=true;}};}
 for(const id of ids)nodes[id]=element(id);nodes.mpName.value='Player';nodes.mpMode.value='online';
 const bridge={ready:false,missions:[],network:{handle(value){game.handled.push(value);},handleTransportStatus(value){statuses.push(value);}},openMultiplayer(session,role){game.opened.push([session,role]);},startMultiplayer(mission){game.started.push(mission);},leaveMultiplayer(){game.left++;}};
 const Multiplayer={BUILD:'test-rules',supported:true,defaultIceServers:[],decodeCode(code){if(code!=='INVITE')throw Error('invalid code');return{};},createSession(settings){
  const session={connected:false,phase:'idle',role:null,remoteName:'Opponent',settings,
   async host(mode){this.mode=mode;this.role='host';this.phase='waiting';settings.onStatus({phase:'waiting',message:'Invite ready'});return 'OFFER';},
   async join(code,mode){assert.equal(code,'INVITE');this.mode=mode;this.role='guest';this.phase='waiting';return 'ANSWER';},
   async accept(code){assert.equal(code,'ANSWER');this.connected=true;this.phase='connected';settings.onStatus({phase:'connected',connected:true,remoteName:'Opponent',latencyMs:10,message:'Connected'});},
   close(){this.phase='closed';this.connected=false;settings.onStatus({phase:'closed',message:'Closed'});}};sessions.push(session);return session;}};
 const window={GameBridge:bridge,Multiplayer,navigator:{clipboard:{async writeText(value){game.copied=value;}}},addEventListener(name,fn){events[name]=fn;},dispatchEvent(event){game.event=event;}};
 vm.runInNewContext(source,{window,document:{getElementById:id=>nodes[id],createElement:()=>element('option')},CustomEvent:class{constructor(type,init){this.type=type;this.detail=init.detail;}}});
 return {nodes,bridge,events,game,sessions,api:window.MultiplayerUI};
}
const settle=()=>new Promise(resolve=>setImmediate(resolve));
async function run(){
 const f=fixture(),n=f.nodes;assert.equal(n.mpHost.disabled,true);f.bridge.ready=true;f.bridge.missions=[{name:'First'},{name:'Second'}];f.events['game-ready']();assert.equal(n.mpHost.disabled,false);assert.equal(n.mpMission.options.length,2);
 await n.mpJoin.fire();await settle();assert.match(n.mpStatus.textContent,/Einladung/);assert.equal(f.sessions.length,0);
 n.mpMode.value='lan';await n.mpMode.fire('change');assert.equal(n.mpTurn.hidden,true);await n.mpHost.fire();await settle();assert.equal(n.mpLocalCode.value,'OFFER');assert.equal(f.sessions[0].mode.lan,true);assert.equal(f.game.opened[0][1],'host');assert.equal(n.mpAccept.hidden,false);assert.equal(n.mpStart.disabled,true);
 await n.mpCopy.fire();assert.equal(f.game.copied,'OFFER');n.mpRemoteCode.value='ANSWER';await n.mpRemoteCode.fire('input');await n.mpAccept.fire();await settle();assert.equal(n.mpStart.disabled,false);assert.equal(n.mpRemoteSection.hidden,true);assert.equal(n.mpReady.hidden,false);
 n.mpMission.value='1';await n.mpStart.fire();await settle();assert.deepEqual(f.game.started,[1]);assert.equal(f.game.event.type,'multiplayer-status');
 await n.mpCancel.fire();assert.equal(f.game.left,1);assert.equal(n.mpLocalCode.value,'');assert.equal(n.mpRemoteCode.value,'');assert.equal(n.mpHost.disabled,false);
 n.mpRemoteCode.value='INVITE';await n.mpJoin.fire();await settle();assert.equal(n.mpLocalCode.value,'ANSWER');assert.equal(f.game.opened.at(-1)[1],'guest');assert.equal(n.mpStart.hidden,true);assert.match(n.mpLocalLabel.textContent,/Antwort/);
 f.sessions.at(-1).settings.onMessage({type:'frame',payload:{frame:1}});assert.equal(f.game.handled.length,1);const priorLeaves=f.game.left;f.api.detachSession();assert.equal(f.game.left,priorLeaves,'bridge-driven cleanup must not recursively leave multiplayer');assert.equal(n.mpLocalCode.value,'');assert.equal(f.api.session,null);
 for(const file of ['dist/index.html','dist/independent/index.html']){
  const html=fs.readFileSync(path.join(root,file),'utf8'),ids=[...html.matchAll(/\bid="([^"]+)"/g)].map(m=>m[1]);assert.equal(ids.length,new Set(ids).size);for(const id of ['mpLobby','mpName','mpMode','mpLocalCode','mpRemoteCode','mpHost','mpJoin','mpAccept','mpStart','mpMission','mpCancel'])assert.ok(ids.includes(id));
  assert.match(html,/data-menu-page="multiplayer"/);assert.ok(html.indexOf('multiplayer.js')<html.indexOf('app.js'));assert.ok(html.indexOf('multiplayer-game.js')<html.indexOf('app.js'));assert.ok(html.indexOf('multiplayer-ui.js')>html.indexOf('app.js'));
 }
 console.log('Multiplayer UI passed: loading gate, invitation validation, host and guest workflows, LAN choice, copy, connected-only start, mission selection, bridge forwarding, reset, script order and both entrypoints.');
}
run().catch(error=>{console.error(error);process.exitCode=1;});
