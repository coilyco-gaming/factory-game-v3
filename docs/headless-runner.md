# Headless simulation runner

`factory_cli` advances any catalog scenario and writes deterministic JSON
lines. A normal run emits one full object snapshot per tick:

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

Each tick includes topology, object state, typed dispatch, bounded per-object
alert histories, and global events. After the last tick, the runner emits one
`{"summary": ..., "liveness": ...}` object. Summary metrics cover ticks,
material flow, dispatch, energy, deployments, deployed generators, world
deletions, and idle ticks.

## Summary-only runs

Long runs can skip the large per-tick snapshots while retaining identical
simulation state:

```bash
ward exec cargo-run -- run --scenario v2-world --ticks 650 --summary-only
```

Summary-only mode emits just the final object. Liveness includes source
lifecycle counts, generators, radar claims, dispatch and fleet occupancy, the
longest queued route, unapplied mutations, and power-line cells. Per-generator
fuel and output maps distinguish the central and remote plants.

An equivalence test advances the same scenario through snapshot and
snapshot-free paths and compares the resulting metrics, liveness, and full
snapshot. The two modes differ only in projection cost.

See [factory-sim.md](factory-sim.md) for gameplay contracts and
[v2-liveness.md](v2-liveness.md) for the 650-tick release proof.
