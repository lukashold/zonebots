from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from functools import partial
import mimetypes, argparse, webbrowser
root = Path(__file__).resolve().parents[1] / 'dist' / 'independent'
mimetypes.add_type('application/wasm', '.wasm')
if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8765);p.add_argument('--no-browser',action='store_true');args=p.parse_args()
    server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(SimpleHTTPRequestHandler,directory=str(root)))
    url=f'http://127.0.0.1:{args.port}/';print(url,flush=True)
    if not args.no_browser:webbrowser.open(url)
    try:server.serve_forever()
    except KeyboardInterrupt:pass
    finally:server.server_close()
