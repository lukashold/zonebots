// Real Wasm flight/impact checks. Arenas and expected numeric rules are synthetic.
'use strict';
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
function arena(type){const width=64,height=48,n=width*height;return{map:{width,height,tileSize:32,world:0,seed:7291,revision:0,original:true,tiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),territory:Array(n).fill(0),originalTiles:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[]},sectors:[],buildings:[],walls:[],originalObjects:[],crates:type==='grunt'?[{x:320,y:600,taken:false}]:[],spawns:[{type,team:0,x:320,y:600,squad:1},{type:'apc',team:1,x:520,y:600,squad:2}]};}
(async()=>{
 const e=(await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{})).instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function req(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 const env={window:{},setTimeout,clearTimeout,Date};vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),env);let contacts=0;
 for(const type of ['grunt','tough','pyro','light','medium','heavy','cannon','missile','launcher','howitzer']){
  let s=req({op:'init',mission:0,difficulty:1,seed:7291,aiTeams:[],originalMission:arena(type)});const id=s.units.find(u=>u.team===1).id;
  for(let tick=1;tick<=100;tick++){s=req({op:'tick',dt:.059});if(s.projectiles.length)break;}
  assert(s.projectiles.length,`${type}: first traveling projectile must exist`);assert.equal(s.units.find(u=>u.id===id).hp,200,`${type}: release is not contact`);
  assert.equal(s.projectiles.length,type==='missile'?2:type==='launcher'?3:1,`${type}: salvo size`);
  for(const p of s.projectiles){assert([p.x,p.y,p.angle,p.size].every(Number.isFinite));assert(p.id<50);assert(p.sprite>=0&&p.sprite<7211);}
  const guest=env.window.MultiplayerGame.perspective(s,'guest');assert(guest.projectiles.every(p=>p.team===1));assert(s.projectiles.every(p=>p.team===0));
  let seenImpact=false,seenDelayed=false,seenFire=false;
  for(let tick=1;tick<=160;tick++){
   s=req({op:'tick',dt:.059});seenFire ||=s.fires.length>0;
   if(s.blasts.some(b=>b.frame===0)){seenImpact=true;seenDelayed ||=s.units.find(u=>u.id===id)?.hp===200;}
   const hp=s.units.find(u=>u.id===id)?.hp??-1;if(hp<200){contacts++;break;}
  }
  assert((s.units.find(u=>u.id===id)?.hp??-1)<200,`${type}: stationary target must eventually receive physical damage`);
  if(type==='pyro')assert(seenFire,'Flamethrower leaves a persistent native fire object');
  else{assert(seenImpact,`${type}: impact must create its independent blast`);assert(seenDelayed,`${type}: frame0 blast must precede hull damage`);}
 }
 // The renderer uses real frame dimensions with synthetic opaque atlas texels.
 const metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));let requested=null,draws=[],transforms=[];
 metadata.frames=new Proxy(metadata.frames,{get(a,k){if(/^\d+$/.test(String(k))){requested=Number(k);assert(a[k],`Missing native sprite${k}`);}return a[k];}});
 const pixels=new Proxy({},{get:(_,k)=>Number(k)%4===3?255:16});
 function canvas(){const c={width:0,height:0,frame:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:pixels})});return c;}
 class Image{set src(v){this.width=String(v).endsWith('sprites.png')?2048:512;this.height=4526;queueMicrotask(()=>this.onload());}}
 const render={window:{},URL,Image,document:{currentScript:{src:'https://local.test/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
 vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),render);const art=render.window.OriginalArt;assert(await art.load());
 const ctx={save(){},restore(){},rotate(a){assert(Number.isFinite(a));transforms.push(['rotate',a]);},translate(x,y){assert([x,y].every(Number.isFinite));transforms.push(['translate',x,y]);},drawImage(im,x,y,w,h){assert([x,y,w,h].every(Number.isFinite));draws.push({frame:im.frame,x,y,w,h});}};
 art.projectiles(ctx,[{x:100,y:200,team:0,kind:'shell',sprite:1634,angle:1,size:8}],{original:true,world:0});
 assert.equal(draws[0].frame,1634);assert.equal(draws[0].w,metadata.frames[1634].w*3,'Native size8 grows the width by1.5');
 const u={type:'grunt',x:100,y:200,team:0,angle:0,aim:0,moving:true,order:'move',cooldown:0,nativeWalk:{frame:3},originalCombat:{}};
 draws=[];art.unit(ctx,u,100,{original:true,world:0});assert.equal(draws[0].frame,85,'Walk frame comes from Rust, independently of wall time');
 art.blasts(ctx,[{x:100,y:200,frame:3,sprite:395,strength:120}],{original:true,world:0});art.fires(ctx,[{x:100,y:200,sprite:5836}],{original:true,world:0});
 console.log(`PASS: ${contacts} heavy/grenade/flame weapon families through real Wasm; release/flight/impact/delayed damage,2/3-round salvos, persistent fire, guest colors, original sprite scaling and native walking frames.`);
})().catch(e=>{console.error(e.stack||e);process.exitCode=1;});
