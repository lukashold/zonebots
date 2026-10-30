// Battlefield terrain is always revealed. Only minimap enemy markers need radar.
(function(global){
  function radarRadius(sectors,building){
    if(building.team!==0||building.kind!=='radar'||building.hp<=0)return 0;
    const owned=sectors.filter(s=>s.owner===0).length;
    return (260+owned*45)*Math.max(0,Math.min(1,building.hp/building.maxHp));
  }
  function radarDetects(sectors,buildings,x,y){
    return buildings.some(b=>{const radius=radarRadius(sectors,b);return radius>0&&Math.hypot(b.x+b.w/2-x,b.y+b.h/2-y)<=radius;});
  }
  global.BattlefieldVisibility=Object.freeze({radarRadius,radarDetects});
})(typeof window==='undefined'?globalThis:window);
