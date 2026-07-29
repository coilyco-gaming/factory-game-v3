# Factory power model

Powered scenarios route fuel to a generator as ordinary logistics demand.
Generation charges the plant battery before connected batteries rebalance by
capacity. Sources, factories, haulers, the plant, and power-line cells own
separate clamped batteries. Mining, dispatch, insertion, and production drain
only the battery owned by that system.

A generator performs one deterministic greedy eight-neighbor route toward the
nearest disconnected static battery. Each missing route cell becomes a
pass-through power line with a 1,000-unit battery. The new cells participate in
the same adjacent capacity-weighted balance before powered work advances.

Snapshots expose every battery and generated line cell. The viewer keeps a
derived total-energy gauge and prints local node charge without taking
simulation authority.
