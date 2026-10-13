# Zonebots

A real-time strategy game for the browser. Two robot armies fight over a map that is split into
sectors. Touch a sector's flag and its factory, workshop and guns are yours; the more sectors you
hold, the faster your factories build. Destroy the enemy fort to win.

The sector-and-flag gameplay is inspired by a mid-90s RTS classic. Zonebots is an independent,
from-scratch implementation: all code, the procedurally drawn graphics, the synthesised sounds and
all levels are original. It contains no code, artwork, audio, maps or other data from any other game.

**Everybody is welcome to contribute – see [CONTRIBUTING.md](CONTRIBUTING.md).**

## Play

No build step, no dependencies. Open `index.html` in any current browser (Windows, macOS, Linux,
tablets), or serve the folder with any static web server:

    python -m http.server 8000

## Features

- campaign with 20 levels in 5 worlds (desert, volcano, arctic, jungle, city)
- skirmish mode on generated maps: pick size, world, tech level and a map number
- three difficulty levels, best times per level
- 6 robot types, 5 vehicles, 3 gun emplacements, tech levels per factory
- abandoned vehicles to crew, workshops that repair, grenade crates, factory rally points
- computer opponent that scales with level and difficulty
- mouse, keyboard and touch controls, minimap, zoom, game speed
- German and English, progress saved in the browser

## Controls

| Input | Action |
| --- | --- |
| Left click / drag box | select units or a building |
| Right click | move, attack, board an abandoned vehicle, assault the fort |
| Right click with a factory selected | set its rally point |
| Ctrl + right click, or F then click | attack-move |
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

    npm test    # headless: generates every level, checks reachability, runs AI vs AI

The code layout, the rules for contributions and a list of open ideas are in
[CONTRIBUTING.md](CONTRIBUTING.md). The repository includes a workflow that publishes the game to
GitHub Pages (enable it once under Settings → Pages → Source: GitHub Actions).

## License

MIT, see [LICENSE](LICENSE).
