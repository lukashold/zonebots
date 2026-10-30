// Attack intent shared by cursor and click handling; Rust resolves combat.
window.AttackCommand=(()=>{
 'use strict';
 const iterableIds=ids=>ids&&typeof ids[Symbol.iterator]==='function'?[...new Set(ids)]:[];
 const live=(state,u)=>Boolean(u)&&Number.isFinite(u.hp)&&(state.map?.original?u.hp>=0:u.hp>0);
 const finitePoint=p=>Boolean(p)&&Number.isFinite(p.x)&&Number.isFinite(p.y);
 const destination=u=>Array.isArray(u?.dest)&&u.dest.length===2&&u.dest.every(Number.isFinite)?u.dest:null;
 const sameDest=(a,b)=>a===null?b===null:b!==null&&a[0]===b[0]&&a[1]===b[1];
 function fighterIds(state,selected){
  if(!state)return [];
  const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  return iterableIds(selected).filter(id=>{const u=units.get(id),d=u&&catalog.get(u.type);return u?.team===0&&live(state,u)&&Number.isFinite(d?.damage)&&d.damage>0&&Number.isFinite(d?.range)&&d.range>0;});
 }
 const enemyUnit=(state,u)=>live(state,u)&&finitePoint(u)&&Number.isFinite(u.team)&&u.team>=0&&u.team!==0;
 const enemyBuilding=b=>finitePoint(b)&&Number.isFinite(b.w)&&b.w>0&&Number.isFinite(b.h)&&b.h>0&&Number.isFinite(b.hp)&&b.hp>0&&Number.isFinite(b.team)&&b.team!==0;
 function inMap(state,p){const m=state.map,t=m?.tileSize||32;return finitePoint(p)&&Number.isFinite(m?.width)&&m.width>0&&Number.isFinite(m?.height)&&m.height>0&&p.x>=0&&p.y>=0&&p.x<m.width*t&&p.y<m.height*t;}
 function bridgeAt(state,p){
  if(!inMap(state,p))return null;
  const m=state.map,t=m.tileSize||32,col=Math.floor(p.x/t),row=Math.floor(p.y/t),cell=row*m.width+col,id=m.bridgeBase?.[cell];
  return Number.isInteger(id)&&id>=0&&Number.isFinite(m.bridgeHp?.[id])&&m.bridgeHp[id]>0?{id,cell,x:(col+.5)*t,y:(row+.5)*t}:null;
 }
 function hitUnit(unit,p,boundsFn){
  const b=typeof boundsFn==='function'?boundsFn(unit):null;
  if(b&&[b.x,b.y,b.w,b.h].every(Number.isFinite)&&b.w>0&&b.h>0)return p.x>=b.x&&p.x<=b.x+b.w&&p.y>=b.y&&p.y<=b.y+b.h;
  return Math.hypot(p.x-unit.x,p.y-unit.y)<22;
 }
 function intent(state,selected,p,options={},unitBoundsFn){
  if(!state||options.paused||options.menu||options.placement||options.commandMode==='placement'||state.winner!==null&&state.winner!==undefined||!finitePoint(p))return null;
  const ids=fighterIds(state,selected);if(!ids.length)return null;
  for(const unit of state.units||[])if(enemyUnit(state,unit)&&hitUnit(unit,p,unitBoundsFn))return {kind:'attack',targetType:'unit',entity:unit,target:unit.id,ids,x:unit.x,y:unit.y};
  for(const b of state.buildings||[])if(enemyBuilding(b)&&p.x>=b.x&&p.x<=b.x+b.w&&p.y>=b.y&&p.y<=b.y+b.h){
   const gate=Array.isArray(b.gate)&&b.gate.length>=2&&b.gate.every(Number.isFinite)?b.gate:null;
   return {kind:'attackBuilding',targetType:'building',entity:b,target:b.id,ids,x:gate?.[0]??b.x+b.w/2,y:gate?.[1]??b.y+b.h+24};
  }
  if(options.commandMode!=='attack'||!inMap(state,p))return null;
  const bridge=bridgeAt(state,p);
  return bridge?{kind:'attackBridge',targetType:'bridge',target:bridge.id,ids,x:p.x,y:p.y}:{kind:'attack',targetType:'point',ids,x:p.x,y:p.y};
 }
 function createTracker(){
  const orders=new Map();
  function resolve(order,state){
   if(order.targetType==='unit'){
    const entity=(state.units||[]).find(u=>u.id===order.id);
    return enemyUnit(state,entity)?{entity,x:entity.x,y:entity.y}:null;
   }
   if(order.targetType==='building'){
    const entity=(state.buildings||[]).find(b=>b.id===order.id);
    return enemyBuilding(entity)?{entity,x:entity.x+entity.w/2,y:entity.y+entity.h/2}:null;
   }
   if(order.targetType==='bridge'){
    const bridge=bridgeAt(state,order.point);
    return bridge&&bridge.id===order.id&&bridge.cell===order.cell?{x:bridge.x,y:bridge.y}:null;
   }
   return order.targetType==='point'&&inMap(state,order.point)?order.point:null;
  }
  function assign(target,state){
   if(!target||!state)return;
   const {targetType,kind}=target;
   if(!['unit','building','bridge','point'].includes(targetType)||kind!==({unit:'attack',building:'attackBuilding',bridge:'attackBridge',point:'attack'})[targetType])return;
   const bridge=targetType==='bridge'?bridgeAt(state,target):null;
   const order={targetType,kind,id:target.target??target.entity?.id,point:{x:target.x,y:target.y},cell:bridge?.cell,started:Number.isFinite(state.time)?state.time:0};
   if(bridge)order.id=bridge.id;
   if(!resolve(order,state))return;
   const units=new Map((state.units||[]).map(u=>[u.id,u]));
   for(const id of fighterIds(state,target.ids)){
    const u=units.get(id),dest=destination(u),unitTarget=u.target??null;
    if(u.order!==kind||(['unit','building'].includes(targetType)&&unitTarget!==order.id)||(targetType==='point'&&unitTarget!==null)||(['building','bridge','point'].includes(targetType)&&!dest))continue;
    if(targetType==='bridge'&&bridgeAt(state,{x:dest[0],y:dest[1]})?.id!==order.id)continue;
    orders.set(id,{...order,expectedTarget:unitTarget,dest:dest?[...dest]:null,hadRoute:Array.isArray(u.path)&&u.path.length>0});
   }
  }
  function clear(ids){for(const id of iterableIds(ids))orders.delete(id);}
  function reset(){orders.clear();}
  function targets(state,selected){
   if(!state||state.winner!==null&&state.winner!==undefined){reset();return [];}
   const eligible=new Set(fighterIds(state,orders.keys())),chosen=new Set(iterableIds(selected)),units=new Map((state.units||[]).map(u=>[u.id,u])),visible=new Map();
   for(const [id,order]of orders){
    const unit=units.get(id),target=resolve(order,state),route=Array.isArray(unit?.path)&&unit.path.length>0;
    const initialEmpty=!order.hadRoute&&state.time===order.started;
    if(!eligible.has(id)||unit.order!==order.kind||(unit.target??null)!==order.expectedTarget||!sameDest(order.dest,destination(unit))||!target||state.time<order.started||order.targetType==='point'&&!route&&!initialEmpty){orders.delete(id);continue;}
    if(order.targetType==='bridge'&&bridgeAt(state,{x:order.dest[0],y:order.dest[1]})?.id!==order.id){orders.delete(id);continue;}
    if(route)order.hadRoute=true;
    if(!chosen.has(id))continue;
    const key=order.targetType==='point'?JSON.stringify(['point',order.point.x,order.point.y]):JSON.stringify([order.targetType,order.id]),old=visible.get(key);
    if(!old||order.started<old.started)visible.set(key,{targetType:order.targetType,kind:order.kind,...target,started:order.started});
   }
   return [...visible.values()];
  }
  return Object.freeze({assign,clear,reset,targets});
 }
 return Object.freeze({intent,fighterIds,createTracker});
})();
