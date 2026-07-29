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

### 2026-07-29 - final C# audit and deletion

- Evidence - every remaining component and world-object class maps to a tested
  Rust subsystem. Example fixtures, Unity wrappers, telemetry bootstrap, YAML
  serialization, and never-shipped editing controls are explicitly obsolete.
- Done - the repository removed 1,726 tracked files: the remaining gameplay
  scripts, Unity-only C# dependency plugins, their metadata, and `tests.csproj`.
- Preserved - scenes, textures, materials, art, fonts, shaders, and TextMesh Pro
  resources remain untouched for a separate presentation decision.
- Next - run the full repository and web-build gates, land the decommission,
  verify canonical main, and close the contract issue.

### 2026-07-29 - final validation

- Evidence - no tracked C#, C# project, or DLL remains. The full pre-commit and
  58-test Rust gate passes after all 129,534 deleted lines leave the tree.
- Evidence - the production Trunk/Wasm build succeeds with the exact source
  state and the live server remains on the player-view distribution.
- Next - land canonical main, verify its commit and clean tree, then close issue
  37 with the parity and deletion evidence.

### 2026-07-29 - canonical CI correction

- Evidence - Forgejo's gate failed only because `docs/FEATURES.md` reached 4,099
  characters against the catalog's 4,000-character cap. An exact release-image
  reproduction identified the same hook and no gameplay failure.
- Done - the feature inventory now preserves the shipped boundaries in 2,835
  characters. Diagnostic-only container files were removed before validation.
- Next - rerun the full gate, push the one evidence-backed correction, verify
  remote CI, and close issue 37.
