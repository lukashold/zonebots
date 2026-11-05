'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs');
const {model,layout,timeText,width}=require('../dist/independent/original-result-view.js');
const caseResult={losses:[17,53,0,0],points:12345,rank:4};
const result=(won,mp)=>model(caseResult,3671,won,mp);
assert.deepEqual(result(true,false).rows.map(x=>x.key),['rank','time','own','enemy','points']);
assert.deepEqual(result(false,false).rows.map(x=>x.key),['time','enemy','own']);
assert.deepEqual(result(true,true).rows.map(x=>x.key),['time','own','enemy']);
assert.equal(timeText(0),'0 : 00 : 00');assert.equal(timeText(3671),'1 : 01 : 11');
assert.equal(result(false,false).action,'Spiel verlassen');
assert.equal(result(true,true).action,'OK');
assert.equal(model(caseResult,null,true,false),null);
const trace=process.env.NATIVE_RESULT_LAYOUT_TRACE,assets=process.env.NATIVE_RESULT_ART_MANIFEST;
if(trace||assets){
 assert.ok(trace&&assets,'Both the original draw trace and extracted font metrics are required');
 const fonts=JSON.parse(fs.readFileSync(assets,'utf8'));
 const measure=(font,text)=>width(font===2?fonts.large:fonts.medium,text);
 const rows=fs.readFileSync(trace,'utf8').trim().split(/\r?\n/).map(JSON.parse);
 assert.equal(rows.length,180);
 const german=rows.filter(row=>row.mode===0&&row.lang===2);
 for(const row of german){
  const record={...caseResult,rank:Math.min(9,Math.floor(row.rank/10))};
  const card=model(record,3671,Boolean(row.win),Boolean(row.mp));
  const g=layout(card,...row.view,measure,fonts.large.height);
  const native=new Map(row.events.filter(e=>e.text!==undefined).map(e=>[e.text,e]));
  const placed=[g.title,...g.entries.flatMap(r=>[{text:r.label,x:r.x,y:r.y},{text:r.value,x:r.valueX,y:r.y}]),g.action];
  assert.equal(placed.length,native.size,`row ${row.id} text count`);
  for(const line of placed){const e=native.get(line.text);assert.ok(e,`row ${row.id}: ${line.text}`);assert.deepEqual([line.x,line.y],[e.x,e.y],`row ${row.id}: ${line.text}`);}
  const close=(a,b)=>Math.abs(a-b)<.006;
  const quad=(rect,actual)=>assert.ok(rect.every((x,i)=>close(x,actual[i])),`row ${row.id}: ${rect} / ${actual}`);
  const [px,py,pw,ph]=g.panel;quad([px,py,px+pw,py+ph],row.events.find(e=>e.texture===5).xy);
  const dividers=row.events.filter(e=>e.texture===7);
  assert.equal(dividers.length,g.entries.length);
  const [dx,dw,dh]=g.divider;
  for(const entry of g.entries)assert.ok(dividers.some(({xy})=>[dx,entry.dividerY,dx+dw,entry.dividerY+dh].every((n,i)=>close(n,xy[i]))),`row ${row.id}: divider ${entry.key}`);
  const button=row.events.filter(e=>e.texture===8);
  const {x,y,w,h}=g.button;quad([x,y,x+w,y+h],[...button[0].xy.slice(0,2),...button.at(-1).xy.slice(2)]);
 }
 assert.equal(german.length,24);
 console.log('Classic result: all 24 German PC draw cases agree on text positions, panel, separators and button.');
}else console.log('Classic result: distinct win, loss and network rows plus original duration format.');
