"""Compare the reconstructed motion constants to the user's local GOG binary.
No original binary or decompiler output is copied into the repository.
"""
from pathlib import Path
import importlib.util,json,re,struct,hashlib
root=Path(__file__).resolve().parents[1]
game=Path(r'C:\GOG Games\Z\Z.exe')
if not game.exists():
    print('SKIP: local GOG executable unavailable; use the synthetic Rust/Wasm traces.')
    raise SystemExit(0)
spec=importlib.util.spec_from_file_location('extractor',root/'tools/extract-original-campaign.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
exe=module.Exe(game)
assert struct.unpack('<f',exe.read(0x475f44,4))[0]==1000.0
assert struct.unpack('<f',exe.read(0x49ccf4,4))[0]==17.0
assert exe.read(0x435ad8,2)==bytes.fromhex('ff02'), 'Shared cursor increment must precede unit dispatch'
source=(root/'independent-engine/src/original_motion.rs').read_text()
block=source.split('const STEPS:',1)[1].split('\n];',1)[0]
patterns=[[int(v)for v in row.split(',')if v]for row in re.findall(r'&\[([0-9,]+)\]',block)]
assert len(patterns)==13
for i,expected in enumerate(patterns):
    ptr=struct.unpack('<I',exe.read(0x499fd4+4*i,4))[0]
    assert struct.unpack('<I',exe.read(ptr,4))[0]==0
    actual=[]
    for n in range(64):
        pair=exe.read(ptr+4+2*n,2)
        if pair==b'\xff\xff':break
        actual.append(pair[0])
    else:raise AssertionError('Missing native pattern terminator')
    assert actual==expected,(i,actual,expected)
print(json.dumps({'verifiedPatterns':13,'nominalHz':17,'integerSdlIntervalMs':59,'exeSha256':hashlib.sha256(game.read_bytes()).hexdigest()}))
print('PASS: all13 movement sequences, initial shared cursors and the normal GOG timing constants match the local executable.')
