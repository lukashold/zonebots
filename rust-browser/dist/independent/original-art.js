// Original GOG resources are loaded from a private/locally extracted pack, never embedded in source.
window.OriginalArt=(()=>{
 const assetBase=new URL('../original/',document.currentScript.src);
 let manifest,atlas,tiles,pixels,ready=false;const cache=new Map(),terrainCache=new Map();
 const worlds=['DESERT','VOLCAN','ARCTIC','JUNGLE','CITY'];
 const FORT={DESERT:[809,1675],ARTIC:[2127,2148],ARCTIC:[2127,2148],CITY:[2304,2431],JUNGLE:[2613,2725],VOLCAN:[2800,2845]};
 const HULL={light:5719,medium:488,heavy:5305,apc:5655},TURRET={light:5731,medium:513,heavy:5329};
 const attack={grunt:[127,5],tough:[226,3],sniper:[127,5],psycho:[2570,2],pyro:[6022,3],laser:[1340,3]};
 const dir=a=>((Math.round((a+Math.PI/2)/(Math.PI/4))%8)+8)%8;
 function cv(w,h){const c=document.createElement('canvas');c.width=w;c.height=h;return c;}
 function sprite(id,world,team=0){const key=`${id}:${world}:${team}`;if(cache.has(key))return cache.get(key);const f=manifest.frames[id];if(!f||!f.w||!f.h)return null;const c=cv(f.w,f.h),ctx=c.getContext('2d'),img=ctx.createImageData(f.w,f.h),pal=manifest.palettes[world]||manifest.palettes[0];for(let y=0;y<f.h;y++)for(let x=0;x<f.w;x++){const at=((f.y+y)*atlas.width+f.x+x)*4;let index=pixels[at];if(!pixels[at+3])continue;if(index>=16&&index<32)index=team===1?index+16:team<0?index+64:index;const p=(y*f.w+x)*4;img.data[p]=pal[index*3];img.data[p+1]=pal[index*3+1];img.data[p+2]=pal[index*3+2];img.data[p+3]=255;}ctx.putImageData(img,0,0);cache.set(key,c);return c;}
 function centered(ctx,id,u,map,offsetY=0){const im=sprite(id,map.world,u.team);if(!im)return false;ctx.drawImage(im,Math.round(u.x-im.width),Math.round(u.y-im.height+offsetY),im.width*2,im.height*2);return true;}
 function unit(ctx,u,time,map){if(!ready||!map.original)return false;const d=dir(u.angle),aim=dir(u.aim),walking=u.moving?Math.floor(time*8)%3:0;
  if(u.team<0&&manifest.neutralUnits?.[u.type]!==undefined)return native(ctx,manifest.neutralUnits[u.type],u.x,u.y,map);
  if(manifest.unitLayouts?.[u.type])return vehicle(ctx,u,time,map,d,aim,walking);
  if(attack[u.type]){const seq=attack[u.type],firing=u.cooldown>0&&u.order!=='move',frame=firing?Math.min(seq[1]-1,Math.floor(time*12)%seq[1]):0,id=u.moving?74+d*4+(Math.floor(time*8)%4):firing?seq[0]+aim*seq[1]+frame:66+d,im=sprite(id,map.world,u.team);if(!im)return false;ctx.drawImage(im,Math.round(u.x-16),Math.round(u.y-30),im.width*2,im.height*2);return true;}
  if(u.type==='jeep')return centered(ctx,u.team<0?5606+d:5590+d*2+(u.moving?Math.floor(time*8)%2:0),u,map);
  if(HULL[u.type]){centered(ctx,HULL[u.type]+(u.type==='apc'?d:d%4)*3+walking,u,map);if(TURRET[u.type])centered(ctx,TURRET[u.type]+aim,u,map);return true;}
  if(u.type==='launcher'){centered(ctx,1845+d*3+walking,u,map);centered(ctx,1869+aim,u,map);return true;}
  if(u.type==='crane')return centered(ctx,u.team<0?5834:5834+d*3+walking,u,map);
  if(u.type==='truck'||u.type==='firetruck')return centered(ctx,u.type==='truck'?5213:5215,u,map);
  if(u.type==='gatling')return centered(ctx,330+aim,u,map);
  if(u.type==='cannon')return centered(ctx,u.team<0?1651+aim:1659+2*aim+(u.cooldown>0?1:0),u,map);
  if(u.type==='howitzer')return centered(ctx,u.team<0?5759+aim:5767+2*aim+(u.cooldown>0?1:0),u,map);
  if(u.type==='missile')return centered(ctx,(u.team<0?1635:1643)+aim,u,map);
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
 function objectDefinition(o){const d=manifest?.objectTypes[o.type];return d&&d.flags!==0x32&&d.flags!==0x31?d:null;}
 function object(ctx,o,map){if(!ready||!map.original)return false;const d=objectDefinition(o);return d?native(ctx,d.sprite,o.x,o.y,map):false;}
 function objectShadows(ctx,objects,map){if(!ready||!map.original)return false;for(const o of objects){const d=objectDefinition(o);if(d&&d.shadowSprite>=0)native(ctx,d.shadowSprite,o.x,o.y,map);}return true;}
 function crate(ctx,box,map){if(!ready||!map.original||box.taken)return false;const d=manifest.objectTypes[10];return d?native(ctx,d.sprite,box.x,box.y,map):false;}
 function objectDepth(o){return o.y-(manifest?.objectTypes[o.type]?.depthOffset||0)*2;}
 function terrain(ctx,map){if(!ready||!map.original||!map.originalTiles)return false;const key=`${map.seed}:${map.revision}`;let image=terrainCache.get(key);if(!image){image=cv(map.width*16,map.height*16);const c=image.getContext('2d');for(let y=0;y<map.height;y++)for(let x=0;x<map.width;x++){const j=y*map.width+x;let i=map.originalTiles[j];if(map.bridgeBase[j]>=0&&map.tiles[j]===3)i=map.waterTile||i;c.drawImage(tiles,(i%32)*16,map.world*256+Math.floor(i/32)*16,16,16,x*16,y*16,16,16);}terrainCache.clear();terrainCache.set(key,image);}ctx.drawImage(image,0,0,map.width*32,map.height*32);return true;}
 function flag(ctx,s,time,map){if(!ready||!map.original)return false;const frame=Math.floor(time*6)%3,im=sprite(1234+(s.owner===0?0:s.owner===1?4:8)+frame,map.world,s.owner);if(!im)return false;ctx.drawImage(im,s.x-2,s.y-26,im.width*2,im.height*2);return true;}
 function effects(ctx,fx,time,map){if(!ready||!map.original)return false;for(const f of fx){if(f.kind!=='explosion')continue;const seq=f.radius>=26?[473,479]:[395,406],id=seq[0]+Math.min(seq[1]-seq[0],Math.floor(f.age/f.life*(seq[1]-seq[0]+1))),im=sprite(id,map.world);if(im)ctx.drawImage(im,f.x-im.width,f.y-im.height,im.width*2,im.height*2);}return true;}
 async function load(){const base=assetBase;const r=await fetch(new URL('sprites.json',base));if(!r.ok)return false;manifest=await r.json();[atlas,tiles]=await Promise.all(['sprites.png','tiles.png'].map(name=>new Promise((resolve,reject)=>{const i=new Image();i.onload=()=>resolve(i);i.onerror=reject;i.src=new URL(name,base);})));const c=cv(atlas.width,atlas.height),x=c.getContext('2d',{willReadFrequently:true});x.drawImage(atlas,0,0);pixels=x.getImageData(0,0,c.width,c.height).data;ready=true;return true;}
 return {load,unit,building,terrain,flag,effects,object,objectShadows,objectDepth,crate,get ready(){return ready;}};
})();
