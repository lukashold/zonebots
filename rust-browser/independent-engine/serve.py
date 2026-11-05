from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from functools import partial
import mimetypes, argparse, webbrowser, os, subprocess, sys, json, socket, re
from urllib.parse import unquote,urlsplit
root = Path(__file__).resolve().parents[1] / 'dist' / 'independent'
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('audio/wav', '.wav')
mimetypes.add_type('audio/ogg', '.ogg')
mimetypes.add_type('audio/mpeg', '.mp3')

def has_bridge80_inputs(campaign):
    """Older v8 packs are valid, but need this specific local enrichment."""
    try:return _bridge80_inputs_valid(campaign)
    except (AttributeError,TypeError,ValueError):return False

def _bridge80_inputs_valid(campaign):
    missions=campaign.get('missions',[])
    byte=lambda n:isinstance(n,int) and not isinstance(n,bool) and 0<=n<=255
    pointers=[0x647210,0x63d1f8,0x6421f8,0x63aa18,0x649a48,0x63f9e0,0x638238,0x6449f0]
    if len(missions)!=20:return False
    for mission in missions:
        disk=mission.get('nativeBridgeDisk26',[]);headers=mission.get('nativeBridgePlaybackHeaders',[]);frames=mission.get('nativeCraneBridgeFrames',[])
        if len(disk)!=12 or any(len(r)!=26 or not all(map(byte,r)) for r in disk):return False
        if len(headers)!=8 or any(r.get('pointer')!=p or len(r.get('bytes',[]))!=1280 or not all(map(byte,r['bytes'])) for r,p in zip(headers,pointers)):return False
        if len(frames)!=4:return False
        for direction,sequence in enumerate(frames):
            if len(sequence)!=12:return False
            if any(len(rows)!=(6 if direction%2==0 else 4) or any(len(r)!=4 or any(not isinstance(n,int) or isinstance(n,bool) or not -32768<=n<=32767 for n in r)for r in rows)for rows in sequence):return False
    return True

def ensure_original_pack(game):
    """Create missing private data locally; no original files enter the repository."""
    repo = Path(__file__).resolve().parents[1]
    private = repo / 'dist' / 'original'
    jobs = []
    campaign_path = private / 'campaign.json'
    try:
        campaign = json.loads(campaign_path.read_text(encoding='utf-8'))
        campaign_current = campaign.get('version', 0) >= 8
        bridge_current = campaign_current and has_bridge80_inputs(campaign)
    except (OSError, ValueError, AttributeError, TypeError):
        campaign_current = False
    if not campaign_current:
        jobs.append(('campaign', repo / 'tools' / 'extract-original-campaign.py',
                     ['--game', str(game), '--output', str(private)]))
    elif not bridge_current:
        jobs.append(('bridge repair inputs', repo / 'tools' / 'extract-original-campaign.py',
                     ['--game', str(game), '--output', str(private), '--enrich-bridge80']))
    if not all((private / name).is_file() for name in ('sprites.png', 'sprites.json', 'tiles.png')):
        jobs.append(('artwork', repo / 'tools' / 'extract-original-art.py',
                     ['--source', str(game), '--out', str(private)]))
    elif (game / 'PNG' / 'DESERT2.png').is_file():
        try:
            colors = json.loads((private / 'sprites.json').read_text(encoding='utf-8')).get('fortResultPalettes', [])
            result_palette_current = len(colors) == 5 and all(len(row) == 63 for row in colors)
        except (OSError, ValueError, TypeError, AttributeError):
            result_palette_current = False
        if not result_palette_current:
            jobs.append(('fort result palette', repo / 'tools' / 'extract-original-art.py',
                         ['--source', str(game), '--out', str(private), '--metadata-only']))
    if not (private / 'audio.json').is_file():
        jobs.append(('audio', repo / 'tools' / 'extract-original-audio.py',
                     ['--source', str(game), '--out', str(private)]))
    if not (private / 'menu.json').is_file() and (game / 'PNG' / 'IPBackground.png').is_file():
        jobs.append(('menu', repo / 'tools' / 'extract-original-menu.py',
                     ['--game', str(game), '--output', str(private)]))
    try:
        hud_current = json.loads((private / 'hud.json').read_text(encoding='utf-8')).get('version', 0) >= 2
    except (OSError, ValueError, AttributeError, TypeError):
        hud_current = False
    if not hud_current and (game / 'de.lproj' / 'NSNEWPAN.DAT').is_file():
        jobs.append(('HUD', repo / 'tools' / 'extract-original-hud.py',
                     ['--game', str(game), '--output', str(private)]))
    for label, script, arguments in jobs:
        print(f'Extracting original {label} from {game} ...', flush=True)
        try:
            subprocess.run([sys.executable, str(script), *arguments], check=True)
        except subprocess.CalledProcessError as error:
            raise SystemExit(
                f'Original {label} extraction failed. Use --game with your local Z game directory. '
                'Artwork extraction also requires Pillow (python -m pip install Pillow).'
            ) from error

class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, films=None, result_art=None, **kwargs):
        self.films = films
        self.result_art = result_art
        super().__init__(*args, **kwargs)

    def translate_path(self,url):
        path=unquote(urlsplit(url).path)
        if path.startswith('/original/films/'):
            name=path.removeprefix('/original/films/')
            if self.films is None or not (name=='films.json' or re.fullmatch(r'e_(?:outro|[dvajc](?:lost1|wint[1-3]|winpl))\.(?:zvf\.gz|ogg)',name)):
                return str(root/'missing')
            target=(self.films/name).resolve()
            return str(target) if target.is_relative_to(self.films) else str(root/'missing')
        if path.startswith('/original/result-view/'):
            name=path.removeprefix('/original/result-view/')
            if self.result_art is None or name not in {
                'manifest.json','background.png','panel.png','divider.png','buttons.png','medium.png','large.png',
            }:
                return str(root/'missing')
            target=(self.result_art/name).resolve()
            return str(target) if target.is_relative_to(self.result_art) else str(root/'missing')
        if path.startswith('/original/'):
            base=root.parent/'original';target=(base/path[len('/original/'):]).resolve()
            return str(target) if target.is_relative_to(base.resolve()) else str(root/'missing')
        return super().translate_path(url)

if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8765);p.add_argument('--no-browser',action='store_true')
    p.add_argument('--lan',action='store_true',help='Allow other devices on your local network to load this game; binds all local interfaces')
    p.add_argument('--films',type=Path,help='Local result film pack for a loopback game')
    p.add_argument('--result-art',type=Path,help='Local classic result textures for a loopback game')
    p.add_argument('--game','--zdir',dest='game',type=Path,default=Path(os.environ.get('Z_GAME_DIR',r'C:\GOG Games\Z')),help='Local Z installation for automatic extraction of missing original art/maps')
    args=p.parse_args()
    films=args.films.expanduser().resolve() if args.films else None
    result_art=args.result_art.expanduser().resolve() if args.result_art else None
    if films is not None and (args.lan or not (films/'films.json').is_file()):
        p.error('--films requires a film pack with films.json and a local loopback game')
    if result_art is not None and (args.lan or not (result_art/'manifest.json').is_file()):
        p.error('--result-art requires the extracted result art and a local loopback game')
    ensure_original_pack(args.game.expanduser().resolve())
    bind='0.0.0.0' if args.lan else '127.0.0.1'
    server=ThreadingHTTPServer((bind,args.port),partial(Handler,directory=str(root),films=films,result_art=result_art))
    port=server.server_port
    options=[flag for flag,active in [('original-films=1',films),('original-result-art=1',result_art)] if active]
    url=f'http://127.0.0.1:{port}/' + ('?'+'&'.join(options) if options else '');print(url,flush=True)
    if args.lan:
        print('LAN mode: other devices on this network can load this game and its local resource pack.',flush=True)
        try:
            addresses=sorted({address for address in socket.gethostbyname_ex(socket.gethostname())[2] if not address.startswith('127.') and ':' not in address})
        except OSError:
            addresses=[]
        for address in addresses:
            print(f'LAN game: http://{address}:{port}/',flush=True)
        if not addresses:
            print(f'LAN game: http://<this-computers-local-IP>:{port}/',flush=True)
        print('Both players: Multiplayer > Local network. Exchange invitation and answer codes.',flush=True)
        print('If another device cannot open the page, allow Python on your private network in the system firewall.',flush=True)
        print('No router port forwarding is needed for devices on the same local network.',flush=True)
    if not args.no_browser:webbrowser.open(url)
    try:server.serve_forever()
    except KeyboardInterrupt:pass
    finally:server.server_close()
