"""Bake new seamless HD micro-materials, without any original-game pixels.

Requires numpy and Pillow. Runtime renderers only decode these small textures;
the spectral noise and relief never run on the browser's animation thread.
The separately generated desert-ground-v2.png is deliberately left untouched.
"""
from pathlib import Path
import numpy as np
from PIL import Image

OUT = Path(__file__).resolve().parents[1] / 'dist' / 'independent' / 'hd-materials'
PALETTES = [
    ['#a38a63', '#777365', '#416e7b', '#74766b'],
    ['#62564b', '#42484a', '#a44227', '#4f5555'],
    ['#bbcdd0', '#80999f', '#315c79', '#779096'],
    ['#78805b', '#736c54', '#315e63', '#667264'],
    ['#879083', '#49575c', '#3e707c', '#717d77'],
]


def field(rng, size, smooth):
    white = rng.normal(0, 1, (size, size))
    f = np.fft.fftfreq(size)
    kx, ky = np.meshgrid(f, f)
    spectrum = np.exp(-(kx*kx+ky*ky) * smooth * smooth)
    result = np.fft.ifft2(np.fft.fft2(white) * spectrum).real
    return (result-result.mean()) / max(.00001, result.std())


def bake(world, kind, color):
    size = 256
    rng = np.random.default_rng(71923+world*137+kind*931)
    broad, medium, fine = (field(rng, size, scale) for scale in [52, 14, 3])
    height = broad*.35+medium*.35+fine*.11
    dx, dy = (np.roll(height, 1, axis=axis)-np.roll(height, -1, axis=axis) for axis in [1, 0])
    shade = 1+broad*.045+medium*.035+fine*.05+(dx*.75+dy*.55)*(.35 if kind==0 else .07)
    if kind == 1:
        shade = 1+broad*.035+medium*.025+fine*.11
        # Exposed aggregates, worn gravel, small darker tar pits.
        shade += np.maximum(0, fine-1.1)*.09-np.maximum(0, -fine-1.4)*.08
    elif kind == 2:
        y, x = np.indices((size, size))
        shade = 1+broad*.05+np.sin(x*.15+y*.08+medium*.7)*.018+fine*.012
    elif kind == 3:
        # Eroded stone relief with fractured mineral veins, lit from upper left.
        ridges = np.abs(medium*.8+broad*.5)
        height = height + np.clip(ridges, 0, 1)*.26
        dx, dy = (np.roll(height, 1, axis=axis)-np.roll(height, -1, axis=axis) for axis in [1, 0])
        shade = 1+broad*.1+fine*.055+(dx*.7+dy*.7)*.95-np.exp(-ridges*18)*.19
    rgb = np.array([int(color[i:i+2],16) for i in [1,3,5]], dtype=float)
    image = np.clip(rgb[None,None,:]*shade[:,:,None],0,255).astype(np.uint8)
    Image.fromarray(image).save(OUT/f'{world}-{["ground","road","water","rock"][kind]}.webp', quality=88, method=6)


if __name__ == '__main__':
    OUT.mkdir(exist_ok=True)
    for world, colors in enumerate(PALETTES):
        for kind, color in enumerate(colors):
            bake(world, kind, color)
    print(f'Baked20new seamless materials ({sum(p.stat().st_size for p in OUT.glob("[0-4]-*.webp")):,}bytes) to {OUT}')
