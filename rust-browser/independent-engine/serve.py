from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from functools import partial
import mimetypes, argparse, webbrowser, os, subprocess, sys
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
    if not (private / 'campaign.json').is_file():
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
    p.add_argument('--game','--zdir',dest='game',type=Path,default=Path(os.environ.get('Z_GAME_DIR',r'C:\GOG Games\Z')),help='Local Z installation for automatic extraction of missing original art/maps')
    args=p.parse_args()
    ensure_original_pack(args.game.expanduser().resolve())
    server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(root)))
    url=f'http://127.0.0.1:{args.port}/';print(url,flush=True)
    if not args.no_browser:webbrowser.open(url)
    try:server.serve_forever()
    except KeyboardInterrupt:pass
    finally:server.server_close()
