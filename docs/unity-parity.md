# Unity parity audit

Git history retains the Unity reference. Rust covers retained contracts in the
component audit and exercises them together at 100x100 scale. See
[unity-feature-audit.md](unity-feature-audit.md) and [v2-world.md](v2-world.md).

## Current parity

* **Content and item definitions** - substantial - Rust owns the active Unity
  catalog with typed IDs, weights, volumes, stack sizes,
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
  factory-output supply, adjacent transfers, nearest providers, radar-owned
  deployment claims, duplicate and occupied-target exclusion, reachability
  filtering, and stale-assignment cancellation.
* **Movement and pathfinding** - substantial - Rust owns scenario-defined grid
  bounds and layouts, occupied static cells, deterministic A-star search,
  cardinal and diagonal movement with corner rules, blocked-target arrival,
  transit cells, no-path reporting, cached deterministic routes, queued movement
  application, and deterministic transit-cell conflict arbitration. Shared
  building endpoints remain available to fleets.
* **Battery and power** - substantial - Rust owns a finite power grid,
  node and hauler batteries, minimum capacity, capacity clamping,
  capacity-weighted adjacent balancing, owner-specific energy consumption,
  fueled and fuel-free generation, deployed coal plants, multiple generators,
  starvation, fuel demand, and energy metrics. Generators build battery-backed
  lines toward the nearest disconnected battery. Per-generator metrics
  distinguish plants, and the viewer projects each line cell.
* **World mutation and deployment** - substantial - Rust owns scenario-starting
  objects, retrieval into hauler cargo, queued movement, distinct drill,
  structure, and generator deployment, topology and source occupancy,
  depleted-ore deletion, drill teardown, and occupancy release at a
  deterministic tick boundary. These cover every retained caller of Unity's
  generic spawn, move, and delete queue.
* **Player and UI interaction** - substantial - Bevy owns playback, reset,
  scenario selection, bounded and accelerated grid navigation, camera following,
  detail-through-overview zoom, focused-cell inspection, object and inventory
  totals, and activity projection. Unity had no player placement, editing, or
  programmable logistics, so those are not migration gaps.
* **Observability** - substantial - snapshots, snapshot-free stepping, JSONL,
  per-generator metrics, liveness summaries, events, bounded alerts, activity,
  and Bevy cover the active Rust systems.

The component inventory has no retained gap. The large-world gate separately
proves wide identities, startup population, and integrated material flow.
The deletion record lives in [csharp-decommission.md](csharp-decommission.md).
