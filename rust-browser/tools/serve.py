"""Local-only game server. Original assets are read on demand, never copied to dist."""
import argparse,http.server,functools,json,mimetypes,os,sys,webbrowser
from pathlib import Path
from urllib.parse import unquote,urlsplit
ROOT=Path(__file__).resolve().parents[1]/'dist'
mimetypes.add_type('application/wasm','.wasm')
class Handler(http.server.SimpleHTTPRequestHandler):
    def translate_path(self,url):
        path=unquote(urlsplit(url).path)
        base=self.server.zdir if path.startswith('/zdata/') else ROOT
        rel=path[len('/zdata/'):] if path.startswith('/zdata/') else path.lstrip('/')
        target=(base/rel).resolve()
        if not target.is_relative_to(base.resolve()):return str(ROOT/'__not_found__')
        return str(target)
    def end_headers(self):
        self.send_header('Cache-Control','no-cache')
        self.send_header('X-Content-Type-Options','nosniff')
        super().end_headers()
    def log_message(self,fmt,*args):
        if args and str(args[0]).startswith('code 404'): return
        if len(args)>1 and str(args[1])=='404':return
        super().log_message(fmt,*args)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8765);p.add_argument('--zdir',default=r'C:\GOG Games\Z');p.add_argument('--no-browser',action='store_true');args=p.parse_args()
    server=http.server.ThreadingHTTPServer(('127.0.0.1',args.port),functools.partial(Handler,directory=str(ROOT)))
    server.zdir=Path(args.zdir).resolve()
    url='http://127.0.0.1:%d/'%server.server_port
    if (server.zdir/'LEVEL01.MAP').exists():url+='?z=zdata/'
    print(url,flush=True)
    if not args.no_browser:webbrowser.open(url)
    try:server.serve_forever()
    except KeyboardInterrupt:pass
    finally:server.server_close()
