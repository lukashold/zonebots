(function(global){
 'use strict';
 const word=n=>(n<<16)>>16;
 function create(){
  let path=null;
  return {advance(scene,current){
   if(!scene?.active){path=null;return null;}
   const request=scene.panRequest;
   if(!Array.isArray(request)||!Number.isInteger(scene.frame)||scene.frame<2)return null;
   if(!path)path={current:current.map(word),target:[word(request[0]-32),word(request[1]+8)],steps:0};
   const due=Math.min(16,scene.frame-2);
   if(due<=path.steps)return null;
   while(path.steps<due){
    const remaining=16-path.steps;
    path.current=path.current.map((v,i)=>word(v+Math.trunc(2*(path.target[i]-v)/remaining)));
    path.steps++;
    if(path.steps===16)path.current=[...path.target];
   }
   return [...path.current];
  }};
 }
 global.FortCamera={create};
})(window);
