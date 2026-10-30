// Original Jeep compositing fixtures independently recovered from00455440.
// Requires the locally extracted private sprite metadata; no artwork is embedded.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),file=path.join(root,'dist/original/sprites.json');
assert(fs.existsSync(file),'Extract the local original artwork first.');
const data=JSON.parse(fs.readFileSync(file,'utf8'));
let requested=null;const invalid=[];
data.frames=new Proxy(data.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
class FakeImage{set src(url){this.width=String(url).endsWith('sprites.png')?2048:512;this.height=String(url).endsWith('sprites.png')?4526:1280;Promise.resolve().then(()=>this.onload());}}
const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'http://local/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>data})};
vm.createContext(env);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),env);
// These values come from the executable's tables, not the renderer's metadata.
const underlay=[0,5614,5618,5622,0,5626,5630,5634];
const bodyOffsets=[[2,8],[-5,11],[-10,6],[-6,0],[0,0],[12,0],[14,5],[10,11]];
const aimOffsets=[[0,0],[3,0],[5,-1],[2,4],[-1,6],[-3,4],[-5,-1],[-3,0]];
const angle=i=>i*Math.PI/4-Math.PI/2;
const plain=v=>JSON.parse(JSON.stringify(v));
function union(calls){const x=Math.min(...calls.map(c=>c.x)),y=Math.min(...calls.map(c=>c.y));return {x,y,w:Math.max(...calls.map(c=>c.x+c.w))-x,h:Math.max(...calls.map(c=>c.y+c.h))-y};}
(async()=>{
 const art=env.window.OriginalArt;assert(await art.load());let calls=[];
 const ctx={drawImage:(im,x,y,w,h)=>calls.push({id:im.frameId,x,y,w,h})};
 let count=0;
 for(let world=0;world<5;world++)for(const team of [0,1])for(let d=0;d<8;d++)for(let aim=0;aim<8;aim++)for(const firing of [false,true]){
  const u={type:'jeep',x:100,y:200,hp:50,maxHp:50,team,moving:false,angle:angle(d),aim:angle(aim),cooldown:firing?1:0,order:'guard',crew:{kind:'grunt',hp:30}},map={original:true,world};
  calls=[];assert(art.unit(ctx,u,0,map));
  const expected=[];
  if(d!==0&&d!==4)expected.push([underlay[d],88,188]);
  expected.push([5590+d*2,88,188]);
  expected.push([5639+aim*2+(firing?1:0),100+(bodyOffsets[d][0]+aimOffsets[aim][0])*2,182+(bodyOffsets[d][1]+aimOffsets[aim][1])*2]);
  assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),expected,`world${world}/team${team}/direction${d}/aim${aim}/firing${firing}`);
  assert.deepEqual(plain(art.unitBounds(u,map)),union(calls),'Click/portrait bounds must contain every drawn layer');
  count++;
 }
 // Exercise the documented browser phase adaptation separately from the
 // recovered static layout. Each wheel phase must stay within its own block,
 // and the weapon follows the body's one-pixel vertical phase.
 let phaseCases=0;
 for(const terrain of [0,7,23,51])for(const time of [0,.06,.12,.18,.24])for(const moving of [false,true])for(let d=0;d<8;d++)for(let aim=0;aim<8;aim++){
  const codes=Array(256).fill(0);codes[6*16+3]=terrain;
  const map={original:true,world:0,width:16,height:16,terrainCodes:codes},u={type:'jeep',x:100,y:200,hp:50,maxHp:50,team:0,moving,angle:angle(d),aim:angle(aim),cooldown:0,order:'guard',crew:{kind:'grunt',hp:30}};
  const tick=Math.floor(time*17),wheel=moving?tick%4:0,body=moving?((terrain===0?wheel:wheel>>1)&1):((tick>>1)&1);
  const expected=[];if(d!==0&&d!==4)expected.push([underlay[d]+wheel,88,188]);
  expected.push([5590+d*2+body,88,188]);
  expected.push([5639+aim*2,100+(bodyOffsets[d][0]+aimOffsets[aim][0])*2,182+(bodyOffsets[d][1]+aimOffsets[aim][1]-body)*2]);
  calls=[];art.unit(ctx,u,time,map);
  assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),expected);
  assert.deepEqual(plain(art.unitBounds(u,map,time)),union(calls));
  assert.deepEqual(plain(art.jeepLayers(u,time,map)),calls.map(({id,x,y})=>({id,x,y})));
  phaseCases++;
 }
 for(let world=0;world<5;world++){
  const u={type:'jeep',x:100,y:200,hp:50,maxHp:50,team:-1,moving:false,angle:0,aim:0,cooldown:0},map={original:true,world};
  calls=[];assert(art.unit(ctx,u,0,map));assert.deepEqual(calls.map(c=>[c.id,c.x,c.y]),[[5214,100,200]]);
  assert.deepEqual(plain(art.unitBounds(u,map)),union(calls));
 }
 assert.deepEqual(invalid,[]);console.log(JSON.stringify({occupiedCases:count,phaseCases,neutralCases:5}));
 console.log('PASS: original Jeep underlay/body/weapon layers, independent heading and aim, firing frames, native origins, neutral artwork and full visual bounds.');
})().catch(error=>{console.error(error);process.exitCode=1;});
