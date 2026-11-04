"""Read-only extraction of original Z campaign data. Output remains local/private.
All simulation coordinates are native game pixels multiplied by two (tileSize32).
Raw source records are kept separately from inferred engine compatibility fields.
"""
from pathlib import Path
from collections import deque,Counter
import argparse,struct,json,hashlib,os

GROUP={0:'cannon',1:'medium',2:'grunt',3:'tough',4:'sniper',5:'heavy',6:'psycho',7:'light',8:'missile',9:'launcher',10:'gatling',11:'jeep',12:'truck',13:'apc',14:'howitzer',15:'firetruck',16:'crane',17:'pyro',18:'laser'}
RAW={0:'cannon',1:'medium',2:'grunt',3:'tough',4:'sniper',5:'heavy',6:'psycho',7:'launcher',8:'light',9:'missile',10:'gatling',11:'jeep',12:'truck',13:'apc',14:'howitzer',15:'firetruck',16:'crane',17:'pyro',18:'laser'}
EQUIPMENT={0:'cannon',1:'medium',2:'grunt',3:'tough',4:'sniper',5:'heavy',6:'launcher',7:'psycho',8:'light',9:'missile',10:'apc',11:'gatling',12:'howitzer',13:'jeep',14:'firetruck',15:'crane',16:'truck',17:'pyro',18:'laser'}
ROBOTS={'grunt','psycho','tough','sniper','pyro','laser'}
GUNS={'cannon','missile','gatling','howitzer'}
WORLDS={'DESERT':0,'VOLCAN':1,'ARTIC':2,'ARCTIC':2,'JUNGLE':3,'CITY':4}
u16=lambda d,o:struct.unpack_from('<H',d,o)[0]
i16=lambda d,o:struct.unpack_from('<h',d,o)[0]
u32=lambda d,o:struct.unpack_from('<I',d,o)[0]
def text(d,o,n):return d[o:o+n].split(b'\0')[0].decode('latin1')
def records(d,size):return [(o,d[o:o+size])for o in range(0,len(d)-size+1,size)if u16(d,o)!=65535]

class Exe:
 def __init__(self,path):
  self.d=path.read_bytes();pe=u32(self.d,60);n=u16(self.d,pe+6);op=u16(self.d,pe+20);self.sections=[]
  for i in range(n):
   o=pe+24+op+i*40;vs,va,rs,raw=struct.unpack_from('<IIII',self.d,o+8);self.sections.append((va,max(vs,rs),raw,rs))
 def read(self,a,n):
  r=a-0x400000
  for va,size,raw,stored in self.sections:
   if va<=r and r+n<=va+size:
    offset=r-va;count=max(0,min(n,stored-offset))
    # The PE loader zero-fills virtual tails/BSS. They are not the unrelated
    # bytes following this section on disk and cannot supply static constants.
    return self.d[raw+offset:raw+offset+count]+bytes(n-count)
  raise ValueError(hex(a))
 def pair(self,a):return struct.unpack('<hh',self.read(a,4))
 def ptr(self,a):return u32(self.read(a,4),0)

def components(indices,w,h):
 left=set(indices);out=[]
 while left:
  start=min(left);left.remove(start);q=[start];cells=[]
  while q:
   j=q.pop();cells.append(j);x,y=j%w,j//w
   for nx,ny in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
    k=ny*w+nx
    if 0<=nx<w and 0<=ny<h and k in left:left.remove(k);q.append(k)
  out.append(cells)
 return out

def wall_tile_lookup(exe,native_world):
 """Recover the world's autotile table, not the unrelated permanent cliffs.

 FUN_0042e4c0 copies 520 bytes through 004939e4, then dereferences the
 16-by-four pointer matrix at 00494168. FUN_0042ef70 uses exposed-side bits
 north=1/east=2/south=4/west=8 and the first entry of each matrix row.
 Native world order differs from the browser atlas order.
 """
 source=exe.read(exe.ptr(0x4939e4+native_world*4),520)
 pair=lambda offset:u16(source,offset)
 tile=lambda value:(value&255)|((value&0x8000)>>7)
 heads=[tile(pair(exe.ptr(0x494168+mask*16)-0x5b6832))for mask in range(16)]
 # The front of certain concave corners uses a different source tile before
 # advancing one native 20-tile row per height step (FUN_0042ef70).
 remap={}
 for src,dst in [(0x74,0x62),(0x58,0x46),(0x5a,0x48),(0x50,0x3e),(0x56,0x44),(0x5c,0x4a)]:remap.setdefault(str(tile(pair(src))),tile(pair(dst)))
 return {'version':1,'headTiles':heads,'interiorTiles':[tile(pair(24+i*2))for i in range(16)],'frontRemap':remap,'frontStep':20,'rubbleTiles':[tile(pair(0xb2+min(i,5)*2))for i in range(8)],'rubbleCode':22,'nativeWorld':native_world,'source':'Z.exe FUN_0042e4c0/0042ef70/0042f580'}

def extract_walls(d,w,h,rawtiles,codes,tiles,lookup):
 """Decode the separate 20 x 138-byte destructible-wall bitmap layer.

 These records precede the visual map planes. The original generates their
 tiles at runtime (0042f1f0), so copying only the planes silently loses every
 destructible wall. The two-pixel scale also applies to these coordinates.
 """
 base_tiles=rawtiles[:];base_codes=codes[:];base_classes=tiles[:];walls=[]
 for record in range(20):
  offset=0x1a01+record*138;r=d[offset:offset+138]
  x0,y0=u16(r,0)//16,u16(r,2)//16;height=r[4];native_hp=struct.unpack_from('<b',r,5)[0]
  def bit(j):return 0<=j<1024 and bool(r[10+(j>>3)]&(0x80>>(j&7)))
  for j in range(1024):
   if not bit(j):continue
   x,y=x0+j%32,y0+j//32
   if not(0<=x<w and 0<=y<h):continue
   mask=15
   for delta,flag,allowed in [(-32,1,j>=32),(1,2,j%32!=31),(32,4,j<993),(-1,8,j%32!=0)]:
    if allowed and bit(j+delta):mask&=~flag
   # The original uses its shared RNG only for fully enclosed texture
   # variants. Keep this choice stable without pretending to recover its
   # state after all preceding engine calls; boundary shape is exact.
   variant=(record*7+j)&15;head=lookup['headTiles'][mask]if mask else lookup['interiorTiles'][variant]
   cells=[(y+dy)*w+x for dy in range(height+1)if y+dy<h]
   hp=max(1,native_hp) # Native zero-HP walls remain solid until any hit.
   walls.append({'id':len(walls),'x':x*32,'y':y*32,'w':32,'h':len(cells)*32,'hp':hp,'maxHp':hp,'cells':cells,'groundTiles':[base_tiles[k]for k in cells],'groundCodes':[base_codes[k]for k in cells],'groundClasses':[base_classes[k]for k in cells],'wallRecord':record,'wallBit':j,'height':height,'nativeHp':native_hp,'neighborMask':mask,'variant':variant,'wallTileIndex':head,'sourceOffset':offset,'sourceBit':j,'confidence':'exact_MAP_bitmap_xy_height_hp_and_EXE_boundary_autotiles;interior_variant_stable_adapter'})
   front=lookup['frontRemap'].get(str(head),head)
   for dy,k in enumerate(cells):
    rawtiles[k]=head if dy==0 else ((front&255)+lookup['frontStep']*dy)&255|(front&256)
    codes[k]=height if dy==0 else 18;tiles[k]=6
 return walls,base_tiles,base_codes,base_classes

def extract_bridges(data,w,h,tiles):
 """0045fef0 loads26-byte records;004097b0 initializes current HP from+10.

 Saved+8 is overwritten on mission start. Zero+10 means destroyed; the native
 initializer sets current HP=-1 and capacity=600. Browser HP0 means unavailable.
 """
 bridgebase=[-1]*(w*h);bridgehp=[];bridgecaps=[];bridges=[]
 for o,r in records(data,26):
  x,y,bw,bh,savedhp,initialhp=struct.unpack_from('<6H',r,0);cells=[]
  for ty in range(y//16,min(h,(y+bh+15)//16)):
   for tx in range(x//16,min(w,(x+bw+15)//16)):
    if tiles[ty*w+tx]==5:cells.append(ty*w+tx)
  if not cells:continue
  destroyed=initialhp==0;hp=0 if destroyed else initialhp;capacity=initialhp or 600
  bid=len(bridges)
  bridges.append({'id':bid,'x':x*2,'y':y*2,'w':bw*2,'h':bh*2,'hp':hp,'maxHp':capacity,'initiallyDestroyed':destroyed,'rawSavedHp':savedhp,'nativeSlot':o//26,'nativeOrientation':u16(r,12),'nativeFlags15':2 if destroyed else r[15],'tiles':cells,'sourceOffset':o,'confidence':'0045fef0_disk_to_runtime_and_004097b0_mission_initialization'})
  bridgehp.append(hp);bridgecaps.append(capacity)
  for j in cells:
   bridgebase[j]=bid
   if destroyed:tiles[j]=3
 return bridgebase,bridgehp,bridgecaps,bridges

def extract(root,out):
 robot=(root/'robots.dat').read_bytes();levels=(root/'levels.dat').read_bytes();exe=Exe(root/'Z.exe')
 groups=[]
 for gid in range(19):
  o=0x4650+gid*82;members=[]
  for k in range(21):
   raw=i16(robot,o+28+k*2)
   if raw<0:break
   members.append(raw)
  groups.append({'id':gid,'formation':robot[o+1],'type':GROUP[gid],'name':text(robot,o+2,25),'members':members,'descriptor48':robot[o+0x48],'buildLevels':{'vehicle':robot[o+73],'robot':robot[o+74],'fort':robot[o+75]}})
 missions=[]
 for number in range(1,21):
  nn=f'{number:02}';d=(root/f'LEVEL{nn}.MAP').read_bytes();w,h=u16(d,0x278d),u16(d,0x278f);n=w*h;world=text(d,0x2771,13).split('.')[0].upper()
  rawtiles=[];codes=[]
  for y in range(h):
   for x in range(w):
    a=d[0x2791+y*128+x];b=d[0x6791+y*128+x];rawtiles.append(a|((b&128)<<1));codes.append(b&127)
  # Compatibility colours/minimap classes; native movement reads terrainCodes.
  # Wet shore38 and water42 must not be collapsed to ordinary land. Native40
  # is deep water/lava and is distinguished from infantry water by the engine.
  tiles=[1 if c==7 else 2 if c in (19,43) else 3 if c in (9,38,39,40,42) else 5 if c==23 else 0 for c in codes]
  cp=(root/f'CPUPLR{nn}.DAT').read_bytes();sectors=[];territory=[-1]*n
  for o in range(0,min(len(cp),64*30),30):
   if i16(cp,o)<0:continue
   x1,y1,x2,y2=struct.unpack_from('<HHHH',cp,o);owner=cp[o+8];sid=len(sectors)
   sectors.append({'id':sid,'x':(cp[o+18]+.5)*32,'y':(cp[o+19]+.5)*32,'owner':owner if owner<4 else -1,'home':False,'rect':[x1,y1,x2,y2],'rawOwner':owner,'rawFlagTile':[cp[o+18],cp[o+19]],'nativeSlot':o//30,'nativeFlags':cp[o+24],'nativePopulation':struct.unpack_from('<b',cp,o+9)[0],'nativeScoringFlags':cp[o+29],'nativePrioritySeed':struct.unpack_from('<i',cp,o+25)[0],'sourceOffset':o})
   for y in range(max(0,y1),min(h,y2)):
    for x in range(max(0,x1),min(w,x2)):
     if territory[y*w+x]<0:territory[y*w+x]=sid
  lookup=[]
  for j,sid in enumerate(territory):
   tx,ty=j%w,j//w
   lookup.append(sid if sid>=0 else territory[(ty+1)*w+tx+1] if tx+1<w and ty+1<h else -1)
  def sec_at(tx,ty):return lookup[ty*w+tx] if 0<=tx<w and 0<=ty<h else -1
  # The program scans raw tile494 markers to set native fort bases (FUN_0045f860).
  # The visible footprint includes the non-494 gate tiles. Keep the first 494
  # marker separate: static group placement uses it, not the visible bbox.
  forts=[]
  fortcodes={16,20,21,30,62,63}
  for cells in components([j for j,t in enumerate(rawtiles)if t==494 or codes[j] in fortcodes],w,h):
   markers=sorted(j for j in cells if rawtiles[j]==494)
   if len(markers)<20:continue
   xs=[j%w for j in cells];ys=[j//w for j in cells];x0,y0,x1,y1=min(xs),min(ys),max(xs),max(ys)
   sid=sec_at((x0+x1)//2,(y0+y1)//2);team=sectors[sid]['owner'] if sid>=0 else -1
   north=[j for j in cells if codes[j]==62];south=[j for j in cells if codes[j]==63]
   gatecells=north or south or [j for j in cells if codes[j]==30]
   gate=[sum(j%w+.5 for j in gatecells)/len(gatecells)*32,sum(j//w+.5 for j in gatecells)/len(gatecells)*32] if gatecells else [(x0+x1+1)*16,(y1+1)*32]
   marker=markers[0]
   forts.append({'x':x0*32,'y':y0*32,'w':(x1-x0+1)*32,'h':(y1-y0+1)*32,'sector':sid,'team':team,'gate':gate,'orientation':'north'if north else 'south','rawBase':[marker%w*16,marker//w*16],'cells':cells})
  assert len(forts)==2,(number,len(forts))
  # Matching levels.dat entry (some files include noncampaign records before level01).
  li=next(i for i in range(len(levels)//240)if text(levels,i*240+20,13).lower()==f'level{nn}.map')
  lv=levels[li*240:(li+1)*240];buildingdata=(root/f'BUILD{nn}.DAT').read_bytes();buildings=[]
  for ft in sorted(forts,key=lambda f:f['team']):
   sid=ft['sector'];sectors[sid]['home']=True
   rawhp=u32(lv,0x80 if ft['team']==0 else 0x7c);hp=rawhp//9*9
   stars=i16(exe.read(0x4978ee+li*2,2),0)
   buildings.append({'id':len(buildings),'kind':'fort','team':ft['team'],'sector':sid,'x':ft['x'],'y':ft['y'],'w':ft['w'],'h':ft['h'],'gate':ft['gate'],'orientation':ft['orientation'],'hp':hp,'maxHp':hp,'rawHp':rawhp,'stars':stars,'starsEvidence':'FUN_0045fa10_i16_004978ee','product':None,'progress':0,'rawBase':ft['rawBase'],'confidence':{'position':'exact_tile494_and_fort_terrain_region','hp':'FUN_0045f860_floor_levels_hp_div9_times9','gate':'terrain_code62_or63','technology':'exact_exe_i16_table'}})
  skipped=[]
  for o,r in records(buildingdata,80):
   typ=r[4]
   if typ==255:skipped.append(o);continue
   if typ not in (0,1,4,8):raise ValueError((number,typ))
   tx,ty=u16(r,0)//16,u16(r,2)//16;kind={0:'repair',1:'vehicle',4:'robot',8:'radar'}[typ];wh=tuple(exe.read(0x498c78+typ*0x4c+0x30,2))
   sid=sec_at(tx+wh[0]//2,ty+wh[1]//2);owner=sectors[sid]['owner'] if sid>=0 else -1;prod=GROUP.get(r[7]);stars=u32(r,12)//1000
   buildings.append({'id':len(buildings),'kind':kind,'team':owner,'sector':sid,'x':u16(r,0)*2,'y':u16(r,2)*2,'w':wh[0]*32,'h':wh[1]*32,'hp':u16(r,0x1c),'maxHp':u16(r,0x1a),'stars':stars,'product':prod,'progress':0,'rawType':typ,'rawAttackOffset':list(struct.unpack('<bb',exe.read(0x498c78+typ*0x4c+0x32,2))),'rawProductId':r[7],'nativeFlags6':r[6],'rawTechnology':u32(r,12),'sourceOffset':o,'gate':[(tx+wh[0]/2)*32,(ty+wh[1])*32],'confidence':{'position':'exact_BUILD_native_xy','size':'exact_exe_descriptor_00498c78_plus30','owner':'territory_lookup','gate':'provisional_bottom_center_exit','progress':'not_recovered_initial_zero'}})
  bridgebase,bridgehp,bridgecaps,bridges=extract_bridges((root/f'BRIDGE{nn}.DAT').read_bytes(),w,h,tiles)
  # Object byte5 points into the hardware table, not the product/group table.
  initial_flag_count=sum(not sector["home"]for sector in sectors)
  objects=[];spawns=[];crates=[]
  for o,r in records((root/f'OBJECT{nn}.DAT').read_bytes(),10):
   x,y,typ,eq=struct.unpack_from('<HHBB',r)
   if typ==37:continue #00433700 skips authored Fire Truck objects
   template=exe.read(0x498f38+typ*26,26);flags=template[18]
   ob={'x':x*2,'y':y*2,'type':typ,'equipmentId':eq,'templateName':text(template,2,16),'templateFlags':flags,'templateSprite':u16(template,0),'sourceOffset':o};objects.append(ob)
   native_flags=0x70 if typ in (5,21,22,23,35,26,29,8,31,24,33,37) else 0 if typ in (11,12,17) else 0x80
   if flags&0x14:native_flags|=0x40
   ob.update({'nativeSlot':initial_flag_count+len(objects)-1,'nativeHp':0 if eq==255 else robot[0x55f0+eq*42+5],'nativeFlags13':native_flags,'templateExtent':struct.unpack_from('<b',template,20)[0],'templateHeight':template[21]})
   if flags==0x32 and eq!=255 and eq in EQUIPMENT:
    spawns.append({'type':EQUIPMENT[eq],'t':EQUIPMENT[eq],'team':-1,'x':x*2,'y':y*2,'squad':0,'source':'OBJECT_equipment','equipmentId':eq,'objectType':typ,'sourceOffset':o,'confidence':'equipment_and_hardware_flag_exact;anchor_needs_review'})
   if typ==10:
    crates.append({'id':len(crates),'x':x*2,'y':y*2,'taken':False,'type':'grenades','sourceOffset':o,'confidence':'exact_OBJECT_grenade_template'})
  #00424dc0 initializes the human half from levels.dat.00417040 independently
  # initializes all26 CPU slots from CPUPLR. Inactive gaps retain their slot IDs.
  human_starts=[(slot,gid)for slot,gid in enumerate(lv[0x55:0x61])if gid!=255]
  cpu_starts=[(slot,cp[0x792+slot*10])for slot in range(26)if cp[0x793+slot*10]&1]
  anchors=[]
  for team in (0,1):
   ft=next(f for f in forts if f['team']==team);fort_id=next(b['id'] for b in buildings if b['kind']=='fort' and b['team']==team);gx,gy=ft['rawBase'];table=exe.ptr((0x47aec0 if team==0 else 0x47aa98)+li*4);point_index=0;gun_index=0 if team==0 else 4
   for slot,gid in (human_starts if team==0 else cpu_starts):
    if gid>=len(groups):raise ValueError(f'Unknown active CPU group {gid} in mission {number},slot{slot}')
    group=groups[gid];typ=group['type'];squad=team*100+slot+1
    hardware=robot[group['members'][0]*44+34];placementclass=robot[0x55f0+hardware*42+1]
    if placementclass in (10,14) and len(group['members'])>1:
     raise NotImplementedError('Transport passenger creation needs hidden-state modelling; no current campaign group uses this branch')
    if typ in GUNS:
     seq=0x4982c8 if team==0 else 0x4981a8;label=exe.read(seq+li*7+gun_index,1)[0];mapped=exe.read(0x498123+label,1)[0];rx,ry=exe.pair(0x479bb4+(mapped-65)*4);gun_index+=1
     if typ!='cannon':rx-=2;ry-=8
     base=(gx+rx,gy+ry);anchorconfidence='exe_exact_gun_anchor'
    elif team==1:
     point_index+=1
     base=(u16(cp,0x794+slot*10),u16(cp,0x796+slot*10));anchorconfidence='CPUPLR_exact_slot_anchor'
    else:
     point_index+=1
     rx,ry=exe.pair(table+point_index*4);base=(gx+rx,gy+ry);anchorconfidence='exe_static_group_anchor'
    mount={'fortId':fort_id} if typ in GUNS else {}
    if team==1 and typ in GUNS:
     # Native gun construction overruns its six-letter socket row for some
     # supplied CPU rosters (M5 yields nativeY16928). Authored CPUPLR positions
     # are an explicit compatibility correction, not a native-parity claim.
     base=(u16(cp,0x794+slot*10),u16(cp,0x796+slot*10));mount={}
     dx,dy=(0,0)if typ=='cannon'else(-2,-8)
     sockets=[(gx+sx+dx,gy+sy+dy)for sx,sy in [(17,170),(113,1),(113,170),(17,1),(113,49),(17,49)]]
     roof=min(sockets,key=lambda p:(p[0]-base[0])**2+(p[1]-base[1])**2)
     if (roof[0]-base[0])**2+(roof[1]-base[1])**2<=12**2:base=roof;mount={'fortId':fort_id}
     anchorconfidence='native_CPUPLR_gun_anchor_compatibility'
    anchors.append({'team':team,'slot':slot,'groupId':gid,'type':typ,'x':base[0]*2,'y':base[1]*2,'rawMembers':group['members'],'confidence':anchorconfidence,**({'nativeAiTaskSector':cp[0x799+slot*10]-1 if cp[0x799+slot*10] else None}if team==1 else {}),**mount})
    for member,raw in enumerate(group['members']):
     # FUN_0041aec0 ASM 0041af08..0041af22: the decompiler drops X +=
     # (member & 3) << 4, but retains only Y += member & 0xfc.
     dx,dy=(member&3)*16,member&0xfc
     spawns.append({'type':RAW[raw],'t':RAW[raw],'team':team,'x':(base[0]+dx)*2,'y':(base[1]+dy)*2,'squad':squad,'rawRobotId':raw,'groupId':gid,'formation':group['formation'],'source':'levels.dat_group','member':member,'memberOffsetNative':[dx,dy],'confidence':anchorconfidence+';member_offsets_exact_0041af08_0041af22',**mount})
  outside=len(sectors);sectors.append({'id':outside,'x':-1024,'y':-1024,'owner':-1,'home':True,'outside':True,'countsForProduction':False})
  compatible=[outside if sid<0 else sid for sid in lookup]
  wall_lookup=wall_tile_lookup(exe,lv[0x6e])
  walls,wall_ground_tiles,wall_ground_codes,wall_ground_classes=extract_walls(d,w,h,rawtiles,codes,tiles,wall_lookup)
  data={'number':number,'name':text(lv,0,20),'map':{'width':w,'height':h,'tileSize':32,'world':WORLDS[world],'worldName':world,'tiles':tiles,'territory':compatible,'originalTerritory':territory,'lookupTerritory':lookup,'originalTiles':rawtiles,'terrainCodes':codes,'bridgeBase':bridgebase,'bridgeHp':bridgehp,'seed':number,'revision':0,'original':True},'sectors':sectors,'buildings':buildings,'spawns':spawns,'startingGroups':anchors,'nativeGroupDescriptors48':[g['descriptor48'] for g in groups],'originalObjects':objects,'bridges':bridges,'crates':crates,'sourceFiles':{'map':f'LEVEL{nn}.MAP','build':f'BUILD{nn}.DAT','objects':f'OBJECT{nn}.DAT','cpu':f'CPUPLR{nn}.DAT','bridge':f'BRIDGE{nn}.DAT'},'diagnostics':{'inactiveBuildRecordsSkipped':skipped,'outsideTiles':territory.count(-1),'outsideLookupTiles':lookup.count(-1),'terrainCodes':dict(Counter(codes)),'unsupportedUnitTypes':sorted({s['type']for s in spawns} - set(RAW.values()))}}
  #004170a0 floods the entire native128x128 plane, including the border beyond
  #the displayed dimensions. Cropping it changes enclosed-sector eligibility.
  terrain_plane=list(d[0x6791:0x6791+128*128])
  for y in range(h):terrain_plane[y*128:y*128+w]=[((rawtiles[y*w+x]&256)>>1)|codes[y*w+x] for x in range(w)]
  data['map']['nativeTerrainPlane']=terrain_plane
  data['map']['nativeLevelRecord']=li
  data['nativeAi']={'levelRecordIndex':li,'decisionControl':cp[0x780],'reservePercent':25,'rawReservePercent':cp[0x781],'populationWeight':cp[0x89f],'terrainPlane':terrain_plane,'initialSectorTasks':[{'slot':slot,'groupId':gid,'sector':cp[0x799+slot*10]-1 if cp[0x799+slot*10] else None}for slot,gid in cpu_starts],'source':'CPUPLR ->00417040/00416b70/00417b30/00419fb0','strategy':'native_clock_with_browser_sector_adapter'}
  #417400 reads all64 CP records, including inactive slots and raw owners.
  data['nativeAi']['sectorRecords']=[{'rect':list(struct.unpack_from('<4h',cp,i*30)),'owner':cp[i*30+8]} for i in range(64)]
  data['walls']=walls
  data['map']['bridgeMaxHp']=bridgecaps
  # Version-9 MAPs already contain the96-record authored native road graph.
  # Keep bounds/packed waypoints and the96-byte next-hop rows verbatim.
  road_records=d[0xa971:0xdc71]
  assert len(road_records)==96*0x88
  data['map']['nativeRoadGraph']={'regions':[{'bounds':list(struct.unpack_from('<4h',road_records,i*0x88)),'waypoint':u16(road_records,i*0x88+8),'next':list(road_records[i*0x88+0x28:(i+1)*0x88])}for i in range(96)]}
  native_buildings=[]
  for slot in range(35):
   r=buildingdata[slot*80:(slot+1)*80];assert len(r)==80
   bid=next((b['id']for b in buildings if b.get('sourceOffset')==slot*80),None)
   native_buildings.append({'nativeSlot':slot,'flags6':r[6],'nativeWord0A':i16(r,10),'nativeByte15':r[21],'kind':-1 if r[4]==255 else r[4],'x':i16(r,0),'y':i16(r,2),'hp':i16(r,28),'maxHp':i16(r,26),'buildingId':bid,'fortProxy':False})
  #0045fa10 fills the first two unused BUILD slots with fort proxies. These
  #hold the pre-constructor HP;40b250 subsequently copies the global fort HP.
  for ft in forts:
   record=next(b for b in native_buildings if b['kind']==-1)
   record.update({'kind':2,'x':ft['rawBase'][0],'y':ft['rawBase'][1]+16,'buildingId':next(b['id']for b in buildings if b['kind']=='fort'and b['team']==ft['team']),'fortProxy':True})
  data['nativeWorldBuildings']=native_buildings
  #41f8f0 ordinary BUILD tables;420ea0 fort uses separate per-row timing.
  fragment_tables={}
  for kind,address in [(0,0x478420),(1,0x4782a0),(4,0x4785b8),(8,0x478798)]:
   rows=[]
   for i in range(50):
    row=list(struct.unpack('<6h',exe.read(address+i*12,12)))
    if row[0]==-1:break
    rows.append(row)
   fragment_tables[kind]=rows
  data['nativeBuildingFragments']=[{'nativeSlot':b['nativeSlot'],'templates':fragment_tables[b['kind']]} for b in native_buildings if b['kind'] in fragment_tables]
  fort_fragments=[]
  for i in range(50):
   row=list(struct.unpack('<6h',exe.read(0x47b5f0+i*12,12)))
   if row[0]==-1:break
   fort_fragments.append(row)
  data['nativeFortFragments']={'templates':fort_fragments}


  #428d20 sets622db8=2;17b30 maps18-(18>>3)-(18>>2) to12.
  #4600bd defaults the cap to50 before loading CPUPLR;10570 reads the
  #loaded CP+8a8 byte with MOVSX, independently of the selected product.
  data['nativeProduction']={'sourceInitializationMode':2,'factor':12,'teamUnitLimit':cp[0x8a8]}
  data['map'].update({'wallTileLookup':wall_lookup,'wallCells':sorted({cell for wall in walls for cell in wall['cells']}),'wallGroundTiles':wall_ground_tiles,'wallGroundCodes':wall_ground_codes,'wallGroundClasses':wall_ground_classes,'nativeTilePlane':list(d[0x2791:0x6791])})
  data['diagnostics']['wallRecords']=len({wall['wallRecord']for wall in walls})
  data['diagnostics']['wallPieces']=len(walls)
  missions.append(data)
 result={'version':8,'sourceExeSha256':hashlib.sha256(exe.d).hexdigest(),'coordinateScale':2,'groups':groups,'missions':missions,'limitations':['CPU fixed-gun coordinates use authored CPUPLR positions with explicit roof alignment compatibility; native construction overruns its six-letter socket table for several supplied rosters.','Initial member positions, five formation masks, 240-slot route ownership, finite 4800/9600 linked pools, the round-robin scheduler and incremental contour workers are recovered; complete global-road assembly, dynamic UNIT overlays and coupled whole-match equivalence remain to be compared.','Transport passenger hidden-state branch is not emitted; no current campaign starting group has transport passengers.','Neutral and occupied hardware constructors, equipment and origin offsets are recovered and tested; browser entity IDs are retained for selection/network compatibility.','Native factory door, exit/protection callbacks and completion stock are recovered; rendering and browser interaction use doubled native coordinates.','Wall positions, height, hit points, exposed-side shapes and world graphics recovered from MAP/EXE. Interior texture variants use a stable adapter instead of unrecovered shared RNG state. Native auxiliary right-edge shadow adjustments are not yet reconstructed.','Terrain visual tile indices exact apart from documented wall variants; compatibility classes serve rendering/minimap and native collision reads terrainCodes.','Outside filler sector must be excluded from production territorial totals.']}
 out.mkdir(parents=True,exist_ok=True);(out/'campaign.json').write_text(json.dumps(result,separators=(',',':')),encoding='utf-8')
 (out/'summary.json').write_text(json.dumps([{'level':m['number'],'name':m['name'],'width':m['map']['width'],'height':m['map']['height'],'sectors':len(m['sectors'])-1,'buildings':len(m['buildings']),'spawns':len(m['spawns']),'walls':len(m['walls']),'unsupported':m['diagnostics']['unsupportedUnitTypes']}for m in missions],indent=2),encoding='utf-8')
 print(f'Extracted {len(missions)} missions to {out / "campaign.json"}')

if __name__=='__main__':
 p=argparse.ArgumentParser(description='Extract private campaign data from your local GOG installation. Output is ignored by Git.')
 p.add_argument('--game','--source',dest='game',type=Path,default=Path(os.environ.get('Z_GAME_DIR',r'C:\GOG Games\Z')),help='Game directory; defaults to Z_GAME_DIR or C:\\GOG Games\\Z')
 p.add_argument('--output','--out',dest='output',type=Path,default=Path(__file__).resolve().parents[1]/'dist'/'original',help='Private output directory, default: repository dist/original')
 a=p.parse_args();extract(a.game.expanduser().resolve(),a.output.expanduser().resolve())
