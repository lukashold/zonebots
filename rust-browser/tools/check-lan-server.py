"""Start the actual local/LAN server on ephemeral ports and verify resource routes."""
from pathlib import Path
import os, subprocess, sys, tempfile, urllib.error, urllib.parse, urllib.request

project = Path(__file__).resolve().parents[1]
for lan in (False, True):
    args = [sys.executable, str(project / 'independent-engine' / 'serve.py'), '--no-browser', '--port', '0']
    if lan:
        args.append('--lan')
    process = subprocess.Popen(args, cwd=project, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, encoding='utf-8', env={**os.environ, 'PYTHONIOENCODING': 'utf-8'})
    try:
        url = process.stdout.readline().strip()
        assert url.startswith('http://127.0.0.1:') and not url.endswith(':0/'), url
        with urllib.request.urlopen(url, timeout=5) as response:
            assert response.status == 200
            assert b'id="world"' in response.read()
        with urllib.request.urlopen(url + 'engine.wasm', timeout=5) as response:
            assert response.status == 200
            assert response.headers['Content-Type'] == 'application/wasm'
            assert response.read(4) == b'\x00asm'
        with urllib.request.urlopen(url + 'original/campaign.json', timeout=5) as response:
            assert response.status == 200
        if lan:
            note = process.stdout.readline().strip()
            assert note.startswith('LAN mode:'), note
            address = process.stdout.readline().strip()
            assert address.startswith('LAN game: http://'), address
    finally:
        process.terminate()
        try:
            process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate(timeout=5)
print('Local/LAN server passed: ephemeral listener, game page, Wasm type/header, local original pack route and explicit LAN address output.')

with tempfile.TemporaryDirectory(prefix='z-result-film-server-') as folder:
    pack = Path(folder).resolve()
    assert pack.is_relative_to(Path(tempfile.gettempdir()).resolve())
    manifest = b'{"version":1,"films":[]}'
    (pack / 'films.json').write_bytes(manifest)
    (pack / 'e_dwint1.ogg').write_bytes(b'local test')
    (pack / 'unrelated.txt').write_bytes(b'must never be served')
    args = [sys.executable, str(project / 'independent-engine' / 'serve.py'), '--no-browser',
            '--port', '0', '--films', str(pack)]
    lan = subprocess.run([*args, '--lan'], capture_output=True, timeout=5)
    assert lan.returncode != 0 and b'local loopback game' in lan.stderr
    process = subprocess.Popen(args, cwd=project, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, encoding='utf-8', env={**os.environ, 'PYTHONIOENCODING': 'utf-8'})
    try:
        url = process.stdout.readline().strip()
        assert url.startswith('http://127.0.0.1:') and url.endswith('/?original-films=1'), url
        for name, data in [('films.json', manifest), ('e_dwint1.ogg', b'local test')]:
            with urllib.request.urlopen(urllib.parse.urljoin(url, '/original/films/' + name), timeout=5) as response:
                assert response.read() == data
        for name in ('unrelated.txt', '../unrelated.txt', 'e_dwint1.exe', 'e_dlost1.ogg'):
            try:
                urllib.request.urlopen(urllib.parse.urljoin(url, '/original/films/' + name), timeout=5)
            except urllib.error.HTTPError as error:
                assert error.code == 404
            else:
                raise AssertionError(f'Unexpected accessible movie file: {name}')
    finally:
        process.terminate()
        try:
            process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate(timeout=5)
print('Original films: explicit loopback only, allowlisted files, no access to unrelated assets.')
