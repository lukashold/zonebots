"""Verify a private campaign enrichment without exposing original resources."""
from pathlib import Path
import argparse,json,hashlib
p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--enriched',type=Path,required=True);p.add_argument('--manifest',type=Path);p.add_argument('--game',type=Path);a=p.parse_args()
before=json.loads(a.baseline.read_text(encoding='utf-8'));after=json.loads(a.enriched.read_text(encoding='utf-8'));fields=('nativeBridgeDisk26','nativeBridgePlaybackHeaders','nativeCraneBridgeFrames')
assert len(before['missions'])==len(after['missions'])==20
for mission in after['missions']:
 for field in fields:assert field in mission
for campaign in(before,after):
 for mission in campaign['missions']:
  for field in fields:mission.pop(field,None)
assert before==after,'Unrelated campaign fields changed'
if a.manifest:
 manifest=json.loads(a.manifest.read_text(encoding='utf-8'));assert manifest['sourceCampaignSha256']==hashlib.sha256(a.baseline.read_bytes()).hexdigest()
 if a.game:
  for row in manifest['sourceFiles']:
   raw=(a.game/row['file']).read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'],row['file']
print('PASS: only the three bridge80 fields differ across20 missions; optional source manifest matches.')
