'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(path.join(__dirname,'../dist/independent/multiplayer.js'),'utf8');
const peers=new Map();let nextPeer=0,maxPacket=0;
class Channel{
 constructor(label){this.label=label;this.readyState='connecting';this.bufferedAmount=0;this.bufferedAmountLowThreshold=0;this.frames=[];this.paused=false;}
 send(frame){assert.equal(this.readyState,'open');assert.ok(frame.byteLength<=12*1024+16);maxPacket=Math.max(maxPacket,frame.byteLength);this.frames.push(frame.slice(0));this.bufferedAmount+=frame.byteLength;
  if(this.paused)return;queueMicrotask(()=>{this.bufferedAmount=Math.max(0,this.bufferedAmount-frame.byteLength);this.other?.onmessage?.({data:frame.slice(0)});if(this.bufferedAmount<=this.bufferedAmountLowThreshold)this.onbufferedamountlow?.();});}
 close(){if(this.readyState==='closed')return;this.readyState='closed';this.onclose?.();if(this.other&&this.other.readyState!=='closed'){this.other.readyState='closed';this.other.onclose?.();}}
}
class Peer{
 constructor(configuration){this.id=String(++nextPeer);this.configuration=configuration;this.iceGatheringState='complete';this.connectionState='new';peers.set(this.id,this);this.events={};}
 createDataChannel(label){return this.channel=new Channel(label);}
 async createOffer(){return {type:'offer',sdp:this.id};}
 async createAnswer(){return {type:'answer',sdp:this.id};}
 async setLocalDescription(value){this.localDescription=value;}
 async setRemoteDescription(value){this.remote=value;if(value.type==='answer'){
  const guest=peers.get(value.sdp),other=new Channel(this.channel.label);this.channel.other=other;other.other=this.channel;guest.channel=other;guest.ondatachannel?.({channel:other});
  this.connectionState=guest.connectionState='connected';this.onconnectionstatechange?.();guest.onconnectionstatechange?.();
  this.channel.readyState=other.readyState='open';this.channel.onopen?.();other.onopen?.();
 }}
 addEventListener(name,fn){this.events[name]=fn;}
 removeEventListener(name){delete this.events[name];}
 close(){this.connectionState='closed';}
}
const context={window:{RTCPeerConnection:Peer,crypto:require('node:crypto').webcrypto},TextEncoder,TextDecoder,ArrayBuffer,DataView,Uint8Array,Date,Math,JSON,Error,setTimeout,clearTimeout,setInterval,clearInterval,
 btoa:s=>Buffer.from(s,'binary').toString('base64'),atob:s=>Buffer.from(s,'base64').toString('binary')};
vm.runInNewContext(source,context);const api=context.window.Multiplayer;
const settle=async()=>{for(let i=0;i<20;i++)await new Promise(resolve=>setImmediate(resolve));};
async function pair(settings={}){
 const hostMessages=[],guestMessages=[],hostStatus=[],guestStatus=[];
 const host=api.createSession({name:'Host',onMessage:m=>hostMessages.push(m),onStatus:s=>hostStatus.push(s),...settings});
 const guest=api.createSession({name:'Guest',onMessage:m=>guestMessages.push(m),onStatus:s=>guestStatus.push(s),...settings});
 const invitation=await host.host({lan:true}),answer=await guest.join(invitation,{lan:true});await host.accept(answer);await settle();
 return {host,guest,hostMessages,guestMessages,hostStatus,guestStatus,invitation,answer};
}
async function run(){
 assert.equal(api.PROTOCOL,1);assert.equal(api.supported,true);assert.throws(()=>api.decodeCode('hello'));
 let p=await pair();assert.equal(p.host.connected,true);assert.equal(p.guest.connected,true);assert.equal(p.host.remoteName,'Guest');assert.equal(p.guest.remoteName,'Host');
 const connected=p.hostStatus.find(s=>s.phase==='connected');assert.equal(connected.connected,true,'connected is only emitted after version handshake');
 assert.equal(peers.get('1').configuration.iceServers.length,0,'LAN does not contact an external STUN server');
 const payload={units:Array.from({length:9000},(_,id)=>({id,x:id/3,y:id/7,name:'Roboter ä🚀'})),map:'x'.repeat(200000)};
 assert.ok(JSON.stringify(payload).length>600000);assert.equal(p.host.send('start',payload),true);await settle();assert.equal(JSON.stringify(p.guestMessages[0]),JSON.stringify({type:'start',payload}));assert.ok(maxPacket<16*1024,'large states must be fragmented below conservative SCTP message size');
 assert.equal(p.guest.send('command',{kind:'move',ids:[42],x:5,y:9}),true);await settle();assert.equal(p.hostMessages[0].payload.ids[0],42);
 assert.equal(p.host.send('$hello',{}),false);assert.equal(p.host.send('huge','x'.repeat(5*1024*1024)),false);
 const hostPeer=peers.get('1');hostPeer.channel.paused=true;hostPeer.channel.bufferedAmount=300000;
 for(let i=0;i<30;i++)assert.equal(p.host.send('frame',{i,padding:'x'.repeat(50000)},{replaceKey:'state'}),true);
 assert.ok(p.host.queuedBytes<51000,'replaceable snapshots must coalesce under backpressure');
 hostPeer.channel.bufferedAmount=0;hostPeer.channel.paused=false;hostPeer.channel.onbufferedamountlow();await settle();assert.equal(p.guestMessages.at(-1).payload.i,29);
 p.host.close();assert.equal(p.host.connected,false);assert.equal(p.guest.connected,false);assert.equal(p.guestStatus.at(-1).phase,'disconnected');assert.equal(p.host.send('command',{}),false);
 p.guest.close();
 const a=api.createSession({build:'rules-A'}),b=api.createSession({build:'rules-B'});const offer=await a.host();await assert.rejects(()=>b.join(offer),/Spielversionen/);a.close();b.close();
 p=await pair();const another=api.createSession(),wrongInvite=await another.host();const wrongGuest=api.createSession(),wrongAnswer=await wrongGuest.join(wrongInvite);await assert.rejects(()=>p.host.accept(wrongAnswer),/anderen Einladung/);another.close();wrongGuest.close();
 const bad=new ArrayBuffer(16);new DataView(bad).setUint32(0,0);const receiving=[...peers.values()].find(v=>v.channel?.readyState==='open'&&v.id!==hostPeer.id);receiving.channel.onmessage({data:bad});await settle();
 assert.equal(p.host.connected&&p.guest.connected,false,'invalid wire framing must close the connection');p.host.close();p.guest.close();
 console.log('Multiplayer transport passed: two-peer handshake, LAN-only ICE, version and invitation isolation, 600KB+ Unicode state chunking, bidirectional commands, snapshot coalescing, bounded packets, malformed-data rejection and disconnect cleanup.');
}
run().catch(error=>{console.error(error);process.exitCode=1;for(const peer of peers.values())peer.channel?.close();});
