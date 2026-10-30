# Zonebots

A real-time strategy game for the browser. Two robot armies fight over a map that is split into
sectors. Touch a sector's flag and its factory, workshop and guns are yours; the more sectors you
hold, the faster your factories build. Destroy the enemy fort to win.

The rules and controls follow the 1996 RTS classic *Z*. Zonebots is an independent, from-scratch
implementation: all code, the procedurally drawn pixel graphics, the synthesised sounds and its own
levels are original. The repository contains no code, artwork, audio, maps or other data from *Z*.

**Original data mode (optional):** if you own *Z* (for example the GOG release), you can pick its
installation folder in the menu. The browser then reads the original 20 maps, their terrain, forts,
buildings, flags and unit sprites from your copy and you play them with this engine. The files are read and stored locally in your
browser only; nothing is uploaded and nothing from the game is part of this project.

**Everybody is welcome to contribute – see [CONTRIBUTING.md](CONTRIBUTING.md).**

## Play

### Rust / WebAssembly version

The active Rust version is now a new independent implementation:
[`rust-browser/INDEPENDENT.txt`](rust-browser/INDEPENDENT.txt).
Its new engine is in `rust-browser/independent-engine/`; the complete browser entrypoint
is `rust-browser/dist/independent/`. It does not load the previous engine, renderer or
original-game data readers. Run `python3 independent-engine/serve.py` from `rust-browser/`.

The separately researched [behavior specification](rust-browser/independent-engine/SPECIFICATION.txt)
records public original-manual/playguide evidence and distinguishes estimated rules and balance.
Artwork was independently authored in a fresh context. The integrator had prior exposure to the
earlier reverse engineering, so this is not claimed to be a formally audited clean-room process
or a complete behavior-identical recreation.

Validate with `cargo test --manifest-path independent-engine/Cargo.toml` and
`node independent-engine/check-wasm.cjs`. Build with `independent-engine/build.ps1`.

The notes below describe the earlier Rust edition retained in source history.

The new [Rust browser version](rust-browser/LIESMICH.txt) lives in `rust-browser/`.
Its simulation runs in Rust/WebAssembly; JavaScript handles rendering, input and original-data loading.
The compiled Wasm file is included. From `rust-browser/`, run:

    python3 tools/serve.py

On Windows, double-click `rust-browser/Start-Spiel.cmd`. On macOS or Linux, use the
command above with Python 3 installed. To load your own original game files automatically:

    python3 tools/serve.py --zdir "/path/to/Z"

The Rust edition now starts with its own artwork, synthesized audio and 20 standalone
missions: no game-folder selection or original installation is needed. Click **Play now**
or use `?mission=1` through `?mission=20` for a direct mission link. New robots, industrial
buildings, vehicle detailing and terrain textures are drawn procedurally in `art.js` and
`render.js`. See [standalone release notes](rust-browser/STANDALONE.txt).

No original game assets are included. macOS browser operation has not yet been tested. This reconstruction is not
yet a verified 1:1 recreation; see [limitations and test results](rust-browser/TESTERGEBNIS.txt).

### JavaScript version

No build step, no dependencies. Open `index.html` in any current browser (Windows, macOS, Linux,
tablets), or serve the folder with any static web server:

    python -m http.server 8000

## Features

- Z rules: capture sectors by their flags, factories build faster the more you hold, a robot walking
  through the enemy fort's gate wins, losing every robot and crewed vehicle loses
- robot squads, crews for vehicles and guns, snipers that take out crews, destructible buildings,
  bridges and boulders, a crane that repairs and rebuilds, personnel carriers, grenade crates
- classic one-button controls like the original, or modern left/right-click controls
- campaign with 20 own levels in 5 worlds (desert, volcano, arctic, jungle, city)
- optional: the 20 original maps from your own copy of *Z*
- skirmish mode on generated maps: pick size, world, tech level and a map number
- three difficulty levels, best times per level
- 6 robot types, 7 vehicles, 4 gun emplacements, tech levels per factory
- computer opponent that scales with level and difficulty
- mouse, keyboard and touch controls, minimap, zoom, game speed
- own soundtrack (a synthesised rock band), synthesised weapon sounds, and robot voice lines spoken
  by the browser; with your own copy of *Z* the original music, sounds and voices are used instead
- German and English, progress saved in the browser

## Controls

Classic (default, like the original): left click your unit to select it, left click anywhere else to
give an order, right click to deselect. Modern (switch in the menu): left click selects, right click orders.

| Input | Action |
| --- | --- |
| Drag a box | select several units |
| Order on an enemy / empty vehicle or gun / own carrier / enemy fort | attack / crew it / board / storm or shell it |
| Crane order on a damaged building or a destroyed bridge | repair / rebuild |
| F then click | attack order, also on bridges and boulders |
| U | unload a personnel carrier |
| Double click | select all units of that type on screen |
| Ctrl+A | select all units |
| Ctrl+1–9 / 1–9 | assign / recall groups |
| X | stop |
| Arrow keys / WASD, screen edge, minimap | scroll |
| Mouse wheel | zoom |
| Space | jump to your fort |
| P / Esc | pause |
| Touch | tap = select or order, drag = scroll, two fingers = zoom, buttons top left |

## Development

    npm test                              # headless: every level, reachability, AI vs AI
    node tools/ztest.js "C:/GOG Games/Z"  # same for the original maps, needs your own copy of Z

The code layout, the rules for contributions and a list of open ideas are in
[CONTRIBUTING.md](CONTRIBUTING.md). The repository includes a workflow that publishes the game to
GitHub Pages (enable it once under Settings → Pages → Source: GitHub Actions).

## License

MIT, see [LICENSE](LICENSE).
