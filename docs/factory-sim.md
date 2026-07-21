# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus the starter `IronOre -> IronBars` data. Items carry a `create_from_nothing` flag for manifest sources, and scenarios define a deposit size and mining speed.
- `factory_sim` - deterministic inventory, mining extraction, typed dispatch protocol, small world topology, route-based hauler movement, factory, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenario

The starter scenario keeps the loop minimal:

- one source node with a finite iron ore deposit, mined into its stockpile at a fixed per-tick speed until the deposit runs dry (items flagged `create_from_nothing` instead manifest each tick, capacity-bounded)
- one hauler with a fixed carry limit and explicit collect or deliver assignment state
- one factory with reserved input/output capacity and visible dispatch intent generation
- a three-node route with a road/intermediate node between source and factory
- deterministic mine, collect, deliver, craft-progress, route-step, and output steps

The first recipe is intentionally small. It exercises the container and production flow without adding pathfinding or rendering.

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each line contains the tick number, the current topology, source, hauler, and factory snapshots, the typed dispatch protocol state, and the events emitted during that tick.
