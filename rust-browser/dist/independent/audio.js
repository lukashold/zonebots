// Original recordings are loaded from the separately provisioned private pack.
window.RadioAudio=(()=>{
 'use strict';
 const base=new URL('../original/',document.currentScript.src),levels={music:.5,effects:.7,voice:.9};
 let manifest=null,loading=null,ctx=null,effectsBus=null,voiceBus=null,unlocked=false,enabled=true,lang='de',scene='menu',world=0;
 let music=null,musicKey='',musicToken=0,voiceToken=0,activeVoice=null,speaking=false,queue=[],rotation=0;
 const buffers=new Map(),recent=new Map(),activeEffects=new Set();
 const labels={capture:['Gebiet erobert!','Territory taken!'],lost:['Gebiet verloren.','Territory lost.'],
  ready:['Verstärkung hergestellt.','Reinforcements manufactured.'],crewed:['Fahrzeug übernommen.','Vehicle captured.'],
  grenades:['Granaten eingesammelt!','Grenades collected!'],victory:['Geschafft!','We have done it!'],
  defeat:['Schlacht verloren.','Battle lost.'],start:['Einheiten bereit.','Units reporting.'],select:['Einheit meldet sich.','Unit reporting.'],
  order:['Wir sind unterwegs.','We are on our way.'],production:['Produktion gestartet.','Starting manufacture.'],cancel:['Produktion abgebrochen.','Manufacturing cancelled.'],
  repair:['Gebäude repariert.','Building repaired.'],vehicleRepaired:['Fahrzeug repariert.','Vehicle repaired.'],bridge:['Brücke repariert.','Bridge repaired.'],underAttack:['Wir werden angegriffen!','We are under attack!']};
 const infantry=new Set(['grunt','psycho','tough','sniper','pyro','laser']),guns=new Set(['gatling','cannon','howitzer','missile']);
 const reports={grunt:'ROB07',psycho:'ROB08',sniper:'ROB09',tough:'ROB10',laser:'ROB11',pyro:'ROB12'};
 const shots={grunt:'RIFLE3',sniper:'RIFLE3',psycho:'MACHGUN2',tough:'MOBIMISS',pyro:'FLAMER',laser:'LASERGUN',
  jeep:'JEEPMGUN',gatling:'GATTGUN',light:'LTANKGUN',medium:'MTANKGUN',heavy:'HTANKGUN',cannon:'MTANKGUN',howitzer:'HTANKGUN',missile:'MOBIMIS2',launcher:'MOBIMISS'};
 async function load(){
  if(!loading)loading=fetch(new URL('audio.json',base)).then(r=>r.ok?r.json():null).then(value=>{manifest=value;syncMusic();pump();return Boolean(manifest);}).catch(()=>false);
  return loading;
 }
 function boot(){
  if(!ctx){const AudioContext=window.AudioContext||window.webkitAudioContext;if(AudioContext){ctx=new AudioContext();effectsBus=ctx.createGain();effectsBus.connect(ctx.destination);voiceBus=ctx.createGain();voiceBus.connect(ctx.destination);}}
  if(ctx?.state==='suspended')Promise.resolve(ctx.resume()).catch(()=>{});
  unlocked=true;mix();load().then(()=>{syncMusic();pump();});syncMusic();pump();return ctx;
 }
 function mix(){
  const scale=enabled?1:0;if(ctx){effectsBus.gain.setTargetAtTime(levels.effects*.65*scale,ctx.currentTime,.025);voiceBus.gain.setTargetAtTime(levels.voice*.9*scale,ctx.currentTime,.025);}
  if(music)music.volume=Math.min(1,levels.music*.55*scale*(speaking?.22:1));
 }
 function musicTrack(){return scene==='battle'?(manifest?.worldMusic?.[world]||'AA16'):({menu:'ipOPTIONS16',briefing:'ipBATTLE16',victory:'ipWIN',defeat:'ipLOSE'}[scene]||'ipOPTIONS16');}
 function syncMusic(){
  if(!manifest||!unlocked)return;
  const key=musicTrack(),alternatives=manifest.music?.[key]||[];
  if(key!==musicKey){musicToken++;music?.pause();music=null;musicKey=key;if(!alternatives.length)return;
   const element=new Audio();element.preload='auto';element.loop=scene==='menu'||scene==='battle'||scene==='briefing';music=element;
   const playable=alternatives.filter(a=>!element.canPlayType||element.canPlayType(a.type));let index=0;const sources=playable.length?playable:alternatives;
   const token=musicToken;const next=()=>{if(token!==musicToken||index>=sources.length)return;element.src=new URL(sources[index++].url,base).href;mix();if(enabled)Promise.resolve(element.play()).catch(()=>{});};
   element.addEventListener('error',next);next();
  }
  mix();if(enabled&&music?.paused)Promise.resolve(music.play()).catch(()=>{});else if(!enabled)music?.pause();
 }
 async function buffer(clip){if(!ctx||!clip)return null;let promise=buffers.get(clip.url);if(!promise){promise=fetch(new URL(clip.url,base)).then(r=>{if(!r.ok)throw Error('Audio unavailable');return r.arrayBuffer();}).then(data=>ctx.decodeAudioData(data)).catch(error=>{if(buffers.get(clip.url)===promise)buffers.delete(clip.url);throw error;});buffers.set(clip.url,promise);}return promise;}
 function stopVoice(){voiceToken++;queue=[];if(activeVoice){try{activeVoice.stop();}catch{}activeVoice=null;}speaking=false;window.speechSynthesis?.cancel();mix();}
 function chosen(kind,detail){
  const type=detail.type||'';
  if(/^(ROB|COMP)\d\d$/.test(kind))return kind;
  if(kind==='select')return reports[type]||['ROB04','ROB05','ROB06'][rotation++%3];
  if(kind==='order')return detail.order==='stop'?'ROB'+String(1+rotation++%3).padStart(2,'0'):'ROB'+String(13+rotation++%12).padStart(2,'0');
  if(kind==='ready')return infantry.has(type)||!type?'COMP02':guns.has(type)?'COMP03':'COMP01';
  if(kind==='crewed')return guns.has(type)?'ROB51':'ROB52';
  return {capture:'ROB49',lost:'COMP08',grenades:'ROB53',victory:'ROB65',defeat:'COMP11',start:'ROB04',production:'COMP04',cancel:'COMP05',repair:'ROB48',vehicleRepaired:'ROB48',bridge:'ROB47',underAttack:'ROB25'}[kind];
 }
 function caption(kind,detail){if(kind==='order'&&detail.order==='stop')return lang==='de'?'Verstanden.':'Acknowledged.';if(kind==='ready')return lang==='de'?(infantry.has(detail.type)||!detail.type?'Roboter hergestellt.':guns.has(detail.type)?'Geschütz hergestellt.':'Fahrzeug hergestellt.'):(infantry.has(detail.type)||!detail.type?'Robot manufactured.':guns.has(detail.type)?'Gun manufactured.':'Vehicle manufactured.');if(kind==='select'&&detail.type){const group={grunt:'Grunts',psycho:'Psychos',sniper:'Scharfschützen',tough:'Toughs',laser:'Lasers',pyro:'Pyros'}[detail.type];return lang==='de'?(group?group+' melden sich.':'Einheit meldet sich.'):'Unit reporting.';}return labels[kind]?.[lang==='de'?0:1]||'';}
 async function pump(){
  if(speaking||!unlocked||!enabled||!queue.length||levels.voice===0)return;
  const item=queue.shift(),token=++voiceToken;speaking=true;mix();
  const finished=()=>{if(token!==voiceToken)return;activeVoice=null;speaking=false;mix();pump();};
  try{
   if(manifest){
    const clip=(manifest.voices?.[lang]||manifest.voices?.de||manifest.voices?.en)?.[item.id];
    if(!clip||!ctx){finished();return;}const decoded=await buffer(clip);if(token!==voiceToken||!enabled)return;
    const source=ctx.createBufferSource();source.buffer=decoded;source.connect(voiceBus);activeVoice=source;source.onended=finished;source.start();
   }else if(window.speechSynthesis&&typeof SpeechSynthesisUtterance!=='undefined'){
    const voice=new SpeechSynthesisUtterance(item.text);voice.lang=lang==='de'?'de-DE':'en-US';voice.volume=levels.voice;voice.onend=finished;voice.onerror=finished;window.speechSynthesis.speak(voice);
   }else finished();
  }catch{finished();}
 }
 function say(kind,detail={}){
  if(typeof detail==='string')detail={type:detail};const text=caption(kind,detail);if(text)window.dispatchEvent(new CustomEvent('radio',{detail:text}));
  const id=chosen(kind,detail);if(!id||!enabled||levels.voice===0)return;
  const key=kind==='select'||kind==='order'?kind:kind+':'+(detail.type||''),interval=kind==='select'?1300:kind==='order'?1600:2500,now=Date.now();
  if(now-(recent.get(key)??-Infinity)<interval)return;recent.set(key,now);
  if(kind==='victory'||kind==='defeat')stopVoice();
  queue.push({id,text});if(queue.length>4)queue.shift();if(!loading)load();if(manifest||loading===null)pump();else loading.then(pump);
 }
 async function effect(kind,type=''){
  if(!enabled||!unlocked||levels.effects===0||activeEffects.size>=28)return;
  const name=kind==='shot'?(shots[type]||'RIFLE3'):kind==='explosion'?(type==='large'?'ZEXPLO5':'EXP1'):kind==='click'?'CLICK1L':kind==='grenade'?'GRENADE':kind.toUpperCase();
  const now=Date.now(),key='fx:'+name;if(now-(recent.get(key)??-Infinity)<45)return;recent.set(key,now);
  const clip=manifest?.effects?.[name];if(!ctx||!clip)return;
  try{const decoded=await buffer(clip);if(!enabled||activeEffects.size>=28)return;const source=ctx.createBufferSource(),gain=ctx.createGain();source.buffer=decoded;gain.gain.value=clip.gain??1;source.connect(gain);gain.connect(effectsBus);activeEffects.add(source);source.onended=()=>{activeEffects.delete(source);gain.disconnect();};source.start();}catch{}
 }
 return {load,unlock:boot,setLanguage(value){lang=value==='en'?'en':'de';},set(key,value){if(!(key in levels)||!Number.isFinite(+value))return;levels[key]=Math.max(0,Math.min(1,+value));if(key==='voice'&&levels.voice===0)stopVoice();mix();if(key==='voice'&&levels.voice>0)pump();},
  mute(value){enabled=!value;if(value){stopVoice();for(const source of activeEffects){try{source.stop();}catch{}}activeEffects.clear();music?.pause();}else syncMusic();mix();},
  setScene(value,index=0){scene=value;world=+index||0;syncMusic();},say,effect,
  status(){return {original:Boolean(manifest),unlocked,enabled,scene,world,music:music?.src||null,musicTime:music?.currentTime||0,musicPaused:music?.paused??true,musicError:music?.error?.code||null,contextState:ctx?.state||'locked',speaking,queued:queue.length};}};
})();
