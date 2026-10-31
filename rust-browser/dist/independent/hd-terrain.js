/* New, code-drawn HD terrain. No game artwork is embedded in this renderer. */
(function (global) {
  'use strict';
  const TILE = 32, CHUNK = 256, PAD = 2, MAX_CACHE_BYTES = 64*1024*1024;
  const cache = new Map(), overviews = new Map();
  let cacheBytes = 0;
  const wetCodes = new Set([9, 38, 39, 40, 42]);
  const palettes = [
    { ground:'#a38a63', wash:'#b89c6d', shade:'#766347', light:'#dfc59a', grit:'#67593e', road:'#7b7768', roadEdge:'#bcaa84', rock:'#74766b', rockHi:'#a2a494', rockLo:'#414a43', water:'#416e7b', waterHi:'#86afb4', shallow:'#669092', bank:'#887a5b' },
    { ground:'#62564b', wash:'#80664f', shade:'#3b3936', light:'#ad8d6a', grit:'#302f2e', road:'#42484a', roadEdge:'#877661', rock:'#4f5555', rockHi:'#8b8b7c', rockLo:'#252c2c', water:'#b53c20', waterHi:'#ffd16b', shallow:'#e76b23', bank:'#332c29' },
    { ground:'#b9cccd', wash:'#d8e0da', shade:'#88a5b2', light:'#f8faf1', grit:'#76939c', road:'#80999f', roadEdge:'#e2e8dd', rock:'#779096', rockHi:'#c8d4cf', rockLo:'#465d6a', water:'#315c79', waterHi:'#a5d9e2', shallow:'#659fb3', bank:'#e4eee8' },
    { ground:'#78805b', wash:'#a3a077', shade:'#4d644a', light:'#c6c391', grit:'#43513b', road:'#736c54', roadEdge:'#a7a078', rock:'#667264', rockHi:'#a2ad8a', rockLo:'#34483d', water:'#315e63', waterHi:'#7aac9b', shallow:'#528d7c', bank:'#77815a' },
    { ground:'#879083', wash:'#a7aa95', shade:'#646f67', light:'#c7ccb7', grit:'#515f58', road:'#49575c', roadEdge:'#a9b0a0', rock:'#717d77', rockHi:'#b1b8a6', rockLo:'#404e4c', water:'#3e707c', waterHi:'#91bdba', shallow:'#64988e', bank:'#879784' }
  ];

  function hash(a,b,c=0) {
    let x = Math.imul(a|0, 374761393) ^ Math.imul(b|0,668265263) ^ Math.imul(c|0,1442695041);
    x = Math.imul(x ^ (x >>> 13),1274126177);
    return ((x ^ (x >>> 16)) >>> 0) / 4294967296;
  }
  function canvas(w,h) {
    const image = document.createElement('canvas');
    image.width = Math.max(1,Math.ceil(w)); image.height = Math.max(1,Math.ceil(h));
    return image;
  }
  function path(ctx,points,fill,stroke,width=1) {
    ctx.beginPath(); ctx.moveTo(points[0][0],points[0][1]);
    for(let i=1;i<points.length;i++)ctx.lineTo(points[i][0],points[i][1]);
    if(fill){ctx.closePath();ctx.fillStyle=fill;ctx.fill();}
    if(stroke){ctx.strokeStyle=stroke;ctx.lineWidth=width;ctx.stroke();}
  }
  function ellipse(ctx,x,y,rx,ry,fill) {
    ctx.beginPath();ctx.ellipse(x,y,rx,ry,0,0,Math.PI*2);ctx.fillStyle=fill;ctx.fill();
  }
  function line(ctx,x,y,x2,y2,color,width=1) {
    ctx.beginPath();ctx.moveTo(x,y);ctx.lineTo(x2,y2);ctx.strokeStyle=color;ctx.lineWidth=width;ctx.stroke();
  }
  function tile(map,x,y) {
    if(x<0||y<0||x>=map.width||y>=map.height)return -1;
    const j=y*map.width+x, raw=Array.isArray(map.tiles[y])?map.tiles[y][x]:map.tiles[j];
    const t=typeof raw==='number'?raw:raw?.terrain||0;
    const code=(map.terrainCodes?.[j]??0)&127;
    // The current tile is authoritative after a bridge has been destroyed.
    if(t===3 && map.bridgeBase?.[j]>=0)return 3;
    if(t===5 || code===23)return 5;
    if(t===3 || wetCodes.has(code))return 3;
    if(t===2 || code===19 || code===43)return 2;
    if(code===7 || (t===1 && (!map.original || !map.terrainCodes)))return 1;
    return t;
  }
  function isWater(map,x,y){const t=tile(map,x,y);return t===3||t===5;}
  function isRoad(map,x,y){const t=tile(map,x,y);return t===1||t===5;}
  function worldKey(map) {
    // Engine snapshots recreate map objects every tick. Key immutable mission
    // identity and explicit geometry revision, not JavaScript object identity.
    return `${map.seed||0}:${map.world||0}:${map.revision||map.version||0}:${map.width}:${map.height}:${map.original?1:0}`;
  }

  function ground(ctx,map,x0,y0,size,p) {
    ctx.fillStyle=p.ground;ctx.fillRect(x0,y0,size,size);
    // Broad transparent mineral washes straddle cell and chunk boundaries.
    for(let gy=Math.floor((y0-120)/96);gy<=(y0+size+120)/96;gy++){
      for(let gx=Math.floor((x0-120)/96);gx<=(x0+size+120)/96;gx++){
        const q=hash(gx,gy,map.world+71),x=gx*96+q*72,y=gy*96+hash(gy,gx,20)*72;
        const g=ctx.createRadialGradient(x,y,1,x,y,90+q*28);
        g.addColorStop(0,q>.45?p.wash+'80':p.shade+'38');g.addColorStop(1,p.ground+'00');
        ctx.fillStyle=g;ctx.fillRect(x-120,y-120,240,240);
      }
    }
    // Fine stones and sand are placed in world space; no regular tile noise seams.
    for(let y=y0-3;y<y0+size+3;y+=4){
      for(let x=x0-3;x<x0+size+3;x+=4){
        const q=hash(x,y,map.seed||0),xx=x+q*3,yy=y+hash(y,x,7)*3;
        if(q<.37)continue;
        ellipse(ctx,xx+.4,yy+.55,q>.94?1.45:.38,q>.94?.6:.25,q>.76?p.light+'66':p.grit+'48');
        if(q>.968){ellipse(ctx,xx,yy,.85,.34,p.light+'bb');}
      }
    }
    // Fractured dry soil, exposed roots, or hairline ice fissures.
    for(let gy=Math.floor(y0/64)-1;gy<=(y0+size)/64;gy++){
      for(let gx=Math.floor(x0/64)-1;gx<=(x0+size)/64;gx++){
        const q=hash(gx,gy,54+map.world);if(q<.37)continue;
        const x=gx*64+q*41,y=gy*64+hash(gy,gx,84)*48;
        const points=[[x-10,y-3],[x-4,y-7],[x+4,y-4],[x+9,y-7],[x+15,y-3],[x+18,y+3]];
        ctx.save();ctx.globalAlpha=map.world===2?.32:.38;
        path(ctx,points,null,p.shade,.8);path(ctx,points.map(v=>[v[0],v[1]-.7]),null,p.light,.5);
        line(ctx,x+4,y-4,x+2,y+3,p.shade,.6);ctx.restore();
      }
    }
    if(map.world===3){
      for(let y=Math.floor((y0-4)/13)*13;y<y0+size+4;y+=13)for(let x=Math.floor((x0-4)/13)*13;x<x0+size+4;x+=13){
        const q=hash(x,y,304);if(q<.64)continue;const xx=x+q*8,yy=y+hash(y,x)*8;
        line(ctx,xx,yy,xx-1.5,yy-3.2,'#365b3b88',.7);line(ctx,xx+.7,yy,xx+2.6,yy-2,'#bdc28b80',.65);
      }
    }
  }

  function water(ctx,map,tx,ty,p) {
    const x=tx*TILE,y=ty*TILE;
    const n=isWater(map,tx,ty-1),s=isWater(map,tx,ty+1),w=isWater(map,tx-1,ty),e=isWater(map,tx+1,ty);
    ctx.save();ctx.beginPath();ctx.rect(x,y,TILE,TILE);ctx.clip();
    ctx.fillStyle=p.water;ctx.fillRect(x,y,TILE,TILE);
    // Every cell samples the same world-space material. A local diagonal
    // gradient would restart its light/deep colours every32 pixels.
    const g=ctx.createLinearGradient(0,0,map.width*TILE,map.height*TILE);
    g.addColorStop(0,p.waterHi+'26');g.addColorStop(.6,p.water+'00');g.addColorStop(1,'#142b3428');
    ctx.fillStyle=g;ctx.fillRect(x,y,TILE,TILE);
    for(let gy=Math.floor((y-96)/80);gy<=(y+TILE+96)/80;gy++)for(let gx=Math.floor((x-96)/80);gx<=(x+TILE+96)/80;gx++){
      const q=hash(gx,gy,map.world+212),xx=gx*80+q*36,yy=gy*80+hash(gy,gx,213)*36;
      const r=45+q*25,swirl=ctx.createRadialGradient(xx,yy,1,xx,yy,r);
      swirl.addColorStop(0,q>.55?p.waterHi+'22':'#142e3518');swirl.addColorStop(1,p.water+'00');
      ctx.fillStyle=swirl;ctx.fillRect(xx-r,yy-r,r*2,r*2);
    }
    // The lighter material belongs to the actual shore, fading inward. Wide
    // lakes and lava pools keep a continuous centre rather than a tile grid.
    function shallows(x1,y1,x2,y2,rx,ry,rw,rh){
      const fade=ctx.createLinearGradient(x1,y1,x2,y2);fade.addColorStop(0,p.shallow+'cc');fade.addColorStop(1,p.shallow+'00');
      ctx.fillStyle=fade;ctx.fillRect(rx,ry,rw,rh);
    }
    if(!n)shallows(x,y,x,y+8,x,y,32,8);
    if(!s)shallows(x,y+32,x,y+24,x,y+24,32,8);
    if(!w)shallows(x,y,x+8,y,x,y,8,32);
    if(!e)shallows(x+32,y,x+24,y,x+24,y,8,32);
    // Bank shadows and a curved wet margin follow the real passability mask.
    ctx.lineCap='round';
    const edges=[];
    if(!n)edges.push([[x,y+1.1],[x+10,y+2.1],[x+21,y+1.4],[x+32,y+1.1]]);
    if(!s)edges.push([[x,y+30.5],[x+11,y+29.8],[x+21,y+30.7],[x+32,y+30.5]]);
    if(!w)edges.push([[x+1.2,y],[x+2,y+11],[x+1.2,y+21],[x+1.2,y+32]]);
    if(!e)edges.push([[x+30.5,y],[x+29.7,y+10],[x+30.4,y+21],[x+30.5,y+32]]);
    for(const points of edges){
      path(ctx,points,null,p.bank,3.5);path(ctx,points.map(v=>[v[0]+.7,v[1]+.7]),null,'#172e352d',2);
      path(ctx,points,null,p.waterHi+'8a',.6);
    }
    // Rounded sandstone/ice corners soften the grid without changing shore cells.
    for(const [cx,cy,a,b] of [[x,y,n,w],[x+32,y,n,e],[x,y+32,s,w],[x+32,y+32,s,e]]){
      if(a||b)continue;ctx.save();ctx.beginPath();ctx.rect(x,y,32,32);ctx.clip();
      ellipse(ctx,cx,cy,3.4,3.4,p.bank);ellipse(ctx,cx,cy,2.3,2.3,p.ground);ctx.restore();
    }
    for(let k=0;k<4;k++){
      const h=hash(tx*9+k,ty,3),xx=x+3+h*25,yy=y+4+hash(ty*7+k,tx,8)*23;
      ctx.beginPath();ctx.moveTo(xx,yy);ctx.quadraticCurveTo(xx+2,yy-1,xx+5+h*4,yy);
      ctx.strokeStyle=p.waterHi+(map.world===1?'9c':'48');ctx.lineWidth=.5;ctx.stroke();
      if(map.world===1 && h>.65)ellipse(ctx,xx,yy,.8,.55,'#ffe8a3b0');
    }
    ctx.restore();
  }

  function roadShape(ctx,map,tx,ty,width) {
    const x=tx*TILE,y=ty*TILE,cx=x+16,cy=y+16;
    const n=isRoad(map,tx,ty-1),s=isRoad(map,tx,ty+1),w=isRoad(map,tx-1,ty),e=isRoad(map,tx+1,ty);
    const inset=(32-width)/2,l=w?x:x+inset,r=e?x+32:x+32-inset,t=n?y:y+inset,b=s?y+32:y+32-inset;
    const nw=!n&&!w?2:0,ne=!n&&!e?2:0,se=!s&&!e?2:0,sw=!s&&!w?2:0;
    // Raw roads often span several adjacent cells. Treat these as one surface,
    // with a shoulder only at its outside boundary, never a grid of lane dots.
    ctx.beginPath();ctx.moveTo(l+nw,t);ctx.lineTo(r-ne,t);ctx.quadraticCurveTo(r,t,r,t+ne);
    ctx.lineTo(r,b-se);ctx.quadraticCurveTo(r,b,r-se,b);ctx.lineTo(l+sw,b);ctx.quadraticCurveTo(l,b,l,b-sw);
    ctx.lineTo(l,t+nw);ctx.quadraticCurveTo(l,t,l+nw,t);ctx.closePath();
    return {n,s,w,e,x,y,cx,cy};
  }
  function road(ctx,map,tx,ty,p) {
    const a=roadShape(ctx,map,tx,ty,32);ctx.fillStyle=p.roadEdge;ctx.fill();
    roadShape(ctx,map,tx,ty,28);ctx.fillStyle=p.road;ctx.fill();
    ctx.save();roadShape(ctx,map,tx,ty,28);ctx.clip();
    for(let k=0;k<70;k++){
      const q=hash(tx*59+k,ty,63),x=a.x+q*32,y=a.y+hash(ty*47+k,tx,15)*32;
      ellipse(ctx,x,y,.22+q*.5,.16+q*.18,k%3?p.light+'22':p.grit+'48');
    }
    // Worn wheel ruts follow the outside lanes of a continuous road; broad
    // road interiors are fine gravel, rather than repeated crosshatching.
    const horizontal=(a.w||a.e)&&(!a.n||!a.s),vertical=(a.n||a.s)&&(!a.w||!a.e);
    ctx.strokeStyle='#202b2d15';ctx.lineWidth=1.2;
    if(vertical)for(const d of [-6,6]){ctx.beginPath();ctx.moveTo(a.cx+d,a.n?a.y:a.cy);ctx.bezierCurveTo(a.cx+d-.4,a.y+10,a.cx+d+.4,a.y+22,a.cx+d,a.s?a.y+32:a.cy);ctx.stroke();}
    if(horizontal)for(const d of [-6,6]){ctx.beginPath();ctx.moveTo(a.w?a.x:a.cx,a.cy+d);ctx.bezierCurveTo(a.x+10,a.cy+d-.4,a.x+22,a.cy+d+.4,a.e?a.x+32:a.cx,a.cy+d);ctx.stroke();}
    if(hash(tx,ty,603)>.78){
      const yy=a.y+hash(ty,tx,604)*24+4,xx=a.x+hash(tx,ty,605)*16;
      path(ctx,[[xx,yy],[xx+5,yy-1.6],[xx+8,yy+.6],[xx+12,yy-.8]],null,p.grit+'38',.55);
    }
    if(map.world===4){
      if(vertical&&a.n&&a.s)line(ctx,a.cx,a.cy-3,a.cx,a.cy+3,'#d9cf9a60',.75);
      if(horizontal&&a.w&&a.e)line(ctx,a.cx-3,a.cy,a.cx+3,a.cy,'#d9cf9a60',.75);
    }
    ctx.restore();
  }
  function bridge(ctx,map,tx,ty,p) {
    const x=tx*TILE,y=ty*TILE,vertical=isRoad(map,tx,ty-1)||isRoad(map,tx,ty+1);
    ctx.save();ctx.translate(x+16,y+16);if(!vertical)ctx.rotate(Math.PI/2);
    const g=ctx.createLinearGradient(-12,0,12,0);g.addColorStop(0,'#4b5758');g.addColorStop(.3,'#a3a99d');g.addColorStop(1,'#626c68');
    ctx.fillStyle='#142d3444';ctx.fillRect(-10,-16,26,32);
    ctx.fillStyle=g;ctx.fillRect(-11,-16,22,32);
    for(let q=-16;q<16;q+=4){line(ctx,-10,q,10,q,'#313f4299',.9);line(ctx,-10,q+.9,10,q+.9,'#d4d7c580',.5);}
    for(const d of [-12,12]){
      line(ctx,d,-16,d,16,'#283638',2.3);line(ctx,d-.5,-16,d-.5,16,'#bcc4ad',.7);
      for(let q=-12;q<16;q+=12){ellipse(ctx,d,q,1.4,1.4,'#3c4848');ellipse(ctx,d-.4,q-.5,.55,.55,'#e0dfb9');}
    }
    ctx.restore();
  }
  function rocks(ctx,map,x0,y0,size,p) {
    // One rock mask and a world-space field of facets avoid hard cuts every
    //32 pixels. The grid remains only in the actual impassable cliff boundary.
    ctx.save();ctx.beginPath();let present=false;
    for(let ty=Math.max(0,Math.floor(y0/32));ty<Math.min(map.height,Math.ceil((y0+size)/32));ty++)for(let tx=Math.max(0,Math.floor(x0/32));tx<Math.min(map.width,Math.ceil((x0+size)/32));tx++){
      if(tile(map,tx,ty)===2){ctx.rect(tx*32,ty*32,32,32);present=true;}
    }
    if(!present){ctx.restore();return;}ctx.clip();
    ctx.fillStyle=p.rockLo;ctx.fillRect(x0,y0,size,size);
    for(let gy=Math.floor((y0-24)/18);gy<=(y0+size+24)/18;gy++)for(let gx=Math.floor((x0-24)/21);gx<=(x0+size+24)/21;gx++){
      const h=hash(gx,gy,79),xx=gx*21+h*13,yy=gy*18+hash(gy,gx,67)*12;
      const rx=10+hash(gx,gy,87)*9,ry=6+hash(gy,gx,89)*7;
      const pts=[[xx-rx,yy],[xx-rx*.65,yy-ry],[xx+rx*.15,yy-ry-1],[xx+rx,yy-ry*.15],[xx+rx*.65,yy+ry],[xx-rx*.6,yy+ry*.7]];
      const g=ctx.createLinearGradient(xx-rx,yy-ry,xx+rx,yy+ry);
      g.addColorStop(0,p.rockHi);g.addColorStop(.42,p.rock);g.addColorStop(1,p.rockLo);
      path(ctx,pts,g,p.rockLo,.7);path(ctx,pts.slice(0,4),null,p.rockHi+'af',.8);
      line(ctx,xx-1,yy-ry*.8,xx+2,yy-1,p.rockLo+'88',.6);
      if(map.world===2)path(ctx,[[xx-rx*.7,yy-ry*.7],[xx-rx*.4,yy-ry],[xx+rx*.1,yy-ry-1],[xx+rx*.55,yy-ry*.4],[xx,yy-ry*.35]],'#e9f0e8e0');
    }
    for(let ty=Math.max(0,Math.floor(y0/32));ty<Math.min(map.height,Math.ceil((y0+size)/32));ty++)for(let tx=Math.max(0,Math.floor(x0/32));tx<Math.min(map.width,Math.ceil((x0+size)/32));tx++){
      if(tile(map,tx,ty)!==2||tile(map,tx,ty+1)===2)continue;const x=tx*32,y=ty*32;
      const g=ctx.createLinearGradient(x,y+23,x,y+32);g.addColorStop(0,p.rockLo+'00');g.addColorStop(1,p.rockLo+'a0');
      ctx.fillStyle=g;ctx.fillRect(x,y+23,32,9);
    }ctx.restore();
  }

  function makeChunk(map,cx,cy,density) {
    // Two real world pixels of neighbouring terrain are rasterised around the
    // chunk. Overlapping this gutter prevents fractional camera transforms
    // from filtering transparent image borders into dark vertical seams.
    const size=CHUNK+PAD*2,image=canvas(size*density,size*density),ctx=image.getContext('2d');
    const x0=cx*CHUNK-PAD,y0=cy*CHUNK-PAD,p=palettes[map.world]||palettes[0];
    ctx.scale(density,density);ctx.translate(-x0,-y0);
    ground(ctx,map,x0,y0,size,p);
    const tx0=Math.max(0,Math.floor(x0/32)),ty0=Math.max(0,Math.floor(y0/32));
    const tx1=Math.min(map.width,Math.ceil((x0+size)/32)),ty1=Math.min(map.height,Math.ceil((y0+size)/32));
    for(let ty=ty0;ty<ty1;ty++)for(let tx=tx0;tx<tx1;tx++){
      const t=tile(map,tx,ty);if(t===3||t===5)water(ctx,map,tx,ty,p);
    }
    rocks(ctx,map,x0,y0,size,p);
    for(let ty=ty0;ty<ty1;ty++)for(let tx=tx0;tx<tx1;tx++){
      const t=tile(map,tx,ty);
      if(t===1)road(ctx,map,tx,ty,p);else if(t===5)bridge(ctx,map,tx,ty,p);
    }
    return image;
  }

  function overview(ctx,map) {
    const key=worldKey(map);let image=overviews.get(key);
    if(!image){
      const scale=Math.min(1,512/(Math.max(map.width,map.height)*32));
      image=canvas(map.width*32*scale,map.height*32*scale);const c=image.getContext('2d'),p=palettes[map.world]||palettes[0];
      c.scale(scale,scale);c.fillStyle=p.ground;c.fillRect(0,0,map.width*32,map.height*32);
      for(let y=0;y<map.height;y++)for(let x=0;x<map.width;x++){
        const t=tile(map,x,y);if(t===0)continue;c.fillStyle=t===3?p.water:t===2?p.rock:t===1?p.road:t===5?'#a2aa9c':p.wash;c.fillRect(x*32,y*32,32,32);
      }
      overviews.clear();overviews.set(key,image);
    }
    ctx.drawImage(image,0,0,map.width*32,map.height*32);return true;
  }
  function terrain(ctx,map,time=0,viewport) {
    if(!map||!map.width||!map.height)return false;
    if(!viewport)return overview(ctx,map);
    const transform=ctx.getTransform?.(),projection=transform?Math.hypot(transform.a,transform.b):1;
    const density=projection<.7?1:2;
    const key=worldKey(map),x0=Math.max(0,Math.floor(viewport.x/CHUNK)),y0=Math.max(0,Math.floor(viewport.y/CHUNK));
    const x1=Math.min(Math.ceil(map.width*32/CHUNK)-1,Math.floor((viewport.x+viewport.w)/CHUNK));
    const y1=Math.min(Math.ceil(map.height*32/CHUNK)-1,Math.floor((viewport.y+viewport.h)/CHUNK));
    ctx.save();ctx.imageSmoothingEnabled=true;ctx.imageSmoothingQuality='high';
    for(let y=y0;y<=y1;y++)for(let x=x0;x<=x1;x++){
      const stamp=`${key}:${density}:${x}:${y}`;let image=cache.get(stamp);
      if(!image){
        image=makeChunk(map,x,y,density);cache.set(stamp,image);cacheBytes+=image.width*image.height*4;
        while(cacheBytes>MAX_CACHE_BYTES){const first=cache.keys().next().value,old=cache.get(first);cacheBytes-=old.width*old.height*4;cache.delete(first);}
      }
      else {cache.delete(stamp);cache.set(stamp,image);}
      ctx.drawImage(image,x*CHUNK-PAD,y*CHUNK-PAD,CHUNK+PAD*2,CHUNK+PAD*2);
    }
    // A low-opacity moving glint makes water alive without rebuilding its chunks.
    const p=palettes[map.world]||palettes[0],phase=(Number(time)||0)*.65;
    ctx.strokeStyle=p.waterHi+'32';ctx.lineWidth=.65;ctx.lineCap='round';
    for(let ty=Math.max(0,Math.floor(viewport.y/32));ty<Math.min(map.height,Math.ceil((viewport.y+viewport.h)/32));ty++){
      for(let tx=Math.max(0,Math.floor(viewport.x/32));tx<Math.min(map.width,Math.ceil((viewport.x+viewport.w)/32));tx++){
        if(tile(map,tx,ty)!==3||hash(tx,ty,6)<.65)continue;
        const q=hash(tx,ty,8),xx=tx*32+6+q*13,yy=ty*32+9+Math.sin(phase+q*12)*1.4;
        ctx.globalAlpha=.3+.6*Math.sin(phase+q*10)**2;ctx.beginPath();ctx.moveTo(xx,yy);ctx.quadraticCurveTo(xx+3,yy-1,xx+7,yy);ctx.stroke();
      }
    }
    ctx.restore();return true;
  }
  global.HDTerrain={terrain,overview,minimap:overview,clear(){cache.clear();overviews.clear();cacheBytes=0;},get cacheSize(){return cache.size;},get cacheBytes(){return cacheBytes;}};
})(window);
