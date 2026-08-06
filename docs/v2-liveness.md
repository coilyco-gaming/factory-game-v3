# Sustained v2-world operation

The gate advances the complete 100x100 `v2-world` beyond starter charge. It
proves continued material flow and remote fuel-backed generation.

## Proof window

The release-only test advances two states for 650 ticks. After tick 500 it zeros
every non-generator battery, removing all distributed starter and stored energy:

* a generator-owned extension reconnecting an active deposit
* remote coal extraction, collection, and delivery after the cutoff
* iron-bar, frame, and motor production
* energy generated specifically by `generator-1`

The remote plant must retain coal at tick 650. This connects extraction,
deployment, fuel delivery, generation, balancing, and downstream production.

Every 50 ticks, both replays must expose identical metrics and liveness.
Per-generator maps distinguish the central and remote plants.

## Structural liveness bounds

Every tick checks bounds derived from the scenario rather than copied constants:

* intents, assignments, routes, and claims cannot exceed their publishing
  surfaces, 15-object fleet, grid area, or four-radar roster
* the world-mutation queue must be empty after each tick boundary
* power-line cells cannot exceed the grid area
* starvation is actor-and-window bounded, with none added in the final 50 ticks

The compact summary omits inventories and events, allowing every intermediate
tick to be inspected without snapshot serialization cost.

## Deadlock prevention

The first run exposed a freight collapse around tick 500. All 15 haulers stayed
assigned without routes, stockpiles filled, and repeated path searches dominated.

The dispatch and topology contracts now prevent that state:

* a bounded cache reuses deterministic A-star results, while topology mutations
  clear cached routes
* sealed transfer endpoints and unreachable source-destination pairs are not
  assigned
* collect and deliver phases reserve one full hauler capacity against demand,
  preventing a fleet from piling onto one nearly full buffer
* an empty source at collection range and any no-path movement cancel the stale
  assignment so the hauler can re-enter arbitration
* generators extend their connected grid to drained active deposits, with link
  count bounded by generators, sources, and factories
* full or depleted sources do not spend mining power or attract new lines

At its 4,096-entry capacity, the cache evicts the lowest ordered pair. It is
excluded from snapshots, equality, and clones because it is not simulation state.

## Measured result

The deterministic tick-650 summary on the implementation host records 8,120
coal, 7,684 copper ore, 10,112 iron ore, and 2,076 stone mined. It records
13,906 units collected, 12,955 delivered, 186 frames, 206 motors, and two
active generators. `generator-1` burns 1,768 coal and generates 35,244 energy.
Three links span 26 cells, and starvation stays at zero. Four haulers remain
assigned, none retains a route, and no mutation is left unapplied.

Before the deadlock fix, one snapshot-free 650-tick release probe took 48.81
seconds. The final release test advances and compares two full replays in 1.65
seconds. The 50-tick release integration test finishes in 0.15 seconds. These
are point-in-time implementation-host measurements, not portable wall-clock
budgets.

Run `ward exec v2-liveness` for the committed proof. The headless runner can
also expose an inspectable final summary without intermediate snapshots:

```bash
ward exec cargo-run -- run --scenario v2-world --ticks 650 --summary-only --exhaust-batteries-at 500
```

The gate proves operation for 150 ticks after a controlled distributed-energy
cutoff. It does not claim infinite steady state, .NET parity, or portable timing.
