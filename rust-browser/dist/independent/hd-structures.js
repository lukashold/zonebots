/* New resolution-independent scenery for the optional HD renderer.
 * Geometry uses game coordinates; no original game bitmap is embedded here. */
window.HDStructures = (() => {
  'use strict';
  const cache = new Map(), SCALE = 3, LIMIT = 40, MAX_BYTES = 48 * 1024 * 1024;
  let cacheBytes = 0,wear;
  const teams = ['#bc403b', '#416ea2', '#778570', '#b99442'];
  const team = n => teams[n] || '#72796d';
  const metal = ['#bfc2b7', '#858e88', '#535d5d', '#293534'];
  function path(c, points, fill, stroke, width = 1) {
    c.beginPath(); c.moveTo(...points[0]); for (const p of points.slice(1)) c.lineTo(...p); c.closePath();
    if (fill) { c.fillStyle = fill; c.fill(); } if (stroke) { c.strokeStyle = stroke; c.lineWidth = width; c.stroke(); }
  }
  function grad(c, x, y, w, h, colors = metal) {
    const g = c.createLinearGradient(x, y, x + w, y + h);
    colors.forEach((v, i) => g.addColorStop(i / (colors.length - 1), v)); return g;
  }
  function round(c, x, y, w, h, r, fill, stroke = '#27363b', width = 1) {
    c.beginPath(); c.roundRect(x, y, w, h, r); c.fillStyle = fill; c.fill();
    if (stroke) { c.strokeStyle = stroke; c.lineWidth = width; c.stroke(); }
  }
  function line(c, x, y, xx, yy, color = '#c8cebd', width = 1) {
    c.beginPath(); c.moveTo(x, y); c.lineTo(xx, yy); c.strokeStyle = color; c.lineWidth = width; c.stroke();
  }
  function ellipse(c, x, y, rx, ry, fill, stroke) {
    c.beginPath(); c.ellipse(x, y, Math.max(.01, rx), Math.max(.01, ry), 0, 0, Math.PI * 2);
    c.fillStyle = fill; c.fill(); if (stroke) { c.strokeStyle = stroke; c.lineWidth = .7; c.stroke(); }
  }
  function bolt(c, x, y, r = 1.25) { ellipse(c, x, y, r, r, '#d3d8cb', '#3b484a'); line(c, x - r / 2, y, x + r / 2, y, '#536064', .5); }
  function panel(c, x, y, w, h, color) {
    round(c, x, y, w, h, 2, color || grad(c, x, y, w, h), '#2c3a40', 1.2);
    if(w>40&&h>30){const dirt=c.createLinearGradient(x,y,x,y+h);dirt.addColorStop(0,'#f5edcb18');dirt.addColorStop(.14,'#e9dab300');dirt.addColorStop(.75,'#292d2620');dirt.addColorStop(1,'#171e1b4f');c.fillStyle=dirt;c.fillRect(x+1,y+1,w-2,h-2);}
    line(c, x + 2, y + 1.4, x + w - 2, y + 1.4, '#e0e5db99', .8);
    for (const xx of [x + 3.4, x + w - 3.4]) for (const yy of [y + 3.4, y + h - 3.4]) if (w > 12 && h > 12) bolt(c, xx, yy, .9);
  }
  function vent(c, x, y, w, h) {
    panel(c, x, y, w, h, '#344348');
    for (let i = 3; i < h - 2; i += 3.3) { line(c, x + 3, y + i, x + w - 3, y + i, '#101d22', 1.8); line(c, x + 3, y + i + .6, x + w - 3, y + i + .6, '#90a09c', .6); }
  }
  function pipe(c, x, y, xx, yy, w = 4) {
    c.lineCap = 'round'; line(c, x + 1, y + 2, xx + 1, yy + 2, '#16272c88', w + 2);
    line(c, x, y, xx, yy, '#3e5057', w); line(c, x - .5, y - .8, xx - .5, yy - .8, '#b1bdba', w * .4); c.lineCap = 'butt';
  }
  function hazard(c, x, y, w, h) {
    c.save(); c.beginPath(); c.rect(x, y, w, h); c.clip(); c.fillStyle = '#e1b758'; c.fillRect(x, y, w, h);
    for (let i = -h; i < w + h; i += 10) path(c, [[x + i, y], [x + i + 4.5, y], [x + i + h + 4.5, y + h], [x + i + h, y + h]], '#26383e');
    c.restore(); line(c, x, y, x + w, y, '#edce84', .6); line(c, x, y + h, x + w, y + h, '#1d2c31', 1);
  }
  function shadow(c, x, y, rx, ry, strength = .34) {
    c.save(); c.translate(x, y); c.scale(1, ry / rx);
    const g = c.createRadialGradient(0, 0, 1, 0, 0, rx);
    g.addColorStop(0, `rgba(15,23,25,${strength})`); g.addColorStop(1, 'rgba(15,23,25,0)');
    ellipse(c, 0, 0, rx, rx, g); c.restore();
  }
  function gate(c, x, y, w, h, accent) {
    panel(c, x - 5, y - 4, w + 10, h + 7, grad(c, x, y, w, h, ['#b3b9ad', '#536166', '#23363d']));
    round(c, x, y, w, h, 1.5, '#142128');
    const g = grad(c, x, y, w, h, ['#66797a', '#46575a', '#26393f']);
    for (let i = 0; i < h - 5; i += 5) { round(c, x + 1, y + i + 1, w - 2, 4, .5, g, '#202f33', .5); line(c, x + 2, y + i + 1.3, x + w - 2, y + i + 1.3, '#bdc5b955', .5); }
    hazard(c, x, y + h - 5, w, 5); panel(c, x - 6, y + 2, 4, h - 3, accent); panel(c, x + w + 2, y + 2, 4, h - 3, accent);
    for (const xx of [x - 4, x + w + 4]) { ellipse(c, xx, y + 4, 1.5, 1.5, '#bcf5b7'); ellipse(c, xx, y + h - 9, 1.5, 1.5, '#d89e55'); }
  }
  function stars(c, x, y, n) {
    panel(c, x - 4, y - 4, 14 * Math.max(1, n) + 4, 13, '#263a40');
    for (let j = 0; j < n; j++) {
      const points = []; for (let i = 0; i < 10; i++) { const a = i * Math.PI / 5 - Math.PI / 2, r = i % 2 ? 2 : 4.5; points.push([x + j * 13 + Math.cos(a) * r, y + 2 + Math.sin(a) * r]); }
      path(c, points, '#dec482', '#968257', .45);
    }
  }
  function tower(c, x, y, accent) {
    round(c, x - 28, y - 4, 56, 43, 11, grad(c, x - 28, y, 56, 43, ['#aeb8b5', '#6e7c7d', '#394e58']), '#283c43', 1.6);
    for (let i = -18; i <= 18; i += 12) { line(c, x + i, y + 9, x + i, y + 32, '#c5cec366', 1); line(c, x + i + 2, y + 9, x + i + 2, y + 32, '#1d333866', 1); }
    ellipse(c, x, y, 29, 19, '#203840'); ellipse(c, x - 1, y - 4, 28, 18, grad(c, x - 24, y - 15, 50, 27, ['#e1e3d4', '#b0b8ac', '#596e73']), '#3b5056');
    ellipse(c, x - 1, y - 5, 21, 12, '#82918d', '#c6cbbd'); ellipse(c, x - 1, y - 6, 15, 8, '#6b7d7d', '#334d55');
    for (let a = 0; a < Math.PI * 2; a += Math.PI / 3) bolt(c, x + Math.cos(a) * 24, y - 4 + Math.sin(a) * 14, 1);
    round(c, x - 26, y + 27, 52, 6, 1, accent, '#253b42', .7);
  }
  function fort(c, b) {
    const accent = team(b.team), w = 320, h = 384;
    shadow(c, 180, 244, 182, 155, .36);
    path(c,[[22,28],[306,26],[332,49],[333,372],[306,397],[39,398],[17,368]],'#1928223d');
    path(c, [[18,20],[300,20],[316,46],[316,360],[292,382],[26,382],[7,359],[7,46]], grad(c, 0, 0, w, h, ['#808a7e','#465a5d','#263d43']), '#253c42', 2);
    // Vertical armored flanks sit below the tower roofs, with a bright sunward
    // bevel and a dark east wall so their height remains clear at low zoom.
    path(c,[[8,48],[20,39],[20,349],[29,366],[29,380],[7,359]],grad(c,7,40,24,330,['#a0aeaa','#5e787c','#304c58']),'#263e46',.8);
    path(c,[[298,39],[316,46],[316,360],[292,382],[292,366],[301,349]],grad(c,292,40,28,340,['#5e787f','#2f4c58','#1a3541']),'#203d48',.8);
    for(const xx of[27,263]){
      panel(c,xx,54,31,263,grad(c,xx,54,31,263,['#819899','#496774','#2b4958']));
      for(let yy=69;yy<314;yy+=35){line(c,xx+2,yy,xx+28,yy,'#183744',1);line(c,xx+2,yy+1.3,xx+28,yy+1.3,'#b1c2b37a',.7);}
      for(let yy=65;yy<313;yy+=70){bolt(c,xx+5,yy,1);bolt(c,xx+25,yy,1);}
      pipe(c,xx+15,142,xx+15,258,4);
    }
    panel(c, 82, 30, 156, 315, grad(c, 70, 40, 170, 300, ['#9ba7a3','#596e75','#354f59']));
    // Recessed armored courtyard: seams, service rails and a central blast hatch.
    panel(c, 91, 43, 138, 294, '#637c84');
    for (let y = 49; y < 328; y += 17) { line(c, 94, y, 227, y, '#233f4e', 1); line(c, 94, y + 1.5, 227, y + 1.5, '#b2c2bf88', .8); }
    for (let x = 99; x < 229; x += 21) line(c, x, 44, x, 335, '#87a3aa55', .7);
    for (const x of [87,232]) pipe(c, x, 38, x, 338, 4);
    round(c, 130, 126, 61, 77, 4, grad(c, 130, 126, 61, 77, ['#9daea9','#6b8a93','#35525e']), '#d7c072', 1.2);
    line(c, 160, 129, 160, 200, '#35515b', 1.5); for (const x of [137,184]) for (const y of [134,195]) bolt(c, x, y, 1.3);
    for (const y of [29,125,365]) for (const x of [64,256]) tower(c, x, y, accent);
    // The six roof platforms deliberately retain the native gun socket centers.
    panel(c, 83, 206, 154, 32, grad(c, 80, 206, 150, 32));
    pipe(c, 89, 215, 231, 215, 5); for (const x of [95,223]) vent(c, x - 6, 220, 13, 13);
    panel(c, 138, 174, 48, 23, '#1e343a'); round(c, 146, 180, 36, 13, 1, '#091a18', '#738c7d', .6);
    const gy = b.orientation === 'north' ? 45 : 292;
    gate(c, 125, gy, 70, 48, accent);
    if (b.orientation === 'north') { panel(c, 114, 6, 92, 32, '#7b8c8d'); hazard(c, 126, 24, 68, 6); }
    else { panel(c, 114, 349, 92, 30, '#7b8c8d'); hazard(c, 126, 353, 68, 6); }
    for (const y of [77,260]) for (const x of [18,294]) { panel(c, x, y, 9, 53, '#576e75'); for(let i=0;i<4;i++)line(c,x+1,y+8+i*10,x+7,y+8+i*10,'#a8b4ad',.8); }
    stars(c, 141, 248, Math.min(5, b.stars || 1));
    // Fine oxidation at exposed joints and a few directional scratches are
    // deterministic surface detail, cached with the structure's vector art.
    for(const xx of[29,56,264,290])for(const yy of[79,149,289]){
      line(c,xx,yy,xx+.5,yy+8,'#976b4477',1.2);line(c,xx+1.2,yy+1,xx+1.5,yy+6,'#c1a26d66',.55);
    }
    for(let i=0;i<32;i++){const xx=31+(i*47%258),yy=48+(i*83%282);line(c,xx,yy,xx+3+(i%4),yy-.8,'#d8d9c52c',.55);}
  }
  function factory(c, b) {
    const w = b.w || 128, h = b.h || 160, accent = team(b.team), robot = b.kind === 'robot';
    shadow(c, w * .58, h * .65, w * .62, h * .47);
    path(c, [[5,17],[w-16,8],[w-3,23],[w-3,h-5],[13,h],[2,h-13]], '#344b51', '#24383d', 1.5);
    panel(c, 6, 16, w - 13, h - 22, grad(c, 10, 12, w, h, ['#b1bab2','#738b8e','#3f5964']));
    panel(c, 13, 12, w - 32, h - 50, grad(c, 14, 10, w - 20, h - 45, ['#c3c9ba','#8c9f9c','#536f79']));
    for (let x = 19; x < w - 24; x += 14) { line(c, x, 18, x, h - 43, '#334f5855', 1); line(c, x + 2, 18, x + 2, h - 43, '#d1d8c966', .8); }
    panel(c, 15, 95, w - 32, 12, accent);
    gate(c, w / 2 - 24, h - 46, 48, 38, accent);
    if (robot) {
      for (const x of [31,w-40]) {
        round(c, x-15, 25, 30, 46, 8, grad(c,x-16,25,32,46), '#3e565e',1.2);
        ellipse(c,x,26,15,8,'#bdc7bb','#4b6267'); ellipse(c,x-2,24,10,4,'#d2d8c8'); vent(c,x-10,36,20,22); pipe(c,x-11,72,x+11,72,4);
      }
      pipe(c,31,76,w-40,76,5); panel(c,50,18,25,20,accent); round(c,54,22,17,11,2,'#112b32');
      for (const x of [58,67]) ellipse(c,x,26,1.8,2.2,'#bbefd6');
      for(let i=0;i<4;i++){panel(c,7,110+i*8,12,6,'#566b70');line(c,10,112+i*8,16,112+i*8,'#c0c4aa',.7);}
      vent(c,w-24,111,15,24);
    } else {
      for(let x=18;x<w-22;x+=24) {panel(c,x,23,20,46);vent(c,x+3,28,14,35);}
      pipe(c,19,81,w-30,81,5); round(c,w-35,67,18,20,4,accent);ellipse(c,w-26,67,9,4,'#c5cbb9');
      panel(c,17,112,19,18,'#2d444c');for(let i=0;i<3;i++)line(c,20,116+i*4,32,116+i*4,'#9db7b3',1);
      vent(c,w-34,111,21,23);
    }
    // Native timer anchors: robot(70,116), vehicle(64,114).
    const tx=robot?68:62,ty=robot?114:112;panel(c,tx,ty,37,17,'#213b41');round(c,tx+2,ty+2,33,13,1,'#0b201b','#8fa58c',.7);
    stars(c,18,104,Math.min(5,b.stars||1));
    hazard(c,20,h-4,w-42,4);for(const x of[9,w-13])for(const y of[23,h-13])bolt(c,x,y,1.5);
  }
  function utility(c,b) {
    const w=b.w||128,h=b.h||(b.kind==='repair'?128:96),accent=team(b.team);
    shadow(c,w*.58,h*.65,w*.6,h*.44);panel(c,5,12,w-10,h-16,grad(c,5,12,w,h));
    panel(c,10,9,w-24,h-40,grad(c,12,9,w-20,h-30,['#c4cbbd','#8ba09f','#52707b']));
    for(let x=18;x<w-17;x+=17)line(c,x,14,x,h-35,'#4e6a7466',1);
    panel(c,11,h-34,w-23,9,accent);
    if(b.kind==='repair'){
      gate(c,w*.48,h-28,w*.33,22,accent);vent(c,19,20,26,38);panel(c,57,22,30,29,accent);
      round(c,69,27,6,19,1,'#e2dfc5',null);round(c,62,34,20,6,1,'#e2dfc5',null);pipe(c,20,70,w-21,70,5);
      for(let i=0;i<3;i++){round(c,13+i*11,h-23,7,13,1,'#9e7c48');ellipse(c,16+i*11,h-24,3.5,1.6,'#c4bb86');}
    }else{
      panel(c,13,24,38,21,'#314f5c');for(let i=0;i<3;i++)round(c,17+i*10,28,7,9,1,grad(c,17,28,8,9,['#c4ddd0','#65a9a8','#2e555f']),null);
      vent(c,16,50,32,11);gate(c,65,h-27,37,20,accent);pipe(c,83,24,83,51,7);ellipse(c,83,50,20,6,'#45636a','#b2c3b8');
    }
    for(const x of[8,w-14])for(const y of[18,h-10])bolt(c,x,y,1.3);
  }
  function ruin(c,b) {
    const w=b.kind==='fort'?320:b.w||128,h=b.kind==='fort'?384:b.h||160;
    shadow(c,w*.55,h*.65,w*.6,h*.42,.44);path(c,[[5,25],[w-14,15],[w-2,h-10],[15,h-2]],'#3f4e4c','#2b3e41',2);
    let seed=(w*17+h*7)|0;const rnd=()=>{seed=(Math.imul(seed,1664525)+1013904223)|0;return(seed>>>0)/4294967296;};
    for(let i=0;i<w*h/190;i++){const x=10+rnd()*(w-20),y=20+rnd()*(h-28),r=3+rnd()*11;path(c,[[x-r,y],[x-r*.5,y-r],[x+r*.8,y-r*.6],[x+r,y+r*.4],[x-r*.5,y+r*.5]],grad(c,x-r,y-r,2*r,2*r,['#a6a99a','#637773','#344a50']),'#263d43',.7);}
    for(let x=13;x<w-12;x+=27){pipe(c,x,h*.55,x+5,h*.25,2);line(c,x+5,h*.25,x+12,h*.3,'#2a4046',2);}
    gate(c,w*.34,h*.76,w*.32,h*.16,team(b.team));
  }
  function weather(c,w,h){
    if(!wear){wear=document.createElement('canvas');wear.width=wear.height=128;const q=wear.getContext('2d'),data=q.createImageData(128,128);let seed=63738;
      for(let n=0;n<128*128;n++){seed=(Math.imul(seed,1664525)+1013904223)|0;const v=seed>>>24,at=n*4,x=n%128,y=Math.floor(n/128),patch=Math.sin(x*.077+y*.02)+Math.sin(y*.12-x*.035);
        const rusty=patch>1.42,chip=v<19;data.data[at]=rusty?99:chip?48:v;data.data[at+1]=rusty?68:chip?49:v;data.data[at+2]=rusty?37:chip?43:v;data.data[at+3]=rusty?46:chip?54:28;}
      q.putImageData(data,0,0);}
    c.save();c.globalCompositeOperation='source-atop';c.fillStyle=c.createPattern(wear,'repeat');c.fillRect(-16,-16,w+32,h+32);
    c.globalAlpha=.27;for(let n=0;n<(w*h/2100);n++){const x=7+(n*47)%Math.max(1,w-14),y=8+(n*73)%Math.max(1,h-16);line(c,x,y,x+2+n%6,y-.6,n%3?'#bdb69c':'#553f28',.6);}
    c.restore();
  }
  function cached(key,w,h,paint) {
    let image=cache.get(key);if(image){cache.delete(key);cache.set(key,image);return image;}
    image=document.createElement('canvas');image.width=Math.ceil((w+32)*SCALE);image.height=Math.ceil((h+32)*SCALE);
    const c=image.getContext('2d');c.scale(SCALE,SCALE);c.translate(16,16);paint(c);weather(c,w,h);
    const bytes=image.width*image.height*4;
    // Canvas storage grows with area, so a count limit alone is insufficient
    // for several large, differently owned or damaged forts.
    if(bytes<=MAX_BYTES){cache.set(key,image);cacheBytes+=bytes;
      while(cache.size>LIMIT||cacheBytes>MAX_BYTES){const oldest=cache.keys().next().value,removed=cache.get(oldest);cacheBytes-=removed.width*removed.height*4;cache.delete(oldest);}}
    return image;
  }
  function smoke(c,x,y,t,scale=1) {
    for(let i=0;i<4;i++){const age=((t*.45+i*.25)%1+1)%1,rx=3+age*11;
      c.save();c.globalAlpha=(1-age)*.25;const g=c.createRadialGradient(x+age*11,y-age*34,0,x+age*11,y-age*34,rx*scale);g.addColorStop(0,'#22343d');g.addColorStop(1,'#374b5500');ellipse(c,x+age*11,y-age*34,rx*scale,rx*scale,g);c.restore();}
  }
  function building(c,b,time=0,map) {
    if(!b||!['fort','robot','vehicle','repair','radar'].includes(b.kind))return false;
    const w=b.kind==='fort'?320:b.w||128,h=b.kind==='fort'?384:b.h||160,dead=b.hp<=0;
    const x=b.kind==='fort'&&b.rawBase?b.rawBase[0]*2:b.x,y=b.kind==='fort'&&b.rawBase?b.rawBase[1]*2:b.y;
    const key=['b',b.kind,b.team,w,h,b.stars,b.orientation,dead].join(':');const image=cached(key,w,h,q=>dead?ruin(q,b):b.kind==='fort'?fort(q,b):['robot','vehicle'].includes(b.kind)?factory(q,b):utility(q,b));
    c.save();c.imageSmoothingEnabled=true;c.imageSmoothingQuality='medium';c.drawImage(image,x-16,y-16,w+32,h+32);
    if(!dead&&b.kind==='radar'){
      const a=time*.55,rx=x+83,ry=y+18; c.save();c.translate(rx,ry);c.scale(Math.max(.12,Math.abs(Math.cos(a))),1);
      c.beginPath();c.moveTo(-28,-12);c.bezierCurveTo(-20,18,20,18,28,-12);c.quadraticCurveTo(0,3,-28,-12);c.fillStyle=grad(c,-28,-12,56,30,['#e0e2cf','#8faaa9','#446575']);c.fill();c.strokeStyle='#43616b';c.lineWidth=1;c.stroke();
      for(let i=-18;i<=18;i+=9){c.beginPath();c.moveTo(i,-8);c.quadraticCurveTo(i*.4,15,0,8);c.strokeStyle='#3b5c6a77';c.lineWidth=.7;c.stroke();}line(c,0,7,0,-14,'#24434e',2);ellipse(c,0,-14,2,2,'#d1b37a');c.restore();
    }
    if(!dead&&b.team>=0){const lx=x+(b.kind==='fort'?109:20),ly=y+(b.kind==='fort'?219:90);ellipse(c,lx,ly,1.8,1.4,(Math.floor(time*2+b.id)%3===0)?'#d5efab':'#66977f');}
    if(dead||(b.maxHp&&b.hp/b.maxHp<.5))smoke(c,x+w*.58,y+h*.49,time,dead?1.5:1);
    c.restore();return true;
  }
  function house(c,type) {
    const p=type===19?['#f1eee0','#b5c9c9','#7f999c']:type===12?['#d0b987','#9f875a','#675c3c']:type===17?['#949489','#646d6a','#3d5054']:type===18?['#c2c9c0','#879c9c','#4b6c79']:['#dbc798','#b0956d','#75674d'];
    shadow(c,18,27,23,11);round(c,3,8,26,23,3,grad(c,3,8,26,23,p),'#4b5047',.8);
    path(c,[[1,11],[9,1],[24,1],[32,11],[27,19],[5,19]],grad(c,1,2,31,17,p),'#706c52',.8);
    line(c,9,2,24,2,'#fff3c688',.7);round(c,13,21,7,11,2,'#34484a','#a19a75',.5);round(c,5,21,5,5,1,'#527c80','#cfbf91',.6);round(c,22,21,5,5,1,'#527c80','#cfbf91',.6);
    if(type===18){vent(c,8,6,16,8);pipe(c,28,12,28,23,2);}if(type===19){path(c,[[1,11],[9,1],[24,1],[31,11],[27,12],[6,12]],'#e3e9df');}
    if(type===12){for(let i=5;i<28;i+=3)line(c,i,9,i,16,'#6c633a66',.6);}
  }
  function rock(c,type,size=64) {
    const snow=type===60||type===61,volc=type===52||type===53,r=type===42?18:size*.35,x=size*.47,y=size*.63;
    shadow(c,x+5,y+8,r*1.3,r*.65);
    const pts=[[x-r,y],[x-r*.65,y-r*.85],[x+r*.18,y-r*1.15],[x+r,y-r*.35],[x+r*.9,y+r*.4],[x-r*.3,y+r*.6]];
    path(c,pts,grad(c,x-r,y-r,r*2,r*2,snow?['#f2f5e9','#c9d9d7','#8ca9b0']:volc?['#7c7772','#49595c','#293f47']:['#a6ac98','#7d8a80','#415d64']),'#3c535b',.8);
    path(c,[pts[0],pts[1],pts[2],[x+r*.1,y-r*.1],[x-r*.4,y+r*.2]],snow?'#e6eee3':volc?'#777977':'#a3ad98');
    line(c,x+r*.18,y-r*1.15,x+r*.1,y-r*.1,snow?'#b1c5c5':'#bdc1a5',.7);line(c,x-r*.4,y+r*.2,x-r*.65,y-r*.85,'#40586166',1);
  }
  function cactus(c,type) {
    const small=type===43,x=small?16:28,y=small?30:60,h=small?23:49,r=small?3.5:5;
    shadow(c,x+4,y+1,small?12:22,6);
    c.lineCap='round';line(c,x+1,y+2,x+1,y-h,'#2f5140',r*2+2);line(c,x,y,x,y-h,grad(c,x-r,y-h,r*2,h,['#a8b275','#6d9566','#335d4b']),r*2);
    for(const side of[-1,1]){const yy=y-h*(side<0?.5:.65),xx=x+side*(small?8:15);c.beginPath();c.moveTo(x,yy);c.lineTo(xx,yy);c.quadraticCurveTo(xx+side*3,yy,xx+side*3,yy-5);c.lineTo(xx+side*3,yy-h*.23);c.strokeStyle='#527e55';c.lineWidth=r*1.5;c.stroke();line(c,xx+side*2,yy-3,xx+side*2,yy-h*.23,'#a6b776',.9);}
    line(c,x-2,y-3,x-2,y-h+2,'#bdc48699',1);line(c,x+2,y-3,x+2,y-h+1,'#254f4266',.8);
    for(let yy=y-7;yy>y-h;yy-=7){line(c,x-r+1,yy,x-r-1,yy-1,'#dad3a5',.6);line(c,x+r-1,yy-3,x+r+1,yy-4,'#cfc99a',.6);}c.lineCap='butt';
  }
  function tree(c,type) {
    const arctic=type===46||type===47,x=31,y=61;shadow(c,x+7,y,27,10);
    pipe(c,x,y,x-3,18,arctic?5:6);
    if(arctic){for(let i=0;i<4;i++){const yy=8+i*11,r=9+i*4;path(c,[[x-3,yy-7],[x+r,yy+16],[x+3,yy+14],[x-r,yy+17]],grad(c,x-r,yy-5,r*2,24,['#d9e5d9','#5c8c80','#2e5e60']),'#325357',.6);path(c,[[x-3,yy-7],[x+r*.6,yy+7],[x+2,yy+4],[x-r*.6,yy+8]],'#d8e5da');}}
    else{for(let i=0;i<7;i++){const a=i*Math.PI*2/7,xx=x+Math.cos(a)*23,yy=20+Math.sin(a)*11;c.beginPath();c.moveTo(x-2,19);c.quadraticCurveTo(xx,yy-19,xx+Math.cos(a)*4,yy+9);c.quadraticCurveTo(xx-7,yy-1,x-2,19);c.fillStyle=grad(c,x,0,xx-x||1,32,['#a6bb68','#548655','#2c5b46']);c.fill();line(c,x-2,19,xx,yy+1,'#bdc88766',.7);}ellipse(c,x-2,19,4,4,'#6c7748');}
  }
  function object(c,o,map) {
    if(!o||[0x31,0x32].includes(o.templateFlags)||[5,8,9,10,21,22,23,24,26,29,31,33,35].includes(Number(o.type)))return false;
    const type=Number(o.type),size=type===43||type===61?32:type>=39?64:32;
    const key='o:'+type,im=cached(key,size,size,q=>{
      if([11,12,17,18,19].includes(type))house(q,type);
      else if([39,40,41,43].includes(type))cactus(q,type);
      else if([46,47,56,57].includes(type))tree(q,type);
      else if(type>=39&&type<=61)rock(q,type,size);
      else if(type>=13&&type<=16){rock(q,52,32);ellipse(q,16,18,9,4,'#e57d35');ellipse(q,15,17,5,2,'#ffd386');}
      else if(type===0){ellipse(q,16,20,12,6,'#445b53','#92a38c');ellipse(q,15,18,4,2,'#d9a358');}
      else if(type===75){ellipse(q,16,20,7,12,'#df793c');ellipse(q,15,20,4,8,'#ffcc6c');}
      else rock(q,48,32);
    });
    c.save();c.imageSmoothingEnabled=true;c.drawImage(im,o.x-16,o.y-16,size+32,size+32);c.restore();return true;
  }
  function crate(c,box,map) {
    if(!box||box.taken)return false;c.save();c.translate(box.x,box.y);shadow(c,18,28,21,7);
    panel(c,2,7,28,24,grad(c,2,7,28,24,['#7f9d61','#426c4a','#234c3e']));panel(c,3,3,26,8,'#75985d');
    for(const x of[7,24])round(c,x,5,2,26,.5,'#c9c797','#2d4c3f',.5);
    round(c,12,16,10,9,1,'#d1d4a3','#254c43',.6);round(c,15,12,4,5,1,'#d1d4a3',null);line(c,18,12,23,14,'#f0eab4',1);
    c.restore();return true;
  }
  function flag(c,s,time=0,map) {
    if(!s)return false;c.save();c.translate(s.x-16,s.y-16);pipe(c,2,32,2,0,2);ellipse(c,2,-1,1.8,1.8,'#d6c989');
    const wave=Math.sin(time*4+s.id)*2.5,g=grad(c,4,1,29,18,[team(s.owner),'#d5d6bc',team(s.owner)]);
    c.beginPath();c.moveTo(4,1);c.bezierCurveTo(13,-3,21,7+wave,32,3+wave);c.lineTo(32,20+wave);c.bezierCurveTo(21,24+wave,13,14,4,18);c.closePath();c.fillStyle=g;c.fill();c.strokeStyle='#142f3f88';c.lineWidth=.6;c.stroke();
    c.beginPath();c.moveTo(4,3);c.bezierCurveTo(14,-1,22,9+wave,31,5+wave);c.strokeStyle='#fff0c266';c.lineWidth=.7;c.stroke();c.restore();return true;
  }
  function objectShadows(){return true;}
  function flagShadows(c,sectors,time,map){for(const s of sectors)shadow(c,s.x+8,s.y+17,21,5,.28);return true;}
  function effects(c,fx,time,map) {
    for(const f of fx||[]){if(!f||!Number.isFinite(f.x)||!Number.isFinite(f.y))continue;
      const life=Number.isFinite(f.life)&&f.life>0?f.life:.5,elapsed=Number.isFinite(f.age)?Math.max(0,f.age):0;
      const radius=Number.isFinite(f.radius)&&f.radius>0?f.radius:22,age=Math.min(1,elapsed/life),r=radius*(.35+age),x=f.x,y=f.y;if(age>=1)continue;
      if(f.kind==='shot'){
        const angle=Number.isFinite(f.angle)?f.angle:0;
        const dx=Number.isFinite(f.x2)?f.x2-x:Math.cos(angle)*radius,dy=Number.isFinite(f.y2)?f.y2-y:Math.sin(angle)*radius;
        const start=Math.max(0,age-.18),end=Math.min(1,age+.23),xx=x+dx*end,yy=y+dy*end;
        c.save();c.globalAlpha=1-age*.6;c.lineCap='round';line(c,x+dx*start,y+dy*start,xx,yy,'#db854449',4);
        line(c,x+dx*start,y+dy*start,xx,yy,'#f5cc80',1.2);ellipse(c,xx,yy,1.5,1.1,'#fff3c3');
        if(age<.35){const glow=c.createRadialGradient(x,y,0,x,y,7*(1-age));glow.addColorStop(0,'#fff8c5cc');glow.addColorStop(1,'#ffb54b00');ellipse(c,x,y,7*(1-age),6*(1-age),glow);}
        c.restore();continue;
      }
      if(f.kind==='grenade'){
        const dx=(Number.isFinite(f.x2)?f.x2:x)-x,dy=(Number.isFinite(f.y2)?f.y2:y)-y,xx=x+dx*age,ground=y+dy*age,yy=ground-Math.sin(age*Math.PI)*Math.min(45,Math.hypot(dx,dy)*.24);
        shadow(c,xx+2,ground+3,4,2,.24);ellipse(c,xx,yy,2.7,3.5,'#749475','#243d3d');line(c,xx-1,yy-3,xx+2,yy-4,'#dfd2a6',1);continue;
      }
      if(f.kind==='smoke'){smoke(c,x,y,(Number.isFinite(time)?time:0)+age*2,Math.max(.5,radius/10));continue;}
      if(f.kind!=='explosion')continue;
      c.save();c.globalAlpha=1-age;const g=c.createRadialGradient(x,y,0,x,y,r);g.addColorStop(0,'#fff8d2');g.addColorStop(.2,'#ffe69b');g.addColorStop(.5,'#e88936');g.addColorStop(.78,'#8d4c30aa');g.addColorStop(1,'#3f4a4900');ellipse(c,x,y,r,r*.8,g);
      for(let i=0;i<9;i++){const a=i*2.399+f.x,r2=r*(.6+age*.9);line(c,x+Math.cos(a)*r2*.6,y+Math.sin(a)*r2*.6,x+Math.cos(a)*r2,y+Math.sin(a)*r2,'#f3cb87',1.5*(1-age));}c.restore();
    }return true;
  }
  return {building,object,crate,flag,objectShadows,flagShadows,effects,
    get cacheStats(){return {entries:cache.size,bytes:cacheBytes,maxBytes:MAX_BYTES,maxEntries:LIMIT};}};
})();
