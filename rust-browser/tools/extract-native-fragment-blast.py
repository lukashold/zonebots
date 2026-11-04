"""Extract numeric420200 crater rules from a local GOG executable; no artwork."""
from pathlib import Path
import argparse,json,runpy,struct

def extract(exe_path,output):
 Exe=runpy.run_path(str(Path(__file__).with_name('extract-original-campaign.py')))['Exe']
 exe=Exe(exe_path);data=[]
 for world in range(5):
  p=exe.ptr(0x49347c+world*4);classes=[]
  for category in range(3):
   values=[]
   while True:
    value=struct.unpack('<h',exe.read(p,2))[0];p+=2
    if value==-1:break
    if not 0<=value<512:raise ValueError('Invalid41f7e0 class index')
    values.append(value)
   classes.append(values)
  words=list(struct.unpack('<36H',exe.read(exe.ptr(0x493fd4+world*4),72)))
  data.append((classes,words))
 text='// Numeric42e1a0/41f7e0 rules. Browser desert,volcanic,arctic,jungle,city\n// correspond to native world indices0,3,1,4,2. No artwork payload.\n'
 text+='pub const CLASSES:[[&[u16];3];5]=[\n'
 for world in [0,3,1,4,2]:text+='['+','.join('&['+','.join(map(str,items))+']' for items in data[world][0])+'],\n'
 text+='];\npub const WORDS:[[u16;36];5]=[\n'
 for world in [0,3,1,4,2]:text+='['+','.join(map(str,data[world][1]))+'],\n'
 text+='];\n';output.write_text(text,encoding='utf-8')
 print('Extracted5 numeric crater/class tables to',output)

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--exe',type=Path,default=Path(r'C:\GOG Games\Z\Z.exe'))
 parser.add_argument('--output',type=Path,default=Path(__file__).resolve().parents[1]/'independent-engine/src/original_fragment_blast_data.rs')
 args=parser.parse_args();extract(args.exe,args.output)
