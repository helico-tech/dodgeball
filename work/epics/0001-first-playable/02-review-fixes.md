---
status: done
created: 2026-10-16
---
# 02: Review fixes

## Goal
Apply the art-direction and gameplay review findings.

## Acceptance criteria
- Every MUST/BUG item fixed or recorded as an issue with a reason.

## Plan
Collect both reviews, fix by priority, re-render shots, re-run sim and tests.

## Progress log
- 2026-10-16: art review (15 items): applied all MUST/SHOULD items; skipped
  bust portraits, big super ball, skatepark ambient (issue 0002).
- 2026-10-16: gameplay review (15 items, with a scripted human probe):
  applied all. Added `--probe`. Tuned to: rounds 48 s, 0% time-ups,
  supers 15%, crews 48-55% (Punks slightly high, issue 0003), ladder for
  a decent player 100/58/24%.
- 2026-10-16: window run checked (starts, runs, user confirmed audio).
- 2026-10-16: second gameplay pass found 3 bugs I introduced (shot clock
  handed the ball over mid wind-up; pass/floor stall; buffered Z turned a
  pickup into a throw) plus CPU catch timing. Fixed; team shot clock;
  probe got pass-stall and Z-mash rows. CPU now jumps super shots, which
  took super spam at stage 3 from 66% to 34%. Ladder [0.5, 0.68, 0.84]:
  decent bot 99/57/36%. Sim: round 62 s, 6% time-ups, hit 36%, catch 16%.
- 2026-10-16: second art pass: bold I, coaches on benches in tracksuits,
  solid 1P tag, B waist in logo, title footer, calmer ad boards, quote box
  under the HUD, wrench sprite. All applied.
- 2026-10-16: published at github.com/helico-tech/dodgeball with macOS,
  Linux and Windows release builds.

## Follow-ups
