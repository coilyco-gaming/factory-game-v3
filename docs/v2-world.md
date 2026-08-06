# V2-scale factory world

The `v2-world` scenario is the game-scale counterpart to the focused component
fixtures. It reconstructs the final Unity scene's operating dimensions and
startup population in the deterministic Rust simulation.

## Startup contract

* 50x50 bounded grid using seed `4382721`.
* 141 iron, 141 copper, and 141 coal deposits. Generated ore stays outside the
  protected ten-cell central radius and never overlaps another deposit.
* One manifest stone source, for 424 source nodes in total.
* Seven foundries and eight upper-tier factories at the v2 central offsets.
* One coal generator with 4,000 starting fuel and the v2 burn, gain, and grid
  capacity values.
* Fifteen 500-weight, 500-volume haulers at their distinct startup offsets.
* Ten starting mining drills in the drill factory.
* Three mining-drill radars plus one coal-plant radar at the historical central
  offsets, targeting iron, copper, and coal deposits independently.

Rust uses a stable local generator with the v2 seed, count, range, exclusion,
and collision rules. It does not claim bit-for-bit compatibility with .NET's
historical random-number stream.

## Active proof

The workspace gate advances the full world for 50 deterministic ticks. That
run keeps source IDs unique through `source-423`, deploys six drills, mines
iron, copper, coal, and manifest stone, collects 1,987 units, delivers 1,481
units, and crafts iron bars, frames, and building materials. These fixed
assertions exercise deployment, long-distance pathfinding, freight arbitration,
power, inserters, production, snapshots, and metrics in one populated world.

The CLI test serializes all 424 sources, 15 factories, 15 haulers, and four
radars in one snapshot. A focused 50x50 lifecycle proof derives the plant
recipe inputs from the catalog, starts with no plant item or remote generator,
and verifies craft, haul, placement, occupancy, identity, inventory, battery,
metrics, events, and viewer state. The browser viewer opens directly into the
same scenario at the full-world extent. `O` switches between that extent and an
approximately 10x10-cell detail view, while shifted grid movement crosses ten
cells per keypress.

## Sustained operation

The release gate also advances two independent full worlds for 650 ticks
without constructing full object snapshots between checkpoints. It observes a
controlled exhaustion of every non-generator battery after tick 500, then
proves that grid reconnection, extraction, collection, delivery, iron bars,
frames, motors, and remote generator output continue. The two replays must
retain identical metrics and liveness state.
Queue and route bounds are derived from the scenario's world, fleet, and grid
sizes. See [v2-liveness.md](v2-liveness.md) for the exact contract and measured
performance.

## Parity boundary

The source audit remains the proof for individual retained Unity contracts.
This scenario is the separate integration and scale proof. Neither kind of test
is presented as a substitute for the other.

The current game-scale gate does not prove bit-for-bit .NET random coordinates
or an indefinite full-map steady state. It proves 150 ticks of continued
operation after a controlled distributed-energy cutoff and sustained remote
fuel-backed power through tick 650. Radar authority owns drill and coal-plant
targeting. See
[deployment-radar.md](deployment-radar.md) and
[remote-coal-plants.md](remote-coal-plants.md).
