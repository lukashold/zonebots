// Shared vehicle-workshop intent. Rust owns routing, occupancy and repair work.
window.WorkshopCommand=(()=>{
 'use strict';
 const idsOf=ids=>ids&&typeof ids[Symbol.iterator]==='function'?[...new Set(ids)]:[];
 const live=(state,u)=>u&&Number.isFinite(u.hp)&&(state.map?.original?u.hp>=0:u.hp>0);
 function vehicleIds(state,selected){
  if(!state)return [];
  const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  return idsOf(selected).filter(id=>{const u=units.get(id),d=u&&catalog.get(u.type);return u?.team===0&&live(state,u)&&d?.class===1&&u.crew!=null&&u.workshop?.phase!=='repairing'&&Number.isFinite(u.maxHp)&&u.hp<u.maxHp;});
 }
 function usable(b){return b?.kind==='repair'&&b.team===0&&Number.isFinite(b.hp)&&b.hp>0;}
 //0041c3f0 addresses the inner repair anchor: native half-size minus8.
 function entrance(b){return {x:b.x+b.w/2-16,y:b.y+b.h/2-16};}
 function intent(state,selected,p,options={}){
  if(!state||options.paused||options.menu||options.placement||options.commandMode==='attack'||state.winner!==null&&state.winner!==undefined||!p||!Number.isFinite(p.x)||!Number.isFinite(p.y))return null;
  const ids=vehicleIds(state,selected);if(!ids.length)return null;
  const b=(state.buildings||[]).find(b=>usable(b)&&p.x>=b.x&&p.x<=b.x+b.w&&p.y>=b.y&&p.y<=b.y+b.h);
  return b?{building:b,kind:'repairVehicle',ids,...entrance(b)}:null;
 }
 function createTracker(){
  const orders=new Map();
  function clear(ids){for(const id of idsOf(ids))orders.delete(id);}
  function reset(){orders.clear();}
  function assign(target,state){if(!target||!state||!usable(target.building))return;for(const id of vehicleIds(state,target.ids)){const u=state.units.find(u=>u.id===id);if(u.order==='repairVehicle'&&u.target===target.building.id)orders.set(id,{buildingId:target.building.id,started:Number.isFinite(state.time)?state.time:0});}}
  function targets(state,selected){
   if(!state||state.winner!==null&&state.winner!==undefined){reset();return [];}
   const units=new Map((state.units||[]).map(u=>[u.id,u])),buildings=new Map((state.buildings||[]).map(b=>[b.id,b])),visible=new Map(),chosen=new Set(idsOf(selected));
   for(const[id,order]of orders){const u=units.get(id),b=buildings.get(order.buildingId);if(!live(state,u)||u.team!==0||u.order!=='repairVehicle'||u.target!==order.buildingId||!usable(b)){orders.delete(id);continue;}if(!chosen.has(id))continue;const old=visible.get(b.id);if(!old||order.started<old.started)visible.set(b.id,{building:b,...entrance(b),started:order.started});}
   return [...visible.values()];
  }
  return Object.freeze({assign,clear,reset,targets});
 }
 return Object.freeze({vehicleIds,intent,createTracker});
})();
