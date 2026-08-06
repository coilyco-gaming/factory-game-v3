# V2-scale factory world

The `v2-world` scenario is the game-scale counterpart to the focused component
fixtures. It reconstructs the final Unity scene's operating dimensions and
startup population in the deterministic Rust simulation.

## Startup contract

* 100x100 bounded grid using seed `4382721`.
* 141 iron, 141 copper, and 141 coal deposits. Generated ore stays outside the
  protected ten-cell central radius and never overlaps another deposit.
* One manifest stone source, for 424 source nodes in total.
* Seven foundries and eight upper-tier factories at the v2 central offsets.
* One coal generator with 4,000 starting fuel and the v2 burn, gain, and grid
  capacity values.
* Fifteen 500-weight, 500-volume haulers at their distinct startup offsets.
* Ten starting mining drills in the drill factory.

Rust uses a stable local generator with the v2 seed, count, range, exclusion,
and collision rules. It does not claim bit-for-bit compatibility with .NET's
historical random-number stream.

## Active proof

The workspace gate advances the full world for 50 deterministic ticks. That
run keeps source IDs unique through `source-423`, deploys three drills, mines
iron, copper, coal, and manifest stone, collects 996 units, delivers 776 units,
and crafts iron bars, frames, and building materials. These fixed assertions
exercise deployment, long-distance pathfinding, freight arbitration, power,
inserters, production, snapshots, and metrics in one populated world.

The CLI test serializes all 424 sources, 15 factories, and 15 haulers in one
snapshot. The browser viewer exposes the same scenario in its control deck.
`O` switches between local detail and a whole-map overview, while shifted grid
movement crosses ten cells per keypress.

## Parity boundary

The source audit remains the proof for individual retained Unity contracts.
This scenario is the separate integration and scale proof. Neither kind of test
is presented as a substitute for the other.

The current game-scale gate does not prove bit-for-bit .NET random coordinates,
separate deployment-radar world objects, autonomous remote coal-plant
expansion, or an indefinitely powered full-map steady state. Drill deployment
uses the Rust dispatcher, and the bounded run starts from the v2 battery
capacities and central fueled generator. Those remaining behaviors require a
longer expansion scenario rather than another component fixture.
