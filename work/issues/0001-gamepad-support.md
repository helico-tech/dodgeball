---
status: todo
type: idea
created: 2026-10-16
---
# Gamepad support

## Problem
Only keyboard input. macroquad has no gamepad API (ADR 0001), so 2P on one
keyboard is cramped even with the extra `, . /` keys for P1.

## Notes
Add `gilrs` in `main.rs` only and map it onto the existing `Pad` struct;
the library stays input-agnostic.
