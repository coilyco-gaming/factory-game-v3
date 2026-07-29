# Unity parity and decommission gates

The retained C# tree is the Rust migration reference. A component can leave only
when Rust owns its behavior, deterministic tests cover its transitions, and the
viewer exposes player-visible state.

## Current parity

* **Content and item definitions** - substantial - Rust owns the complete
  active Unity item catalog with typed IDs, weights, volumes, stack sizes,
  recipes, craft timing, output multipliers, finite deposits, manifest
  resources, and spawnable-object flags. General object construction remains.
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
  General spawnable output construction remains.
* **Dispatch and receivers** - substantial - Rust owns typed collect, deliver,
  retrieve, and deploy intents, deterministic multi-hauler arbitration,
  in-flight demand accounting, receiver assignment, coal delivery,
  factory-output supply, one-phase-per-tick lifecycle transitions, adjacent
  transfers, and occupied-target avoidance. General world-object-type matching
  and configurable buffer policy remain.
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
  Automatic power-line placement remains.
* **World mutation and deployment** - partial - Rust owns scenario-starting
  objects, retrieval into hauler cargo, deployment at a target, queued movement,
  source activation, depleted-ore deletion, drill teardown, and occupancy
  release at a deterministic tick boundary. General object spawn and deletion
  plus player placement remain.
* **Player and UI interaction** - early - Bevy owns playback controls and
  read-only projection. World editing, selection, placement, and programmable
  logistics controls remain.
* **Observability** - substantial - snapshots, JSONL, run metrics, events, the
  activity feed, and the Bevy projection cover the active Rust systems.

The ordered removal gates and progress links live in
[unity-decommission-plan.md](unity-decommission-plan.md).
