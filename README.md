# Zonebots

A real-time strategy game for the browser. Two robot armies fight over a map that is split into
sectors. Touch a sector's flag and its factory, workshop and guns are yours; the more sectors you
hold, the faster your factories build. Destroy the enemy fort to win.

The rules and controls follow the 1996 RTS classic *Z*. The JavaScript edition uses
procedural artwork, synthesized sounds and its own levels. The current Rust edition
also uses reverse-engineered rules; original artwork, audio and maps are excluded
from this public repository.

**Original data mode (optional):** if you own *Z* (for example the GOG release), you can pick its
installation folder in the menu. The browser then reads the original 20 maps, their terrain, forts,
buildings, flags and unit sprites from your copy and you play them with this engine. The files are read and stored locally in your
browser only; nothing is uploaded and nothing from the game is part of this project.

**Everybody is welcome to contribute – see [CONTRIBUTING.md](CONTRIBUTING.md).**

## Play

### Rust / WebAssembly version

The current Rust/Wasm edition is a reverse-engineering reconstruction of the user's
GOG release. It uses newly written Rust code with rules checked against Ghidra
decompilation and assembly. It is not a clean-room edition or a verified complete
1:1 recreation. See [provenance](rust-browser/INDEPENDENT.txt) and
[verified behavior and remaining differences](rust-browser/ORIGINAL-ABGLEICH.txt).

The original artwork and campaign resources are **not included in this repository**.
The owner's private hosted edition has a separately authorized resource pack and
requires no game-folder upload. Public-source users can extract their own locally
owned installation:

    cd rust-browser
    python3 independent-engine/serve.py --game "/path/to/Z"

On Windows, `Start-Spiel.cmd` also discovers `C:\GOG Games\Z` automatically.
Artwork extraction requires Python and Pillow. Once created, `dist/original/` stays
local and Git-ignored. The browser runs Rust/WebAssembly and does not execute the
original Windows binary. macOS browsers are supported by this architecture but have
not been tested on a real Mac.

The active engine is `rust-browser/independent-engine/`; rendering and input are in
`rust-browser/dist/independent/`. It now reads the 20 original campaign layouts,
building technology and start groups from the separately extracted pack. Production
work/rates, weapon values and selected damage/crew rules are checked against the
original. The AI, projectile trajectories, animation timing and other behaviors
still have documented differences. Original music, German/English voices and weapon effects now load from the separate private resource pack. On-map factory clocks use the original digit sprites. Idle units seek nearby flags and vacant hardware; explicit commands retain priority.

Owned factories now start a repeating production order immediately, including on capture. Robot factories and forts default to Grunts; choosing another valid product preserves that repeated order. The standard-production button restores the default without leaving the factory idle. Capture preserves existing valid production and its remaining work.

From `rust-browser/`, validate with:

    cargo test --manifest-path independent-engine/Cargo.toml
    node independent-engine/check-wasm.cjs
    node tools/check-original.cjs
    node tools/check-automatic-production.cjs
    node tools/check-factory-timer.cjs
    node tools/check-fort-guns.cjs
    node tools/check-jeep-render.cjs
    node tools/check-flag-command.cjs
    node tools/check-capture-hand.cjs
    node tools/check-vehicle-command.cjs
    node tools/check-entry-arrow.cjs
    node tools/check-vehicle-input.cjs
    node tools/check-original-audio.cjs

With robots selected, vacant vehicles and guns use the original entry-arrow artwork. The command marker remains visible while the selected robots approach; it clears when the target is crewed or the order changes. The arrow artwork and hotspot are checked against the executable; marker centering and animation cadence are browser adaptations.

The original-data, clock and audio checks require the local private resource pack. Build Wasm with
`independent-engine/build.ps1`. Original resources retain their respective owners'
copyright; the repository's code license does not cover that resource pack.

The sections below describe the earlier JavaScript implementation.

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
