'use strict';
// Original frame IDs/strides independently recorded from fresh Ghidra evidence:
//0041ad30 maps weapon classes to states;00455f10 reads00496e78+4*state.
// No original image pixels or campaign geometry are embedded in this test.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const privateMetadata=JSON.parse(fs.readFileSync(path.join(root,'dist/original/sprites.json'),'utf8'));
const source=fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8');
const native={
 cannon:{state:4,base:330,stride:1,neutral:352,fireRate:60},
 gatling:{state:51,base:1659,stride:2,neutral:1657,fireRate:97},
 missile:{state:50,base:1643,stride:1,neutral:1641,fireRate:30},
 howitzer:{state:52,base:5767,stride:2,neutral:5765,fireRate:10},
};
function firingCases(fixture){
 const fresh=(100-fixture.fireRate)/17;
 // Explicit expected phases distinguish a shot acknowledgement from the whole
 // reload interval. Damaged rates must use their own interval, not the default.
 return [
  {name:'idle',cooldown:0,phase:0},
  {name:'fallback-fresh',cooldown:fresh,phase:1},
  {name:'fallback-mid-reload',cooldown:fresh/2,phase:0},
  {name:'explicit-fresh',cooldown:fresh,fireRate:fixture.fireRate,phase:1},
  {name:'explicit-mid-reload',cooldown:fresh/2,fireRate:fixture.fireRate,phase:0},
  {name:'fresh-rounding-epsilon',cooldown:fresh+5e-7,fireRate:fixture.fireRate,phase:1},
  {name:'reload-not-fresh',cooldown:fresh+.001,fireRate:fixture.fireRate,phase:0},
  {name:'damaged50-fresh',cooldown:50/17,fireRate:50,phase:1},
  {name:'damaged50-mid-reload',cooldown:25/17,fireRate:50,phase:0},
  {name:'damaged50-old-default',cooldown:fresh,fireRate:50,phase:0},
  {name:'damaged60-fresh',cooldown:40/17,fireRate:60,phase:1},
  {name:'damaged60-mid-reload',cooldown:20/17,fireRate:60,phase:0},
  {name:'damaged60-old-default',cooldown:fresh,fireRate:60,phase:0},
 ];
}
function renderer(legacy=false){
 const metadata=JSON.parse(JSON.stringify(privateMetadata));if(legacy)delete metadata.gunLayouts;
 let requested=null;const invalid=[];
 metadata.frames=new Proxy(metadata.frames,{get(target,key){if(typeof key==='string'&&/^\d+$/.test(key)){requested=Number(key);if(!target[key])invalid.push(requested);}return target[key];}});
 function canvas(){const c={width:0,height:0,frameId:requested};c.getContext=()=>({createImageData:(w,h)=>({data:new Uint8ClampedArray(w*h*4)}),putImageData(){},drawImage(){},getImageData:()=>({data:new Uint8ClampedArray(c.width*c.height*4)})});return c;}
 class FakeImage{set src(url){const sprite=String(url).endsWith('sprites.png');this.width=sprite?2048:512;this.height=sprite?Math.max(...metadata.frames.map(f=>f.y+f.h))+2:1280;Promise.resolve().then(()=>this.onload());}}
 const env={window:{},URL,Image:FakeImage,document:{currentScript:{src:'https://local.test/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
 vm.createContext(env);vm.runInContext(source,env);return {art:env.window.OriginalArt,metadata,invalid};
}
async function run(){
 let cases=0;
 for(const legacy of [false,true]){
  const {art,metadata,invalid}=renderer(legacy);assert(await art.load());
  for(const [kind,fixture] of Object.entries(native)){
   assert.equal(metadata.neutralUnits[kind],fixture.neutral,`${kind}: neutral original object descriptor`);
   if(!legacy){const layout=metadata.gunLayouts?.[kind];assert(layout,`${kind}: extracted gun layout missing`);assert.deepEqual([layout.state,layout.base,layout.stride],[fixture.state,fixture.base,fixture.stride],`${kind}: extracted native state descriptor`);}
   const family=new Set(Array.from({length:8*fixture.stride},(_,i)=>fixture.base+i));
   for(let world=0;world<5;world++)for(const team of [-1,0,1])for(let direction=0;direction<8;direction++)for(const shot of firingCases(fixture)){
    const u={type:kind,team,x:123.75,y:456.25,angle:0,aim:direction*Math.PI/4-Math.PI/2,cooldown:shot.cooldown};
    if(shot.fireRate!==undefined)u.originalCombat={fireRate:shot.fireRate};
    const expected=team<0?fixture.neutral:fixture.base+direction*fixture.stride+(fixture.stride===2?shot.phase:0),calls=[];
    const ctx={drawImage:(image,x,y,w,h)=>calls.push({id:image.frameId,x,y,w,h})},map={original:true,world};
    assert(art.unit(ctx,u,0,map));assert.equal(calls.length,1);
    const drawn=calls[0],frame=metadata.frames[expected],label=`${legacy?'fallback':'metadata'}/${kind}/world${world}/team${team}/dir${direction}/${shot.name}`;
    assert.equal(drawn.id,expected,label);
    assert.deepEqual([drawn.x,drawn.y,drawn.w,drawn.h],[124,456,frame.w*2,frame.h*2],`${label}: raw origin and actual frame dimensions`);
    for(const method of ['unitBounds','attackBounds']){
     const bounds=art[method](u,map,0);assert(bounds,`${label}: ${method}`);
     assert.deepEqual([bounds.x,bounds.y,bounds.w,bounds.h],[drawn.x,drawn.y,drawn.w,drawn.h],`${label}: ${method} follows rendered gun`);
    }
    if(team>=0)assert(family.has(drawn.id),`${label}: crewing/firing must preserve weapon family`);
    cases++;
   }
  }
  assert.deepEqual(invalid,[],'Every neutral, occupied and firing frame must exist in the private atlas');
 }
 console.log(`PASS ${cases} gun cases: four independent native families, all five worlds, neutral/red/blue ownership, eight directions, fresh-shot-only phases (default, explicit, damaged50/60 and reload/rounding cases), exact original frame dimensions and matching attack/selection rectangles, including packs without gunLayouts.`);
}
run().catch(error=>{console.error(error);process.exitCode=1;});
