"""Verify native bridge initialization separately from the Wasm adapter."""
from pathlib import Path
import importlib.util, json, struct

root=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('campaign_extractor',root/'tools/extract-original-campaign.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
def record(x,saved,current):
    return struct.pack('<6H',x,16,16,16,saved,current)+bytes(14)
tiles=[0]*25;tiles[6]=tiles[7]=5
base,hp,caps,bridges=module.extract_bridges(record(16,500,0)+record(32,300,600),5,5,tiles)
assert hp==[0,600] and caps==[600,600]
assert bridges[0]['initiallyDestroyed'] and not bridges[1]['initiallyDestroyed']
assert bridges[0]['rawSavedHp']==500 and bridges[1]['rawSavedHp']==300
assert tiles[6]==3 and tiles[7]==5 and base[6:8]==[0,1]
# Invalid/sentinel records and records with no real deck cells are not imported.
assert module.extract_bridges(record(65535,0,0)+record(320,500,0),5,5,[0]*25)[1]==[]

campaign_path=root/'dist/original/campaign.json';game=Path(r'C:\GOG Games\Z')
if campaign_path.exists() and game.exists():
    campaign=json.loads(campaign_path.read_text(encoding='utf-8'))
    assert campaign['version']>=3
    broken={};total=0
    for mission in campaign['missions']:
        source=(game/f"BRIDGE{mission['number']:02}.DAT").read_bytes()
        assert len(mission['map']['bridgeHp'])==len(mission['map']['bridgeMaxHp'])
        for bridge in mission['bridges']:
            raw=source[bridge['sourceOffset']:bridge['sourceOffset']+26]
            saved,initial=struct.unpack_from('<HH',raw,8)
            assert bridge['rawSavedHp']==saved
            assert bridge['hp']==(initial if initial else 0)
            assert bridge['maxHp']==(initial or 600)
            assert mission['map']['bridgeHp'][bridge['id']]==bridge['hp']
            assert mission['map']['bridgeMaxHp'][bridge['id']]==bridge['maxHp']
            assert all(mission['map']['tiles'][cell]==(5 if initial else 3) for cell in bridge['tiles'])
            if not initial:broken[mission['number']]=broken.get(mission['number'],0)+1
            total+=1
    assert total==49 and broken=={14:4,15:5,17:2,18:4,20:4}
    print(json.dumps({'verifiedOriginalBridges':total,'initiallyDestroyed':sum(broken.values()),'missions':broken}))
print('PASS: bridge field+10, overwritten savedHP, zero-state destruction,600HP reconstruction, deck cells and actual source records.')
