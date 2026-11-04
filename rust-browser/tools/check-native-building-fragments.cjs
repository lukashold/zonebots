// Retained Rust records -> actual guest RPC -> OriginalDisplay endpoint.
// Baseline Wasm supplies only the valid envelope, not the new simulation.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),clone=x=>JSON.parse(JSON.stringify(x));
const fromLog=(file,prefix,fallback)=>{if(!file)return fallback;const line=fs.readFileSync(file,'utf8').split(/\r?\n/).find(l=>l.startsWith(prefix));assert(line,`real Rust ${prefix} trace is required when a log is supplied`);return JSON.parse(line.slice(prefix.length));};
const fort=fromLog(process.argv[2],'FORT_QUEUE ',[
 {id:0,kind:'sprite',nativeSlot:0,palette:0,nativeLiteralPalette:true,queueY:236,rotation:0,scale:1,source:'fortFragment',sprite:120,stage:'queue',x:600,y:436},
 {id:50,kind:'sprite',nativeSlot:1,palette:0,nativeLiteralPalette:true,queueY:1236,rotation:0,scale:1,source:'fortFragment',sprite:120,stage:'queue',x:600,y:1436}]);
const ordinary=fromLog(process.argv[3],'BUILD_FRAGMENT_QUEUE ',[
 {id:150,kind:'sprite',nativeSlot:3,palette:0,nativeLiteralPalette:true,queueY:776,rotation:0,scale:1,source:'buildingFragment',sprite:1782,stage:'queue',x:820,y:976},
 {id:151,kind:'sprite',nativeSlot:3,palette:0,nativeLiteralPalette:true,queueY:796,rotation:0,scale:1,source:'buildingFragment',sprite:407,stage:'queue',x:840,y:996}]);
assert.equal(fort.length,2);assert.equal(ordinary.length,2);const rows=[...ordinary,...fort];
assert(rows.every(r=>r.palette===0&&r.nativeLiteralPalette===true&&r.y-r.queueY===200),'41f69f WORD0x6400 retains literal palette0 and offset100 separately');
const palettes=new WeakMap(),window={OriginalArt:{ready:true,nativePalette:c=>palettes.get(c)??0,selectNativePalette:(c,p)=>palettes.set(c,p)},GameGraphics:{isHD:()=>false}};
vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/original-display.js'),'utf8'),{window});
function draw(picture){const draws=[];assert(window.OriginalDisplay.draw({},picture,{sprite:f=>draws.push([f.id,f.sprite,f.x,f.y,f.team]),objectShadows:()=>draws.push(['shadows'])}));return draws;}
assert.deepEqual(draw({map:{original:true},units:[],buildings:[],nativeDisplayQueue:rows}),[['shadows'],...rows.map(r=>[r.id,r.sprite,r.x,r.y,0])]);
(async()=>{
 const{instance}=await WebAssembly.instantiate(fs.readFileSync(path.join(root,'dist/independent/engine.wasm')),{}),e=instance.exports,request=new TextEncoder().encode(JSON.stringify({op:'init',mission:0,seed:742,difficulty:1})),p=e.alloc(request.length);new Uint8Array(e.memory.buffer,p,request.length).set(request);let n;try{n=e.run(p,request.length);}finally{e.free(p,request.length);}const base=JSON.parse(new TextDecoder().decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!base.error,base.error);
 const w={};vm.runInNewContext(fs.readFileSync(path.join(root,'dist/independent/multiplayer-game.js'),'utf8'),{window:w,setTimeout,clearTimeout});const guest=w.MultiplayerGame.create({run(){assert.fail('guest may not simulate');}});guest.bind({connected:true,send:()=>true},'guest');
 const teamRow={...rows[0],source:'object',nativeLiteralPalette:undefined,id:42,nativeSlot:42,palette:0};base.nativeDisplayQueue=[...rows,teamRow];assert(guest.handle({type:'game:init',payload:{epoch:1,seq:0,meta:{name:'Literal fragments'},state:base}}));
 let seq=0,cases=1;for(let i=0;i<rows.length;i++)assert.deepEqual(clone(guest.viewState.nativeDisplayQueue[i]),rows[i]);assert.equal(guest.viewState.nativeDisplayQueue.at(-1).palette,1,'an ordinary team palette still swaps');assert(rows.every(r=>r.palette===0));
 const guestPicture={...guest.viewState,map:{original:true},units:[],buildings:[]};assert.deepEqual(draw(guestPicture),[['shadows'],...rows.map(r=>[r.id,r.sprite,r.x,r.y,0]),[teamRow.id,teamRow.sprite,teamRow.x,teamRow.y,1]],'actual guest records retain literal0 at the OriginalDisplay endpoint');
 const frame=queue=>({type:'game:frame',payload:{epoch:1,seq:++seq,state:{...base,nativeDisplayQueue:queue}}});
 for(const[source,last]of[['fortFragment',3],['buildingFragment',34]]){
  for(const slot of[0,last]){const row={...rows[0],source,nativeSlot:slot,id:slot*50+49};assert(guest.handle(frame([row])));assert.equal(guest.viewState.nativeDisplayQueue[0].palette,0);cases++;}
  const valid={...rows[0],source,nativeSlot:0,id:0};
  for(const patch of[{nativeSlot:last+1},{nativeSlot:-1},{nativeSlot:1},{id:(last+1)*50},{id:-1},{id:50},{kind:'effect'},{kind:'unit'},{stage:'pre'},{sprite:Infinity},{palette:1},{palette:256},{x:NaN},{nativeLiteralPalette:false},{nativeLiteralPalette:undefined},{nativeLiteralPalette:'true'},{source:'object'}]){assert.equal(guest.handle(frame([{...valid,...patch}])),false,`${source} ${JSON.stringify(patch)}`);cases++;}
 }
 guest.close();console.log(`Native BUILD/Fort fragments: ${rows.length} retained Rust rows, literal0 through OriginalDisplay and ${cases} actual guest RPC cases; team remap remains separate.`);
})().catch(e=>{console.error(e);process.exitCode=1;});
