// Behavioral checks through the shipped Wasm ABI. All positions/terrain are synthetic.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),near=(a,b)=>assert(Math.abs(a-b)<1e-8,`${a} != ${b}`);
function field(){const width=64,height=48,n=width*height;return{map:{width,height,tileSize:32,world:0,seed:7291,revision:0,original:true,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),territory:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[]},sectors:[],buildings:[],crates:[],originalObjects:[],walls:[],spawns:[{type:'sniper',team:0,x:320,y:600,squad:1},{type:'grunt',team:1,x:560,y:600,squad:2},...[320,352,384].map(x=>({type:'grunt',team:0,x,y:900,squad:3})),{type:'gatling',team:1,x:1900,y:1400,squad:4}]};}
(async()=>{
 const e=(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{})).instance.exports,enc=new TextEncoder(),dec=new TextDecoder();let state;
 function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const next=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!next.error,next.error);state={...state,...next};return state;}
 req({op:'init',mission:0,difficulty:1,seed:7291,aiTeams:[],originalMission:field()});const shooter=state.units[0].id,target=state.units[1].id;
 const tick=dt=>req({op:'tick',dt,compact:true,knownMapSeed:7291,knownMapRevision:0});
 tick(.058);near(state.time,0);assert.equal(state.units[0].nativeFire,undefined);tick(.001);near(state.time,1/17);
 //44d920 scans only at its native idle search actions. Measure the firing
 //windup from actual acquisition, independently of the idle RNG sequence.
 for(let i=0;i<160&&!state.units[0].nativeFire;i++)tick(.059);
 assert(state.units[0].nativeFire,'Native idle search must eventually acquire this visible enemy');assert.equal(state.units[0].nativeFire.frame,0);
 for(let t=2;t<=12;t++){tick(.059);assert.equal(state.bullets.length,0);assert.equal(state.units.find(u=>u.id===target).hp,30);}
 tick(.059);const bullet=state.bullets.find(b=>b.team===0);assert(bullet,'Native release must serialize the traveling bullet');assert.equal(bullet.x,366);assert.equal(bullet.y,616);assert.equal(state.units.find(u=>u.id===target).hp,30,'Release is not contact');
 assert(state.events.some(e=>e.kind==='shot'&&e.team===0));
 const window={};vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),{window,setTimeout,clearTimeout,Date});
 const guest=window.MultiplayerGame.perspective(state,'guest');assert.equal(guest.bullets.find(b=>b.id===bullet.id).team,1);assert.equal(bullet.team,0,'Perspective must not mutate authority');
 for(let t=14;t<=19;t++){tick(.059);assert.equal(state.units.find(u=>u.id===target).hp,30);}
 tick(.059);assert.equal(state.units.find(u=>u.id===target).hp,20);assert(!state.bullets.some(b=>b.id===bullet.id&&b.team===0));
 const robots=state.units.filter(u=>u.squad===3).map(u=>u.id);req({op:'command',kind:'move',ids:robots,x:800,y:900});
 const expected=[[800,900],[768,868],[768,932]];robots.forEach((id,i)=>assert.deepEqual(state.units.find(u=>u.id===id).dest,expected[i]));
 req({op:'advance',steps:500,compact:true,knownMapSeed:7291,knownMapRevision:0});
 robots.forEach((id,i)=>{const u=state.units.find(u=>u.id===id);near(u.x,expected[i][0]);near(u.y,expected[i][1]);assert.equal(u.path.length,0);});
 // A supplied original strategy clock must not activate AI in PvP or aiTeams:[] fixtures.
 const m=field();m.nativeAi={decisionControl:127};m.sectors=[{id:0,x:800,y:600,owner:-1,home:false,countsForProduction:true}];
 for(const multiplayer of [false,true]){req({op:'init',mission:0,difficulty:1,seed:7291,aiTeams:[],multiplayer,originalMission:m});assert(state.units.every(u=>u.path.length===0));for(let i=0;i<8;i++)tick(.059);assert(state.units.every(u=>u.order==='guard'));}
 console.log('PASS: native59ms boundary,12-handler windup,15-pixel traveling bullet, delayed physical contact, canonical/guest projectile colors, V formation arrival, disabled AI and PvP isolation.');
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
