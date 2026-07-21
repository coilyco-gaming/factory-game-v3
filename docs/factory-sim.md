# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus the starter `IronOre -> IronBars` data. Items carry a `create_from_nothing` flag for manifest sources, and scenarios define a source list (item, deposit, mining speed) plus a hauler count.
- `factory_sim` - deterministic inventory, mining extraction, typed dispatch protocol with multi-hauler arbitration, a hub-shaped world topology, hauler movement, factory, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenarios

The world generalizes to N sources and N haulers around one factory:

- each source node has its own extractor: a finite deposit mined into its stockpile at a fixed per-tick speed until it runs dry, or a manifest item (`create_from_nothing`) created each tick, capacity-bounded
- haulers have a fixed carry limit and explicit collect or deliver assignment state, and can be assigned from any position
- the factory has reserved input/output capacity and visible dispatch intent generation
- the topology is hub-shaped: every source and the factory hang off one road node, so any trip is at most two hops and routing needs no search
- dispatch arbitration is deterministic: factory demand minus in-flight cargo is handed to unassigned empty haulers in index order (collect-phase haulers count at carry limit), so demand is never double-served
- deterministic mine, intent-refresh, assign, collect, deliver, craft-progress, and move steps

Starter scenarios: `iron-bars` (one source, one hauler) and `iron-bars-fleet` (one richer source, three haulers competing over a six-unit input buffer - only two are needed per wave).

The first recipe is intentionally small. It exercises the container and production flow without adding pathfinding or rendering.

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each tick line contains the tick number, the current topology, source, hauler, and factory snapshots, the typed dispatch protocol state, and the events emitted during that tick.

After the last tick the CLI prints one final `{"summary": ...}` line with deterministic run totals: ticks, per-item mined and crafted counts, dispatches assigned, units collected and delivered, and idle ticks (ticks that emitted no events). The distinguishing `summary` key keeps tick lines and the summary mechanically separable.
