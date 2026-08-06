# Sustained v2-world operation

The complete 50x50 `v2-world` runs beyond starter charge, proving continued
material flow and remote fuel-backed generation.

## Proof window

The release-only test advances two states for 650 ticks. After tick 500 it zeros
every non-generator battery, removing all distributed starter and stored energy:

* a generator-owned extension reconnecting an active deposit
* remote coal extraction, collection, and delivery after the cutoff
* iron-bar, frame, and motor production
* energy generated specifically by `generator-1`

The remote plant must retain coal at tick 650. This connects extraction,
deployment, fuel delivery, generation, balancing, and downstream production.
Every 50 ticks, both replays expose identical metrics and liveness. Generator
maps distinguish the central and remote plants.

## Structural liveness bounds

Every tick checks bounds derived from the scenario rather than copied constants:

* intents, assignments, routes, and claims cannot exceed their publishing
  surfaces, 15-object fleet, grid area, or four-radar roster
* the world-mutation queue must be empty after each tick boundary
* power-line cells cannot exceed the grid area
* starvation is actor-and-window bounded, with none added in the final 50 ticks

The summary omits inventories and events, avoiding snapshot serialization.

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
* loaded unassigned haulers rejoin compatible reachable demand before empty
  carriers collect more freight
* radars release transfer-inaccessible targets so dense deployments cannot pin
  a permanently unreachable claim
* generators extend their connected grid to drained active deposits, with link
  count bounded by generators, sources, and factories
* full or depleted sources do not spend mining power or attract new lines

At its 4,096-entry capacity, the cache evicts the lowest ordered pair. It is
excluded from snapshots, equality, and clones because it is not simulation state.

## Measured result

The deterministic tick-650 summary on the implementation host records 10,788
coal, 10,216 copper ore, 26,508 iron ore, and 3,056 stone mined. It records
16,951 units collected, 14,583 delivered, 186 frames, 220 motors, and two
active generators. `generator-1` burns 1,912 coal and generates 36,159 energy.
Seven links span 61 cells, and starvation stays at zero. Seven haulers remain
assigned, two retain bounded routes of at most 12 cells, and no mutation is
left unapplied.

An early 650-tick 100x100 release probe took 48.81 seconds. The current 50x50
test compares two replays in 1.16 seconds. These point measurements span
different bounds and are neither portable budgets nor a direct benchmark.

Run `ward exec v2-liveness` for the committed proof. The headless runner can
also expose an inspectable final summary without intermediate snapshots:

```bash
ward exec cargo-run -- run --scenario v2-world --ticks 650 --summary-only --exhaust-batteries-at 500
```

The gate proves 150 post-cutoff ticks, not infinite steady state or .NET parity.
