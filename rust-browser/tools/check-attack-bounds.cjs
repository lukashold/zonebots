// Attack hit rectangles must match this renderer, including every composite part.
// The original sprite pack is read locally; no original artwork is embedded.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),metadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
function canvas(){const c={width:0,height:0};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
const kinds=['grunt','psycho','tough','sniper','pyro','laser','jeep','light','medium','heavy','apc','launcher','crane','truck','firetruck','gatling','cannon','howitzer','missile'];
const angle=i=>i*Math.PI/4-Math.PI/2;
function union(calls){const x=Math.min(...calls.map(p=>p.x)),y=Math.min(...calls.map(p=>p.y));return {x,y,w:Math.max(...calls.map(p=>p.x+p.w))-x,h:Math.max(...calls.map(p=>p.y+p.h))-y};}
(async()=>{
 const art=env.window.OriginalArt,map={original:true,world:0};
 const sample={type:'grunt',x:123.7,y:456.2,team:0,angle:0,aim:0,hp:100,maxHp:100,cooldown:0,moving:false,order:'guard'};
 assert.equal(art.attackBounds(sample,map),null);assert(await art.load());
 let calls=[],cases=0;const ctx={drawImage:(image,x,y,w,h)=>calls.push({x,y,w,h})};
 function check(unit,current,time){const before=JSON.stringify(unit);calls=[];assert(art.unit(ctx,unit,time,current),unit.type);assert(calls.length>0,unit.type);const bounds=art.attackBounds(unit,current,time);assert(bounds,unit.type);assert.deepEqual({...bounds},union(calls),`${unit.type}: attack bounds must cover all actual rendered layers`);assert.equal(JSON.stringify(unit),before);cases++;}
 const poses=[{moving:false,cooldown:0,hp:100,cranePhase:0,time:0},{moving:true,cooldown:0,hp:100,cranePhase:31,time:.21},{moving:false,cooldown:.8,hp:10,cranePhase:63,time:.39}];
 for(let world=0;world<5;world++){
  const current={original:true,world,width:64,terrainCodes:[]};
  for(const type of kinds)for(const team of [0,1])for(let heading=0;heading<8;heading++)for(let aim=0;aim<8;aim++)for(const pose of poses){
   check({...sample,...pose,type,team,angle:angle(heading),aim:angle(aim)},current,pose.time);
  }
  for(const type of Object.keys(metadata.neutralUnits))check({...sample,type,team:-1},current,.21);
  for(const type of ['grunt','medium','crane','jeep'])check({...sample,type,x:-20.3,y:-10.4,angle:angle(5),aim:angle(2)},current,.21);
 }
 const robot={...sample,x:200,y:300},box=art.attackBounds(robot,map),head={x:box.x+box.w/2,y:box.y+2};
 assert(Math.hypot(head.x-robot.x,head.y-robot.y)>22,'Regression must include a visible robot head outside the old feet radius');
 assert(head.x>=box.x&&head.x<=box.x+box.w&&head.y>=box.y&&head.y<=box.y+box.h);
 assert.equal(art.unitBounds(robot,map),null,'Existing robot selection-bounds contract must remain unchanged');
 assert.equal(art.unitBounds({...sample,type:'medium'},map),null,'Existing tank selection-bounds contract must remain unchanged');
 for(const invalid of [null,{...sample,x:NaN},{...sample,y:Infinity},{...sample,type:'unknown'},{...sample,type:'jeep',angle:NaN},{...sample,type:'heavy',aim:undefined}])assert.equal(art.attackBounds(invalid,map),null);
 check({...sample,type:'jeep',team:-1,angle:undefined,aim:undefined},map,0);
 assert.equal(art.attackBounds(sample,{original:false}),null);assert.equal(art.attackBounds(sample,undefined),null);
 console.log(JSON.stringify({worlds:5,types:kinds.length,cases}));
 console.log('PASS: full rendered attack bounds for every robot, vehicle and gun, all headings/aims, movement/fire/damage/crane phases, neutral hardware, fractional/negative anchors and unchanged selection bounds.');
})().catch(error=>{console.error(error);process.exitCode=1;});
