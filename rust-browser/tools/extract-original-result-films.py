"""Convert the installed game's separate movie index/video files for local use."""
from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path
import shutil
import struct
import subprocess
from tempfile import TemporaryDirectory


WORLDS = "dvajc"
FILMS = tuple(
    f"e_{world}{suffix}"
    for world in WORLDS
    for suffix in (("lost1", "wint1", "wint2", "wint3") if world == "c"
                   else ("lost1", "wint1", "wint2", "wint3", "winpl"))
)
WIDTH, HEIGHT = 320, 200
PIXELS = WIDTH * HEIGHT
JV_HEADER_SIZE = 104
FRAME_RECORD = struct.Struct("<IIIBBBB")
PACK_HEADER = struct.Struct("<4sHHHHBB")
RGB_TABLE = bytes((index * 255 // 63 if index < 64 else 0) for index in range(256))


def normalize(index: bytes, source: bytes):
    """Reorder the game's split packets for an external JV decoder."""
    if not index.startswith(b"JV00") or len(index) < JV_HEADER_SIZE:
        raise ValueError("Unexpected movie header")
    width, height, count, period = struct.unpack_from("<4H", index, 80)
    if (width, height) != (WIDTH, HEIGHT) or not 1 <= count <= 1200 or not 1 <= period <= 1000:
        raise ValueError("Unexpected movie dimensions or timing")
    if len(index) != JV_HEADER_SIZE + count * FRAME_RECORD.size:
        raise ValueError("Incomplete movie index")
    records, video, palettes = bytearray(), bytearray(), []
    offset = 0
    for frame, record in enumerate(FRAME_RECORD.iter_unpack(index[JV_HEADER_SIZE:])):
        size, audio_size, image_size, palette_flag, audio_type, image_type, _ = record
        palette_size = 768 if palette_flag else 0
        if audio_size or audio_type or palette_size + image_size > size or offset + size > len(source):
            raise ValueError(f"Unsupported frame record {frame}")
        palette = source[offset:offset + palette_size]
        if palette and max(palette) >= 64:
            raise ValueError(f"Invalid palette in frame {frame}")
        pixels = source[offset + palette_size:offset + palette_size + image_size]
        offset += size
        if not pixels or image_type == 0:
            # A sequence of zero commands preserves the preceding indexed image.
            image_type, pixels = (1, bytes(PIXELS // 256)) if frame else (2, b"\0")
        elif image_type == 2:
            if pixels[0] >= 128:
                raise ValueError(f"Unsupported signed fill in frame {frame}")
            pixels = pixels[:1]
        elif image_type != 1:
            raise ValueError(f"Unsupported video type in frame {frame}")
        records += FRAME_RECORD.pack(len(palette) + len(pixels), 0, len(pixels), bool(palette), 0, image_type, 0)
        video += pixels + palette
        palettes.append(palette)
    if offset != len(source):
        raise ValueError("Movie data size does not match its index")
    tail = 0 if index[103] & 64 else (index[103] >> 3) & 7
    return index[:JV_HEADER_SIZE] + records + video, palettes, period, tail, index[103] & 7


def pack_clip(source: Path, out: Path, stem: str, language: str, ffmpeg: str) -> dict:
    base = source / "CUTS" / stem
    assembled, palettes, period, tail, lead = normalize(
        base.with_suffix(".jvi").read_bytes(), base.with_suffix(".jvv").read_bytes()
    )
    audio = source / f"{language}.lproj" / f"{stem}.ogg"
    if not audio.is_file():
        raise FileNotFoundError(audio)
    target = out / f"{stem}.zvf.gz"
    scratch = out / f"{stem}.zvf.gz.part"
    with TemporaryDirectory(prefix="z-result-film-", dir=out) as temp:
        tmp = Path(temp).resolve()
        if not tmp.is_relative_to(out.resolve()):
            raise ValueError("Unexpected temporary directory")
        container, raw = tmp / "movie.jv", tmp / "video.raw"
        container.write_bytes(assembled)
        proc = subprocess.run(
            [ffmpeg, "-hide_banner", "-loglevel", "error", "-i", str(container), "-map", "0:v:0",
             "-pix_fmt", "pal8", "-fps_mode", "passthrough", "-f", "rawvideo", str(raw)],
            capture_output=True,
        )
        expected = len(palettes) * (PIXELS + 1024)
        # Older JV demuxers may report EOF as an I/O error; count every complete plane.
        if not raw.is_file() or raw.stat().st_size != expected:
            raise RuntimeError(proc.stderr.decode(errors="replace")[-1000:])
        with scratch.open("wb") as output, gzip.GzipFile(fileobj=output, mode="wb", mtime=0) as gz, raw.open("rb") as indices:
            gz.write(PACK_HEADER.pack(b"ZVF1", WIDTH, HEIGHT, len(palettes), period, tail, lead))
            previous = 0
            for palette in palettes:
                gz.write(bytes([bool(palette)]))
                if palette:
                    gz.write(palette.translate(RGB_TABLE))
                plane = int.from_bytes(indices.read(PIXELS), "little")
                indices.seek(1024, 1)
                gz.write((plane ^ previous).to_bytes(PIXELS, "little"))
                previous = plane
    scratch.replace(target)
    sound = out / f"{stem}.ogg"
    shutil.copyfile(audio, sound)
    return {"id": stem, "frames": len(palettes), "frameMs": period, "video": target.name, "audio": sound.name,
            "bytes": target.stat().st_size + sound.stat().st_size}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=Path(r"C:\GOG Games\Z"))
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--ffmpeg", default=shutil.which("ffmpeg"))
    parser.add_argument("--language", choices=("de", "en", "es", "fr", "it"), default="de")
    parser.add_argument("--clip", action="append", choices=FILMS)
    args = parser.parse_args()
    if not args.ffmpeg:
        parser.error("FFmpeg is required; set --ffmpeg to its executable path")
    source, out = args.source.resolve(), args.out.resolve()
    if out.is_relative_to(source):
        parser.error("Choose an output folder outside the original installation")
    out.mkdir(parents=True, exist_ok=True)
    entries = [pack_clip(source, out, stem, args.language, args.ffmpeg) for stem in dict.fromkeys(args.clip or FILMS)]
    (out / "films.json").write_text(json.dumps({"version": 1, "films": entries}, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"clips": len(entries), "frames": sum(row["frames"] for row in entries),
                      "bytes": sum(row["bytes"] for row in entries)}))


if __name__ == "__main__":
    main()
