// Indexed original status-panel assets are supplied by the separate private pack.
window.OriginalHud=(()=>{
 'use strict';
 const base=new URL('../original/',document.currentScript.src),cache=new Map(),urls=new Map();
 let manifest,atlas,pixels,ready=false,lastState,lastUnit,lastPainter,skinWorld=-1;
 const captions={grunt:81,pyro:82,laser:83,tough:84,psycho:85,sniper:86};
 //00469478 indexes0049c6c0 by the selected animation family.00469190 draws
 // descriptor[0] as the caption, descriptor[1] as the hardware image, and
 // descriptor[2] as its native vertical offset. These are NOT equipment order.
 const hardware={cannon:[94,106,8],medium:[99,111,1],missile:[88,100,4],gatling:[92,104,5],howitzer:[90,102,5],launcher:[93,105,1],heavy:[91,103,2],light:[96,108,1],apc:[98,110,4],jeep:[89,101,4],crane:[95,107,3],truck:[97,109,5],firetruck:[97,109,5]};
 const $=id=>document.getElementById(id),mobile=()=>window.innerWidth<=760;
 function sidebarWidth(){return mobile()?0:ready?window.innerHeight/5:240;}
 function frame(id,world=0,team=0){
  const key=[id,world,team].join(':');if(cache.has(key))return cache.get(key);
  const f=manifest?.frames[id],palette=manifest?.palettes[world]||manifest?.palettes[0];if(!f||!pixels||!palette)return null;
  const canvas=document.createElement('canvas');canvas.width=f.w;canvas.height=f.h;
  const ctx=canvas.getContext('2d'),image=ctx.createImageData(f.w,f.h);
  for(let y=0;y<f.h;y++)for(let x=0;x<f.w;x++){
   const at=((f.y+y)*atlas.width+f.x+x)*4,out=(y*f.w+x)*4;if(!pixels[at+3])continue;
   let index=pixels[at];if(team===1&&index>=16&&index<32)index+=16;
   image.data[out]=palette[index*3];image.data[out+1]=palette[index*3+1];image.data[out+2]=palette[index*3+2];image.data[out+3]=255;
  }
  ctx.putImageData(image,0,0);cache.set(key,canvas);return canvas;
 }
 function paint(canvas,id,world=0,team=0){
  const image=frame(id,world,team);if(!canvas||!image)return false;
  const ctx=canvas.getContext('2d');ctx.clearRect(0,0,canvas.width,canvas.height);ctx.imageSmoothingEnabled=false;
  const scale=Math.min(canvas.width/image.width,canvas.height/image.height);
  ctx.drawImage(image,(canvas.width-image.width*scale)/2,(canvas.height-image.height*scale)/2,image.width*scale,image.height*scale);return true;
 }
 function texture(id,world=0){const key=id+':'+world;if(!urls.has(key)){const image=frame(id,world);if(!image)return '';urls.set(key,`url(${image.toDataURL()})`);}return urls.get(key);}
 function skin(button,id,world=0){if(!button)return;const url=texture(id,world);if(button.style.backgroundImage!==url)button.style.backgroundImage=url;}
 function resize(){
  document.documentElement.style.setProperty('--hud-scale',String(window.innerHeight/480));
  document.documentElement.style.setProperty('--sidebar-width',sidebarWidth()+'px');
  const mini=$('mini');
  if(ready&&!mobile()){
   if(mini.parentElement!==$('nativeHud'))$('nativeHud').appendChild(mini);
   const map=lastState?.map,ratio=map?map.width/map.height:1;
   const width=ratio>=1?88:88*ratio,height=ratio>=1?88/ratio:88;
   mini.style.width=width+'px';mini.style.height=height+'px';mini.style.left=(4+(88-width)/2)+'px';mini.style.top=(300+(88-height)/2)+'px';
   document.querySelector('.overlayButtons').style.bottom='auto';
  }else if(mobile()){
   if(mini.parentElement!==$('hud'))$('hud').appendChild(mini);
   const map=lastState?.map,ratio=map?map.width/map.height:1;
   const width=ratio>=1?168:168*ratio,height=ratio>=1?168/ratio:168;
   mini.style.width=width+'px';mini.style.height=height+'px';mini.style.left='';mini.style.top='';
  }
 }
 function update(state,unit,renderUnit){
  lastState=state;lastUnit=unit;lastPainter=renderUnit;if(!ready||!state)return;
  const world=state.map.world,portrait=$('portrait'),art=$('hudUnitArt'),hd=Boolean(window.GameGraphics?.isHD()),density=hd?Math.max(2,Math.min(4,Math.ceil(window.innerHeight/480*(window.devicePixelRatio||1)))):1;
  if(art.width!==96*density||art.height!==78*density){art.width=96*density;art.height=78*density;}
  const ctx=art.getContext('2d');if(hd)ctx.setTransform(density,0,0,density,0,0);else ctx.setTransform?.(1,0,0,1,0,0);
  if(skinWorld!==world){document.documentElement.style.setProperty('--hud-metal',texture(130,world));document.documentElement.style.setProperty('--hud-radio',texture(115,world));skinWorld=world;}
  ctx.clearRect(0,0,art.width,art.height);ctx.imageSmoothingEnabled=Boolean(window.GameGraphics?.isHD());
  const descriptor=hardware[unit?.type],background=frame(descriptor?114:115,world);
  if(background)ctx.drawImage(background,0,0,96,78);
  if(unit){
   if(descriptor&&!window.GameGraphics?.isHD()){const image=frame(descriptor[1],world,unit.team);if(image)ctx.drawImage(image,0,descriptor[2]*2);}
   else if(renderUnit){const preview=document.createElement('canvas');preview.width=preview.height=96*density;const pc=preview.getContext('2d');if(hd)pc.scale(density,density);renderUnit(pc,unit);ctx.drawImage(preview,9,0,78,78);}
  }
  const caption=descriptor?.[0]??captions[unit?.type];
  if(caption!==undefined)paint($('hudCaption'),caption,world,unit?.team||0);
  $('hudCaption').hidden=!unit||caption===undefined;
  $('unitName').style.color=unit&&caption===undefined?'#f2e9d5':'';
  // Static original communication portrait; do not invent speech/facial timing.
  const pc=portrait.getContext('2d');pc.clearRect(0,0,96,96);pc.imageSmoothingEnabled=false;
  const bg=frame(manifest.backgroundStart+[0,1,3,5,4][world],world);
  if(bg)pc.drawImage(bg,8,0,80,96);
  if(unit){const head=frame(manifest.headStart+2,world,unit.team);if(head)pc.drawImage(head,8,32,80,64);}
  const health=$('hudHealth'),hc=health.getContext('2d');hc.clearRect(0,0,80,6);
  const back=frame(64,world),bar=frame(62,world);if(back)hc.drawImage(back,0,0,80,6);
  if(unit&&bar){const fraction=Math.max(0,Math.min(1,unit.hp/Math.max(1,unit.maxHp)));hc.drawImage(bar,0,0,bar.width*fraction,bar.height,0,0,80*fraction,6);}
  skin($('nativeHud'),0,world);
  for(const [id,index] of Object.entries({cycleR:27,cycleG:30,cycleV:33,cycleB:24,overlayT:15,overlayD:18,overlayZ:21,hudMenu:36}))skin($(id),index+(id.startsWith('overlay')&&$(id).classList.contains('active')?1:0),world);
  $('portrait').title=unit?'Auswahl zentrieren: '+unit.type:'Keine Einheit ausgewählt';
  $('unitInfo').title=$('unitInfo').textContent;$('unitOrderInfo').title=$('unitOrderInfo').textContent;
  resize();
 }
 async function load(){
  try{
   const response=await fetch(new URL('hud.json',base));if(!response.ok)return false;
   const data=await response.json();if(!data.atlas?.startsWith('hud/')||!data.frames?.length||data.palettes?.length!==5)return false;
   const image=await new Promise((resolve,reject)=>{const image=new Image();image.onload=()=>resolve(image);image.onerror=reject;image.src=new URL(data.atlas,base);});
   const canvas=document.createElement('canvas');canvas.width=image.width;canvas.height=image.height;const ctx=canvas.getContext('2d');ctx.drawImage(image,0,0);
   atlas=image;pixels=ctx.getImageData(0,0,image.width,image.height).data;manifest=data;ready=true;
   document.documentElement.classList.add('original-hud-ready');resize();window.dispatchEvent(new Event('resize'));
   if(lastState)update(lastState,lastUnit,lastPainter);return true;
  }catch{return false;}
 }
 $('hudMenu')?.addEventListener('click',()=>$('pauseButton').click());
 window.addEventListener('resize',resize);load();
 return Object.freeze({load,update,resize,sidebarWidth,isReady:()=>ready});
})();
