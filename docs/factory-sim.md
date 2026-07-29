# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus the active Unity item catalog. Items carry manifest and spawnable-object flags, and scenarios define sources, indexed factories, hauler capacity, power, starting objects, and occupied-grid layouts.
- `factory_sim` - deterministic inventory, mining extraction, adjacent automated insertion, indexed factories, typed dispatch with multi-hauler arbitration, queued deployment and movement mutations, occupied-grid A-star pathfinding, transit collision arbitration, fuel-burning power, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenarios

The world generalizes to N sources, N factories, and N haulers:

- each source node has its own extractor: a finite deposit mined into its stockpile at a fixed per-tick speed until it runs dry, or a manifest item (`create_from_nothing`) created each tick, capacity-bounded
- haulers have fixed quantity, weight, and volume limits plus explicit collect, deliver, retrieve, or deploy assignment state, and can be assigned from any position
- recipes may have multiple ingredients: crafting starts only when every input is stocked and consumes all of them
- every factory has reserved input capacity per ingredient and advertises one dispatch intent per under-buffered input, in item-id order
- automated inserters pull each required item at rate five from any distinct
  source or factory container in the eight neighboring cells
- scenario layouts define grid bounds, object positions, and static obstacles,
  deterministic A-star routing supports cardinal and diagonal movement, allows
  arrival at an occupied target, and reports a sealed route
- haulers queue movement for the world-mutation boundary; competing moves into
  one transit cell resolve in hauler order while building endpoints remain
  shareable
- dispatch arbitration is deterministic: factory demand minus in-flight cargo is handed to unassigned empty haulers in index order (collect-phase haulers count at carry limit), so demand is never double-served
- deterministic mine, intent-refresh, assign, collect, deliver, craft-progress, and move steps
- powered scenarios generate a finite shared grid from fuel, route coal to the
  plant as ordinary logistics demand, and charge mining, dispatch, and
  production before those systems can advance
- deployment scenarios start with a spawnable drill in factory inventory,
  retrieve it into a capable hauler, carry it to a dormant source, and queue
  source activation for the deterministic world-mutation boundary
- finite ore and its empty drill queue ordered deletion after the last stockpile
  is hauled, then release the source's occupied grid cell

Starter scenarios cover single and multi-hauler iron bars, two-input building
materials, powered ironworks, mining-drill deployment, an obstacle convoy that
arbitrates a one-cell transit lane, and a five-factory chain that converts iron
and copper ore into mining drills through adjacent inserters.

The first recipe is intentionally small. It exercises the container and production flow without adding pathfinding or rendering.

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each tick line contains the tick number, the current topology, source, hauler, and indexed factory snapshots, the typed dispatch protocol state, and the events emitted during that tick.

After the last tick the CLI prints one final `{"summary": ...}` line with deterministic run totals: ticks, material flow, dispatch, energy, deployments, world deletions, and idle ticks. The `summary` key keeps tick lines and the summary mechanically separable.

The migration and C# deletion gates are tracked in
[unity-parity.md](unity-parity.md).
