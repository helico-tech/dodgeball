---
status: todo
type: bug
created: 2026-10-16
---
# Rare instant throw right after a pickup

## Problem
`--probe` row Mash/Patient still shows 6-16 "insta" throws per 96 matches
(a wind-up within 10 frames of side 0 picking up the ball, while the bot
waits 30 frames). Down from ~580 per pickup batch before the buffer fix.

## Notes
Likely a human-side CPU teammate holding the ball while control has not
switched yet (control stays during WindUp/Catch), whose AI `plan_throw`
delay can be ~10 frames. Check `update_control` busy rule and give AI
mates of a human a longer minimum hold.
