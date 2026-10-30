// Regression tests with a fake WebAudio/Speech backend; these do not prove speaker output.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
global.window=global;global.CustomEvent=class {constructor(type,init){this.type=type;this.detail=init.detail;}};
const captions=[],spoken=[];global.dispatchEvent=e=>captions.push(e.detail);
const param=()=>({value:0,setValueAtTime(v){this.value=v;},setTargetAtTime(v){this.value=v;},exponentialRampToValueAtTime(v){this.value=v;}});
let resumed=0;
global.AudioContext=class {
 constructor(){this.state='running';this.currentTime=0;this.sampleRate=8000;this.destination={};}
 node(){return {connect(){},start(){},stop(){},gain:param(),frequency:param(),detune:param(),Q:param()};}
 createGain(){return this.node();}createOscillator(){return this.node();}createWaveShaper(){return this.node();}createBiquadFilter(){return this.node();}createBufferSource(){return this.node();}
 createBuffer(ch,n){return {getChannelData:()=>new Float32Array(n)};}
 resume(){resumed++;this.state='running';return Promise.resolve();}
};
global.speechSynthesis={getVoices:()=>[{lang:'de-DE',name:'Test'}],cancel(){},speak(u){spoken.push(u);}};
global.SpeechSynthesisUtterance=class{constructor(text){this.text=text;}};
const base=path.resolve(__dirname,'..');
for(const f of ['data','map','zdata','game','sound'])require(path.join(base,'dist/js',f+'.js'));
const S=ZG.Synth;
S.music('battle');S.ctx().state='suspended';S.music('battle');assert.equal(resumed,1,'same-track music must resume after autoplay suspension');S.stopMusic();
S.say('ROB49','de',true);S.say('COMP08','de',true);
assert.equal(spoken.length,1,'second announcement waits rather than cancelling the first');
assert.equal(captions[0].text,'Gebiet erobert!');assert.equal(spoken[0].lang,'de-DE');
spoken[0].onend();assert.equal(spoken.length,2);assert.equal(spoken[1].text,'Gebiet verloren.');spoken[1].onend();
S.configure({speech:0});S.say('COMP02','de',true);assert.equal(spoken.length,2);assert.equal(captions.at(-1).text,'Roboter hergestellt.');
(async()=>{
 await ZRust.load(fs.readFileSync(path.join(base,'dist/engine.wasm')));
 const samples=[];
 for(let difficulty=0;difficulty<4;difficulty++){
   const g=ZG.newGame(0,{diff:difficulty,aiTeams:[0,1]});
   samples.push({rate:g.ai.aiRate,interval:g.ai.aiInt,wait:g.ai.aiWait,player:g.units.filter(u=>u.team===0).length});
   const state=ZRust.call({op:'advance',steps:12000,dt:0.1});assert(state.over,`difficulty ${difficulty} match completes`);
 }
 assert(samples[0].player>samples[1].player,'easy gives extra starting robots');
 for(let i=1;i<4;i++){assert(samples[i].rate>samples[i-1].rate);assert(samples[i].interval<samples[i-1].interval);assert(samples[i].wait<samples[i-1].wait);}
 console.log(JSON.stringify(samples));console.log('PASS: audio resume, speech queue/captions/mute, four distinct Rust/Wasm difficulties and completed matches.');
})().catch(e=>{console.error(e);process.exitCode=1;});
