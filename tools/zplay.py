"""Play Zonebots with your own copy of Z, without picking the folder in the browser.

Serves the game and the Z folder on this computer only (127.0.0.1) and opens the browser.
The Z files never leave your computer.

    python tools/zplay.py "C:/GOG Games/Z"

The server stops by itself after 30 minutes without requests.
"""
import functools, http.server, os, sys, threading, time, webbrowser

GAME = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ZDIR = sys.argv[1] if len(sys.argv) > 1 else r'C:\GOG Games\Z'
last = time.time()


class Handler(http.server.SimpleHTTPRequestHandler):
    def translate_path(self, path):
        global last
        last = time.time()
        clean = path.split('?', 1)[0].split('#', 1)[0]
        if clean.startswith('/zdata/'):
            rel = os.path.normpath(clean[len('/zdata/'):]).lstrip('\\/')
            if rel.startswith('..'):
                return os.path.join(GAME, '__nothing__')
            return os.path.join(ZDIR, rel)
        return super().translate_path(path)

    def log_message(self, *args):
        pass


def watchdog():
    while True:
        time.sleep(60)
        if time.time() - last > 1800:
            os._exit(0)


if __name__ == '__main__':
    if not os.path.isfile(os.path.join(ZDIR, 'LEVEL01.MAP')):
        print('No Z installation found in', ZDIR)
        sys.exit(1)
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=GAME))
    url = 'http://127.0.0.1:%d/?z=zdata/' % server.server_address[1]
    threading.Thread(target=watchdog, daemon=True).start()
    webbrowser.open(url)
    print('Zonebots with original Z data:', url)
    server.serve_forever()
