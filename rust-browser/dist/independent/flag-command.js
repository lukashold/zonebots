// Capture intent shared by the flag cursor and the click handler.
// This only acknowledges an order; ownership changes come from the simulation.
window.FlagCommand=(()=>{
 'use strict';
 function alive(state,u){return u?.team===0&&Number.isFinite(u.hp)&&(state.map?.original?u.hp>=0:u.hp>0);}
 function iterableIds(ids){return ids&&typeof ids[Symbol.iterator]==='function'?[...new Set(ids)]:[];}
 function mobileIds(state,selectedIds){
  const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  return iterableIds(selectedIds).filter(id=>{const u=units.get(id),speed=u&&catalog.get(u.type)?.speed;return alive(state,u)&&Number.isFinite(speed)&&speed>0;});
 }
 function intent(state,selectedIds,point,options={},boundsFn){
  if(!state||options.paused||options.menu||options.commandMode==='attack'||state.winner!==null&&state.winner!==undefined)return null;
  if(!point||!Number.isFinite(point.x)||!Number.isFinite(point.y)||typeof boundsFn!=='function')return null;
  if(!selectedIds||typeof selectedIds[Symbol.iterator]!=='function')return null;
  const ids=mobileIds(state,selectedIds);
  if(!ids.length)return null;
  // Flags are drawn in sector order; inspect the last drawn flag first.
  const sectors=state.sectors||[];
  for(let i=sectors.length-1;i>=0;i--){
   const sector=sectors[i];
   // The original flag matcher also accepts already owned flags as move targets.
   if(sector.home||sector.countsForProduction===false||!Number.isFinite(sector.x)||!Number.isFinite(sector.y))continue;
   const b=boundsFn(sector);
   if(!b||![b.x,b.y,b.w,b.h].every(Number.isFinite)||b.w<=0||b.h<=0)continue;
   if(point.x>=b.x&&point.x<=b.x+b.w&&point.y>=b.y&&point.y<=b.y+b.h)return {sector,ids,x:sector.x,y:sector.y};
  }
  return null;
 }
 // Keep the destination marker with its order, even while another unit is selected.
 // Expired orders are removed rather than revived when the unit is selected again.
 function createTracker(){
  const orders=new Map();
  function assign(target,state){
   if(!target||!state)return;
   const sector=(state.sectors||[]).find(s=>s.id===target.sector?.id);
   if(!sector||sector.home||sector.countsForProduction===false)return;
   const started=Number.isFinite(state.time)?state.time:0;
   for(const id of mobileIds(state,target.ids))orders.set(id,{sectorId:sector.id,ownerAtIssue:target.sector.owner,started});
  }
  function clear(ids){for(const id of iterableIds(ids))orders.delete(id);}
  function reset(){orders.clear();}
  function targets(state,selectedIds){
   if(!state||state.winner!==null&&state.winner!==undefined){reset();return [];}
   const units=new Map((state.units||[]).map(u=>[u.id,u])),sectors=new Map((state.sectors||[]).map(s=>[s.id,s]));
   const selected=new Set(iterableIds(selectedIds)),visible=new Map();
   for(const [id,order]of orders){
    const unit=units.get(id),sector=sectors.get(order.sectorId);
    if(!alive(state,unit)||unit.order!=='move'||!Array.isArray(unit.path)||!unit.path.length||!sector||sector.home||sector.countsForProduction===false||!Number.isFinite(sector.x)||!Number.isFinite(sector.y)||order.ownerAtIssue!==0&&sector.owner===0){orders.delete(id);continue;}
    if(!selected.has(id))continue;
    const old=visible.get(sector.id);
    if(!old||order.started<old.started)visible.set(sector.id,{sector,x:sector.x,y:sector.y,started:order.started});
   }
   return [...visible.values()];
  }
  return Object.freeze({assign,clear,reset,targets});
 }
 return Object.freeze({intent,createTracker});
})();
