# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus starter production data. Items carry manifest and spawnable-object flags, and scenarios define sources, hauler capacity, power, and starting factory objects.
- `factory_sim` - deterministic inventory, mining extraction, typed dispatch protocol with multi-hauler arbitration, queued deployment mutation, a hub-shaped world topology, hauler movement, fuel-burning power grid, factory, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenarios

The world generalizes to N sources and N haulers around one factory:

- each source node has its own extractor: a finite deposit mined into its stockpile at a fixed per-tick speed until it runs dry, or a manifest item (`create_from_nothing`) created each tick, capacity-bounded
- haulers have fixed quantity, weight, and volume limits plus explicit collect, deliver, retrieve, or deploy assignment state, and can be assigned from any position
- recipes may have multiple ingredients: crafting starts only when every input is stocked and consumes all of them
- the factory has reserved input capacity per ingredient and advertises one dispatch intent per under-buffered input, in item-id order
- the topology is hub-shaped: every source and the factory hang off one road node, so any trip is at most two hops and routing needs no search
- dispatch arbitration is deterministic: factory demand minus in-flight cargo is handed to unassigned empty haulers in index order (collect-phase haulers count at carry limit), so demand is never double-served
- deterministic mine, intent-refresh, assign, collect, deliver, craft-progress, and move steps
- powered scenarios generate a finite shared grid from fuel, route coal to the
  plant as ordinary logistics demand, and charge mining, dispatch, and
  production before those systems can advance
- deployment scenarios start with a spawnable drill in factory inventory,
  retrieve it into a capable hauler, carry it to a dormant source, and queue
  source activation for the deterministic world-mutation boundary

Starter scenarios: `iron-bars` (one source, one hauler), `iron-bars-fleet` (one richer source, three haulers competing over a six-unit input buffer - only two are needed per wave), `building-materials` (a finite iron ore source plus a manifest stone source feeding a two-ingredient recipe with two haulers), `powered-ironworks` (iron and coal sources feeding a factory plus a fuel-burning plant over one shared finite grid), and `deployment-demo` (a hauler installs a mining drill before ore extraction can begin).

The first recipe is intentionally small. It exercises the container and production flow without adding pathfinding or rendering.

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each tick line contains the tick number, the current topology, source, hauler, and factory snapshots, the typed dispatch protocol state, and the events emitted during that tick.

After the last tick the CLI prints one final `{"summary": ...}` line with deterministic run totals: ticks, per-item mined and crafted counts, dispatches assigned, units collected and delivered, fuel burned, energy generated and consumed, power starvations, deployments, and idle ticks (ticks that emitted no events). The distinguishing `summary` key keeps tick lines and the summary mechanically separable.

The migration and C# deletion gates are tracked in
[unity-parity.md](unity-parity.md).
