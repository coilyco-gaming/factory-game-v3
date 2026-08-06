# Unity parity audit

Git history retains the Unity reference. The original migration covered the
major systems, and a source-level pass against the final Unity tests now tracks
the remaining narrow contracts in
[unity-feature-audit.md](unity-feature-audit.md).

## Current parity

* **Content and item definitions** - substantial - Rust owns the complete
  active Unity item catalog with typed IDs, weights, volumes, stack sizes,
  recipes, craft timing, output multipliers, finite deposits, manifest
  resources, and spawnable-object flags. Spawnable warehouse construction is
  now proven through the general build-site path.
* **Resource containers** - substantial - Rust owns weight and volume capacity,
  item reservations, exact and partial insert, remove, transfer,
  eight-neighbor automated inserters, and adjacent retrievers over source and
  factory containers.
* **Mining** - substantial - Rust owns deterministic extraction speed, finite
  depletion, manifest creation, capacity limits, and energy-gated mining.
  Mining drills can be retrieved, transported, deployed, activated, drained
  after ore depletion, and automatically torn down.
* **Production** - substantial - Rust owns indexed factories, multi-input
  recipes, input consumption, craft progress, output multipliers, output
  capacity, energy-gated crafting, and multi-building production chains.
  Spawnable factory inventory can now dispatch into world construction.
* **Dispatch and receivers** - substantial - Rust owns typed collect, deliver,
  retrieve, and deploy intents, deterministic multi-hauler arbitration,
  in-flight demand accounting, receiver assignment, coal delivery,
  factory-output supply, one-phase-per-tick lifecycle transitions, adjacent
  transfers, nearest matching provider and deployment-target selection,
  duplicate-target exclusion, scenario-configurable per-factory buffers, and
  occupied-target avoidance.
* **Movement and pathfinding** - substantial - Rust owns scenario-defined grid
  bounds and layouts, occupied static cells, deterministic A-star search,
  cardinal and diagonal movement with corner rules, blocked-target arrival,
  transit cells, no-path reporting, per-tick path recalculation, queued movement
  application, and deterministic transit-cell conflict arbitration. Shared
  building endpoints remain available to fleets.
* **Battery and power** - substantial - Rust owns a finite power grid,
  node and hauler batteries, minimum capacity, capacity clamping,
  capacity-weighted adjacent balancing, owner-specific energy consumption,
  fuel-burning generation, starvation, coal demand, and energy metrics.
  Generators greedily construct battery-backed lines toward the nearest
  disconnected battery, and the viewer projects each line cell.
* **World mutation and deployment** - substantial - Rust owns scenario-starting
  objects, retrieval into hauler cargo, deployment at a target, queued movement,
  source activation, general spawnable-item construction at build sites,
  topology replacement, structure occupancy, depleted-ore deletion, drill
  teardown, and occupancy release at a deterministic tick boundary. These
  cover every retained caller of Unity's generic spawn, move, and delete queue.
* **Player and UI interaction** - substantial - Bevy owns playback, reset,
  scenario selection, bounded grid navigation, camera following, one-through-
  ten keyboard and wheel zoom, focused-cell inspection, object and inventory
  totals, and activity projection. Unity has no player placement, object
  editing, or programmable logistics interaction, so those speculative gaps
  are obsolete rather than migration requirements.
* **Observability** - substantial - snapshots, JSONL, run metrics, events, the
  activity feed, and the Bevy projection cover the active Rust systems.

The completed deletion record lives in
[csharp-decommission.md](csharp-decommission.md).
