# Unity parity unattended run, part 1

Contract: [Forgejo issue 37](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/37)

### 2026-07-29 - launch

- Goal - land working Rust variants for every retained Unity gameplay feature.
- Evidence - canonical `main` and the task checkout both started at `f0b817a`;
  the worktree was clean and had no divergence.
- Decisions - use issue 37 as the pollable report, keep this append-only journal,
  land validated slices directly to canonical main, and pause between meaningful
  viewer slices long enough for Trunk to finish its Wasm rebuild.
- Non-goals - no art-direction redesign, production deployment, hosted-state
  mutation, reusable asset deletion, or work outside `factory-game-v3`.
- Next - inventory the retained C# feature families and map them to current Rust
  code, tests, viewer state, and decommission gates.

### 2026-07-29 - contract checkpoint

- Done - issue 37 holds the run contract, this journal is tracked, and the
  repository test gate now falls back cleanly when GNU `timeout` is unavailable.
- Evidence - `just test` passed the complete pre-commit suite and all 38
  Rust tests; commit `f2ee8a3` landed on canonical main.
- Decision - use the normal full `just test` surface at every later
  checkpoint instead of maintaining a task-only validation verb.
- Next - classify every retained C# gameplay file by Rust coverage and select
  the next slice from the largest coherent uncovered dependency chain.

### 2026-07-29 - content audit

- Evidence - the active Unity catalog contains fourteen item definitions: four
  resources, six products, and four spawnable buildings.
- Decision - the Rust catalog preserves every active recipe, physical value,
  timing value, output multiplier, manifest flag, and spawn flag. Commented
  Unity definitions stay out of scope.
- Next - validate and land the catalog, then use it as the fixed input for the
  multi-factory production graph.
