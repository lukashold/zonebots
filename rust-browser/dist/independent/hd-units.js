/* New code-native HD artwork. No sprites or pixels from the original game. */
(function (global) {
  'use strict';
  const TAU = Math.PI * 2, DENSITY = 2, CACHE_LIMIT = 384, MAX_BYTES=48*1024*1024;
  const cache = new Map(),fallbacks=new Map(),pending=new Map();let cacheBytes=0,frameToken,budget=Infinity,frameStart=0,frameSequence=0,wear;
  const clock=()=>typeof performance==='object'?performance.now():Date.now(),metrics={hits:0,misses:0,fallbacks:0,rasterMs:0,maxRasterMs:0,rasters:0};
  const robots = new Set(['grunt', 'psycho', 'tough', 'sniper', 'pyro', 'laser']);
  const guns = new Set(['gatling', 'cannon', 'missile', 'howitzer']);
  const types = ['grunt', 'psycho', 'tough', 'sniper', 'pyro', 'laser', 'jeep', 'light', 'medium', 'heavy', 'apc', 'crane', 'gatling', 'cannon', 'missile', 'launcher', 'howitzer', 'truck', 'firetruck'];
  const palettes = [
    {dark:'#452527', main:'#a84338', light:'#ca7b64', trim:'#dfaa81', eye:'#e5c773'},
    {dark:'#243640', main:'#46708b', light:'#85a3ad', trim:'#b5c5bf', eye:'#c7ddc6'},
    {dark:'#384238', main:'#78816c', light:'#b0b79d', trim:'#e0d7b3', eye:'#e4ce86'},
    {dark:'#443b18', main:'#a9983a', light:'#dbcd71', trim:'#fff1ac', eye:'#ffe68b'}
  ];
  const steel = {ink:'#1c2121', dark:'#333b3a', mid:'#747c73', light:'#b5baaa', edge:'#e2e0c5'};
  const neutral={dark:'#353535',main:'#777777',light:'#b8b8b8',trim:'#d7d7d7',eye:'#e0e0e0'};
  function palette(team) { if(team<0)return neutral;return palettes[team === 0 ? 0 : team === 1 ? 1 : team === 3 ? 3 : 2]; }
  function gradient(c,x,y,w,h,colors) { const g=c.createLinearGradient(x,y,x+w,y+h);colors.forEach((color,i)=>g.addColorStop(i/(colors.length-1),color));return g; }
  function path(c,points,fill,stroke=steel.ink,width=.8) { c.beginPath();points.forEach((p,i)=>i?c.lineTo(p[0],p[1]):c.moveTo(p[0],p[1]));c.closePath();if(fill){c.fillStyle=fill;c.fill();}if(stroke){c.lineWidth=width;c.strokeStyle=stroke;c.stroke();} }
  function line(c,x,y,x2,y2,color,width=1) { c.beginPath();c.moveTo(x,y);c.lineTo(x2,y2);c.strokeStyle=color;c.lineWidth=width;c.lineCap='round';c.stroke(); }
  function oval(c,x,y,rx,ry,fill,stroke=null,width=.7) { c.beginPath();c.ellipse(x,y,Math.max(.01,rx),Math.max(.01,ry),0,0,TAU);c.fillStyle=fill;c.fill();if(stroke){c.strokeStyle=stroke;c.lineWidth=width;c.stroke();} }
  function round(c,x,y,w,h,r,fill,stroke=steel.ink,width=.7) {
    r=Math.min(r,w/2,h/2);c.beginPath();c.moveTo(x+r,y);c.lineTo(x+w-r,y);c.quadraticCurveTo(x+w,y,x+w,y+r);c.lineTo(x+w,y+h-r);c.quadraticCurveTo(x+w,y+h,x+w-r,y+h);c.lineTo(x+r,y+h);c.quadraticCurveTo(x,y+h,x,y+h-r);c.lineTo(x,y+r);c.quadraticCurveTo(x,y,x+r,y);c.closePath();c.fillStyle=fill;c.fill();if(stroke){c.lineWidth=width;c.strokeStyle=stroke;c.stroke();}
  }
  function bolt(c,x,y,r=.7) { oval(c,x,y,r,r,steel.dark);line(c,x-r*.5,y-r*.35,x+r*.4,y-r*.35,steel.edge,.35); }
  function point(cx,cy,a,l,s,z=0) { return [cx+Math.cos(a)*l-Math.sin(a)*s,cy+(Math.sin(a)*l+Math.cos(a)*s)*.63-z]; }
  function deck(c,cx,cy,a,poly,z,fill,edge=steel.ink) { const p=poly.map(q=>point(cx,cy,a,q[0],q[1],z));path(c,p,fill,edge);return p; }
  function block(c,cx,cy,a,poly,z,h,color,light) {
    const low=poly.map(q=>point(cx,cy,a,q[0],q[1],z)),high=poly.map(q=>point(cx,cy,a,q[0],q[1],z+h));
    const faces=poly.map((_,i)=>({i,j:(i+1)%poly.length,y:(low[i][1]+low[(i+1)%poly.length][1])/2})).sort((a,b)=>a.y-b.y);
    for(const f of faces)path(c,[low[f.i],low[f.j],high[f.j],high[f.i]],gradient(c,low[f.i][0],high[f.i][1],8,h,[color,steel.dark]));
    path(c,high,gradient(c,cx-22,cy-h-20,38,35,[light,color]));
    for(let i=0;i<high.length;i++){const j=(i+1)%high.length;if(high[j][0]>high[i][0])line(c,...high[i],...high[j],light,.55);}
    return high;
  }
  function shadow(c,x,y,rx,ry) { c.save();c.translate(x,y);c.scale(1,ry/rx);const g=c.createRadialGradient(0,0,0,0,0,rx);g.addColorStop(0,'#11181199');g.addColorStop(.62,'#11181145');g.addColorStop(1,'#11181100');oval(c,0,0,rx,rx,g);c.restore(); }
  function barrel(c,x,y,a,length,width,color=steel.mid) {
    const ex=x+Math.cos(a)*length,ey=y+Math.sin(a)*length*.63;
    line(c,x,y,ex,ey,steel.ink,width+1.5);line(c,x-.6,y-.6,ex-.6,ey-.6,color,width);
    line(c,x-1,y-1,ex-1,ey-1,steel.light,Math.max(.55,width*.2));
    const ring=point(x,y,a,length-3,0);line(c,...ring,ex,ey,steel.dark,width+1);
    oval(c,ex,ey,Math.max(1,width*.43),Math.max(.8,width*.28),steel.ink);
    return [ex,ey];
  }
  function crew(c,x,y,p,scale=1,a=0) {
    c.save();c.translate(x,y);c.scale(scale,scale);
    round(c,-4,-7,8,8,2,gradient(c,-4,-7,7,9,[p.light,p.dark]));
    oval(c,Math.cos(a),-9,3.7,3.6,gradient(c,-3,-12,6,7,[steel.edge,steel.mid,steel.dark]),steel.ink);
    if(Math.sin(a)>-.45){round(c,-2+Math.cos(a),-10,4.2,1.9,.5,steel.ink,null);line(c,-1+Math.cos(a),-9.2,1+Math.cos(a),-9.2,p.eye,.65);}
    line(c,-3,-5,-5,-2,steel.mid,2);line(c,3,-5,5,-2,steel.mid,2);c.restore();
  }
  function robot(c,u,p,a,aim,phase) {
    const cx=48,cy=70,bulky=u.type==='tough'||u.type==='pyro',w=bulky?8.2:6.9;
    const step=u.moving?Math.sin(phase*Math.PI/2)*2.8:0;
    shadow(c,cx+1,cy+3,13,4.2);
    // Back-mounted equipment keeps the six robot silhouettes recognisable.
    if(u.type==='pyro')for(const s of [-1,1]){round(c,cx+s*7-2.5,cy-24,5,17,2,gradient(c,cx-7,cy-24,10,15,['#e7c086','#846541','#352f2a']));round(c,cx+s*7-2,cy-19,4,3,.4,p.dark);}
    else if(u.type==='laser'){round(c,cx-6,cy-24,12,17,3,steel.dark);for(const s of [-1,1])round(c,cx+s*3-1.3,cy-22,2.6,11,1,gradient(c,0,cy-22,0,12,['#b6fff0','#3f968e','#1e434b']),null);}
    else round(c,cx-5.5,cy-22,11,12,2,gradient(c,cx-5,cy-22,9,10,[steel.light,steel.dark]));
    const legs=[-1,1].map(s=>({s,x:cx+s*3.7+Math.cos(a)*step*s,y:cy-1+Math.sin(a)*step*s*.63})).sort((q,r)=>q.y-r.y);
    for(const q of legs){line(c,cx+q.s*3.5,cy-10,q.x,q.y,steel.ink,5.4);line(c,cx+q.s*3.5-.7,cy-10,q.x-.7,q.y,steel.mid,3.3);oval(c,q.x,q.y-1,2.1,2.1,steel.light,steel.dark);round(c,q.x-3.3,q.y,7,4.3,1.2,gradient(c,q.x-3,q.y,5,4,[steel.light,p.dark]),steel.ink);}
    const gun=()=>{
      const hx=cx+Math.cos(aim)*7,hy=cy-17+Math.sin(aim)*4.5;
      const length=u.type==='sniper'?23:u.type==='tough'?20:u.type==='laser'?20:16;
      const gx=hx-Math.cos(aim)*3,gy=hy-Math.sin(aim)*2;
      if(u.type==='tough'){
        barrel(c,gx,gy,aim,length,5.4,'#9baf92');const q=point(gx,gy,aim,5,0);oval(c,q[0],q[1],4,2.7,gradient(c,q[0]-3,q[1]-3,5,5,[p.light,p.dark]),steel.ink);
      }else if(u.type==='psycho'){
        for(const s of [-1,1])barrel(c,gx-Math.sin(aim)*s*3,gy+Math.cos(aim)*s*2,aim,length,2,steel.mid);
      }else{
        const end=barrel(c,gx,gy,aim,length,u.type==='laser'?4.2:u.type==='pyro'?3.4:2.7,u.type==='pyro'?'#b89a67':steel.mid);
        if(u.type==='sniper'){const q=point(gx,gy,aim,7,0,3);barrel(c,q[0],q[1],aim,6,2,steel.dark);oval(c,q[0],q[1],1.2,1.2,'#a8d7c4');}
        if(u.type==='laser'){oval(c,...end,2.4,1.7,'#3a8888',steel.ink);oval(c,...end,1.25,.9,'#baffeb');}
        if(u.type==='pyro'){line(c,cx-7,cy-12,hx,hy,steel.dark,2.3);oval(c,...end,1,.75,'#edbf72');}
      }
      line(c,cx+Math.cos(aim)*5,cy-18,hx,hy,steel.dark,3.5);oval(c,hx,hy,2.4,2,steel.light,steel.dark);
    };
    if(Math.sin(aim)<-.25)gun();
    path(c,[[cx-w,cy-24],[cx+w-1,cy-24],[cx+w+1.5,cy-18],[cx+w-1,cy-10],[cx-w+1,cy-10],[cx-w-1.5,cy-18]],gradient(c,cx-w,cy-24,w*2,16,[p.light,p.main,p.dark]));
    line(c,cx-w+1,cy-23,cx+w-2,cy-23,p.trim,.7);
    round(c,cx-3.4,cy-17,6.8,4,1,steel.dark);line(c,cx-2,cy-15.6,cx+2,cy-15.6,p.light,.65);
    for(const s of [-1,1]){oval(c,cx+s*(w+1),cy-22,3.5,4.3,gradient(c,cx-8,cy-26,15,10,[steel.edge,p.main,p.dark]),steel.ink);if(s!==(Math.cos(aim)>0?1:-1)){line(c,cx+s*(w+1),cy-19,cx+s*(w+2),cy-12,steel.mid,3);oval(c,cx+s*(w+2),cy-12,2,2.3,steel.light,steel.dark);}}
    // Curved steel helmet, recessed face, paired lenses and a distinct brow.
    const hx=cx+Math.cos(aim)*1.5,hy=cy-33;
    path(c,[[hx-6.3,hy-3.5],[hx-4,hy-6.5],[hx+3.5,hy-6.5],[hx+6,hy-3],[hx+5.5,hy+4],[hx+2.5,hy+6],[hx-3.5,hy+5.5],[hx-6,hy+2]],gradient(c,hx-6,hy-7,12,14,[steel.edge,steel.light,steel.mid,steel.dark]),steel.ink);
    line(c,hx-4,hy-6,hx+3,hy-6,'#edf1e4',.6);
    if(Math.sin(aim)>-.5){const shift=Math.cos(aim)*1.8;round(c,hx-5+shift,hy-2,9.4,3.7,1.1,steel.ink,null);for(const s of [-1,1])oval(c,hx+shift+s*2.1,hy,.75,u.type==='psycho'?.5:.8,p.eye);line(c,hx-2+shift,hy+4,hx+2+shift,hy+4,steel.dark,1);}
    else {round(c,hx-4.5,hy-2,9,4,1.3,p.dark,steel.dark);line(c,hx-3,hy-1,hx+3,hy-1,p.light,.55);}
    for(const s of [-1,1])oval(c,hx+s*6.5,hy+1,1.5,2.2,steel.dark,steel.ink);
    if(u.type==='tough'){path(c,[[hx-6,hy-5],[hx+4,hy-5],[hx+5,hy-2],[hx-6,hy-2]],p.main,steel.dark);line(c,hx-5,hy-4,hx+3,hy-4,p.light,.7);}
    if(u.type==='psycho'){line(c,hx-3,hy-7,hx+2,hy-7,p.main,1.8);line(c,hx+5,hy-4,hx+6,hy-10,steel.dark,.8);}
    if(u.type==='sniper')round(c,hx-4,hy-1.8,3.5,3.4,1,'#947449',steel.dark);
    if(Math.sin(aim)>=-.25)gun();
    for(const s of [-1,1])bolt(c,cx+s*4.7,cy-21,.55);
  }
  function tracks(c,cx,cy,a,L,W,phase) {
    for(const s of [-1,1]){
      block(c,cx,cy,a,[[-L-1,s*W-3.8],[L+1,s*W-3.8],[L+2,s*W+3.8],[-L-2,s*W+3.8]],0,6,'#3b4547','#79817c');
      for(let n=-L;n<L;n+=3.8){const q=point(cx,cy,a,n+phase*.5,s*W-3,6.2),r=point(cx,cy,a,n+phase*.5,s*W+3,6.2);line(c,...q,...r,'#bdbea3',.65);}
      for(let n=-L+3;n<L;n+=6){const q=point(cx,cy,a,n,s*W,1);oval(c,...q,2.8,2.15,steel.dark,steel.ink);bolt(c,...q,.7);}
    }
  }
  function wheels(c,cx,cy,a,L,W) {
    const parts=[];for(const l of [-L,L])for(const s of [-W,W])parts.push({q:point(cx,cy,a,l,s,1),l,s});
    parts.sort((a,b)=>a.q[1]-b.q[1]);for(const w of parts){oval(c,w.q[0],w.q[1],5.6,4.4,gradient(c,w.q[0]-5,w.q[1]-4,8,8,['#536166','#111b20']),steel.ink);oval(c,w.q[0]-.4,w.q[1]-.4,2.7,2.5,gradient(c,w.q[0]-2,w.q[1]-3,4,5,[steel.light,steel.dark]),steel.ink);bolt(c,...w.q,.7);}
  }
  function vehicle(c,u,p,a,aim,phase,occupied) {
    const cx=48,cy=68,heavy=u.type==='heavy',jeep=u.type==='jeep',wheeled=jeep||u.type==='truck'||u.type==='firetruck';
    const L=heavy?25:jeep?19:u.type==='truck'||u.type==='firetruck'?25:u.type==='light'?17:21,W=heavy?15:jeep?11:u.type==='light'?10.3:12.8;
    shadow(c,cx+2,cy+3,L+8,W*.68);
    if(wheeled)wheels(c,cx,cy,a,L-6,W);else tracks(c,cx,cy,a,L,W,phase);
    const hull=[[-L,-W+3],[L-7,-W+3],[L,-W+7],[L,W-7],[L-7,W-3],[-L,W-3],[-L-2,W-7],[-L-2,-W+7]];
    const body=!occupied?'#777777':jeep?'#76856b':u.type==='firetruck'?'#a14b43':'#687568',bodyLight=!occupied?'#bfbfbf':jeep?'#c0c9a4':u.type==='firetruck'?'#e79b7c':'#b3bea1';
    block(c,cx,cy,a,hull,5,6,body,bodyLight);
    deck(c,cx,cy,a,[[-L+3,-W+4],[L-7,-W+4],[L-7,-W+7],[-L+3,-W+7]],11.1,p.main);
    deck(c,cx,cy,a,[[-L+3,W-7],[L-7,W-7],[L-7,W-4],[-L+3,W-4]],11.2,p.main);
    if(jeep){
      deck(c,cx,cy,a,[[-14,-7],[-1,-7],[-1,7],[-14,7]],11.5,steel.ink);
      for(const s of [-1,1]){const seat=point(cx,cy,a,-9,s*4.5,12);oval(c,...seat,3.3,2.6,'#786e55',steel.dark);}
      if(occupied){const q=point(cx,cy,a,-4,-3.5,14);crew(c,q[0],q[1],p,.73,a);}
      const glass=[point(cx,cy,a,1,-8,10),point(cx,cy,a,1,8,10),point(cx,cy,a,2,8,21),point(cx,cy,a,2,-8,21)];
      path(c,glass,gradient(c,cx-7,cy-23,12,13,['#d9f0dfb5','#597b8999','#263f4ecc']),steel.dark,1.2);
      line(c,...glass[2],...glass[3],steel.light,1.1);line(c,...point(cx,cy,a,1,0,10),...point(cx,cy,a,2,0,21),steel.light,.9);
      block(c,cx,cy,a,[[5,-7],[16,-6],[16,6],[5,7]],11,2,body,bodyLight);
      const q=point(cx,cy,a,-10,2,15);oval(c,...q,4.2,2.6,steel.dark,steel.ink);barrel(c,q[0],q[1]-3,aim,20,2.8,steel.mid);
      for(const s of [-1,1]){const q=point(cx,cy,a,-17,s*8,12);line(c,...q,...point(cx,cy,a,0,s*8,12),steel.mid,1.5);}
    }else if(u.type==='truck'||u.type==='firetruck'){
      block(c,cx,cy,a,[[7,-9],[21,-9],[23,-6],[23,8],[7,8]],11,13,body,bodyLight);
      deck(c,cx,cy,a,[[12,-9],[20,-9],[20,-8.4],[12,-8.4]],21,'#92c9d0');
      const wind=[point(cx,cy,a,21,-7,15),point(cx,cy,a,21,6,15),point(cx,cy,a,21,6,22),point(cx,cy,a,21,-7,22)];path(c,wind,gradient(c,cx-8,cy-26,12,11,['#bde9dc','#3c6370']),steel.dark);
      if(u.type==='truck'){
        block(c,cx,cy,a,[[-22,-9],[5,-9],[5,9],[-22,9]],11,11,occupied?'#7b806b':'#777777',occupied?'#b9bda2':'#bfbfbf');
        for(let n=-19;n<5;n+=6){const q=point(cx,cy,a,n,-9,12),r=point(cx,cy,a,n,-9,21);line(c,...q,...r,'#454f45',.9);}
      }else{
        block(c,cx,cy,a,[[-22,-9],[5,-9],[5,9],[-22,9]],11,10,occupied?'#9b5249':'#777777',occupied?'#d38b6f':'#bfbfbf');
        deck(c,cx,cy,a,[[-19,-7],[4,-7],[4,7],[-19,7]],21.2,'#a7b2ae');
        for(const s of [-1,1]){line(c,...point(cx,cy,a,-19,s*4,23),...point(cx,cy,a,4,s*4,23),steel.dark,1.2);for(let n=-17;n<4;n+=4)line(c,...point(cx,cy,a,n,-4,23),...point(cx,cy,a,n,4,23),steel.light,.9);}
        const q=point(cx,cy,a,13,0,25);oval(c,...q,2.7,1.8,'#ee9c65',steel.dark);
      }
    }else{
      for(let n=0;n<5;n++){const q=point(cx,cy,a,-L+3+n*1.5,-7,12),r=point(cx,cy,a,-L+3+n*1.5,7,12);line(c,...q,...r,steel.dark,.85);}
      if(u.type==='apc'){
        block(c,cx,cy,a,[[-14,-9],[11,-9],[15,-4],[15,4],[11,9],[-14,9]],11,10,body,bodyLight);
        for(const s of [-1,1])for(let n=-10;n<11;n+=6){const q=point(cx,cy,a,n,s*9,17);round(c,q[0]-1.6,q[1]-1,3.2,2,1,'#26424a',steel.dark,.4);line(c,q[0]-1,q[1]-.5,q[0]+.5,q[1]-.5,'#a7d8ce',.4);}
        const q=point(cx,cy,a,-5,0,22);oval(c,...q,5.5,3.2,steel.dark,steel.ink);oval(c,q[0]-1,q[1]-1,4.5,2.5,bodyLight,steel.dark);
        const t=point(cx,cy,a,8,0,23);barrel(c,t[0],t[1],aim,14,2.5,steel.mid);
      }else if(u.type==='crane'){
        block(c,cx,cy,a,[[5,-8],[15,-8],[15,5],[5,5]],11,15,'#ad9155','#e0c785');
        const glass=[point(cx,cy,a,15,-6,17),point(cx,cy,a,15,4,17),point(cx,cy,a,15,4,24),point(cx,cy,a,15,-6,24)];path(c,glass,gradient(c,cx-10,cy-24,12,10,['#bde3d6','#375d68']),steel.dark);
        const q=point(cx,cy,a,-6,0,15),end=point(q[0],q[1],aim,29,0,20);oval(c,...q,6.5,4,steel.dark,steel.ink);line(c,q[0],q[1]-2,end[0],end[1],steel.ink,7.5);line(c,q[0]-1,q[1]-3,end[0]-1,end[1]-1,'#ba9b5c',5);line(c,q[0]-1,q[1]-4,end[0]-1,end[1]-2,'#f2dca4',1);
        line(c,end[0],end[1],end[0],end[1]+13,steel.dark,.85);c.beginPath();c.moveTo(end[0],end[1]+13);c.bezierCurveTo(end[0]+1,end[1]+19,end[0]-6,end[1]+19,end[0]-5,end[1]+14);c.strokeStyle=steel.dark;c.lineWidth=1.8;c.stroke();
      }else if(u.type==='launcher'){
        const q=point(cx,cy,a,0,0,14);oval(c,...q,10,6,steel.dark,steel.ink);
        block(c,q[0],q[1],aim,[[-12,-10],[12,-10],[12,10],[-12,10]],0,9,body,bodyLight);
        for(const s of [-6,0,6]){const q2=point(q[0],q[1],aim,13,s,5);oval(c,...q2,2.6,2.3,steel.ink);oval(c,q2[0]-.4,q2[1]-.4,1.7,1.5,'#d9d6b7');}
        const q2=point(q[0],q[1],aim,-7,-10,9);line(c,...q2,q2[0]-1,q2[1]-10,steel.dark,.8);
      }else{
        const tx=cx+Math.cos(a),ty=cy-14+Math.sin(a)*.63,r=heavy?13:u.type==='medium'?10.5:9;
        oval(c,tx,ty+2.5,r,r*.64,steel.dark,steel.ink);oval(c,tx,ty,r,r*.65,gradient(c,tx-r,ty-r,2*r,r*2,[bodyLight,body,steel.dark]),steel.ink);
        oval(c,tx-3,ty-3,r*.5,r*.22,'#d7ddba66');
        barrel(c,tx+Math.cos(aim)*5,ty+Math.sin(aim)*3,aim,heavy?30:u.type==='medium'?26:22,heavy?5.6:u.type==='medium'?4.6:3.8);
        if(heavy)barrel(c,tx-Math.sin(aim)*5,ty+Math.cos(aim)*3-2,aim,18,2.3,body);
        oval(c,tx-3,ty-4,4,2.4,steel.dark,steel.ink);oval(c,tx-3.5,ty-5,3.2,1.7,bodyLight,steel.dark);bolt(c,tx-3,ty-5,.6);
        const q=point(cx,cy,a,-11,-7,12);line(c,...q,q[0]-1,q[1]-15,steel.dark,.7);oval(c,q[0]-1,q[1]-15,.6,.8,steel.light);
        deck(c,tx,ty,aim,[[-4,-r+1],[5,-r+1],[5,-r+3],[-4,-r+3]],.2,p.main);
      }
    }
    for(const s of [-1,1]){const q=point(cx,cy,a,L-1,s*(W-5),10);oval(c,...q,1.9,1.3,'#f8e4aa',steel.dark);const r=point(cx,cy,a,-L,s*(W-4),9);oval(c,...r,1.3,.8,'#a84235',steel.ink);}
    for(const l of [-L+4,L-8])for(const s of [-1,1]){const q=point(cx,cy,a,l,s*(W-4),12);bolt(c,...q,.65);}
  }
  function gun(c,u,p,aim,occupied) {
    const cx=48,cy=68;
    shadow(c,cx+2,cy+3,26,10);
    oval(c,cx,cy,22,11,gradient(c,cx-20,cy-10,36,20,[steel.light,steel.dark]),steel.ink);
    oval(c,cx,cy-3,21,10,gradient(c,cx-20,cy-13,36,20,occupied?['#c2c8ae','#5d6f66','#314249']:['#c5c5c5','#777777','#373737']),steel.ink);
    oval(c,cx,cy-5,16,7.2,steel.dark,steel.ink);oval(c,cx-1,cy-6,15,6.8,gradient(c,cx-15,cy-12,25,14,[steel.light,steel.mid,steel.dark]),steel.ink);
    for(let n=0;n<10;n++){const angle=n*TAU/10;bolt(c,cx+Math.cos(angle)*18,cy-3+Math.sin(angle)*8.5,.7);}
    for(const s of [-1,1])block(c,cx,cy,aim,[[-5,s*15-2],[6,s*15-2],[6,s*15+2],[-5,s*15+2]],0,4,p.dark,p.light);
    const ty=cy-13;
    if(u.type==='missile'){
      oval(c,cx,ty+5,10,5.5,p.dark,steel.ink);
      block(c,cx,ty,aim,[[-11,-13],[13,-13],[13,13],[-11,13]],0,11,p.main,p.light);
      for(const s of [-8,0,8])for(const z of [3,9]){const q=point(cx,ty,aim,14,s,z);oval(c,...q,2.8,2.35,steel.ink,steel.mid,.6);oval(c,q[0]-.3,q[1]-.4,1.8,1.4,'#d9c9a0');oval(c,q[0]+.2,q[1],.6,.6,p.dark);}
      const q=point(cx,ty,aim,-7,-13,12);line(c,...q,q[0]-1,q[1]-10,steel.dark,.85);oval(c,q[0]-1,q[1]-10,1,.7,p.light);
    }else if(u.type==='gatling'){
      block(c,cx,ty,aim,[[-8,-7],[7,-7],[7,7],[-8,7]],0,8,p.main,p.light);
      for(const s of [-1,1])for(const z of [-1,1])barrel(c,cx-Math.sin(aim)*s*2,ty-5+Math.cos(aim)*s*1.3+z,aim,28,1.5,steel.mid);
      const middle=point(cx,ty-5,aim,13,0);oval(c,...middle,3.9,3.1,steel.dark,steel.ink);oval(c,middle[0]-.7,middle[1]-.7,3.1,2.2,steel.light,steel.dark);
      const feed=point(cx,ty,aim,-3,-9,3);oval(c,...feed,5,3.7,p.dark,steel.ink);line(c,feed[0]-2,feed[1]-2,feed[0]+2,feed[1]-2,p.light,.8);
    }else if(u.type==='howitzer'){
      block(c,cx,ty,aim,[[-11,-10],[9,-10],[12,-6],[12,6],[9,10],[-11,10]],0,9,p.main,p.light);
      const mount=point(cx,ty,aim,-1,0,8);oval(c,...mount,8,5,steel.dark,steel.ink);
      barrel(c,mount[0],mount[1]-3,aim,31,7.5,'#8d9e94');
      const end=point(mount[0],mount[1]-3,aim,28,0);line(c,end[0]-Math.sin(aim)*5,end[1]+Math.cos(aim)*3,end[0]+Math.sin(aim)*5,end[1]-Math.cos(aim)*3,steel.dark,2.2);
      for(const s of [-1,1])line(c,...point(cx,ty,aim,-8,s*8,8),...point(cx,ty,aim,8,s*8,8),steel.light,1.6);
    }else{
      oval(c,cx,ty+1,12,7.8,p.dark,steel.ink);oval(c,cx-1,ty-2,11.5,7.2,gradient(c,cx-12,ty-9,22,15,[p.light,p.main,p.dark]),steel.ink);
      barrel(c,cx+Math.cos(aim)*3,ty-3+Math.sin(aim)*2,aim,34,5.8);
      const q=point(cx,ty,aim,-5,-8,5);round(c,q[0]-2,q[1]-1.5,4,3,1,steel.dark);line(c,q[0]-1,q[1]-.5,q[0]+1,q[1]-.5,'#9fe2d1',.7);
    }
    if(occupied){const q=point(cx,cy,aim,-13,0,10);crew(c,q[0],q[1],p,.8,aim);}
    else {const q=point(cx,cy,aim,-13,0,9);oval(c,...q,3.5,2.1,'#303c39',steel.ink);}
  }
  function direction(a){return((Math.round((Number(a)||0)*8/Math.PI)%16)+16)%16;}
  function pose(u,time){
    const moving=Boolean(u.moving),rawAim=direction(u.aim==null?u.angle:u.aim),aim=robots.has(u.type)||guns.has(u.type)?Math.round(rawAim/2)*2%16:rawAim,body=guns.has(u.type)?0:robots.has(u.type)?moving?(Math.round(direction(u.angle)/2)*2)%16:0:direction(u.angle);
    const phase=robots.has(u.type)&&moving?(Math.floor(time*7+Number(u.id||0)*.5)%2?1:3):0;
    return{body,aim,phase,occupied:u.team>=0,key:[u.type,u.team,body,aim,phase].join(':')};
  }
  function weather(c){
    if(!wear){wear=document.createElement('canvas');wear.width=wear.height=96;const q=wear.getContext('2d'),data=q.createImageData(96,96);let seed=90173;
      for(let n=0;n<96*96;n++){seed=(Math.imul(seed,1664525)+1013904223)|0;const v=seed>>>24,at=n*4;data.data[at]=v<32?92:v;data.data[at+1]=v<32?64:v;data.data[at+2]=v<32?43:v;data.data[at+3]=v<32?22:14;}q.putImageData(data,0,0);}
    c.save();c.globalCompositeOperation='source-atop';c.drawImage(wear,0,0,96,96);c.globalAlpha=.25;
    for(let n=0;n<12;n++){const x=19+(n*23)%59,y=28+(n*13)%47;line(c,x,y,x+2+(n%4),y-.7,n%3?'#e0d6b0':'#594530',.45);}c.restore();
  }
  function raster(u,time,known){
    const info=known||pose(u,time),{body,aim,phase,occupied,key}=info;
    if(cache.has(key)){const hit=cache.get(key);cache.delete(key);cache.set(key,hit);metrics.hits++;return hit;}
    const started=clock(),image=document.createElement('canvas');image.width=image.height=96*DENSITY;
    const ctx=image.getContext('2d');ctx.scale(DENSITY,DENSITY);ctx.lineJoin='round';ctx.lineCap='round';
    const p=palette(u.team),a=body*Math.PI/8,b=aim*Math.PI/8;
    if(robots.has(u.type))robot(ctx,u,p,a,b,phase);else if(guns.has(u.type))gun(ctx,u,p,b,occupied);else vehicle(ctx,u,p,a,b,phase,occupied);
    weather(ctx);
    const value={image,crop:robots.has(u.type)?[17,28,65,51]:u.type==='crane'?[6,8,84,81]:[3,17,90,72]};
    cache.set(key,value);cacheBytes+=image.width*image.height*4;fallbacks.set(u.type+':'+u.team,value);
    while(cache.size>CACHE_LIMIT||cacheBytes>MAX_BYTES){const oldest=cache.keys().next().value,old=cache.get(oldest);cacheBytes-=old.image.width*old.image.height*4;cache.delete(oldest);}
    const elapsed=clock()-started;metrics.rasters++;metrics.rasterMs+=elapsed;metrics.maxRasterMs=Math.max(metrics.maxRasterMs,elapsed);metrics.misses++;return value;
  }
  function beginFrame(token){
    if(token!==undefined&&frameToken===token)return;frameToken=token;budget=4;frameStart=clock();frameSequence++;
    for(const [key,job]of pending)if(job.lastFrame<frameSequence-2)pending.delete(key);
    for(const [key,job]of pending){if(budget<=0||clock()-frameStart>3.5)break;pending.delete(key);if(!cache.has(key)){raster(job.u,job.time,job.info);budget--;}}
  }
  function requested(u,time){
    const info=pose(u,time);if(cache.has(info.key))return raster(u,time,info);
    if(budget>0&&(budget===Infinity||clock()-frameStart<3.5)){if(budget!==Infinity)budget--;pending.delete(info.key);return raster(u,time,info);}
    // Moving entities keep their last complete pose while new raster work is
    // shared across frames, rather than blocking the next animation frame.
    pending.set(info.key,{u:{type:u.type,team:u.team,angle:u.angle,aim:u.aim,moving:u.moving,id:u.id},time,info,lastFrame:frameSequence});while(pending.size>256)pending.delete(pending.keys().next().value);
    metrics.fallbacks++;return fallbacks.get(u.type+':'+u.team)||null;
  }
  function quick(c,u,b){
    const p=palette(u.team);c.save();c.translate(b.x+b.w/2,b.y+b.h*.7);c.scale(b.w/64,b.h/64);shadow(c,2,4,22,8);
    if(robots.has(u.type)){round(c,-8,-23,16,22,2,p.main);round(c,-6,-37,12,13,2,steel.mid);round(c,-4,-32,8,3,1,steel.dark);line(c,-5,-1,-6,11,steel.dark,5);line(c,5,-1,6,11,steel.dark,5);}
    else if(guns.has(u.type)){oval(c,0,0,23,9,steel.mid,steel.dark);round(c,-10,-18,20,15,3,p.main);barrel(c,0,-16,Number(u.aim)||0,28,4);}
    else{round(c,-24,-7,48,17,3,steel.dark);round(c,-20,-17,40,23,3,steel.mid);round(c,-10,-22,20,13,3,p.main);if(u.type!=='truck'&&u.type!=='crane')barrel(c,0,-20,Number(u.aim)||0,29,4);}c.restore();
  }
  function fallbackBounds(u) {
    if(robots.has(u.type))return {x:u.x-16,y:u.y-31,w:38,h:42};
    if(guns.has(u.type))return {x:u.x,y:u.y,w:60,h:50};
    return {x:u.x-22,y:u.y-26,w:62,h:50};
  }
  function unit(ctx,u,time=0,map,bounds) {
    if(!u||!types.includes(u.type)||![u.x,u.y].every(Number.isFinite))return false;
    const b=bounds&&[bounds.x,bounds.y,bounds.w,bounds.h].every(Number.isFinite)&&bounds.w>0&&bounds.h>0?bounds:fallbackBounds(u),sprite=requested(u,Number(time)||0);if(!sprite){quick(ctx,u,b);return true;}const s=sprite.crop;
    ctx.save();ctx.imageSmoothingEnabled=true;ctx.imageSmoothingQuality='medium';ctx.drawImage(sprite.image,s[0]*DENSITY,s[1]*DENSITY,s[2]*DENSITY,s[3]*DENSITY,b.x,b.y,b.w,b.h);ctx.restore();
    if(u.maxHp>0&&u.hp/u.maxHp<.3){const x=b.x+b.w*.45,y=b.y+b.h*.35;for(let n=0;n<3;n++){const phase=((Number(time)||0)*.55+n*.31+u.id*.13)%1;ctx.save();ctx.globalAlpha=(1-phase)*.22;oval(ctx,x+phase*9,y-phase*23,2.3+phase*4,2.6+phase*5,'#293639');ctx.restore();}}
    return true;
  }
  global.HDUnits=Object.freeze({unit,beginFrame,types:Object.freeze(types.slice()),clearCache(){cache.clear();fallbacks.clear();pending.clear();cacheBytes=0;budget=Infinity;frameToken=undefined;},get cacheSize(){return cache.size;},get cacheStats(){return{entries:cache.size,bytes:cacheBytes,maxEntries:CACHE_LIMIT,maxBytes:MAX_BYTES};},get stats(){return{...metrics,cacheSize:cache.size,cacheBytes,pending:pending.size};}});
})(window);
