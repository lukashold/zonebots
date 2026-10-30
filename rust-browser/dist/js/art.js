// Original, procedural artwork. No extracted game images or external assets.
(function () {
  const ZG = globalThis.ZG, cache = new Map();
  const teams = [['#d84c32','#7a251e','#ff9861'],['#4276bd','#223b6a','#93c4ed'],['#918c79','#514d44','#cec4a3']];
  function make(w,h) { const c=document.createElement('canvas'); c.width=w;c.height=h;return c; }
  function memo(k,f) { if(!cache.has(k))cache.set(k,f());return cache.get(k); }
  const rnd=(x,y)=>{let n=Math.imul(x+17,374761393)^Math.imul(y+31,668265263);n=Math.imul(n^(n>>>13),1274126177);return ((n^(n>>>16))>>>0)/4294967296;};
  function rect(c,color,x,y,w,h){c.fillStyle=color;c.fillRect(Math.round(x),Math.round(y),Math.round(w),Math.round(h));}
  function oval(c,col,x,y,w,h){for(let j=0;j<h;j++){const q=Math.sqrt(Math.max(0,1-Math.pow((j+.5-h/2)/(h/2),2)));rect(c,col,x+w/2-q*w/2,y+j,Math.max(1,q*w),1);}}
  function plate(c,x,y,w,h,col,hi='#ccc5a6',lo='#42423c') {rect(c,lo,x,y,w,h);rect(c,col,x+1,y+1,w-2,h-2);rect(c,hi,x+1,y+1,w-2,1);rect(c,hi,x+1,y+1,1,h-2);}
  function rivets(c,x,y,w,h){for(const xx of [x+2,x+w-3])for(const yy of [y+2,y+h-3]){rect(c,'#343c3b',xx,yy,2,2);rect(c,'#d1cbb3',xx,yy,1,1);}}
  function stripes(c,x,y,w){rect(c,'#252a28',x,y,w,3);for(let i=0;i<w;i++)if(i%7<3)rect(c,'#d4ae42',x+i,y,1,3);}
  const Art=ZG.Art={};
  Art.robot=function(type,team,view,frame){return memo(`robot:${type}:${team}:${view}:${frame}`,()=>{
    const out=make(20,22),c=out.getContext('2d'),p=teams[team<0?2:team],side=view==='left'||view==='right',back=view==='up';
    const foot=frame===1?-1:frame===2?1:0, bob=frame?1:0;
    // Separate steel limbs, heavy boots and a broad, cylindrical metal head.
    rect(c,'#323b3b',6,16,3,4+foot);rect(c,'#89948b',6,16,1,3+foot);
    rect(c,'#343c3b',11,16,3,4-foot);rect(c,'#a3aaa0',11,16,1,3-foot);
    plate(c,5,19+foot,5,2,'#505954','#7a8176','#242c29');plate(c,11,19-foot,5,2,'#505954','#7a8176','#242c29');
    plate(c,6,9+bob,9,8,p[0],p[2],p[1]);rect(c,p[1],7,14+bob,7,2);rect(c,'#d0c4a0',9,13+bob,3,2);
    if(back){plate(c,7,10+bob,6,6,'#5e6860','#a3ac98','#303c33');for(let i=0;i<3;i++)rect(c,'#27322c',8,11+bob+i,4,1);}
    oval(c,'#202d2d',5,1+bob,11,9);oval(c,'#6e817c',6,1+bob,9,8);oval(c,'#b6c5ac',6,1+bob,8,5);
    rect(c,'#e1e3ba',8,1+bob,5,1);rect(c,'#4d625c',6,7+bob,8,1);
    if(!back){const ex=side?11:7;rect(c,'#263e36',ex,4+bob,side?4:7,2);rect(c,'#e3e59d',ex+1,4+bob,2,1);if(!side)rect(c,'#e3e59d',12,4+bob,1,1);rect(c,'#45574e',10,7+bob,3,1);}
    plate(c,3,10+bob,4,4,p[0],p[2],p[1]);rect(c,'#adb8a3',4,14+bob,2,3);
    plate(c,14,10+bob,3,4,p[0],p[2],p[1]);rect(c,'#b2bda6',15,14+bob,2,2);
    const wy=12+bob;
    if(type==='tough'){plate(c,9,8+bob,11,4,'#626b43','#a4ac72','#303e2c');rect(c,'#202c26',18,8+bob,2,4);}
    else if(type==='pyro'){plate(c,2,9+bob,3,7,'#6f7848','#b3b782','#343e2f');rect(c,'#67745b',13,wy,6,3);rect(c,'#d07b37',18,wy,2,2);}
    else if(type==='laser'){plate(c,12,wy,7,3,'#bbc7a7','#e4e6bf','#63746a');rect(c,'#a7eef0',18,wy,2,2);}
    else {rect(c,'#25302d',12,wy,type==='sniper'?8:6,2);rect(c,'#7b8b7c',12,wy,5,1);if(type==='psycho')rect(c,'#35463a',13,wy+2,4,2);if(type==='sniper')rect(c,'#bac2a1',14,wy-1,3,1);}
    if(view==='left'){const flip=make(20,22),fc=flip.getContext('2d');fc.translate(20,0);fc.scale(-1,1);fc.drawImage(out,0,0);return flip;}
    return out;
  });};
  Art.building=function(b){return memo(`building:${b.kind}:${b.team}:${b.tech}:${b.w}:${b.h}`,()=>{
    const w=b.w*16,h=b.h*16+10,out=make(w,h),c=out.getContext('2d'),p=teams[b.team<0?2:b.team];
    rect(c,'#403c30',3,12,w-3,h-12);
    function drum(x,y,d){
      oval(c,'#283532',x,y+8,d,12);rect(c,'#69756a',x,y+5,d,12);
      rect(c,'#8e9886',x+2,y+5,3,12);rect(c,'#414e46',x+d-4,y+5,3,12);
      for(let j=8;j<17;j+=4)rect(c,'#48584e',x+1,y+j,d-2,1);
      oval(c,'#34483e',x,y,d,12);oval(c,'#bbc2a4',x+1,y+1,d-2,9);oval(c,'#788b76',x+3,y+3,d-6,5);
      rect(c,p[0],x+2,y+12,d-4,3);rect(c,p[2],x+3,y+12,d-6,1);
    }
    if(b.kind==='fort'){
      plate(c,5,10,w-10,h-15,'#7a8371','#c1c4a2','#37473a');
      plate(c,14,4,w-28,h-16,'#98a18a','#d0d4b3','#526452');
      for(let yy=7;yy<h-20;yy+=4){rect(c,'#6f7c69',17,yy,w-34,1);rect(c,'#b2baa0',17,yy+1,w-34,1);}
      plate(c,w/2-9,9,18,11,p[0],p[2],p[1]);rect(c,'#e3dba5',w/2-5,13,10,3);
      drum(0,1,17);drum(w-17,1,17);drum(0,h-23,17);drum(w-17,h-23,17);
      plate(c,18,h-20,w-36,18,'#9ba38a','#d5d6b0','#455740');
      rect(c,'#25382e',w/2-8,h-16,16,16);rect(c,'#15271f',w/2-5,h-13,10,13);stripes(c,w/2-8,h-18,16);
      rect(c,'#bcd599',w/2-11,h-13,2,4);rect(c,'#bcd599',w/2+9,h-13,2,4);
    }else{
      // Recessed hangar, raised ribbed roof, industrial piping and roof vents.
      plate(c,1,h-17,w-2,16,'#7e897c','#bdc4aa','#3b4b40');
      plate(c,1,6,w-2,h-21,'#8a9481','#c4c9ab','#4b5d4b');
      for(let xx=4;xx<w-3;xx+=4){rect(c,'#5b705c',xx,9,1,h-28);rect(c,'#b2bba0',xx+1,9,1,h-28);}
      rect(c,p[1],2,h-23,w-4,6);rect(c,p[0],2,h-22,w-4,3);rect(c,p[2],2,h-22,w-4,1);
      const door=b.kind==='vehicle'?22:14;
      plate(c,w/2-door/2-2,h-16,door+4,16,'#4c5b4c','#b8bfa2','#25362b');
      rect(c,'#1e3027',w/2-door/2,h-13,door,13);for(let yy=h-12;yy<h-4;yy+=3)rect(c,'#4a5b4a',w/2-door/2+1,yy,door-2,1);
      stripes(c,w/2-door/2,h-17,door);rect(c,'#e1df88',3,h-13,2,3);rect(c,'#dc774e',w-5,h-13,2,3);
      if(b.kind==='robot'){
        plate(c,5,8,12,10,'#3c5145','#bdc9ac','#293e30');oval(c,'#9da98b',7,10,8,6);rect(c,'#283b2c',9,12,4,1);
        rect(c,'#3e5044',w-9,1,3,9);rect(c,'#d7bf6a',w-10,1,5,2);
      }else if(b.kind==='radar'){
        rect(c,'#384d3d',w/2-1,2,3,13);oval(c,'#d3d8bc',w/2-12,0,24,10);oval(c,'#82947c',w/2-10,2,20,6);rect(c,'#e4e1be',w/2,0,1,8);
      }else if(b.kind==='repair'){
        plate(c,w/2-8,7,16,11,'#c4cbb2');rect(c,'#527254',w/2-2,8,4,9);rect(c,'#527254',w/2-6,11,12,3);
      }else{
        plate(c,5,1,7,14,'#6e8170','#c2c9ae','#364d3b');rect(c,'#263e2d',6,1,5,3);
        plate(c,w-18,9,12,7,'#576b58','#b0bba0','#344b37');for(let xx=w-16;xx<w-7;xx+=2)rect(c,'#2c4131',xx,11,1,4);
      }
      for(let i=0;i<b.tech;i++)rect(c,'#dfc969',w-5-i*4,h-21,2,2);
      rivets(c,1,6,w-2,h-7);
    }
    return out;
  });};
  Art.detailVehicle=function(c,w,h,p){
    // Painted steel wear, engine louvers, bolts and directional highlights.
    for(let i=0;i<5;i++){const x=3+Math.floor(rnd(i,w)*(w-6)),y=3+Math.floor(rnd(i,h)*(h-6));rect(c,i%2?'#bbc1a4':'#47514b',x,y,1,1);}
    for(let yy=4;yy<h-3;yy+=2)rect(c,'#35423a',4,yy,3,1);
    rect(c,'#dfd99d',w-3,3,1,2);rect(c,'#dfd99d',w-3,h-5,1,2);
  };
  Art.menu=function(){return memo('menu',()=>{
    const lv=ZG.makeLevel(8,3,2,{theme:0,seed:8091,river:1}),map=ZG.genMap(lv),th=ZG.THEMES[0];
    const t=ZG.renderTerrain(map,th,lv.seed),c=t.canvas.getContext('2d');
    for(const b of map.buildings)c.drawImage(Art.building(b),b.tx*16,b.ty*16-10);
    for(const s of map.spawns){const img=ZG.unitIcon(s.t,s.team);c.drawImage(img,s.tx*16-6,s.ty*16-10);}
    return t.canvas.toDataURL();
  });};
})();
