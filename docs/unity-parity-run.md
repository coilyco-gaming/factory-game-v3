# Unity parity unattended run

Contract: [Forgejo issue 37](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/37)

## Journal

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
- Evidence - `ward exec test` passed the complete pre-commit suite and all 38
  Rust tests; commit `f2ee8a3` landed on canonical main.
- Decision - use the normal full `ward exec test` surface at every later
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

### 2026-07-29 - multi-factory production

- Done - one indexed model now owns every factory in content, topology, world
  state, snapshots, dispatch demand, production stepping, and Bevy projection.
- Evidence - the drill-chain scenario deterministically converts iron and
  copper ore through five factories into mining drills. Tests prove both the
  complete chain and rejection of nonadjacent resource containers.
- Decision - automated inserters use Unity's eight-neighbor search and rate of
  five, with one energy charged per transfer attempt when a power grid exists.
- Correction - active Unity raw resources and bar recipes use the default
  one-tick craft time. Rust values now match that source instead of the earlier
  two-tick demonstration approximation.
- Next - validate the complete gate, let Trunk finish the Wasm rebuild, and land
  the player-visible slice.

### 2026-07-29 - multi-factory validation

- Evidence - the complete pre-commit suite and all 44 Rust tests pass. Trunk
  completed the release Wasm build and applied the new distribution to the
  live local server.
- Decision - this slice is a remote checkpoint because it changes the snapshot
  schema and exposes a new viewer scenario. Later parity work builds only on
  the indexed factory model.
- Next - commit and push the slice to canonical main, then port adjacent
  retrieval and general dispatch target policy.
