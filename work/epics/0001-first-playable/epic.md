---
status: in-progress
created: 2026-10-16
---
# Epic 0001: First playable

## Goal
A complete, fun dodgeball game that looks and sounds like an early-90s
16-bit release (California Games, Ski or Die, Super Dodge Ball).

## Scope
3v3 matches, four crews and courts, 1P ladder with continue, 2P versus,
demo mode, generated chiptune audio, the five D's of dodgeball as moves,
and the coach's wrench when you fall behind.

## Done when
- All screens work in the window and in `--shots`.
- `cargo test` passes; `--sim` shows rounds of about a minute, no stuck
  matches, and team win rates within 40-60%.
- An art review and a gameplay review found no open MUST items.

## Stories
- [01-core-game](01-core-game.md)
- [02-review-fixes](02-review-fixes.md)
