"""Prepare privately owned Z recordings for browsers; output stays Git-ignored.

RAW is unsigned 8-bit mono PCM. GOG Z.exe 0046dcb0 reads the sample frequency
from the sound descriptor (+0x16), doubles its samples and opens SDL at 22050 Hz
(0046de20). The source descriptors use 11025 Hz. WAV preserves the source PCM.
The optional FFmpeg conversion adds an MP3 alternate to the original Ogg music.
"""
from pathlib import Path
import argparse,hashlib,json,os,shutil,struct,subprocess,wave

MUSIC=['AA16','aC16','aJ16','ipBATTLE16','ipOPTIONS16','ipWIN','ipLOSE']

def locate_ffmpeg(explicit):
    if explicit:return str(Path(explicit).expanduser().resolve())
    found=shutil.which('ffmpeg')
    if found:return found
    try:
        import imageio_ffmpeg
        return imageio_ffmpeg.get_ffmpeg_exe()
    except ImportError:return None

def descriptor(exe,name):
    needle=name.encode('ascii')+b'\0';offset=exe.find(needle)
    if offset<4:return {'sampleRate':11025,'gain':1.0}
    rate=struct.unpack_from('<H',exe,offset+18)[0]
    volume=struct.unpack_from('<H',exe,offset+20)[0]
    if rate<4000 or rate>96000:rate=11025
    return {'sampleRate':rate,'gain':min(1.0,volume/127)}

def extract(source,out,ffmpeg=None):
    source=source.expanduser().resolve();out=out.expanduser().resolve()
    exe=(source/'Z.exe').read_bytes();audio=out/'audio';audio.mkdir(parents=True,exist_ok=True)
    manifest={'version':1,'sourceExeSha256':hashlib.sha256(exe).hexdigest(),
              'format':'PCM unsigned 8-bit mono at descriptor sample frequency, wrapped in WAV',
              'voices':{},'effects':{},'music':{},'worldMusic':['AA16','aC16','AA16','aJ16','aC16']}
    files=[]
    def record(path,origin):
        files.append({'path':str(path.relative_to(out)).replace('\\','/'),'source':str(origin.relative_to(source)).replace('\\','/'),
                      'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    def pcm(path,destination):
        data=path.read_bytes();info=descriptor(exe,path.stem.upper());destination.parent.mkdir(parents=True,exist_ok=True)
        with wave.open(str(destination),'wb') as w:
            w.setnchannels(1);w.setsampwidth(1);w.setframerate(info['sampleRate']);w.writeframes(data)
        record(destination,path)
        return {'url':str(destination.relative_to(out)).replace('\\','/'),'duration':len(data)/info['sampleRate'],**info}
    for lang in ['de','en']:
        folder=source/(lang+'.lproj');clips={}
        for path in sorted(folder.glob('*.RAW')):
            if not (path.stem.upper().startswith('ROB') or path.stem.upper().startswith('COMP')):continue
            clips[path.stem.upper()]=pcm(path,audio/'voice'/lang/(path.stem.upper()+'.wav'))
        if clips:manifest['voices'][lang]=clips
    for path in sorted((source/'audio').glob('*.RAW')):
        manifest['effects'][path.stem.upper()]=pcm(path,audio/'effects'/(path.stem.upper()+'.wav'))
    encoder=locate_ffmpeg(ffmpeg)
    for stem in MUSIC:
        original=source/(stem+'.ogg')
        if not original.is_file():continue
        destination=audio/'music'/(stem+'.ogg');destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(original,destination);record(destination,original)
        alternatives=[]
        if encoder:
            mp3=destination.with_suffix('.mp3')
            subprocess.run([encoder,'-hide_banner','-loglevel','error','-nostdin','-y','-i',str(original),
                            '-vn','-codec:a','libmp3lame','-q:a','3','-map_metadata','-1',str(mp3)],check=True)
            alternatives.append({'url':str(mp3.relative_to(out)).replace('\\','/'),'type':'audio/mpeg'})
            record(mp3,original)
        alternatives.append({'url':str(destination.relative_to(out)).replace('\\','/'),'type':'audio/ogg; codecs="vorbis"'})
        manifest['music'][stem]=alternatives
    manifest['files']=files
    pending=out/'audio.json.tmp';pending.write_text(json.dumps(manifest,separators=(',',':')),encoding='utf-8');pending.replace(out/'audio.json')
    print(json.dumps({'voices':{k:len(v)for k,v in manifest['voices'].items()},'effects':len(manifest['effects']),
                      'musicTracks':len(manifest['music']),'mp3Alternates':bool(encoder),'bytes':sum(f['bytes']for f in files)}))
    return manifest

if __name__=='__main__':
    root=Path(__file__).resolve().parents[1]
    p=argparse.ArgumentParser();p.add_argument('--source','--game',type=Path,default=Path(os.environ.get('Z_GAME_DIR',r'C:\GOG Games\Z')))
    p.add_argument('--out','--output',type=Path,default=root/'dist/original');p.add_argument('--ffmpeg')
    args=p.parse_args();extract(args.source,args.out,args.ffmpeg)
