// Exercise pursuit through the real Rust/Wasm ABI on synthetic, asset-free maps.
// Marker checks load the same frontend tracker used by the game.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');let wasm=path.join(root,'dist/independent/engine.wasm');
if(process.argv[2]){assert.equal(process.argv[2],'--wasm');assert(process.argv[3]);wasm=path.resolve(process.argv[3]);}
const trackerEnv={window:{}};vm.createContext(trackerEnv);vm.runInContext(fs.readFileSync(path.join(root,'dist/independent/attack-command.js'),'utf8'),trackerEnv);
(async()=>{
 const {instance}=await WebAssembly.instantiate(fs.readFileSync(wasm),{}),e=instance.exports,enc=new TextEncoder(),dec=new TextDecoder();
 function request(v){const b=enc.encode(JSON.stringify(v)),p=e.alloc(b.length);new Uint8Array(e.memory.buffer,p,b.length).set(b);let n;try{n=e.run(p,b.length);}finally{e.free(p,b.length);}const s=JSON.parse(dec.decode(new Uint8Array(e.memory.buffer,e.output(),n)));assert(!s.error,s.error);return s;}
 function fixture({type='grunt',target='crane',team=0,distance=640}={}){
  const width=64,height=40,n=width*height;
  return {map:{width,height,tileSize:32,world:0,tiles:Array(n).fill(0),territory:Array(n).fill(0),bridgeBase:Array(n).fill(-1),bridgeHp:[],seed:117,revision:0,original:true,originalTiles:Array(n).fill(0),terrainCodes:Array(n).fill(0),waterTile:0},sectors:[{id:0,x:1936,y:1168,owner:1-team,home:true,countsForProduction:true}],buildings:[],crates:[],originalObjects:[],spawns:[{type,team,x:144,y:336,squad:1},{type:target,team:1-team,x:144+distance,y:336,squad:2},{type:'gatling',team:1-team,x:1936,y:1168,squad:3}]};
 }
 const init=(m,aiTeams=[])=>request({op:'init',mission:0,difficulty:1,seed:117,aiTeams,originalMission:m});
 const unit=(s,id)=>{const u=s.units.find(u=>u.id===id);assert(u,`Unit${id} disappeared`);return u;};
 const command=(s,id,kind,extra={})=>request({op:'command',team:unit(s,id).team,ids:[id],kind,...extra});
 const tick=(steps=1)=>{let s;for(let i=0;i<steps;i++)s=request({op:'tick',dt:0.1});return s;};
 const point=u=>[u.x,u.y],tail=u=>u.path.at(-1),distance=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
 function sight(s,a,b){const n=Math.floor(distance(a,b)/12);for(let i=1;i<n;i++){const t=i/n,j=Math.floor((a[1]+(b[1]-a[1])*t)/32)*s.map.width+Math.floor((a[0]+(b[0]-a[0])*t)/32);if(s.map.tiles[j]===2)return false;}return true;}
 function firingGoal(s,id,targetId){const u=unit(s,id),v=unit(s,targetId),end=tail(u),range=s.catalog.find(d=>d.type===u.type).range;assert(end,'Missing firing route');assert(distance(end,point(v))<=range+16,'Pursuit endpoint should be a current firing position');assert(sight(s,end,point(v)),'Pursuit endpoint must have target line of sight');}
 let s=init(fixture()),attacker=s.units[0].id,target=s.units[1].id;
 const originalTarget=point(unit(s,target));s=command(s,attacker,'attack',{target,x:originalTarget[0],y:originalTarget[1]});
 assert(Math.hypot(unit(s,attacker).path.at(-1)[0]-unit(s,attacker).x,unit(s,attacker).path.at(-1)[1]-unit(s,attacker).y)>96,'Fixture must retain substantial untraveled route distance while target moves');
 const tracker=trackerEnv.window.AttackCommand.createTracker();tracker.assign({kind:'attack',targetType:'unit',target,ids:[attacker],x:originalTarget[0],y:originalTarget[1]},s);
 assert.equal(tracker.targets(s,new Set([attacker])).length,1);
 s=command(s,target,'move',{x:originalTarget[0],y:1008});s=tick(60);
 assert(distance(point(unit(s,target)),originalTarget)>160,'Fixture target must move significantly');
 assert(unit(s,attacker).path.length>0,'Pursuer should still be en route');
 assert(unit(s,attacker).path.some(([,y])=>y>originalTarget[1]+16),'Old route must bend toward a displaced enemy before its old endpoint is reached');
 firingGoal(s,attacker,target);
 assert(distance(unit(s,attacker).dest,point(unit(s,target)))<8,'Directed destination should track the current target, allowing one snapshot frame');
 assert.equal(tracker.targets(s,new Set([attacker])).length,1,'Moving-target marker must survive engine replanning');
 // Stop the enemy: the pursuer should reach valid firing range and stop advancing.
 s=command(s,target,'stop');let fired=false;
 for(let i=0;i<300;i++){s=tick();if(s.events.some(e=>e.kind==='shot'&&e.team===0)){fired=true;break;}}
 assert(fired,'Pursuer should eventually fire at the stopped target');
 const inRange=point(unit(s,attacker));s=tick(2);assert(distance(point(unit(s,attacker)),inRange)<1,'Pursuer should not drive through an enemy once it can shoot');
 // Moving away again must restart pursuit after the route was cleared in range.
 const stoppedTarget=point(unit(s,target));s=command(s,target,'move',{x:stoppedTarget[0]+400,y:stoppedTarget[1]});let resumed=false;
 for(let i=0;i<25;i++){s=tick();resumed ||= unit(s,attacker).path.length>0||unit(s,attacker).moving;}
 assert(resumed,'Target leaving range must restart pursuit');
 assert(distance(point(unit(s,attacker)),inRange)>4,'Pursuer should resume movement');
 // Explicit Stop and a new move order must cancel all pursuit state.
 s=command(s,attacker,'stop');const stopped=point(unit(s,attacker));s=tick(15);
 assert(distance(point(unit(s,attacker)),stopped)<0.001);assert.equal(unit(s,attacker).target,null);assert.equal(tracker.targets(s,new Set([attacker])).length,0);
 s=command(s,attacker,'move',{x:stopped[0],y:stopped[1]-160});s=tick(10);
 assert.equal(unit(s,attacker).order,'move');assert.equal(unit(s,attacker).target,null);assert(unit(s,attacker).y<stopped[1]);
 // Vehicles can choose a reachable firing cell on their own river bank;
 // they do not need a route onto the target's inaccessible land component.
 const bank=fixture({type:'heavy',distance:384});
 for(let y=0;y<bank.map.height;y++){const j=y*bank.map.width+10;bank.map.terrainCodes[j]=9;bank.map.tiles[j]=3;}
 s=init(bank);attacker=s.units[0].id;target=s.units[1].id;s=command(s,target,'stop');s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});s=tick();
 assert(unit(s,attacker).path.length>0);assert(unit(s,attacker).path.every(([x])=>x<320),'Firing route must stay on own river bank');firingGoal(s,attacker,target);
 let bankShot=false;for(let i=0;i<120;i++){s=tick();assert(unit(s,attacker).x<320);if(s.events.some(e=>e.kind==='shot'&&e.team===0)){bankShot=true;break;}}assert(bankShot,'Same-bank firing position should eventually yield a shot');
 // Mere distance is insufficient when a nearby target is behind a rock wall.
 const wall=fixture({type:'heavy',distance:160});for(let y=0;y<20;y++){const j=y*wall.map.width+7;wall.map.terrainCodes[j]=19;wall.map.tiles[j]=2;}
 s=init(wall);attacker=s.units[0].id;target=s.units[1].id;s=command(s,target,'stop');s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});s=tick();
 assert(!s.events.some(e=>e.kind==='shot'&&e.team===0),'Nearby blocked target must not be shot through wall');assert(unit(s,attacker).path.some(([,y])=>y>=640),'Blocked sight line requires routing around the wall');firingGoal(s,attacker,target);
 // A crew leaving the chosen vehicle changes it to neutral and ends the attack.
 s=init(fixture());attacker=s.units[0].id;target=s.units[1].id;s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});
 s=command(s,target,'unload');assert.equal(unit(s,target).team,-1);s=tick();
 assert.equal(unit(s,attacker).target,null,'Neutralized target must not remain a combat target');assert.equal(unit(s,attacker).path.length,0,'Neutral target must not leave a stale chase route');
 // Target loss on actual combat death must clear the old route on the next tick.
 s=init(fixture({type:'heavy',target:'grunt',distance:448}));attacker=s.units[0].id;target=s.units[1].id;s=command(s,target,'stop');s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});
 for(let i=0;i<600&&s.units.some(u=>u.id===target);i++)s=tick();assert(!s.units.some(u=>u.id===target),'Fixture target must die in actual combat');s=tick();
 assert.equal(unit(s,attacker).target,null);assert.equal(unit(s,attacker).path.length,0,'Dead target must clear stale pursuit');
 // A stationary cannon can fire, but never follow a moving target physically.
 s=init(fixture({type:'cannon'}));attacker=s.units[0].id;target=s.units[1].id;const gun=point(unit(s,attacker));s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});s=command(s,target,'move',{x:1008,y:1008});s=tick(30);assert(distance(point(unit(s,attacker)),gun)<0.001);
 // AI flag selection must not replace a valid directed attack when its route
 // is intentionally empty because the enemy is already within firing range.
 s=init(fixture({team:1,distance:128}),[1]);attacker=s.units[0].id;target=s.units[1].id;s=command(s,target,'stop');s=command(s,attacker,'attack',{target,x:unit(s,target).x,y:unit(s,target).y});s=tick(45);
 assert.equal(unit(s,attacker).order,'attack','AI retargeted an active close-range attack to a flag');assert.equal(unit(s,attacker).target,target);
 console.log('Pursuit passed: nonempty-route refresh, moving-target marker, stop/fire/restart range, explicit Stop/move override, reachable same-bank/clear-sight firing positions, neutralization/death cleanup, stationary gun and AI directed-target retention.');
})().catch(error=>{console.error(error.stack||error);process.exitCode=1;});
