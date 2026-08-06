# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed items and scenarios for sources, factories, radars,
  build sites, fleets, power, starting objects, and grid layouts.
- `factory_sim` - deterministic gameplay, exclusive radar claims, typed fleet
  dispatch, queued drill, structure, and generator mutations, indexed topology,
  cached A-star routes, power, alerts, metrics, and tick stepping.
- `factory_cli` - a headless runner that emits JSON tick snapshots or advances
  through a summary-only path for long-running proofs.

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
- dispatch skips sealed transfer endpoints and unreachable source-destination
  pairs, reserves a hauler's full carrying capacity for in-flight demand, and
  cancels stale empty-source or no-path assignments. Unassigned loaded haulers
  rejoin reachable demand before empty collection
- scenario layouts define grid bounds, object positions, and static obstacles,
  deterministic A-star routing supports cardinal and diagonal movement, allows
  arrival at an occupied target, reports a sealed route, caches a bounded set
  of stable endpoint pairs, and invalidates the cache after topology mutations
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
- deployment radars claim compatible dormant sources before factories and
  haulers supply drills. See [deployment-radar.md](deployment-radar.md)
- coal-plant radar claims use the same transport phases but create a typed
  generator and source occupancy instead of activating a drill. See
  [remote-coal-plants.md](remote-coal-plants.md)
- construction scenarios advertise spawnable factory inventory, retrieve it
  into a physically capable hauler, and replace the destination build-site node
  with an occupied structure at the world-mutation boundary
- finite ore and its empty drill queue ordered deletion after the last stockpile
  is hauled, then release the source's occupied grid cell

The fifteen layouts and their proof goals are listed in
[factory-scenarios.md](factory-scenarios.md).

## Headless runner

The CLI's snapshot and summary-only output contracts are documented in
[headless-runner.md](headless-runner.md). The sustained 50x50 release gate is
documented in [v2-liveness.md](v2-liveness.md).

The migration and C# deletion gates are tracked in
[unity-parity.md](unity-parity.md).
