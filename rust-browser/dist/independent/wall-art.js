// New HD stonework over the original map's recovered wall masks. Pixel mode
// uses the native BLK tiles reconstructed by the importer and simulation.
window.WallArt=(()=>{
 'use strict';
 const cache=new Map(),LIMIT=128,DENSITY=2;
 const palettes=[['#b7aa88','#8f8369','#5c5546','#dad0ae'],['#8c8274','#665f56','#393b39','#b5aa91'],['#cad9d9','#9aafb4','#536d79','#edf3e8'],['#a4aa8d','#7c8973','#465e50','#d0d5b1'],['#adb4ab','#7b8883','#3e5659','#d4dbcb']];
 const key=w=>`${w.wallRecord??0}:${Math.round(w.x)}:${Math.round(w.y)}`;
 function path(c,points,fill,stroke,width=.6){c.beginPath();points.forEach((p,i)=>i?c.lineTo(...p):c.moveTo(...p));c.closePath();c.fillStyle=fill;c.fill();if(stroke){c.strokeStyle=stroke;c.lineWidth=width;c.stroke();}}
 function line(c,x,y,xx,yy,color,width=.7){c.beginPath();c.moveTo(x,y);c.lineTo(xx,yy);c.strokeStyle=color;c.lineWidth=width;c.stroke();}
 function hash(a,b){let n=Math.imul(a+19,374761393)^Math.imul(b+31,668265263);n=Math.imul(n^(n>>>13),1274126177);return((n^(n>>>16))>>>0)/4294967296;}
 function gradient(c,x,y,xx,yy,colors){const g=c.createLinearGradient(x,y,xx,yy);colors.forEach((color,i)=>g.addColorStop(i/(colors.length-1),color));return g;}
 function tile(world,mask,variant,damaged,dead){
  const stamp=[world,mask,variant,damaged,dead].join(':');let image=cache.get(stamp);if(image){cache.delete(stamp);cache.set(stamp,image);return image;}
  const p=palettes[world]||palettes[0];image=document.createElement('canvas');image.width=44*DENSITY;image.height=108*DENSITY;const c=image.getContext('2d');c.scale(DENSITY,DENSITY);c.translate(4,4);
  if(dead){
   for(let i=0;i<16;i++){const q=hash(i,variant),x=q*29,y=10+hash(variant,i+41)*75,r=1.5+q*3.5;path(c,[[x-r,y],[x-r*.6,y-r],[x+r*.5,y-r*.6],[x+r,y+r*.5],[x-r*.4,y+r]],i%3?p[1]:p[0],p[2],.35);}
  }else{
   const south=Boolean(mask&4),east=Boolean(mask&2),west=Boolean(mask&8);
   if(!south){
    const shadow=c.createLinearGradient(0,85,0,103);shadow.addColorStop(0,'#101c1b70');shadow.addColorStop(1,'#101c1b00');c.fillStyle=shadow;c.fillRect(3,86,33,17);
    path(c,[[0,25],[32,25],[32,91],[0,91]],gradient(c,0,25,22,94,[p[1],p[2]]),p[2]);
    for(let row=0;row<6;row++){
     const y=27+row*10.5,offset=row%2?-8:0;
     for(let col=-1;col<4;col++){
      const x=col*15+offset,left=Math.max(.3,x+.7),right=Math.min(31.7,x+14.1);if(right<=left)continue;
      const q=hash(row*7+col+variant,world),g=gradient(c,left,y,right,y+9,[q>.5?p[0]:p[1],p[1],p[2]]);
      path(c,[[left,y+.8],[right-.6,y],[right,y+8.5],[left+.5,y+9]],g,p[2],.55);line(c,left+1,y+1.5,right-1,y+1.1,p[3]+'69',.55);
      for(let n=0;n<4;n++){const xx=left+hash(n+col,row+variant)*(right-left),yy=y+2+hash(row+n,col+variant)*5;line(c,xx,yy,xx+1.2,yy+.2,p[2]+'70',.35);}
     }
    }
    if(!east)path(c,[[28,26],[32,24],[32,91],[28,88]],p[2]+'a0');
    line(c,0,91,32,91,'#17251eaa',1.3);
   }
   path(c,[[west?0:1,1],[east?32:30,1],[32,7],[32,28],[0,28],[0,7]],gradient(c,0,0,28,30,[p[3],p[0],p[1]]),p[2],.75);
   line(c,1,2,30,2,p[3],1.3);line(c,1,26,31,26,p[2]+'a0',1.1);
   for(let i=0;i<18;i++){const x=2+hash(i,variant)*27,y=4+hash(variant,i+33)*19;line(c,x,y,x+.8,y+.2,i%2?p[3]+'70':p[2]+'65',.45);}
   line(c,15,2,16,26,p[2]+'70',.7);if(!west)line(c,1,4,1,26,p[3]+'c0',.7);
   if(damaged){line(c,7,4,11,11,p[2],1);line(c,11,11,9,18,p[2],.9);line(c,9,18,15,26,p[2],.9);if(!south){line(c,15,29,18,48,p[2],1.1);line(c,18,48,14,65,p[2],1.1);}}
  }
  cache.set(stamp,image);while(cache.size>LIMIT)cache.delete(cache.keys().next().value);return image;
 }
 function draw(ctx,state,viewport,forceHD=false){
  if((!forceHD&&!window.GameGraphics?.isHD())||!state?.walls?.length)return;
  const live=new Map(state.walls.filter(w=>w.hp>0).map(w=>[key(w),w])),world=state.map.world;
  const visible=w=>!viewport||(w.x+44>=viewport.x&&w.y+108>=viewport.y&&w.x<=viewport.x+viewport.w&&w.y<=viewport.y+viewport.h);
  const walls=state.walls.filter(visible).sort((a,b)=>(a.hp>0)-(b.hp>0)||a.y-b.y||a.x-b.x);
  ctx.save();ctx.imageSmoothingEnabled=true;
  for(const w of walls){// Native bits mark exposed sides; this painter uses connected neighbours.
   let mask=Number.isInteger(w.neighborMask)?(~w.neighborMask&15):0;if(!Number.isInteger(w.neighborMask))for(const[dx,dy,bit]of[[0,-32,1],[32,0,2],[0,32,4],[-32,0,8]])if(live.has(key({...w,x:w.x+dx,y:w.y+dy})))mask|=bit;
   const variant=((w.variant??w.id??0)%4+4)%4,image=tile(world,mask,variant,w.hp>0&&w.hp<w.maxHp*.65,w.hp<=0);ctx.drawImage(image,w.x-4,w.y-4,44,108);
  }
  ctx.restore();
 }
 return Object.freeze({draw,clear(){cache.clear();},get cacheSize(){return cache.size;}});
})();
