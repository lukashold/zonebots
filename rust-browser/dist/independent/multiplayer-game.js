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
  return {...state,nativePortrait:state.nativePortraits?.find(p=>p.recipientTeam===1)?{...state.nativePortraits.find(p=>p.recipientTeam===1),recipientTeam:0}:state.nativePortrait,nativePortraits:state.nativePortraits?.map(p=>({...p,recipientTeam:flip(p.recipientTeam)})),units:state.units.map(u=>({...u,team:flip(u.team),crew:remapCrew(u.crew),cargo:(u.cargo||[]).map(remapCrew)})),
   objects:(state.objects||[]).map(o=>Number.isInteger(o.nativeTeam)?{...o,nativeTeam:flip(o.nativeTeam)}:o),
   nativeTrailEffects:(state.nativeTrailEffects||[]).map(f=>({...f})),
   nativeEffects:(state.nativeEffects||[]).map(f=>f.primitive==='colorPoint'?{...f,...(Number.isInteger(f.nativeTintOwner)?{nativeTintOwner:flip(f.nativeTintOwner),colorIndex:(flip(f.nativeTintOwner)*16+24)&255}:{})}:{...f,...(Number.isInteger(f.team)&&!f.nativeLiteralPalette?{team:flip(f.team)}:{})}),
   nativeAmbientVisuals:(state.nativeAmbientVisuals||[]).map(f=>({...f,team:flip(f.team),palette:flip(f.palette)})),
   nativeAmbientAnimations:(state.nativeAmbientAnimations||[]).map(f=>({...f,...(Number.isInteger(f.team)?{team:flip(f.team)}:{})})),
   nativeCosmetic20Visuals:(state.nativeCosmetic20Visuals||[]).map(f=>({...f,...(Number.isInteger(f.team)?{team:flip(f.team)}:{})})),
   nativeLavaVisuals:(state.nativeLavaVisuals||[]).map(f=>({...f,...(Number.isInteger(f.team)?{team:flip(f.team)}:{})})),
   nativeObjectVisuals:state.nativeObjectVisuals,
   nativeDisplayQueue:(state.nativeDisplayQueue||[]).map(r=>({...r,...(Number.isInteger(r.palette)&&!r.nativeLiteralPalette?{palette:flip(r.palette)}:{}),...(Number.isInteger(r.team)?{team:flip(r.team)}:{}),...(object(r.art)?{art:{...r.art,team:flip(r.art.team)}}:{}),...(Array.isArray(r.nativeLayers)?{nativeLayers:r.nativeLayers.map(l=>({...l,palette:flip(l.palette)}))}:{}),...(object(r.draw)?{draw:{...r.draw,...(Number.isInteger(r.draw.team)?{team:flip(r.draw.team)}:{}),...(Number.isInteger(r.draw.palette)?{palette:flip(r.draw.palette)}:{})}}:{})})),
   nativeTrackVisuals:(state.nativeTrackVisuals||[]).map(f=>({...f,...(Number.isInteger(f.team)?{team:flip(f.team)}:{})})),
   nativeDeathVisuals:(state.nativeDeathVisuals||[]).map(f=>({...f,team:flip(f.team)})),
   buildings:state.buildings.map(b=>({...b,team:flip(b.team)})),sectors:state.sectors.map(s=>({...s,owner:flip(s.owner)})),
   bullets:(state.bullets||[]).map(b=>({...b,team:flip(b.team)})),projectiles:(state.projectiles||[]).map(p=>({...p,team:flip(p.team)})),blasts:(state.blasts||[]).map(b=>({...b,team:flip(b.team)})),fires:(state.fires||[]).map(f=>({...f,team:flip(f.team)})),events:(state.events||[]).map(remapEvent),winner:flip(state.winner)};
 }
 function validState(state,full=false){
  if(!object(state)||!Number.isFinite(state.time)||state.time<0||![null,0,1].includes(state.winner))return false;
  if(state.nativeUiClock!==undefined&&(!Number.isInteger(state.nativeUiClock)||state.nativeUiClock< -32768||state.nativeUiClock>32767))return false;
  if(state.nativePortraits!==undefined&&(!Array.isArray(state.nativePortraits)||state.nativePortraits.length!==2||!state.nativePortraits.every((p,i)=>object(p)&&p.recipientTeam===i&&integer(p.mode,4)&&Number.isInteger(p.message)&&p.message>=-2147483648&&p.message<=2147483647&&['phraseIndex','phraseCounter','phraseFrame'].every(k=>Number.isInteger(p[k])&&p[k]>=-32768&&p[k]<=32767)&&['nativeSlot','rawSpeakerSlot'].every(k=>p[k]===null||integer(p[k],239))&&(p.unitId===null||integer(p.unitId)))))return false;
  if(Array.isArray(state.events)&&!state.events.every(e=>!['nativeVoice','nativeSample'].includes(e?.kind)||e.nativeAudioId===undefined||typeof e.nativeAudioId==='string'&&/^[0-9]{1,20}$/.test(e.nativeAudioId)&&integer(e.recipientTeam,1)&&(e.actorTeam===null||integer(e.actorTeam,3))))return false;
  for(const [key,limit]of [['units',512],['buildings',256],['sectors',256],['crates',512],['effects',2048],['events',MAX_EVENTS]])if(!Array.isArray(state[key])||state[key].length>limit)return false;
  if(state.bullets!==undefined&&(!Array.isArray(state.bullets)||state.bullets.length>200||!state.bullets.every(b=>object(b)&&integer(b.id,199)&&[b.x,b.y,b.angle].every(Number.isFinite)&&[0,1].includes(b.team)&&typeof b.laser==='boolean'&&integer(b.phase,1))))return false;
  if(state.projectiles!==undefined&&(!Array.isArray(state.projectiles)||state.projectiles.length>50||!state.projectiles.every(p=>object(p)&&integer(p.id,49)&&[p.x,p.y,p.angle,p.size].every(Number.isFinite)&&[0,1].includes(p.team)&&['shell','grenade','rocket','flame','howitzer'].includes(p.kind)&&integer(p.sprite,7210)&&p.size>=0&&p.size<=256)))return false;
  if(state.blasts!==undefined&&(!Array.isArray(state.blasts)||state.blasts.length>50||!state.blasts.every(b=>object(b)&&integer(b.id,49)&&[b.x,b.y].every(Number.isFinite)&&[0,1,255].includes(b.team)&&integer(b.frame,31)&&integer(b.strength,255)&&integer(b.sprite,7210)&&typeof b.water==='boolean')))return false;
  if(state.fires!==undefined&&(!Array.isArray(state.fires)||state.fires.length>200||!state.fires.every(f=>object(f)&&integer(f.id,199)&&[f.x,f.y].every(Number.isFinite)&&[0,1].includes(f.team)&&integer(f.frame,5)&&integer(f.sprite,7210))))return false;
  if(state.nativeEffects!==undefined&&(!Array.isArray(state.nativeEffects)||state.nativeEffects.length>2000||!state.nativeEffects.every(f=>{
   if(!object(f)||!integer(f.id,1999))return false;
   if(f.primitive==='colorPoint')return integer(f.colorIndex,255)&&(f.nativeTintOwner===undefined||integer(f.nativeTintOwner,255)&&f.colorIndex===((f.nativeTintOwner*16+24)&255))&&[f.x,f.y].every(n=>Number.isSafeInteger(n)&&n%2===0&&n>=-65536&&n<=65534)&&[f.nativeDrawX,f.nativeDrawY].every(n=>Number.isInteger(n)&&n>=-32768&&n<=32767)&&['sprite','team','scale','rotation','source','nativeInheritedPalette','nativeLiteralPalette','nativeWhitePalette'].every(k=>!own(f,k));
   if(own(f,'nativeInheritedPalette')||own(f,'source'))return f.source==='native4666c0'&&f.nativeInheritedPalette===true&&integer(f.sprite,7210)&&f.scale===1&&f.rotation===0&&[f.x,f.y].every(n=>Number.isSafeInteger(n)&&n%2===0&&n>=-65536&&n<=65534)&&[f.nativeDrawX,f.nativeDrawY].every(n=>Number.isInteger(n)&&n>=-32768&&n<=32767)&&['primitive','colorIndex','nativeTintOwner','team','nativeLiteralPalette','nativeWhitePalette'].every(k=>!own(f,k));
   return(f.nativeLiteralPalette===undefined||typeof f.nativeLiteralPalette==='boolean'&&(!f.nativeLiteralPalette||integer(f.team,3)))&&f.primitive===undefined&&f.colorIndex===undefined&&integer(f.sprite,7210)&&[f.x,f.y,f.scale,f.rotation].every(Number.isFinite)&&Math.abs(f.scale)<=1025;
  })))return false;
  if(state.nativeTrailEffects!==undefined&&(!Array.isArray(state.nativeTrailEffects)||state.nativeTrailEffects.length>8128||!state.nativeTrailEffects.every(f=>object(f)&&integer(f.id,63)&&f.source==='buildingTrail'&&f.stage==='late'&&f.nativeInheritedPalette===true&&Number.isInteger(f.sprite)&&f.sprite>=-32768&&f.sprite<=32767&&[f.x,f.y].every(n=>Number.isSafeInteger(n)&&n%2===0&&n>=-65536&&n<=65534)&&[f.nativeDrawX,f.nativeDrawY].every(n=>Number.isInteger(n)&&n>=-32768&&n<=32767)&&f.scale===0.5&&Number.isFinite(f.rotation)&&!own(f,'team')&&!own(f,'nativeLiteralPalette')&&!own(f,'primitive')&&!own(f,'colorIndex')&&!own(f,'nativeTintOwner')&&!own(f,'nativeWhitePalette'))))return false;
  if(state.nativeDeathVisuals!==undefined&&(!Array.isArray(state.nativeDeathVisuals)||state.nativeDeathVisuals.length>15||!state.nativeDeathVisuals.every(f=>object(f)&&integer(f.id,14)&&integer(f.sprite,7210)&&[f.x,f.y,f.scale,f.rotation].every(Number.isFinite)&&[0,1,2,3].includes(f.team))))return false;
  if(state.nativeAmbientVisuals!==undefined&&(!Array.isArray(state.nativeAmbientVisuals)||state.nativeAmbientVisuals.length>100||!state.nativeAmbientVisuals.every(f=>object(f)&&integer(f.id,99)&&integer(f.sprite,7210)&&[f.x,f.y,f.queueY,f.scale,f.rotation].every(Number.isFinite)&&integer(f.team,3)&&integer(f.palette,3)&&integer(f.flags52,255)&&integer(f.flags53,255)&&Math.abs(f.scale)<=1025)))return false;
  for(const [key,limit]of [['nativeAmbientAnimations',150],['nativeCosmetic20Visuals',40],['nativeLavaVisuals',32],['nativeTrackVisuals',500]])if(state[key]!==undefined&&(!Array.isArray(state[key])||state[key].length>limit||!state[key].every(f=>object(f)&&integer(f.id,limit-1)&&integer(f.sprite,7210)&&[f.x,f.y,f.scale,f.rotation].every(Number.isFinite)&&Math.abs(f.scale)<=1025&&(f.team==null||integer(f.team,3))&&(f.nativeInheritedPalette===undefined||typeof f.nativeInheritedPalette==='boolean')&&(f.nativeWhitePalette===undefined||typeof f.nativeWhitePalette==='boolean')&&(f.queueY===undefined||Number.isFinite(f.queueY)))))return false;
  if(state.nativeObjectVisuals!==undefined&&(!Array.isArray(state.nativeObjectVisuals)||state.nativeObjectVisuals.length>200||!state.nativeObjectVisuals.every(o=>object(o)&&integer(o.id,4095)&&typeof o.nativeActive==='boolean'&&[o.x,o.y].every(Number.isFinite)&&integer(o.nativeAnimation18,255)&&Number.isInteger(o.nativeSpriteOffset21)&&o.nativeSpriteOffset21>=-128&&o.nativeSpriteOffset21<=127)))return false;
  if(state.nativeDisplayQueue!==undefined&&(!Array.isArray(state.nativeDisplayQueue)||state.nativeDisplayQueue.length>4000||!state.nativeDisplayQueue.every(r=>{
   if(!object(r)||!['building','unit','sprite','effect','repairClock','repairOverlay'].includes(r.kind)||!['pre','queue'].includes(r.stage))return false;
   if(r.nativeCraneBoundary!==undefined&&(r.kind!=='unit'||r.art?.type!=='crane'||r.nativeLayers!==undefined||!['missingWork9a','invalidW21','missingWorld','missingForts','missingHardware','negativeScissor','unsupportedPalette'].includes(r.nativeCraneBoundary)))return false;
   if(r.nativeLiteralPalette!==undefined&&(typeof r.nativeLiteralPalette!=='boolean'||r.nativeLiteralPalette&&(r.kind!=='sprite'||!['wreckOverlay','seasonalOverlay','buildingFragment','fortFragment'].includes(r.source)||r.palette!==0)))return false;
   if(r.source==='retiredUnit'&&(r.kind!=='sprite'||r.stage!=='queue'||!integer(r.nativeSlot,239)||r.nativeLiteralPalette!==undefined||!integer(r.palette,7)))return false;
   if(['buildingFragment','fortFragment'].includes(r.source)){const last=r.source==='fortFragment'?3:34;if(r.kind!=='sprite'||r.stage!=='queue'||r.nativeLiteralPalette!==true||r.palette!==0||!integer(r.nativeSlot,last)||!integer(r.id,last*50+49)||Math.floor(r.id/50)!==r.nativeSlot)return false;}
   if(r.source==='craneWork'&&(r.kind!=='sprite'||r.stage!=='queue'||!integer(r.nativeActorSlot,239)||!integer(r.palette,3)||!Number.isSafeInteger(r.id)||r.scale!==1||r.rotation!==0||r.queueY!==r.y||![r.x,r.y].every(n=>Number.isSafeInteger(n)&&n%2===0&&n>=-65536&&n<=65534)))return false;
   if(r.kind==='repairOverlay')return r.stage==='pre'&&integer(r.id)&&integer(r.nativeSlot,34)&&integer(r.team,3)&&[r.x,r.y].every(Number.isSafeInteger);
   if(r.kind==='repairClock')return r.stage==='pre'&&integer(r.id)&&integer(r.nativeSlot,34)&&[r.x,r.y].every(Number.isSafeInteger)&&(r.seconds==null||Number.isInteger(r.seconds)&&r.seconds>=-2147483648&&r.seconds<=2147483647);
   if(r.kind==='building'){const a=r.art;return r.stage==='pre'&&integer(r.id)&&(r.radarDishQueued===undefined||typeof r.radarDishQueued==='boolean')&&(a===undefined||object(a)&&integer(a.id)&&typeof a.kind==='string'&&a.kind.length<=32&&Number.isInteger(a.team)&&a.team>=-1&&a.team<=3&&[a.x,a.y,a.w,a.h,a.hp,a.maxHp,a.stars,a.progress,a.completionTicks,r.time].every(Number.isFinite)&&(a.product==null||typeof a.product==='string'&&a.product.length<=32)&&(a.rawBase==null||Array.isArray(a.rawBase)&&a.rawBase.length===2&&a.rawBase.every(Number.isFinite))&&(a.orientation==null||typeof a.orientation==='string'&&a.orientation.length<=32)&&typeof a.nativeFortDestroyed==='boolean'&&(a.nativeWorkshopDrawQueued===undefined||typeof a.nativeWorkshopDrawQueued==='boolean')&&integer(a.nativeFlagFrame,3)&&(r.seconds==null||integer(r.seconds)));}
   if(!Number.isFinite(r.queueY))return false;
   if(r.nativeLayers!==undefined&&(r.kind!=='unit'||!Array.isArray(r.nativeLayers)||r.nativeLayers.length<1||r.nativeLayers.length>5||!r.nativeLayers.every(l=>object(l)&&Number.isInteger(l.sprite)&&l.sprite>=-32768&&l.sprite<=32767&&Number.isSafeInteger(l.x)&&Number.isSafeInteger(l.y)&&integer(l.palette,7)&&[0,1,3,4,5,8,9].includes(l.flags)&&((l.flags===0||l.flags===1)?l.clip===undefined:Array.isArray(l.clip)&&l.clip.length===4&&l.clip.every(Number.isSafeInteger)&&l.clip[2]===512&&l.clip[3]>=0&&(l.flags!==3||l.clip[3]<=254)))))return false;
   if(r.kind==='effect'){const f=r.draw;return object(f)&&integer(f.sprite,32767)&&[f.x,f.y,f.scale,f.rotation].every(Number.isFinite)&&Math.abs(f.scale)<=1025&&(f.team==null||integer(f.team,255));}
   if(r.art!==undefined){const a=r.art,h=a?.nativeHardware;if(r.kind!=='unit'||!object(a)||typeof a.type!=='string'||a.type.length>32||![a.x,a.y,a.angle,a.aim,a.hp,a.maxHp,a.cooldown,r.time].every(Number.isFinite)||!Number.isInteger(a.team)||a.team< -1||a.team>3||typeof a.moving!=='boolean'||!object(a.originalCombat)||!integer(a.originalCombat.fireRate,255)||h!=null&&(!object(h)||!Number.isInteger(h.hatchFrame)||h.hatchFrame< -1||h.hatchFrame>4||!integer(h.hatchBody,2)))return false;}
   return integer(r.id)&&integer(r.palette,255)&&[r.x,r.y].every(Number.isFinite)&&(r.kind==='unit'&&r.sprite==null||integer(r.sprite,32767));
  })))return false;
  if(state.objects!==undefined&&(!Array.isArray(state.objects)||state.objects.length>4096))return false;
  if(state.walls!==undefined&&(!Array.isArray(state.walls)||state.walls.length>16384||!state.walls.every(w=>object(w)&&integer(w.id)&&[w.x,w.y,w.hp,w.maxHp].every(Number.isFinite))))return false;
  if(!state.units.every(u=>object(u)&&integer(u.id)&&[u.x,u.y,u.hp,u.maxHp,u.angle,u.aim].every(Number.isFinite)&&[-1,0,1].includes(u.team)))return false;
  if(!state.units.every(u=>(u.nativeObjectSprite===undefined||integer(u.nativeObjectSprite,7210))&&(u.nativeTarget46===undefined||Number.isInteger(u.nativeTarget46)&&u.nativeTarget46>=-32768&&u.nativeTarget46<=32767)&&(u.nativeCounter68===undefined||Number.isInteger(u.nativeCounter68)&&u.nativeCounter68>=-128&&u.nativeCounter68<=127)&&(u.nativeMask98===undefined||integer(u.nativeMask98,255))&&(u.nativeContact6f===undefined||integer(u.nativeContact6f,255))&&(u.nativeContact70===undefined||integer(u.nativeContact70,255))&&(u.nativeContactPausebc===undefined||integer(u.nativeContactPausebc,255))&&(u.nativeWait72===undefined||Number.isInteger(u.nativeWait72)&&u.nativeWait72>=-32768&&u.nativeWait72<=32767)&&(u.nativeTerrainTag63===undefined||Number.isInteger(u.nativeTerrainTag63)&&u.nativeTerrainTag63>=-128&&u.nativeTerrainTag63<=127)&&(u.nativeCounter97===undefined||Number.isInteger(u.nativeCounter97)&&u.nativeCounter97>=-128&&u.nativeCounter97<=127)&&(u.nativeRange4==null||Number.isInteger(u.nativeRange4)&&u.nativeRange4>=-128&&u.nativeRange4<=127)&&(u.nativePose7e===undefined||Number.isInteger(u.nativePose7e)&&u.nativePose7e>=-32768&&u.nativePose7e<=32767)&&(u.nativePose80===undefined||Number.isInteger(u.nativePose80)&&u.nativePose80>=-128&&u.nativePose80<=127)&&(u.nativeTransition88===undefined||Number.isInteger(u.nativeTransition88)&&u.nativeTransition88>=-32768&&u.nativeTransition88<=32767)&&(u.nativeElevation76===undefined||Number.isInteger(u.nativeElevation76)&&u.nativeElevation76>=-128&&u.nativeElevation76<=127)&&(u.nativeCell10===undefined||integer(u.nativeCell10,16384))&&(u.nativeBucket38===undefined||integer(u.nativeBucket38,255))&&(u.nativeCounter67===undefined||Number.isInteger(u.nativeCounter67)&&u.nativeCounter67>=-128&&u.nativeCounter67<=127)&&(u.nativeCounter66===undefined||integer(u.nativeCounter66,255))&&(u.nativeReload7b===undefined||Number.isInteger(u.nativeReload7b)&&u.nativeReload7b>=-128&&u.nativeReload7b<=127)&&(u.nativeDirectiona2===undefined||Number.isInteger(u.nativeDirectiona2)&&u.nativeDirectiona2>=-128&&u.nativeDirectiona2<=127)&&(u.nativeDirectiona3===undefined||Number.isInteger(u.nativeDirectiona3)&&u.nativeDirectiona3>=-128&&u.nativeDirectiona3<=127)&&(u.nativeCounter84===undefined||Number.isInteger(u.nativeCounter84)&&u.nativeCounter84>=-32768&&u.nativeCounter84<=32767)&&(u.nativePose82===undefined||Number.isInteger(u.nativePose82)&&u.nativePose82>=-32768&&u.nativePose82<=32767)&&(u.nativeDirection34===undefined||integer(u.nativeDirection34,255))&&(u.nativeField9e===undefined||Number.isInteger(u.nativeField9e)&&u.nativeField9e>=-128&&u.nativeField9e<=127)&&(u.nativeFieldb3===undefined||Number.isInteger(u.nativeFieldb3)&&u.nativeFieldb3>=-128&&u.nativeFieldb3<=127)&&(u.nativeFootprint18===undefined||integer(u.nativeFootprint18))&&(u.nativePermission1c===undefined||integer(u.nativePermission1c))&&(u.nativeTarget42===undefined||Number.isInteger(u.nativeTarget42)&&u.nativeTarget42>=-32768&&u.nativeTarget42<=32767)&&(u.nativeTarget44===undefined||Number.isInteger(u.nativeTarget44)&&u.nativeTarget44>=-32768&&u.nativeTarget44<=32767)&&(u.nativeAxis81===undefined||integer(u.nativeAxis81,255))&&(u.nativeSpeed8b===undefined||Number.isInteger(u.nativeSpeed8b)&&u.nativeSpeed8b>=-128&&u.nativeSpeed8b<=127)&&(u.nativeCounter9f===undefined||integer(u.nativeCounter9f,255))&&(u.nativeCyclea0===undefined||integer(u.nativeCyclea0,255))&&(u.nativeField62===undefined||integer(u.nativeField62,255))&&(u.nativeTrait7a===undefined||Number.isInteger(u.nativeTrait7a)&&u.nativeTrait7a>=-128&&u.nativeTrait7a<=127)&&(u.nativeControl06===undefined||Number.isInteger(u.nativeControl06)&&u.nativeControl06>=-128&&u.nativeControl06<=127)&&(u.nativeProtectionaa===undefined||integer(u.nativeProtectionaa,255))&&(u.nativeOrder5c===undefined||Number.isInteger(u.nativeOrder5c)&&u.nativeOrder5c>=-128&&u.nativeOrder5c<=127)&&(u.nativeFlags9c===undefined||integer(u.nativeFlags9c,255))&&(u.nativeClass7c===undefined||integer(u.nativeClass7c,255))&&(u.nativeFrame32===undefined||Number.isInteger(u.nativeFrame32)&&u.nativeFrame32>=-32768&&u.nativeFrame32<=32767)&&(u.nativeTimer3c===undefined||Number.isInteger(u.nativeTimer3c)&&u.nativeTimer3c>=-128&&u.nativeTimer3c<=127)&&(u.nativePhase35===undefined||integer(u.nativePhase35,255))&&(u.nativeFlags64===undefined||integer(u.nativeFlags64,255))&&(u.nativeInitiativeScan===undefined||object(u.nativeInitiativeScan)&&integer(u.nativeInitiativeScan.flags71,255)&&integer(u.nativeInitiativeScan.a4,255))))return false;
  if(!state.units.every(u=>u.workshop==null||object(u.workshop)&&(u.workshop.nativeSlot==null||integer(u.workshop.nativeSlot,34))))return false;
  if(!state.units.every(u=>u.nativeHardware==null||object(u.nativeHardware)&&integer(u.nativeHardware.turret,7)&&Number.isInteger(u.nativeHardware.hatchFrame)&&u.nativeHardware.hatchFrame>=-1&&u.nativeHardware.hatchFrame<=4&&integer(u.nativeHardware.hatchBody,2)&&[-1,0,1].includes(u.nativeHardware.hatchMotion)))return false;
  if(!state.units.every(u=>u.nativeContact==null||object(u.nativeContact)&&integer(u.nativeContact.phase,2)&&integer(u.nativeContact.frame,10)&&typeof u.nativeContact.side==='boolean'))return false;
  if(!state.units.every(u=>u.nativeTransport==null||object(u.nativeTransport)&&['approach','entering','inside','exiting','released'].includes(u.nativeTransport.phase)&&integer(u.nativeTransport.direction,7)&&integer(u.nativeTransport.frame,6)&&typeof u.nativeTransport.visible==='boolean'&&Number.isInteger(u.nativeTransport.timer)&&u.nativeTransport.timer>=-128&&u.nativeTransport.timer<=127))return false;
  if(!state.units.every(u=>u.nativeMoveFrame===undefined||integer(u.nativeMoveFrame,3)))return false;
  if(!state.units.every(u=>u.nativeIdle==null||object(u.nativeIdle)&&integer(u.nativeIdle.frame,2)&&integer(u.nativeIdle.direction,7)&&integer(u.nativeIdle.desiredDirection,7)&&[-1,1,0].includes(u.nativeIdle.turnStep)&&['timer','lookCounter','lookChance'].every(k=>Number.isInteger(u.nativeIdle[k])&&u.nativeIdle[k]>=-128&&u.nativeIdle[k]<=127)&&integer(u.nativeIdle.searchTimer,64)))return false;
  if(!state.units.every(u=>u.nativeCover==null||object(u.nativeCover)&&u.nativeCover.state35===43&&integer(u.nativeCover.frame32,23)&&u.nativeCover.timer82===-1&&u.nativeCover.animation89===4&&u.nativeCover.counter7e===3&&integer(u.nativeCover.remaining80,4)))return false;
  if(!state.buildings.every(b=>object(b)&&integer(b.id)&&[b.x,b.y,b.w,b.h,b.hp,b.maxHp].every(Number.isFinite)&&(b.nativeFortDestroyed===undefined||typeof b.nativeFortDestroyed==='boolean')&&[-1,0,1].includes(b.team)))return false;
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
   result.ids=[...new Set(request.ids)].filter(id=>{const u=units.get(id);return u&&u.team===team&&u.nativeTransport?.visible!==false&&(state.map.original?u.hp>=0:u.hp>0);});
   if(!result.ids.length)return null;result.kind=request.kind;
   if(request.kind==='move'&&own(request,'formation')){if(!integer(request.formation,4))return null;result.formation=request.formation;}
   if(own(request,'target')){if(!integer(request.target))return null;result.target=request.target;}
   if(['enter','board','repair','repairVehicle','attackBuilding'].includes(result.kind)&&!own(result,'target'))return null;
   if(result.kind==='repairVehicle'){const b=state.buildings.find(b=>b.id===result.target&&b.kind==='repair'&&b.team===team&&(b.kind==='fort'&&typeof b.nativeFortDestroyed==='boolean'?!b.nativeFortDestroyed:b.hp>0));if(!b)return null;result.ids=result.ids.filter(id=>{const u=units.get(id),d=state.catalog.find(d=>d.type===u.type);return d?.class===1&&u.crew!=null&&u.hp<u.maxHp;});if(!result.ids.length)return null;}
   if(result.kind==='attackWall'&&(!own(result,'target')||!(state.walls||[]).some(w=>w.id===result.target&&w.hp>0)))return null;
   if(own(request,'x')||own(request,'y')){if(!position())return null;}
   if(own(request,'nativeClick')){const p=request.nativeClick,m=state.map;if(!['move','attack','attackBuilding','attackWall','attackBridge'].includes(result.kind)||!Array.isArray(p)||p.length!==2||!p.every(n=>integer(n,32767))||p[0]>=m.width*16||p[1]>=m.height*16)return null;result.nativeClick=[...p];if(own(request,'nativeFlagMode')){if(result.kind!=='move'||typeof request.nativeFlagMode!=='boolean')return null;result.nativeFlagMode=request.nativeFlagMode;}}
   if(own(request,'nativePrimary')){if(!integer(request.nativePrimary)||!result.ids.includes(request.nativePrimary))return null;result.nativePrimary=request.nativePrimary;}
   if(own(request,'impactX')||own(request,'impactY')){const x=request.impactX,y=request.impactY,m=state.map;if(result.kind!=='attackBuilding'||![x,y].every(Number.isFinite)||x<0||y<0||x>m.width*32||y>m.height*32)return null;result.impactX=x;result.impactY=y;}
   if(['move','attackBridge','repairBridge'].includes(result.kind)&&!own(result,'x'))return null;
   if(result.kind==='attack'&&!own(result,'target')&&!own(result,'x'))return null;
  }else{
   if(!integer(request.id))return null;
   const b=state.buildings.find(b=>b.id===request.id&&b.team===team&&(b.kind==='fort'&&typeof b.nativeFortDestroyed==='boolean'?!b.nativeFortDestroyed:b.hp>0));if(!b)return null;result.id=request.id;
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
  let hostAudio=null,guestAudio=null,ownAudio=null,audioSequence=0,lastGuestAudioSequence=-1;
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
  const localAudio=value=>object(value)&&typeof value.soundEnabled==='boolean'?{soundEnabled:value.soundEnabled}:null;
  const wireAudio=value=>object(value)&&Object.keys(value).length===1&&typeof value.soundEnabled==='boolean'?{soundEnabled:value.soundEnabled}:null;
  const sameAudio=(a,b)=>a?.soundEnabled===b?.soundEnabled;
  function audioRequest(request){return {...request,nativeListenerAudio:{...(hostAudio?{'0':hostAudio}:{}),...(guestAudio?{'1':guestAudio}:{})}};}
  function setNativeAudioSettings(value){
   if(!active||!session?.connected)return false;const settings=localAudio(value);if(!settings)return false;if(sameAudio(settings,ownAudio))return true;
   if(role==='guest'){const seq=audioSequence+1;if(!send('game:audioSettings',{epoch,seq,settings}))return false;audioSequence=seq;ownAudio=settings;return true;}
   ownAudio=settings;hostAudio=settings;if(canonical)run({op:'audioSettings'});return true;
  }
  function refreshHostAudio(){const settings=localAudio(options.nativeAudioSettings?.());if(settings&&!sameAudio(settings,ownAudio)){ownAudio=settings;hostAudio=settings;}}
  function run(request){
   const compact=canonical?{compact:true,knownMapSeed:canonical.map.seed,knownMapRevision:canonical.map.revision}:{};
   const result=options.run(audioRequest({...request,...compact}));if(result?.error)throw Error(String(result.error));
   return apply(result);
  }
  function mapStamp(){return canonical?canonical.map.seed+':'+canonical.map.revision:'';}
  function wallStamp(){return (canonical?.walls||[]).map(w=>`${w.id}:${w.hp}:${w.neighborMask??0}:${w.wallTileIndex??0}`).join('|');}
  function dynamic(events){const result={...canonical,events};delete result.catalog;if(mapStamp()===lastMap){delete result.objects;delete result.map;}if(wallStamp()===lastWalls)delete result.walls;return result;}
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
   actionBudget=40;previewBudget=20;pauseBudget=4;budgetAt=Date.now();hostAudio=guestAudio=ownAudio=null;audioSequence=0;lastGuestAudioSequence=-1;return api;
  }
  function start(initRequest,meta={}){
   if(role!=='host'||!session?.connected)return false;
   clearTimeout(readyTimer);epoch++;sequence=0;receivedSequence=-1;canonical=view=null;frameClock=0;pendingEvents=[];active=true;paused=true;awaitingReady=true;
   try{
    ownAudio=localAudio(options.nativeAudioSettings?.());hostAudio=ownAudio;guestAudio=null;audioSequence=0;lastGuestAudioSequence=-1;
    const request=audioRequest({...initRequest,op:'init',multiplayer:true,difficulty:1,aiTeams:[],compact:false});
    const result=options.run(request);if(result?.error)throw Error(String(result.error));options.onNativeAudioSession?.();apply(result);
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
   try{refreshHostAudio();const result=run({op:'tick',dt:Math.min(.1,dt)});frameClock+=Math.min(.1,dt);if(frameClock>=FRAME_STEP){frameClock%=FRAME_STEP;broadcast();}if(canonical.winner!==null){broadcast(true);commitPause(true,'Schlacht beendet.');}return result;}catch(error){fail(error.message);return null;}
  }
  function tickUi(dt){
   if(role!=='host'||!active||!paused||awaitingReady||!session?.connected||!canonical?.map.original||!Number.isFinite(dt)||dt<=0)return null;
   try{refreshHostAudio();const result=run({op:'uiTick',dt:Math.min(.1,dt)});frameClock+=Math.min(.1,dt);if(frameClock>=FRAME_STEP){frameClock%=FRAME_STEP;broadcast();}return result;}catch(error){fail(error.message);return null;}
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
     epoch=p.epoch;receivedSequence=0;active=true;paused=true;awaitingReady=false;frameClock=0;pendingEvents=[];canonical=null;options.onNativeAudioSession?.();apply(p.state,false);
     const meta={mission:canonical.mission,difficulty:1,seed:p.meta.seed,original:Boolean(canonical.map.original),name:String(p.meta.name||'Mehrspielerpartie').slice(0,96)};
     options.onStart?.(meta,role);options.onPause?.(true,'Warte auf Gastgeber …');ownAudio=localAudio(options.nativeAudioSettings?.());audioSequence=0;send('game:ready',{epoch,...(ownAudio?{nativeAudio:ownAudio,audioSeq:0}:{})});return true;
    }
    if(!active||p.epoch!==epoch)return false;
    if(type==='game:audioSettings'){
     if(role!=='host'||awaitingReady||!integer(p.seq)||p.seq<=lastGuestAudioSequence)return false;
     const settings=wireAudio(p.settings);if(!settings)return false;
     lastGuestAudioSequence=p.seq;guestAudio=settings;run({op:'audioSettings'});return true;
    }
    if(type==='game:ready'){
     if(role!=='host'||!awaitingReady)return false;if(p.nativeAudio!==undefined){const settings=wireAudio(p.nativeAudio);if(!settings||p.audioSeq!==0)return false;guestAudio=settings;lastGuestAudioSequence=0;run({op:'audioSettings'});}clearTimeout(readyTimer);awaitingReady=false;commitPause(false,'Partie gestartet.');info('playing','Mehrspielerpartie gestartet.');return true;
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
  const api=Object.freeze({bind,start,handle,action,tick,tickUi,setNativeAudioSettings,requestPreview,setPaused,handleTransportStatus,close,get state(){return canonical;},get viewState(){return view;},get role(){return role;},get isHost(){return role==='host';},get active(){return active;},get paused(){return paused;},get waiting(){return awaitingReady;}});
  return api;
 }
 global.MultiplayerGame=Object.freeze({create,perspective,validateAction});
})(window);
