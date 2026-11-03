"""Compare optional private campaign CPU rosters to the owned mission controls."""
from pathlib import Path
import json,struct,importlib.util
root=Path(__file__).resolve().parents[1];game=Path('C:/GOG Games/Z')
if not(game/'CPUPLR01.DAT').exists():
 print('SKIP: optional local GOG campaign unavailable');raise SystemExit(0)
spec=importlib.util.spec_from_file_location('campaign_reader',root/'tools/extract-original-campaign.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
campaign=json.loads((root/'dist/original/campaign.json').read_text(encoding='utf8'));assert campaign['version']>=5
robots=(game/'robots.dat').read_bytes();groups=units=0
for level in campaign['missions']:
 cp=(game/f"CPUPLR{level['number']:02d}.DAT").read_bytes()
 native_map=(game/f"LEVEL{level['number']:02d}.MAP").read_bytes()
 graph=level['map']['nativeRoadGraph']['regions'];assert len(graph)==96
 for i,r in enumerate(graph):
  offset=0xa971+i*0x88
  assert r['bounds']==list(struct.unpack_from('<4h',native_map,offset))
  assert r['waypoint']==struct.unpack_from('<H',native_map,offset+8)[0]
  assert r['next']==list(native_map[offset+0x28:offset+0x88])
 assert level['nativeAi']['reservePercent']==25
 assert len(level['nativeAi']['terrainPlane'])==128*128
 assert all(0<=u['x']<=level['map']['width']*32 and 0<=u['y']<=level['map']['height']*32 for u in level['spawns'])
 active=[(i,cp[0x792+i*10])for i in range(26)if cp[0x793+i*10]&1]
 actual=[g for g in level['startingGroups']if g['team']==1]
 assert [(g['slot'],g['groupId'])for g in actual]==active,(level['number'],'CPU roster mismatch')
 for slot,gid in active:
  members=[]
  for n in range(21):
   raw=struct.unpack_from('<h',robots,0x4650+gid*82+28+n*2)[0]
   if raw<0:break
   members.append(m.RAW[raw])
  spawned=[u for u in level['spawns']if u['team']==1 and u['squad']==101+slot]
  assert [u['type']for u in spawned]==members,(level['number'],slot)
  task=next(t for t in level['nativeAi']['initialSectorTasks']if t['slot']==slot)
  assert task['sector']==(cp[0x799+slot*10]-1 if cp[0x799+slot*10]else None)
  groups+=1;units+=len(spawned)
 for sector in level['sectors']:
  if sector.get('outside'):continue
  offset=sector['nativeSlot']*30
  assert sector['nativeFlags']==cp[offset+24]
  assert sector['nativeScoringFlags']==cp[offset+29]
  assert sector['nativePrioritySeed']==struct.unpack_from('<i',cp,offset+25)[0]
print(json.dumps({'missions':20,'verifiedCpuGroups':groups,'verifiedCpuUnits':units}))
print('PASS: independent26-slot enemy rosters, members, inactive gaps, scripted tasks and sector metadata.')
