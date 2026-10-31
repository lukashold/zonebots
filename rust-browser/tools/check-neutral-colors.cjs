'use strict';
// Synthetic indexed pixels exercise the real sprite decoder without embedding
// original artwork. Native palette4 uses80..95 for faction pixels16..31.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.join(__dirname,'..'),types=['medium','heavy','light','apc','jeep','crane','firetruck','launcher','cannon','missile','gatling','howitzer','truck'];
const neutralUnits={medium:5165,heavy:5168,light:5172,apc:5173,jeep:5214,crane:5834,firetruck:5215,launcher:5169,cannon:352,missile:1641,gatling:1657,howitzer:5765};
const metadata={frames:Array.from({length:7211},()=>({x:0,y:0,w:3,h:1})),neutralUnits,palettes:Array.from({length:5},(_,world)=>{
 const colors=Array(768).fill(0);for(const [index,rgb]of [[16,[164+world,0,0]],[32,[0,0,164+world]],[80,[100+world,100+world,100+world]],[100,[38+world,47+world,55+world]]])colors.splice(index*3,3,...rgb);return colors;
})};
function originalFixture(){
 function canvas(){const result={width:0,height:0};result.getContext=()=>({createImageData(w,h){return{data:new Uint8ClampedArray(w*h*4)};},putImageData(image){result.image=image;},drawImage(){},getImageData(){return{data:new Uint8ClampedArray([16,16,16,255,100,100,100,255,16,16,16,0])};}});return result;}
 class Image{constructor(){this.width=3;this.height=1;}set src(_){queueMicrotask(()=>this.onload());}}
 const sandbox={window:{},URL,Image,document:{currentScript:{src:'https://example.test/independent/original-art.js'},createElement:canvas},fetch:async()=>({ok:true,json:async()=>metadata})};
 vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/original-art.js'),'utf8'),sandbox);return sandbox.window.OriginalArt;
}
function hdFixture(){
 let colors=[];
 const context=new Proxy({createImageData(w,h){return{data:new Uint8ClampedArray(w*h*4)};},createLinearGradient(){return{addColorStop(_,color){colors.push(color);}};},createRadialGradient(){return{addColorStop(_,color){colors.push(color);}};}},{get:(target,key)=>target[key]||(()=>{}),set(target,key,value){if(typeof value==='string'&&/Style$/.test(key))colors.push(value);target[key]=value;return true;}});
 const sandbox={window:{},document:{createElement:()=>({width:0,height:0,getContext:()=>context})}};
 vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/hd-units.js'),'utf8'),sandbox);
 return{art:sandbox.window.HDUnits,draw(unit){colors=[];sandbox.window.HDUnits.unit(context,unit,0,{}, {x:0,y:0,w:64,h:64});return colors.slice();},context};
}
function unit(type,team){return{id:12,type,team,x:64,y:96,hp:100,maxHp:100,angle:0,aim:0,moving:false,cooldown:0,order:'guard',crew:team<0?null:{kind:'grunt',hp:30}};}
(async()=>{
 const original=originalFixture();assert(await original.load());let originals=0;
 for(let world=0;world<5;world++)for(const type of types){
  // Reuse the same entity through neutral→red→blue→neutral, as in crewing,
  // capture and unload. Cached colored sprites must not leak to neutral ones.
  const vehicle=unit(type,-1);
  for(const team of [-1,0,1,-1]){
   vehicle.team=team;vehicle.crew=team<0?null:{kind:'grunt',hp:30};const before=JSON.stringify(vehicle),draws=[];
   assert(original.unit({drawImage(image){draws.push(image);}},vehicle,0,{world,original:true}));assert(draws.length);
   const expected=team<0?[100+world,100+world,100+world,255]:team===0?[164+world,0,0,255]:[0,0,164+world,255];
   for(const image of draws){assert.deepEqual(Array.from(image.image.data.slice(0,4)),expected,`${type}/world${world}/team${team}: faction pixels`);assert.deepEqual(Array.from(image.image.data.slice(4,8)),[38+world,47+world,55+world,255],'non-faction metal must keep its original palette');assert.deepEqual(Array.from(image.image.data.slice(8,12)),[0,0,0,0],'transparent pixels stay transparent');}
   assert.equal(JSON.stringify(vehicle),before,'drawing must not change hardware ownership or type');originals++;
  }
 }
 // The installed palettes independently confirm that the native neutral ramp
 // really is grayscale in every world, rather than a guessed tint.
 const privateMetadata=path.join(root,'dist/original/sprites.json');if(fs.existsSync(privateMetadata))for(const pal of JSON.parse(fs.readFileSync(privateMetadata,'utf8')).palettes)for(let index=80;index<96;index++){assert.equal(pal[index*3],pal[index*3+1]);assert.equal(pal[index*3+1],pal[index*3+2]);}
 const hd=hdFixture();let hdCases=0;
 for(const type of types){
  for(const team of [-1,0,1,-1]){
   const vehicle=unit(type,team),before=JSON.stringify(vehicle),colors=hd.draw(vehicle);
   if(team<0){assert(colors.includes('#777777'),`${type}: neutral faction paint must be gray`);assert(!colors.includes('#a84338')&&!colors.includes('#46708b'),`${type}: no red/blue team paint before crew`);}
   else assert(colors.includes(team===0?'#a84338':'#46708b'),`${type}: crew ownership must select its team paint`);
   assert.equal(JSON.stringify(vehicle),before);hd.art.clearCache();hdCases++;
  }
 }
 // Exhaust the frame preparation budget after caching a red Jeep. A newly
 // neutral Jeep must use gray quick art, never that vehicle's old red raster.
 hd.draw(unit('jeep',0));hd.art.beginFrame(1);for(const type of ['medium','heavy','light','apc'])hd.draw(unit(type,0));const fallbackColors=hd.draw(unit('jeep',-1));assert(fallbackColors.includes('#777777'));assert(!fallbackColors.includes('#a84338'));assert(hd.art.stats.pending>0);
 console.log(`Neutral hardware colors passed: ${originals} original pixel/lifecycle cases, ${hdCases} HD team-paint cases, native gray ramps, untouched material/alpha/ownership and gray budget fallback.`);
})().catch(error=>{console.error(error);process.exitCode=1;});
