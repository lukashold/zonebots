// Grenade-crate intent and presentation state; Rust owns the actual pickup.
window.PickupCommand=(()=>{
 'use strict';
 const iterableIds=ids=>ids&&typeof ids[Symbol.iterator]==='function'?[...new Set(ids)]:[];
 const available=crate=>Boolean(crate)&&!crate.taken&&Number.isFinite(crate.x)&&Number.isFinite(crate.y);
 const crateKey=crate=>JSON.stringify([crate.x,crate.y]);
 const destination=unit=>Array.isArray(unit?.dest)&&unit.dest.length===2&&unit.dest.every(Number.isFinite)?unit.dest:null;
 function infantryIds(state,ids){
  const units=new Map((state.units||[]).map(u=>[u.id,u])),catalog=new Map((state.catalog||[]).map(d=>[d.type,d]));
  return iterableIds(ids).filter(id=>{const u=units.get(id);return u?.team===0&&Number.isFinite(u.hp)&&(state.map?.original?u.hp>=0:u.hp>0)&&catalog.get(u.type)?.class===0;});
 }
 function intent(state,selected,point,options={},boundsFn){
  if(!state||options.paused||options.menu||options.placement||['attack','placement'].includes(options.commandMode)||state.winner!==null&&state.winner!==undefined)return null;
  if(!point||!Number.isFinite(point.x)||!Number.isFinite(point.y))return null;
  const ids=infantryIds(state,selected);if(!ids.length)return null;
  // Crates share the battlefield's stable Y sort. Inspect the last painted first.
  const crates=(state.crates||[]).map((crate,index)=>({crate,index})).filter(({crate})=>available(crate)).sort((a,b)=>b.crate.y-a.crate.y||b.index-a.index);
  for(const {crate}of crates){
   const supplied=typeof boundsFn==='function'?boundsFn(crate):null;
   const b=supplied??{x:crate.x-12,y:crate.y-10,w:24,h:20};
   if(![b.x,b.y,b.w,b.h].every(Number.isFinite)||b.w<=0||b.h<=0)continue;
   if(point.x>=b.x&&point.x<=b.x+b.w&&point.y>=b.y&&point.y<=b.y+b.h)return {crate,ids,x:crate.x,y:crate.y};
  }
  return null;
 }
 function createTracker(){
  const orders=new Map();
  function assign(target,state){
   if(!target||!state||!available(target.crate))return;
   const key=crateKey(target.crate),crate=(state.crates||[]).find(c=>available(c)&&crateKey(c)===key);
   if(!crate)return;
   const units=new Map((state.units||[]).map(u=>[u.id,u])),started=Number.isFinite(state.time)?state.time:0;
   for(const id of infantryIds(state,target.ids)){
    const unit=units.get(id),dest=destination(unit);
    if(unit.order!=='move'||!dest||!Array.isArray(unit.path))continue;
    orders.set(id,{key,dest:[...dest],started,hadRoute:unit.path.length>0});
   }
  }
  function clear(ids){for(const id of iterableIds(ids))orders.delete(id);}
  function reset(){orders.clear();}
  function targets(state,selected){
   if(!state||state.winner!==null&&state.winner!==undefined){reset();return [];}
   const eligible=new Set(infantryIds(state,orders.keys())),chosen=new Set(iterableIds(selected));
   const units=new Map((state.units||[]).map(u=>[u.id,u])),crates=new Map((state.crates||[]).filter(available).map(c=>[crateKey(c),c])),visible=new Map();
   for(const [id,order]of orders){
    const unit=units.get(id),dest=destination(unit),crate=crates.get(order.key),route=Array.isArray(unit?.path)&&unit.path.length>0;
    const initialEmpty=!order.hadRoute&&state.time===order.started;
    if(!eligible.has(id)||unit.order!=='move'||!dest||dest[0]!==order.dest[0]||dest[1]!==order.dest[1]||!crate||state.time<order.started||!route&&!initialEmpty){orders.delete(id);continue;}
    if(route)order.hadRoute=true;
    if(!chosen.has(id))continue;
    const old=visible.get(order.key);
    if(!old||order.started<old.started)visible.set(order.key,{crate,x:crate.x,y:crate.y,started:order.started});
   }
   return [...visible.values()];
  }
  return Object.freeze({assign,clear,reset,targets});
 }
 return Object.freeze({intent,createTracker});
})();
