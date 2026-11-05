window.ResultFilm=(()=>{
 'use strict';
 const WIDTH=320,HEIGHT=200,PIXELS=WIDTH*HEIGHT,HEADER=14,STEP=40,MAX_FRAMES=1800;
 const MAX_BYTES=HEADER+MAX_FRAMES*(PIXELS+769);
 const signed=n=>n<128?n:n-256;
 function open(data){
  const bytes=data instanceof Uint8Array?data:new Uint8Array(data);
  if(bytes.length<HEADER||bytes[0]!==90||bytes[1]!==86||bytes[2]!==70||bytes[3]!==49)throw Error('Ungültiger Z-Film.');
  const d=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
  const width=d.getUint16(4,true),height=d.getUint16(6,true),frames=d.getUint16(8,true),frameMs=d.getUint16(10,true);
  const tailFade=bytes[12],leadFade=bytes[13];
  if(width!==WIDTH||height!==HEIGHT||!frames||frames>MAX_FRAMES||!frameMs||frameMs>1000||tailFade>7||leadFade>7)throw Error('Ungültige Filmdaten.');
  const offsets=new Uint32Array(frames);
  let offset=HEADER;
  for(let i=0;i<frames;i++){
   const flag=bytes[offset];
   if(flag!==0&&flag!==1)throw Error('Unvollständiges Filmbild.');
   offsets[i]=offset;
   offset+=1+(flag?768:0)+PIXELS;
  }
  if(offset!==bytes.length)throw Error('Ungültige Filmlänge.');
  function decoder(){
   const pixels=new Uint8Array(PIXELS);
   let next=0;
   function read(){
    if(next>=frames)return null;
    const frame=next++;
    let offset=offsets[frame];
    const changed=bytes[offset++]===1;
    const change=changed?bytes.subarray(offset,offset+768):null;
    if(changed)offset+=768;
    for(let i=0;i<PIXELS;i++)pixels[i]^=bytes[offset+i];
    return {frame,pixels,change};
   }
   return {read};
  }
  return {width,height,frames,frameMs,tailFade,leadFade,decoder};
 }
 function timeline(film){
  const display=new Uint8Array(768),colors=new Uint8Array(768),blank=new Uint8Array(768),read=film.decoder().read;
  let current=null,count=0,fade=null;
  function startFade(to,steps){
   if(!steps){display.set(to);fade=null;return;}
   fade={from:display.slice(),to,total:steps,step:0};
  }
  function fadeStep(){
   if(!fade)return;
   const f=fade;
   for(let i=0;i<display.length;i++){
    const a=signed(f.from[i]),b=signed(f.to[i]);
    display[i]=a+Math.trunc((b-a)*f.step/f.total);
   }
   if(++f.step>f.total)fade=null;
  }
  function tick(audioMs){
   if(count>=film.frames)return null;
   const due=Math.floor(Math.max(0,audioMs)/film.frameMs);
   if(current&&due!==0&&due<count){
    colors.set(display);return {frame:current.frame,pixels:current.pixels,palette:colors};
   }
   fadeStep();
   current=read();
   if(!current)return null;
   count++;
   colors.set(display);
   if(current.change)startFade(current.change,film.leadFade);
   if(film.tailFade&&film.frames-count===film.tailFade)startFade(blank,film.tailFade);
   return {frame:current.frame,pixels:current.pixels,palette:colors};
  }
  return {tick};
 }
 async function load(url,fetcher=fetch,{signal,rawBytes}={}){
  const response=await fetcher(url,{signal});
  if(!response.ok||!response.body)throw Error('Der Film ist nicht verfügbar.');
  const stream=response.body.pipeThrough(new DecompressionStream('gzip'));
  const reader=stream.getReader(),chunks=[];
  const exact=Number.isInteger(rawBytes)&&rawBytes>=HEADER&&rawBytes<=MAX_BYTES?new Uint8Array(rawBytes):null;
  let size=0;
  try{
   for(;;){
    const {done,value}=await reader.read();
    if(done)break;
    const before=size;size+=value.length;
    if(size>(exact?.length??MAX_BYTES)){await reader.cancel();throw Error('Der Film ist zu groß.');}
    if(exact)exact.set(value,before);else chunks.push(value);
   }
  }finally{reader.releaseLock();}
  if(exact){if(exact.length!==size)throw Error('Ungültige Filmlänge.');return open(exact);}
  const bytes=new Uint8Array(size);let offset=0;
  for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}
  return open(bytes);
 }
 function paint(ctx,image,frame){
  const {pixels,palette}=frame,out=image.data;
  for(let i=0,p=0;i<pixels.length;i++,p+=4){
   const src=pixels[i]*3;
   out[p]=palette[src];out[p+1]=palette[src+1];out[p+2]=palette[src+2];out[p+3]=255;
  }
  ctx.putImageData(image,0,0);
 }
 function select(mission,won,multiplayer=false){
  if(multiplayer||!Number.isInteger(mission)||mission<0||mission>=20)return null;
  const world='dvajc'[Math.floor(mission/4)],part=mission%4;
  const suffix=!won?'lost1':part<3?'wint'+(part+1):world==='c'?'wint1':'winpl';
  return 'e_'+world+suffix;
 }
 async function play(canvas,audio,film,{signal,onFrame}={}){
  if(signal?.aborted)return false;
  await audio.play();
  if(signal?.aborted){audio.pause();return false;}
  canvas.width=film.width;canvas.height=film.height;
  const ctx=canvas.getContext('2d'),image=ctx.createImageData(film.width,film.height),stream=timeline(film);
  const origin=performance.now();
  return new Promise((resolve,reject)=>{
   let handle=0,processed=-1,settled=false;
   function finish(ok,error){
    if(settled)return;
    settled=true;cancelAnimationFrame(handle);audio.pause();
    audio.removeEventListener('error',broken);signal?.removeEventListener('abort',stop);
    if(error)reject(error);else resolve(ok);
   }
   function broken(){finish(false,Error('Der Filmton konnte nicht abgespielt werden.'));}
   function stop(){finish(false);}
   function update(now){
    const elapsed=Math.max(0,now-origin),target=Math.floor(elapsed/STEP);
    const sample=audio.ended?Math.max(elapsed,audio.currentTime*1000):audio.currentTime*1000;
    let output=null;
    while(processed<target){
     const offset=(target-++processed)*STEP;
     const frame=stream.tick(Math.max(0,sample-offset));
     if(!frame){
      if(output){paint(ctx,image,output);onFrame?.(output.frame,film.frames);}
      finish(true);return;
     }
     output=frame;
    }
    if(output){paint(ctx,image,output);onFrame?.(output.frame,film.frames);}
    handle=requestAnimationFrame(update);
   }
   audio.addEventListener('error',broken,{once:true});signal?.addEventListener('abort',stop,{once:true});
   handle=requestAnimationFrame(update);
  });
 }
 const script=typeof document==='undefined'?null:document.currentScript?.src;
 let assets=null,catalogue=null,panel=null,current=null;
 function prepare(installed=false){
  if(!script||assets)return;
  const local=['127.0.0.1','localhost','[::1]'].includes(window.location?.hostname)&&
   new URLSearchParams(window.location.search).get('original-films')==='1';
  if(!local&&!installed)return;
  assets=new URL('../original/films/',script);
  fetch(new URL('films.json',assets)).then(r=>r.ok?r.json():null).then(data=>{
   if(data?.version!==1||!Array.isArray(data.films)||data.films.length>25)return;
   catalogue=new Map(data.films.filter(row=>row&&/^e_(?:outro|[dvajc](?:lost1|wint[1-3]|winpl))$/.test(row.id)&&
    row.video===row.id+'.zvf.gz'&&row.audio===row.id+'.ogg').map(row=>[row.id,row]));
  }).catch(()=>{});
 }
 function buildPanel(){
  const layer=document.createElement('section'),screen=document.createElement('canvas'),status=document.createElement('p'),
   skip=document.createElement('button'),resume=document.createElement('button'),controls=document.createElement('div');
  layer.id='originalResultFilm';layer.hidden=true;layer.setAttribute('role','dialog');layer.setAttribute('aria-modal','true');
  layer.setAttribute('aria-label','Originalfilm');screen.setAttribute('aria-label','Filmbild');status.setAttribute('role','status');
  skip.textContent='Film überspringen';resume.textContent='Film starten';resume.hidden=true;
  controls.append(resume,skip);layer.append(screen,status,controls);document.body.append(layer);
  skip.onclick=()=>finish(true);
  window.addEventListener('keydown',event=>{
   if(current&&event.key==='Escape'){event.preventDefault();event.stopImmediatePropagation();finish(true);}
  },true);
  return {layer,screen,status,skip,resume};
 }
 function finish(notify){
  const pending=current;
  if(!pending)return;
  current=null;
  pending.ctrl.abort();panel.layer.hidden=true;panel.resume.onclick=null;panel.resume.hidden=true;
  if(notify)pending.onFinish?.();
 }
 function offer(mission,won,network,{muted=false,volume=1,onFinish}={}){
  return offerClip(select(mission,won,network),{muted,volume,onFinish});
 }
 function offerClip(id,{muted=false,volume=1,onFinish}={}){
  const row=catalogue?.get(id);
  if(!row)return false;
  finish(false);panel??=buildPanel();
  const pending={ctrl:new AbortController(),onFinish};current=pending;
  panel.layer.hidden=false;panel.status.textContent='Film wird geladen …';panel.resume.hidden=true;panel.screen.hidden=true;panel.skip.focus();
  const audio=new Audio(new URL(row.audio,assets));audio.preload='none';audio.muted=muted;audio.volume=Math.max(0,Math.min(1,Number(volume)||0));
  load(new URL(row.video,assets),fetch,{signal:pending.ctrl.signal,rawBytes:row.rawBytes}).then(film=>{
   if(current!==pending)return;
   const start=()=>{panel.resume.hidden=true;panel.screen.hidden=false;panel.status.textContent='';
    play(panel.screen,audio,film,{signal:pending.ctrl.signal}).then(()=>{
     if(current===pending)finish(true);
    }).catch(error=>{
     if(current!==pending)return;
     if(error.name==='NotAllowedError'){panel.status.textContent='Film abspielbereit.';panel.resume.hidden=false;panel.resume.focus();}
     else finish(true);
    });
   };
   panel.resume.onclick=start;start();
  }).catch(()=>{if(current===pending)finish(true);});
  return true;
 }
 prepare();
 return {open,timeline,load,paint,select,play,offer,offerOutro:options=>offerClip('e_outro',options),enablePrivatePack:()=>prepare(true),cancel:()=>finish(false),stepMs:STEP};
})();
