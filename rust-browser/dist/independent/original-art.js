// Original GOG resources are loaded from a private/locally extracted pack, never embedded in source.
window.OriginalArt=(()=>{
 const assetBase=new URL('../original/',document.currentScript.src);
 let manifest,atlas,tiles,pixels,ready=false;const cache=new Map(),terrainCache=new Map(),cursorCache=new Map(),entryCursorCache=new Map(),attackCursorCache=new Map();
 const worlds=['DESERT','VOLCAN','ARCTIC','JUNGLE','CITY'];
 const FORT={DESERT:[809,1675],ARTIC:[2127,2148],ARCTIC:[2127,2148],CITY:[2304,2431],JUNGLE:[2613,2725],VOLCAN:[2800,2845]};
 const HULL={light:5719,medium:488,heavy:5305,apc:5655},TURRET={light:5731,medium:513,heavy:5329};
 const attack={grunt:[127,5],tough:[226,3],sniper:[127,5],psycho:[2570,2],pyro:[6022,3],laser:[1340,3]};
 const guns=new Set(['gatling','cannon','howitzer','missile']);
 const dir=a=>((Math.round((a+Math.PI/2)/(Math.PI/4))%8)+8)%8;
 function cv(w,h){const c=document.createElement('canvas');c.width=w;c.height=h;return c;}
 function sprite(id,world,team=0){const key=`${id}:${world}:${team}`;if(cache.has(key))return cache.get(key);const f=manifest.frames[id];if(!f||!f.w||!f.h)return null;const c=cv(f.w,f.h),ctx=c.getContext('2d'),img=ctx.createImageData(f.w,f.h),pal=manifest.palettes[world]||manifest.palettes[0];for(let y=0;y<f.h;y++)for(let x=0;x<f.w;x++){const at=((f.y+y)*atlas.width+f.x+x)*4;let index=pixels[at];if(!pixels[at+3])continue;if(index>=16&&index<32)index=team===1?index+16:team<0?index+64:index;const p=(y*f.w+x)*4;img.data[p]=pal[index*3];img.data[p+1]=pal[index*3+1];img.data[p+2]=pal[index*3+2];img.data[p+3]=255;}ctx.putImageData(img,0,0);cache.set(key,c);return c;}
 function centered(ctx,id,u,map,offsetY=0){const im=sprite(id,map.world,u.team);if(!im)return false;ctx.drawImage(im,Math.round(u.x-im.width),Math.round(u.y-im.height+offsetY),im.width*2,im.height*2);return true;}
 function gunFrame(u){const aim=dir(u.aim);if(u.team<0&&manifest?.neutralUnits?.[u.type]!==undefined)return manifest.neutralUnits[u.type];return ({gatling:330+aim,cannon:u.team<0?1651+aim:1659+2*aim+(u.cooldown>0?1:0),howitzer:u.team<0?5759+aim:5767+2*aim+(u.cooldown>0?1:0),missile:(u.team<0?1635:1643)+aim})[u.type];}
 //00455f10 queues raw X; queue elevation is cancelled again in004549f0.
 // These are top-left sprite origins, including for the fort's fixed gun sockets.
 function unitBounds(u,map,time=0){if(!ready||!map?.original)return null;if(u.team<0&&manifest.neutralUnits?.[u.type]!==undefined){const f=manifest.frames[manifest.neutralUnits[u.type]];return f?{x:Math.round(u.x),y:Math.round(u.y),w:f.w*2,h:f.h*2}:null;}if(u.type==='jeep'){const layers=jeepLayers(u,time,map).map(p=>({...p,x:Math.round(p.x),y:Math.round(p.y),frame:manifest.frames[p.id]})).filter(p=>p.frame);if(!layers.length)return null;const x=Math.min(...layers.map(p=>p.x)),y=Math.min(...layers.map(p=>p.y));return {x,y,w:Math.max(...layers.map(p=>p.x+p.frame.w*2))-x,h:Math.max(...layers.map(p=>p.y+p.frame.h*2))-y};}if(!guns.has(u.type))return null;const f=manifest.frames[gunFrame(u)];return f?{x:Math.round(u.x),y:Math.round(u.y),w:f.w*2,h:f.h*2}:null;}
 // Contextual attacks use the whole currently rendered unit, including heads,
 // barrels and crane arms. Collect renderer rectangles instead of duplicating
 // its offsets; the older selection/portrait bounds keep their existing contract.
 function attackBounds(u,map,time=0){
  if(!ready||!map?.original||!u||!Number.isFinite(u.x)||!Number.isFinite(u.y))return null;
  const neutral=u.team<0&&manifest.neutralUnits?.[u.type]!==undefined;
  if(!neutral&&(!Number.isFinite(u.angle)||!Number.isFinite(u.aim)))return null;
  const known=unitBounds(u,map,time);if(known)return known;
  let x=Infinity,y=Infinity,right=-Infinity,bottom=-Infinity;
  const collector={drawImage(image,left,top,w,h){if(![left,top,w,h].every(Number.isFinite)||w<=0||h<=0)return;x=Math.min(x,left);y=Math.min(y,top);right=Math.max(right,left+w);bottom=Math.max(bottom,top+h);}};
  unit(collector,u,time,map);
  return Number.isFinite(x)?{x,y,w:right-x,h:bottom-y}:null;
 }
 function fortGuns(b,units){return b.kind==='fort'?units.filter(u=>u.fortId===b.id&&guns.has(u.type)):[];}
 //00455440: the body already contains the seated driver. Wheels and the
 // independently aimed weapon are separate layers. IDs, order and offsets are
 // from Ghidra/ASM; phase timing and firing still use the browser engine clock.
 function jeepLayers(u,time,map){
  if(!ready||!map?.original)return [];
  if(u.team<0)return [{id:manifest.neutralUnits?.jeep??5214,x:u.x,y:u.y}];
  const p=manifest.unitLayouts?.jeep,d=dir(u.angle),aim=dir(u.aim);
  if(!p)return [{id:5590+d*2,x:u.x-12,y:u.y-12}];
  const tick=Math.max(0,Math.floor(time*17)),wheelPhase=u.moving?tick%4:0;
  const terrain=(map.terrainCodes?.[Math.floor(u.y/32)*map.width+Math.floor(u.x/32)]||0)&127;
  const bodyPhase=u.moving?(([7,51,23].includes(terrain)?wheelPhase>>1:wheelPhase)&1):(tick>>1&1);
  const x=u.x+p.anchor[0]*2,y=u.y+p.anchor[1]*2,parts=[];
  if(p.underlay[d])parts.push({id:p.underlay[d]+wheelPhase,x,y});
  parts.push({id:p.body+d*p.stride+bodyPhase,x,y});
  parts.push({id:p.weapon+aim*2+(u.cooldown>0?1:0),x:x+(p.weaponAnchor[0]+p.bodyOffsets[d][0]+p.aimOffsets[aim][0])*2,y:y+(p.weaponAnchor[1]+p.bodyOffsets[d][1]+p.aimOffsets[aim][1]-bodyPhase)*2});
  return parts;
 }
 function unit(ctx,u,time,map){if(!ready||!map.original)return false;const d=dir(u.angle),aim=dir(u.aim),walking=u.moving?Math.floor(time*8)%3:0;
  if(u.type==='jeep'){for(const p of jeepLayers(u,time,map))native(ctx,p.id,p.x,p.y,map,u.team<0?null:u.team);return true;}
  if(u.team<0&&manifest.neutralUnits?.[u.type]!==undefined)return native(ctx,manifest.neutralUnits[u.type],u.x,u.y,map);
  if(guns.has(u.type))return native(ctx,gunFrame(u),u.x,u.y,map,u.team);
  if(manifest.unitLayouts?.[u.type])return vehicle(ctx,u,time,map,d,aim,walking);
  if(attack[u.type]){const seq=attack[u.type],firing=u.cooldown>0&&u.order!=='move',frame=firing?Math.min(seq[1]-1,Math.floor(time*12)%seq[1]):0,id=u.moving?74+d*4+(Math.floor(time*8)%4):firing?seq[0]+aim*seq[1]+frame:66+d,im=sprite(id,map.world,u.team);if(!im)return false;ctx.drawImage(im,Math.round(u.x-16),Math.round(u.y-30),im.width*2,im.height*2);return true;}
  if(HULL[u.type]){centered(ctx,HULL[u.type]+(u.type==='apc'?d:d%4)*3+walking,u,map);if(TURRET[u.type])centered(ctx,TURRET[u.type]+aim,u,map);return true;}
  if(u.type==='launcher'){centered(ctx,1845+d*3+walking,u,map);centered(ctx,1869+aim,u,map);return true;}
  if(u.type==='crane')return centered(ctx,u.team<0?5834:5834+d*3+walking,u,map);
  if(u.type==='truck'||u.type==='firetruck')return centered(ctx,u.type==='truck'?5213:5215,u,map);
  return false;
 }
 function vehicle(ctx,u,time,map,d,aim,walking){
  const p=manifest.unitLayouts[u.type],x=u.x+p.anchor[0]*2,y=u.y+p.anchor[1]*2,hullDirection=d%p.directions,hullOffset=p.hullOffsets[hullDirection],hull=u.hp<Math.floor(u.maxHp/4)&&p.damagedHull?p.damagedHull:p.hull;
  const draw=(id,dx=0,dy=0)=>native(ctx,id,x+dx*2,y+dy*2,map,u.team);
  const turret=p.turretOffsets?.[aim],tx=turret?(p.turretAnchor[0]+turret[0]):0,ty=turret?(p.turretAnchor[1]+turret[1]):0;
  const armPhase=Math.max(0,Math.min(63,Math.trunc(u.cranePhase||0))),armBehind=u.type==='crane'&&d>=3&&d<=5;
  if(armBehind)draw(p.arm+armPhase,turret[0]+p.armOffsets[aim][0],turret[1]+p.armOffsets[aim][1]);
  draw(hull+hullDirection*p.stride+walking,hullOffset[0],hullOffset[1]);
  if(p.turret)draw(p.turret+aim,tx,ty);
  if(p.barrel)draw(p.barrel+aim,tx+p.barrelOffsets[aim][0],ty+p.barrelOffsets[aim][1]);
  if(p.driver)draw(p.driver+Math.floor(time*17)%8,p.driverOffsets[d][0],p.driverOffsets[d][1]);
  if(u.type==='crane'&&!armBehind)draw(p.arm+armPhase,tx+p.armOffsets[aim][0],ty+p.armOffsets[aim][1]);
  return true;
 }
 function native(ctx,id,x,y,map,team=null){const im=sprite(id,map.world,team);if(!im)return false;ctx.drawImage(im,Math.round(x),Math.round(y),im.width*2,im.height*2);return true;}
 function buildingPart(ctx,id,b,map,offset=[0,0],team=null){return native(ctx,id,b.x+offset[0]*2,b.y+offset[1]*2,map,team);}
 function stars(ctx,b,map,descriptor){const n=Math.trunc(b.stars);if(descriptor.stars&&descriptor.starsOffset&&n>0)buildingPart(ctx,descriptor.stars+n-1,b,map,descriptor.starsOffset,b.team);}
 function teamOverlay(ctx,b,map,descriptor,dead){if(b.team<0||b.team>3||(dead&&b.kind==='radar'))return;buildingPart(ctx,descriptor.overlay+(dead?1:0),b,map,descriptor.overlayOffset,b.team);}
 function machinery(ctx,b,time,map,descriptor){
  const parts=manifest.buildingMachinery?.[b.kind];if(!parts)return;
  // Sprite IDs, native offsets and damage gates are from Ghidra. The original
  // wall-clock animation cadence is unmeasured; phases use a deterministic clock.
  const tick=Math.max(0,Math.floor(time*17)),owned=b.team>=0&&b.team<4,max=b.maxHp||b.maxhp||b.hp;
  const above=n=>b.hp>max-Math.trunc(max/5)*n,healthy20=owned&&above(4),healthy40=owned&&above(3),productive=owned&&Boolean(b.product)&&b.progress>0;
  const part=(address,frame=0,repeatIndex=0)=>{const p=parts[address];if(!p)return;const f=((frame%p.frames.length)+p.frames.length)%p.frames.length,offset=[p.offset[0]+(p.step?.[0]||0)*repeatIndex,p.offset[1]+(p.step?.[1]||0)*repeatIndex+(p.yDelta?.[f]||0)];buildingPart(ctx,p.frames[f],b,map,offset,owned?b.team:-1);};
  if(b.kind==='vehicle'){
   stars(ctx,b,map,descriptor);part('0040b550',((tick+1)&15)>>2&1);part('0040b5a0',owned?tick%4:0);part('0040b5f0a',productive?tick&1:0);part('0040b5f0b',productive?tick%8:0);if(healthy20){part('0040b6b0a');part('0040b6b0b');}
  }else if(b.kind==='robot'){
   part('0040bc20',((tick+1)&15)===8?1:0);stars(ctx,b,map,descriptor);part('0040bce0',productive?Math.floor(tick/4)%8:0);part('0040bd50',healthy40?tick%12:0);
   const bits=healthy40?tick%63+1:1;for(let i=0;i<6;i++)part('0040bde0',bits>>i&1,i);
   if(healthy20)part('0040bf20');part('0040bf80',productive?tick%6:0);
   // 0040bfe0 truncates its damage threshold to a signed byte in Z.exe.
   const conveyorThreshold=(Math.trunc(max)-3*Math.trunc(max/5))<<24>>24;
   if(owned&&b.hp>conveyorThreshold)for(let i=0;i<6;i++)part('0040bfe0',tick&1,i);
  }else if(b.kind==='radar'){
   part('0040c8a0',healthy20?1:0);part('0040c930',tick&1);part('0040ca70',healthy40?tick%12:0);part('0040c9b0',healthy20?tick%8:0);
  }else if(b.kind==='repair'){
   part('0040cdf0',healthy40?1:0);part('0040ce60',above(3)?tick%3:0);part('0040ced0',healthy40?1:0);teamOverlay(ctx,b,map,descriptor,false);
   if(!owned)part('0040cda0');else if(tick<3)part('0040cda0',tick+1);
  }
 }
 function building(ctx,b,time,map){if(!ready||!map.original)return false;const world=worlds[map.world],dead=b.hp<=0;
  if(b.kind==='fort'){const base=FORT[world][dead?1:0],orientation=b.orientation==='north'?4:0,anchor=b.rawBase?[b.rawBase[0]*2,b.rawBase[1]*2]:[b.x,b.y];for(const [i,x,y]of [[0,0,16],[1,120,16],[orientation+2,0,0],[orientation+3,120,0],[orientation+4,0,112],[orientation+5,120,112]]){const im=sprite(base+i,map.world,b.team);if(im)ctx.drawImage(im,anchor[0]+x*2,anchor[1]+y*2,im.width*2,im.height*2);}return true;}
  const descriptor=manifest.buildings?.[b.kind]?.[world];if(!descriptor)return false;if(!buildingPart(ctx,dead?descriptor.destroyed:descriptor.intact,b,map))return false;
  if(dead){teamOverlay(ctx,b,map,descriptor,true);return true;}
  if(b.kind!=='repair')teamOverlay(ctx,b,map,descriptor,false);machinery(ctx,b,time,map,descriptor);return true;
 }
 // Production-clock fields use the original green font, independently of the
 // selected factory window. Native anchors and character advance are verified
 // against0040b6b0/0040c490 and0040ab90 ->0040ac30 ->00467b80.
 function productionText(b,seconds,time=0){
  if(!b||b.team<0||b.hp<=0||!b.product||!['vehicle','robot','fort'].includes(b.kind))return null;
  //0040ad60 holds68 native updates after completion. The producer draws
  //0:00 only when00622d40&2;00434a70 advances that phase every other update.
  //The17Hz global phase is the web clock adaptation, not a measured OS scheduler.
  if(b.completionTicks>0)return ((Math.max(0,Math.floor(time*17))+1)>>1&2)?'0:00':null;
  if(seconds===null)return '*:**';
  if(!Number.isFinite(seconds))return null;
  const value=Math.min(599,Math.max(0,Math.trunc(seconds)));
  return `${Math.floor(value/60)}:${String(value%60).padStart(2,'0')}`;
 }
 function production(ctx,b,seconds,map,time=0){
  if(!ready||!map.original)return false;
  const text=productionText(b,seconds,time);if(text===null)return false;
  const offset=({vehicle:[32,57],robot:[35,58],fort:[74,90]})[b.kind];
  const anchor=b.kind==='fort'&&b.rawBase?[b.rawBase[0]*2,b.rawBase[1]*2]:[b.x,b.y];
  let x=anchor[0]+offset[0]*2;const y=anchor[1]+offset[1]*2;
  //00467c37 indexes the original character-width table0049cc20: digits4,
  // colon/asterisk3 native pixels. Atlas frame widths include transparent pad.
  for(const ch of text){native(ctx,0x214+ch.charCodeAt(0),x,y,map);x+=(ch===':'||ch==='*'?3:4)*2;}
  return true;
 }
 function objectDefinition(o){const d=manifest?.objectTypes[o.type];return d&&d.flags!==0x32&&d.flags!==0x31?d:null;}
 function object(ctx,o,map){if(!ready||!map.original)return false;const d=objectDefinition(o);return d?native(ctx,d.sprite,o.x,o.y,map):false;}
 function objectShadows(ctx,objects,map){if(!ready||!map.original)return false;for(const o of objects){const d=objectDefinition(o);if(d&&d.shadowSprite>=0)native(ctx,d.shadowSprite,o.x,o.y,map);}return true;}
 function crateBounds(box,map){if(!ready||!map?.original||!box||box.taken||!Number.isFinite(box.x)||!Number.isFinite(box.y))return null;const d=manifest.objectTypes[10],f=d&&manifest.frames[d.sprite];return f&&f.w>0&&f.h>0?{x:Math.round(box.x),y:Math.round(box.y),w:f.w*2,h:f.h*2}:null;}
 function crate(ctx,box,map){if(!ready||!map.original||box.taken)return false;const d=manifest.objectTypes[10];return d?native(ctx,d.sprite,box.x,box.y,map):false;}
 function objectDepth(o){return o.y-(manifest?.objectTypes[o.type]?.depthOffset||0)*2;}
 function terrain(ctx,map){if(!ready||!map.original||!map.originalTiles)return false;const key=`${map.seed}:${map.revision}`;let image=terrainCache.get(key);if(!image){image=cv(map.width*16,map.height*16);const c=image.getContext('2d');for(let y=0;y<map.height;y++)for(let x=0;x<map.width;x++){const j=y*map.width+x;let i=map.originalTiles[j];if(map.bridgeBase[j]>=0&&map.tiles[j]===3)i=map.waterTile||i;c.drawImage(tiles,(i%32)*16,map.world*256+Math.floor(i/32)*16,16,16,x*16,y*16,16,16);}terrainCache.clear();terrainCache.set(key,image);}ctx.drawImage(image,0,0,map.width*32,map.height*32);return true;}
 //0041d110 queues map flags480..483 and ground shadows484..487, at the
 // territory's raw tile origin. Large1234/1238 banners belong to forts;
 //1242..1245 are their shadows. The four-frame cadence uses the web clock.
 //0040bb50 ->00465cb0 maps neutral cloth16..31 to grayscale80..95.
 function flagPhase(time){return Math.max(0,Math.floor(time*6))%4;}
 function flagBounds(s,time,map){if(!ready||!map?.original)return null;const f=manifest.frames[480+flagPhase(time)];return f?{x:Math.round(s.x-16),y:Math.round(s.y-16),w:f.w*2,h:f.h*2}:null;}
 function flag(ctx,s,time,map){if(!ready||!map.original)return false;return native(ctx,480+flagPhase(time),s.x-16,s.y-16,map,s.owner);}
 function flagShadows(ctx,sectors,time,map){if(!ready||!map?.original)return false;const id=484+flagPhase(time);for(const s of sectors)native(ctx,id,s.x-16,s.y-16,map);return true;}
 //00434c60 chooses cursor45 for a selectable flag; its native offset is(-9,-7).
 function captureCursor(map){if(!ready||!map?.original)return 'pointer';if(cursorCache.has(map.world))return cursorCache.get(map.world);const im=sprite(45,map.world,null);if(!im)return 'pointer';const c=cv(im.width*2,im.height*2),ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.drawImage(im,0,0,c.width,c.height);const value=`url("${c.toDataURL('image/png')}") 18 14, pointer`;cursorCache.set(map.world,value);return value;}
 //00441430 maps cursor45 to the four original grasping-hand frames2755..2758.
 // The original acknowledgement lasts23 draw ticks; the web's wall-clock cadence
 // is adapted, rather than inferred from the game's production clock.
 function captureHand(ctx,x,y,map,elapsed=0){if(!ready||!map?.original)return false;return native(ctx,2755+(Math.max(0,Math.floor(elapsed*8))%4),x-18,y-14,map);}
 //00435789 selects white entry arrow6 for eligible unoccupied hardware;
 //0043579b/004357a5 give native offsets(-7,-11), hence CSS hotspot(14,22).
 function entryCursor(map){if(!ready||!map?.original)return 'pointer';if(entryCursorCache.has(map.world))return entryCursorCache.get(map.world);const im=sprite(6,map.world,null);if(!im)return 'pointer';const c=cv(im.width*2,im.height*2),ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.drawImage(im,0,0,c.width,c.height);const value=`url("${c.toDataURL('image/png')}") 14 22, pointer`;entryCursorCache.set(map.world,value);return value;}
 //004414ae maps cursor6 to grayscale acknowledgement2833..2836. These
 // local coordinates identify the cursor hotspot; the web clock adapts timing.
 function entryArrow(ctx,x,y,map,elapsed=0){if(!ready||!map?.original)return false;return native(ctx,2833+(Math.max(0,Math.floor(elapsed*8))%4),x-14,y-22,map);}
 //00434c60 selects white attack crosshair4 with native offsets(-8,-8).
 // Red sprite5 is not assumed to encode range; that mapping is unverified.
 function attackCursor(map){if(!ready||!map?.original)return 'crosshair';if(attackCursorCache.has(map.world))return attackCursorCache.get(map.world);const im=sprite(4,map.world,null);if(!im)return 'crosshair';const c=cv(im.width*2,im.height*2),ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.drawImage(im,0,0,c.width,c.height);const value=`url("${c.toDataURL('image/png')}") 16 16, crosshair`;attackCursorCache.set(map.world,value);return value;}
 //00441430 maps cursor4 to acknowledgement2783..2786. The 8fps clock and
 // target-centered placement of persistent feedback are browser adaptations.
 function attackMarker(ctx,x,y,map,elapsed=0){if(!ready||!map?.original)return false;return native(ctx,2783+(Math.max(0,Math.floor(elapsed*8))%4),x-16,y-16,map);}
 function effects(ctx,fx,time,map){if(!ready||!map.original)return false;for(const f of fx){if(f.kind!=='explosion')continue;const seq=f.radius>=26?[473,479]:[395,406],id=seq[0]+Math.min(seq[1]-seq[0],Math.floor(f.age/f.life*(seq[1]-seq[0]+1))),im=sprite(id,map.world);if(im)ctx.drawImage(im,f.x-im.width,f.y-im.height,im.width*2,im.height*2);}return true;}
 async function load(){const base=assetBase;const r=await fetch(new URL('sprites.json',base));if(!r.ok)return false;manifest=await r.json();[atlas,tiles]=await Promise.all(['sprites.png','tiles.png'].map(name=>new Promise((resolve,reject)=>{const i=new Image();i.onload=()=>resolve(i);i.onerror=reject;i.src=new URL(name,base);})));const c=cv(atlas.width,atlas.height),x=c.getContext('2d',{willReadFrequently:true});x.drawImage(atlas,0,0);pixels=x.getImageData(0,0,c.width,c.height).data;ready=true;return true;}
 return {load,unit,unitBounds,attackBounds,jeepLayers,fortGuns,building,production,productionText,terrain,flag,flagShadows,flagBounds,captureCursor,captureHand,entryCursor,entryArrow,attackCursor,attackMarker,effects,object,objectShadows,objectDepth,crate,crateBounds,get ready(){return ready;}};
})();
