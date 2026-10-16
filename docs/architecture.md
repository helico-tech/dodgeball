# Architecture

One crate: a library with all game code, and a thin binary (`src/main.rs`)
that owns the window, keyboard and speakers through macroquad.

## Modules
- `gfx` — `Frame`: a 256x224 software framebuffer (SNES resolution).
  Primitives, dithering, polygons, a 5x7 bitmap font and `logo()` big text.
  Colors are snapped to the SNES 15-bit grid by `rgb()`.
- `sprites` — players are built from skeleton poses (joint positions) and
  10x10 head bitmaps, rasterised once into bitmaps of color *roles*, with an
  automatic outline. `teams::palette` maps roles to colors (palette swap).
- `teams` — the four crews, their players, stats and home courts.
- `game` — `Match`: the pure simulation (players, ball, rules, CPU AI).
  Stepped at 60 Hz with two `Pad`s. Emits `Sfx` events; never draws.
- `court` — `Scenery`: static background drawn once per match, plus the
  animated layer (crowd, water, snow, lights).
- `render` — draws a `Match` (depth-sorted sprites, shadows, effects, HUD).
- `app` — scene flow: title, select, versus, play, continue, champion.
- `audio` — chiptune synth: sound effects and songs as WAV bytes.
- `headless` — `--shots`, `--sim` and `--probe` (no window needed).

## Balance targets
Tune with `--sim` and `--probe` after any rule change:
- CPU vs CPU: rounds 45-70 s, time-ups under 12%, supers 10-15% of
  throws, every crew 47-53% (with 1000 matches).
- Probe "Decent" bot across the ladder: about 80-100 / 55 / 30% wins.
- Exploits: super spam under 40% at stage 3, stalling no better than
  playing.

## Key rules
- The game is deterministic given a seed and the pad inputs; nothing in the
  library reads the clock, the keyboard or the window.
- Everything visible goes through `Frame`, so headless screenshots show
  exactly what the window shows.
- World coordinates: x across (centre line at 128), y depth (0 = far line,
  72 = near line), z height. `game::proj` maps them to the screen.
