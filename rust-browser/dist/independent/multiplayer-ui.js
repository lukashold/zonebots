// Two-player lobby. Codes and optional relay credentials remain in this page only.
(function(global){
 'use strict';
 const $=id=>document.getElementById(id),page=$('mpLobby');if(!page)return;
 let session=null,busy=false,role=null,notice='Partie erstellen oder eine Einladung einfügen.';
 const terminal=new Set(['failed','closed','disconnected']);
 const bridge=()=>global.GameBridge;
 function message(text,kind=''){notice=String(text||'');$('mpStatus').textContent=notice;$('mpStatus').dataset.kind=kind;}
 function missions(){
  const list=bridge()?.missions||[],select=$('mpMission'),old=select.value;
  if(select.options.length!==list.length){select.replaceChildren();list.forEach((mission,index)=>{const option=document.createElement('option');option.value=index;option.textContent=`${index+1} / ${mission.name||'Mission '+(index+1)}`;select.append(option);});if(old&&Number(old)<list.length)select.value=old;}
  render();
 }
 function render(){
  const ready=Boolean(bridge()?.ready&&global.Multiplayer?.supported),connected=Boolean(session?.connected),active=Boolean(session&&!terminal.has(session.phase));
  $('mpHost').disabled=busy||active||!ready;$('mpJoin').disabled=busy||active||!ready;
  $('mpName').disabled=busy||active;$('mpMode').disabled=busy||active;
  for(const id of ['mpTurnUrl','mpTurnUser','mpTurnPassword'])$(id).disabled=busy||active||$('mpMode').value==='lan';
  $('mpTurn').hidden=$('mpMode').value==='lan';
  $('mpRoleButtons').hidden=active||connected;
  $('mpLocalSection').hidden=!$('mpLocalCode').value||connected;
  $('mpRemoteSection').hidden=connected;
  $('mpAccept').hidden=role!=='host'||!session||terminal.has(session.phase);
  $('mpAccept').disabled=busy||!$('mpRemoteCode').value.trim()||!$('mpLocalCode').value;
  $('mpJoin').hidden=role==='host'&&active;
  $('mpJoin').disabled=busy||active||!ready;
  $('mpReady').hidden=!connected;
  $('mpMission').disabled=role!=='host'||busy||!connected;
  $('mpStart').hidden=role!=='host';$('mpStart').disabled=busy||!connected||!ready||!$('mpMission').options.length;
  $('mpCancel').hidden=!session&&!busy;
  $('mpRemoteLabel').textContent=role==='host'&&active?'2 · Antwortcode des Mitspielers':'Einladung des Gastgebers';
  $('mpLocalLabel').textContent=role==='guest'?'2 · Antwort an den Gastgeber zurückgeben':'1 · Einladung an den Mitspieler geben';
  $('mpReadyText').textContent=role==='host'?`${session?.remoteName||'Mitspieler'} ist verbunden. Mission wählen und gemeinsam starten.`:'Verbunden. Der Gastgeber wählt die Mission und startet die Partie.';
  if(!ready&&!busy&&!session)message(global.Multiplayer?.supported?'Spiel wird geladen …':'WebRTC ist in diesem Browser nicht verfügbar. Bitte einen aktuellen Browser verwenden.',global.Multiplayer?.supported?'':'error');
 }
 function detachSession(){
  const previous=session;session=null;busy=false;role=null;previous?.close();
  $('mpLocalCode').value='';$('mpRemoteCode').value='';message('Partie erstellen oder eine Einladung einfügen.');render();
 }
 function reset(showNotice=true,preserveBusy=false){
  const previous=session,wasBusy=busy;detachSession();if(previous)bridge()?.leaveMultiplayer?.();if(preserveBusy)busy=wasBusy;
  if(showNotice)message('Verbindung beendet. Du kannst eine neue Partie erstellen.');render();
 }
 function config(){
  const result={name:$('mpName').value.trim().slice(0,24)||'Spieler',build:global.Multiplayer.BUILD};
  if($('mpMode').value==='online'&&$('mpTurnUrl').value.trim()){
   const url=$('mpTurnUrl').value.trim();if(!/^turns?:[^\s]+$/i.test(url))throw Error('Bitte eine gültige TURN-Adresse eingeben, z.B. turns:server:5349.');
   result.iceServers=[...global.Multiplayer.defaultIceServers,{urls:url,username:$('mpTurnUser').value,credential:$('mpTurnPassword').value}];
  }
  return result;
 }
 function create(nextRole){
  const settings=config();if(session)reset(false,true);role=nextRole;let next;
  next=global.Multiplayer.createSession({...settings,
   onMessage(value){if(session!==next)return;try{const handled=bridge()?.network?.handle(value);handled?.catch?.(error=>message(error.message,'error'));}catch(error){message(error.message,'error');}},
   onStatus(status){
    if(session!==next)return;message(status.message||(status.phase==='connected'?`${status.remoteName||'Mitspieler'} verbunden${status.latencyMs!==null?' · '+status.latencyMs+' ms':''}`:notice),terminal.has(status.phase)?'error':status.connected?'connected':'');
    bridge()?.network?.handleTransportStatus?.(status);global.dispatchEvent(new CustomEvent('multiplayer-status',{detail:status}));render();
   }
  });session=next;bridge().openMultiplayer(next,nextRole);return next;
 }
 async function action(work){if(busy)return;busy=true;render();try{await work();}catch(error){message(error?.message||'Die Verbindung konnte nicht hergestellt werden.','error');}finally{busy=false;render();}}
 $('mpHost').addEventListener('click',()=>action(async()=>{
  const next=create('host');$('mpRemoteCode').value='';$('mpLocalCode').value=await next.host({lan:$('mpMode').value==='lan'});render();
 }));
 $('mpJoin').addEventListener('click',()=>action(async()=>{
  const invitation=$('mpRemoteCode').value.trim();if(!invitation)throw Error('Bitte zuerst die Einladung des Gastgebers einfügen.');global.Multiplayer.decodeCode(invitation);
  const next=create('guest');$('mpRemoteCode').value=invitation;$('mpLocalCode').value=await next.join(invitation,{lan:$('mpMode').value==='lan'});render();
 }));
 $('mpAccept').addEventListener('click',()=>action(async()=>{if(!session||role!=='host')throw Error('Bitte zuerst eine Partie erstellen.');await session.accept($('mpRemoteCode').value);}));
 $('mpStart').addEventListener('click',()=>action(async()=>{
  if(!session?.connected||role!=='host')throw Error('Bitte zuerst den Mitspieler verbinden.');message('Partie wird gemeinsam gestartet …');await bridge().startMultiplayer(Number($('mpMission').value));
 }));
 $('mpCopy').addEventListener('click',async()=>{
  const value=$('mpLocalCode').value;if(!value)return;
  try{if(!global.navigator?.clipboard?.writeText)throw Error();await global.navigator.clipboard.writeText(value);message(role==='host'?'Einladung kopiert. Jetzt an den Mitspieler geben.':'Antwort kopiert. Jetzt an den Gastgeber zurückgeben.');}
  catch{$('mpLocalCode').focus();$('mpLocalCode').select();message('Der Code ist markiert. Mit Strg+C oder ⌘C kopieren.');}
 });
 $('mpCancel').addEventListener('click',()=>reset());
 $('mpMode').addEventListener('change',()=>{message($('mpMode').value==='lan'?'Beide Geräte müssen im selben Netzwerk sein.':'Einladung und Antwort werden direkt zwischen euch ausgetauscht.');render();});
 $('mpRemoteCode').addEventListener('input',render);
 global.addEventListener('game-ready',()=>{if(!session)message('Partie erstellen oder eine Einladung einfügen.');missions();});
 global.addEventListener('multiplayer-info',event=>{if(event.detail?.message||typeof event.detail==='string')message(event.detail?.message||event.detail,event.detail?.error?'error':'');});
 global.addEventListener('beforeunload',()=>session?.close());
 missions();
 global.MultiplayerUI=Object.freeze({reset,detachSession,get session(){return session;}});
})(window);
