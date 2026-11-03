//68c80/68d30 packed portrait composition. Bitmap pixels/phrase sequences are
//supplied only by the owner's private HUD pack.
window.OriginalFace=(()=>{
 'use strict';
 const eye=[[15,16],[15,16],[15,15],[15,16],[15,17]],mouth=[[11,8],[11,8],[11,7],[11,8],[11,9]],collar=[[2,26],[2,21],[2,23],[2,21],[2,23]],overlay=[[21,23],[21,23],[21,23],[21,23],[21,23]];
 const special=[[23,31],[23,24],[23,18],[24,13],[25,6],[26,0],[29,2],[33,7],[30,26],[26,17],[26,12],[26,11],[27,14],[29,18],[31,22],[34,29]],specialFrames=[0,1,2,2,3,4,4,4,5,5,5,6,6,6,5,5];
 function decode(packed,base=0){packed&=65535;let mode=packed>>6&3,look=packed>>2&3,extra=-1;
  if(mode===3){extra=packed&15;mode=0;look=-1;}
  return [(packed&3)<<2,look,mode?-1:packed>>12&15,mode?-1:packed>>8&15,mode,packed>>4&3,extra,0,base,0];
 }
 function layers(packed,head,nativeWorld,vehicle,layout,origin=[0,0]){
  head=Math.max(0,Math.min(4,head|0));const base=layout.headStarts[head],d=decode(packed,base),[x,y]=origin,draws=[];
  const add=(sprite,dx=0,dy=0)=>draws.push({sprite,x:x+dx,y:y+dy});
  add(layout.backgroundStart+(vehicle?2:[0,3,4,1,5][nativeWorld]??0));
  add(base+d[4]+1,4,3+d[5]);
  if(d[2]>=0)add(base+d[2]+4,eye[head][0],eye[head][1]+d[5]);
  if(d[3]>=0)add(base+d[3]+20,mouth[head][0],mouth[head][1]+d[5]);
  add(base,collar[head][0],collar[head][1]);
  if(d[1]>=0)add(base+d[1]+31,overlay[head][0],overlay[head][1]+d[0]);
  if(d[6]>=0)add(base+specialFrames[d[6]]+33,special[d[6]][0],special[d[6]][1]);
  return draws;
 }
 //46ad10 reads the preceding frame index;46ad70 updates it afterwards.
 function phraseWord(words,tick){if(!Array.isArray(words)||!words.length)return 15;const index=tick<=1?0:Math.floor((tick-1)*72/17);return index<words.length?words[index]:15;}
 return Object.freeze({decode,layers,phraseWord});
})();
