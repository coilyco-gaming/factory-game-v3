# Unity parity and decommission gates

The retained C# tree is the behavioral reference for the Rust migration. A C#
component can leave the repository only when the Rust simulation owns its
behavior, deterministic tests cover the important state transitions, and the
viewer exposes any state a player previously needed to understand.

## Current parity

* **Content and item definitions** - partial - Rust owns typed item and scenario
  IDs, weights, volumes, stack sizes, recipes, craft timing, output multipliers,
  finite deposits, and manifest resources. Spawnable object definitions and the
  full Unity item catalog remain.
* **Resource containers** - substantial - Rust owns weight and volume capacity,
  item reservations, exact and partial insert, remove, and transfer behavior.
  Inserter and retriever scheduling remain.
* **Mining** - substantial - Rust owns deterministic extraction speed, finite
  depletion, manifest creation, capacity limits, and energy-gated mining.
  Mining-drill deployment and automatic teardown remain.
* **Production** - substantial - Rust owns multi-input recipes, input
  consumption, craft progress, output multipliers, output capacity, and
  energy-gated crafting. Multiple factories and spawnable outputs remain.
* **Dispatch and receivers** - partial - Rust owns typed collect and deliver
  intents, deterministic multi-hauler arbitration, in-flight demand accounting,
  receiver assignment, coal delivery to a separate consumer, and lifecycle
  transitions. Retrieve and deploy verbs, target-type matching, buffer policy,
  and blocked-adjacency behavior remain.
* **Movement and pathfinding** - early - Rust owns deterministic multi-hop
  movement through a hub topology. General grid occupancy, A-star pathfinding,
  path invalidation, and deferred movement queues remain.
* **Battery and power** - substantial - Rust owns a finite power grid,
  fuel-burning generation, capacity clamping, shared energy consumption,
  starvation, coal demand, and energy metrics. The grid is an aggregate of the
  Unity adjacent-battery balancing network. Per-object batteries and automatic
  power-line placement remain.
* **World mutation and deployment** - missing - Rust does not yet own queued
  spawn, move, or delete operations, player placement, retrieval, or deployment.
* **Player and UI interaction** - early - Bevy owns playback controls and
  read-only projection. World editing, selection, placement, and programmable
  logistics controls remain.
* **Observability** - substantial - snapshots, JSONL, run metrics, events, the
  activity feed, and the Bevy projection cover the active Rust systems.

## Decommission order

1. The agent finishes dispatch grammar parity with retrieve and deploy.
2. The agent replaces the hub topology with a general occupied grid and
   deterministic pathfinding.
3. The agent ports deferred world mutation, deployment, and object lifecycle.
4. The agent ports the remaining content catalog and multi-building production
   chains.
5. The agent adds player-facing programming and logistics policy controls.
6. The agent proves every retained C# gameplay component is either covered by a
   Rust test and viewer surface or explicitly retired as obsolete.
7. The agent removes `Assets/Scripts/`, `tests.csproj`, and C#-only plugins in a
   dedicated deletion change after the proof is complete.

Retained visual assets have a separate decision. C# decommission does not
require deleting reusable textures, materials, or other art.
