"""Start the actual local/LAN server on ephemeral ports and verify resource routes."""
from pathlib import Path
import os, subprocess, sys, urllib.request

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
