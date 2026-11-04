// Real Rust callback snapshots -> actual guest RPC -> OriginalDisplay.
// Usage: node tools/replay-native-walk-cached-rpc.cjs <Rust --nocapture log>
// Run Rust with NATIVE_WALK_CALLER_DIRECTORY and Z_WALK_RPC_TRACE=1.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
assert(process.argv[2],'supply the actual Rust callback trace; no synthetic state substitutes for the simulation');
const rows=fs.readFileSync(process.argv[2],'utf8').split(/\r?\n/).filter(l=>/^WALK_(MAIN_)?RPC /.test(l)).map(l=>({main:l.startsWith('WALK_MAIN_RPC '),state:JSON.parse(l.slice(l.indexOf(' ')+1))}));
assert.equal(rows.length,5,'four SP/MP before/after-cache callbacks plus one actual main loop');
const root=path.resolve(__dirname,'..'),palettes=new WeakMap(),window={OriginalArt:{ready:true,nativePalette:c=>palettes.get(c)??0,selectNativePalette:(c,p)=>palettes.set(c,p)},GameGraphics:{isHD:()=>false}};
vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),{window,setTimeout,clearTimeout});
vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/original-display.js'),'utf8'),{window});
const clone=x=>JSON.parse(JSON.stringify(x));let phase5=0,phase2=0;
for(const{main,state}of rows){const original=JSON.stringify(state),guest=window.MultiplayerGame.create({run(){assert.fail('guest must never run the simulation');}});guest.bind({connected:true,send:()=>true},'guest');assert(guest.handle({type:'game:init',payload:{epoch:1,seq:0,meta:{name:'Native cached removal'},state}}));const view=guest.viewState;assert.equal(view.units.some(u=>u.id===2),false,'the removed cached victim never returns in the guest state');assert.equal(view.nativeDisplayQueue.some(r=>r.kind==='unit'&&r.id===2),false,'no revived victim draw row');
 for(const u of state.units){const v=view.units.find(v=>v.id===u.id);assert.equal(v.team,u.team===0?1:0);assert.equal(v.nativePhase35,u.nativePhase35);assert.equal(v.nativeCell10,u.nativeCell10);assert.equal(v.nativeTarget46,u.nativeTarget46);assert.equal('nativeUpstreamReactions'in v,false);}
 const scanner=view.units.find(u=>u.id===(main?3:1));assert(scanner);assert.equal(scanner.nativePhase35,main?8:state.units[0].nativePhase35);if(!main){phase5+=scanner.nativePhase35===5;phase2+=scanner.nativePhase35===2;}
 assert(guest.handle({type:'game:frame',payload:{epoch:1,seq:1,state}}));assert.deepEqual(clone(guest.viewState.units),clone(view.units));const draws=[];assert(window.OriginalDisplay.draw({},guest.viewState,{unit:(u,sprite)=>draws.push({id:u.id,palette:u.nativeDrawPalette,sprite:sprite?.sprite}),sprite:f=>draws.push({sprite:f.sprite}),building:()=>{},objectShadows:()=>{}}));assert(draws.some(d=>d.id===scanner.id),'the real surviving scanner reaches OriginalDisplay');assert(draws.every(d=>d.id!==2));assert.equal(JSON.stringify(state),original,'perspective and rendering leave the host snapshot unchanged');guest.close();}
assert.equal(phase5,2);assert.equal(phase2,2);console.log('Native cached target: 5 real Rust snapshots, 10 guest RPC calls and 5 OriginalDisplay callbacks; no victim revival or guest simulation.');
