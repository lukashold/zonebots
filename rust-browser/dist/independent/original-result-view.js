/* The classic result cards use separate win/loss paths and a 480x320 PC texture grid. */
(function(root,factory){
 const api=factory(root);if(typeof module==='object'&&module.exports)module.exports=api;else root.OriginalResultView=api;
})(typeof window==='object'?window:null,function(window){
 'use strict';
 const RANKS=['Rekrut','Korporal','Unteroffizier','Oberleutnant','Kapitän','Major','Oberst','General','Oberbefehlshaber','Feldherr'];
 const LABELS={rank:'RANG',time:'SCHLACHT DAUERTE',own:'EINHEITEN VERLOREN',enemy:'EINHEITEN AUSGESCHALTET',points:'PUNKTE'};
 const trunc=Math.trunc;
 const i8=n=>n<<24>>24, i16=n=>n<<16>>16;
 const two=n=>String(n).padStart(2,'0');
 function timeText(seconds){
  seconds|=0;const hours=trunc(seconds/3600),rest=seconds-i16(hours)*3600;
  const minutes=i8(trunc(rest/60));return `${i8(hours)} : ${two(minutes)} : ${two(i8(rest-minutes*60))}`;
 }
 function model(result,seconds,won,network){
  if(!Array.isArray(result?.losses)||result.losses.length!==4||!Number.isInteger(seconds))return null;
  const values={time:timeText(seconds),own:String(result.losses[0]).padStart(3),enemy:String(result.losses.slice(1).reduce((sum,value)=>i16(sum+value),0)).padStart(3)};
  const keys=won?['time','own','enemy']:['time','enemy','own'];
  if(won&&!network&&RANKS[result.rank]){keys.unshift('rank');values.rank=RANKS[result.rank];}
  if(won&&!network&&Number.isInteger(result.points)){keys.push('points');values.points=String(result.points).padStart(6);}
  return {won,network,title:won?'SIE HABEN GEWONNEN':'SIE HABEN VERLOREN',action:won?'OK':'Spiel verlassen',rows:keys.map(k=>({key:k,label:LABELS[k],value:values[k]}))};
 }
 function layout(view,w,h,measure,height=29){
  // The original 480x320 background scales independently on each axis.
  // Keep a minimum aspect ratio for the browser card so the controls also fit on phones.
  const width=Math.min(w,h*1.85),left=(w-width)/2, sx=width/1024,sy=h/768,ysx=width/1152;
  const panelX=left+width/2-669*sx/2,panelY=h/2-412*sy/2;
  const step=(362*sy-20*ysx)/5;
  const labelX=panelX+40*sx,rightX=panelX+(669-40)*sx;
  const title={text:view.title,font:2,x:trunc(w/2-measure(2,view.title)*sx/2),y:trunc((144+height/2)*sy)};
  const entries=view.rows.map(r=>{
   const slot=r.key==='rank'?0:r.key==='time'?1:r.key==='points'?4:(r.key==='own')===view.won?2:3;
   const rowY=panelY+step/2+10*ysx+slot*step;
   return {...r, x:trunc(labelX), valueX:trunc(rightX-measure(1,r.value)*sx), y:trunc(rowY), dividerY:rowY+step*.3};
  });
  const button={x:w/2-119*sx,y:h-212*sy,w:238*sx,h:56*sy};
  const action={text:view.action,font:2,x:trunc(w/2),y:trunc(button.y+button.h*.6+(view.won?3*sx:width/160))};
  return {w,h,scale:sx,title,entries,button,action,panel:[panelX,panelY,669*sx,362*sy],divider:[panelX+20*sx,609*sx,12*sy]};
 }
 function width(font,text){
  const chars=[...text];return chars.reduce((sum,c,i)=>{
   const glyph=font.glyphs[c];return sum+(glyph?glyph[6]+(font.kerning[c+(chars[i+1]??'')]??0):font.space);
  },0);
 }
 if(!window)return Object.freeze({model,layout,timeText,width});
 const document=window.document, source=document.currentScript.src;
 const fallbackMeasure=(font,text)=>{const ctx=canvas?.getContext('2d');if(!ctx)return text.length*14;ctx.font=`bold ${font===2?28:22}px Arial`;return ctx.measureText(text).width;};
 const loadImage=url=>new Promise((resolve,reject)=>{const image=new Image();image.onload=()=>resolve(image);image.onerror=reject;image.src=url;});
 let view=null,host=null,button=null,canvas=null,words=null,skin=null,hover=false,loading;
 function requestSkin(){
  if(loading||!['localhost','127.0.0.1','[::1]'].includes(window.location.hostname))return;
  const base=new URL('../original/result-view/',source);
  // The loopback pack is optional. Enable it in the local URL after extracting the assets.
  if(!new URLSearchParams(window.location.search).has('original-result-art'))return;
  loading=fetch(new URL('manifest.json',base)).then(r=>r.ok?r.json():null).then(async manifest=>{
   if(manifest?.version!==1)return;
   const names=['background','panel','divider','buttons','medium','large'];
   const files=await Promise.all(names.map(name=>loadImage(new URL(`${name}.png`,base))));
   skin={...Object.fromEntries(names.map((name,i)=>[name,files[i]])),fonts:{1:manifest.medium,2:manifest.large}};
   paint();
  }).catch(()=>{});
 }
 function rect(ctx,image,sourceBox,destination){ctx.drawImage(image,...sourceBox,...destination);}
 function text(ctx,line,g){
  const {text,font,x,y,center}=line;let left=x;const metrics=skin.fonts[font],chars=[...text];
  if(center)left=trunc(x-width(metrics,text)*g.scale/2);
  for(let i=0;i<chars.length;i++){
   const c=chars[i],glyph=metrics.glyphs[c];if(!glyph){left+=metrics.space*g.scale;continue;}
   const [sx,sy,sw,sh,dx,dy,advance]=glyph;
   if(sw>0&&sh>0)rect(ctx,font===2?skin.large:skin.medium,[sx,sy,sw,sh],[left+dx*g.scale,y+dy*g.scale,sw*g.scale,sh*g.scale]);
   left+=(advance+(metrics.kerning[c+(chars[i+1]??'')]??0))*g.scale;
  }
 }
 function spriteButton(ctx,g){
  const {x,y,w,h}=g.button, e=Math.min(32/3,h/3,w/3), dx=[x,x+e,x+w-e,x+w],dy=[y,y+e,y+h-e,y+h], shift=hover?32:0;
  for(let j=0;j<3;j++)for(let i=0;i<3;i++)rect(ctx,skin.buttons,[i*32/3,shift+j*32/3,32/3,32/3],[dx[i],dy[j],dx[i+1]-dx[i],dy[j+1]-dy[j]]);
 }
 function item(tag,description,value,x,y,size){
  const n=document.createElement(tag);n.className='classic-result-text';n.textContent=value;
  if(description)n.setAttribute('aria-label',description);
  Object.assign(n.style,{left:`${x}px`,top:`${y}px`,fontSize:`${size}px`});return n;
 }
 function paint(){
  if(!host||!view)return;
  const w=window.innerWidth,h=window.innerHeight;if(w<1||h<1)return;
  const measure=skin?(font,t)=>width(skin.fonts[font],t):fallbackMeasure;
  const g=layout(view,w,h,measure,skin?.fonts[2]?.height??29);
  host.classList.toggle('with-result-art',Boolean(skin));
  Object.assign(button.style,{left:`${g.button.x}px`,top:`${g.button.y}px`,width:`${g.button.w}px`,height:`${g.button.h}px`,fontSize:`${Math.max(12,23*g.scale)}px`});
  button.classList.toggle('with-result-art',Boolean(skin));
  words.replaceChildren(item('h2',null,view.title,g.title.x,g.title.y,28*g.scale));
  for(const row of g.entries){
   const [x,w,h]=g.divider, divider=document.createElement('div');
   divider.className='classic-result-divider';divider.setAttribute('aria-hidden','true');
   Object.assign(divider.style,{left:`${x}px`,top:`${row.dividerY}px`,width:`${w}px`,height:`${h}px`});
   words.append(item('div',null,row.label,row.x,row.y,22*g.scale),item('div',row.label+': '+row.value.trim(),row.value,row.valueX,row.y,22*g.scale),divider);
  }
  const panel=host.querySelector('.classic-result-panel');
  Object.assign(panel.style,{left:`${g.panel[0]}px`,top:`${g.panel[1]}px`,width:`${g.panel[2]}px`,height:`${g.panel[3]}px`});
  if(!skin)return;
  const ratio=Math.min(2,window.devicePixelRatio||1);
  canvas.width=Math.ceil(w*ratio);canvas.height=Math.ceil(h*ratio);const ctx=canvas.getContext('2d');
  ctx.setTransform(ratio,0,0,ratio,0,0);ctx.imageSmoothingEnabled=true;
  rect(ctx,skin.background,[0,0,480,320],[0,0,w,h]);
  rect(ctx,skin.panel,[0,0,669*.46875,362*5/12],g.panel);
  const [x,dw,dh]=g.divider;
  for(const row of g.entries)rect(ctx,skin.divider,[0,0,607*.46875,12*5/12],[x,row.dividerY,dw,dh]);
  text(ctx,g.title,g);
  for(const row of g.entries){text(ctx,{text:row.label,font:1,x:row.x,y:row.y},g);text(ctx,{text:row.value,font:1,x:row.valueX,y:row.y},g);}
  spriteButton(ctx,g);text(ctx,{...g.action,center:true},g);
 }
 function clear(){
  if(!host)return;host.classList.remove('classic-result','with-result-art');button.classList.remove('classic-result-action','with-result-art');
  for(const key of ['left','top','width','height','fontSize'])button.style[key]=key==='left'?'50%':key==='top'?'55%':'';
  host=button=canvas=words=view=null;hover=false;
 }
 function show(box,action,result,seconds,won,network){
  const next=model(result,seconds,won,network);if(!next){clear();return false;}
  host=box;button=action;view=next;box.classList.add('classic-result');action.classList.add('classic-result-action');
  action.textContent=next.action;
  const panel=document.createElement('div');panel.className='classic-result-panel';panel.setAttribute('aria-hidden','true');
  canvas=document.createElement('canvas');canvas.className='classic-result-art';canvas.setAttribute('aria-hidden','true');
  words=document.createElement('div');words.className='classic-result-words';box.replaceChildren(panel,canvas,words);
  paint();requestSkin();return true;
 }
 window.addEventListener('resize',()=>{if(host&&!host.hidden)paint();});
 for(const [name,value] of [['pointerenter',true],['pointerleave',false],['focus',true],['blur',false]]){
  document.getElementById('newBattle')?.addEventListener(name,()=>{if(skin&&host&&!host.hidden){hover=value;paint();}});
 }
 return Object.freeze({show,clear,model,layout,timeText,width});
});
