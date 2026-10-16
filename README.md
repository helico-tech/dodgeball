# DODGE OR DIE!

*Radical beach dodgeball, 1992 style.*

A 16-bit style dodgeball game in Rust, in the spirit of California Games,
Ski or Die and Super Dodge Ball. Everything you see and hear is generated
in code: a 256x224 software framebuffer (SNES resolution), pixel-art
sprites built from skeleton poses, and a chiptune synth.

![Title](docs/screenshots/title.png)

| | |
|---|---|
| ![Boardwalk](docs/screenshots/boardwalk.png) | ![Slopes](docs/screenshots/slopes.png) |
| ![Super shot](docs/screenshots/super-shot.png) | ![Coach's wrench](docs/screenshots/wrench.png) |

## Features
- 3 vs 3, best of three rounds, four crews with their own stats.
- Four courts: Malibu Beach, Venice Boardwalk, Powder Peak, Dogtown Skatepark.
- 1P ladder against the CPU (with continues), 2P versus, demo mode.
- The five D's of dodgeball: **dodge, duck, dip, dive and dodge**.
- Fall behind and your coach throws a wrench at you. If you can dodge a
  wrench, you can dodge a ball (and your next throw is a super shot).

## Download
Get the latest build from the [Releases](../../releases) page.

- **macOS** (Apple Silicon and Intel): unzip `DodgeOrDie-macos.zip` and
  open `Dodge or Die.app`. The app is not signed by Apple, so the first
  time right-click it and choose *Open*, or run
  `xattr -dr com.apple.quarantine "Dodge or Die.app"`.
- **Windows** (x86_64): unzip `dodge-or-die-windows-x86_64.zip` and run
  `dodgeball.exe`. Windows SmartScreen may warn because the exe is not
  signed: choose *More info* and then *Run anyway*.
- **Linux** (x86_64): unpack `dodge-or-die-linux-x86_64.tar.gz` and run
  `./dodgeball`. Needs OpenGL, X11 and ALSA (present on most desktops).

## Controls
| | Player 1 | Player 2 |
|---|---|---|
| Move | Arrows | W A S D |
| Throw / catch | Z or `,` | F |
| Jump | X or `.` | G |
| Pass / duck | C or `/` | H |
| Start / pause | Enter | Tab |

- Double-tap left or right to run. Jump while running to dive.
- Throw at the very top of a jump for a flaming **super shot**.
- Press catch just before the ball reaches you. Without the ball, the pass
  key ducks.
- Hold the ball longer than 5 seconds and it goes to the other side.
- Esc goes back to the title (and quits from there). F1 toggles scanlines.

## Build from source
```sh
cargo run --release
```
On Linux you need `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`.
On Windows the release build has no console window, so run the developer
tools below from a debug build (`cargo run -- --sim 100`) to see output.

Developer tools, no window needed:
- `cargo run --release -- --shots <dir>` renders screenshots of every screen.
- `cargo run --release -- --sim 1000` prints CPU-vs-CPU balance stats.
- `cargo run --release -- --probe 6` plays scripted "human" bots against the
  CPU ladder to find exploits.

See [AGENTS.md](AGENTS.md) and [docs/architecture.md](docs/architecture.md)
for how it is built.
