# Remote coal-plant construction

The v2 world can produce, haul, and place a coal plant at a remote coal
deposit. The shipped scenario starts with only its central plant and no plant
item in factory inventory. The remote plant must come through the retained
four-input recipe and the normal dispatch lifecycle.

## Distinct deployment contract

`radar-3` targets coal with `coal_plant`. Its claim shares the ordered exclusion
set with the coal mining-drill radar, so the two object types cannot select the
same deposit. A factory with a finished plant advertises retrieval to that
claim. A capable hauler retrieves one item, transports it, and queues a
generator mutation at transfer range.

The mutation validates the item, coal target, depletion state, and occupancy
before changing the world. It does not use the drill activation mutation. The
coal source keeps `deployed: false` and records its new `occupied_by` generator
identity instead.

## Generator contract

The first remote object is `generator-1`, preserving the existing central
`generator-0` and using the wide generator identity space. It shares the
claimed source position and exposes `coal_plant` as its object item.

The plant starts with empty coal inventory, a 4,000-unit coal reservation, and
an empty 10,000-unit battery. That battery replaces the dormant source battery
at the occupied site. Its generator contract burns four coal for 160 energy. On
later ticks it advertises normal coal delivery, builds an automatic power line
to the powered central generator component, and participates in spatial battery
balancing. It does not attach to a merely closer source or factory battery.

Snapshots expose the typed generator, inventory, battery, topology position,
occupied source, intended link target, and complete ordered line path. Metrics
count both the general deployment and the typed generator deployment, then
separate generated, balanced, consumed, and starved energy. Fuel burn and
generation are also attributed to each generator, so a sustained run can prove
the remote plant itself remains active. Events cover assignment, retrieval,
queued placement, world creation, claim release, power linking, and generation.
The viewer labels the generator type, link target, path length, and occupied
coal site in detail and overview modes.

## 100x100 proof and performance

The focused lifecycle test keeps the 100x100 topology, all 424 sources, four
radars, 15 factories, and 15 haulers. It derives the exact coal-plant inputs
from the catalog and suppresses unrelated factory demand, but never seeds a
plant item or remote generator. This keeps routine validation focused while
still exercising the real 40-tick recipe, inventory, fleet, pathfinding, and
mutation paths.

The focused simulation proof now continues through remote coal delivery, fuel
burn, generation, battery charge, and grid balancing while retaining the full
100x100 world. It and the viewer proof each finish in about 0.4 to 0.5 seconds
in a warm debug build on the implementation host, while the focused simulation
proof takes about 0.05 seconds in release mode. After deterministic route
caching and sealed-endpoint dispatch filtering, the full unseeded 50-tick v2
release test finishes in 0.15 seconds. The 650-tick post-starter-power proof is
covered in [v2-liveness.md](v2-liveness.md). These are point-in-time checks
rather than portable wall-clock budgets.
