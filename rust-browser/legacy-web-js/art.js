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
  Art.vehicle=function(type,team,frame){return memo(`hull:${type}:${team}:${frame}`,()=>{
    const p=teams[team<0?2:team],tank=['light','medium','heavy','crane'].includes(type);
    const w=type==='heavy'?28:type==='jeep'?22:26,h=type==='heavy'?21:type==='jeep'?15:18,out=make(w,h),c=out.getContext('2d');
    if(tank){
      for(const yy of [0,h-5]){
        plate(c,1,yy,w-2,5,'#3c443c','#8a9280','#202b25');
        for(let xx=2;xx<w-2;xx+=3){rect(c,'#171f1c',xx+(frame||0),yy+1,1,3);rect(c,'#9b9f85',xx,yy+1,2,1);}
      }
    }else for(const yy of [0,h-4])for(const xx of [3,w-7]){plate(c,xx,yy,5,4,'#252e27','#616c5c','#131c17');rect(c,'#86947b',xx+2,yy+1,1,2);}
    // Chamfered hull silhouette, upper armor planes and exposed engine deck.
    rect(c,'#293b2d',1,5,w-2,h-10);plate(c,3,3,w-6,h-6,p[1],p[2],'#28392b');
    plate(c,5,4,w-10,h-8,p[0],p[2],p[1]);
    for(let yy=5;yy<h-5;yy+=2){rect(c,'#253a2c',5,yy,5,1);rect(c,'#94a084',5,yy+1,5,1);}
    rect(c,p[2],w-7,5,3,h-10);rect(c,'#d5d3a5',w-5,4,2,2);rect(c,'#d5d3a5',w-5,h-6,2,2);
    for(const yy of [4,h-5])for(const xx of [4,w-5]){rect(c,'#e2dcb5',xx,yy,1,1);rect(c,'#283b2c',xx+1,yy+1,1,1);}
    if(type==='jeep'){
      plate(c,8,4,7,h-8,'#263f35','#788f7b','#15271d');rect(c,'#b0d3bc',14,5,2,h-10);rect(c,'#77a89a',14,5,1,h-10);
      for(const yy of [5,h-7])rect(c,'#89947d',9,yy,3,2);
    }else if(type==='apc'||type==='truck'||type==='firetruck'){
      plate(c,4,4,13,h-8,type==='firetruck'?'#a34432':'#758362','#c0c4a0','#364731');
      for(let xx=6;xx<16;xx+=3)rect(c,'#465d3f',xx,5,1,h-10);
      rect(c,'#a4cfb9',w-7,5,2,h-10);
    }else if(type==='crane'){
      plate(c,8,6,17,5,'#c7a94d','#f0d681','#5a5631');for(let xx=10;xx<23;xx+=4)rect(c,'#4e5130',xx,7,2,3);
    }
    for(let n=0;n<7;n++)rect(c,n%2?'#c4c7a1':'#4a563e',10+Math.floor(rnd(n,w)*(w-15)),5+Math.floor(rnd(n,h)*(h-10)),1,1);
    return out;
  });};
  Art.turret=function(type,team){
    const spec={light:[16,9,8],medium:[19,11,10],heavy:[22,13,12]}[type];if(!spec)return null;
    return memo(`turret:${type}:${team}`,()=>{
      const [w,h,d]=spec,p=teams[team<0?2:team],out=make(w,h),c=out.getContext('2d');
      for(const yy of type==='heavy'?[3,h-4]:[Math.floor(h/2)]){
        rect(c,'#223329',d-2,yy-1,w-d+2,3);rect(c,'#bec3a7',d-1,yy-1,w-d,1);rect(c,'#748675',d-1,yy,w-d,1);rect(c,'#151f19',w-2,yy-1,2,3);
      }
      oval(c,'#253c2a',0,0,d,h);oval(c,p[1],1,1,d-2,h-2);oval(c,p[0],1,1,d-3,h-4);
      rect(c,p[2],3,1,d-6,1);oval(c,'#374c34',3,3,4,3);rect(c,'#a0ad85',4,3,2,1);rect(c,'#d9d5a6',d-3,h-4,1,1);
      return out;
    });
  };
  Art.smoke=function(c,x,y,time,seed,intensity){
    for(let i=0;i<3;i++){
      const p=(time*.35+i/3+seed*.17)%1,r=2+p*5;
      c.globalAlpha=(1-p)*intensity;oval(c,'#333b36',x+p*8+Math.sin(p*4+seed)*2-r,y-p*22-r,r*2,r*1.6);
      c.globalAlpha=(1-p)*intensity*.6;oval(c,'#8d9280',x+p*8-r,y-p*22-r,r*1.6,r);
    }c.globalAlpha=1;
  };
  Art.blast=function(c,f){
    const p=f.t/f.life,x=f.x/2,y=f.y/2,r=Math.max(2,f.r/2*(.3+p*.7));
    if(p<.65){c.globalAlpha=(1-p)*.65;oval(c,'#aa7134',x-r*1.5,y-r*.7,r*3,r*1.5);c.globalAlpha=1;}
    for(let i=0;i<8;i++){
      const ang=i*Math.PI/4+f.x,dist=r*p*1.6,cx=x+Math.cos(ang)*dist,cy=y+Math.sin(ang)*dist*.7-p*4;
      const size=Math.max(1,(1-p)*(3+r*.35));
      oval(c,p>.65?'#4c5147':'#a84727',cx-size,cy-size,size*2,size*2);
      if(p<.65){oval(c,'#ed8f36',cx-size*.7,cy-size*.8,size*1.4,size*1.3);oval(c,'#ffe5a4',cx-size*.4,cy-size*.6,size*.8,size*.8);}
      if(p<.5)rect(c,'#fff1b9',cx+Math.cos(ang)*r,cy+Math.sin(ang)*r,1,1);
    }
    if(p>.4)Art.smoke(c,x,y, p*2,Math.floor(f.x),.4*(1-p));
  };
  Art.landscape=function(map,theme){
    const out=make(map.W*16,map.H*16),c=out.getContext('2d');
    for(let ty=1;ty<map.H-1;ty++)for(let tx=1;tx<map.W-1;tx++){
      if(map.tiles[ty*map.W+tx]!==ZG.T.ROUGH || rnd(tx,ty)<.35)continue;
      const x=tx*16+3+Math.floor(rnd(ty,tx)*6),y=ty*16+7;
      c.globalAlpha=.25;oval(c,'#172c20',x-2,y+3,12,5);c.globalAlpha=1;
      if(theme.key==='jungle'){
        rect(c,'#624f2d',x+2,y-3,3,9);
        for(const [dx,dy,r] of [[0,0,5],[5,-3,5],[1,-6,4]]){
          oval(c,'#28472c',x+dx-r,y+dy-r,r*2,r*2);
          oval(c,'#4c7334',x+dx-r,y+dy-r,r*2-2,r*2-2);
          oval(c,'#8a9b49',x+dx-r+1,y+dy-r+1,r,r-1);
        }
      }else if(theme.key==='desert'){
        plate(c,x+3,y-6,3,12,'#75804b','#a6a16a','#485735');
        rect(c,'#4d633b',x,y-3,2,5);rect(c,'#a0a36a',x,y-3,1,4);rect(c,'#657646',x+1,y,3,2);
        rect(c,'#52663c',x+6,y-5,2,5);rect(c,'#909a5d',x+6,y-5,1,4);
      }else if(theme.key==='arctic'){
        oval(c,'#8ba7ae',x-1,y,11,6);oval(c,'#edf2df',x-2,y-3,9,7);rect(c,'#c4dbd4',x+4,y-1,3,5);
      }else if(theme.key==='city'){
        plate(c,x-2,y-3,9,7,'#6e746b','#b9b8a0','#3c4c43');rect(c,'#424d43',x+3,y-4,3,6);rect(c,'#c0aa76',x-1,y-2,2,1);
      }else{
        oval(c,'#282c27',x-2,y-2,11,8);oval(c,'#646256',x-2,y-3,8,5);rect(c,'#b96f3b',x+3,y,3,1);
      }
    }
    return out;
  };
  Art.menu=function(){return memo('menu',()=>{
    const lv=ZG.makeLevel(8,3,2,{theme:0,seed:8091,river:1}),map=ZG.genMap(lv),th=ZG.THEMES[0];
    const t=ZG.renderTerrain(map,th,lv.seed),c=t.canvas.getContext('2d');
    for(const b of map.buildings)c.drawImage(Art.building(b),b.tx*16,b.ty*16-10);
    for(const s of map.spawns){const img=ZG.unitIcon(s.t,s.team);c.drawImage(img,s.tx*16-6,s.ty*16-10);}
    return t.canvas.toDataURL();
  });};
})();
