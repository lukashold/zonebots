// Real Rust Main snapshots -> guest RPC -> OriginalDisplay. No guest simulation.
// Run the common Rust tests with NATIVE_COMMON_RPC_OUTPUT, then supply that file.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
assert(process.argv[2],'actual Rust Main snapshot file is required');
const states=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));assert.equal(states.length,7);
const root=path.resolve(__dirname,'..'),palette=new WeakMap(),window={OriginalArt:{ready:true,nativePalette:c=>palette.get(c)??0,selectNativePalette:(c,p)=>palette.set(c,p)},GameGraphics:{isHD:()=>false}};
for(const file of['multiplayer-game.js','original-display.js'])vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent',file),'utf8'),{window,setTimeout,clearTimeout});
const phases=[9,10,53,54,55,58,60];let calls=0,draws=0;
for(const[stateIndex,state]of states.entries()){const original=JSON.stringify(state),actor=state.units[0],guest=window.MultiplayerGame.create({run(){assert.fail('guest cannot simulate');}});assert.equal(actor.nativePhase35,phases[stateIndex]);assert.equal('nativeHardwareCommon'in state,false);assert.equal('nativeCommonCallback'in actor,false);guest.bind({connected:true,send:()=>true},'guest');
 for(const type of['game:init','game:frame']){const envelope=type==='game:init'?{epoch:1,seq:0,meta:{name:'Original hardware common'},state}:{epoch:1,seq:1,state};assert(guest.handle({type,payload:envelope}),`phase${actor.nativePhase35}/${type}`);calls++;const view=guest.viewState,copy=view.units.find(u=>u.id===actor.id);assert.equal(copy.nativePhase35,actor.nativePhase35);assert.equal(copy.nativeFrame32,actor.nativeFrame32);assert.equal(copy.nativeCounter68,actor.nativeCounter68);assert.equal(copy.team,1-actor.team);const rendered=[];assert(window.OriginalDisplay.draw({},view,{unit:(u,s)=>rendered.push({id:u.id,sprite:s?.sprite}),sprite:()=>{},building:()=>{},objectShadows:()=>{}}));assert(rendered.some(r=>r.id===actor.id),`phase${actor.nativePhase35} actual Main row reaches draw`);draws++;}
 assert.equal(JSON.stringify(state),original);guest.close();}
console.log(`Native common: ${states.length} actual Main snapshots, ${calls} guest RPC and ${draws} OriginalDisplay callbacks; phases ${phases.join('/')}.`);
