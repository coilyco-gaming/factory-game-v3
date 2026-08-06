# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus the active Unity item catalog. Items carry manifest and spawnable-object flags, and scenarios define sources, indexed factories, build sites, hauler capacity, power, starting objects, and occupied-grid layouts.
- `factory_sim` - deterministic inventory, mining, insertion and retrieval, wide indexed factories and sources, typed multi-hauler dispatch, queued deployment, construction and movement, cached occupied-grid A-star routes, transit arbitration, per-object batteries and alerts, spatial power balancing, automatic power lines, multi-generator power, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenarios

The world generalizes to N sources, factories, haulers, and generators:

- each source node has its own extractor: a finite deposit mined into its stockpile at a fixed per-tick speed until it runs dry, or a manifest item (`create_from_nothing`) created each tick, capacity-bounded
- haulers have fixed quantity, weight, and volume limits plus explicit collect, deliver, retrieve, or deploy assignment state, and can be assigned from any position
- recipes accept multiple ingredients or manifest a create-from-nothing item.
  Start counts as the first craft tick, and output blocking preserves inputs
- every factory has reserved input capacity per ingredient and advertises one dispatch intent per under-buffered input, in item-id order
- automated inserters pull each required item at rate five from any distinct
  source or factory container in the eight neighboring cells
- collect and retrieve receivers pull from adjacent source or factory
  containers, advance exactly one phase per tick, and never enter occupied
  building cells
- scenario layouts define grid bounds, object positions, and static obstacles,
  deterministic A-star routing supports cardinal and diagonal movement, allows
  arrival at an occupied target, and reports a sealed route
- haulers queue movement for the world-mutation boundary, competing moves into
  one transit cell resolve in hauler order while building endpoints remain
  shareable
- dispatch arbitration applies programmable priorities before deterministic
  destination, item, and hauler tie-breaks. See
  [dispatch-policy.md](dispatch-policy.md)
- deterministic mine, intent-refresh, assign, collect, deliver, craft-progress, and move steps
- powered scenarios use the node-owned battery and automatic-line model in
  [factory-power.md](factory-power.md), including fueled, fuel-free, and mixed
  generation
- deployment scenarios start with a spawnable drill in factory inventory,
  retrieve it into a capable hauler, carry it to a dormant source, and queue
  source activation for the deterministic world-mutation boundary
- construction scenarios advertise spawnable factory inventory, retrieve it
  into a physically capable hauler, and replace the destination build-site node
  with an occupied structure at the world-mutation boundary
- finite ore and its empty drill queue ordered deletion after the last stockpile
  is hauled, then release the source's occupied grid cell

The twelve layouts and their proof goals are listed in
[factory-scenarios.md](factory-scenarios.md).

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each tick contains topology, object snapshots,
typed dispatch, per-object alert histories, and that tick's global events.

After the last tick the CLI prints one final `{"summary": ...}` line with deterministic run totals: ticks, material flow, dispatch, energy, deployments, world deletions, and idle ticks. The `summary` key keeps tick lines and the summary mechanically separable.

The migration and C# deletion gates are tracked in
[unity-parity.md](unity-parity.md).
