"""Pack upgrade tests. Generated numeric inputs, no original files required."""
from pathlib import Path
import importlib.util,json,tempfile,copy,struct,hashlib
from unittest.mock import patch

repo=Path(__file__).resolve().parents[1]
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
extract=module('bridge80_extract',repo/'tools/extract-original-campaign.py')
serve=module('bridge80_serve',repo/'independent-engine/serve.py')
class NumericExe:
 def __init__(self,path):self.d=path.read_bytes()
 def ptr(self,p):
  if 0x496210<=p<0x496220:return 0x1000+(p-0x496210)//4*0x100
  direction=(p-0x1000)//0x100;part=(p&255)//4
  return 0xffffffff if part==(6 if direction%2==0 else 4)else 0x2000+direction*0x1000+part*0x100
 def read(self,p,n):return struct.pack('<4h',p%30000,-12,7,3)
with tempfile.TemporaryDirectory(prefix='z-bridge80-import-',dir=repo)as temp:
 root=Path(temp);game=root/'game';game.mkdir();out=root/'dist/original';out.mkdir(parents=True)
 (game/'Z.exe').write_bytes(b'numeric source fixture')
 for n in range(1,21):(game/f'BRIDGE{n:02}.DAT').write_bytes(bytes((i+n)%256 for i in range(312)))
 for prefix in 'VH':
  for i in range(4):(game/f'{prefix}SHWING{i}.DAT').write_bytes(bytes((j+i)%256 for j in range(1280)))
 original={'version':8,'sourceExeSha256':hashlib.sha256((game/'Z.exe').read_bytes()).hexdigest(),'unknownWarmRoot':[9,4],'groups':[{'custom':True}],'missions':[{'number':n,'map':{'retained':[n,82]},'customField':{'keep':None}}for n in range(1,21)]}
 path=out/'campaign.json';path.write_text(json.dumps(original))
 with patch.object(extract,'Exe',NumericExe):extract.enrich_bridge80(game,out)
 enriched=json.loads(path.read_text());assert serve.has_bridge80_inputs(enriched)
 stripped=copy.deepcopy(enriched)
 for m in stripped['missions']:
  for field in extract.BRIDGE80_FIELDS:m.pop(field)
 assert stripped==original,'enrichment changed unrelated campaign data'
 saved=path.read_bytes()
 with patch.object(extract,'Exe',NumericExe):extract.enrich_bridge80(game,out)
 assert path.read_bytes()==saved,'enrichment must be idempotent'
 (game/'HSHWING3.DAT').unlink()
 try:
  with patch.object(extract,'Exe',NumericExe):extract.enrich_bridge80(game,out)
  raise AssertionError('missing source must fail')
 except FileNotFoundError:pass
 assert path.read_bytes()==saved,'failed upgrade overwrote the installed campaign'
 for name in ('sprites.png','sprites.json','tiles.png','audio.json','menu.json','hud.json'):(out/name).write_text('{"version":2}')
 calls=[]
 with patch.object(serve,'__file__',str(root/'independent-engine/serve.py')),patch.object(serve.subprocess,'run',lambda command,**kw:calls.append(command)):
  serve.ensure_original_pack(game);assert not calls
  path.write_text(json.dumps(original));serve.ensure_original_pack(game)
  assert len(calls)==1 and calls[0][-1]=='--enrich-bridge80',calls
  calls.clear();malformed=copy.deepcopy(enriched);malformed['missions'][3]['nativeBridgePlaybackHeaders']=None;path.write_text(json.dumps(malformed));serve.ensure_original_pack(game)
  assert len(calls)==1 and calls[0][-1]=='--enrich-bridge80','malformed optional bridge inputs must not replace unrelated v8 fields'
  calls.clear();original['version']=7;path.write_text(json.dumps(original));serve.ensure_original_pack(game)
  assert len(calls)==1 and '--enrich-bridge80'not in calls[0],calls
 print('PASS:20-mission field-only enrichment; idempotence; failed-source atomicity; current/v8/old-version local and LAN startup selection.')
