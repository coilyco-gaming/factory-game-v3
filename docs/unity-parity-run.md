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
