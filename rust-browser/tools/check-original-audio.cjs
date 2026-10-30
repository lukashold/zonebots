// Exercise real private audio declarations and the public audio lifecycle.
// Original recordings are inputs; none are embedded in this test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),privateRoot=path.join(root,'dist/original');
const manifest=JSON.parse(fs.readFileSync(path.join(privateRoot,'audio.json'),'utf8'));
for(const lang of ['de','en']){
 assert.equal(Object.keys(manifest.voices[lang]).length,95);
 for(const [name,clip] of Object.entries(manifest.voices[lang])){
  const wav=fs.readFileSync(path.join(privateRoot,clip.url));assert.equal(wav.toString('ascii',0,4),'RIFF');
  assert.equal(wav.readUInt16LE(22),1);assert.equal(wav.readUInt16LE(34),8);assert.equal(wav.readUInt32LE(24),clip.sampleRate);
  assert(Math.abs(wav.readUInt32LE(40)/clip.sampleRate-clip.duration)<.0001);
 }
}
assert.equal(Object.keys(manifest.music).length,7);assert(manifest.music.AA16.some(a=>a.type==='audio/mpeg'));
const sources=[],music=[],fetches=[],captions=[],spoken=[];
class Gain {constructor(){this.gain={value:1,setTargetAtTime:()=>{}};}connect(){}disconnect(){}}
class Source {connect(){}start(){this.started=true;sources.push(this);}stop(){this.stopped=true;this.onended?.();}}
class Context {constructor(){this.currentTime=0;this.state='running';this.destination={};}createGain(){return new Gain();}createBufferSource(){return new Source();}resume(){this.state='running';return Promise.resolve();}decodeAudioData(data){return Promise.resolve({url:data.url,duration:1});}}
class Audio {
 constructor(){this.paused=true;this.listeners={};music.push(this);}canPlayType(type){return type==='audio/mpeg'?'probably':'';}
 play(){this.paused=false;this.played=(this.played||0)+1;return Promise.resolve();}pause(){this.paused=true;}addEventListener(k,fn){this.listeners[k]=fn;}
}
const scope={URL,Audio,Date,Math,Set,Map,Promise,console,SpeechSynthesisUtterance:class{constructor(text){this.text=text;}},
 CustomEvent:class{constructor(type,options){this.type=type;this.detail=options.detail;}},
 document:{currentScript:{src:'https://private.example/independent/audio.js'}},
 fetch:async url=>{const u=new URL(url);fetches.push(u.pathname);if(u.pathname==='/original/audio.json')return {ok:true,json:async()=>manifest};
  const file=path.join(privateRoot,u.pathname.replace(/^\/original\//,''));const data=fs.readFileSync(file),array=data.buffer.slice(data.byteOffset,data.byteOffset+data.byteLength);array.url=u.pathname;return {ok:true,arrayBuffer:async()=>array};},
 dispatchEvent:event=>captions.push(event),AudioContext:Context,speechSynthesis:{cancel(){},speak(voice){spoken.push(voice);}}};
scope.window=scope;vm.createContext(scope);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/audio.js'),'utf8'),scope);
const a=scope.RadioAudio,flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
(async()=>{
 assert.equal(await a.load(),true);assert.equal(music.length,0,'Loading must not autoplay');
 a.setScene('battle',0);a.unlock();await flush();assert(music.at(-1).src.endsWith('/AA16.mp3'));assert.equal(music.at(-1).loop,true);
 a.say('select',{type:'grunt'});await flush();assert(sources.at(-1).buffer.url.endsWith('/voice/de/ROB07.wav'));
 a.say('ready',{type:'medium'});await flush();assert.equal(a.status().queued,1);
 sources.at(-1).onended();await flush();assert(sources.at(-1).buffer.url.endsWith('/voice/de/COMP01.wav'));
 a.mute(true);const count=sources.length;assert.equal(a.status().speaking,false);assert.equal(a.status().queued,0);assert(music.at(-1).paused);
 a.say('capture');await flush();assert.equal(sources.length,count,'Mute must prevent original speech');
 a.mute(false);assert(!music.at(-1).paused);a.setLanguage('en');a.say('capture');await flush();assert(sources.at(-1).buffer.url.endsWith('/voice/en/ROB49.wav'));
 sources.at(-1).onended();await flush();a.say('order',{order:'stop'});await flush();assert(/\/voice\/en\/ROB0[123]\.wav$/.test(sources.at(-1).buffer.url),'Stop must acknowledge instead of announcing movement');
 assert.equal(spoken.length,0,'An available original pack must never use synthesized speech');
 await a.effect('shot','grunt');assert(sources.at(-1).buffer.url.endsWith('/effects/RIFLE3.wav'));
 a.setScene('battle',1);assert(music.at(-1).src.endsWith('/aC16.mp3'));
 a.setScene('victory');assert(music.at(-1).src.endsWith('/ipWIN.mp3'));assert.equal(music.at(-1).loop,false);
 a.setScene('menu');assert(music.at(-1).src.endsWith('/ipOPTIONS16.mp3'));assert.equal(music.at(-1).loop,true);
 assert(captions.some(e=>e.type==='radio'&&e.detail.includes('hergestellt')));
 console.log(JSON.stringify({voices:190,effects:Object.keys(manifest.effects).length,music:7,decodedSources:sources.length,fetches:fetches.length}));
 console.log('PASS: original PCM WAVs, MP3 music, no autoplay before unlock, separate selection/manufacture, German/English clips, mute/queue cancellation, weapon effect and scene changes.');
})().catch(error=>{console.error(error);process.exitCode=1;});
