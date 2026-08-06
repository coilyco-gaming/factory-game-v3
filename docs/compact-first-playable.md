# Compact First-Playable Loop

The player-facing game starts with a deliberately provisional 16x16 freight
yard. Its size is an experiment, not a permanent map contract.

## Planning contract

- One warehouse/export hub is fixed near the center of the map.
- Four visible iron and copper deposits sit around the perimeter.
- Three mining trucks begin on a small warehouse road apron.
- Roads are free, immediate planning marks. They have no refunds, upkeep,
  construction time, capacity, or congestion behavior.
- Trucks may occupy the same road cell and may never traverse a non-road cell.
- A building consumes one allowance slot and must front an existing road.
- Each placed building is configured for iron bars or copper bars.
- The starting allowance is two buildings. Finished-goods sales unlock further
  slots at increasing cumulative thresholds.

## Running contract

Deposits mine into local stockpiles. Automatic dispatch sends a truck over the
authored road network to collect matching ore, deliver it to a configured
building, collect finished bars, and return them to the warehouse. The market
buys warehouse bars against accumulating demand. Demand increases every market
cycle, and fulfilled demand records both sales and revenue.

The player authors the network and production intent. Dispatch remains
automatic. The focused migration scenarios retain their existing unrestricted
A-star, power, radar, construction, and deployment proofs outside this new
player-loop state.

## Snapshot boundary

`CompactSnapshot` is the immutable presentation boundary. It exposes map
bounds, deposits, authored roads, placed and configured buildings, truck cargo
and routes, warehouse stock, current market demand, cumulative sales, revenue,
building allowance, and events. Bevy reads that snapshot and sends explicit
edit commands back through `CompactGame`.
