//545b0/4549f0 shared queue. Native order is supplied by Rust; plain records
//inherit or select palettes at submission time, after BUILD's direct pass.
window.OriginalDisplay=(()=>{
 'use strict';
 const active=state=>Boolean(state?.map?.original&&window.OriginalArt?.ready&&!window.GameGraphics?.isHD()&&Array.isArray(state.nativeDisplayQueue));
 function sort(records,key=r=>r.queueY){for(let end=records.length-1;end>0;end--)for(let i=0;i<end;i++)if(key(records[end])<key(records[i]))[records[end],records[i]]=[records[i],records[end]];return records;}
 function palette(request,current=0,forced=false){if(request===255)return current;const p=forced?8:request&15;return p<=3?p:p<=7?255:-1;}
 function draw(ctx,state,adapters){if(!active(state))return false;const rows=state.nativeDisplayQueue,units=new Map(state.units.map(u=>[u.id,u])),buildings=new Map(state.buildings.map(b=>[b.id,b]));
  const submit=row=>{const current=window.OriginalArt.nativePalette?.(ctx)??0,p=palette(row.palette??255,current);window.OriginalArt.selectNativePalette?.(ctx,p);const f={...row,team:p,nativeInheritedPalette:false,nativeWhitePalette:p===255};adapters.sprite(f);};
  for(const row of rows){if(row.stage!=='pre')continue;if(row.kind==='building'){const b=row.art??buildings.get(row.id);if(b)adapters.building(row.radarDishQueued?{...b,nativeRadarDishQueued:true}:b,row.time,row.seconds);}else if(row.kind==='repairOverlay')window.OriginalArt.repairOverlay?.(ctx,row,state.map);else if(row.kind==='repairClock')window.OriginalArt.clock?.(ctx,row.x,row.y,row.seconds,state.map);else if(row.kind==='sprite')submit(row);}
  adapters.objectShadows?.();
  for(const row of rows){if(row.stage==='pre')continue;if(row.kind==='sprite'){submit(row);continue;}if(row.kind==='effect'){adapters.sprite(row.draw);continue;}if(row.kind==='unit'){const u=units.get(row.id),current=window.OriginalArt.nativePalette?.(ctx)??0,p=palette(row.palette??255,current);window.OriginalArt.selectNativePalette?.(ctx,p);const sprite=Number.isInteger(row.sprite)?{sprite:row.sprite,x:row.x,y:row.y,team:p,scale:1,rotation:0,nativeWhitePalette:p===255}:null;
   if(sprite){if(u)adapters.unit({...u,x:row.x,y:row.y,nativeDrawPalette:p,nativeDrawSprite:sprite},sprite);else adapters.sprite(sprite);}
   else if(row.art)adapters.unit({...row.art,id:row.id,nativeDrawPalette:p,...(row.nativeLayers?{nativeDrawLayers:row.nativeLayers}:{})},null,row.time);
   else if(u)adapters.unit({...u,nativeDrawPalette:p},null);
  }}
  return true;
 }
 return Object.freeze({active,sort,palette,draw});
})();
