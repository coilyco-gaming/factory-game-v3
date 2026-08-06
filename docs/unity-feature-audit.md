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
  fuel delivery, factory-output supply, nearest matching world-object
  resolution, per-factory buffer thresholds, duplicate-target exclusion, and
  occupied-target avoidance.
* **Movement and pathfinding** - working - Rust owns deterministic A-star
  routing, diagonals, occupied targets, queued movement, recalculation,
  no-path state, and transit conflict arbitration.
* **Battery** - working - Rust owns node and hauler batteries, minimum capacity,
  clamped charge, overdraw rejection, capacity-weighted adjacent balancing,
  owner-specific system costs, and starvation.
* **Generation** - parity gap - Rust proves one fuel-burning plant. The final
  Unity tests also prove multiple generators and fuel-free generation.
* **Automatic power lines** - working - Rust owns one-time nearest-target
  selection, greedy eight-neighbor construction, pass-through line cells,
  1,000-capacity line batteries, snapshots, and viewer projection.
* **World mutation** - working - Rust owns scenario creation, spawnable
  factory-output retrieval, build-site deployment, topology replacement,
  spawned-structure occupancy, source activation, queued movement, and every
  retained automatic deletion path.
* **Player controls and UI** - working - Rust owns playback, reset, scenario
  selection, bounded grid navigation, camera following, keyboard and wheel
  zoom, focused-cell inspection, object and stored-resource totals, metrics,
  and activity presentation. Player placement, object editing, and logistics
  programming are explicitly obsolete audit entries because Unity never
  exposes those interactions.
* **Observability** - working - Rust owns structured snapshots, event feeds,
  deterministic metrics, and headless JSONL output.
* **Per-object alerts** - parity gap - the final Unity object model retains ten
  alerts per object, deduplicates repeated messages, and refreshes last-seen
  ticks. Rust currently exposes only a global event feed.

Issue #45 restores the overlooked production contract. Generalized generation
and per-object alerts remain the audited implementation queue. The repository
still retains the exact Unity source evidence in Git history.
