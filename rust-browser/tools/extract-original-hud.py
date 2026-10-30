"""Extract the locally owned, indexed original status panel. No original bytes in source."""
from pathlib import Path
import argparse
import hashlib
import json
import struct
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
    heads,head_sha=decode(game/'SHEADBI0.DAT')
    # Keep source group indices:0..130 panel,131..136 backgrounds,137..176 head parts.
    images=frames+backgrounds+heads
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
    manifest=dict(version=1,atlas='hud/atlas.png',frames=metadata,palettes=palettes,
        panelCount=len(frames),backgroundStart=len(frames),headStart=len(frames)+len(backgrounds),
        sources=[dict(path='de.lproj/NSNEWPAN.DAT',sha256=sha),dict(path='SHEADBAK.DAT',sha256=bg_sha),dict(path='SHEADBI0.DAT',sha256=head_sha),*palette_sources],
        evidence=dict(loader='004620b0',decoder='00461ff0/00461f00/00461e00',palette='00465620',robotCaption='00468fc0'),
        notes=['Original indexed bitmap pixels, dimensions and world palettes; responsive DOM placement is a web adaptation.', 'Static Grunt communicator portrait; original speech-driven facial state machine is not reconstructed.'])
    (output/'hud').mkdir(parents=True,exist_ok=True)
    atlas.save(output/'hud'/'atlas.png',optimize=True)
    (output/'hud.json').write_text(json.dumps(manifest,separators=(',',':'))+'\n',encoding='utf-8')
    print(f'Original HUD: {len(images)} indexed bitmaps, {atlas.width}x{atlas.height}; private output {output}')


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--game',type=Path,default=Path(r'C:\GOG Games\Z'));parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();extract(args.game,args.output)
