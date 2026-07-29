# Unity parity unattended run, part 4

Contract: [Forgejo issue 37](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/37)

### 2026-07-29 - player interaction audit

- Evidence - retained Unity player input is camera-grid movement with WASD or
  arrows, Q/E or wheel zoom from one through ten, reset to map center, and
  automatic status inspection of objects at the focused grid position.
- Evidence - retained UI adds pause/reset, focused-object status, and global
  tick, energy, object, and stored-resource totals. Unity exposes no player
  placement, object-editing, or logistics-programming interaction. Those items
  were speculative audit gaps, not retained features.
- Done - Bevy now owns bounded grid focus, camera following, keyboard and wheel
  zoom, a visible focus cursor, focused-cell inspection, and authoritative
  object and inventory totals. Playback and scenario controls remain intact.
- Next - validate the live viewer and full gate, then land this visible slice.

### 2026-07-29 - player interaction validation

- Evidence - the full gate and all 58 Rust tests pass. Focus movement clamps at
  world bounds, zoom clamps to Unity's one-through-ten range, and focused status
  plus inventory totals derive from the authoritative immutable snapshot.
- Evidence - Trunk applied the new Wasm distribution with camera following,
  gold focus cursor, zoom input, inspection, and aggregate status.
- Next - land, perform the final retained-C# audit, and remove the superseded
  gameplay tree only if every remaining file is covered or obsolete.
