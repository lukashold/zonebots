"""Decode locally owned Z artwork; output is excluded from Git. No executables are copied."""
from pathlib import Path
import argparse,struct,json,hashlib
from PIL import Image

def write_json(path,data,**options):
    pending=path.with_suffix(path.suffix+'.tmp')
    pending.write_text(json.dumps(data,**options),encoding='utf-8')
    pending.replace(path)

def executable_metadata(source):
    """Ghidra-verified tables from the installed GOG executable, without copying it."""
    exe=(source/'Z.exe').read_bytes();pe=struct.unpack_from('<I',exe,60)[0];sections=[]
    for i in range(struct.unpack_from('<H',exe,pe+6)[0]):
        offset=pe+24+struct.unpack_from('<H',exe,pe+20)[0]+i*40
        vs,va,rs,start=struct.unpack_from('<IIII',exe,offset+8);sections.append((va,rs,start))
    def memory(address,size):
        rva=address-0x400000
        for va,n,start in sections:
            if va<=rva and rva+size<=va+n:return exe[start+rva-va:start+rva-va+size]
        raise ValueError(hex(address))
    object_types=[]
    for i in range(78):
        d=memory(0x498f38+i*26,26)
        object_types.append(dict(sprite=struct.unpack_from('<H',d)[0],shadowSprite=struct.unpack_from('<h',d,22)[0],flags=d[18],depthOffset=struct.unpack_from('<b',d,19)[0],size=[d[20],d[21]]))
    buildings={};original_order=['DESERT','ARCTIC','CITY','VOLCAN','JUNGLE']
    offsets={'vehicle':[32,48],'robot':[16,64],'radar':[0,32],'repair':[0,48]}
    for kind,table,overlay_index,building_type in [('vehicle',0x4781bc,1,1),('robot',0x47818c,0,4),('radar',0x478770,2,8),('repair',0x47875c,3,0)]:
        buildings[kind]={}
        for i,name in enumerate(original_order):
            intact,dead=struct.unpack('<HH',memory(table+i*4,4));overlay=struct.unpack('<H',memory(0x4942d4+i*8+overlay_index*2,2))[0]
            stars=struct.unpack('<H',memory(0x49415c+i*2,2))[0] if kind in ['robot','vehicle'] else None
            stars_offset=list(struct.unpack('<hh',memory(0x478594+building_type*4,4))) if stars else None
            # Robot-factory destroyed drawing in Z.exe indexes this pair table with
            # world*8, including one out-of-range frame. Keep the matching world art.
            buildings[kind][name]=dict(intact=intact,destroyed=dead,overlay=overlay,overlayOffset=offsets[kind],stars=stars,starsOffset=stars_offset)
    def part(x,y,first,count=1,**extra):
        return dict(offset=[x,y],frames=list(range(first,first+count)),**extra)
    machinery={
        'vehicle':{'0040b550':part(16,48,2119,2),'0040b5a0':part(14,32,2109,4),'0040b5f0a':part(23,39,2113,2),'0040b5f0b':part(10,-1,2099,8),'0040b6b0a':part(13,47,2116),'0040b6b0b':part(42,47,2116)},
        'robot':{'0040bc20':part(16,48,2107,2),'0040bce0':part(10,-2,2091,8),'0040bd50':part(13,13,2872,12),'0040bde0':part(34,14,2903,2,repeat=6,step=[4,0]),'0040bf20':part(16,31,2122),'0040bf80':part(38,39,2897,6),'0040bfe0':part(26,68,2123,2,repeat=6,step=[-3,0])},
        'radar':{'0040c9b0':part(16,-18,1488,8,yDelta=list(struct.unpack('<8b',memory(0x4781a0,8)))),'0040c8a0':part(41,0,2891,2),'0040c930':part(16,22,2914,2),'0040ca70':part(18,14,2872,12)},
        'repair':{'0040cdf0':part(18,5,2905,2),'0040ce60':part(16,32,2907,3),'0040ced0':part(6,16,2910,2),'0040cda0':part(16,49,1890,4)}
    }
    def points(address,count):return [list(struct.unpack('<hh',memory(address+i*4,4))) for i in range(count)]
    # Specialized composite renderers override parts of the generic animation
    # table: notably APC uses eight directions with stride3 in00455af0.
    unit_layouts={
        #00455440 builds the Jeep from suspension/wheels, occupied body and an
        # independently aimed weapon. Body sprites already include the driver.
        'jeep':dict(renderer='00455440',anchor=[-6,-6],body=5590,stride=2,directions=8,underlay=[struct.unpack('<h',memory(0x496c04+i*2,2))[0] for i in range(8)],weapon=5639,bodyOffsets=points(0x496e0c,8),aimOffsets=points(0x496c84,8),weaponAnchor=[6,-3]),
        'medium':dict(renderer='00455c30',anchor=[-6,0],hull=488,damagedHull=500,stride=3,directions=4,hullOffsets=points(0x496dfc,4),turret=513,turretOffsets=points(0x496ba4,8),turretAnchor=[6,-3],barrel=521,barrelOffsets=points(0x496ca4,8)),
        # Heavy hull offset table004d1900 is zero-initialized BSS, unlike the
        # other literal tables. The renderer reads its four zero pairs.
        'heavy':dict(renderer='00455110',anchor=[-6,-6],hull=5305,stride=3,directions=4,hullOffsets=[[0,0]]*4,turret=5329,turretOffsets=points(0x496d24,8),turretAnchor=[6,-3]),
        'light':dict(renderer='00455290',anchor=[-6,-6],hull=5719,damagedHull=5747,stride=3,directions=4,hullOffsets=points(0x496c14,4),turret=5731,turretOffsets=points(0x496ce4,8),turretAnchor=[6,-3]),
        'apc':dict(renderer='00455af0',anchor=[-6,-6],hull=5655,stride=3,directions=8,hullOffsets=[[0,0]]*8,driver=5679,driverOffsets=points(0x496e2c,8)),
        'crane':dict(renderer='00455820',anchor=[-6,-6],hull=5794,stride=3,directions=8,hullOffsets=[[0,0]]*8,turret=5818,turretOffsets=points(0x496dbc,8),turretAnchor=[6,-3],arm=5826,armOffsets=points(0x496cc4,8))
    }
    neutral_indices={'medium':8,'heavy':26,'light':29,'apc':31,'jeep':33,'crane':35,'firetruck':37,'launcher':24,'cannon':5,'missile':21,'gatling':22,'howitzer':23}
    return dict(buildings=buildings,buildingMachinery=machinery,objectTypes=object_types,unitLayouts=unit_layouts,neutralUnits={name:object_types[i]['sprite'] for name,i in neutral_indices.items()})

def write_manifest(source,out):
    files=[dict(name=p.name,bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in out.iterdir() if p.is_file() and p.name!='manifest.json']
    write_json(out/'manifest.json',dict(source=str(source),files=files),indent=2)
    return files

def refresh_metadata(source,out):
    """Update the existing private atlas metadata, preserving artwork and maps."""
    path=out/'sprites.json';data=json.loads(path.read_text(encoding='utf-8'))
    data.update(executable_metadata(source))
    write_json(path,data,separators=(',',':'))
    write_manifest(source,out)
    print(json.dumps(dict(sprites=data['count'],metadataOnly=True)))

def extract(source,out):
    out.mkdir(parents=True,exist_ok=True)
    raw=(source/'en.lproj'/'sprites.rsc').read_bytes();count=struct.unpack_from('<H',raw)[0]
    frames=[];decoded=[];x=y=row=0;width=2048
    for i in range(count):
        off=struct.unpack_from('<I',raw,2+i*4)[0];h=raw[off];w=raw[off+1]*8
        if x+w>width:x=0;y+=row+2;row=0
        frames.append(dict(x=x,y=y,w=w,h=h));header=(2+h+3)&~3;q=w//4;pixels=bytearray(w*h)
        for plane in range(4):
            for yy in range(h):
                start=off+header+plane*q*h+yy*q
                pixels[yy*w+plane:yy*w+w:4]=raw[start:start+q]
        decoded.append(bytes(pixels));x+=w+2;row=max(row,h)
    atlas=Image.new('LA',(width,y+row+2),(0,0))
    for f,px in zip(frames,decoded):
        if not f['w'] or not f['h']:continue
        packed=bytearray(len(px)*2);packed[0::2]=px;packed[1::2]=bytes(255 if p else 0 for p in px)
        atlas.paste(Image.frombytes('LA',(f['w'],f['h']),bytes(packed)),(f['x'],f['y']))
    atlas.save(out/'sprites.png',optimize=True)
    worlds=['DESERT','VOLCAN','ARCTIC','JUNGLE','CITY'];palettes=[];tiles=Image.new('RGBA',(512,256*5))
    for world,name in enumerate(worlds):
        data=(source/(name+'.PAL')).read_bytes();palette=[min(255,p*4+(p>>4)) for p in data[:768]];palettes.append(palette)
        blk=(source/(name+'.BLK')).read_bytes()
        for i in range(512):
            pixels=blk[i*256:(i+1)*256];rgba=bytes(channel for p in pixels for channel in (*palette[p*3:p*3+3],255));tile=Image.frombytes('RGBA',(16,16),rgba)
            tiles.paste(tile,((i%32)*16,world*256+(i//32)*16))
    tiles.save(out/'tiles.png',optimize=True)
    metadata=dict(count=count,frames=frames,palettes=palettes,sourceSha256=hashlib.sha256(raw).hexdigest(),**executable_metadata(source))
    write_json(out/'sprites.json',metadata,separators=(',',':'))
    files=write_manifest(source,out)
    print(json.dumps(dict(sprites=count,atlas=atlas.size,bytes=sum(f['bytes']for f in files),files=files)))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source',type=Path,default=Path(r'C:\GOG Games\Z'));p.add_argument('--out',type=Path,required=True);p.add_argument('--metadata-only',action='store_true');args=p.parse_args()
    if args.metadata_only:refresh_metadata(args.source,args.out)
    else:extract(args.source,args.out)
