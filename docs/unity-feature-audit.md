# Unity feature audit

This source-level audit maps the final Unity gameplay snapshot at `4d683d6^` to
its Rust replacement. “Working” means deterministic tests and, where visual, a
surface exposed by the Bevy/Wasm viewer. The original completion claim was
reopened when the final Unity tests exposed narrower component contracts.

* **Content catalog** - working - `factory_content` owns all active resources,
  products, spawnable recipes, physical values, timing, multipliers, manifest
  flags, and spawn flags from `FactoryGameContent.cs`.
* **Resource containers** - working - Rust owns capacity, weight, volume,
  reservations, insertion, removal, transfer, eight-neighbor automated
  insertion, and adjacent receiver retrieval from source or factory containers.
* **Mining** - working - Rust owns finite and manifest extraction, deployment
  requirements, power gating, depletion, and source/drill teardown.
* **Production** - working - Rust owns indexed factories, multi-input and
  ingredientless manifest recipes, start-tick progress, output gating, power
  gating, and an end-to-end five-factory mining-drill chain.
* **Dispatch** - working - Rust owns collect, deliver, retrieve, deploy,
  fleet arbitration, in-flight work, one-phase-per-tick receiver transitions,
  fuel delivery, factory-output supply, radar-owned nearest-target discovery,
  persistent claims and release, per-factory buffer thresholds,
  duplicate-target exclusion, and occupied-target avoidance.
* **Movement and pathfinding** - working - Rust owns deterministic A-star
  routing, diagonals, occupied targets, queued movement, recalculation,
  no-path state, and transit conflict arbitration.
* **Battery** - working - Rust owns node and hauler batteries, minimum capacity,
  clamped charge, overdraw rejection, capacity-weighted adjacent balancing,
  owner-specific system costs, and starvation.
* **Generation** - working - Rust owns fueled and fuel-free generators,
  deployed coal plants, per-generator inventory, dispatch, batteries, links,
  and alerts, generation before balancing, overcharge clamping without wasted
  fuel, and combined output from multiple generators.
* **Automatic power lines** - working - Rust owns one-time nearest-target
  selection, greedy eight-neighbor construction, pass-through line cells,
  1,000-capacity line batteries, snapshots, and viewer projection.
* **World mutation** - working - Rust owns scenario creation, spawnable
  factory-output retrieval, distinct drill, structure, and generator
  deployment, topology and source occupancy, queued movement, and every
  retained automatic deletion path.
* **Player controls and UI** - working - Rust owns playback, reset, scenario
  selection, bounded grid navigation, camera following, keyboard and wheel
  zoom, focused-cell inspection, object and stored-resource totals, metrics,
  and activity presentation. Player placement, object editing, and logistics
  programming are explicitly obsolete audit entries because Unity never
  exposes those interactions.
* **Observability** - working - Rust owns structured snapshots, event feeds,
  deterministic metrics, and headless JSONL output.
* **Per-object alerts** - working - typed source, factory, hauler, structure,
  and generator histories retain ten messages per object, refresh repeats by
  exact message, serialize in snapshots, and stay separate from tick events.

Issues #45, #46, and #47 restore the overlooked production, alert, and
generation contracts. A final comparison against every retained Unity
component test found no remaining gameplay contract gap. The exact Unity
source evidence remains in Git history.
