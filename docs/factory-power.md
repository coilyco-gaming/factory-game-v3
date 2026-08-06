# Factory power model

Powered scenarios start with any number of generators and may add remote coal
plants through [remote-coal-plants.md](remote-coal-plants.md). Fueled generators
advertise ordinary logistics demand, while fuel-free generators need no
inventory or dispatch. Every generator charges its own clamped battery before
connected batteries rebalance by capacity. A full battery consumes no fuel,
and an inactive zero-output generator neither burns nor requests it.

Each generator performs one deterministic greedy eight-neighbor route toward
the nearest disconnected static battery. Missing route cells become
pass-through power lines with 1,000-unit batteries. Sources, factories,
haulers, generators, and line cells then share the same adjacent
capacity-weighted balance before powered work advances.

Snapshots expose each generator, battery, and generated line cell. The viewer
derives total and local charge without taking simulation authority.
