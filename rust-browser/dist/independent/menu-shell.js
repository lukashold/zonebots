// Original menu textures are supplied only by the private/local resource pack.
window.MenuShell=(()=>{
 'use strict';
 const script=document.currentScript.src,base=new URL('../original/',script),menu=document.getElementById('menu');
 const pages=Array.from(document.querySelectorAll('[data-menu-page]'));
 function show(name){
  if(!pages.some(page=>page.dataset.menuPage===name))name='home';
  pages.forEach(page=>{page.hidden=page.dataset.menuPage!==name;});
  if(menu)menu.dataset.page=name;
 }
 for(const button of document.querySelectorAll('[data-menu-open]'))button.addEventListener('click',()=>show(button.dataset.menuOpen));
 document.getElementById('fullscreenHelpClose')?.addEventListener('click',()=>{document.getElementById('fullscreenHelp').hidden=true;});
 async function load(){
  try{
   const response=await fetch(new URL('menu.json',base));if(!response.ok)return false;
   const manifest=await response.json(),assets=manifest.assets||{};
   for(const key of ['background','title','panel']){
    const asset=assets[key];if(!asset||!asset.path?.startsWith('menu/'))continue;
    const url=new URL(asset.path,base);document.documentElement.style.setProperty('--menu-'+key,`url(${JSON.stringify(url.href)})`);
    if(key==='panel'&&asset.content?.[2]>0&&asset.content?.[3]>0)document.documentElement.style.setProperty('--menu-panel-size',`${asset.width/asset.content[2]*100}% ${asset.height/asset.content[3]*100}%`);
   }
   if(assets.title?.path?.startsWith('menu/')){
    try{
     await new Promise((resolve,reject)=>{const image=new Image();image.onload=resolve;image.onerror=reject;image.src=new URL(assets.title.path,base);});
     document.documentElement.classList.add('original-menu-ready');
    }catch{/* Keep the readable Z fallback if the private title image is missing. */}
   }
   const labelAsset=assets.menuText;
   if(labelAsset){
    const image=await new Promise((resolve,reject)=>{const image=new Image();image.onload=()=>resolve(image);image.onerror=reject;image.src=new URL(labelAsset.path,base);});
    for(const button of document.querySelectorAll('[data-original-label]')){
     const definition=manifest.labels?.[button.dataset.originalLabel],canvas=button.querySelector('canvas');if(!definition||!canvas)continue;
     const ctx=canvas.getContext('2d');
     function paint(highlight=false){const r=highlight?definition.hover:definition.normal;if(!r)return;ctx.clearRect(0,0,canvas.width,canvas.height);ctx.imageSmoothingEnabled=false;const scale=Math.min(1.1,(canvas.width-6)/r[2],(canvas.height-4)/r[3]);ctx.drawImage(image,...r,(canvas.width-r[2]*scale)/2,(canvas.height-r[3]*scale)/2,r[2]*scale,r[3]*scale);}
     paint();button.classList.add('has-original-label');
     button.addEventListener('pointerenter',()=>paint(true));button.addEventListener('pointerleave',()=>paint());
     button.addEventListener('focus',()=>paint(true));button.addEventListener('blur',()=>paint());
    }
   }
   return true;
  }catch{return false;}
 }
 show('home');load();
 return Object.freeze({show,load});
})();
