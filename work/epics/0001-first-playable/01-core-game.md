---
status: done
created: 2026-10-16
---
# 01: Core game

## Goal
Engine, art, audio, rules, AI and scene flow.

## Acceptance criteria
- Window build plays with keyboard and sound; headless shots and sim work.

## Plan
Software framebuffer (ADR 0001), skeleton sprites, procedural courts,
`Match` simulation, `App` scenes, synth audio.

## Progress log
- 2026-10-16: built all modules; sim: rounds ~63 s, hit rate 39%.
- 2026-10-16: user asked for the five D's and the coach's wrench. Added
  Duck (pass key without the ball) and Dive (jump while running) moves,
  dodge call-outs named after the D's, a round-1 chant, a title ticker,
  and the coach who throws a wrench when a human crew is behind
  (dodge it = next throw is a super shot; a hit costs no heart).

## Follow-ups
- See 02-review-fixes.
