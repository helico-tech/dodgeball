# 0001. Software framebuffer with macroquad as the window

## Status
accepted

## Context
The game must look like a real early-90s 16-bit game, and reviewers (people
and agents) must be able to judge the look and the gameplay without a
window. GPU drawing with modern scaling and blending makes it easy to break
the pixel grid, and is hard to screenshot headless.

## Decision
Draw every pixel in software into a 256x224 `Frame`. macroquad only uploads
that frame as a nearest-filtered texture, scales it by an integer factor
and plays the generated WAV sounds. The library has no macroquad
dependency in its logic; `--shots` writes PNGs from the same `Frame`.

## Consequences
- Pixel-perfect, era-correct output, identical in the window and in PNGs.
- Headless review and balance simulation are cheap.
- No GPU effects; anything fancy (scanlines aside) must be done per pixel.
- macroquad has no gamepad API, so only keyboard input for now.
