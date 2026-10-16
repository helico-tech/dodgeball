# DODGE OR DIE!

A 16-bit style dodgeball game in Rust, in the spirit of California Games,
Ski or Die and Super Dodge Ball. 3 vs 3, four crews, four courts, 1P ladder
or 2P versus. All art and sound are generated in code.

## Build and run
- `cargo run --release` — play (window, keyboard, sound).
- `cargo test` — smoke tests (matches finish, all screens render).
- `cargo run -- --sim 200` — CPU-vs-CPU balance stats.
- `cargo run -- --shots <dir>` — scripted PNG screenshots (3x) of every
  screen, key moments and a sprite sheet. Use this to review visuals.

## Controls
- P1: arrows, Z (or ,) throw/catch, X (or .) jump, C (or /) pass/duck,
  Enter start/pause.
- P2: WASD, F throw/catch, G jump, H pass/duck, Tab start/pause.
- Double-tap left/right to run. Throw at the very top of a jump = super
  shot. Jump while running = dive. Pass key without the ball = duck.
- Hold the ball over 5 s and it goes to the other side (shot clock).
- Esc: back to title (quits on the title). F1: scanlines on/off.

## Map
- [docs/architecture.md](docs/architecture.md) — modules and key rules.
- [docs/adr/](docs/adr/) — decisions.
- [work/queue.md](work/queue.md) — what to pick up next.
- [work/epics/](work/epics/) and [work/issues/](work/issues/).
