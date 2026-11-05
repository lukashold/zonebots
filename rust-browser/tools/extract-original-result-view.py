"""Prepare the optional classic result art from a local Z installation."""
from pathlib import Path
import argparse
import hashlib
import json
import struct

ASSETS = {
    'background': 'PNG/Background.png',
    'panel': 'PNG/BoxInfo.png',
    'divider': 'PNG/BoxDivide.png',
    'buttons': 'PNG/Buttons.png',
    'medium': 'Fonts/Medium.png',
    'large': 'Fonts/Large.png',
}


def metrics(payload, atlas_size):
    if len(payload) < 18 or payload[:5] != b'AHF\x04\x01':
        raise ValueError('Expected a version 4 AHF font with single-byte characters')
    count = struct.unpack_from('<H', payload, 6)[0]
    space = payload[8]
    pairs, height = struct.unpack_from('<H2xH', payload, 10)
    if not 0 < count <= 256 or not space or not height or len(payload) != 18 + count * 25 + pairs * 8:
        raise ValueError('Incomplete font tables')
    chars = payload[18:18 + count]
    glyphs, kerning = {}, {}
    for i, code in enumerate(chars):
        name = bytes([code]).decode('cp1252', errors='replace')
        sx, sy, sw, sh = struct.unpack_from('<4i', payload, 18 + count + 16 * i)
        dx, advance = struct.unpack_from('<hH', payload, 18 + 17 * count + 4 * i)
        dy, = struct.unpack_from('<i', payload, 18 + 21 * count + 4 * i)
        if min(sx, sy, sw, sh) < 0 or sx + sw > atlas_size[0] or sy + sh > atlas_size[1]:
            raise ValueError(f'Glyph outside atlas: {code}')
        if name != '\ufffd':
            glyphs[name] = [sx, sy, sw, sh, dx, dy, advance]
    for i in range(pairs):
        key, adjust = struct.unpack_from('<Ib', payload, 18 + 25 * count + 8 * i)
        chars = bytes([(key >> 16) & 255, key & 255]).decode('cp1252', errors='replace')
        if '\ufffd' not in chars:
            kerning[chars] = adjust
    return dict(height=height, space=space, glyphs=glyphs, kerning=kerning)


def prepare(game, output):
    from PIL import Image
    textures = {key: (game / relative).read_bytes() for key, relative in ASSETS.items()}
    manifest = {'version': 1, 'textures': {}}
    for key, payload in textures.items():
        with Image.open(game / ASSETS[key]) as image:
            image.verify()
            dims = image.size
        manifest['textures'][key] = {'size': dims, 'sha256': hashlib.sha256(payload).hexdigest()}
        if key in ('medium', 'large'):
            manifest[key] = metrics((game / 'Fonts' / (key.title() + '.fnt')).read_bytes(), dims)
    output.mkdir(parents=True, exist_ok=True)
    for key, data in textures.items():
        (output / f'{key}.png').write_bytes(data)
    (output / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, separators=(',', ':')) + '\n', encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='Local original/result-view directory')
    args = parser.parse_args()
    prepare(args.game, args.output)
