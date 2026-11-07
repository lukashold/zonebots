'use strict';
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),assert=require('node:assert/strict');
const {runReplay,validate}=require('./mission-replay.cjs');
const root=path.resolve(__dirname,'..');
function fixture(){
  const n=32*32;
  return {number:1,map:{width:32,height:32,tileSize:32,world:0,original:true,seed:41,revision:0,
    tiles:Array(n).fill(0),territory:Array.from({length:n},(_,i)=>i%32<16?0:1),terrainCodes:Array(n).fill(0),
    originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[]},
    sectors:[{id:0,x:128,y:128,owner:0,home:true,countsForProduction:true},{id:1,x:900,y:900,owner:1,home:true,countsForProduction:true}],
    buildings:[0,1].map(team=>({id:team,kind:'fort',team,sector:team,x:team?800:64,y:team?800:64,w:128,h:160,hp:2000,maxHp:2000,stars:1,product:'grunt',progress:0,remainingWork:5000})),
    spawns:[{type:'grunt',team:0,x:224,y:256,squad:1},{type:'grunt',team:1,x:736,y:736,squad:2}],crates:[],originalObjects:[]};
}
(async()=>{
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'z-replay-check-'));
  try{
    const script={version:1,mission:1,ticks:24,aiTeams:[],actions:[{tick:5,request:{op:'audioSettings',nativeAudio:{soundEnabled:false}}}]};
    const options={script,campaign:{missions:[fixture()]},wasm:process.env.REPLAY_ENGINE_WASM||path.join(root,'dist/independent/engine.wasm')};
    const original=path.join(dir,'reference.jsonl');
    const recorded=await runReplay({...options,output:original});
    assert.equal(recorded.tick,24);assert.equal(recorded.observations,25);assert.equal(recorded.fullParity,false);
    const repeated=await runReplay({...options,reference:original});
    assert.equal(repeated.compared,true);assert.equal(repeated.referenceSource,'remake');assert.equal(repeated.actionsApplied,1);
    const rows=fs.readFileSync(original,'utf8').trimEnd().split('\n').map(JSON.parse);
    const corrupt=structuredClone(rows);corrupt[6].state.rng[0]^=1;
    const ref=path.join(dir,'corrupt.jsonl'),out=path.join(dir,'failed.jsonl');
    fs.writeFileSync(ref,corrupt.map(JSON.stringify).join('\n')+'\n');
    await assert.rejects(runReplay({...options,reference:ref,output:out}),error=>error.difference?.tick===5&&error.difference.path==='$.state.rng.0');
    assert(!fs.existsSync(out));assert(fs.existsSync(out+'.partial'));
    fs.writeFileSync(ref,rows.slice(0,-1).map(JSON.stringify).join('\n')+'\n');
    await assert.rejects(runReplay({...options,reference:ref}),/truncated/);
    fs.writeFileSync(ref,rows.concat(rows.at(-1)).map(JSON.stringify).join('\n')+'\n');
    await assert.rejects(runReplay({...options,reference:ref}),/trailing records/);
    await assert.rejects(runReplay({...options,reference:original,script:{...script,ticks:25}}),/header mismatch/);
    await assert.rejects(runReplay({...options,output:original}),/already exists/);
    assert.throws(()=>validate({...script,actions:[{tick:1,request:{op:'advance',steps:10}}]}),/cannot initialize or advance/);
    assert.throws(()=>validate({...script,actions:[{tick:25,request:{op:'command'}}]}),/ordered ticks/);
    console.log('Replay checks passed: isolated instances, exact callbacks, first RNG mismatch, truncated/extra reference rejection, input identity and non-overwriting output. No original-match claim.');
  }finally{
    const base=path.resolve(os.tmpdir())+path.sep;
    assert(path.resolve(dir).startsWith(base)&&path.basename(dir).startsWith('z-replay-check-'));
    fs.rmSync(dir,{recursive:true});
  }
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
