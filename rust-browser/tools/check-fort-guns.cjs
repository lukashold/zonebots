// Regression checks against locally extracted private resources; no original map geometry is embedded.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
const campaign=JSON.parse(fs.readFileSync(path.join(root,'dist/original/campaign.json'),'utf8'));
let requested=null;const invalid=[];
metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
const kinds=['gatling','cannon','howitzer','missile'];
(async()=>{
 const art=env.window.OriginalArt;assert(await art.load());let calls=[];
 const ctx={drawImage:(im,x,y,w,h)=>calls.push({id:im.frameId,x,y,w,h})};let drawCases=0;
 for(let world=0;world<5;world++)for(const type of kinds)for(const team of [-1,0,1,2,3])for(let facing=0;facing<8;facing++)for(const cooldown of [0,1]){
  const u={type,team,x:123,y:456,angle:0,aim:facing*Math.PI/4-Math.PI/2,cooldown};const map={original:true,world};calls=[];
  assert(art.unit(ctx,u,0,map));assert.equal(calls.length,1,`${type} must use one ordinary gun sprite`);
  const draw=calls[0],frame=metadata.frames[draw.id];
  assert.equal(draw.x,u.x,`${type}: original raw X is top-left, never subtract frame width`);
  assert.equal(draw.y,u.y,`${type}: original raw Y is top-left, never subtract frame height`);
  assert.equal(draw.w,frame.w*2);assert.equal(draw.h,frame.h*2);
  const box=art.unitBounds(u,map);assert(box);
  assert.deepEqual([box.x,box.y,box.w,box.h],[draw.x,draw.y,draw.w,draw.h],`${type}: hit/selection bounds must match its rendered rectangle`);
  if(team===-1){const neutral={x:draw.x,y:draw.y};calls=[];art.unit(ctx,{...u,team:0},0,map);assert.deepEqual([calls[0].x,calls[0].y],[neutral.x,neutral.y],`${type}: crewing must not move the sprite origin`);}
  drawCases++;
 }
 assert.equal(art.unitBounds({type:'grunt'}, {original:true}),null);
 assert.equal(art.unitBounds({type:'gatling'}, {original:false}),null);
 // Association is structural, so killing/replacing a driver cannot detach a roof gun.
 const fort={id:11,kind:'fort',team:0};
 const sample=[{id:1,type:'gatling',team:1,fortId:11},{id:2,type:'cannon',team:-1,fortId:11},{id:3,type:'howitzer',team:0,fortId:12},{id:4,type:'missile',team:0,fortId:null},{id:5,type:'grunt',team:0,fortId:11}];
 assert.deepEqual(Array.from(art.fortGuns(fort,sample),u=>u.id),[1,2]);
 assert.equal(art.fortGuns({...fort,kind:'robot'},sample).length,0);
 let mounted=0,ground=0;
 assert.equal(campaign.missions.length,20);
 for(const mission of campaign.missions){
  const forts=mission.buildings.filter(b=>b.kind==='fort'),ids=new Set(forts.map(b=>b.id));
  const guns=mission.spawns.filter(u=>kinds.includes(u.type));
  for(const gun of guns){
   if(gun.fortId===undefined||gun.fortId===null){ground++;continue;}
   mounted++;assert(ids.has(gun.fortId),`Mission ${mission.number}: dangling fortId`);
   assert.equal(gun.source,'levels.dat_group','Only original fixed starting guns carry fort association');
   const owner=forts.find(b=>b.id===gun.fortId);assert.equal(owner.team,gun.team);
   assert(art.fortGuns(owner,guns).includes(gun));
   assert(art.fortGuns({...owner,team:1-owner.team},guns).includes(gun),'Owner changes cannot remove a mounted gun from its fort layer');
  }
  // Exercise actual fort composition followed by every associated gun, including dead-driver ownership.
  for(const b of forts){calls=[];assert(art.building(ctx,b,0,mission.map));const fortDraws=calls.length;assert.equal(fortDraws,6);
   const roof=art.fortGuns(b,guns);for(const u of roof)assert(art.unit(ctx,{...u,angle:0,aim:0,cooldown:0,team:-1},0,mission.map));
   assert.equal(calls.length,fortDraws+roof.length);
   roof.forEach((u,i)=>assert.deepEqual([calls[fortDraws+i].x,calls[fortDraws+i].y],[u.x,u.y]));
  }
 }
 assert.equal(mounted,68);assert.equal(ground,95);assert.deepEqual(invalid,[]);
 // Actual app Z-order wiring is covered by check-fort-layer.cjs.
 console.log(JSON.stringify({drawCases,missions:20,mounted,ground}));
 console.log('PASS: raw gun origins, all types/aims/teams/worlds, matching bounds, no crewing jump, fortId association and complete campaign mounted/ground classification.');
})().catch(error=>{console.error(error);process.exitCode=1;});
