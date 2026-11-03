"""Synthetic PE regression: virtual tails are loader-zeroed, never file spillover."""
from pathlib import Path
import importlib.util,struct
root=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('campaign_reader',root/'tools/extract-original-campaign.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
data=bytearray(0x240);struct.pack_into('<I',data,60,0x80)
struct.pack_into('<H',data,0x86,1);struct.pack_into('<H',data,0x94,0)
struct.pack_into('<IIII',data,0x98+8,64,0x2000,16,0x200)
data[0x200:0x210]=bytes(range(16));data[0x210:0x220]=b'NOT_BSS_CONTENT!'
class Buffer:
 def read_bytes(self):return bytes(data)
exe=module.Exe(Buffer())
assert exe.read(0x402000,16)==bytes(range(16))
assert exe.read(0x40200c,8)==bytes(range(12,16))+bytes(4)
assert exe.read(0x402010,32)==bytes(32)
assert exe.read(0x40203f,1)==bytes(1)
try:exe.read(0x40203f,2)
except ValueError:pass
else:raise AssertionError('Read escaped its mapped section')
print('PASS: stored PE bytes, partial raw/virtual reads, zero-filled BSS and section bounds.')
