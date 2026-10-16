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
- P1: arrows, Z throw/catch, X jump, C pass, Enter start/pause.
- P2: WASD, F throw/catch, G jump, H pass, Tab start/pause.
- Double-tap left/right to run. Throw at the top of a jump = super shot.
- Esc: back to title (quits on the title). F1: scanlines on/off.

## Map
- [docs/architecture.md](docs/architecture.md) — modules and key rules.
- [docs/adr/](docs/adr/) — decisions.
- [work/queue.md](work/queue.md) — what to pick up next.
- [work/epics/](work/epics/) and [work/issues/](work/issues/).
