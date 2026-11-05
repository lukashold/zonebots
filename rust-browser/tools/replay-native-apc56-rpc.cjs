// Actual owned56 Rust Main snapshots -> guest RPC -> OriginalDisplay.
// No guest simulation, fabricated state or new protocol/Build constants.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
assert(process.argv[2],'actual Rust Main snapshot file is required');
const states=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));assert.equal(states.length,2);
const frontend=process.argv[3]?path.resolve(process.argv[3]):path.resolve(__dirname,'../dist/independent');
const palette=new WeakMap(),window={OriginalArt:{ready:true,nativePalette:c=>palette.get(c)??0,selectNativePalette:(c,p)=>palette.set(c,p)},GameGraphics:{isHD:()=>false}};
for(const file of['multiplayer-game.js','original-display.js'])vm.runInNewContext(fs.readFileSync(path.join(frontend,file),'utf8'),{window,setTimeout,clearTimeout});
let calls=0,draws=0;
for(const state of states){const original=JSON.stringify(state),actor=state.units[0],guest=window.MultiplayerGame.create({run(){assert.fail('guest cannot simulate');}});assert.equal(actor.nativePhase35,56);assert.equal('nativeHardwareCommon'in state,false);assert.equal('nativeCommonCallback'in actor,false);guest.bind({connected:true,send:()=>true},'guest');
 for(const type of['game:init','game:frame']){const envelope=type==='game:init'?{epoch:1,seq:0,meta:{name:'Original APC56 wrapper'},state}:{epoch:1,seq:1,state};assert(guest.handle({type,payload:envelope}),type);calls++;const view=guest.viewState,copy=view.units.find(u=>u.id===actor.id);for(const key of['nativePhase35','nativeFrame32','nativeCounter68'])assert.equal(copy[key],actor[key]);assert.equal(copy.team,1-actor.team);const rendered=[];assert(window.OriginalDisplay.draw({},view,{unit:(u,s)=>rendered.push({id:u.id,sprite:s?.sprite}),sprite:()=>{},building:()=>{},objectShadows:()=>{}}));assert(rendered.some(r=>r.id===actor.id),'actual APC56 Main row reaches draw');draws++;}
 assert.equal(JSON.stringify(state),original);guest.close();}
console.log(`APC56: ${states.length} actual SP/MP Main snapshots, ${calls} guest RPC and ${draws} OriginalDisplay callbacks; phase56 and normal team palette preserved.`);
