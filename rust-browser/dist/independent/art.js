/* Original, code-native pixel artwork. Animation time is expressed in seconds. */
(function (global) {
  'use strict';
  const TAU = Math.PI * 2;
  const sprites = new Map(), structures = new Map(), lands = new Map();
  const robotTypes = new Set(['grunt', 'psycho', 'tough', 'sniper', 'pyro', 'laser']);
  const gunTypes = new Set(['gatling', 'cannon', 'missile']);
  const teams = [
    { dark:'#582a2d', shade:'#853a36', main:'#b84c3d', light:'#e3825b', bright:'#ffd197', flag:'#dc553e', eye:'#fff1af' },
    { dark:'#24354f', shade:'#355b7a', main:'#4b85aa', light:'#86bed5', bright:'#d1e8da', flag:'#67bde1', eye:'#d5f5ef' },
    { dark:'#3b403b', shade:'#626b5a', main:'#8b9278', light:'#b5b99a', bright:'#e0d8b4', flag:'#c1bea1', eye:'#fbdf88' }
  ];
  const metal = { ink:'#202828', deep:'#303a3b', dark:'#465052', mid:'#697374', light:'#a1aaa3', edge:'#d2d5ba', rust:'#795945' };
  function team(n) { return teams[n === 0 ? 0 : n === 1 ? 1 : 2]; }
  function canvas(w,h) { const c = document.createElement('canvas'); c.width = Math.ceil(w); c.height = Math.ceil(h); return c; }
  function rect(c,x,y,w,h,col) { if(w <= 0 || h <= 0) return; c.fillStyle=col; c.fillRect(Math.round(x),Math.round(y),Math.round(w),Math.round(h)); }
  function ellipse(c,x,y,rx,ry,col) {
    c.fillStyle=col; rx=Math.max(1,Math.round(rx)); ry=Math.max(1,Math.round(ry));
    for(let v=-ry;v<=ry;v++) { const q=Math.floor(rx*Math.sqrt(Math.max(0,1-v*v/(ry*ry)))); c.fillRect(Math.round(x)-q,Math.round(y)+v,q*2+1,1); }
  }
  function poly(c,pts,col) {
    let low=Infinity,high=-Infinity; for(const p of pts){low=Math.min(low,p[1]);high=Math.max(high,p[1]);}
    c.fillStyle=col;
    for(let y=Math.ceil(low);y<Math.ceil(high);y++){
      const xs=[]; for(let i=0,j=pts.length-1;i<pts.length;j=i++){
        const a=pts[j],b=pts[i]; if((a[1]<=y&&b[1]>y)||(b[1]<=y&&a[1]>y)) xs.push(a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1]));
      }
      xs.sort((a,b)=>a-b); for(let i=0;i+1<xs.length;i+=2)c.fillRect(Math.ceil(xs[i]),y,Math.max(1,Math.ceil(xs[i+1])-Math.ceil(xs[i])),1);
    }
  }
  function line(c,x1,y1,x2,y2,col,width=1){
    x1=Math.round(x1);y1=Math.round(y1);x2=Math.round(x2);y2=Math.round(y2);
    const dx=Math.abs(x2-x1),sx=x1<x2?1:-1,dy=-Math.abs(y2-y1),sy=y1<y2?1:-1;let err=dx+dy;
    for(let n=0;n<512;n++){rect(c,x1-Math.floor(width/2),y1-Math.floor(width/2),width,width,col);if(x1===x2&&y1===y2)break;const e=2*err;if(e>=dy){err+=dy;x1+=sx;}if(e<=dx){err+=dx;y1+=sy;}}
  }
  function outline(c,pts,col=metal.ink,w=1) { for(let i=0;i<pts.length;i++){const a=pts[i],b=pts[(i+1)%pts.length];line(c,a[0],a[1],b[0],b[1],col,w);} }
  function bolt(c,x,y) { rect(c,x,y,2,2,metal.ink);rect(c,x,y,1,1,metal.edge); }
  function hash(x,y,s=0){let n=Math.imul(x|0,374761393)^Math.imul(y|0,668265263)^Math.imul(s|0,1442695041);n=Math.imul(n^(n>>>13),1274126177);return ((n^(n>>>16))>>>0)/4294967296;}
  function seeded(s){return function(){s=(Math.imul(s,1664525)+1013904223)|0;return(s>>>0)/4294967296;};}
  function direction(a){return ((Math.round((Number(a)||0)/(Math.PI/4))%8)+8)%8;}
  function limit(cache,size){if(cache.size>size){let n=0;for(const k of cache.keys()){cache.delete(k);if(++n>size/4)break;}}}
  function shadow(c,x,y,rx,ry){ellipse(c,x+2,y+2,rx,ry,'#20282852');ellipse(c,x+1,y+1,rx-2,Math.max(1,ry-1),'#20282832');}
  function worldPoint(cx,cy,a,l,s,z=0){return[cx+l*Math.cos(a)-s*Math.sin(a),cy+(l*Math.sin(a)+s*Math.cos(a))*.68-z];}
  function plane(c,cx,cy,a,pts,z,col){const p=pts.map(v=>worldPoint(cx,cy,a,v[0],v[1],z));poly(c,p,col);return p;}
  function prism(c,cx,cy,a,pts,z,height,top,sideA,sideB){
    const lo=pts.map(v=>worldPoint(cx,cy,a,v[0],v[1],z)),hi=pts.map(v=>worldPoint(cx,cy,a,v[0],v[1],z+height));
    const faces=[];for(let i=0;i<pts.length;i++){const j=(i+1)%pts.length;faces.push({p:[lo[i],lo[j],hi[j],hi[i]],v:(lo[i][1]+lo[j][1])/2,i});}
    faces.sort((a,b)=>a.v-b.v);for(const f of faces){poly(c,f.p,f.i%2?sideA:sideB);outline(c,f.p,metal.ink);}
    poly(c,hi,top);outline(c,hi,metal.ink);return hi;
  }
  function smoke(c,x,y,t,seed,scale=1){
    for(let i=0;i<5;i++){const q=(t*.45+i*.21+hash(seed,i))%1;const drift=Math.sin(q*3+seed)*4;ellipse(c,x+drift+q*6,y-q*24*scale,(2+q*4)*scale,(2+q*3)*scale,q<.3?'#3b3935aa':q<.7?'#6c696071':'#ada79537');}
    if(Math.floor(t*8+seed)%5<2)rect(c,x-1,y-2,2,3,'#e9a356');
  }
  function robot(c,type,p,bodyDir,aimDir,walk){
    const cx=32,cy=40,a=bodyDir*Math.PI/4,aa=aimDir*Math.PI/4,fx=Math.cos(a),fy=Math.sin(a),ax=Math.cos(aa),ay=Math.sin(aa);
    const bulky=type==='tough'||type==='pyro',wide=bulky?7:6;
    shadow(c,cx,cy+3,10,3);
    // Backpack silhouettes and cables are drawn behind the articulated figure.
    if(type==='pyro'){
      for(const k of [-1,1]){rect(c,cx+k*6-2,cy-15,4,13,metal.ink);rect(c,cx+k*6-1,cy-14,2,10,'#aa8d4e');rect(c,cx+k*6-1,cy-11,2,2,p.dark);rect(c,cx+k*6-1,cy-14,1,2,'#e5c580');}
      line(c,cx-7,cy-4,cx+ax*7,cy-6+ay*4,'#393a30',2);
    } else if(type==='laser'){
      rect(c,cx-5,cy-15,10,12,metal.ink);rect(c,cx-4,cy-14,8,10,p.dark);rect(c,cx-3,cy-13,2,7,'#82d1bc');rect(c,cx+1,cy-13,2,7,'#456b67');
    } else {rect(c,cx-5,cy-13,10,8,metal.deep);rect(c,cx-4,cy-12,8,3,metal.mid);}
    const stride=[0,2,0,-2][walk];
    for(const side of [-1,1]){
      const step=stride*side, lx=cx+side*3+fx*step, ly=cy-2+fy*step*.65;
      line(c,cx+side*3,cy-5,lx,ly+4,metal.ink,5);line(c,cx+side*3,cy-5,lx,ly+2,metal.mid,3);
      rect(c,lx-2,ly+2,6,4,metal.ink);rect(c,lx-2,ly+2,5,2,p.shade);rect(c,lx-2,ly+2,2,1,p.light);rect(c,lx-1,ly+5,5,1,'#141e20');
    }
    const drawWeapon=()=>{
      const handX=cx+ax*6,handY=cy-9+ay*5;
      const length=type==='sniper'?21:type==='tough'?16:type==='laser'?17:14;
      const sx=handX-ax*3,sy=handY-ay*2,ex=handX+ax*length,ey=handY+ay*length*.68;
      if(type==='tough'){
        line(c,sx,sy,ex,ey,metal.ink,6);line(c,sx-1,sy-1,ex-1,ey-1,'#8b8f73',4);line(c,sx-1,sy-2,ex-1,ey-2,'#d2c49a');
        ellipse(c,ex,ey,3,3,metal.ink);ellipse(c,ex,ey,1,1,'#7a493c');
      }else if(type==='pyro'){
        line(c,sx,sy,ex,ey,metal.ink,4);line(c,sx,sy-1,ex,ey-1,'#b18c4f',2);rect(c,ex-1,ey-1,2,2,'#eac686');
      }else{
        line(c,sx,sy,ex,ey,metal.ink,type==='laser'?5:4);line(c,sx-1,sy-1,ex-1,ey-1,type==='laser'?'#6dada7':metal.mid,2);
        line(c,handX+ax*3,handY+ay*2,handX+ax*7,handY+ay*5,p.dark,3);
        if(type==='sniper'){line(c,handX+ax*3,handY+ay*2-3,handX+ax*8,handY+ay*5-3,metal.ink,2);rect(c,handX+ax*7,handY+ay*5-4,2,1,'#b4d8c4');}
        if(type==='laser'){ellipse(c,ex,ey,2,2,'#27464a');rect(c,ex-1,ey-1,2,2,'#a6f9dc');}
        if(type==='psycho'){line(c,cx-ax*2-4,cy-7,cx-ax*2-4+ax*11,cy-7+ay*7,metal.ink,3);line(c,cx-ax*2-5,cy-8,cx-ax*2-5+ax*10,cy-8+ay*6,metal.light);}
      }
      rect(c,handX-2,handY-1,4,3,metal.dark);rect(c,handX-2,handY-1,3,1,metal.light);
    };
    if(ay<-.15)drawWeapon();
    // Chest plates have a consistent upper-left light source in every direction.
    const chest=[[cx-wide,cy-14],[cx+wide-1,cy-14],[cx+wide+1,cy-10],[cx+wide-1,cy-4],[cx-wide+1,cy-4],[cx-wide-1,cy-10]];
    poly(c,chest,metal.ink);poly(c,[[cx-wide+1,cy-13],[cx+wide-2,cy-13],[cx+wide-1,cy-6],[cx-wide+2,cy-5]],p.main);
    line(c,cx-wide+1,cy-13,cx+wide-2,cy-13,p.light);line(c,cx-wide+1,cy-12,cx-wide+2,cy-7,p.light);
    rect(c,cx+wide-3,cy-11,2,6,p.shade);rect(c,cx-2,cy-9,4,2,metal.dark);rect(c,cx-2,cy-9,1,1,p.bright);
    rect(c,cx-wide+1,cy-5,wide*2-2,2,metal.deep);rect(c,cx-1,cy-5,3,2,metal.light);
    for(const side of [-1,1]){
      const shoulderX=cx+side*(wide+1),shoulderY=cy-12+(fy<-.2?1:0);
      ellipse(c,shoulderX,shoulderY,3,3,metal.ink);ellipse(c,shoulderX,shoulderY-1,2,2,p.shade);rect(c,shoulderX-1,shoulderY-2,2,1,p.light);
      if(side!==(ax>=0?1:-1)){line(c,shoulderX,shoulderY+2,shoulderX+side,cy-5,metal.dark,3);rect(c,shoulderX+side-1,cy-5,3,3,metal.mid);}
    }
    // Rounded steel head, recessed brow, cheeks, and direction-dependent eyes.
    const hx=cx+Math.round(ax*1.4),hy=cy-23;
    poly(c,[[hx-4,hy],[hx+3,hy],[hx+6,hy+3],[hx+6,hy+8],[hx+3,hy+11],[hx-4,hy+11],[hx-6,hy+8],[hx-6,hy+3]],metal.ink);
    poly(c,[[hx-3,hy+1],[hx+3,hy+1],[hx+5,hy+4],[hx+4,hy+9],[hx-3,hy+10],[hx-5,hy+7],[hx-5,hy+4]],metal.mid);
    rect(c,hx-3,hy+1,6,2,metal.light);rect(c,hx-4,hy+3,2,3,metal.light);rect(c,hx+3,hy+4,2,4,metal.dark);
    rect(c,hx-3,hy+9,5,1,metal.dark);rect(c,hx-1,hy+10,3,1,metal.deep);
    if(ay<-.45){rect(c,hx-3,hy+4,6,3,p.shade);rect(c,hx-2,hy+4,4,1,p.light);rect(c,hx-2,hy+8,4,1,metal.deep);rect(c,hx-4,hy+5,1,2,metal.edge);}
    else {
      const eyeShift=Math.round(ax*2),ey=hy+5+(ay>.5?1:0);
      rect(c,hx-4+eyeShift,ey-1,7,4,metal.deep);rect(c,hx-4+eyeShift,ey-1,7,1,'#151f22');
      rect(c,hx-3+eyeShift,ey,2,2,p.eye);rect(c,hx+eyeShift,ey,2,type==='psycho'?1:2,p.eye);
      if(type==='sniper'){rect(c,hx-3+eyeShift,ey,2,2,'#edbd73');rect(c,hx+eyeShift,ey,2,1,'#94b6a5');}
      rect(c,hx-2+eyeShift,ey+4,3,1,metal.ink);rect(c,hx+4,hy+6,2,3,metal.dark);
    }
    if(type==='tough'){rect(c,hx-4,hy+1,7,2,p.main);rect(c,hx-3,hy+1,5,1,p.light);}
    if(type==='psycho'){rect(c,hx-3,hy+2,5,1,p.main);rect(c,hx+4,hy-1,1,4,metal.ink);}
    if(ay>=-.15)drawWeapon();
    if(type==='grunt'){rect(c,cx-4,cy-12,2,2,p.bright);}
  }
  function vehicle(c,type,p,bodyDir,aimDir,walk){
    const cx=40,cy=46,a=bodyDir*Math.PI/4,aa=aimDir*Math.PI/4;
    const heavy=type==='heavy',L=heavy?19:type==='apc'?18:type==='light'?14:16,W=heavy?13:type==='jeep'?10:12;
    shadow(c,cx,cy+2,L+5,10);
    if(type==='jeep'){
      const wheels=[];for(const l of [-11,11])for(const s of [-10,10]){const q=worldPoint(cx,cy,a,l,s,1);wheels.push({q,l,s});}wheels.sort((a,b)=>a.q[1]-b.q[1]);
      for(const w of wheels){ellipse(c,w.q[0],w.q[1],5,4,metal.ink);ellipse(c,w.q[0]-1,w.q[1]-1,3,2,metal.dark);rect(c,w.q[0]-2,w.q[1]-2,2,1,metal.light);}
      prism(c,cx,cy,a,[[-15,-8],[10,-8],[15,-5],[15,5],[10,8],[-15,8]],3,5,p.main,p.dark,p.shade);
      plane(c,cx,cy,a,[[-9,-6],[0,-6],[0,6],[-9,6]],9,metal.ink);plane(c,cx,cy,a,[[-8,-5],[-3,-5],[-3,5],[-8,5]],9,'#685e45');
      const windshield=[worldPoint(cx,cy,a,2,-7,9),worldPoint(cx,cy,a,2,7,9),worldPoint(cx,cy,a,3,7,17),worldPoint(cx,cy,a,3,-7,17)];
      poly(c,windshield,metal.ink);const wi=windshield.map(v=>[v[0],v[1]-1]);poly(c,wi,'#516c70');line(c,wi[2][0],wi[2][1],wi[3][0],wi[3][1],'#a5bcaa');
      const split=worldPoint(cx,cy,a,2,0,10),splitTop=worldPoint(cx,cy,a,3,0,17);line(c,split[0],split[1],splitTop[0],splitTop[1],metal.dark,2);
      plane(c,cx,cy,a,[[5,-6],[12,-5],[12,5],[5,6]],9,p.light);
      for(const side of [-1,1]){const q=worldPoint(cx,cy,a,14,side*5,6);ellipse(c,q[0],q[1],2,2,metal.ink);rect(c,q[0]-1,q[1]-1,2,1,'#f9e6a5');}
      const q=worldPoint(cx,cy,a,-7,0,10);ellipse(c,q[0],q[1],4,3,metal.dark);barrel(c,q[0],q[1]-3,aa,17,3,metal.mid);
      return;
    }
    // Track belts are separate raised, articulated assemblies, not rotated images.
    for(const side of [-1,1]){
      prism(c,cx,cy,a,[[-L,-4+side*W],[L,-4+side*W],[L,4+side*W],[-L,4+side*W]],0,5,'#444a43','#252e2c','#303b36');
      for(let l=-L+1;l<L;l+=4){const q=worldPoint(cx,cy,a,l+(walk%2),side*W,5),r=worldPoint(cx,cy,a,l+(walk%2),side*W+3,5);line(c,q[0],q[1],r[0],r[1],'#8b8b6f');}
    }
    const hull=[[-L+1,-W+3],[L-6,-W+3],[L,-W+7],[L,W-7],[L-6,W-3],[-L+1,W-3],[-L-2,W-7],[-L-2,-W+7]];
    prism(c,cx,cy,a,hull,5,6,p.main,p.dark,p.shade);
    const inset=hull.map(v=>[v[0]*.84,v[1]*.78]);plane(c,cx,cy,a,inset,12,p.light);
    const rear=plane(c,cx,cy,a,[[-L+1,-W+5],[-L+9,-W+5],[-L+9,W-5],[-L+1,W-5]],13,p.shade);outline(c,rear,p.dark);
    for(let n=0;n<5;n++){const v1=worldPoint(cx,cy,a,-L+2+n,-W+6,13),v2=worldPoint(cx,cy,a,-L+2+n,W-6,13);line(c,v1[0],v1[1],v2[0],v2[1],metal.ink);}
    for(const v of [[L-5,-W+5],[L-5,W-5],[-L+2,-W+5],[-L+2,W-5]]){const q=worldPoint(cx,cy,a,v[0],v[1],12);bolt(c,q[0],q[1]);}
    if(type==='apc'){
      prism(c,cx,cy,a,[[-10,-7],[9,-7],[12,-4],[12,4],[9,7],[-10,7]],11,7,p.main,p.dark,p.shade);
      const hatch=worldPoint(cx,cy,a,-4,0,19);ellipse(c,hatch[0],hatch[1],6,3,metal.deep);ellipse(c,hatch[0],hatch[1]-1,5,2,p.light);line(c,hatch[0]-2,hatch[1]-1,hatch[0]+2,hatch[1]-1,metal.dark);
      for(const side of [-1,1])for(let n=0;n<3;n++){const q=worldPoint(cx,cy,a,-7+n*5,side*7,16);rect(c,q[0]-1,q[1],3,2,metal.ink);rect(c,q[0]-1,q[1],2,1,'#829e9b');}
      const turret=worldPoint(cx,cy,a,6,0,19);ellipse(c,turret[0],turret[1],4,3,metal.dark);barrel(c,turret[0],turret[1]-2,aa,12,2,metal.mid);
    }else if(type==='crane'){
      prism(c,cx,cy,a,[[3,-7],[12,-7],[12,3],[3,3]],11,12,p.main,p.dark,p.shade);
      const glass=plane(c,cx,cy,a,[[5,-7],[10,-7],[10,-6],[5,-6]],20,'#9ab6a7');outline(c,glass,metal.deep);
      const q=worldPoint(cx,cy,a,-5,0,13);ellipse(c,q[0],q[1],6,3,metal.ink);ellipse(c,q[0],q[1]-2,5,3,'#b29654');
      const end=[q[0]+Math.cos(aa)*23,q[1]+Math.sin(aa)*13-14];line(c,q[0],q[1]-3,end[0],end[1],metal.ink,6);line(c,q[0]-1,q[1]-5,end[0]-1,end[1]-2,'#ccad62',3);line(c,q[0]-1,q[1]-6,end[0]-1,end[1]-3,'#efe0a0');
      line(c,end[0],end[1],end[0],end[1]+11,metal.deep);line(c,end[0],end[1]+11,end[0]-3,end[1]+13,metal.ink,2);line(c,end[0]-3,end[1]+13,end[0]-5,end[1]+10,metal.ink,2);
    }else{
      const tx=cx+Math.cos(a)*1,ty=cy-13+Math.sin(a)*.68;
      ellipse(c,tx,ty+3,heavy?11:8,heavy?7:6,metal.ink);ellipse(c,tx,ty,heavy?11:8,heavy?7:6,p.dark);
      ellipse(c,tx-1,ty-2,heavy?10:7,heavy?6:5,p.main);ellipse(c,tx-3,ty-4,heavy?6:4,2,p.light);
      const bx=tx+Math.cos(aa)*5,by=ty+Math.sin(aa)*3-1;barrel(c,bx,by,aa,heavy?25:type==='medium'?21:18,heavy?5:4,metal.mid);
      if(heavy){const x=tx-Math.sin(aa)*4,y=ty+Math.cos(aa)*3-3;barrel(c,x,y,aa,14,2,p.shade);}
      ellipse(c,tx-3,ty-5,3,2,metal.ink);ellipse(c,tx-3,ty-6,2,1,p.light);rect(c,tx+3,ty-4,3,2,metal.deep);rect(c,tx+3,ty-4,2,1,'#a7d3c0');
      const aerial=worldPoint(cx,cy,a,-10,-5,12);line(c,aerial[0],aerial[1],aerial[0]-1,aerial[1]-12,metal.ink);rect(c,aerial[0]-1,aerial[1]-13,1,2,metal.edge);
    }
    for(const side of [-1,1]){const q=worldPoint(cx,cy,a,L-1,side*(W-6),9);rect(c,q[0]-1,q[1]-1,3,2,'#eee1b2');rect(c,q[0]+1,q[1],1,2,p.dark);}
  }
  function barrel(c,x,y,a,length,width,col){
    const ex=x+Math.cos(a)*length,ey=y+Math.sin(a)*length*.68;
    line(c,x,y,ex,ey,metal.ink,width+2);line(c,x-1,y-1,ex-1,ey-1,col,width);line(c,x-1,y-2,ex-1,ey-2,metal.light,1);
    const mx=x+Math.cos(a)*(length-3),my=y+Math.sin(a)*(length-3)*.68;line(c,mx,my,ex,ey,metal.deep,width+1);rect(c,ex-1,ey-1,2,2,'#141c20');
  }
  function emplacement(c,type,p,aimDir){
    const cx=40,cy=43,a=aimDir*Math.PI/4;shadow(c,cx,cy+2,18,8);
    ellipse(c,cx,cy,17,9,metal.ink);ellipse(c,cx,cy-2,16,8,metal.dark);ellipse(c,cx-2,cy-4,12,5,metal.mid);
    for(let k=0;k<8;k++){const q=k*TAU/8;bolt(c,cx+Math.cos(q)*13,cy-3+Math.sin(q)*6);}
    for(const s of [-1,1]){rect(c,cx+s*14-2,cy-2,5,6,metal.ink);rect(c,cx+s*14-1,cy-2,3,3,p.shade);rect(c,cx+s*14-1,cy-2,2,1,p.light);}
    ellipse(c,cx,cy-7,9,6,p.dark);ellipse(c,cx-1,cy-9,8,5,p.main);ellipse(c,cx-3,cy-11,4,2,p.light);
    if(type==='missile'){
      prism(c,cx,cy-8,a,[[-7,-8],[10,-8],[10,8],[-7,8]],0,8,p.main,p.dark,p.shade);
      for(let n=0;n<4;n++){const s=-6+n*4,q=worldPoint(cx,cy-8,a,11,s,4);ellipse(c,q[0],q[1],2,2,metal.ink);rect(c,q[0]-1,q[1]-1,2,2,'#d9bf8f');rect(c,q[0],q[1],1,1,p.main);}
      const q=worldPoint(cx,cy-8,a,-4,-4,9);line(c,q[0],q[1],q[0],q[1]-10,metal.ink);rect(c,q[0]-1,q[1]-11,3,2,p.light);
    }else if(type==='gatling'){
      for(const side of [-1,1]){const x=cx-Math.sin(a)*side*3,y=cy-10+Math.cos(a)*side*2;barrel(c,x,y,a,21,2,metal.mid);}
      const q=[cx+Math.cos(a)*12,cy-10+Math.sin(a)*8];ellipse(c,q[0],q[1],4,3,metal.dark);ellipse(c,q[0]-1,q[1]-1,3,2,metal.light);
      rect(c,cx-8,cy-10,5,5,p.dark);rect(c,cx-8,cy-10,4,1,p.light);
    }else {barrel(c,cx,cy-10,a,27,6,metal.mid);ellipse(c,cx-4,cy-12,3,2,metal.dark);rect(c,cx-5,cy-13,3,1,p.light);}
  }
  function unit(ctx,u,time=0){
    if(!u)return;const type=u.type||'grunt',body=direction(u.angle),aim=direction(u.aim==null?u.angle:u.aim),frame=u.moving?((Math.floor(time*9+Number(u.id||0)*.7)%4)+4)%4:0;
    const size=robotTypes.has(type)?64:80,ground=robotTypes.has(type)?40:gunTypes.has(type)?43:46,key=[type,u.team,body,aim,frame].join(':');
    let s=sprites.get(key);if(!s){s=canvas(size,size);const c=s.getContext('2d');if(robotTypes.has(type))robot(c,type,team(u.team),body,aim,frame);else if(gunTypes.has(type))emplacement(c,type,team(u.team),aim);else vehicle(c,type,team(u.team),body,aim,frame);sprites.set(key,s);limit(sprites,1536);}
    ctx.imageSmoothingEnabled=false;ctx.drawImage(s,Math.round(u.x-size/2),Math.round(u.y-ground));
    if(u.maxHp>0&&u.hp/u.maxHp<.4)smoke(ctx,u.x+3,u.y-10,time,Number(u.id)||7,.65);
  }
  function plate(c,x,y,w,h,base=metal.mid){
    rect(c,x,y,w,h,metal.ink);rect(c,x+1,y+1,w-2,h-2,base);line(c,x+1,y+1,x+w-2,y+1,metal.light);line(c,x+1,y+2,x+1,y+h-2,metal.light);line(c,x+w-2,y+2,x+w-2,y+h-2,metal.dark);line(c,x+2,y+h-2,x+w-2,y+h-2,metal.dark);
  }
  function vent(c,x,y,w,h){plate(c,x,y,w,h,metal.dark);for(let i=3;i<h-2;i+=3){rect(c,x+3,y+i,w-6,1,metal.ink);rect(c,x+3,y+i+1,w-6,1,metal.mid);} }
  function windowRow(c,x,y,n,spacing=9){for(let i=0;i<n;i++){rect(c,x+i*spacing,y,6,5,metal.ink);rect(c,x+i*spacing,y,5,1,metal.light);rect(c,x+i*spacing+1,y+2,4,2,'#6f9e9c');rect(c,x+i*spacing+1,y+2,2,1,'#b0c8ad');}}
  function hazard(c,x,y,w,h){
    c.save();c.beginPath();c.rect(x,y,w,h);c.clip();
    rect(c,x,y,w,h,'#d7ae57');for(let n=-h;n<w+h;n+=9){poly(c,[[x+n,y],[x+n+4,y],[x+n+h+4,y+h],[x+n+h,y+h]],'#343b35');}
    c.restore();line(c,x,y,x+w-1,y,metal.light);line(c,x,y+h-1,x+w-1,y+h-1,metal.ink);
  }
  function factoryGate(c,x,y,w,h,p){
    plate(c,x-3,y-3,w+6,h+5,p.dark);rect(c,x,y,w,h,metal.ink);rect(c,x+2,y+1,w-4,h-2,'#555c53');
    for(let k=3;k<h-2;k+=4){rect(c,x+2,y+k,w-4,2,'#343f3b');rect(c,x+3,y+k,w-6,1,'#8b9280');}
    rect(c,x+w/2-1,y+1,2,h-1,metal.deep);hazard(c,x,y+h-6,w,5);rect(c,x-2,y+h-1,w+4,2,'#b3ae8e');
    for(const s of [-1,1]){const q=x+(s===1?w+3:-5);rect(c,q,y+1,3,4,metal.ink);rect(c,q+1,y+2,1,2,s===1?'#86bd8b':'#d7a358');}
  }
  function pipe(c,x,y,w){line(c,x,y,x+w,y,metal.ink,5);line(c,x,y-1,x+w,y-1,metal.mid,3);line(c,x,y-2,x+w,y-2,metal.light);for(let i=2;i<w;i+=12){rect(c,x+i,y-3,2,5,metal.deep);rect(c,x+i,y-3,1,2,metal.light);} }
  function turret(c,x,y,r,p){
    ellipse(c,x,y+4,r+1,r*.58,metal.ink);ellipse(c,x,y,r,r*.58,metal.dark);ellipse(c,x-1,y-3,r-1,r*.55,p.main);ellipse(c,x-3,y-5,r-4,2,p.light);
    rect(c,x-5,y-3,10,3,metal.ink);rect(c,x-4,y-3,8,1,metal.light);rect(c,x-2,y-2,4,1,'#121e21');barrel(c,x,y-1,Math.PI/2,13,4,metal.mid);
    for(const s of [-1,1])bolt(c,x+s*(r-3),y-2);
  }
  function buildingBase(kind,p,w,h,stars){
    const s=canvas(w+12,h+12),c=s.getContext('2d'),rng=seeded(w*53+h*17+stars*19);c.translate(6,6);
    // Ground footings, drains, bolts, and a cast shadow tie each building to terrain.
    poly(c,[[5,17],[w-10,13],[w+1,26],[w+4,h-2],[w-2,h+3],[11,h+3],[1,h-5]],'#2027284d');
    poly(c,[[3,17],[w-8,13],[w-2,20],[w-2,h-4],[w-8,h],[5,h],[1,h-5]],'#5a5d50');
    outline(c,[[3,17],[w-8,13],[w-2,20],[w-2,h-4],[w-8,h],[5,h],[1,h-5]],'#343d37');
    for(let i=0;i<w;i+=13){rect(c,i+4,h-3,7,1,'#8a8e75');}
    if(kind==='fort'){
      const front=h-31;
      poly(c,[[12,22],[w-16,22],[w-7,33],[w-7,h-16],[w-18,h-7],[12,h-7],[5,h-17],[5,33]],metal.ink);
      rect(c,12,29,w-26,h-44,'#747b6b');rect(c,9,front,w-19,23,'#4b5550');rect(c,12,front+1,w-25,5,p.shade);
      for(let y=38;y<front-5;y+=14){for(let x=18+(Math.floor(y/14)%2)*7;x<w-20;x+=15){rect(c,x,y,10,1,'#555f53');rect(c,x,y+1,10,1,'#8d927b');}}
      // An elevated command keep, side buttresses, and four independent gun towers.
      const kw=w*.47,kx=(w-kw)/2,ky=24,kh=h*.45;
      poly(c,[[kx,ky+10],[kx+kw,ky+10],[kx+kw,ky+kh],[kx,ky+kh]],'#4e5a54');
      poly(c,[[kx-3,ky+9],[kx+5,ky],[kx+kw-8,ky],[kx+kw+3,ky+9],[kx+kw,ky+kh-11],[kx-1,ky+kh-11]],p.shade);
      poly(c,[[kx,ky+6],[kx+7,ky-2],[kx+kw-8,ky-2],[kx+kw,ky+6],[kx+kw-2,ky+kh-18],[kx+1,ky+kh-18]],metal.mid);
      line(c,kx+7,ky-2,kx+kw-8,ky-2,metal.edge);line(c,kx,ky+6,kx+7,ky-2,metal.light);line(c,kx+kw,ky+7,kx+kw-2,ky+kh-17,metal.dark,2);
      plate(c,kx+8,ky+8,kw-17,20,p.dark);windowRow(c,kx+12,ky+12,Math.max(2,Math.floor((kw-20)/9)));
      vent(c,kx+9,ky+35,kw-19,13);pipe(c,kx+5,ky+kh-7,kw-10);
      for(const x of [18,w-19])for(const y of [28,front-4])turret(c,x,y,15,p);
      // Armored gate, security slots, and a band of team color at the front wall.
      factoryGate(c,w/2-21,front+3,42,24,p);windowRow(c,26,front+8,2);windowRow(c,w-41,front+8,2);
      for(const x of [10,w-15]){plate(c,x,front+3,6,21,metal.dark);bolt(c,x+2,front+5);bolt(c,x+2,front+18);}
      const antennaX=w/2+13;line(c,antennaX,ky+6,antennaX,ky-15,metal.ink,2);line(c,antennaX-1,ky-14,antennaX-1,ky-3,metal.light);line(c,antennaX-8,ky-9,antennaX+6,ky-9,metal.dark);rect(c,antennaX-1,ky-16,2,2,'#dc9770');
      ellipse(c,w/2-11,ky+4,7,3,metal.ink);ellipse(c,w/2-12,ky+2,6,2,p.light);
    }else{
      const roofBottom=h-31,wallBottom=h-7;
      poly(c,[[7,22],[w-12,22],[w-6,29],[w-6,wallBottom],[7,wallBottom]],metal.ink);
      rect(c,8,26,w-16,wallBottom-27,'#59635a');rect(c,9,roofBottom,w-18,wallBottom-roofBottom,'#4a554f');
      poly(c,[[7,21],[17,10],[w-19,10],[w-8,21],[w-8,roofBottom],[7,roofBottom]],'#737e6c');
      line(c,17,10,w-19,10,metal.edge);line(c,7,21,17,10,metal.light);line(c,8,22,8,roofBottom-1,metal.light);line(c,w-9,22,w-9,roofBottom,metal.deep,2);
      line(c,8,roofBottom,w-9,roofBottom,metal.ink,2);line(c,10,roofBottom+2,w-10,roofBottom+2,p.main,3);
      for(let x=14;x<w-14;x+=13){line(c,x,16,x,roofBottom-3,'#8a927c');line(c,x+1,16,x+1,roofBottom-3,'#5b685c');}
      const gateW=kind==='vehicle'?42:kind==='repair'?38:30,gateX=kind==='radar'?w-43:w/2-gateW/2;
      factoryGate(c,gateX,roofBottom+5,gateW,h-roofBottom-14,p);
      if(kind==='robot'){
        // Twin assembly housings, cooling plant, and copper service pipes.
        for(const x of [21,w-25]){ellipse(c,x,26,11,8,metal.ink);ellipse(c,x-1,23,10,7,metal.mid);ellipse(c,x-3,20,6,2,metal.light);vent(c,x-6,24,13,8);rect(c,x-8,34,16,4,p.shade);}
        pipe(c,24,39,w-49);windowRow(c,13,roofBottom+8,1);windowRow(c,w-21,roofBottom+8,1);
        rect(c,w/2-7,18,14,9,metal.deep);rect(c,w/2-5,20,10,5,p.shade);rect(c,w/2-4,21,2,2,'#b6dcc7');rect(c,w/2+2,21,2,2,'#b6dcc7');
      }else if(kind==='vehicle'){
        for(let x=17;x<w-20;x+=18){plate(c,x,17,14,20,metal.mid);vent(c,x+2,20,10,14);}
        pipe(c,16,roofBottom-6,w-33);rect(c,12,roofBottom+9,6,10,p.dark);rect(c,w-20,roofBottom+9,6,10,p.dark);
        // Small service tank and ladder on the right roof flank.
        ellipse(c,w-16,30,5,3,metal.ink);rect(c,w-21,23,11,7,p.shade);ellipse(c,w-16,23,5,3,p.light);
      }else if(kind==='repair'){
        plate(c,16,17,24,20,p.dark);rect(c,25,20,6,14,'#d7c58d');rect(c,21,24,14,6,'#d7c58d');vent(c,w-42,18,27,17);
        pipe(c,15,roofBottom-5,w-30);line(c,17,roofBottom-5,17,roofBottom-15,metal.dark,3);line(c,18,roofBottom-6,18,roofBottom-15,metal.light);
        for(let i=0;i<3;i++){rect(c,11+i*5,roofBottom+9,3,8,metal.ink);rect(c,12+i*5,roofBottom+9,1,5,'#bd9861');}
      }else if(kind==='radar'){
        plate(c,13,17,28,21,p.shade);windowRow(c,17,22,2);vent(c,16,30,21,6);
        const mastX=w-30,mastY=29;line(c,mastX,mastY+14,mastX,mastY-8,metal.ink,5);line(c,mastX-1,mastY+12,mastX-1,mastY-8,metal.mid,3);ellipse(c,mastX,mastY+14,12,4,p.dark);
        windowRow(c,13,roofBottom+9,2);rect(c,12,roofBottom+17,18,2,metal.deep);
      }
      // Wall seams, footings, drainage, and individual fasteners.
      for(const x of [10,w-14]){rect(c,x,roofBottom+5,4,wallBottom-roofBottom-3,metal.deep);rect(c,x,roofBottom+5,1,wallBottom-roofBottom-3,metal.mid);bolt(c,x+1,wallBottom-4);}
      for(let y=19;y<roofBottom-2;y+=6){rect(c,w-13,y,5,1,metal.deep);rect(c,w-13,y-1,1,1,metal.light);}
    }
    // Reproducible weathering is individual to the material, without a texture grid.
    for(let i=0;i<Math.round(w*h/150);i++){const x=10+rng()*(w-22),y=18+rng()*(h-35);if(rng()<.6){rect(c,x,y,2+rng()*3,1,'#242f2a30');rect(c,x,y-1,1,1,'#e1d4a225');}}
    const plaqueY=kind==='fort'?h-53:h-39;plate(c,w/2-13,plaqueY,26,7,p.dark);
    for(let i=0;i<Math.min(5,Math.max(0,stars||0));i++){const x=w/2-10+i*4;poly(c,[[x,plaqueY+4],[x+1,plaqueY+2],[x+2,plaqueY+4],[x+1,plaqueY+5]],'#d9c286');}
    return s;
  }
  function ruinBase(kind,w,h){
    const s=canvas(w+12,h+12),c=s.getContext('2d'),rng=seeded(w*29+h*31);c.translate(6,6);
    ellipse(c,w/2+4,h*.58,w*.48,h*.35,'#2b302b5c');
    poly(c,[[7,24],[w-12,18],[w-4,27],[w-5,h-9],[w-15,h-3],[9,h-4],[3,h-13]],'#5a5d4b');
    for(let k=0;k<w*h/100;k++){
      const x=9+rng()*(w-22),y=22+rng()*(h-34),r=2+rng()*7;
      poly(c,[[x-r,y],[x-r+2,y-r],[x+r-2,y-r+1],[x+r,y+2],[x+1,y+r*.6],[x-r+1,y+2]],k%3?'#3b4840':'#6b7160');
      line(c,x-r+2,y-r,x+r-2,y-r+1,k%3?'#78816b':'#a39d7a');
    }
    // Jagged remaining walls, twisted reinforcement, and a half-buried gate.
    poly(c,[[8,h-30],[13,h-39],[18,h-31],[22,h-35],[25,h-26],[w-19,h-26],[w-16,h-38],[w-12,h-33],[w-8,h-39],[w-7,h-7],[8,h-7]],'#343f38');
    poly(c,[[9,h-28],[14,h-35],[20,h-29],[23,h-31],[25,h-22],[w-21,h-22],[w-16,h-32],[w-11,h-29],[w-9,h-33],[w-9,h-10],[9,h-10]],'#67705e');
    for(let k=0;k<5;k++){const x=16+k*(w-30)/5;line(c,x,h-28,x-2,h-40-rng()*6,metal.ink);line(c,x-2,h-40,x+2,h-45,metal.dark);}
    plate(c,w/2-18,h-24,36,17,'#535d50');for(let k=0;k<3;k++)line(c,w/2-16,h-21+k*4,w/2+15,h-22+k*4,metal.ink);hazard(c,w/2-16,h-13,32,3);
    for(let k=0;k<5;k++){const x=12+rng()*(w-22),y=h-8+rng()*5;rect(c,x,y,5+rng()*6,2,'#2e3932');rect(c,x,y,4,1,'#96977b');}
    pipe(c,w*.25,h*.5,w*.27);line(c,w*.25,h*.5,w*.22,h*.5+8,metal.dark,3);
    if(kind==='fort'){ellipse(c,22,31,15,8,metal.ink);ellipse(c,21,29,12,6,'#5e6859');line(c,22,26,28,32,metal.ink,2);rect(c,16,29,11,2,metal.ink);}
    return s;
  }
  function building(ctx,b,time=0){
    if(!b)return;const kind=b.kind||'robot',w=Math.round(b.w||(kind==='fort'?144:96)),h=Math.round(b.h||(kind==='fort'?144:80)),p=team(b.team),key=[kind,b.team,w,h,b.stars||0].join(':');
    if(b.hp<=0){let s=structures.get('ruin:'+key);if(!s){s=ruinBase(kind,w,h);structures.set('ruin:'+key,s);limit(structures,120);}ctx.imageSmoothingEnabled=false;ctx.drawImage(s,Math.round(b.x)-6,Math.round(b.y)-6);smoke(ctx,b.x+w*.55,b.y+h*.55,time,Math.round(b.x+b.y),1);return;}
    let s=structures.get(key);if(!s){s=buildingBase(kind,p,w,h,b.stars||0);structures.set(key,s);limit(structures,120);}
    const x=Math.round(b.x),y=Math.round(b.y);ctx.imageSmoothingEnabled=false;ctx.drawImage(s,x-6,y-6);
    const px=x+(kind==='fort'?28:12),py=y+(kind==='fort'?17:19);line(ctx,px,py+16,px,py-14,metal.ink,2);line(ctx,px,py-13,px,py+14,metal.light);
    const wave=Math.round(Math.sin(time*5+x*.1)*2);poly(ctx,[[px+1,py-13],[px+15,py-11+wave],[px+15,py-3+wave],[px+8,py-5],[px+1,py-5]],p.dark);poly(ctx,[[px+1,py-13],[px+14,py-11+wave],[px+14,py-5+wave],[px+7,py-6],[px+1,py-6]],p.flag);line(ctx,px+2,py-12,px+10,py-11,p.light);
    if(kind==='radar'){
      const rx=x+w-30,ry=y+14,a=time*.45,ex=Math.cos(a)*15,ey=Math.sin(a)*4;
      line(ctx,rx,ry+10,rx,ry-5,metal.dark,3);
      poly(ctx,[[rx-ex,ry-ey],[rx+ex,ry+ey],[rx+ex*.67,ry+ey*.67+8],[rx-ex*.67,ry-ey*.67+8]],metal.ink);
      poly(ctx,[[rx-ex,ry-ey-1],[rx+ex,ry+ey-1],[rx+ex*.65,ry+ey*.65+6],[rx-ex*.65,ry-ey*.65+6]],'#b8bd9c');
      line(ctx,rx-ex*.8,ry-ey*.8+1,rx+ex*.8,ry+ey*.8+1,'#e1dec0');line(ctx,rx,ry+6,rx+Math.sin(a)*8,ry-6,metal.dark,2);rect(ctx,rx+Math.sin(a)*8-1,ry-7,3,2,p.light);
    }
    if(b.progress>0&&b.progress<1){const q=Math.floor(time*5)%3;rect(ctx,x+w/2-4+q*3,y+h-36,2,1,q===1?'#f4d487':'#93c5a4');}
    if(b.maxHp>0&&b.hp/b.maxHp<.65){smoke(ctx,x+w*.72,y+h*.3,time,Math.round(x+y),1.2);if(b.hp/b.maxHp<.3)smoke(ctx,x+w*.3,y+h*.48,time+1,Math.round(x-y),1.6);}
  }
  const biomes=[
    {base:'#a48c5d',patch:'#aa9568',light:'#bda675',dark:'#89764f',grain:'#d2b984',road:'#77725b',edge:'#c6b183',rock:'#777968',rockLight:'#a0a18a',rockDark:'#4e594f',water:'#477e83',waterLight:'#71a49f',bank:'#6f887b',rough:'#8a8854'},
    {base:'#665d50',patch:'#716252',light:'#85745d',dark:'#4e4c45',grain:'#9f8265',road:'#45494a',edge:'#8c7d66',rock:'#4d5654',rockLight:'#7f8372',rockDark:'#313d3b',water:'#b34228',waterLight:'#f28d38',bank:'#3e3f38',rough:'#685c43'},
    {base:'#b7c5c1',patch:'#c7d3cc',light:'#e3e5d2',dark:'#91a9ab',grain:'#f3f0da',road:'#798e93',edge:'#d3d9c5',rock:'#7c9396',rockLight:'#b5c1b6',rockDark:'#526b74',water:'#3b697d',waterLight:'#83b5bc',bank:'#d5dfd0',rough:'#99aaa3'},
    {base:'#687751',patch:'#77845a',light:'#95a06c',dark:'#515f42',grain:'#b4aa75',road:'#716e53',edge:'#9a9265',rock:'#626e5c',rockLight:'#8e9a79',rockDark:'#3b5147',water:'#376f68',waterLight:'#6d9f84',bank:'#7d8b60',rough:'#4d6c46'},
    {base:'#858a7c',patch:'#929486',light:'#abb09a',dark:'#6d776d',grain:'#c3bea2',road:'#505c5e',edge:'#9ba38f',rock:'#717b73',rockLight:'#a3ad9b',rockDark:'#465750',water:'#467a7e',waterLight:'#7ba5a0',bank:'#6f8478',rough:'#697b59'}
  ];
  function tileAt(map,x,y){if(x<0||y<0||x>=map.width||y>=map.height)return -1;const q=Array.isArray(map.tiles[y])?map.tiles[y][x]:map.tiles[y*map.width+x];return typeof q==='number'?q:(q&&q.terrain)||0;}
  function coast(c,x,y,z,mask,p,seed){
    const same=(bit)=>(mask&bit)!==0;const t=same(1)?0:3,b=same(2)?0:4,l=same(4)?0:3,r=same(8)?0:4;
    const points=[[x+l+2,y+t],[x+z-r-3,y+t],[x+z-r,y+t+4],[x+z-r,y+z-b-4],[x+z-r-4,y+z-b],[x+l+3,y+z-b],[x+l,y+z-b-4],[x+l,y+t+4]];
    if(same(1)){points[0]=[x+l,y];points[1]=[x+z-r,y];}if(same(2)){points[4]=[x+z-r,y+z];points[5]=[x+l,y+z];}if(same(4)){points[0][0]=x;points[7][0]=x;points[6][0]=x;points[5][0]=x;}if(same(8)){points[1][0]=x+z;points[2][0]=x+z;points[3][0]=x+z;points[4][0]=x+z;}
    poly(c,points,p.water);
    for(let k=0;k<7;k++){const xx=x+3+hash(seed,k)*Math.max(1,z-7),yy=y+4+hash(seed,k,9)*Math.max(1,z-8);rect(c,xx,yy,2+hash(k,seed,2)*5,1,k%3?p.waterLight:p.water);}
    const edges=[];if(!same(1))edges.push([[x+l+3,y+t],[x+z-r-4,y+t]]);if(!same(2))edges.push([[x+l+3,y+z-b],[x+z-r-4,y+z-b]]);if(!same(4))edges.push([[x+l,y+t+4],[x+l,y+z-b-4]]);if(!same(8))edges.push([[x+z-r,y+t+4],[x+z-r,y+z-b-4]]);
    for(const e of edges){line(c,e[0][0],e[0][1],e[1][0],e[1][1],p.bank,2);line(c,e[0][0]-1,e[0][1]-1,e[1][0]-1,e[1][1]-1,p.waterLight);}
  }
  function road(c,map,x,y,z,p,type){
    const tx=x/z,ty=y/z,match=(xx,yy)=>{const t=tileAt(map,xx,yy);return t===1||t===5;},n=match(tx,ty-1),s=match(tx,ty+1),w=match(tx-1,ty),e=match(tx+1,ty),r=Math.round(z*.34),cx=x+z/2,cy=y+z/2;
    ellipse(c,cx,cy,r+2,r+2,p.edge);if(n)rect(c,cx-r-2,y,r*2+5,z/2,p.edge);if(s)rect(c,cx-r-2,cy,r*2+5,z/2+1,p.edge);if(w)rect(c,x,cy-r-2,z/2,r*2+5,p.edge);if(e)rect(c,cx,cy-r-2,z/2+1,r*2+5,p.edge);
    ellipse(c,cx,cy,r,r,p.road);if(n)rect(c,cx-r,y,r*2+1,z/2,p.road);if(s)rect(c,cx-r,cy,r*2+1,z/2+1,p.road);if(w)rect(c,x,cy-r,z/2,r*2+1,p.road);if(e)rect(c,cx,cy-r,z/2+1,r*2+1,p.road);
    const rng=seeded(x*79+y*43+map.world*67);for(let i=0;i<10;i++){const xx=x+rng()*z,yy=y+rng()*z;if((Math.abs(xx-cx)<r&&((n&&yy<cy)||(s&&yy>cy)))||(Math.abs(yy-cy)<r&&((w&&xx<cx)||(e&&xx>cx)))||Math.hypot(xx-cx,yy-cy)<r-2)rect(c,xx,yy,1+rng()*3,1,i%3?p.dark:p.light);}
    if(type===5){
      const vertical=(n||s)&&!(w&&e);if(vertical){for(let q=y;q<y+z;q+=5){rect(c,cx-r,q,r*2+1,4,'#9a987d');line(c,cx-r,q,cx+r,q,'#d1c6a0');line(c,cx-r,q+3,cx+r,q+3,'#5f695e');}line(c,cx-r-2,y,cx-r-2,y+z,metal.dark,2);line(c,cx+r+2,y,cx+r+2,y+z,metal.dark,2);}else{for(let q=x;q<x+z;q+=5){rect(c,q,cy-r,4,r*2+1,'#9a987d');line(c,q,cy-r,q,cy+r,'#d1c6a0');line(c,q+3,cy-r,q+3,cy+r,'#5f695e');}line(c,x,cy-r-2,x+z,cy-r-2,metal.dark,2);line(c,x,cy+r+2,x+z,cy+r+2,metal.dark,2);}
    }else if(map.world===4){if(n&&s)rect(c,cx,cy-4,1,8,'#c7bc86');else if(w&&e)rect(c,cx-4,cy,8,1,'#c7bc86');}
  }
  function rocks(c,map,x,y,z,p){
    const tx=x/z,ty=y/z,rng=seeded(x*359+y*887+map.world*67),adj=(dx,dy)=>tileAt(map,tx+dx,ty+dy)===2;
    const left=adj(-1,0)?0:3,right=adj(1,0)?0:3,top=adj(0,-1)?0:3,bottom=adj(0,1)?0:3;
    poly(c,[[x+left,y+top+4],[x+left+4,y+top],[x+z-right-5,y+top],[x+z-right,y+top+5],[x+z-right,y+z-bottom],[x+left,y+z-bottom]],p.rockDark);
    for(let n=0;n<6;n++){
      const xx=x+3+rng()*(z-8),yy=y+4+rng()*(z-9),rx=5+rng()*8,ry=4+rng()*6;
      const shape=[[xx-rx,yy],[xx-rx+2,yy-ry],[xx+rx*.3,yy-ry-2],[xx+rx,yy-ry*.2],[xx+rx-1,yy+ry],[xx-rx+3,yy+ry]];
      poly(c,shape,p.rockDark);poly(c,[[xx-rx+1,yy-1],[xx-rx+3,yy-ry],[xx+rx*.3,yy-ry-2],[xx+rx-2,yy-2],[xx+rx-4,yy+ry-3],[xx-rx+3,yy+ry-2]],p.rock);
      line(c,xx-rx+3,yy-ry,xx+rx*.3,yy-ry-2,p.rockLight);line(c,xx-rx+2,yy-ry+1,xx-rx+1,yy-1,p.rockLight);line(c,xx,yy-ry+1,xx-2,yy+2,p.rockDark);rect(c,xx-3,yy-ry+2,3,1,p.rockLight);
      if(map.world===2){poly(c,[[xx-rx+2,yy-ry],[xx+rx*.3,yy-ry-2],[xx+rx-2,yy-3],[xx,yy-4],[xx-2,yy-1],[xx-rx+2,yy-2]],p.light);}
    }
    if(!adj(0,1)){for(let q=3;q<z-3;q+=6){line(c,x+q,y+z-7,x+q-1,y+z-2,p.rockDark,2);rect(c,x+q+1,y+z-8,3,1,p.rockLight);}}
  }
  function scrub(c,x,y,z,world,p,seed){
    const rng=seeded(seed);for(let k=0;k<10;k++){
      const xx=x+2+rng()*(z-4),yy=y+2+rng()*(z-4);
      if(world===2){ellipse(c,xx,yy,4+rng()*4,2,'#8ca6a5');line(c,xx-3,yy-2,xx+3,yy-2,'#e2e7d5');}
      else if(world===1){poly(c,[[xx-3,yy+1],[xx-2,yy-3],[xx+2,yy-4],[xx+4,yy],[xx+2,yy+2]],p.rockDark);rect(c,xx-1,yy-3,2,1,p.rockLight);}
      else if(world===3){ellipse(c,xx,yy+1,5,2,'#344d35');line(c,xx,yy,xx-5,yy-3,'#54784d',2);line(c,xx,yy,xx+4,yy-5,'#8eaa68',2);line(c,xx,yy,xx+6,yy-1,'#5d8750',2);line(c,xx,yy,xx-2,yy-6,'#8eaa68');rect(c,xx,yy,1,2,'#af9c5c');}
      else if(world===4&&k%3===0){poly(c,[[xx-3,yy],[xx-1,yy-3],[xx+4,yy-2],[xx+3,yy+2],[xx-2,yy+2]],p.rockDark);rect(c,xx-1,yy-3,3,1,p.rockLight);}
      else{line(c,xx-3,yy,xx-4,yy-4,p.dark);line(c,xx,yy+1,xx,yy-5,p.rough,2);line(c,xx+1,yy,xx+3,yy-4,p.light);rect(c,xx-2,yy+1,5,1,p.dark);}
    }
  }
  function makeTerrain(map){
    const z=map.tileSize||32,w=map.width*z,h=map.height*z,world=map.world||0,p=biomes[world]||biomes[0],s=canvas(w,h),c=s.getContext('2d');
    rect(c,0,0,w,h,p.base);
    // Large overlapping mineral / soil washes use world coordinates, so tile edges never acquire a color seam.
    for(let y=-80;y<h+80;y+=65)for(let x=-80;x<w+80;x+=73){const v=hash(x,y,world);ellipse(c,x+v*60,y+hash(y,x,world)*50,30+v*60,12+hash(x,y,44)*24,v>.55?p.patch:p.base);}
    for(let y=0;y<h;y+=3)for(let x=0;x<w;x+=3){const q=hash(x,y,world+72);if(q>.84)rect(c,x+hash(y,x)*2,y,1+(q>.97?2:0),1,q>.96?p.grain:q>.91?p.dark:p.light);}
    // Fine fragments and subtle wind-carved streaks are placed independently of map cells.
    for(let y=13;y<h;y+=29)for(let x=7;x<w;x+=37){const q=hash(x,y,world);if(q>.57){const xx=x+q*20,yy=y+hash(y,x)*12;line(c,xx,yy,xx+3+q*7,yy+1,p.dark);line(c,xx+1,yy-1,xx+3+q*4,yy,p.light);}}
    for(let ty=0;ty<map.height;ty++)for(let tx=0;tx<map.width;tx++){
      const t=tileAt(map,tx,ty),x=tx*z,y=ty*z;if(t===3||t===5){let mask=0;if([3,5].includes(tileAt(map,tx,ty-1)))mask|=1;if([3,5].includes(tileAt(map,tx,ty+1)))mask|=2;if([3,5].includes(tileAt(map,tx-1,ty)))mask|=4;if([3,5].includes(tileAt(map,tx+1,ty)))mask|=8;coast(c,x,y,z,mask,p,tx*71+ty*29);}
    }
    for(let ty=0;ty<map.height;ty++)for(let tx=0;tx<map.width;tx++){
      const t=tileAt(map,tx,ty),x=tx*z,y=ty*z;if(t===4)scrub(c,x,y,z,world,p,tx*373+ty*997+world*111);if(t===2)rocks(c,map,x,y,z,p);
    }
    for(let ty=0;ty<map.height;ty++)for(let tx=0;tx<map.width;tx++){const t=tileAt(map,tx,ty);if(t===1||t===5)road(c,map,tx*z,ty*z,z,p,t);}
    return s;
  }
  function terrain(ctx,map){
    if(!map||!map.width||!map.height)return;const stamp=[map.width,map.height,map.tileSize||32,map.world||0,map.seed||0,map.revision||map.version||0].join(':');let s=lands.get(stamp);
    if(!s){s=makeTerrain(map);lands.set(stamp,s);while(lands.size>3)lands.delete(lands.keys().next().value);}
    ctx.imageSmoothingEnabled=false;ctx.drawImage(s,0,0);
  }
  function effects(ctx,fx,time=0){
    if(!fx)return;ctx.imageSmoothingEnabled=false;
    for(let i=0;i<fx.length;i++){
      const f=fx[i],age=Math.max(0,Number(f.age)||0),life=Math.max(.001,Number(f.life)||.5),q=age/life;if(q>=1)continue;
      const x=Math.round(f.x),y=Math.round(f.y),r=f.radius||16,seed=Math.round(x*13+y*31+i*7);
      if(f.kind==='shot'){
        if(Number.isFinite(f.x2)&&Number.isFinite(f.y2)){
          const dx=f.x2-x,dy=f.y2-y,start=Math.max(0,q-.24),end=Math.min(1,q+.25);
          line(ctx,x+dx*start,y+dy*start,x+dx*end,y+dy*end,'#bc753f80',3);line(ctx,x+dx*start,y+dy*start,x+dx*end,y+dy*end,'#fbd497');rect(ctx,x+dx*end,y+dy*end,2,1,'#fff1bd');continue;
        }
        const a=Number(f.angle)||0,len=(1-q)*(f.radius||8),dx=Math.cos(a)*len,dy=Math.sin(a)*len*.68;
        line(ctx,x-dx,y-dy,x+dx,y+dy,'#b9653266',3);line(ctx,x-dx*.7,y-dy*.7,x+dx*.7,y+dy*.7,'#ffd18a',2);rect(ctx,x-1,y-1,2,2,'#fff4c2');
        for(let k=0;k<3;k++){const a=hash(seed,k)*TAU;rect(ctx,x+Math.cos(a)*r*q,y+Math.sin(a)*r*q*.7,1,1,'#e9ae69');}
      }else if(f.kind==='explosion'){
        shadow(ctx,x,y+2,r*(.65+q*.3),r*.3);const core=Math.max(0,1-q*1.7);
        // Uneven lobe silhouettes, glowing interior fragments, expanding debris, then rising smoke.
        for(let k=0;k<7;k++){
          const a=hash(seed,k)*TAU,reach=r*(.17+q*.7),xx=x+Math.cos(a)*reach,yy=y+Math.sin(a)*reach*.7-q*r*.45,rr=r*(.25+hash(seed,k,8)*.22)*(1-q*.6);
          ellipse(ctx,xx+1,yy+2,rr,rr*.8,q<.4?'#574733':'#4b4b435f');
          if(core>0){ellipse(ctx,xx,yy,rr*.85,rr*.7,'#b64e2c');ellipse(ctx,xx-1,yy-1,rr*.6,rr*.5,q<.23?'#ffc86a':'#e5843e');ellipse(ctx,xx-2,yy-2,rr*.3,rr*.3,q<.2?'#fff1b5':'#f9b85a');}
        }
        for(let k=0;k<11;k++){const a=hash(seed,k,71)*TAU,rr=r*(.4+q*1.4),xx=x+Math.cos(a)*rr,yy=y+Math.sin(a)*rr*.6-q*(1-q)*r;const s=q<.3?2:1;rect(ctx,xx,yy,s,s,q<.5?'#ebad60':'#655748');if(q<.35)line(ctx,xx,yy,xx-Math.cos(a)*3,yy-Math.sin(a)*2,'#f6ce82');}
        if(q>.25){for(let k=0;k<4;k++){const xx=x+(hash(seed,k,99)-.5)*r,yy=y-q*r*1.2-k*r*.14,rr=r*(.12+q*.25);ellipse(ctx,xx,yy,rr,rr*.8,q<.7?'#7775657d':'#9a958a3c');ellipse(ctx,xx-rr*.25,yy-rr*.3,rr*.55,rr*.3,'#b0a48b33');}}
      }
    }
  }
  global.IndependentArt=Object.freeze({unit,building,terrain,effects});
})(window);
