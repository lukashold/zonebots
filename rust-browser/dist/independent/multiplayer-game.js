/* Host-authoritative two-player simulation. The guest never runs a game tick.
 * The transport carries state and commands only, never sprite/audio resources. */
(function(global){
 'use strict';
 const ACTIONS=new Set(['command','produce','placeGun']);
 const ORDERS=new Set(['move','attack','attackBuilding','attackWall','attackBridge','repair','repairVehicle','repairBridge','enter','board','stop','unload']);
 const STATIC=['map','catalog','objects','walls'],FRAME_STEP=1/8,MAX_EVENTS=1024;
 const own=(o,k)=>Object.prototype.hasOwnProperty.call(o,k),object=o=>o!==null&&typeof o==='object'&&!Array.isArray(o);
 const integer=(n,max=0xffffffff)=>Number.isSafeInteger(n)&&n>=0&&n<=max;
 const flip=n=>n===0?1:n===1?0:n;
 function remapEvent(event){if(!object(event))return event;const result={...event};for(const key of Object.keys(result))if(['team','owner','winner','previous'].includes(key)||key.endsWith('Team'))result[key]=flip(result[key]);return result;}
 function remapCrew(crew){if(!object(crew))return crew;const result={...crew};if(own(result,'team'))result.team=flip(result.team);return result;}
 function perspective(state,role){
  if(!state||role!=='guest')return state;
  return {...state,units:state.units.map(u=>({...u,team:flip(u.team),crew:remapCrew(u.crew),cargo:(u.cargo||[]).map(remapCrew)})),
   buildings:state.buildings.map(b=>({...b,team:flip(b.team)})),sectors:state.sectors.map(s=>({...s,owner:flip(s.owner)})),
   events:(state.events||[]).map(remapEvent),winner:flip(state.winner)};
 }
 function validState(state,full=false){
  if(!object(state)||!Number.isFinite(state.time)||state.time<0||![null,0,1].includes(state.winner))return false;
  for(const [key,limit]of [['units',512],['buildings',256],['sectors',256],['crates',512],['effects',2048],['events',MAX_EVENTS]])if(!Array.isArray(state[key])||state[key].length>limit)return false;
  if(state.walls!==undefined&&(!Array.isArray(state.walls)||state.walls.length>16384||!state.walls.every(w=>object(w)&&integer(w.id)&&[w.x,w.y,w.hp,w.maxHp].every(Number.isFinite))))return false;
  if(!state.units.every(u=>object(u)&&integer(u.id)&&[u.x,u.y,u.hp,u.maxHp,u.angle,u.aim].every(Number.isFinite)&&[-1,0,1].includes(u.team)))return false;
  if(!state.buildings.every(b=>object(b)&&integer(b.id)&&[b.x,b.y,b.w,b.h,b.hp,b.maxHp].every(Number.isFinite)&&[-1,0,1].includes(b.team)))return false;
  if(!state.sectors.every(s=>object(s)&&integer(s.id)&&[s.x,s.y].every(Number.isFinite)&&[-1,0,1].includes(s.owner)))return false;
  if(!state.effects.every(f=>object(f)&&[f.x,f.y,f.age,f.life].every(Number.isFinite)))return false;
  if(full||state.map){const m=state.map;if(!object(m)||!integer(m.width,128)||!integer(m.height,128)||m.width<1||m.height<1||!integer(m.seed)||!integer(m.revision))return false;const count=m.width*m.height;if(!Array.isArray(m.tiles)||m.tiles.length!==count||!Array.isArray(m.territory)||m.territory.length!==count)return false;}
  if(full&&(!Array.isArray(state.catalog)||state.catalog.length>64||!Array.isArray(state.objects)||state.objects.length>4096))return false;
  return true;
 }
 function validateAction(request,team,state,preview=false){
  if(!object(request)||!state||!integer(team,1))return null;
  const op=request.op;if(preview?op!=='previewGun':!ACTIONS.has(op))return null;
  const result={op,team},position=()=>{
   const x=request.x,y=request.y,m=state.map;if(!Number.isFinite(x)||!Number.isFinite(y)||x<0||y<0||x>m.width*32||y>m.height*32)return false;
   result.x=x;result.y=y;return true;
  };
  if(op==='command'){
   if(!ORDERS.has(request.kind)||!Array.isArray(request.ids)||!request.ids.length||request.ids.length>120||!request.ids.every(id=>integer(id)))return null;
   const units=new Map(state.units.map(u=>[u.id,u]));
   result.ids=[...new Set(request.ids)].filter(id=>{const u=units.get(id);return u&&u.team===team&&(state.map.original?u.hp>=0:u.hp>0);});
   if(!result.ids.length)return null;result.kind=request.kind;
   if(own(request,'target')){if(!integer(request.target))return null;result.target=request.target;}
   if(['enter','board','repair','repairVehicle','attackBuilding'].includes(result.kind)&&!own(result,'target'))return null;
   if(result.kind==='repairVehicle'){const b=state.buildings.find(b=>b.id===result.target&&b.kind==='repair'&&b.team===team&&b.hp>0);if(!b)return null;result.ids=result.ids.filter(id=>{const u=units.get(id),d=state.catalog.find(d=>d.type===u.type);return d?.class===1&&u.crew!=null&&u.hp<u.maxHp;});if(!result.ids.length)return null;}
   if(result.kind==='attackWall'&&(!own(result,'target')||!(state.walls||[]).some(w=>w.id===result.target&&w.hp>0)))return null;
   if(own(request,'x')||own(request,'y')){if(!position())return null;}
   if(['move','attackBridge','repairBridge'].includes(result.kind)&&!own(result,'x'))return null;
   if(result.kind==='attack'&&!own(result,'target')&&!own(result,'x'))return null;
  }else{
   if(!integer(request.id))return null;
   const b=state.buildings.find(b=>b.id===request.id&&b.team===team&&b.hp>0);if(!b)return null;result.id=request.id;
   if(op==='produce'){
    const product=request.product??null;
    if(product!==null&&(typeof product!=='string'||product.length>24||!state.catalog.some(d=>d.type===product)||(state.buildMenus?.[b.id]&&!state.buildMenus[b.id].includes(product))))return null;
    result.product=product;
   }else{
    const slot=request.slot??0;if(!integer(slot,3)||!position())return null;result.slot=slot;
   }
  }
  return result;
 }
 function create(options={}){
  if(typeof options.run!=='function')throw Error('Multiplayer benötigt einen Spielkern.');
  let session=null,role=null,canonical=null,view=null,active=false,paused=true,awaitingReady=false,epoch=0,sequence=0,receivedSequence=-1;
  let frameClock=0,lastMap='',lastWalls='',pendingEvents=[],nextRequest=1,readyTimer=null;
  let actionBudget=40,previewBudget=20,pauseBudget=4,budgetAt=Date.now();
  const info=(phase,message,extra={})=>options.onInfo?.({phase,message,...extra});
  const send=(type,payload,settings)=>session?.send(type,payload,settings)===true;
  const validRequestId=id=>typeof id==='string'&&id.length>0&&id.length<=64||integer(id);
  const requestId=id=>validRequestId(id)?id:nextRequest++;
  function budgets(kind){const now=Date.now(),delta=Math.max(0,Math.min(60,(now-budgetAt)/1000));budgetAt=now;actionBudget=Math.min(40,actionBudget+delta*20);previewBudget=Math.min(20,previewBudget+delta*15);pauseBudget=Math.min(4,pauseBudget+delta*2);if(kind==='action'){if(actionBudget<1)return false;actionBudget--;return true;}if(kind==='preview'){if(previewBudget<1)return false;previewBudget--;return true;}if(pauseBudget<1)return false;pauseBudget--;return true;}
  function apply(part,events=true){
   const state={...part};for(const key of STATIC)if(!own(state,key)&&canonical)state[key]=canonical[key];
   if(!validState(state,true))throw Error('Ungültiger Multiplayer-Spielzustand.');
   canonical=state;view=perspective(state,role);
   if(events&&role==='host'&&state.events?.length)pendingEvents.push(...state.events);
   if(pendingEvents.length>MAX_EVENTS)pendingEvents.splice(0,pendingEvents.length-MAX_EVENTS);
   options.apply?.(view);return view;
  }
  function run(request){
   const compact=canonical?{compact:true,knownMapSeed:canonical.map.seed,knownMapRevision:canonical.map.revision}:{};
   const result=options.run({...request,...compact});if(result?.error)throw Error(String(result.error));
   return apply(result);
  }
  function mapStamp(){return canonical?canonical.map.seed+':'+canonical.map.revision:'';}
  function wallStamp(){return (canonical?.walls||[]).map(w=>`${w.id}:${w.hp}:${w.neighborMask??0}:${w.wallTileIndex??0}`).join('|');}
  function dynamic(events){const result={...canonical,events};delete result.catalog;delete result.objects;if(mapStamp()===lastMap)delete result.map;if(wallStamp()===lastWalls)delete result.walls;return result;}
  function transmitState(type,extra={},events=[]){
   const seq=sequence+1,result=send(type,{epoch,seq,state:dynamic(events),...extra});
   if(result){sequence=seq;lastMap=mapStamp();lastWalls=wallStamp();}return result;
  }
  function broadcast(force=false){
   if(role!=='host'||!active||awaitingReady||!canonical)return false;
   if(!force&&Number(session?.queuedBytes)>512*1024)return false;
   if(transmitState('game:frame',{},pendingEvents.slice())){pendingEvents=[];return true;}return false;
  }
  function commitPause(value,reason='',notify=true){
   if(typeof value!=='boolean'||!active)return false;
   if(value&&role==='host'&&!awaitingReady)broadcast(true);
   paused=value;options.onPause?.(paused,reason);
   if(notify&&role==='host')send('game:pause',{epoch,paused,reason:String(reason).slice(0,160)});
   return true;
  }
  function fail(message){paused=true;options.onPause?.(true,message);info('failed',message);return false;}
  function bind(nextSession,nextRole){
   if(!nextSession||typeof nextSession.send!=='function'||!['host','guest'].includes(nextRole))throw Error('Ungültige Multiplayer-Verbindung.');
   clearTimeout(readyTimer);session=nextSession;role=nextRole;canonical=view=null;active=false;paused=true;awaitingReady=false;epoch=sequence=0;receivedSequence=-1;frameClock=0;lastMap='';lastWalls='';pendingEvents=[];
   actionBudget=40;previewBudget=20;pauseBudget=4;budgetAt=Date.now();return api;
  }
  function start(initRequest,meta={}){
   if(role!=='host'||!session?.connected)return false;
   clearTimeout(readyTimer);epoch++;sequence=0;receivedSequence=-1;canonical=view=null;frameClock=0;pendingEvents=[];active=true;paused=true;awaitingReady=true;
   try{
    const request={...initRequest,op:'init',multiplayer:true,difficulty:1,aiTeams:[],compact:false};
    const result=options.run(request);if(result?.error)throw Error(String(result.error));apply(result);
    const details={mission:canonical.mission,difficulty:1,seed:request.seed,original:Boolean(canonical.map.original),name:String(meta.name||'Mehrspielerpartie').slice(0,96)};
    options.onStart?.(details,role);options.onPause?.(true,'Warte auf Mitspieler …');lastMap=mapStamp();lastWalls=wallStamp();
    if(!send('game:init',{epoch,seq:0,meta:details,state:canonical,paused:true}))return fail('Der Spielzustand konnte nicht gesendet werden.');
    if(awaitingReady){info('waiting','Mitspieler lädt die Karte …');readyTimer=setTimeout(()=>{if(active&&awaitingReady)fail('Der Mitspieler hat den Spielstart nicht bestätigt.');},30000);readyTimer?.unref?.();}
    return true;
   }catch(error){active=false;return fail(error.message||'Mehrspielerpartie konnte nicht gestartet werden.');}
  }
  function rejected(id,message,preview=false){
   send(preview?'game:previewResult':'game:result',{epoch,requestId:id,error:message});return false;
  }
  function action(request,id){
   if(!active||awaitingReady||paused)return null;id=requestId(id);
   if(!session?.connected){fail('Verbindung zum Mitspieler unterbrochen.');return null;}
   const command=validateAction(request,role==='guest'?1:0,canonical);if(!command){info('rejected','Dieser Auftrag ist nicht möglich.');return null;}
   if(role==='guest'){if(!send('game:action',{epoch,requestId:id,request:command})){info('failed','Auftrag konnte nicht gesendet werden.');options.onAction?.({error:'Keine Verbindung'},id);}return null;}
   try{const result=run(command);if(canonical.winner!==null){broadcast(true);commitPause(true,'Schlacht beendet.');}return result;}catch(error){fail(error.message);return null;}
  }
  function requestPreview(request,id){
   id=requestId(id);const command=active&&!awaitingReady&&!paused?validateAction({...request,op:'previewGun'},role==='guest'?1:0,canonical,true):null;
   if(!command){const result={valid:false,reason:'Kein eigener gültiger Platzierungsauftrag'};options.onPreview?.(result,id);return result;}
   if(role==='guest'){if(!send('game:preview',{epoch,requestId:id,request:command}))options.onPreview?.({valid:false,reason:'Keine Verbindung'},id);return null;}
   const result=options.run(command);options.onPreview?.(result,id);return result;
  }
  function tick(dt){
   if(role!=='host'||!active||paused||awaitingReady||!Number.isFinite(dt)||dt<=0)return null;
   if(!session?.connected){fail('Verbindung zum Mitspieler unterbrochen.');return null;}
   try{const result=run({op:'tick',dt:Math.min(.1,dt)});frameClock+=Math.min(.1,dt);if(frameClock>=FRAME_STEP){frameClock%=FRAME_STEP;broadcast();}if(canonical.winner!==null){broadcast(true);commitPause(true,'Schlacht beendet.');}return result;}catch(error){fail(error.message);return null;}
  }
  function setPaused(value,reason='Pause'){
   if(!active||typeof value!=='boolean'||awaitingReady)return false;
   if(role==='guest'){const sent=send('game:pauseRequest',{epoch,paused:value});if(!sent){info('failed','Pausenauftrag konnte nicht gesendet werden.');return false;}if(value){paused=true;options.onPause?.(true,reason);}return true;}
   return commitPause(value,reason);
  }
  function handleTransportStatus(status){
   if(!active||!object(status)||status.connected||!['reconnecting','disconnected','failed','closed'].includes(status.phase))return false;
   return commitPause(true,'Verbindung unterbrochen. Die Partie ist angehalten.',false);
  }
  function close(notify=true){
   if(notify&&active)send('game:close',{epoch});clearTimeout(readyTimer);active=false;paused=true;awaitingReady=false;pendingEvents=[];
   options.onPause?.(true,'Mehrspielerpartie beendet.');info('closed','Mehrspielerpartie beendet.');session=null;return true;
  }
  function handle(message){
   if(!object(message)||typeof message.type!=='string'||!object(message.payload))return false;
   const {type,payload:p}=message;
   try{
    if(type==='game:init'){
     if(role!=='guest'||!integer(p.epoch)||p.epoch<=epoch||!validState(p.state,true)||!object(p.meta))return false;
     epoch=p.epoch;receivedSequence=0;active=true;paused=true;awaitingReady=false;frameClock=0;pendingEvents=[];canonical=null;apply(p.state,false);
     const meta={mission:canonical.mission,difficulty:1,seed:p.meta.seed,original:Boolean(canonical.map.original),name:String(p.meta.name||'Mehrspielerpartie').slice(0,96)};
     options.onStart?.(meta,role);options.onPause?.(true,'Warte auf Gastgeber …');send('game:ready',{epoch});return true;
    }
    if(!active||p.epoch!==epoch)return false;
    if(type==='game:ready'){
     if(role!=='host'||!awaitingReady)return false;clearTimeout(readyTimer);awaitingReady=false;commitPause(false,'Partie gestartet.');info('playing','Mehrspielerpartie gestartet.');return true;
    }
    if(type==='game:frame'||type==='game:result'&&p.state){
     if(role!=='guest'||!integer(p.seq)||p.seq<=receivedSequence||!validState(p.state))return false;
     receivedSequence=p.seq;apply(p.state,false);if(type==='game:result')options.onAction?.(view,p.requestId);return true;
    }
    if(type==='game:result'){
     if(role!=='guest')return false;const result={error:String(p.error||'Auftrag abgewiesen').slice(0,160)};options.onAction?.(result,p.requestId);info('rejected',result.error);return true;
    }
    if(type==='game:action'){
     if(!validRequestId(p.requestId))return false;
     if(role!=='host'||!budgets('action')||paused||awaitingReady)return rejected(p.requestId,'Die Partie ist angehalten.');
     const command=validateAction(p.request,1,canonical);if(!command)return rejected(p.requestId,'Ungültiger Auftrag oder fremde Einheit.');
     run(command);transmitState('game:result',{requestId:p.requestId},[]);if(canonical.winner!==null){broadcast(true);commitPause(true,'Schlacht beendet.');}return true;
    }
    if(type==='game:preview'){
     if(!validRequestId(p.requestId))return false;
     if(role!=='host'||!budgets('preview')||paused||awaitingReady)return rejected(p.requestId,'Die Partie ist angehalten.',true);
     const command=validateAction(p.request,1,canonical,true);if(!command)return rejected(p.requestId,'Ungültige Platzierung.',true);
     const result=options.run(command);send('game:previewResult',{epoch,requestId:p.requestId,result});return true;
    }
    if(type==='game:previewResult'){
     if(role!=='guest')return false;const result=p.error?{valid:false,reason:String(p.error).slice(0,160)}:p.result;
     if(!object(result)||typeof result.valid!=='boolean')return false;options.onPreview?.(result,p.requestId);return true;
    }
    if(type==='game:pauseRequest'){
     if(role!=='host'||typeof p.paused!=='boolean'||awaitingReady)return false;
     if(!budgets('pause')){send('game:pause',{epoch,paused,reason:'Pausenauftrag zu schnell. Aktueller Zustand des Gastgebers.'});return false;}
     return commitPause(p.paused,'Mitspieler hat '+(p.paused?'pausiert.':'fortgesetzt.'));
    }
    if(type==='game:pause'){
     if(role!=='guest'||typeof p.paused!=='boolean')return false;paused=p.paused;options.onPause?.(paused,String(p.reason||'Pause').slice(0,160));if(!paused)info('playing','Mehrspielerpartie gestartet.');return true;
    }
    if(type==='game:close')return close(false);
   }catch(error){return fail(error.message||'Mehrspielerzustand konnte nicht verarbeitet werden.');}
   return false;
  }
  const api=Object.freeze({bind,start,handle,action,tick,requestPreview,setPaused,handleTransportStatus,close,get state(){return canonical;},get viewState(){return view;},get role(){return role;},get isHost(){return role==='host';},get active(){return active;},get paused(){return paused;},get waiting(){return awaitingReady;}});
  return api;
 }
 global.MultiplayerGame=Object.freeze({create,perspective,validateAction});
})(window);
