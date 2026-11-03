"""Extract the locally owned, indexed original status panel. No original bytes in source."""
from pathlib import Path
import argparse
import hashlib
import json
import struct
import importlib.util
from PIL import Image


def decode(path):
    data=path.read_bytes()
    _,count,offset=struct.unpack_from('<3H',data)
    # The final entry's unused word overlaps the first two bitmap bytes.
    if offset!=4+count*8:raise ValueError(f'Invalid panel header: {path}')
    images=[]
    for index in range(count):
        height,width=struct.unpack_from('<HH',data,6+index*8)
        length=width*height
        if not width or not height or offset+length>len(data):raise ValueError(f'Invalid panel frame {index}')
        pixels=data[offset:offset+length];offset+=length
        #00465620: small textures use transparent index0; large panel strips are opaque.
        opaque=width>256 or height>256
        packed=bytearray(length*2);packed[0::2]=pixels
        packed[1::2]=bytes(255 if value or opaque else 0 for value in pixels)
        images.append(Image.frombytes('LA',(width,height),bytes(packed)))
    if offset!=len(data):raise ValueError(f'Unexpected trailing panel data: {path}')
    return images,hashlib.sha256(data).hexdigest()


def extract(game,output):
    source=game/'de.lproj'/'NSNEWPAN.DAT'
    frames,sha=decode(source)
    backgrounds,bg_sha=decode(game/'SHEADBAK.DAT')
    head_sets=[decode(game/f'SHEADBI{i}.DAT') for i in range(5)]
    #004620b0 loads all five distinct head sets after the panel/backgrounds.
    images=frames+backgrounds
    head_starts=[]
    for heads,_ in head_sets:
        head_starts.append(len(images));images+=heads
    atlas=Image.new('LA',(1024,2048),(0,0));metadata=[];x=y=row=0
    for image in images:
        width,height=image.size
        if x+width>atlas.width:x=0;y+=row+2;row=0
        atlas.paste(image,(x,y));metadata.append(dict(x=x,y=y,w=width,h=height))
        x+=width+2;row=max(row,height)
    atlas=atlas.crop((0,0,1024,y+row+2))
    palettes=[];palette_sources=[]
    for world in ['DESERT','VOLCAN','ARCTIC','JUNGLE','CITY']:
        data=(game/(world+'.PAL')).read_bytes()
        palettes.append([min(255,value*4+(value>>4)) for value in data[:768]])
        palette_sources.append(dict(path=world+'.PAL',sha256=hashlib.sha256(data).hexdigest()))
    spec=importlib.util.spec_from_file_location('z_campaign_reader',Path(__file__).with_name('extract-original-campaign.py'))
    reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)
    exe=reader.Exe(game/'Z.exe')
    pairs=lambda a,n:[list(struct.unpack('<hh',exe.read(a+i*4,4))) for i in range(n)]
    phrase_bytes=(game/'PHRASES.BIN').read_bytes()
    if len(phrase_bytes)!=64*0x228:raise ValueError('Unexpected PHRASES.BIN record layout')
    phrases=[]
    for i in range(64):
        length=struct.unpack_from('<I',phrase_bytes,i*0x228+4)[0]
        if length>256:raise ValueError('Invalid phrase face-frame count')
        phrases.append(list(struct.unpack_from('<'+'H'*length,phrase_bytes,i*0x228+0x28)))
    #46a610 adds127 to the voice-table ID before playing the sound descriptor.
    voice_phrases={}
    for i in range(74):
        phrase,sound,_,reserved=exe.read(0x4988f0+i*4,4)
        if reserved or phrase>=64:raise ValueError('Unexpected original voice-face table')
        if not 1<=sound<=95:continue
        for path in (game/'de.lproj').glob('ROB*.RAW'):
            at=exe.d.find(path.stem.encode('ascii')+b'\0')
            if at>=2 and struct.unpack_from('<H',exe.d,at-2)[0]==sound+127:voice_phrases[path.stem.upper()]=phrase
    face=dict(headStarts=head_starts,headKinds={'tough':0,'pyro':1,'laser':1,'grunt':2,'psycho':3,'sniper':4},
        eyes=pairs(0x49c6ac,5),mouth=pairs(0x49c620,5),collar=pairs(0x49cc2c,5),overlay=pairs(0x49cc18,5),special=pairs(0x49ccf8,16),
        specialFrames=list(struct.unpack('<16H',exe.read(0x49ccd4,32))),phrases=phrases,voicePhrases=voice_phrases,
        defaultWord=15,clip=[46,32],source='00468c80/00468d30/0046a650/0046ad10/0046ad70')
    manifest=dict(version=2,atlas='hud/atlas.png',frames=metadata,palettes=palettes,face=face,
        panelCount=len(frames),backgroundStart=len(frames),headStart=len(frames)+len(backgrounds),
        sources=[dict(path='de.lproj/NSNEWPAN.DAT',sha256=sha),dict(path='SHEADBAK.DAT',sha256=bg_sha),*[dict(path=f'SHEADBI{i}.DAT',sha256=value[1]) for i,value in enumerate(head_sets)],dict(path='PHRASES.BIN',sha256=hashlib.sha256(phrase_bytes).hexdigest()),*palette_sources],
        evidence=dict(loader='004620b0',decoder='00461ff0/00461f00/00461e00',palette='00465620',robotCaption='00468fc0'),
        notes=['Original indexed bitmap pixels, all five head sets, native face composition coordinates and PHRASES sequences. Responsive DOM placement and the browser voice queue are adapters.'])
    (output/'hud').mkdir(parents=True,exist_ok=True)
    atlas.save(output/'hud'/'atlas.png',optimize=True)
    (output/'hud.json').write_text(json.dumps(manifest,separators=(',',':'))+'\n',encoding='utf-8')
    print(f'Original HUD: {len(images)} indexed bitmaps, {atlas.width}x{atlas.height}; private output {output}')


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--game',type=Path,default=Path(r'C:\GOG Games\Z'));parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();extract(args.game,args.output)
