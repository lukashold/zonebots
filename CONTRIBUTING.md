# Contributing to Zonebots

Everybody is welcome to work on Zonebots: code, levels, balance, translations, bug reports, ideas.
Issues and pull requests may be written in English or German.

## Getting started

There is no build step and there are no dependencies.

1. Fork and clone the repository.
2. Open `index.html` in a browser, or run `python -m http.server 8000` in the folder and open `http://localhost:8000`.
3. Change something, reload the page.
4. Run the headless test before you open a pull request (needs Node.js 18 or newer):

       npm test

   It generates all 20 levels, checks that every flag and both forts are reachable, and lets two
   computer players finish a match on each level. The same test runs on every pull request.

## Where things are

| File | Contents |
| --- | --- |
| `js/data.js` | unit stats, colour themes, level parameters, difficulty, all UI texts (German and English) |
| `js/map.js` | map generator (sectors, roads, rivers, buildings, bridges, boulders) and A* pathfinding |
| `js/zdata.js` | optional loader for the original *Z* data files a player owns (format notes at the top) |
| `js/game.js` | the simulation: orders, combat, production, flags, computer opponent. No DOM access, so it runs in Node |
| `js/render.js` | all drawing. Graphics are canvas code, there are no image files |
| `js/main.js` | browser front end: input, HUD, menus, sound, main loop |
| `tools/simtest.js` | the headless test |
| `tools/ztest.js` | the same checks for the original maps, run against your own copy of *Z* |

Everything hangs off one global object, `ZG`. Scripts are plain `<script>` files so the game also
works when `index.html` is opened straight from disk.

## Rules

- **Original work only.** Do not add code, graphics, sounds, music, maps, names or texts taken from
  commercial games or from any source whose licence does not allow it. This includes "extracted" or
  "converted" files from a game you own. The loader in `js/zdata.js` reads a player's own copy at
  runtime; the data itself must never enter the repository (`.gitignore` blocks the usual file types).
  Draw it, synthesise it or write it yourself, or use material that is clearly licensed for reuse and
  say where it comes from in the pull request.
- **Keep it dependency-free** unless there is a strong reason. No frameworks, no bundler.
- **Keep `js/game.js` free of DOM and browser APIs** so the headless test keeps working.
- **Every user-visible text goes into `ZG.TEXT`** in `js/data.js`, in both German and English.
- **Match the surrounding style:** two spaces, semicolons, single quotes, `const`/`let`.
- **One topic per pull request**, with a short description of what changed and how you tested it.
  Balance changes should say which levels you played.

By contributing you agree that your contribution is released under the MIT licence of this project.

## Ideas that need somebody

- original data mode: tell the robot types apart in sprites.rsc (all robots share one look so far),
  find the personnel carrier, crane and gatling sprites, rocks and crates outside the desert, which
  water was walkable (some flags sit on islands), the starting units and what the radar station does
- radar building and fog of war
- map editor and hand-made levels (the generator in `js/map.js` can already load parameters)
- two-player mode (hot seat, or over WebRTC)
- smarter computer opponent: grouped attacks, defending sectors, collecting crates
- sprite animation, better sound, music
- more languages
- saving and loading a running match

Open an issue first if you plan something large, so nobody does the same work twice.
