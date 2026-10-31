from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from functools import partial
import mimetypes, argparse, webbrowser, os, subprocess, sys, json, socket
from urllib.parse import unquote,urlsplit
root = Path(__file__).resolve().parents[1] / 'dist' / 'independent'
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('audio/wav', '.wav')
mimetypes.add_type('audio/ogg', '.ogg')
mimetypes.add_type('audio/mpeg', '.mp3')

def ensure_original_pack(game):
    """Create missing private data locally; no original files enter the repository."""
    repo = Path(__file__).resolve().parents[1]
    private = repo / 'dist' / 'original'
    jobs = []
    campaign_path = private / 'campaign.json'
    try:
        campaign_current = json.loads(campaign_path.read_text(encoding='utf-8')).get('version', 0) >= 2
    except (OSError, ValueError, AttributeError, TypeError):
        campaign_current = False
    if not campaign_current:
        jobs.append(('campaign', repo / 'tools' / 'extract-original-campaign.py',
                     ['--game', str(game), '--output', str(private)]))
    if not all((private / name).is_file() for name in ('sprites.png', 'sprites.json', 'tiles.png')):
        jobs.append(('artwork', repo / 'tools' / 'extract-original-art.py',
                     ['--source', str(game), '--out', str(private)]))
    if not (private / 'audio.json').is_file():
        jobs.append(('audio', repo / 'tools' / 'extract-original-audio.py',
                     ['--source', str(game), '--out', str(private)]))
    if not (private / 'menu.json').is_file() and (game / 'PNG' / 'IPBackground.png').is_file():
        jobs.append(('menu', repo / 'tools' / 'extract-original-menu.py',
                     ['--game', str(game), '--output', str(private)]))
    if not (private / 'hud.json').is_file() and (game / 'de.lproj' / 'NSNEWPAN.DAT').is_file():
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
    def translate_path(self,url):
        path=unquote(urlsplit(url).path)
        if path.startswith('/original/'):
            base=root.parent/'original';target=(base/path[len('/original/'):]).resolve()
            return str(target) if target.is_relative_to(base.resolve()) else str(root/'missing')
        return super().translate_path(url)

if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8765);p.add_argument('--no-browser',action='store_true')
    p.add_argument('--lan',action='store_true',help='Allow other devices on your local network to load this game; binds all local interfaces')
    p.add_argument('--game','--zdir',dest='game',type=Path,default=Path(os.environ.get('Z_GAME_DIR',r'C:\GOG Games\Z')),help='Local Z installation for automatic extraction of missing original art/maps')
    args=p.parse_args()
    ensure_original_pack(args.game.expanduser().resolve())
    bind='0.0.0.0' if args.lan else '127.0.0.1'
    server=ThreadingHTTPServer((bind,args.port),partial(Handler,directory=str(root)))
    port=server.server_port
    url=f'http://127.0.0.1:{port}/';print(url,flush=True)
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
