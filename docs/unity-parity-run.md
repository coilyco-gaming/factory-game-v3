# Unity parity unattended run

Contract: [Forgejo issue 37](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/37)

## Journal

Earlier append-only entries live in [part 1](unity-parity-run-1.md).

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

### 2026-07-29 - adjacent retrieval and output dispatch

- Done - receivers now pull from adjacent source or factory containers, remain
  outside occupied building cells, and advance only one collect, deliver,
  retrieve, or deploy phase per tick in Unity order.
- Evidence - the distributed frame line keeps its foundry and frame factory
  nonadjacent. Haulers discover the foundry's output intent, collect iron bars,
  cross the grid, and satisfy the downstream factory. Deterministic tests also
  reject out-of-range collection and delivery.
- Decision - factory outputs become ordinary collect supply when the item is
  not spawnable. Spawnable output continues through retrieve and deploy.
- Next - validate the full gate, wait for the eighth viewer scenario to rebuild,
  and land the slice.

### 2026-07-29 - retrieval validation

- Evidence - the complete pre-commit suite and all 45 Rust tests pass. Trunk
  applied the release Wasm distribution with the freight-line control.
- Decision - three capped documents were split into linked guides and journal
  parts after the repository gate rejected further growth. No journal entry was
  discarded or rewritten.
- Next - land the checkpoint, then port per-object batteries and adjacent
  energy balancing on top of the indexed world.
