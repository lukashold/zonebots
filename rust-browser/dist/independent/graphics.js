// Presentation only: both renderers share the same Rust simulation and native hit geometry.
window.GameGraphics=(()=>{
 'use strict';
 const original=window.OriginalArt,key='z.graphics.v1';
 let mode='original',previewScene=null;
 try{if(localStorage.getItem(key)==='hd')mode='hd';}catch{/* Storage may be disabled. */}
 const root=document.documentElement;
 const isHD=()=>mode==='hd';
 function reflect(){root.dataset.graphicsMode=mode;for(const select of document.querySelectorAll('[data-graphics-select]'))select.value=mode;for(const radio of document.querySelectorAll('[data-graphics-choice]'))radio.checked=radio.value===mode;for(const label of document.querySelectorAll('[data-graphics-current]'))label.textContent=isHD()?'HD-Remake':'Originalgrafik';}
 function setMode(value){if(value!=='original'&&value!=='hd')return false;const changed=mode!==value;mode=value;try{localStorage.setItem(key,mode);}catch{}reflect();if(changed)window.dispatchEvent(new Event('graphicschange'));return true;}
 function unit(ctx,u,time,map){if(!isHD())return original.unit(ctx,u,time,map);const bounds=original.attackBounds?.(u,map,time)||original.unitBounds?.(u,map,time);return window.HDUnits?.unit(ctx,u,time,map,bounds)??false;}
 function delegate(name,module,args){if(!isHD())return original[name]?.(...args)??false;return window[module]?.[name]?.(...args)??false;}
 const art={
  unit,
  terrain(ctx,map,time=0,viewport){return isHD()?(window.HDTerrain?.terrain(ctx,map,time,viewport)??false):original.terrain(ctx,map);},
  minimap(ctx,map){return isHD()?(window.HDTerrain?.minimap(ctx,map)??false):original.terrain(ctx,map);},
  building(...args){return delegate('building','HDStructures',args);},
  object(...args){return delegate('object','HDStructures',args);},
  objectShadows(...args){return delegate('objectShadows','HDStructures',args);},
  flag(...args){return delegate('flag','HDStructures',args);},
  flagShadows(...args){return delegate('flagShadows','HDStructures',args);},
  crate(ctx,box,map){if(!isHD())return original.crate(ctx,box,map);return window.HDStructures?.crate(ctx,box,map,original.crateBounds?.(box,map))??false;},
  effects(...args){return delegate('effects','HDStructures',args);},
  // The original production display is authoritative for completion holds and timing.
  production(...args){return original.production?.(...args)??false;}
 };
 function paintPreviews(){
  if(!previewScene||!original.ready)return;
  const map=previewScene.map,fort=previewScene.buildings.find(b=>b.kind==='fort'&&b.team===0);if(!fort)return;
  for(const canvas of document.querySelectorAll('[data-graphics-preview]')){
   const ctx=canvas.getContext('2d'),hd=canvas.dataset.graphicsPreview==='hd',scale=canvas.width/560,x=fort.x-110,y=fort.y-70;
   ctx.save();ctx.clearRect(0,0,canvas.width,canvas.height);ctx.imageSmoothingEnabled=hd;ctx.scale(scale,scale);ctx.translate(-x,-y);
   if(hd)window.HDTerrain?.terrain(ctx,map,0,{x,y,w:560,h:canvas.height/scale});else original.terrain(ctx,map);
   const b={...fort,product:'grunt'};
   if(hd)window.HDStructures?.building(ctx,b,0,map);else original.building(ctx,b,0,map);
   for(const spawn of previewScene.spawns.filter(s=>s.team===0)){
    const u={...spawn,type:spawn.type||spawn.t,angle:Math.PI/2,aim:Math.PI/2,moving:false,cooldown:0,hp:100,maxHp:100};
    if(hd)window.HDUnits?.unit(ctx,u,0,map,original.attackBounds?.(u,map,0));else original.unit(ctx,u,0,map);
   }
   ctx.restore();
  }
 }
 for(const select of document.querySelectorAll('[data-graphics-select]'))select.addEventListener('change',()=>setMode(select.value));
 for(const radio of document.querySelectorAll('[data-graphics-choice]'))radio.addEventListener('change',()=>{if(radio.checked)setMode(radio.value);});
 window.addEventListener('storage',event=>{if(event.key===key){mode=event.newValue==='hd'?'hd':'original';reflect();window.dispatchEvent(new Event('graphicschange'));}});
 reflect();
 return Object.freeze({art,isHD,setMode,get mode(){return mode;},setPreviewScene(scene){previewScene=scene;paintPreviews();},paintPreviews});
})();
