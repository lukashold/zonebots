//Actual WASM ABI regression for the accepted Browser opcode4 boundary.
//Usage: node tools/check-native-stop.cjs path/to/engine.wasm
const fs=require('node:fs'),assert=require('node:assert/strict'),path=require('node:path');
const wasmPath=path.resolve(process.argv[2]||path.join(__dirname,'../dist/independent/engine.wasm'));
function mission(team){const n=4096;return {map:{width:64,height:64,tileSize:32,world:0,original:true,seed:4300,revision:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),terrainCodes:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],nativeRoadGraph:{regions:[]},nativeTerrainPlane:Array(16384).fill(0)},sectors:[],buildings:[],spawns:[{type:'medium',team,x:320,y:320,squad:1},{type:'gatling',team:1-team,x:1800,y:1800,squad:2}],crates:[],originalObjects:[],walls:[],nativeWorldBuildings:Array.from({length:35},(_,nativeSlot)=>({nativeSlot,kind:-1,x:-1,y:-1,hp:-1,maxHp:-1,flags6:0,buildingId:null}))};}
(async()=>{let cases=0;
for(const team of [0,1])for(const multiplayer of [false,true])for(const repeated of [false,true]){
 const e=(await WebAssembly.instantiate(fs.readFileSync(wasmPath),{})).instance.exports,enc=new TextEncoder(),dec=new TextDecoder();let state;
 function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const r=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert.equal(r.error,undefined);state={...state,...r};return state;}
 req({op:'init',mission:0,difficulty:1,seed:4300,multiplayer,aiTeams:[],originalMission:mission(team)});const id=state.units[0].id;
 const actor=()=>state.units.find(u=>u.id===id);
 req({op:'command',kind:'move',ids:[id],team,x:1400,y:320});for(let i=0;i<10;i++)req({op:'tick',dt:.059});assert.equal(state.nativeRouteBoundary,null);const before=[actor().x,actor().y];
 req({op:'command',kind:'stop',ids:[id],team});if(repeated)req({op:'command',kind:'stop',ids:[id],team});assert.equal(actor().nativeOrder5c,4);assert.equal(actor().order,'guard');assert.equal(actor().target,null);assert.equal(actor().dest,null);assert.equal(actor().holdPosition,true);assert.equal(actor().explicitStop,true);
 for(let i=0;i<12;i++){req({op:'tick',dt:.059});assert.equal(state.nativeRouteBoundary,null,`team${team} mp${multiplayer} repeat${repeated} tick${i}`);assert.equal(actor().nativeOrder5c,0);}
 assert.notDeepEqual([actor().x,actor().y],before,'native patterned movement can continue; this is not an immediate-halt promise');cases++;
}
console.log(JSON.stringify({result:'PASS',cases,wasm:wasmPath}));
})().catch(e=>{console.error(e.stack);process.exitCode=1;});
