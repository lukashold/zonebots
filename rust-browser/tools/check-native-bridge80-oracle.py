"""Optional private fieldwise native comparison; ordinary Rust hash-golden
tests run unconditionally without original media or this diagnostic.
Streams Rust bytes through memory, never writes another giant raw trace file.
"""
import argparse,base64,json,os,pathlib,subprocess,sys,zlib
parser=argparse.ArgumentParser();parser.add_argument('--trace-dir',type=pathlib.Path,required=True);parser.add_argument('--manifest',type=pathlib.Path);a=parser.parse_args();root=pathlib.Path(__file__).resolve().parents[1];manifest=a.manifest or root/'independent-engine/Cargo.toml'
expected={}
for kind,name in [('work','V46-bridge80-work.jsonl'),('reader','V46-bridge80-reader.jsonl')]:
 with(a.trace_dir/name).open()as f:
  header=json.loads(next(f))
  for line in f:
   r=json.loads(line);expected[kind,r['scenario'],r.get('tick',0)]=r
env=os.environ.copy();env['Z_NATIVE_TRACE_DIR']=str(a.trace_dir.resolve());env['Z_BRIDGE_DUMP']='1';count=draws=queue=0
proc=subprocess.Popen(['cargo','test','--manifest-path',str(manifest),'crane_bridge::tests::original_bridge80_','--','--nocapture','--test-threads=1'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env)
for line in proc.stdout:
 marker='BRIDGE80_RAW ';at=line.find(marker)
 if at<0:
  if any(w in line for w in ['FAILED','panicked','error[','error:']):print(line,end='')
  continue
 r=json.loads(line[at+len(marker):]);original=expected.pop((r['kind'],r['scenario'],r['tick']));want=zlib.decompress(base64.b64decode(original['after']));actual=bytes.fromhex(r['raw'])
 if actual!=want:
  diffs=[i for i,(x,y)in enumerate(zip(actual,want))if x!=y];raise AssertionError((r['kind'],r['scenario'],r['tick'],len(actual),len(want),diffs[:12]))
 if r['kind']=='work':assert r['queue']==original['queue'];queue+=len(r['queue'])//16
 else:assert r['draws']==original['draws'];draws+=len(r['draws'])
 count+=1
 if count%1024==0:print(f'Compared {count} entire original records',flush=True)
status=proc.wait();assert status==0,status;assert not expected,len(expected);assert count==5728,count
print(json.dumps({'cases':count,'allRecordedBytesMatch':True,'originalCraneQueueRows':queue,'originalLateDraws':draws}))
