# Factory power model

Powered scenarios start with any number of generators and may add remote coal
plants through [remote-coal-plants.md](remote-coal-plants.md). Fueled generators
advertise ordinary logistics demand, while fuel-free generators need no
inventory or dispatch. Every generator charges its own clamped battery before
connected batteries rebalance by capacity. A full battery consumes no fuel,
and an inactive zero-output generator neither burns nor requests it.

Each static generator performs one deterministic greedy eight-neighbor route
toward the nearest static battery. A deployed generator instead targets the
nearest powered generator, matching the original generator-to-generator
network intent without attaching itself to an arbitrary remote source battery.
Missing route cells become pass-through power lines with 1,000-unit batteries.
When a factory or mineable source reaches zero outside that component, each
generator can add one deterministic nearest-consumer extension per tick.
Existing cells are reused, and a connected target is never linked again. Full
or depleted sources neither spend mining power nor attract a line. Sources,
factories, haulers, generators, and line cells then share adjacent
capacity-weighted balance. Dormant and disconnected charged sources stay
isolated.

Snapshots expose each generator, battery, generated line cell, and every
ordered generator-owned path. The viewer renders their union and labels a
generator's target or aggregate link count without taking simulation authority.

Run metrics distinguish energy generated at plants, energy moved during
capacity-weighted balancing, energy consumed by powered work, and starvation
events. Moved energy counts only units transferred between batteries, not the
total energy present in a component.
