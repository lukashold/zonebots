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

Completed stationary guns wait in up to four factory storage slots. Choose **Place guns** or a factory storage button, then left-click a valid position in that factory's territory. Right-click or Escape cancels placement while preserving the gun. Fort placements snap to free roof sockets; the preview and final placement share the Rust validation. Robot and vehicle production still deploys automatically.

Vehicle workshops now accept an explicit repair order: capture the workshop territory, select a damaged crewed vehicle, then command it onto your workshop (classic left click or modern right click). It approaches, queues if occupied, repairs hidden inside, and exits with a restored hull. The panel shows its progress instead of an empty production menu. Native work, territory/health rates, repair cursor and selected stat restoration are checked against the original; queue/portal/cancellation behavior remains a browser adaptation. Stop, capture and destruction release occupants safely. All 21 original workshops were tested through real combat damage and full repair. [Findings and limits](rust-browser/ORIGINAL-WERKSTATT.txt). Both multiplayer clients must reload after this update.

Gun artwork now keeps each original weapon family across crewing: the previously swapped occupied Cannon/Gatling sprite blocks are corrected using native animation descriptors. Neutral models, weapon values and hardware identity remain consistent through unload/re-entry, including approaches within a single map tile.

Unoccupied hardware now uses the original neutral gray palette in pixel mode and gray team paint in HD. Crewing, unloading and recapturing change only the team paint, without changing the weapon type.

The campaign importer now reconstructs 6,038 destructible wall pieces from the additional wall layer in all 20 original maps. Walls block movement, sight and gun placement; eligible shells, rockets and collected grenades can break them open. Original tile forms and exposed ends update after collapse, and both multiplayer peers receive damage and opened routes. The optional HD view draws its own matching stonework. See [original wall findings and limits](rust-browser/ORIGINAL-WAENDE.txt). Existing local packs with an older campaign format are refreshed automatically by the local launcher. Both multiplayer players must reload the page after this update.

The menu now uses locally extracted original title, background and metal-panel textures, with separate new-game, options and help views. It remains usable without the private menu pack. `tools/extract-original-menu.py` copies the optional textures from an owned GOG installation; the local server prepares them automatically when available. These images are never included in this public repository. The responsive arrangement is a web adaptation of the original artwork.

The Rust edition also offers **Originalgrafik** and **HD-Remake** in the new-game
menu and options. The HD renderer newly draws all 19 unit types, buildings, flags,
crates, effects and five terrain biomes with smooth high-resolution code-native art.
Both modes use the same simulation, native map geometry and hit areas; switching
through options preserves the current battle and the preference is stored locally.
Original clocks, audio and menu/HUD resources still use the separate private pack.
HD terrain, unit and scenery caches have fixed memory limits. The HD code contains
no original game pixels. This optional presentation does not establish new original
behavior parity.

The HD renderer now loads baked, newly authored materials and a generated desert
texture instead of building expensive pixel noise during animation. It draws only
visible objects, amortizes sprite preparation, interpolates movement, and uses
compact Wasm snapshots. HUD and minimap refresh independently from rendering.
See [measured draw times](rust-browser/HD-PERFORMANCE.txt) for the isolated native
Canvas benchmark and its limits; these numbers are not a browser FPS guarantee.

**Two-player multiplayer:** choose **Mehrspieler**, then Internet or local network.
The host gives an invitation code, the guest returns an answer code, and the host
starts the shared match. A real encrypted WebRTC data channel carries validated
commands and authoritative snapshots; the guest never simulates a second game.
The game supports independent graphics preferences, fair normal damage, production,
gun placement, pause and disconnect handling. The private hosted page stays private:
an invitation code does not grant website access. Some internet/router combinations
require a separately configured TURN relay. [Setup and limitations](rust-browser/MULTIPLAYER.txt).

For local network play, run `rust-browser/Start-LAN.cmd` on Windows or
`sh rust-browser/Start-LAN.command` on macOS/Linux, then open the printed LAN address
on both devices. This explicitly serves the host's local resource pack to LAN
players. No game-folder upload is required. The ordinary launcher remains loopback-only.
Two browser clients were tested over the LAN interface on one PC; remote physical
machines and separate internet connections were not independently tested.

Use **Vollbild** in the menu or in battle to request native browser fullscreen. Escape or the same button exits. If embedding permissions prevent fullscreen, the game displays browser instructions. Chrome browser shortcuts: F11 on Windows/Linux, Ctrl+Command+F on macOS. Returning to the main menu pauses the current battle; **Fortsetzen** resumes it while the page remains open.

The in-game sidebar now loads the original steel panel, German unit captions, portraits, health bars and control textures from a separate optional HUD pack. Its native 96×480 panel scales to the browser height; small screens retain the compact interface and minimap. `tools/extract-original-hud.py` prepares the locally owned indexed bitmaps and world palettes. The current communication portrait is static; the original speech/facial state machine remains incomplete.

Movement now uses original terrain masks for infantry, tracked vehicles, Jeep/APC and crane. Vehicles avoid water; infantry can cross supported water types, with deep-water and volcanic restrictions. Road/bridge route prices and relative terrain speeds come from Ghidra-verified tables. A* prefers useful roads, respects diagonal distance and coastal corners, and rechecks movement segments when bridges or terrain change. The original route-candidate planner and full movement scheduler remain partly adapted.

Navigation now explicitly compares a passable direct route with the candidate and reduces redundant tile-center waypoints to arbitrary-angle legs. Legal clicked endpoints remain exact, and unused movement time carries through waypoints. A separate browser travel-time guard prevents slow road detours while preserving useful roads; continuous terrain and building-edge checks protect coasts, walls, bridges and fort gate corridors. Native direct-price and movement-probe findings, plus remaining hierarchy/occupancy/footprint limits, are documented in [ORIGINAL-WEGE.txt](rust-browser/ORIGINAL-WEGE.txt).

Directed attacks now track the live enemy while the old route is still active. Pursuers choose reachable firing positions with range and line of sight, stop to fire and resume when the enemy escapes. Moving target markers stay attached to the same enemy ID. Stop, replacement orders or invalid targets cancel pursuit; strategic AI does not overwrite an active attack merely because its firing path is empty.

Original factories now hold production for 68 native updates after completion and flash `0:00`, then repeat automatically. Grunts, Psychos and Snipers use collected grenades against the original eligible vehicle/cannon classes, with one grenade per committed attack and the verified 50 base body-contact damage path. The existing immediate-hit model remains: the native throw wind-up, trajectory, scatter and blast area are not yet reconstructed. The howitzer also uses its proper original artillery profile.

A systematic audit also fixes stale routes after unloading, moving-APC boarding, crane-only repairs/reconstruction, structure line-of-sight and mounted-gun targeting, paused/focus-lost UI input, stale selections/factory menus, multiplayer connection/pause races, and voice mute/retry handling. Original rubble eligibility was freshly checked in Ghidra. All concrete findings from this review are closed with regression coverage; this does not establish complete native behavior parity. [Audit scope and evidence](rust-browser/FEHLERPRUEFUNG.txt).

From `rust-browser/`, validate with:

    cargo test --manifest-path independent-engine/Cargo.toml
    node tools/check-all.cjs
    node independent-engine/check-wasm.cjs
    node tools/check-original.cjs
    node tools/check-neutral-colors.cjs
    node tools/check-walls.cjs
    node tools/check-wall-target.cjs
    node tools/check-wall-input.cjs
    node tools/check-wall-art.cjs
    node tools/check-wall-multiplayer.cjs
    node tools/check-automatic-production.cjs
    node tools/check-gun-placement-input.cjs
    node tools/check-factory-timer.cjs
    node tools/check-fort-guns.cjs
    node tools/check-jeep-render.cjs
    node tools/check-flag-command.cjs
    node tools/check-capture-hand.cjs
    node tools/check-vehicle-command.cjs
    node tools/check-pickup-command.cjs
    node tools/check-attack-command.cjs
    node tools/check-attack-cursor.cjs
    node tools/check-attack-bounds.cjs
    node tools/check-attack-input.cjs
    node tools/check-crate-bounds.cjs
    node tools/check-pickup-input.cjs
    node tools/check-entry-arrow.cjs
    node tools/check-vehicle-input.cjs
    node tools/check-menu-shell.cjs
    node tools/check-fullscreen.cjs
    node tools/check-original-hud.cjs
    node tools/check-original-grenades.cjs
    node tools/check-original-terrain.cjs
    node tools/check-route-quality.cjs
    node tools/check-workshop-input.cjs
    node tools/check-workshop-multiplayer.cjs
    node tools/check-workshop-art.cjs
    node tools/check-workshop-campaign.cjs
    node tools/check-gun-identity.cjs
    node tools/check-gun-sprites.cjs
    node tools/check-pursuit.cjs
    node tools/check-original-audio.cjs

Combat units show the original crosshair over attack targets. Confirm with the normal command button; **F / Attack** also enables aiming and **Escape** cancels it. A crosshair follows the selected attack target. Manual targets in weapon range now take precedence over opportunistic fire; transporters and cranes are filtered out of mixed attack selections. Empty-ground aiming retains the existing attack-move behavior.

With robots selected, grenade crates show the original hand cursor. Clicking a crate sends only the selected infantry to its pickup point and keeps a hand at the destination until it is collected or the order ends. The existing Rust pickup and audio event determine the actual inventory change.

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
