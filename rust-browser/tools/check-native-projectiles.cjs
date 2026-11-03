// Real Wasm flight/impact checks. Arenas and expected numeric rules are synthetic.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
function arena(type){const width=64,height=48,n=width*height;return{map:{width,height,tileSize:32,world:0,seed:7291,revision:0,original:true,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),territory:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[]},sectors:[],buildings:[],walls:[],originalObjects:[],crates:type==='grunt'?[{x:320,y:600,taken:false}]:[],spawns:[{type,team:0,x:320,y:600,squad:1},{type:'gatling',team:1,x:520,y:600,squad:2},{type:'crane',team:0,x:1600,y:1300,squad:3}]};}
(async()=>{
 const e=(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{})).instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 const env={window:{},setTimeout,clearTimeout,Date};vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),env);let contacts=0;
 for(const type of ['grunt','tough','pyro','light','medium','heavy','cannon','missile','launcher','howitzer']){
  let s=req({op:'init',mission:0,difficulty:1,seed:7291,aiTeams:[],originalMission:arena(type)});const target=s.units.find(u=>u.team===1),id=target.id,targetHp=target.hp; //A fixed gun cannot dodge the incoming shot.
  for(let tick=1;tick<=100;tick++){s=req({op:'tick',dt:.059});if(s.projectiles.length)break;}
  assert(s.projectiles.length,`${type}: first traveling projectile must exist`);assert.equal(s.units.find(u=>u.id===id).hp,targetHp,`${type}: release is not contact`);
  assert.equal(s.projectiles.length,type==='missile'?2:type==='launcher'?3:1,`${type}: salvo size`);
  for(const p of s.projectiles){assert([p.x,p.y,p.angle,p.size].every(Number.isFinite));assert(p.id<50);assert(p.sprite>=0&&p.sprite<7211);}
  const guest=env.window.MultiplayerGame.perspective(s,'guest');assert(guest.projectiles.every(p=>p.team===1));assert(s.projectiles.every(p=>p.team===0));
  let seenImpact=false,seenDelayed=false,seenFire=false;
  for(let tick=1;tick<=160;tick++){
   s=req({op:'tick',dt:.059});seenFire ||=s.fires.length>0;
   if(s.blasts.some(b=>b.frame===0)){seenImpact=true;seenDelayed ||=s.units.find(u=>u.id===id)?.hp===targetHp;}
   const hp=s.units.find(u=>u.id===id)?.hp??-1;if(hp<targetHp){contacts++;break;}
  }
  assert((s.units.find(u=>u.id===id)?.hp??-1)<targetHp,`${type}: stationary target must eventually receive physical damage`);
  if(type==='pyro')assert(seenFire,'Flamethrower leaves a persistent native fire object');
  else{assert(seenImpact,`${type}: impact must create its independent blast`);assert(seenDelayed,`${type}: frame0 blast must precede hull damage`);}
 }
 // The renderer uses real frame dimensions with synthetic opaque atlas texels.
 const metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));let requested=null,draws=[],transforms=[];
 metadata.frames=new Proxy(metadata.frames,{get(a,k){if(/^\d+$/.test(String(k))){requested=Number(k);assert(a[k],`Missing native sprite${k}`);}return a[k];}});
 const pixels=new Proxy({},{get:(_,k)=>Number(k)%4===3?255:16});
 function canvas(){const c={width:0,height:0,frame:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(image){c.pixels=image.data;},drawImage(){},getImageData:()=>({data:pixels})});return c;}
 class Image{set src(v){this.width=String(v).endsWith('sprites.png')?2048:512;this.height=4526;queueMicrotask(()=>this.onload());}}
 const render={window:{},URL,Image,document:{currentScript:{src:'https://local.test/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
 vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),render);const art=render.window.OriginalArt;assert(await art.load());
 const ctx={save(){},restore(){},rotate(a){assert(Number.isFinite(a));transforms.push(['rotate',a]);},scale(x,y){assert([x,y].every(Number.isFinite));transforms.push(['scale',x,y]);},translate(x,y){assert([x,y].every(Number.isFinite));transforms.push(['translate',x,y]);},drawImage(im,x,y,w,h){assert([x,y,w,h].every(Number.isFinite));draws.push({frame:im.frame,x,y,w,h,pixel:Array.from(im.pixels?.slice(0,4)||[])});}};
 art.projectiles(ctx,[{x:100,y:200,team:0,kind:'shell',sprite:1634,angle:1,size:8}],{original:true,world:0});
 assert.equal(draws[0].frame,1634);assert.equal(draws[0].w,metadata.frames[1634].w*3,'Native size8 grows the width by1.5');
 const u={type:'grunt',x:100,y:200,team:0,angle:0,aim:0,moving:true,order:'move',cooldown:0,nativeWalk:{frame:3},originalCombat:{}};
 draws=[];art.unit(ctx,u,100,{original:true,world:0});assert.equal(draws[0].frame,85,'Walk frame comes from Rust, independently of wall time');
 for(const [phase,type,base,stride]of [[20,'grunt',127,5],[21,'tough',226,3],[66,'grunt',2522,4],[73,'pyro',6022,3],[74,'laser',1340,3]])for(let direction=0;direction<8;direction++)for(let f=0;f<stride;f++){draws=[];art.unit(ctx,{...u,type,moving:false,nativePhase35:phase,nativeFrame32:f,aim:direction*Math.PI/4-Math.PI/2},100,{original:true,world:0});assert.equal(draws[0].frame,base+direction*stride+f,'55f10 raw point-fire state chooses the original bank/frame');}
 art.blasts(ctx,[{x:100,y:200,frame:3,sprite:395,strength:120}],{original:true,world:0});art.fires(ctx,[{x:100,y:200,sprite:5836}],{original:true,world:0});
 draws=[];transforms=[];art.nativeEffects(ctx,[{x:100,y:200,sprite:557,scale:.5,rotation:1}],{original:true,world:0});
 assert.deepEqual(transforms,[['translate',116,216],['rotate',1],['scale',.5,.5]],'466ac0 pivots about the16x16 padded quad for sprite557, not its visible16x10 pixels');
 assert.deepEqual([draws[0].x,draws[0].y,draws[0].w,draws[0].h],[-16,-16,32,20]);
 draws=[];const effect={x:100,y:200,sprite:557,scale:1,rotation:0},map={original:true,world:0};
 art.nativeEffects(ctx,[{...effect,team:1}],map);const blue=draws.at(-1).pixel;
 assert.deepEqual(blue,[...metadata.palettes[0].slice(32*3,32*3+3),255]);
 art.nativeEffects(ctx,[{...effect,team:null,nativeInheritedPalette:true}],map);assert.deepEqual(draws.at(-1).pixel,blue,'The no-palette-write branch keeps the preceding draw palette');
 art.nativeEffects(ctx,[{...effect,nativeWhitePalette:true}],map);assert.deepEqual(draws.at(-1).pixel,[255,255,255,255],'465cb0 signed negative palette produces opaque white for nonzero index');
 art.nativeEffects(ctx,[{...effect,nativeInheritedPalette:true}],map);assert.deepEqual(draws.at(-1).pixel,[255,255,255,255],'Quad/no-palette-write also inherits white');
 console.log(`PASS: ${contacts} heavy/grenade/flame weapon families through real Wasm; release/flight/impact/delayed damage,2/3-round salvos, persistent fire, guest colors, original sprite scaling and native walking frames.`);
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
