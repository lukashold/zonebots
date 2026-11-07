'use strict';
// Local deterministic input runner. Observed native checkpoints are a subset
// of the game state. A remake self-comparison is never an original-game oracle.
const fs=require('node:fs'),path=require('node:path'),readline=require('node:readline');
const {createHash}=require('node:crypto');
const SCHEMA='z-mission-replay-v1',CHECKPOINT='z-native-checkpoint-v1';
const hash=x=>createHash('sha256').update(x).digest('hex');
function canonical(value){
  if(Array.isArray(value))return '['+value.map(canonical).join(',')+']';
  if(value&&typeof value==='object')return '{'+Object.keys(value).sort().map(k=>JSON.stringify(k)+':'+canonical(value[k])).join(',')+'}';
  return JSON.stringify(value);
}
function difference(a,b,at='$'){
  if(Object.is(a,b))return null;
  if(a===null||b===null||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return {path:at,actual:a,expected:b};
  const keys=[...new Set([...Object.keys(a),...Object.keys(b)])].sort();
  for(const k of keys){const d=difference(a[k],b[k],at+'.'+k);if(d)return d;}
  return null;
}
function validate(script){
  if(script.version!==1||!Number.isInteger(script.mission)||script.mission<1||script.mission>20)throw Error('Expected version 1 and mission 1..20');
  if(!Number.isInteger(script.ticks)||script.ticks<1||script.ticks>1000000)throw Error('ticks must be 1..1000000');
  if(script.difficulty!==undefined&&(!Number.isInteger(script.difficulty)||script.difficulty<0||script.difficulty>3))throw Error('difficulty must be 0..3');
  if(script.aiTeams!==undefined&&(!Array.isArray(script.aiTeams)||script.aiTeams.some(n=>n!==0&&n!==1)))throw Error('Invalid AI teams');
  if(!Array.isArray(script.actions))throw Error('Expected actions array');
  let previous=1;
  for(const a of script.actions){
    if(!Number.isInteger(a.tick)||a.tick<previous||a.tick>script.ticks)throw Error('Actions must have ordered ticks within the replay');
    previous=a.tick;
    if(!a.request||!['command','produce','placeGun','audioSettings'].includes(a.request.op))throw Error('Replay actions cannot initialize or advance the clock');
  }
}
async function runReplay({script,campaign,wasm,output,reference,onProgress}){
  validate(script);
  const mission=campaign.missions.find(m=>m.number===script.mission);
  if(!mission)throw Error('Mission absent from supplied campaign');
  const binary=fs.readFileSync(wasm);
  const header={type:'header',schema:SCHEMA,checkpointSchema:CHECKPOINT,source:'remake',
    wasmSha256:hash(binary),mission:script.mission,missionSha256:hash(canonical(mission)),
    inputSha256:hash(canonical(script)),requestedTicks:script.ticks,
    scope:'RNG, native clocks, active UNIT fields, path pool/mask/scratch hashes; not a complete process image'};
  let fd,stream,reader,iter,referenceHeader,partial;
  const emit=row=>{if(fd!==undefined)fs.writeSync(fd,JSON.stringify(row)+'\n');};
  const nextReference=async()=>{
    const r=await iter.next();if(r.done)throw Error('Reference truncated before its terminal record');
    return JSON.parse(r.value);
  };
  try{
    if(reference){
      stream=fs.createReadStream(reference,{encoding:'utf8'});
      reader=readline.createInterface({input:stream,crlfDelay:Infinity});iter=reader[Symbol.asyncIterator]();
      referenceHeader=await nextReference();
      for(const key of ['type','schema','checkpointSchema','mission','missionSha256','inputSha256','requestedTicks','scope']){
        if(referenceHeader[key]!==header[key])throw Error('Reference header mismatch: '+key);
      }
      if(!['remake','gog-executable'].includes(referenceHeader.source))throw Error('Unknown reference provenance');
      if(referenceHeader.source==='gog-executable'&&referenceHeader.executableSha256!=='ed5b16c486824c0761235dbdffcb3cda7e9f9df6188186aef7f662981a44e43e')throw Error('Unexpected original executable hash');
    }
    if(output){
      output=path.resolve(output);partial=output+'.partial';
      if(fs.existsSync(output))throw Error('Output already exists: '+output);
      if(reference&&path.resolve(reference).toLowerCase()===output.toLowerCase())throw Error('Output cannot replace reference');
      fd=fs.openSync(partial,'wx');emit(header);
    }
    const {instance}=await WebAssembly.instantiate(binary,{}),e=instance.exports;
    const encoder=new TextEncoder(),decoder=new TextDecoder();
    function request(v){
      const bytes=encoder.encode(JSON.stringify(v)),p=e.alloc(bytes.length);let n;
      try{new Uint8Array(e.memory.buffer,p,bytes.length).set(bytes);n=e.run(p,bytes.length);}finally{e.free(p,bytes.length);}
      const result=JSON.parse(decoder.decode(new Uint8Array(e.memory.buffer,e.output(),n)));
      if(result.error)throw Error(result.error);return result;
    }
    let snapshot=request({op:'init',mission:script.mission-1,difficulty:script.difficulty??1,seed:7163,
      aiTeams:script.aiTeams??[1],originalMission:mission,nativeCalendar:{month:1,day:1}});
    const compact={compact:true,knownMapSeed:snapshot.map.seed,knownMapRevision:snapshot.map.revision};
    let action=0,observations=0,tick=0;
    for(;tick<=script.ticks;tick++){
      if(tick>0){
        while(action<script.actions.length&&script.actions[action].tick===tick){request({...compact,...script.actions[action++].request});}
        snapshot=request({op:'tick',dt:.059,...compact});
      }
      const state=request({op:'replayCheckpoint'});
      if(state.schema!==CHECKPOINT)throw Error('Wasm lacks native replay checkpoints');
      if(state.tick!==tick)throw Error('Native callback count diverged at tick '+tick);
      const row={type:'checkpoint',tick,state};emit(row);observations++;
      if(state.boundary!==null)throw Error('Native boundary at tick '+tick+': '+state.boundary);
      if(reference){
        const expected=await nextReference(),d=difference(row,expected);
        if(d){const error=Error('First replay difference at tick '+tick+' '+d.path);error.difference={tick,...d};throw error;}
      }
      if(onProgress&&tick>0&&tick%1700===0)onProgress({tick,requested:script.ticks});
      if(state.winner!==null||tick===script.ticks)break;
    }
    const end={type:'end',tick,observations,actionsApplied:action,winner:snapshot.winner};
    if(reference){
      const expected=await nextReference(),d=difference(end,expected);if(d)throw Error('Reference terminal mismatch at '+d.path);
      if(!(await iter.next()).done)throw Error('Reference has trailing records');
    }
    emit(end);if(fd!==undefined){fs.closeSync(fd);fd=undefined;fs.renameSync(partial,output);}
    return {...end,referenceSource:referenceHeader?.source??null,compared:!!reference,fullParity:false};
  }finally{
    if(fd!==undefined)fs.closeSync(fd);
    if(reader)reader.close();if(stream)stream.destroy();
    // Keep a failed partial trace to diagnose the first mismatch. Never promote it.
  }
}
module.exports={runReplay,validate,difference,canonical};
if(require.main===module){
  const args=process.argv.slice(2),root=path.resolve(__dirname,'..');
  function arg(name,fallback){const i=args.indexOf(name);if(i<0)return fallback;if(!args[i+1]||args[i+1].startsWith('--'))throw Error(name+' needs a value');return args[i+1];}
  (async()=>{
    const script=JSON.parse(fs.readFileSync(arg('--script'),'utf8'));
    const campaign=JSON.parse(fs.readFileSync(arg('--campaign',path.join(root,'dist/original/campaign.json')),'utf8'));
    const result=await runReplay({script,campaign,wasm:arg('--wasm',path.join(root,'dist/independent/engine.wasm')),
      output:arg('--out'),reference:arg('--reference'),onProgress:p=>process.stderr.write(JSON.stringify(p)+'\n')});
    console.log(JSON.stringify(result));
  })().catch(error=>{console.error(JSON.stringify({error:error.message,difference:error.difference}));process.exitCode=1;});
}
