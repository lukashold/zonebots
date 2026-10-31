// Real Wasm ABI check. Campaign schema is read from the local private pack;
// all arena coordinates/terrain below are synthetic, with no embedded game data.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),options={
 campaign:path.join(root,'dist/original/campaign.json'),
 wasm:path.join(root,'dist/independent/engine.wasm'),
};
for(let i=2;i<process.argv.length;i+=2){
 const key=process.argv[i].replace(/^--/,'');
 assert(['campaign','wasm'].includes(key),`Unknown option ${process.argv[i]}`);
 assert(process.argv[i+1],`Missing value for ${process.argv[i]}`);
 options[key]=path.resolve(process.argv[i+1]);
}
assert(fs.existsSync(options.campaign),'Missing private campaign: extract your own local game');
const campaign=JSON.parse(fs.readFileSync(options.campaign,'utf8'));
assert(campaign.missions?.length,'No private campaign mission schema available');
const clone=value=>JSON.parse(JSON.stringify(value));

function fixture({target='apc',x=800,team=1,pickup=true,wall=false}={}){
 const mission=clone(campaign.missions[0]),size=32,count=size*size;
 mission.number=1;mission.name='Synthetic original grenade contact';
 // The synthetic arena replaces every campaign layer, including v2 walls.
 mission.walls=[];for(const key of ['wallCells','wallGroundTiles','wallGroundCodes','wallGroundClasses','wallTileLookup'])delete mission.map[key];
 Object.assign(mission.map,{width:size,height:size,tileSize:32,world:0,seed:1,revision:0,
  original:true,tiles:Array(count).fill(0),territory:Array(count).fill(0),
  bridgeBase:Array(count).fill(-1),bridgeHp:[],originalTiles:Array(count).fill(0),terrainCodes:Array(count).fill(0)});
 // A directed attack may now step around a single tile to acquire sight.
 // Use a sealed wall to isolate ammunition preservation while sight stays blocked.
 if(wall)for(let y=0;y<size;y++){mission.map.tiles[y*size+11]=2;mission.map.terrainCodes[y*size+11]=19;}
 mission.sectors=[{id:0,x:64,y:64,owner:0,home:true,countsForProduction:true}];
 mission.buildings=[];mission.originalObjects=[];
 mission.spawns=[{type:'grunt',team:0,x:320,y:320,squad:1},
  {type:target,team,x,y:320,squad:2}];
 if(team!==1)mission.spawns.push({type:'apc',team:1,x:900,y:900,squad:3});
 mission.crates=pickup?[{x:320,y:320,taken:false}]:[];
 return mission;
}

(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(options.wasm),{});
 const e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function request(value){
  const bytes=enc.encode(JSON.stringify(value)),ptr=e.alloc(bytes.length);
  new Uint8Array(e.memory.buffer,ptr,bytes.length).set(bytes);
  let length;try{length=e.run(ptr,bytes.length);}finally{e.free(ptr,bytes.length);}
  const state=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),length)));
  assert(!state.error,state.error);return state;
 }
 const init=mission=>request({op:'init',mission:0,difficulty:1,seed:411,aiTeams:[],originalMission:mission});
 const tick=(dt=0.05)=>request({op:'tick',dt});
 const unit=(state,id)=>{const found=state.units.find(u=>u.id===id);assert(found,`Unit ${id} vanished`);return found;};
 const playerShots=state=>state.events.filter(e=>e.kind==='shot'&&e.team===0);
 function collected(mission){
  let state=init(mission);assert.equal(state.map.original,true);
  const shooter=state.units[0].id,target=state.units[1].id;
  assert.equal(unit(state,shooter).grenades,0);
  state=tick(0);
  assert.equal(unit(state,shooter).grenades,8);
  assert.equal(state.crates[0].taken,true);
  assert.equal(state.events.filter(e=>e.kind==='grenades'&&e.team===0).length,1);
  state=tick(0);
  assert.equal(unit(state,shooter).grenades,8,'Taken crate must not grant ammunition twice');
  assert.equal(state.events.filter(e=>e.kind==='grenades').length,0);
  return {state,shooter,target};
 }
 function firstAttack(shooter,target){
  request({op:'command',ids:[shooter],kind:'attack',target});
  for(let i=0;i<400;i++){
   const state=tick();
   if(playerShots(state).length)return state;
   assert.equal(state.winner,null,'Fixture must remain alive until the attack');
  }
  assert.fail('The real Rust route never reached firing range');
 }

 let {state,shooter,target}=collected(fixture());
 assert.equal(playerShots(state).length,0,'Out-of-range target must preserve inventory');
 state=firstAttack(shooter,target);
 assert.equal(playerShots(state).length,1);
 assert.equal(playerShots(state)[0].weapon,'grenade','Serialized event must distinguish grenade audio');
 assert.equal(playerShots(state)[0].type,'grunt');
 assert.equal(unit(state,shooter).grenades,7,'Exactly one grenade per committed contact');
 assert.equal(unit(state,target).hp,150,'APC hull200 receives verified body contact50');
 assert.equal(unit(state,target).crew.hp,30,'Grenade bypasses bullet crew rolls');
 state=tick();
 assert.equal(unit(state,shooter).grenades,7);
 assert.equal(unit(state,target).hp,150);
 assert.equal(playerShots(state).length,0,'Cooldown cannot launch another grenade');

 state=init(fixture({pickup:false,x:420}));shooter=state.units[0].id;target=state.units[1].id;
 state=tick();
 assert.equal(unit(state,shooter).grenades,0);
 assert.equal(unit(state,target).hp,200,'Unarmed Grunt bullet is reduced to zero by APC armor');
 assert.equal(playerShots(state)[0].weapon,'primary');

 ({state,shooter,target}=collected(fixture({target:'gatling'})));
 state=firstAttack(shooter,target);
 assert.equal(playerShots(state)[0].weapon,'primary','Gatling class5 does not trigger the class1/2 grenade branch');
 assert.equal(unit(state,shooter).grenades,8);

 for(const config of [{x:420,wall:true},{x:420,team:0}]){
  ({state,shooter,target}=collected(fixture(config)));
  state=request({op:'command',ids:[shooter],kind:'attack',target});
  for(let i=0;i<2;i++)state=tick();
  assert.equal(unit(state,shooter).grenades,8,'Blocked/friendly target must preserve ammunition');
  assert.equal(playerShots(state).length,0);
 }
 console.log('Original grenades passed with real Wasm: one crate pickup (+8), out-of-range preservation, eligible contact50 and one grenade event, cooldown, empty inventory, class5 primary fallback, wall and friendly-target preservation.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
