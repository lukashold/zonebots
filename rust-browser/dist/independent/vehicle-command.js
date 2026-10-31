// Vehicle-entry intent shared by pointer feedback and click handling.
// Boarding itself is performed by the simulation, not by this presentation state.
window.VehicleCommand=(()=>{
 'use strict';
 function live(state,u){return Boolean(u)&&Number.isFinite(u.hp)&&(state.map?.original?u.hp>=0:u.hp>0);}
 function iterableIds(ids){return ids&&typeof ids[Symbol.iterator]==='function'?[...new Set(ids)]:[];}
 function infantryIds(state,selectedIds){
  const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  return iterableIds(selectedIds).filter(id=>{const u=units.get(id);return u?.team===0&&live(state,u)&&catalog.get(u.type)?.class===0;});
 }
 function entryKind(state,u,catalog){
  if(!live(state,u)||u.workshop?.phase==='repairing'||!Number.isFinite(u.x)||!Number.isFinite(u.y))return null;
  const cls=catalog.get(u.type)?.class;
  if(cls!==1&&cls!==2)return null;
  if(u.team<0&&u.crew==null)return 'enter';
  if(u.team===0&&u.type==='apc'&&(Array.isArray(u.cargo)?u.cargo.length:0)<6)return 'board';
  return null;
 }
 function hit(unit,point,boundsFn){
  const b=typeof boundsFn==='function'?boundsFn(unit):null;
  if(b&&[b.x,b.y,b.w,b.h].every(Number.isFinite)&&b.w>0&&b.h>0)return point.x>=b.x&&point.x<=b.x+b.w&&point.y>=b.y&&point.y<=b.y+b.h;
  return Math.hypot(point.x-unit.x,point.y-unit.y)<22;
 }
 function intent(state,selectedIds,point,options={},boundsFn){
  if(!state||options.paused||options.menu||options.commandMode==='attack'||state.winner!==null&&state.winner!==undefined)return null;
  if(!point||!Number.isFinite(point.x)||!Number.isFinite(point.y))return null;
  const ids=infantryIds(state,selectedIds);
  if(!ids.length)return null;
  const catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  // Match the existing unit hit-order; the cursor and click use this same result.
  for(const unit of state.units||[]){const kind=entryKind(state,unit,catalog);if(kind&&hit(unit,point,boundsFn))return {unit,kind,ids,x:unit.x,y:unit.y};}
  return null;
 }
 function createTracker(){
  const orders=new Map();
  function assign(target,state){
   if(!target||!state)return;
   const unit=(state.units||[]).find(u=>u.id===target.unit?.id),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
   const kind=entryKind(state,unit,catalog);
   if(!kind||kind!==target.kind)return;
   const started=Number.isFinite(state.time)?state.time:0;
   for(const id of infantryIds(state,target.ids))orders.set(id,{unitId:unit.id,kind,started});
  }
  function clear(ids){for(const id of iterableIds(ids))orders.delete(id);}
  function reset(){orders.clear();}
  function targets(state,selectedIds){
   if(!state||state.winner!==null&&state.winner!==undefined){reset();return [];}
   const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
   const selected=new Set(iterableIds(selectedIds)),visible=new Map();
   for(const [id,order]of orders){
    const robot=units.get(id),unit=units.get(order.unitId);
    // An empty path is allowed: enter/board builds its route on the next tick,
    // and the simulation can keep following an APC while that carrier moves.
    if(robot?.team!==0||!live(state,robot)||catalog.get(robot.type)?.class!==0||robot.order!==order.kind||robot.target!==order.unitId||entryKind(state,unit,catalog)!==order.kind){orders.delete(id);continue;}
    if(!selected.has(id))continue;
    const old=visible.get(unit.id);
    if(!old||order.started<old.started)visible.set(unit.id,{unit,kind:order.kind,x:unit.x,y:unit.y,started:order.started});
   }
   return [...visible.values()];
  }
  return Object.freeze({assign,clear,reset,targets});
 }
 return Object.freeze({intent,createTracker});
})();
