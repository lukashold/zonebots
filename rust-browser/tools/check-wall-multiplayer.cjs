// Real Wasm plus two controllers: static wall geometry persists between sparse updates.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.join(__dirname,'..'),source=fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),copy=x=>JSON.parse(JSON.stringify(x));
function api(){const window={};vm.runInNewContext(source,{window,Date,setTimeout,clearTimeout});return window.MultiplayerGame;}
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{}),e=instance.exports;
 function run(v){const bytes=Buffer.from(JSON.stringify(v)),ptr=e.alloc(bytes.length);new Uint8Array(e.memory.buffer,ptr,bytes.length).set(bytes);let n;try{n=e.run(ptr,bytes.length);}finally{e.free(ptr,bytes.length);}return JSON.parse(Buffer.from(new Uint8Array(e.memory.buffer,e.output(),n)).toString());}
 const n=32*32,map={width:32,height:32,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:412,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0)};
 const scene={map,sectors:[{id:0,x:64,y:64,owner:0,home:true}],buildings:[],spawns:[{type:'heavy',team:0,x:96,y:96,squad:1},{type:'heavy',team:1,x:528,y:720,squad:2}],crates:[],originalObjects:[],walls:[{id:17,x:512,y:512,w:32,h:96,hp:90,maxHp:90}]};
 let guestRuns=0;const frames=[],toGuest=[],toHost=[],host=api().create({run}),guest=api().create({run(){guestRuns++;throw Error('Guest cannot tick');}});
 host.bind({connected:true,queuedBytes:0,send(type,payload){const message=copy({type,payload});frames.push(message);toGuest.push(message);return true;}},'host');
 guest.bind({connected:true,send(type,payload){toHost.push(copy({type,payload}));return true;}},'guest');
 function pump(){let limit=100;while(toGuest.length||toHost.length){assert.ok(limit-->0);while(toGuest.length)guest.handle(toGuest.shift());while(toHost.length)host.handle(toHost.shift());}}
 assert.equal(host.start({op:'init',originalMission:scene,seed:412},{name:'Wall replication'}),true);pump();assert.equal(guest.state.walls[0].hp,90);
 for(let i=0;i<6;i++){host.tick(.05);pump();}
 const quiet=frames.filter(m=>m.type==='game:frame').at(-1);assert.ok(quiet);assert.equal(quiet.payload.state.walls,undefined,'unchanged geometry must not consume every network frame');assert.equal(guest.state.walls[0].hp,90,'guest retains omitted walls');
 const unit=guest.viewState.units.find(u=>u.team===0&&u.type==='heavy');assert.ok(unit);
 guest.action({op:'command',kind:'attackWall',ids:[unit.id],target:17},'wall-order');pump();assert.equal(host.state.units.find(u=>u.id===unit.id).order,'attackWall');
 for(let i=0;i<400&&host.state.walls[0].hp>0;i++){host.tick(.05);pump();}
 for(let i=0;i<4;i++){host.tick(.05);pump();}
 assert.equal(host.state.walls[0].hp,0);assert.equal(guest.state.walls[0].hp,0);assert.deepEqual(copy(guest.state.walls),copy(host.state.walls));assert.deepEqual(copy(guest.state.map),copy(host.state.map));assert.ok(host.state.map.revision>0);assert.ok(host.state.walls[0].cells.every(j=>host.state.map.tiles[j]!==6));
 assert.ok(frames.some(m=>m.type==='game:frame'&&m.payload.state.walls?.some(w=>w.id===17&&w.hp===0)),'collapse must be sent before walls can be omitted again');assert.equal(guestRuns,0);host.close();guest.close();
 console.log('Wall multiplayer passed: real guest attack, sparse unchanged-wall frames, retained geometry, replicated HP/collapse/revision and opened cells; guest never runs the engine.');
})().catch(error=>{console.error(error);process.exitCode=1;});
