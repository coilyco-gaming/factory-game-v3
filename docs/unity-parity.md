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
  item reservations, exact and partial insert, remove, transfer, and
  eight-neighbor automated inserter scheduling. Retriever scheduling remains.
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
  in-flight demand accounting, receiver assignment, coal delivery to a
  separate consumer, and lifecycle transitions. General target-type matching,
  buffer policy, and blocked-adjacency behavior remain.
* **Movement and pathfinding** - substantial - Rust owns scenario-defined grid
  bounds and layouts, occupied static cells, deterministic A-star search,
  cardinal and diagonal movement with corner rules, blocked-target arrival,
  transit cells, no-path reporting, per-tick path recalculation, queued movement
  application, and deterministic transit-cell conflict arbitration. Shared
  building endpoints remain available to fleets.
* **Battery and power** - substantial - Rust owns a finite power grid,
  fuel-burning generation, capacity clamping, shared energy consumption,
  starvation, coal demand, and energy metrics. The grid is an aggregate of the
  Unity adjacent-battery balancing network. Per-object batteries and automatic
  power-line placement remain.
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

## Decommission order

1. The agent expands deferred world mutation from deployed source activation to
   general spawn, move, delete, and teardown operations.
2. The agent ports general spawnable output construction.
3. The agent adds player-facing programming and logistics policy controls.
4. The agent proves every retained C# gameplay component is either covered by a
   Rust test and viewer surface or explicitly retired as obsolete.
5. The agent removes `Assets/Scripts/`, `tests.csproj`, and C#-only plugins in a
   dedicated deletion change after the proof is complete.

Retained visual assets have a separate decision. C# decommission does not
require deleting reusable textures, materials, or other art.

## Decommission progress

The proof and removed first batch live in
[csharp-decommission.md](csharp-decommission.md).
