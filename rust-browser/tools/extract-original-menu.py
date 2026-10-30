"""Copy locally owned GOG menu textures into the ignored private resource pack.

Original PNG bytes are preserved. The public tool contains asset names and crop
metadata only; it never embeds original pixels or copies the game executable.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os

from PIL import Image


ASSETS = {
    'background': 'PNG/IPBackground.png',
    'title': 'PNG/Splash.png',
    'panel': 'PNG/IPBoxCentreWide.png',
    'panelNarrow': 'PNG/IPBoxCentreNarrow.png',
    'left': 'PNG/IPBoxLeft.png',
    'right': 'PNG/IPBoxRight.png',
    'divider': 'PNG/IPBoxDivide.png',
    'buttons': 'PNG/Buttons.png',
    'menuText': 'de.lproj/frontend.png',
}
LABELS = ('newGame', 'loadGame', 'replay', 'multiplayer', 'highscores')


def alpha_content(image):
    box = image.getchannel('A').getbbox() if image.mode == 'RGBA' else (0, 0, *image.size)
    if box is None:
        raise ValueError('Empty menu texture')
    x0, y0, x1, y1 = box
    return [x0, y0, x1 - x0, y1 - y0]


def text_regions(image):
    """Find the ten normal/highlighted text rows in the original German atlas."""
    alpha = image.getchannel('A')
    runs, first = [], None
    for y in range(alpha.height + 1):
        occupied = y < alpha.height and alpha.crop((0, y, alpha.width, y + 1)).getbbox()
        if occupied and first is None:
            first = y
        elif not occupied and first is not None:
            box = alpha.crop((0, first, alpha.width, y)).getbbox()
            runs.append([box[0], first, box[2] - box[0], y - first])
            first = None
    if len(runs) != len(LABELS) * 2:
        raise ValueError(f'Expected ten original menu text rows, found {len(runs)}')
    return {name: {'asset': 'menuText', 'normal': runs[i * 2], 'hover': runs[i * 2 + 1]}
            for i, name in enumerate(LABELS)}


def extract(game, output):
    # Validate every source before mutating a previously prepared resource pack.
    plans, assets, labels = [], {}, None
    for name, relative in ASSETS.items():
        source = game / relative
        payload = source.read_bytes()
        with Image.open(source) as image:
            image.load()
            width, height = image.size
            content = alpha_content(image)
            if name == 'menuText':
                labels = text_regions(image)
        target = f'menu/{name}.png'
        assets[name] = {
            'path': target, 'source': relative, 'width': width, 'height': height,
            'content': content, 'bytes': len(payload),
            'sha256': hashlib.sha256(payload).hexdigest(),
        }
        plans.append((target, payload))

    manifest = {
        'version': 1,
        'source': 'Original PNG menu textures from the local GOG installation',
        'assets': assets,
        'labels': labels,
        'notes': [
            'Texture padding remains in the original files; content rectangles exclude unused alpha.',
            'The low-resolution Splash composition avoids the publisher overlays in IPSplash.',
            'Browser setup controls and responsive layout are adaptations, not an exact original menu state machine.',
        ],
    }
    (output / 'menu').mkdir(parents=True, exist_ok=True)
    for relative, payload in plans:
        (output / relative).write_bytes(payload)
    pending = output / 'menu.json.tmp'
    pending.write_text(json.dumps(manifest, ensure_ascii=False, separators=(',', ':')), encoding='utf-8')
    pending.replace(output / 'menu.json')
    print(json.dumps({'assets': len(assets), 'bytes': sum(a['bytes'] for a in assets.values()),
                      'manifest': str(output / 'menu.json')}, ensure_ascii=False))
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, default=Path(os.environ.get('Z_GAME_DIR', r'C:\GOG Games\Z')))
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'dist' / 'original')
    args = parser.parse_args()
    extract(args.game.expanduser().resolve(), args.output.expanduser().resolve())
