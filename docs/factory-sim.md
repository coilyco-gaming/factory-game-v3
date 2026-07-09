# Factory Simulation Workspace

The first Rust migration slice lives in a small workspace with no Bevy dependency.

## Crates

- `factory_content` - typed item and scenario IDs plus the starter `IronOre -> IronBars` data.
- `factory_sim` - deterministic inventory, hauler, factory, and tick stepping.
- `factory_cli` - a headless runner that emits one JSON snapshot per tick.

## Scenario

The starter scenario keeps the loop minimal:

- one source node with iron ore stock
- one hauler with a fixed carry limit
- one factory with reserved input/output capacity
- deterministic collect, deliver, craft-progress, and output steps

The first recipe is intentionally small. It exercises the container and production flow without adding pathfinding or rendering.

## Run

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

The CLI prints JSON lines. Each line contains the tick number, the current source, hauler, and factory snapshots, and the events emitted during that tick.
