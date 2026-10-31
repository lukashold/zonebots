/* Two-player encrypted WebRTC transport. Game authority stays with the host.
 * ICE/signaling: https://webrtc.org/getting-started/peer-connections
 * Data channels: https://webrtc.org/getting-started/data-channels
 * No game assets, session codes or TURN credentials are stored by this module. */
(function(global){
 'use strict';
 const PROTOCOL=1,BUILD='zonebots-mp-2-walls',PREFIX='ZMP1.',MAGIC=0x5a4d5031;
 const CHUNK_BYTES=12*1024,HEADER_BYTES=16,MAX_MESSAGE_BYTES=4*1024*1024,MAX_QUEUE_BYTES=8*1024*1024;
 const HIGH_WATER=256*1024,LOW_WATER=64*1024,HEARTBEAT_MS=2000,TIMEOUT_MS=15000;
 const defaultIceServers=Object.freeze([{urls:'stun:stun.l.google.com:19302'}]);
 const encoder=new TextEncoder(),decoder=new TextDecoder();
 function token(){const bytes=new Uint8Array(16);if(global.crypto?.getRandomValues)global.crypto.getRandomValues(bytes);else for(let i=0;i<bytes.length;i++)bytes[i]=Math.floor(Math.random()*256);return Array.from(bytes,b=>b.toString(16).padStart(2,'0')).join('');}
 function base64(bytes){let result='';for(let i=0;i<bytes.length;i+=4096)result+=String.fromCharCode(...bytes.subarray(i,i+4096));return btoa(result).replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'');}
 function decodeCode(code){
  const value=String(code||'').replace(/\s/g,'');if(value.length>100000||!value.startsWith(PREFIX))throw Error('Ungültiger Verbindungscode. Bitte den vollständigen Code einfügen.');
  try{const encoded=value.slice(PREFIX.length),bytes=Uint8Array.from(atob(encoded.replace(/-/g,'+').replace(/_/g,'/')),c=>c.charCodeAt(0)),data=JSON.parse(decoder.decode(bytes));
   if(data.v!==PROTOCOL||!['host','guest'].includes(data.role)||typeof data.session!=='string'||data.session.length!==32||typeof data.build!=='string'||!['offer','answer'].includes(data.description?.type)||typeof data.description.sdp!=='string'||data.description.sdp.length>65000)throw Error();return data;
  }catch{throw Error('Der Verbindungscode ist unvollständig oder verwendet eine andere Spielversion.');}
 }
 function createSession(options={}){
  let pc=null,channel=null,role=null,session=token(),closed=false,phase='idle',remoteName='',latencyMs=null;
  let handshake=false,lastSeen=Date.now(),lastPing=0,heartbeat=null,disconnectTimer=null,gatherCancel=null;
  let queue=[],active=null,pendingBytes=0,nextId=1,receiveChain=Promise.resolve(),incoming=new Map();
  const build=String(options.build||BUILD),name=String(options.name||'Spieler').trim().slice(0,32)||'Spieler';
  const status=(next,message='')=>{phase=next;options.onStatus?.({phase,role,connected:phase==='connected'&&handshake,remoteName,latencyMs,message});};
  function fail(message){if(closed)return;status('failed',message);dispose();}
  function dispose(){
   closed=true;clearInterval(heartbeat);clearTimeout(disconnectTimer);gatherCancel?.();gatherCancel=null;
   queue=[];active=null;pendingBytes=0;incoming.clear();
   if(channel){channel.onclose=channel.onerror=channel.onmessage=channel.onopen=channel.onbufferedamountlow=null;try{channel.close();}catch{}}
   if(pc){pc.onconnectionstatechange=pc.oniceconnectionstatechange=pc.ondatachannel=null;try{pc.close();}catch{}}
   handshake=false;
  }
  function close(){if(closed)return;status('closed','Verbindung beendet.');dispose();}
  function envelope(type,payload){return encoder.encode(JSON.stringify({v:PROTOCOL,type,payload}));}
  function enqueue(type,payload,replaceKey='',internal=false){
   if(closed||!channel||channel.readyState!=='open'||(!internal&&!handshake))return false;
   let bytes;try{bytes=envelope(type,payload);}catch{return false;}
   if(bytes.length>MAX_MESSAGE_BYTES)return false;
   if(replaceKey){for(let i=queue.length-1;i>=0;i--)if(queue[i].replaceKey===replaceKey){pendingBytes-=queue[i].bytes.length;queue.splice(i,1);}}
   if(pendingBytes+bytes.length>MAX_QUEUE_BYTES)return false;
   queue.push({id:nextId++>>>0,bytes,offset:0,part:0,total:Math.ceil(bytes.length/CHUNK_BYTES),replaceKey});pendingBytes+=bytes.length;
   flush();return true;
  }
  function flush(){
   if(closed||channel?.readyState!=='open')return;
   try{
    while(channel.bufferedAmount<HIGH_WATER){
     if(!active)active=queue.shift();if(!active)break;
     const count=Math.min(CHUNK_BYTES,active.bytes.length-active.offset),buffer=new ArrayBuffer(HEADER_BYTES+count),view=new DataView(buffer);
     view.setUint32(0,MAGIC);view.setUint32(4,active.id);view.setUint16(8,active.part);view.setUint16(10,active.total);view.setUint32(12,active.bytes.length);
     new Uint8Array(buffer,HEADER_BYTES).set(active.bytes.subarray(active.offset,active.offset+count));
     channel.send(buffer);active.offset+=count;active.part++;
     if(active.offset===active.bytes.length){pendingBytes-=active.bytes.length;active=null;}
    }
   }catch(error){fail('Die Datenverbindung konnte nicht senden. Bitte die Partie neu verbinden.');}
  }
  function delivered(bytes){
   let message;try{message=JSON.parse(decoder.decode(bytes));}catch{fail('Ungültige Spieldaten empfangen.');return;}
   if(message.v!==PROTOCOL||typeof message.type!=='string'||message.type.length>40){fail('Die Netzwerkversionen passen nicht zusammen.');return;}
   if(message.type==='$hello'){
    const hello=message.payload;if(hello?.build!==build||hello?.protocol!==PROTOCOL||hello?.session!==session||hello?.role===role||!['host','guest'].includes(hello?.role)){fail('Die Spielversionen passen nicht zusammen. Beide Seiten bitte neu laden.');return;}
    if(handshake)return;remoteName=String(hello.name||'Mitspieler').slice(0,32);handshake=true;lastSeen=Date.now();status('connected','Mitspieler verbunden.');return;
   }
   if(!handshake){fail('Die Gegenstelle hat den Spielstart nicht bestätigt.');return;}
   if(message.type==='$ping'){enqueue('$pong',message.payload,'',true);return;}
   if(message.type==='$pong'){if(Number.isFinite(message.payload?.at)){latencyMs=Math.max(0,Date.now()-message.payload.at);status('connected');}return;}
   if(message.type.startsWith('$'))return;
   options.onMessage?.({type:message.type,payload:message.payload});
  }
  async function received(data){
   if(closed)return;if(data&&typeof data.arrayBuffer==='function')data=await data.arrayBuffer();
   if(closed)return;const buffer=ArrayBuffer.isView(data)?data.buffer.slice(data.byteOffset,data.byteOffset+data.byteLength):data;
   if(!buffer||typeof buffer.byteLength!=='number'||buffer.byteLength<HEADER_BYTES||buffer.byteLength>CHUNK_BYTES+HEADER_BYTES){fail('Ungültiges Netzwerkpaket.');return;}
   const view=new DataView(buffer);if(view.getUint32(0)!==MAGIC){fail('Falsches Netzwerkprotokoll.');return;}
   const id=view.getUint32(4),part=view.getUint16(8),total=view.getUint16(10),size=view.getUint32(12),length=buffer.byteLength-HEADER_BYTES;
   if(size===0||size>MAX_MESSAGE_BYTES||total!==Math.ceil(size/CHUNK_BYTES)||part>=total||length!==Math.min(CHUNK_BYTES,size-part*CHUNK_BYTES)){fail('Das Netzwerkpaket überschreitet die erlaubte Größe.');return;}
   let entry=incoming.get(id);
   if(!entry){if(part!==0||incoming.size>=2){fail('Die Netzwerkpakete sind unvollständig.');return;}entry={bytes:new Uint8Array(size),size,total,next:0};incoming.set(id,entry);}
   if(entry.total!==total||entry.size!==size||entry.next!==part){fail('Die Reihenfolge der Netzwerkpakete stimmt nicht.');return;}
   entry.bytes.set(new Uint8Array(buffer,HEADER_BYTES),part*CHUNK_BYTES);entry.next++;lastSeen=Date.now();
   if(entry.next===entry.total){incoming.delete(id);delivered(entry.bytes);}
  }
  function attach(nextChannel){
   if(channel&&channel!==nextChannel){try{nextChannel.close();}catch{}return;}channel=nextChannel;
   let opened=false;channel.binaryType='arraybuffer';channel.bufferedAmountLowThreshold=LOW_WATER;
   channel.onbufferedamountlow=flush;
   channel.onmessage=event=>{receiveChain=receiveChain.then(()=>received(event.data)).catch(()=>fail('Die Spieldaten konnten nicht gelesen werden.'));};
   channel.onerror=()=>fail('Die Datenverbindung ist fehlgeschlagen. Bitte erneut verbinden.');
   channel.onclose=()=>{if(!closed){status('disconnected','Mitspieler hat die Verbindung getrennt.');dispose();}};
   channel.onopen=()=>{
    if(opened||closed)return;opened=true;status('connecting','Spielversion wird geprüft …');lastSeen=Date.now();
    enqueue('$hello',{protocol:PROTOCOL,build,name,role,session},'',true);
    heartbeat=setInterval(()=>{
     if(closed)return;const now=Date.now();if(now-lastSeen>TIMEOUT_MS){fail('Mitspieler antwortet nicht mehr. Die Partie wurde angehalten.');return;}
     if(handshake&&now-lastPing>=HEARTBEAT_MS){lastPing=now;enqueue('$ping',{at:now},'ping',true);}flush();
    },500);
   };
   if(channel.readyState==='open')channel.onopen();
  }
  function peer(lan){
   if(closed)throw Error('Diese Verbindung ist beendet. Bitte eine neue Partie erstellen.');
   if(pc)throw Error('Es besteht bereits eine Verbindung. Bitte zuerst trennen.');
   if(!global.RTCPeerConnection)throw Error('Dieser Browser unterstützt keinen WebRTC-Multiplayer. Bitte einen aktuellen Browser verwenden.');
   const iceServers=lan?[]:Array.isArray(options.iceServers)?options.iceServers:defaultIceServers;
   pc=new global.RTCPeerConnection({iceServers,iceCandidatePoolSize:0});
   pc.ondatachannel=event=>{if(event.channel.label!=='zonebots-'+PROTOCOL){event.channel.close();return;}attach(event.channel);};
   pc.onconnectionstatechange=()=>{
    if(closed)return;const state=pc.connectionState;
    if(state==='failed'){fail('Keine direkte Verbindung möglich. Im Internet kann ein TURN-Server oder ein anderes Netzwerk erforderlich sein.');return;}
    if(state==='disconnected'){
     status('reconnecting','Verbindung unterbrochen. Es wird auf den Mitspieler gewartet …');
     clearTimeout(disconnectTimer);disconnectTimer=setTimeout(()=>fail('Verbindung verloren. Bitte eine neue Verbindung herstellen.'),10000);
    }else if(state==='connected'){clearTimeout(disconnectTimer);disconnectTimer=null;if(handshake)status('connected','Verbindung wiederhergestellt.');}
    else if(state==='closed'&&!closed){status('disconnected','Verbindung geschlossen.');dispose();}
   };
   return pc;
  }
  function waitForIce(){
   if(pc.iceGatheringState==='complete')return Promise.resolve();
   return new Promise((resolve,reject)=>{
    const timeout=setTimeout(done,Math.max(1000,Math.min(30000,options.gatherTimeoutMs||12000)));
    function done(){clearTimeout(timeout);pc?.removeEventListener('icegatheringstatechange',changed);gatherCancel=null;closed?reject(Error('Verbindung abgebrochen.')):resolve();}
    function changed(){if(pc.iceGatheringState==='complete')done();}
    gatherCancel=done;pc.addEventListener('icegatheringstatechange',changed);
   });
  }
  function code(){return PREFIX+base64(encoder.encode(JSON.stringify({v:PROTOCOL,build,role,session,description:{type:pc.localDescription.type,sdp:pc.localDescription.sdp}})));}
  async function host(settings={}){
   role='host';const connection=peer(Boolean(settings.lan));status('gathering','Einladung wird erstellt …');attach(connection.createDataChannel('zonebots-'+PROTOCOL,{ordered:true}));
   await connection.setLocalDescription(await connection.createOffer());await waitForIce();status('waiting','Einladung bereit. Antwortcode des Mitspielers einfügen.');return code();
  }
  async function join(invitation,settings={}){
   const offer=decodeCode(invitation);if(offer.role!=='host'||offer.description.type!=='offer')throw Error('Zum Beitreten wird der Einladungscode des Gastgebers benötigt.');if(offer.build!==build)throw Error('Die Spielversionen passen nicht zusammen. Beide Seiten bitte neu laden.');
   role='guest';session=offer.session;const connection=peer(Boolean(settings.lan));status('gathering','Antwort wird erstellt …');await connection.setRemoteDescription(offer.description);await connection.setLocalDescription(await connection.createAnswer());await waitForIce();if(!handshake)status('waiting','Antwortcode bereit. Bitte an den Gastgeber zurückgeben.');return code();
  }
  async function accept(answer){
   if(role!=='host'||!pc||closed)throw Error('Zuerst eine neue Partie als Gastgeber erstellen.');
   const data=decodeCode(answer);if(data.role!=='guest'||data.description.type!=='answer'||data.session!==session)throw Error('Dieser Antwortcode gehört zu einer anderen Einladung.');if(data.build!==build)throw Error('Die Spielversionen passen nicht zusammen.');
   status('connecting','Verbindung zum Mitspieler wird aufgebaut …');await pc.setRemoteDescription(data.description);return true;
  }
  return Object.freeze({host,join,accept,close,send(type,payload,settings={}){if(typeof type!=='string'||!type||type.startsWith('$')||type.length>40)return false;return enqueue(type,payload,String(settings.replaceKey||''));},get connected(){return !closed&&handshake&&phase==='connected';},get role(){return role;},get phase(){return phase;},get latencyMs(){return latencyMs;},get remoteName(){return remoteName;},get queuedBytes(){return pendingBytes;}});
 }
 global.Multiplayer=Object.freeze({PROTOCOL,BUILD,createSession,decodeCode,get supported(){return Boolean(global.RTCPeerConnection);},defaultIceServers});
})(window);
